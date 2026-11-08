use super::catalog::thumbs;
use super::roblox::{get_json, post};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};

pub async fn rules() -> Result<Value, String> {
    get_json("https://avatar.roblox.com/v1/avatar-rules", None).await
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Worn {
    pub id: u64,
    pub name: String,
    pub asset_type: u64,
    pub asset_type_name: String,
    pub thumb: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Avatar {
    pub avatar_type: String,
    pub body_colors: Map<String, Value>,
    pub scales: Map<String, Value>,
    pub assets: Vec<Worn>,
}

async fn post_ok(url: &str, cookie: &str, body: Value, what: &str) -> Result<Value, String> {
    let r = post(url, Some(cookie), body).await?;
    let status = r.status();
    let v: Value = r.json().await.unwrap_or(Value::Null);
    if status == 401 {
        return Err("This account's login expired. Remove it and add it again.".into());
    }
    if !status.is_success() {
        let msg = v["errors"][0]["message"]
            .as_str()
            .filter(|m| !m.is_empty())
            .unwrap_or("Roblox said no");
        return Err(format!("{what} failed: {msg}"));
    }
    Ok(v)
}

pub async fn current(cookie: &str) -> Result<Avatar, String> {
    let v = get_json("https://avatar.roblox.com/v1/avatar", Some(cookie)).await?;
    let mut assets: Vec<Worn> = v["assets"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|a| {
            Some(Worn {
                id: a["id"].as_u64()?,
                name: a["name"].as_str().unwrap_or_default().to_string(),
                asset_type: a["assetType"]["id"].as_u64().unwrap_or(0),
                asset_type_name: a["assetType"]["name"]
                    .as_str()
                    .unwrap_or_default()
                    .to_string(),
                thumb: None,
            })
        })
        .collect();
    let t = thumbs(
        &assets
            .iter()
            .map(|a| ("Asset".to_string(), a.id))
            .collect::<Vec<_>>(),
    )
    .await;
    for a in &mut assets {
        a.thumb = t.get(&("Asset".to_string(), a.id)).cloned();
    }
    Ok(Avatar {
        avatar_type: v["playerAvatarType"].as_str().unwrap_or("R15").to_string(),
        body_colors: v["bodyColors"].as_object().cloned().unwrap_or_default(),
        scales: v["scales"].as_object().cloned().unwrap_or_default(),
        assets,
    })
}

pub async fn render(user_id: u64) -> Result<String, String> {
    for _ in 0..12 {
        let v = get_json(&format!("https://thumbnails.roblox.com/v1/users/avatar?userIds={user_id}&size=720x720&format=Png&isCircular=false"), None).await?;
        let d = &v["data"][0];
        if d["state"] == "Completed" {
            if let Some(u) = d["imageUrl"].as_str() {
                return Ok(u.to_string());
            }
        }
        if d["state"] == "Blocked" {
            return Err("Roblox won't show this avatar's picture.".into());
        }
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    }
    Err("Roblox is still drawing the avatar. Try Refresh in a moment.".into())
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Owned {
    pub id: u64,
    pub name: String,
    pub thumb: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OwnedPage {
    pub items: Vec<Owned>,
    pub next_cursor: Option<String>,
}

pub async fn inventory(
    user_id: u64,
    cookie: &str,
    asset_type: u64,
    cursor: Option<String>,
) -> Result<OwnedPage, String> {
    let mut url = format!(
        "https://inventory.roblox.com/v2/users/{user_id}/inventory/{asset_type}?limit=100&sortOrder=Desc"
    );
    if let Some(c) = cursor.filter(|c| !c.is_empty()) {
        url += &format!("&cursor={}", urlencoding::encode(&c));
    }
    let v = get_json(&url, Some(cookie)).await?;
    let mut items: Vec<Owned> = v["data"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|a| {
            Some(Owned {
                id: a["assetId"].as_u64()?,
                name: a["assetName"]
                    .as_str()
                    .or(a["name"].as_str())
                    .unwrap_or_default()
                    .to_string(),
                thumb: None,
            })
        })
        .collect();
    let t = thumbs(
        &items
            .iter()
            .map(|a| ("Asset".to_string(), a.id))
            .collect::<Vec<_>>(),
    )
    .await;
    for a in &mut items {
        a.thumb = t.get(&("Asset".to_string(), a.id)).cloned();
    }
    Ok(OwnedPage {
        items,
        next_cursor: v["nextPageCursor"].as_str().map(str::to_string),
    })
}

pub async fn owns(user_id: u64, cookie: &str, item_type: &str, id: u64) -> Result<bool, String> {
    let kind = if item_type.eq_ignore_ascii_case("bundle") {
        "Bundle"
    } else {
        "Asset"
    };
    let v = get_json(
        &format!("https://inventory.roblox.com/v1/users/{user_id}/items/{kind}/{id}/is-owned"),
        Some(cookie),
    )
    .await?;
    Ok(v.as_bool().unwrap_or(false))
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Outfit {
    pub id: u64,
    pub name: String,
    pub thumb: Option<String>,
}

pub async fn outfits(user_id: u64, cookie: &str) -> Result<Vec<Outfit>, String> {
    let v = get_json(&format!("https://avatar.roblox.com/v1/users/{user_id}/outfits?page=1&itemsPerPage=50&isEditable=true"), Some(cookie)).await?;
    let mut list: Vec<Outfit> = v["data"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|o| {
            Some(Outfit {
                id: o["id"].as_u64()?,
                name: o["name"].as_str().unwrap_or_default().to_string(),
                thumb: None,
            })
        })
        .collect();
    if !list.is_empty() {
        let ids = list
            .iter()
            .map(|o| o.id.to_string())
            .collect::<Vec<_>>()
            .join(",");
        if let Ok(t) = get_json(&format!("https://thumbnails.roblox.com/v1/users/outfits?userOutfitIds={ids}&size=150x150&format=Png&isCircular=false"), None).await {
            for d in t["data"].as_array().into_iter().flatten() {
                if let (Some(id), Some(u)) = (d["targetId"].as_u64(), d["imageUrl"].as_str()) {
                    if let Some(o) = list.iter_mut().find(|o| o.id == id) { o.thumb = Some(u.to_string()); }
                }
            }
        }
    }
    Ok(list)
}

pub async fn wear(cookie: &str, asset_ids: &[u64]) -> Result<Vec<u64>, String> {
    let assets: Vec<Value> = asset_ids.iter().map(|id| json!({ "id": id })).collect();
    let v = post_ok(
        "https://avatar.roblox.com/v2/avatar/set-wearing-assets",
        cookie,
        json!({ "assets": assets }),
        "Changing what you wear",
    )
    .await?;
    Ok(v["invalidAssetIds"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(Value::as_u64)
        .collect())
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BodyColors {
    head_color_id: u64,
    torso_color_id: u64,
    right_arm_color_id: u64,
    left_arm_color_id: u64,
    right_leg_color_id: u64,
    left_leg_color_id: u64,
}

pub async fn set_body_colors(cookie: &str, c: &BodyColors) -> Result<(), String> {
    post_ok(
        "https://avatar.roblox.com/v1/avatar/set-body-colors",
        cookie,
        serde_json::to_value(c).unwrap(),
        "Changing skin color",
    )
    .await
    .map(|_| ())
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Scales {
    height: f64,
    width: f64,
    head: f64,
    depth: f64,
    proportion: f64,
    body_type: f64,
}

pub async fn set_scales(cookie: &str, s: &Scales) -> Result<(), String> {
    post_ok(
        "https://avatar.roblox.com/v1/avatar/set-scales",
        cookie,
        serde_json::to_value(s).unwrap(),
        "Changing body scale",
    )
    .await
    .map(|_| ())
}

pub async fn set_type(cookie: &str, avatar_type: &str) -> Result<(), String> {
    if avatar_type != "R6" && avatar_type != "R15" {
        return Err("Avatar type must be R6 or R15.".into());
    }
    post_ok(
        "https://avatar.roblox.com/v1/avatar/set-player-avatar-type",
        cookie,
        json!({ "playerAvatarType": avatar_type }),
        "Changing avatar type",
    )
    .await
    .map(|_| ())
}

pub async fn wear_outfit(cookie: &str, outfit_id: u64) -> Result<Vec<u64>, String> {
    let d = get_json(
        &format!("https://avatar.roblox.com/v1/outfits/{outfit_id}/details"),
        Some(cookie),
    )
    .await?;
    if let Some(t) = d["playerAvatarType"].as_str() {
        set_type(cookie, t).await?;
    }
    if let Ok(c) = serde_json::from_value::<BodyColors>(d["bodyColors"].clone()) {
        set_body_colors(cookie, &c).await?;
    }
    if let Ok(s) = serde_json::from_value::<Scales>(d["scale"].clone())
        .or_else(|_| serde_json::from_value::<Scales>(d["scales"].clone()))
    {
        set_scales(cookie, &s).await?;
    }
    let ids: Vec<u64> = d["assets"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|a| a["id"].as_u64())
        .collect();
    wear(cookie, &ids).await
}

pub async fn redraw(cookie: &str) {
    let _ = post(
        "https://avatar.roblox.com/v1/avatar/redraw-thumbnail",
        Some(cookie),
        json!({}),
    )
    .await;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn body_color_and_scale_shapes_match_roblox() {
        let c: BodyColors = serde_json::from_value(json!({ "headColorId": 194, "torsoColorId": 23, "rightArmColorId": 194, "leftArmColorId": 194, "rightLegColorId": 102, "leftLegColorId": 102 })).unwrap();
        assert_eq!(serde_json::to_value(&c).unwrap()["torsoColorId"], 23);
        let s: Scales = serde_json::from_value(json!({ "height": 1.0, "width": 1.0, "head": 1.0, "depth": 1.0, "proportion": 0.0, "bodyType": 0.0 })).unwrap();
        assert_eq!(serde_json::to_value(&s).unwrap()["bodyType"], 0.0);
    }
}
