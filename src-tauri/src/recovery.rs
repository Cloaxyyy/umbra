use super::{logwatch::Session, roblox::Target};
use serde::Serialize;
use std::{
    collections::HashMap,
    sync::{LazyLock, Mutex},
    time::{SystemTime, UNIX_EPOCH},
};

pub const WINDOW_SECS: u64 = 15 * 60;
pub const MAX_TRIES: usize = 5;
const LOG_SIZE: usize = 30;
const USER_LEFT: &[u32] = &[264, 273, 285];

#[derive(Serialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Disconnect {
    pub user_id: u64,
    pub place_id: u64,
    pub universe_id: u64,
    pub game_name: Option<String>,
    pub reason: String,
    pub code: Option<u32>,
    pub at: u64,
    pub action: String,
}

static LAST: LazyLock<Mutex<HashMap<u64, Target>>> = LazyLock::new(|| Mutex::new(HashMap::new()));
static TRIES: LazyLock<Mutex<HashMap<u64, Vec<u64>>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));
static LOG: Mutex<Vec<Disconnect>> = Mutex::new(Vec::new());

fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

pub fn remember(user_id: u64, target: &Target) {
    if !matches!(target, Target::App) {
        LAST.lock().unwrap().insert(user_id, target.clone());
    }
}

pub fn user_left(code: Option<u32>) -> bool {
    code.is_some_and(|c| USER_LEFT.contains(&c))
}

fn prune(list: &mut Vec<u64>, at: u64) {
    list.retain(|t| at.saturating_sub(*t) < WINDOW_SECS);
}

pub fn allow(user_id: u64, at: u64) -> Option<usize> {
    let mut tries = TRIES.lock().unwrap();
    let list = tries.entry(user_id).or_default();
    prune(list, at);
    if list.len() >= MAX_TRIES {
        return None;
    }
    list.push(at);
    Some(list.len())
}

pub fn target_for(user_id: u64, session: Option<&Session>, attempt: usize) -> Option<Target> {
    let last = LAST.lock().unwrap().get(&user_id).cloned();
    if let Some(Target::Place { place, .. }) = &last {
        if super::roblox::link_code(place).is_some() {
            return last;
        }
    }
    match session {
        Some(s) => Some(Target::Place {
            place: s.place_id.to_string(),
            job_id: s.job_id.clone().filter(|_| attempt <= 1),
        }),
        None => last,
    }
}

pub fn note(d: Disconnect) {
    let mut log = LOG.lock().unwrap();
    log.insert(0, d);
    log.truncate(LOG_SIZE);
}

pub fn recent() -> Vec<Disconnect> {
    LOG.lock().unwrap().clone()
}

pub fn record(session: &Session, reason: &str, action: &str) -> Disconnect {
    let d = Disconnect {
        user_id: session.user_id,
        place_id: session.place_id,
        universe_id: session.universe_id,
        game_name: session.game_name.clone(),
        reason: reason.to_string(),
        code: super::logwatch::reason_code(reason),
        at: now(),
        action: action.to_string(),
    };
    note(d.clone());
    d
}

#[cfg(test)]
mod tests {
    use super::*;

    fn session(user: u64, job: Option<&str>) -> Session {
        Session {
            user_id: user,
            place_id: 920587237,
            universe_id: 383310974,
            job_id: job.map(str::to_string),
            ..Default::default()
        }
    }

    #[test]
    fn limits_retries_per_window() {
        let user = 900_000_001;
        for i in 1..=MAX_TRIES {
            assert_eq!(allow(user, 1_000), Some(i));
        }
        assert_eq!(allow(user, 1_000), None);
        assert_eq!(allow(user, 1_000 + WINDOW_SECS), Some(1));
    }

    #[test]
    fn picks_same_server_first_then_any() {
        let user = 900_000_002;
        let s = session(user, Some("abc-123"));
        assert_eq!(
            target_for(user, Some(&s), 1),
            Some(Target::Place {
                place: "920587237".into(),
                job_id: Some("abc-123".into())
            })
        );
        assert_eq!(
            target_for(user, Some(&s), 2),
            Some(Target::Place {
                place: "920587237".into(),
                job_id: None
            })
        );
    }

    #[test]
    fn private_servers_rejoin_their_link() {
        let user = 900_000_003;
        let link = Target::Place {
            place: "https://www.roblox.com/games/606849621?privateServerLinkCode=48727401".into(),
            job_id: None,
        };
        remember(user, &link);
        assert_eq!(
            target_for(user, Some(&session(user, Some("x"))), 1),
            Some(link)
        );
    }

    #[test]
    fn leaving_on_purpose_is_not_a_drop() {
        assert!(user_left(Some(285)));
        assert!(user_left(Some(273)));
        assert!(!user_left(Some(267)));
        assert!(!user_left(None));
    }
}
