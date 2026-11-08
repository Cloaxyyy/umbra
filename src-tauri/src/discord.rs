use super::logwatch::Session;
use serde_json::{Value, json};
use std::{
    fs::{File, OpenOptions},
    io::{Read, Write},
};

pub struct Discord {
    pipe: Option<File>,
    app_id: String,
    last: Option<Value>,
}

impl Default for Discord {
    fn default() -> Self {
        Self {
            pipe: None,
            app_id: String::new(),
            last: None,
        }
    }
}

fn frame(op: u32, body: &Value) -> Vec<u8> {
    let data = serde_json::to_vec(body).unwrap();
    let mut out = Vec::with_capacity(8 + data.len());
    out.extend_from_slice(&op.to_le_bytes());
    out.extend_from_slice(&(data.len() as u32).to_le_bytes());
    out.extend_from_slice(&data);
    out
}

fn read_frame(f: &mut File) -> std::io::Result<(u32, Value)> {
    let mut head = [0u8; 8];
    f.read_exact(&mut head)?;
    let op = u32::from_le_bytes(head[0..4].try_into().unwrap());
    let len = u32::from_le_bytes(head[4..8].try_into().unwrap()) as usize;
    if len > 1 << 20 {
        return Err(std::io::Error::other("frame too large"));
    }
    let mut body = vec![0u8; len];
    f.read_exact(&mut body)?;
    Ok((op, serde_json::from_slice(&body).unwrap_or(Value::Null)))
}

pub fn activity(s: &Session, join_button: bool) -> Value {
    let name = s
        .game_name
        .clone()
        .unwrap_or_else(|| "a Roblox game".into());
    let mut buttons = vec![
        json!({ "label": "View game", "url": format!("https://www.roblox.com/games/{}", s.place_id) }),
    ];
    if let (true, Some(job)) = (join_button, &s.job_id) {
        buttons.insert(0, json!({ "label": "Join server", "url": format!("https://www.roblox.com/games/start?placeId={}&gameInstanceId={job}", s.place_id) }));
    }
    let mut a = json!({
        "details": format!("Playing {name}"),
        "timestamps": { "start": s.joined_at },
        "assets": { "large_text": name },
        "buttons": buttons,
    });
    if let Some(loc) = &s.location {
        a["state"] = json!(format!("Server in {loc}"));
    }
    if let Some(icon) = &s.game_icon {
        a["assets"]["large_image"] = json!(icon);
    }
    a
}

impl Discord {
    fn connect(&mut self) -> Result<(), String> {
        for n in 0..10 {
            let Ok(mut f) = OpenOptions::new()
                .read(true)
                .write(true)
                .open(format!(r"\\.\pipe\discord-ipc-{n}"))
            else {
                continue;
            };
            f.write_all(&frame(0, &json!({ "v": 1, "client_id": self.app_id })))
                .map_err(|e| e.to_string())?;
            let (op, v) = read_frame(&mut f).map_err(|e| e.to_string())?;
            if op == 2 || v["evt"] == "ERROR" || v["code"].is_number() {
                return Err(format!(
                    "Discord refused the application id: {}",
                    v["message"].as_str().unwrap_or("unknown error")
                ));
            }
            self.pipe = Some(f);
            return Ok(());
        }
        Err("Discord isn't running.".into())
    }

    pub fn set(&mut self, app_id: &str, activity: Option<Value>) -> Result<(), String> {
        if app_id != self.app_id {
            self.disconnect();
            self.app_id = app_id.to_string();
        }

        if self.pipe.is_none() {
            self.last = None;
            self.connect()?;
        }
        if activity == self.last {
            return Ok(());
        }
        let body = json!({
            "cmd": "SET_ACTIVITY",
            "args": { "pid": std::process::id(), "activity": activity.clone().unwrap_or(Value::Null) },
            "nonce": format!("{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()),
        });
        let f = self.pipe.as_mut().unwrap();
        let sent = f.write_all(&frame(1, &body)).and_then(|_| read_frame(f));
        match sent {
            Ok((_, v)) if v["evt"] == "ERROR" => Err(format!(
                "Discord rejected the status: {}",
                v["data"]["message"].as_str().unwrap_or("unknown")
            )),
            Ok(_) => {
                self.last = activity;
                Ok(())
            }
            Err(_) => {
                self.pipe = None;
                Err("Lost the connection to Discord.".into())
            }
        }
    }

    pub fn disconnect(&mut self) {
        if let Some(mut f) = self.pipe.take() {
            let _ = f.write_all(&frame(2, &json!({})));
        }
        self.last = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frames_are_little_endian_with_length() {
        let f = frame(1, &json!({ "a": 1 }));
        assert_eq!(&f[0..4], &[1, 0, 0, 0]);
        assert_eq!(
            u32::from_le_bytes(f[4..8].try_into().unwrap()) as usize,
            f.len() - 8
        );
    }

    #[test]
    fn join_button_only_when_allowed() {
        let s = Session {
            place_id: 1,
            job_id: Some("abc".into()),
            game_name: Some("Doors".into()),
            joined_at: 5,
            ..Default::default()
        };
        let off = activity(&s, false);
        assert_eq!(off["buttons"].as_array().unwrap().len(), 1);
        assert_eq!(off["details"], "Playing Doors");
        let on = activity(&s, true);
        assert_eq!(on["buttons"][0]["label"], "Join server");
        assert!(
            on["buttons"][0]["url"]
                .as_str()
                .unwrap()
                .contains("gameInstanceId=abc")
        );
        assert!(off.get("state").is_none());
    }
}
