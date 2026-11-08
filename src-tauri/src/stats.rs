use super::{discover::GameCard, personal::Visit};
use serde::Serialize;
use std::collections::{BTreeSet, HashMap};

const DAY: i64 = 86_400;
const TOP_GAMES: usize = 12;

#[derive(Serialize, Clone, Debug, Default, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GameStat {
    pub universe_id: u64,
    pub root_place_id: u64,
    pub secs: u64,
    pub sessions: u64,
    pub last: u64,
    pub name: Option<String>,
    pub icon: Option<String>,
}

#[derive(Serialize, Clone, Debug, Default, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AccountStat {
    pub user_id: u64,
    pub secs: u64,
    pub sessions: u64,
}

#[derive(Serialize, Clone, Debug, Default, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Stats {
    pub total_secs: u64,
    pub sessions: u64,
    pub days_played: u64,
    pub streak: u64,
    pub longest_secs: u64,
    pub first: Option<u64>,
    pub week: Vec<u64>,
    pub games: Vec<GameStat>,
    pub accounts: Vec<AccountStat>,
}

fn day_of(ts: u64, tz: i64) -> i64 {
    (ts as i64 + tz).div_euclid(DAY)
}

pub fn compute(visits: &[Visit], user: Option<u64>, now: u64, tz: i64) -> Stats {
    let mine: Vec<&Visit> = visits
        .iter()
        .filter(|v| user.is_none_or(|u| v.user_id == u))
        .collect();
    let today = day_of(now, tz);
    let mut out = Stats {
        week: vec![0; 7],
        ..Default::default()
    };
    let mut days = BTreeSet::new();
    let mut games: HashMap<u64, GameStat> = HashMap::new();
    let mut accounts: HashMap<u64, AccountStat> = HashMap::new();
    for v in &mine {
        out.total_secs += v.secs;
        out.sessions += 1;
        out.longest_secs = out.longest_secs.max(v.secs);
        out.first = Some(out.first.map_or(v.start, |f| f.min(v.start)));
        let d = day_of(v.start, tz);
        days.insert(d);
        let back = today - d;
        if (0..7).contains(&back) {
            out.week[(6 - back) as usize] += v.secs;
        }
        let g = games.entry(v.universe_id).or_insert_with(|| GameStat {
            universe_id: v.universe_id,
            root_place_id: v.place_id,
            ..Default::default()
        });
        g.secs += v.secs;
        g.sessions += 1;
        if v.start >= g.last {
            g.last = v.start;
        }
        let a = accounts.entry(v.user_id).or_insert_with(|| AccountStat {
            user_id: v.user_id,
            ..Default::default()
        });
        a.secs += v.secs;
        a.sessions += 1;
    }
    out.days_played = days.len() as u64;
    let mut cursor = if days.contains(&today) {
        today
    } else {
        today - 1
    };
    while days.contains(&cursor) {
        out.streak += 1;
        cursor -= 1;
    }
    let mut games: Vec<GameStat> = games.into_values().collect();
    games.sort_by(|a, b| b.secs.cmp(&a.secs).then(b.last.cmp(&a.last)));
    games.truncate(TOP_GAMES);
    out.games = games;
    let mut accounts: Vec<AccountStat> = accounts.into_values().collect();
    accounts.sort_by(|a, b| b.secs.cmp(&a.secs));
    out.accounts = accounts;
    out
}

pub fn name_games(stats: &mut Stats, cards: &HashMap<u64, GameCard>) {
    for g in &mut stats.games {
        if let Some(c) = cards.get(&g.universe_id) {
            g.name = Some(c.name.clone());
            g.icon = c.icon.clone();
            g.root_place_id = c.root_place_id;
        }
    }
}

pub fn playtime(visits: &[Visit], universe_id: u64) -> (u64, u64, u64) {
    visits
        .iter()
        .filter(|v| v.universe_id == universe_id)
        .fold((0, 0, 0), |(secs, n, last), v| {
            (secs + v.secs, n + 1, last.max(v.start))
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn visit(user: u64, universe: u64, start: u64, secs: u64) -> Visit {
        Visit {
            user_id: user,
            universe_id: universe,
            place_id: universe * 10,
            start,
            secs,
        }
    }

    #[test]
    fn totals_week_and_streak() {
        let now = 10 * DAY as u64 + 3_600;
        let v = vec![
            visit(1, 100, now - 600, 300),
            visit(1, 100, now - DAY as u64, 1_200),
            visit(1, 200, now - 2 * DAY as u64, 600),
            visit(1, 200, now - 5 * DAY as u64, 60),
            visit(2, 300, now - 30, 30),
        ];
        let s = compute(&v, Some(1), now, 0);
        assert_eq!(s.total_secs, 2_160);
        assert_eq!(s.sessions, 4);
        assert_eq!(s.days_played, 4);
        assert_eq!(s.streak, 3);
        assert_eq!(s.longest_secs, 1_200);
        assert_eq!(s.week, vec![0, 60, 0, 0, 600, 1_200, 300]);
        assert_eq!(s.games[0].universe_id, 100);
        assert_eq!(s.games[0].secs, 1_500);
        assert_eq!(s.accounts.len(), 1);
        let everyone = compute(&v, None, now, 0);
        assert_eq!(everyone.accounts.len(), 2);
        assert_eq!(playtime(&v, 200), (660, 2, now - 2 * DAY as u64));
    }

    #[test]
    fn streak_survives_until_tomorrow_and_respects_timezone() {
        let now = 20 * DAY as u64 + 600;
        let v = vec![
            visit(1, 1, now - DAY as u64, 10),
            visit(1, 1, now - 2 * DAY as u64, 10),
        ];
        assert_eq!(compute(&v, None, now, 0).streak, 2);
        let late = vec![visit(1, 1, 20 * DAY as u64 - 600, 10)];
        assert_eq!(compute(&late, None, now, 0).days_played, 1);
        assert_eq!(compute(&late, None, now, 3_600).week[6], 10);
    }
}
