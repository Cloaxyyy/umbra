use reqwest::{Client, Response, StatusCode, header::COOKIE};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::HashMap;

fn client() -> Client {
    Client::builder()
        .user_agent("Umbra/0.1")
        .build()
        .expect("http client")
}

fn with_cookie(rb: reqwest::RequestBuilder, cookie: Option<&str>) -> reqwest::RequestBuilder {
    match cookie {
        Some(c) => rb.header(COOKIE, format!(".ROBLOSECURITY={c}")),
        None => rb,
    }
}

pub(crate) async fn get_json(url: &str, cookie: Option<&str>) -> Result<Value, String> {
    let r = with_cookie(client().get(url), cookie)
        .send()
        .await
        .map_err(|e| format!("Could not reach Roblox: {e}"))?;
    if !r.status().is_success() {
        return Err(expired_or(&r, "Loading from Roblox"));
    }
    r.json().await.map_err(|e| e.to_string())
}

pub(crate) async fn post(url: &str, cookie: Option<&str>, body: Value) -> Result<Response, String> {
    let mut token: Option<String> = None;
    for _ in 0..2 {
        let mut rb = with_cookie(client().post(url), cookie)
            .header("Referer", "https://www.roblox.com/")
            .header("Origin", "https://www.roblox.com")
            .json(&body);
        if let Some(t) = &token {
            rb = rb.header("x-csrf-token", t);
        }
        let r = rb
            .send()
            .await
            .map_err(|e| format!("Could not reach Roblox: {e}"))?;
        if r.status() == StatusCode::FORBIDDEN && token.is_none() {
            if let Some(t) = r
                .headers()
                .get("x-csrf-token")
                .and_then(|v| v.to_str().ok())
            {
                token = Some(t.to_string());
                continue;
            }
        }
        return Ok(r);
    }
    Err("Roblox rejected the request.".into())
}

fn expired_or(r: &Response, what: &str) -> String {
    if r.status() == StatusCode::UNAUTHORIZED {
        "This account's login expired. Remove it and add it again.".into()
    } else {
        format!("{what} failed ({})", r.status())
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct User {
    pub id: u64,
    pub name: String,
    pub display_name: String,
}

pub async fn authenticated_user(cookie: &str) -> Result<User, String> {
    let r = with_cookie(
        client().get("https://users.roblox.com/v1/users/authenticated"),
        Some(cookie),
    )
    .send()
    .await
    .map_err(|e| format!("Could not reach Roblox: {e}"))?;
    if !r.status().is_success() {
        return Err(expired_or(&r, "Checking the account"));
    }
    r.json().await.map_err(|e| e.to_string())
}

pub async fn auth_ticket(cookie: &str) -> Result<String, String> {
    let r = post(
        "https://auth.roblox.com/v1/authentication-ticket",
        Some(cookie),
        json!({}),
    )
    .await?;
    if !r.status().is_success() {
        return Err(expired_or(&r, "Getting a launch ticket"));
    }
    r.headers()
        .get("rbx-authentication-ticket")
        .and_then(|v| v.to_str().ok())
        .map(str::to_string)
        .ok_or_else(|| "Roblox did not return a launch ticket.".into())
}

pub async fn user_id_by_name(username: &str) -> Result<u64, String> {
    let r = post(
        "https://users.roblox.com/v1/usernames/users",
        None,
        json!({ "usernames": [username], "excludeBannedUsers": true }),
    )
    .await?;
    let v: Value = r.json().await.map_err(|e| e.to_string())?;
    v["data"][0]["id"]
        .as_u64()
        .ok_or_else(|| format!("No Roblox user named \"{username}\"."))
}

#[derive(Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Presence {
    pub user_id: u64,

    pub user_presence_type: u8,
    #[serde(default)]
    pub last_location: Option<String>,
    #[serde(default)]
    pub place_id: Option<u64>,
    #[serde(default)]
    pub game_id: Option<String>,
    #[serde(default)]
    pub universe_id: Option<u64>,
}

pub async fn presence(cookie: &str, ids: &[u64]) -> Result<Vec<Presence>, String> {
    let r = post(
        "https://presence.roblox.com/v1/presence/users",
        Some(cookie),
        json!({ "userIds": ids }),
    )
    .await?;
    if !r.status().is_success() {
        return Err(expired_or(&r, "Checking who's online"));
    }
    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct Resp {
        user_presences: Vec<Presence>,
    }
    Ok(r.json::<Resp>()
        .await
        .map_err(|e| e.to_string())?
        .user_presences)
}

pub async fn headshots(ids: &[u64]) -> HashMap<u64, String> {
    let list = ids.iter().map(u64::to_string).collect::<Vec<_>>().join(",");
    let url = format!(
        "https://thumbnails.roblox.com/v1/users/avatar-headshot?userIds={list}&size=150x150&format=Png&isCircular=false"
    );
    let Ok(r) = client().get(url).send().await else {
        return HashMap::new();
    };
    let Ok(v) = r.json::<Value>().await else {
        return HashMap::new();
    };
    v["data"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|d| Some((d["targetId"].as_u64()?, d["imageUrl"].as_str()?.to_string())))
        .collect()
}

#[derive(Deserialize, Serialize, Clone, Debug, PartialEq)]
#[serde(
    tag = "kind",
    rename_all = "lowercase",
    rename_all_fields = "camelCase"
)]
pub enum Target {
    App,

