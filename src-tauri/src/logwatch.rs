use serde::Serialize;
use std::{
    collections::HashMap,
    fs,
    io::{Read, Seek, SeekFrom},
    path::PathBuf,
    sync::Mutex,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

#[derive(Debug, PartialEq)]
pub enum Event {
    Joining {
        job_id: String,
        place_id: u64,
    },
    Joined {
        place_id: u64,
        universe_id: u64,
        user_id: u64,
    },
    Server {
        ip: String,
    },
    Left,
}

fn field<'a>(line: &'a str, key: &str) -> Option<&'a str> {
    let i = line.find(key)? + key.len();
    line[i..]
        .split(|c: char| c == ',' || c.is_whitespace())
        .next()
        .filter(|s| !s.is_empty())
}

pub fn parse_line(line: &str) -> Option<Event> {
    if let Some(i) = line.find("! Joining game '") {
        let rest = &line[i + 16..];
        let (job, rest) = rest.split_once('\'')?;
        let place = rest
            .trim_start()
            .strip_prefix("place ")?
            .split_whitespace()
            .next()?
            .parse()
            .ok()?;
        return Some(Event::Joining {
            job_id: job.to_string(),
            place_id: place,
        });
    }
    if line.contains("Report game_join_loadtime:") {
        return Some(Event::Joined {
            place_id: field(line, "placeid:")?.parse().ok()?,
            universe_id: field(line, "universeid:")?.parse().ok()?,
            user_id: field(line, "userid:")?.parse().ok()?,
        });
    }
    if let Some(i) = line.find("UDMUX Address = ") {
        let ip = line[i + 16..].split(',').next()?.trim();
        if ip.parse::<std::net::IpAddr>().is_ok() {
            return Some(Event::Server { ip: ip.to_string() });
        }
    }
    if line.contains("Disconnected from server for reason") {
        return Some(Event::Left);
    }
    None
}

#[derive(Serialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct Session {
    pub user_id: u64,
    pub place_id: u64,
    pub universe_id: u64,
    pub job_id: Option<String>,
    pub joined_at: u64,
    pub server_ip: Option<String>,
    pub game_name: Option<String>,
    pub game_icon: Option<String>,
    pub location: Option<String>,
}

#[derive(Default)]
struct FileState {
    offset: u64,
    partial: String,
    pending_job: Option<(String, u64)>,
    session: Option<Session>,
}

#[derive(Default)]
pub struct Watcher {
    files: Mutex<HashMap<PathBuf, FileState>>,
    games: Mutex<HashMap<u64, (String, Option<String>)>>,
    places: Mutex<HashMap<String, String>>,
}

pub(crate) fn logs_dir() -> Option<PathBuf> {
    dirs::data_local_dir().map(|d| d.join("Roblox").join("logs"))
}
fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

impl Watcher {
    pub fn scan(&self, since: SystemTime) {
        let Some(dir) = logs_dir() else { return };
        let Ok(rd) = fs::read_dir(dir) else { return };
        let mut files = self.files.lock().unwrap();
        for e in rd.flatten() {
            let p = e.path();
            let name = p.file_name().and_then(|n| n.to_str()).unwrap_or_default();
            if !name.contains("_Player_")
                || name.contains("CrashHandler")
                || !name.ends_with(".log")
            {
                continue;
            }
            let Ok(meta) = e.metadata() else { continue };
            let created = meta
                .created()
                .or_else(|_| meta.modified())
                .unwrap_or(UNIX_EPOCH);
            if created + Duration::from_secs(5) < since {
                continue;
            }
            let st = files.entry(p.clone()).or_default();
            if meta.len() <= st.offset {
                continue;
            }
            let Ok(mut f) = fs::File::open(&p) else {
                continue;
            };
            if f.seek(SeekFrom::Start(st.offset)).is_err() {
                continue;
            }
            let mut buf = Vec::new();
            if f.take(4 * 1024 * 1024).read_to_end(&mut buf).is_err() {
                continue;
            }
            st.offset += buf.len() as u64;
            let text = std::mem::take(&mut st.partial) + &String::from_utf8_lossy(&buf);
            let mut lines: Vec<&str> = text.split('\n').collect();
            st.partial = lines.pop().unwrap_or_default().to_string();
            for line in lines {
                apply(st, line);
            }
        }
    }

    pub fn sessions(&self) -> HashMap<u64, Session> {
        let mut out: HashMap<u64, Session> = HashMap::new();
        for st in self.files.lock().unwrap().values() {
            if let Some(s) = &st.session {
                if out
                    .get(&s.user_id)
                    .is_none_or(|o| o.joined_at <= s.joined_at)
                {
                    out.insert(s.user_id, s.clone());
                }
            }
        }
        out
    }

    pub fn forget(&self, user_id: u64) {
        for st in self.files.lock().unwrap().values_mut() {
            if st.session.as_ref().is_some_and(|s| s.user_id == user_id) {
                st.session = None;
            }
        }
    }

