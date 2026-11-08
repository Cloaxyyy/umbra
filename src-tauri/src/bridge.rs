use super::{accounts, favorites, launch, logwatch, roblox, settings};
use md5::{Digest, Md5};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{
    collections::HashMap,
    fs,
    io::Read,
    sync::{LazyLock, Mutex, mpsc},
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use tauri::{AppHandle, Emitter, Manager};
use tiny_http::{Header, Method, Request, Response, Server};

pub const PORT: u16 = 27150;
const PAIR_WAIT: Duration = Duration::from_secs(90);
const MAX_BODY: u64 = 64 * 1024;
const MAX_LAUNCH: usize = 20;
const ORIGINS: &[&str] = &[
    "chrome-extension://",
    "moz-extension://",
    "safari-web-extension://",
];
const HOSTS: &[&str] = &["127.0.0.1:27150", "localhost:27150"];
const CLIENT_HEADER: &str = "X-Umbra-Client";
const CLIENT_VALUE: &str = "extension";

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct Grant {
    id: String,
    hash: String,
    label: String,
    origin: String,
    created: u64,
}

static LOCK: Mutex<()> = Mutex::new(());
static PENDING: LazyLock<Mutex<HashMap<String, mpsc::Sender<bool>>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

fn file() -> std::path::PathBuf {
    super::paths::base().join("bridge.json")
}

fn read() -> Vec<Grant> {
    fs::read_to_string(file())
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

fn write(list: &[Grant]) -> Result<(), String> {
    fs::create_dir_all(super::paths::base()).map_err(|e| e.to_string())?;
    fs::write(file(), serde_json::to_string_pretty(list).unwrap()).map_err(|e| e.to_string())
}

fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

fn random_hex(bytes: usize) -> Result<String, String> {
    use windows_sys::Win32::Security::Cryptography::{
        BCRYPT_USE_SYSTEM_PREFERRED_RNG, BCryptGenRandom,
    };
    let mut buf = vec![0u8; bytes];
    let status = unsafe {
        BCryptGenRandom(
            std::ptr::null_mut(),
            buf.as_mut_ptr(),
            buf.len() as u32,
            BCRYPT_USE_SYSTEM_PREFERRED_RNG,
        )
    };
    if status != 0 {
        return Err("Windows couldn't create a secure token.".into());
    }
    Ok(buf.iter().map(|b| format!("{b:02x}")).collect())
}

fn hash(token: &str) -> String {
    let mut h = Md5::new();
    h.update(token.as_bytes());
    h.finalize().iter().map(|b| format!("{b:02x}")).collect()
}

fn valid(token: &str) -> bool {
    let h = hash(token);
    let _g = LOCK.lock().unwrap();
    read().iter().any(|g| g.hash == h)
}

fn clean_label(s: &str) -> String {
    let t: String = s.chars().filter(|c| !c.is_control()).take(40).collect();
    let t = t.trim();
    if t.is_empty() {
        "Browser extension".into()
    } else {
        t.to_string()
    }
}

pub fn grants() -> Vec<Value> {
    let _g = LOCK.lock().unwrap();
    read()
        .into_iter()
        .map(|g| json!({ "id": g.id, "label": g.label, "origin": g.origin, "created": g.created }))
        .collect()
}

pub fn revoke(id: Option<&str>) -> Result<(), String> {
    let _g = LOCK.lock().unwrap();
    let mut all = read();
    match id {
        Some(id) => all.retain(|g| g.id != id),
        None => all.clear(),
    }
    write(&all)
}

pub fn answer(id: &str, allow: bool) -> Result<(), String> {
    let tx = PENDING
        .lock()
        .unwrap()
        .remove(id)
        .ok_or("That request already expired.")?;
    let _ = tx.send(allow);
    Ok(())
}

pub fn start(app: AppHandle) {
    std::thread::spawn(move || {
        let Ok(server) = Server::http(("127.0.0.1", PORT)) else {
            return;
        };
        for req in server.incoming_requests() {
            let app = app.clone();
            std::thread::spawn(move || handle(app, req));
        }
    });
}

fn header(req: &Request, name: &str) -> Option<String> {
    req.headers()
        .iter()
        .find(|h| h.field.as_str().as_str().eq_ignore_ascii_case(name))
        .map(|h| h.value.as_str().to_string())
}

fn reply(req: Request, origin: Option<&str>, status: u16, body: Value) {
    let mut r = Response::from_string(body.to_string())
        .with_status_code(status)
        .with_header(Header::from_bytes("Content-Type", "application/json").unwrap());
    if let Some(o) = origin {
        for (k, v) in [
            ("Access-Control-Allow-Origin", o),
            (
                "Access-Control-Allow-Headers",
                "authorization, content-type",
            ),
            ("Access-Control-Allow-Methods", "GET, POST, OPTIONS"),
            ("Vary", "Origin"),
        ] {
            r = r.with_header(Header::from_bytes(k, v).unwrap());
        }
    }
    let _ = req.respond(r);
}

fn fail(msg: &str) -> Value {
    json!({ "error": msg })
}

fn is_extension(origin: &str) -> bool {
    ORIGINS.iter().any(|p| origin.starts_with(p))
}

fn origin_allowed(origin: Option<&str>) -> bool {
    origin.is_none_or(|o| o == "null" || is_extension(o))
}

fn handle(app: AppHandle, mut req: Request) {
    let origin = header(&req, "Origin");
    let from_ext = origin_allowed(origin.as_deref());
    let host_ok = header(&req, "Host").is_some_and(|h| HOSTS.contains(&h.as_str()));
    let origin = origin.filter(|o| is_extension(o));
    if !from_ext || !host_ok {
        reply(
            req,
            None,
            403,
            fail("Only the Umbra browser extension can use this."),
        );
        return;
    }
    if *req.method() == Method::Options {
        if origin.is_some() {
            reply(req, origin.as_deref(), 204, Value::Null);
        } else {
            reply(
                req,
                None,
                403,
                fail("Only the Umbra browser extension can use this."),
            );
        }
        return;
    }
    if header(&req, CLIENT_HEADER).as_deref() != Some(CLIENT_VALUE) {
        reply(
            req,
            origin.as_deref(),
            403,
            fail("Only the Umbra browser extension can use this."),
        );
        return;
    }
    let path = req.url().split('?').next().unwrap_or_default().to_string();
    let query = req
        .url()
        .split_once('?')
        .map(|(_, q)| q.to_string())
        .unwrap_or_default();
    let method = req.method().clone();
    let token =
        header(&req, "Authorization").and_then(|v| v.strip_prefix("Bearer ").map(str::to_string));
    let paired = token.as_deref().is_some_and(valid);
    let enabled = settings::load().extension_bridge;
    let mut body = String::new();
    let _ = req.as_reader().take(MAX_BODY).read_to_string(&mut body);
    let body: Value = serde_json::from_str(&body).unwrap_or(Value::Null);

    if method == Method::Get && path == "/v1/hello" {
        let version = app.package_info().version.to_string();
        reply(
            req,
            origin.as_deref(),
            200,
            json!({ "app": "umbra", "version": version, "enabled": enabled, "paired": paired && enabled }),
        );
        return;
    }
    if !enabled {
        reply(
            req,
            origin.as_deref(),
            403,
            fail("The browser extension connection is turned off in Umbra's settings."),
        );
        return;
    }
    if method == Method::Post && path == "/v1/pair" {
        let (status, out) = pair(
            &app,
            origin.as_deref().unwrap_or("browser extension"),
            &body,
        );
        reply(req, origin.as_deref(), status, out);
        return;
    }
    if !paired {
        reply(
            req,
            origin.as_deref(),
            401,
            fail("Connect the extension to Umbra first."),
        );
        return;
    }
    let (status, out) = match (method, path.as_str()) {
        (Method::Get, "/v1/state") => (200, state(&app)),
        (Method::Get, "/v1/playtime") => playtime(&query),
        (Method::Post, "/v1/launch") => launch_many(&app, &body),
        (Method::Post, "/v1/favorite") => favorite(&app, &body),
        (Method::Post, "/v1/client") => client(&app, &body),
        (Method::Post, "/v1/unpair") => {
            let h = hash(token.as_deref().unwrap_or_default());
            let _g = LOCK.lock().unwrap();
            let mut all = read();
            all.retain(|g| g.hash != h);
            match write(&all) {
                Ok(()) => (200, json!({ "ok": true })),
                Err(e) => (500, fail(&e)),
            }
        }
        _ => (404, fail("Unknown request.")),
    };
    reply(req, origin.as_deref(), status, out);
}

fn pair(app: &AppHandle, origin: &str, body: &Value) -> (u16, Value) {
    let label = clean_label(body["label"].as_str().unwrap_or_default());
    let id = match random_hex(8) {
        Ok(id) => id,
        Err(e) => return (500, fail(&e)),
    };
    let (tx, rx) = mpsc::channel();
    {
        let mut p = PENDING.lock().unwrap();
        if !p.is_empty() {
            return (
                429,
                fail("Umbra is already asking about another connection. Answer it first."),
            );
        }
        p.insert(id.clone(), tx);
    }
    super::show_main(app);
    let _ = app.emit(
        "bridge-pair",
        json!({ "id": id, "label": label, "origin": origin }),
    );
    let allowed = rx.recv_timeout(PAIR_WAIT).unwrap_or(false);
    PENDING.lock().unwrap().remove(&id);
    if !allowed {
        return (403, fail("Umbra didn't allow the connection."));
    }
    let token = match random_hex(32) {
        Ok(t) => t,
        Err(e) => return (500, fail(&e)),
    };
    let grant = Grant {
        id: random_hex(6).unwrap_or_else(|_| now().to_string()),
        hash: hash(&token),
        label,
        origin: origin.to_string(),
        created: now(),
    };
    let _g = LOCK.lock().unwrap();
    let mut all = read();
    all.push(grant);
    match write(&all) {
        Ok(()) => (200, json!({ "token": token })),
        Err(e) => (500, fail(&e)),
    }
}

fn query_u64(query: &str, key: &str) -> Option<u64> {
    query
        .split('&')
        .filter_map(|kv| kv.split_once('='))
        .find(|(k, _)| *k == key)
        .and_then(|(_, v)| v.parse().ok())
}

fn playtime(query: &str) -> (u16, Value) {
    let Some(id) = query_u64(query, "universeId") else {
        return (400, fail("Missing the game."));
    };
    let (secs, sessions, last) = super::stats::playtime(&super::personal::history(), id);
    (
        200,
        json!({ "secs": secs, "sessions": sessions, "last": last }),
    )
}

fn state(app: &AppHandle) -> Value {
    let accounts = tauri::async_runtime::block_on(accounts::list());
    let store = super::list_profiles();
    let profiles: Vec<Value> = store
        .profiles
        .iter()
        .map(|p| json!({ "id": p.id, "name": p.name, "color": p.color }))
        .collect();
    let sessions = app.state::<logwatch::Watcher>().sessions();
    let mut running = app.state::<launch::Running>().snapshot();
    for c in &mut running {
        c.game = c.user_id.and_then(|u| sessions.get(&u).cloned());
    }
    let favorites: Vec<u64> = favorites::list().iter().map(|f| f.universe_id).collect();
    json!({
        "accounts": accounts,
        "profiles": { "defaultId": store.default_id, "profiles": profiles, "games": store.games },
        "running": running,
        "favorites": favorites,
    })
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct LaunchBody {
    user_ids: Vec<u64>,
    target: roblox::Target,
    #[serde(default)]
    profile_id: Option<String>,
}

fn launch_many(app: &AppHandle, body: &Value) -> (u16, Value) {
    let Ok(b) = serde_json::from_value::<LaunchBody>(body.clone()) else {
        return (400, fail("That launch request isn't valid."));
    };
    if b.user_ids.is_empty() || b.user_ids.len() > MAX_LAUNCH {
        return (400, fail("Pick between 1 and 20 accounts."));
    }
    let mut started = 0;
    let mut errors = vec![];
    for (i, id) in b.user_ids.iter().enumerate() {
        if i > 0 {
            std::thread::sleep(Duration::from_millis(1500));
        }
        let res = tauri::async_runtime::block_on(super::launch_account(
            app.clone(),
            app.state::<launch::Running>(),
            *id,
            b.target.clone(),
            b.profile_id.clone(),
        ));
        match res {
            Ok(_) => started += 1,
            Err(e) => errors.push(json!({ "userId": id, "error": e })),
        }
    }
    let status = if started > 0 { 200 } else { 502 };
    (status, json!({ "started": started, "errors": errors }))
}

fn favorite(app: &AppHandle, body: &Value) -> (u16, Value) {
    let Ok(f) = serde_json::from_value::<favorites::Favorite>(body.clone()) else {
        return (400, fail("That favorite isn't valid."));
    };
    match favorites::toggle(f) {
        Ok(added) => {
            let _ = app.emit("favorites-changed", ());
            (200, json!({ "added": added }))
        }
        Err(e) => (500, fail(&e)),
    }
}

fn client(app: &AppHandle, body: &Value) -> (u16, Value) {
    let Some(pid) = body["pid"].as_u64().map(|p| p as u32) else {
        return (400, fail("Missing the client to control."));
    };
    let action = body["action"].as_str().unwrap_or_default();
    let res = match action {
        "close" => app.state::<launch::Running>().kill(pid).map(|_| pid),
        "rejoin" | "hop" => tauri::async_runtime::block_on(super::move_client(
            app.clone(),
            app.state::<launch::Running>(),
            app.state::<logwatch::Watcher>(),
            pid,
            action.to_string(),
            None,
            None,
        )),
        _ => return (400, fail("Unknown action.")),
    };
    match res {
        Ok(pid) => (200, json!({ "pid": pid })),
        Err(e) => (502, fail(&e)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn labels_and_tokens() {
        assert_eq!(clean_label("  Chrome on Windows  "), "Chrome on Windows");
        assert_eq!(clean_label("\u{0007}"), "Browser extension");
        assert_eq!(clean_label(&"x".repeat(80)).len(), 40);
        let a = random_hex(32).unwrap();
        let b = random_hex(32).unwrap();
        assert_eq!(a.len(), 64);
        assert_ne!(a, b);
        assert_eq!(hash("abc"), "900150983cd24fb0d6963f7d28e17f72");
        assert_eq!(
            query_u64("universeId=383310974&x=1", "universeId"),
            Some(383310974)
        );
        assert_eq!(query_u64("universeId=abc", "universeId"), None);
        assert_eq!(query_u64("", "universeId"), None);
        assert!(origin_allowed(None));
        assert!(origin_allowed(Some("null")));
        assert!(origin_allowed(Some("moz-extension://4b1c-uuid")));
        assert!(origin_allowed(Some(
            "chrome-extension://hhndfjiapiginofjainnfgheokibklni"
        )));
        assert!(!origin_allowed(Some("https://www.roblox.com")));
        assert!(!origin_allowed(Some("http://127.0.0.1:8080")));
        assert!(!is_extension("null"));
    }
}