    Place {
        place: String,
        job_id: Option<String>,
    },

    Follow {
        username: String,
    },
}

pub fn parse_place(input: &str) -> Option<u64> {
    let s = input.trim();
    if let Ok(n) = s.parse() {
        return Some(n);
    }
    let rest = &s[s.find("/games/")? + 7..];
    rest.split(|c: char| !c.is_ascii_digit())
        .next()?
        .parse()
        .ok()
}

pub fn link_code(input: &str) -> Option<String> {
    let i = input.find("privateServerLinkCode=")? + "privateServerLinkCode=".len();
    let code: String = input[i..]
        .chars()
        .take_while(|c| c.is_ascii_alphanumeric() || *c == '-' || *c == '_')
        .collect();
    (!code.is_empty() && code.len() <= 64).then_some(code)
}

fn valid_job_id(s: &str) -> bool {
    !s.is_empty() && s.len() <= 64 && s.chars().all(|c| c.is_ascii_hexdigit() || c == '-')
}

pub async fn placelauncher(target: &Target, tracker: u64) -> Result<Option<String>, String> {
    let base = "https://www.roblox.com/Game/PlaceLauncher.ashx";
    Ok(match target {
        Target::App => None,
        Target::Place { place, job_id } => {
            let pid = parse_place(place).ok_or("Enter a place ID or a roblox.com/games link.")?;
            if let Some(code) = link_code(place) {
                return Ok(Some(format!(
                    "{base}?request=RequestPrivateGame&browserTrackerId={tracker}&placeId={pid}&accessCode=&linkCode={code}"
                )));
            }
            match job_id.as_deref().map(str::trim).filter(|j| !j.is_empty()) {
                Some(j) if !valid_job_id(j) => {
                    return Err("That server ID doesn't look right.".into());
                }
                Some(j) => Some(format!(
                    "{base}?request=RequestGameJob&browserTrackerId={tracker}&placeId={pid}&gameId={j}&isPlayTogetherGame=false"
                )),
                None => Some(format!(
                    "{base}?request=RequestGame&browserTrackerId={tracker}&placeId={pid}&isPlayTogetherGame=false"
                )),
            }
        }
        Target::Follow { username } => {
            let uid = user_id_by_name(username.trim()).await?;
            Some(format!(
                "{base}?request=RequestFollowUser&browserTrackerId={tracker}&userId={uid}"
            ))
        }
    })
}

pub fn launch_uri(ticket: &str, placelauncher: Option<&str>, now_ms: u128, tracker: u64) -> String {
    let mode = if placelauncher.is_some() {
        "play"
    } else {
        "app"
    };
    let mut uri =
        format!("roblox-player:1+launchmode:{mode}+gameinfo:{ticket}+launchtime:{now_ms}");
    if let Some(pl) = placelauncher {
        uri += &format!("+placelauncherurl:{}", urlencoding::encode(pl));
    }
    uri += &format!(
        "+browsertrackerid:{tracker}+robloxLocale:en_us+gameLocale:en_us+channel:+LaunchExp:InApp"
    );
    uri
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_places() {
        assert_eq!(parse_place(" 920587237 "), Some(920587237));
        assert_eq!(
            parse_place("https://www.roblox.com/games/920587237/Adopt-Me"),
            Some(920587237)
        );
        assert_eq!(
            parse_place("https://www.roblox.com/games/606849621?privateServerLinkCode=x"),
            Some(606849621)
        );
        assert_eq!(parse_place("adopt me"), None);
        assert_eq!(
            link_code("https://www.roblox.com/games/606849621/Jailbreak?privateServerLinkCode=48727401&x=1").as_deref(),
            Some("48727401")
        );
        assert_eq!(
            link_code("https://www.roblox.com/games/1?privateServerLinkCode=a+b"),
            Some("a".into())
        );
        assert_eq!(link_code("https://www.roblox.com/games/1"), None);
    }

    #[test]
    fn rejects_bad_job_ids() {
        assert!(valid_job_id("4c8f6e1a-2b3d-4e5f-9a0b-1c2d3e4f5a6b"));
        assert!(!valid_job_id("abc+launchmode:edit"));
        assert!(!valid_job_id(""));
    }

    #[test]
    fn builds_uris() {
        let app = launch_uri("T", None, 5, 7);
        assert!(app.starts_with(
            "roblox-player:1+launchmode:app+gameinfo:T+launchtime:5+browsertrackerid:7"
        ));
        let play = launch_uri("T", Some("https://x/?a=1&b=2"), 5, 7);
        assert!(play.contains("+launchmode:play+"));
        assert!(play.contains("+placelauncherurl:https%3A%2F%2Fx%2F%3Fa%3D1%26b%3D2+"));
    }

    #[test]
    fn target_json_shape() {
        let t: Target =
            serde_json::from_str(r#"{"kind":"place","place":"1","jobId":"ab-12"}"#).unwrap();
        assert_eq!(
            t,
            Target::Place {
                place: "1".into(),
                job_id: Some("ab-12".into())
            }
        );
        let t: Target = serde_json::from_str(r#"{"kind":"app"}"#).unwrap();
        assert_eq!(t, Target::App);
    }
}