    pub async fn enrich(&self, with_location: bool) {
        let wanted: Vec<(u64, Option<String>)> = self
            .sessions()
            .values()
            .map(|s| (s.universe_id, s.server_ip.clone()))
            .collect();
        for (universe, ip) in wanted {
            let have = self.games.lock().unwrap().contains_key(&universe);
            if !have {
                let name = super::roblox::get_json(
                    &format!("https://games.roblox.com/v1/games?universeIds={universe}"),
                    None,
                )
                .await
                .ok()
                .and_then(|v| v["data"][0]["name"].as_str().map(str::to_string));
                let icon = super::roblox::get_json(&format!("https://thumbnails.roblox.com/v1/games/icons?universeIds={universe}&size=512x512&format=Png&isCircular=false"), None).await.ok()
                    .and_then(|v| v["data"][0]["imageUrl"].as_str().map(str::to_string));
                if let Some(name) = name {
                    self.games.lock().unwrap().insert(universe, (name, icon));
                }
            }
            if let (true, Some(ip)) = (with_location, ip) {
                let known = self.places.lock().unwrap().contains_key(&ip);
                if !known {
                    if let Ok(v) =
                        super::roblox::get_json(&format!("https://ipinfo.io/{ip}/json"), None).await
                    {
                        let parts: Vec<&str> = ["city", "region", "country"]
                            .iter()
                            .filter_map(|k| v[*k].as_str())
                            .collect();
                        let mut uniq: Vec<&str> = vec![];
                        for p in parts {
                            if !uniq.contains(&p) {
                                uniq.push(p);
                            }
                        }
                        if !uniq.is_empty() {
                            self.places.lock().unwrap().insert(ip, uniq.join(", "));
                        }
                    }
                }
            }
        }
        let games = self.games.lock().unwrap().clone();
        let places = self.places.lock().unwrap().clone();
        for st in self.files.lock().unwrap().values_mut() {
            if let Some(s) = &mut st.session {
                if let Some((n, i)) = games.get(&s.universe_id) {
                    s.game_name = Some(n.clone());
                    s.game_icon = i.clone();
                }
                s.location = if with_location {
                    s.server_ip.as_ref().and_then(|ip| places.get(ip).cloned())
                } else {
                    None
                };
            }
        }
    }
}

fn apply(st: &mut FileState, line: &str) {
    match parse_line(line) {
        Some(Event::Joining { job_id, place_id }) => st.pending_job = Some((job_id, place_id)),
        Some(Event::Joined {
            place_id,
            universe_id,
            user_id,
        }) => {
            let job_id = st
                .pending_job
                .take()
                .filter(|(_, p)| *p == place_id)
                .map(|(j, _)| j);
            st.session = Some(Session {
                user_id,
                place_id,
                universe_id,
                job_id,
                joined_at: now(),
                ..Default::default()
            });
        }
        Some(Event::Server { ip }) => {
            if let Some(s) = &mut st.session {
                s.server_ip = Some(ip);
            }
        }
        Some(Event::Left) => st.session = None,
        None => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const JOINING: &str = "2026-11-07T20:39:31.748Z,1.748139,9ac4,6 [FLog::Output] ! Joining game '87014612-77be-4905-befd-d5a074345780' place 13379208636 at 10.186.8.224";
    const JOINED: &str = "2026-11-07T20:39:31.748Z,1.748139,9ac4,6 [FLog::GameJoinLoadTime] Report game_join_loadtime: placeid:13379208636, join_time:0.66, universeid:4658598196, referral_page:RequestGame, sid:b4d8, clienttime:1794083971.74, userid:1234567890, ";
    const SERVER: &str = "2026-11-07T20:39:31.755Z,1.755169,8580,7 [FLog::Network] UDMUX Address = 128.116.48.33, Port = 55479 | RCC Server Address = 10.186.8.224, Port = 55479";
    const LEFT: &str = "2026-11-07T20:41:07.762Z,97.762634,9ac4,6,Info [DFLog::RbxTransportDummyClient] Disconnected from server for reason: Player: 285 (DisconnectClientInitiated)";

    #[test]
    fn parses_real_lines() {
        assert_eq!(
            parse_line(JOINING),
            Some(Event::Joining {
                job_id: "87014612-77be-4905-befd-d5a074345780".into(),
                place_id: 13379208636
            })
        );
        assert_eq!(
            parse_line(JOINED),
            Some(Event::Joined {
                place_id: 13379208636,
                universe_id: 4658598196,
                user_id: 1234567890
            })
        );
        assert_eq!(
            parse_line(SERVER),
            Some(Event::Server {
                ip: "128.116.48.33".into()
            })
        );
        assert_eq!(parse_line(LEFT), Some(Event::Left));
        assert_eq!(
            parse_line("2026-11-07T20:39:31Z [FLog::Network] Join snapshot timer: 0.81"),
            None
        );
    }

    #[test]
    #[ignore]
    fn real_logs() {
        let w = Watcher::default();
        w.scan(UNIX_EPOCH);
        let files = w.files.lock().unwrap().len();
        tauri::async_runtime::block_on(w.enrich(false));
        let live = w.sessions();
        println!(
            "read {files} logs; sessions still open at end of log: {}",
            live.len()
        );
        for s in live.values() {
            println!(
                "  user {} in {:?} (place {}, job {:?}, server {:?})",
                s.user_id, s.game_name, s.place_id, s.job_id, s.server_ip
            );
        }
        assert!(files > 0, "no Roblox logs found");
    }

    #[test]
    fn builds_and_ends_a_session() {
        let mut st = FileState::default();
        for l in [JOINING, JOINED, SERVER] {
            apply(&mut st, l);
        }
        let s = st.session.clone().unwrap();
        assert_eq!(
            (
                s.user_id,
                s.universe_id,
                s.job_id.as_deref(),
                s.server_ip.as_deref()
            ),
            (
                1234567890,
                4658598196,
                Some("87014612-77be-4905-befd-d5a074345780"),
                Some("128.116.48.33")
            )
        );
        apply(&mut st, LEFT);
        assert!(st.session.is_none());
    }
}
