mod accounts;
mod audio;
mod avatar;
mod bridge;
mod builder;
mod catalog;
mod deploy;
mod discord;
mod discover;
mod dpapi;
mod extras;
mod favorites;
mod launch;
mod logwatch;
mod mods;
mod overlay;
mod paths;
mod personal;
mod players;
mod profiles;
mod recovery;
mod roblox;
mod settings;
mod spending;
mod stats;
mod theme;
mod trades;
mod winctl;

use serde::Serialize;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tauri::{AppHandle, Emitter, Manager, State, WebviewUrl, WebviewWindowBuilder};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Status {
    installed: Option<String>,
    latest: deploy::ClientVersion,
    is_protocol_handler: bool,
    data_dir: String,
}

#[tauri::command]
async fn get_status() -> Result<Status, String> {
    let s = settings::load();
    let latest = deploy::latest(&s.channel).await?;
    let installed = s.installed_version.filter(|g| paths::is_installed(g));
    Ok(Status {
        installed,
        latest,
        is_protocol_handler: launch::is_protocol_handler(),
        data_dir: paths::base().display().to_string(),
    })
}

async fn ensure_version(app: &AppHandle, guid: &str, latest: &str) -> Result<(), String> {
    if !paths::valid_guid(guid) {
        return Err("That build id doesn't look right.".into());
    }
    if !paths::is_installed(guid) {
        deploy::install(app, guid).await.map_err(|e| {
            if guid != latest { format!("Couldn't get pinned build {guid}. Roblox may have removed it; follow the latest build instead. ({e})") } else { e }
        })?;
        if guid == latest {
            let mut s = settings::load();
            s.installed_version = Some(guid.to_string());
            settings::save(&s)?;
        }
    }
    let mut keep = profiles::pinned_versions();
    keep.push(latest.to_string());
    keep.push(guid.to_string());
    deploy::prune(&keep);
    Ok(())
}

async fn ensure_installed(app: &AppHandle) -> Result<String, String> {
    let latest = deploy::latest(&settings::load().channel)
        .await?
        .client_version_upload;
    ensure_version(app, &latest, &latest).await?;
    Ok(latest)
}

async fn start_client(
    app: &AppHandle,
    running: &launch::Running,
    arg: &str,
    user_id: Option<u64>,
    label: String,
    profile_id: Option<String>,
) -> Result<u32, String> {
    let profile = profiles::resolve(profile_id.as_deref());
    let latest = deploy::latest(&settings::load().channel)
        .await?
        .client_version_upload;
    let guid = profile
        .pinned_version
        .clone()
        .unwrap_or_else(|| latest.clone());
    ensure_version(app, &guid, &latest).await?;
    let p = profile.clone();
    let rebuild_blocked = running.any_for_profile(&profile.id);
    let dir = tokio::task::spawn_blocking(move || builder::build(&p, &guid))
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| {
            if rebuild_blocked {
                format!("{e} (another client is using this instance)")
            } else {
                e
            }
        })?;
    if settings::load().multi_instance {
        launch::hold_singleton_mutex();
    }
    let pid = running.add(
        launch::start(&dir, arg)?,
        user_id,
        label,
        profile.id.clone(),
    );
    profiles::record_launch(&profile.id);
    Ok(pid)
}

#[tauri::command]
async fn install(app: AppHandle) -> Result<String, String> {
    ensure_installed(&app).await
}

#[tauri::command]
async fn launch(
    app: AppHandle,
    running: State<'_, launch::Running>,
    arg: Option<String>,
    profile_id: Option<String>,
) -> Result<u32, String> {
    let label = if arg.is_some() {
        "From website"
    } else {
        "Roblox"
    };
    start_client(
        &app,
        &running,
        arg.as_deref().unwrap_or("--app"),
        None,
        label.into(),
        profile_id,
    )
    .await
}

#[tauri::command]
async fn launch_account(
    app: AppHandle,
    running: State<'_, launch::Running>,
    user_id: u64,
    target: roblox::Target,
    profile_id: Option<String>,
) -> Result<u32, String> {
    let cookie = accounts::cookie(user_id)?;
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
    let tracker = (now.as_nanos() % 1_000_000_000_000) as u64;
    let placelauncher = roblox::placelauncher(&target, tracker).await?;
    let ticket = roblox::auth_ticket(&cookie).await?;
    let uri = roblox::launch_uri(&ticket, placelauncher.as_deref(), now.as_millis(), tracker);
    let label = match &target {
        roblox::Target::App => "Roblox app".to_string(),
        roblox::Target::Place { place, .. } => {
            format!("Place {}", roblox::parse_place(place).unwrap_or_default())
        }
        roblox::Target::Follow { username } => format!("Following {}", username.trim()),
    };
    let for_game = match &target {
        roblox::Target::Place { place, .. } => {
            roblox::parse_place(place).and_then(profiles::game_profile)
        }
        _ => None,
    };
    let profile_id = profile_id
        .or(for_game)
        .or_else(|| accounts::profile_of(user_id));
    recovery::remember(user_id, &target);
    start_client(&app, &running, &uri, Some(user_id), label, profile_id).await
}

#[derive(Serialize)]
struct RunningInfo {
    running: Vec<launch::Instance>,
    exited: Vec<launch::Exited>,
}

#[tauri::command]
fn list_running(
    running: State<'_, launch::Running>,
    watcher: State<'_, logwatch::Watcher>,
) -> RunningInfo {
    let (mut alive, exited) = running.poll();
    for e in &exited {
        profiles::record_exit(&e.profile_id, &e.label, e.secs, e.crashed);
        if let Some(u) = e.user_id {
            watcher.forget(u);
        }
    }
    let sessions = watcher.sessions();
    for c in &mut alive {
        c.game = c.user_id.and_then(|u| sessions.get(&u).cloned());
    }
    RunningInfo {
        running: alive,
        exited,
    }
}

#[tauri::command]
async fn catalog_categories() -> Result<serde_json::Value, String> {
    catalog::categories().await
}

#[tauri::command]
async fn search_catalog(
    keyword: String,
    category: u64,
    subcategory: Option<u64>,
    sort: u8,
    cursor: Option<String>,
) -> Result<catalog::Page, String> {
    catalog::search(&keyword, category, subcategory, sort, cursor).await
}

#[tauri::command]
async fn avatar_rules() -> Result<serde_json::Value, String> {
    avatar::rules().await
}

#[tauri::command]
async fn get_avatar(user_id: u64) -> Result<avatar::Avatar, String> {
    avatar::current(&accounts::cookie(user_id)?).await
}

#[tauri::command]
async fn render_avatar(user_id: u64) -> Result<String, String> {
    avatar::render(user_id).await
}

#[tauri::command]
async fn avatar_inventory(
    user_id: u64,
    asset_type: u64,
    cursor: Option<String>,
) -> Result<avatar::OwnedPage, String> {
    avatar::inventory(user_id, &accounts::cookie(user_id)?, asset_type, cursor).await
}

#[tauri::command]
async fn avatar_owns(user_id: u64, item_type: String, id: u64) -> Result<bool, String> {
    avatar::owns(user_id, &accounts::cookie(user_id)?, &item_type, id).await
}

#[tauri::command]
async fn avatar_outfits(user_id: u64) -> Result<Vec<avatar::Outfit>, String> {
    avatar::outfits(user_id, &accounts::cookie(user_id)?).await
}

#[tauri::command]
async fn avatar_wear(user_id: u64, asset_ids: Vec<u64>) -> Result<Vec<u64>, String> {
    let c = accounts::cookie(user_id)?;
    let bad = avatar::wear(&c, &asset_ids).await?;
    avatar::redraw(&c).await;
    Ok(bad)
}

#[tauri::command]
async fn avatar_wear_outfit(user_id: u64, outfit_id: u64) -> Result<Vec<u64>, String> {
    let c = accounts::cookie(user_id)?;
    let bad = avatar::wear_outfit(&c, outfit_id).await?;
    avatar::redraw(&c).await;
    Ok(bad)
}

#[tauri::command]
async fn avatar_set_colors(user_id: u64, colors: avatar::BodyColors) -> Result<(), String> {
    let c = accounts::cookie(user_id)?;
    avatar::set_body_colors(&c, &colors).await?;
    avatar::redraw(&c).await;
    Ok(())
}

#[tauri::command]
async fn avatar_set_scales(user_id: u64, scales: avatar::Scales) -> Result<(), String> {
    let c = accounts::cookie(user_id)?;
    avatar::set_scales(&c, &scales).await?;
    avatar::redraw(&c).await;
    Ok(())
}

#[tauri::command]
async fn avatar_set_type(user_id: u64, avatar_type: String) -> Result<(), String> {
    let c = accounts::cookie(user_id)?;
    avatar::set_type(&c, &avatar_type).await?;
    avatar::redraw(&c).await;
    Ok(())
}

#[tauri::command]
fn overlay_target() -> Option<u32> {
    overlay::target()
}

#[tauri::command]
fn overlay_pick(pid: u32) {
    overlay::retarget(pid)
}

#[tauri::command]
fn overlay_hide(app: AppHandle) {
    overlay::hide(&app, true)
}

#[tauri::command]
fn set_client_priority(
    running: State<'_, launch::Running>,
    pid: u32,
    level: String,
) -> Result<(), String> {
    running.set_priority(pid, &level)
}

fn register_hotkey(app: &AppHandle, s: &settings::Settings) -> Result<(), String> {
    use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut};
    let gs = app.global_shortcut();
    let _ = gs.unregister_all();
    if !s.overlay_enabled {
        return Ok(());
    }
    let shortcut: Shortcut = s
        .overlay_hotkey
        .parse()
        .map_err(|_| format!("Umbra can't use \"{}\" as a hotkey.", s.overlay_hotkey))?;
    gs.register(shortcut).map_err(|e| {
        format!(
            "Couldn't use {} as the overlay hotkey. Another app may already use it. ({e})",
            s.overlay_hotkey
        )
    })
}

#[tauri::command]
fn set_overlay_hotkey(app: AppHandle, hotkey: String, enabled: bool) -> Result<(), String> {
    let mut s = settings::load();
    let old = (s.overlay_hotkey.clone(), s.overlay_enabled);
    s.overlay_hotkey = hotkey.trim().to_string();
    s.overlay_enabled = enabled;
    if let Err(e) = register_hotkey(&app, &s) {
        s.overlay_hotkey = old.0;
        s.overlay_enabled = old.1;
        let _ = register_hotkey(&app, &s);
        return Err(e);
    }
    settings::save(&s)
}

#[tauri::command]
async fn move_client(
    app: AppHandle,
    running: State<'_, launch::Running>,
    watcher: State<'_, logwatch::Watcher>,
    pid: u32,
    mode: String,
    place_id: Option<u64>,
    job_id: Option<String>,
) -> Result<u32, String> {
    let (user_id, profile_id) = running.owner(pid).ok_or("That client already closed.")?;
    let user_id = user_id.ok_or(
        "This client was started without a saved account, so Umbra can't move it. Launch it from an account instead.",
    )?;
    let session = watcher.sessions().get(&user_id).cloned();
    let (place, job) = match mode.as_str() {
        "rejoin" => {
            let s = session.ok_or("This client isn't in a game yet.")?;
            (s.place_id, s.job_id)
        }
        "hop" => {
            let s = session.ok_or("This client isn't in a game yet.")?;
            let page = discover::servers(s.place_id, true, true, None).await?;
            let open: Vec<_> = page
                .servers
                .iter()
                .filter(|x| Some(&x.id) != s.job_id.as_ref() && x.playing < x.max_players)
                .take(10)
                .collect();
            if open.is_empty() {
                return Err("There are no other open servers right now.".into());
            }
            let n = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .subsec_nanos() as usize;
            (s.place_id, Some(open[n % open.len()].id.clone()))
        }
        "to" => (place_id.ok_or("Pick a game to move to.")?, job_id),
        _ => return Err("Unknown move.".into()),
    };
    running.kill_and_wait(pid)?;
    watcher.forget(user_id);
    tokio::time::sleep(Duration::from_millis(1200)).await;
    let new = launch_account(
        app,
        running,
        user_id,
        roblox::Target::Place {
            place: place.to_string(),
            job_id: job,
        },
        Some(profile_id),
    )
    .await?;
    overlay::retarget(new);
    Ok(new)
}

#[tauri::command]
fn get_autostart() -> bool {
    launch::autostart()
}

#[tauri::command]
fn set_autostart(on: bool) -> Result<(), String> {
    launch::set_autostart(on)
}

#[tauri::command]
fn discord_status(state: State<'_, DiscordState>) -> Option<String> {
    state.0.lock().unwrap().1.clone()
}

struct DiscordState(std::sync::Mutex<(discord::Discord, Option<String>)>);

fn unix_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

async fn auto_rejoin(
    app: AppHandle,
    user_id: u64,
    session: Option<logwatch::Session>,
    reason: String,
) {
    let s = session.clone().unwrap_or(logwatch::Session {
        user_id,
        ..Default::default()
    });
    let Some(attempt) = recovery::allow(user_id, unix_now()) else {
        let _ = app.emit("auto-rejoin", recovery::record(&s, &reason, "paused"));
        return;
    };
    let Some(target) = recovery::target_for(user_id, session.as_ref(), attempt) else {
        let _ = app.emit("auto-rejoin", recovery::record(&s, &reason, "unknown"));
        return;
    };
    let _ = app.emit("auto-rejoin", recovery::record(&s, &reason, "rejoining"));
    tokio::time::sleep(Duration::from_secs(4)).await;
    let running = app.state::<launch::Running>();
    let profile = running.pid_of(user_id).and_then(|pid| {
        let p = running.owner(pid).map(|o| o.1);
        let _ = running.kill_and_wait(pid);
        p
    });
    app.state::<logwatch::Watcher>().forget(user_id);
    tokio::time::sleep(Duration::from_millis(1200)).await;
    if let Err(e) = launch_account(app.clone(), app.state(), user_id, target, profile).await {
        let _ = app.emit(
            "auto-rejoin",
            recovery::record(&s, &format!("{reason}. Rejoin failed: {e}"), "failed"),
        );
    }
}

fn recover(app: &AppHandle, watcher: &logwatch::Watcher) {
    let running = app.state::<launch::Running>();
    for e in running.reap() {
        if let (true, Some(u)) = (e.crashed, e.user_id) {
            if accounts::auto_rejoin_of(u) {
                tauri::async_runtime::spawn(auto_rejoin(
                    app.clone(),
                    u,
                    None,
                    "Roblox crashed".into(),
                ));
            }
        }
    }
    let ours = running.alive_users();
    for (session, reason) in watcher.take_drops() {
        let code = logwatch::reason_code(&reason);
        if recovery::user_left(code) || !ours.contains(&session.user_id) {
            continue;
        }
        if accounts::auto_rejoin_of(session.user_id) {
            tauri::async_runtime::spawn(auto_rejoin(
                app.clone(),
                session.user_id,
                Some(session),
                reason,
            ));
        } else {
            let _ = app.emit(
                "client-dropped",
                recovery::record(&session, &reason, "none"),
            );
        }
    }
}

fn window_tools(app: &AppHandle, s: &settings::Settings, tick: u64) {
    let running = app.state::<launch::Running>();
    if s.label_windows && tick % 5 == 0 {
        let names = accounts::display_names();
        for c in running.snapshot() {
            if let Some(n) = c.user_id.and_then(|u| names.get(&u)) {
                winctl::label(c.pid, &format!("Roblox \u{b7} {n}"));
            }
        }
    }
    if s.trim_background && tick % 60 == 0 {
        running.trim_background();
    }
    let clients = running.mute_states();
    let pids: Vec<u32> = clients.iter().map(|c| c.0).collect();
    let fg = overlay::foreground_pid();
    let mut last = LAST_FOCUSED.lock().unwrap();
    if pids.contains(&fg) {
        *last = Some(fg);
    }
    let focused = last.filter(|p| pids.contains(p));
    drop(last);
    for (pid, muted) in audio::desired(&clients, focused, s.mute_background) {
        audio::ensure(pid, muted, tick % 5 == 0);
    }
    audio::forget_except(&pids);
}

static LAST_FOCUSED: std::sync::Mutex<Option<u32>> = std::sync::Mutex::new(None);

fn background_loop(app: AppHandle) {
    let since = SystemTime::now();
    let mut tick: u64 = 0;
    loop {
        std::thread::sleep(Duration::from_secs(2));
        tick += 1;
        let s = settings::load();
        let watcher = app.state::<logwatch::Watcher>();
        watcher.scan(since);
        tauri::async_runtime::block_on(watcher.enrich(s.server_location));
        recover(&app, &watcher);
        window_tools(&app, &s, tick);

        let state = app.state::<DiscordState>();
        let mut guard = state.0.lock().unwrap();
        let (dc, status) = &mut *guard;
        if !s.discord_enabled || s.discord_app_id.trim().is_empty() {
            dc.disconnect();
            *status = None;
            continue;
        }
        let ours = app.state::<launch::Running>().alive_users();
        let sessions = watcher.sessions();
        let current = ours
            .iter()
            .filter_map(|u| sessions.get(u))
            .max_by_key(|x| x.joined_at);
        let activity = current.map(|x| discord::activity(x, s.discord_join_button));
        let playing = activity.is_some();
        *status = Some(match dc.set(s.discord_app_id.trim(), activity) {
            Ok(()) if playing => "Connected, showing your game".into(),
            Ok(()) => "Connected, waiting for a game".into(),
            Err(e) => e,
        });
    }
}

fn show_main(app: &AppHandle) {
    if let Some(w) = app.get_webview_window("main") {
        let _ = w.show();
        let _ = w.unminimize();
        let _ = w.set_focus();
    }
}

#[tauri::command]
fn set_account_profile(user_id: u64, profile_id: Option<String>) -> Result<(), String> {
    accounts::set_profile(user_id, profile_id)
}

#[tauri::command]
async fn get_spending(user_id: u64, tz_offset_min: i32) -> Result<spending::Spending, String> {
    let cookie = accounts::cookie(user_id)?;
    spending::load(user_id, &cookie, unix_now(), tz_offset_min as i64 * 60).await
}

#[tauri::command]
fn set_client_mute(
    running: State<'_, launch::Running>,
    pid: u32,
    muted: bool,
) -> Result<(), String> {
    running.set_muted(pid, muted)?;
    audio::set_mute(pid, muted).map(|_| ())
}

#[tauri::command]
fn set_account_rejoin(user_id: u64, on: bool) -> Result<(), String> {
    accounts::set_auto_rejoin(user_id, on)
}

#[tauri::command]
fn recent_disconnects() -> Vec<recovery::Disconnect> {
    recovery::recent()
}

#[tauri::command]
fn trim_client(running: State<'_, launch::Running>, pid: u32) -> Result<u64, String> {
    running.trim(pid)
}

#[tauri::command]
fn arrange_windows(running: State<'_, launch::Running>) -> usize {
    let pids: Vec<u32> = running.snapshot().iter().map(|c| c.pid).collect();
    winctl::arrange(&pids)
}

#[tauri::command]
async fn games_updated(
    universe_ids: Vec<u64>,
) -> Result<std::collections::HashMap<u64, String>, String> {
    extras::games_updated(&universe_ids).await
}

#[tauri::command]
async fn get_stats(user_id: Option<u64>, tz_offset_min: i32) -> Result<stats::Stats, String> {
    let visits = tokio::task::spawn_blocking(personal::history)
        .await
        .map_err(|e| e.to_string())?;
    let mut s = stats::compute(&visits, user_id, unix_now(), tz_offset_min as i64 * 60);
    let ids: Vec<u64> = s.games.iter().map(|g| g.universe_id).collect();
    let cards = personal::infos(&ids)
        .await
        .into_iter()
        .map(|(k, v)| (k, v.card))
        .collect();
    stats::name_games(&mut s, &cards);
    Ok(s)
}

#[tauri::command]
fn bridge_answer(id: String, allow: bool) -> Result<(), String> {
    bridge::answer(&id, allow)
}

#[tauri::command]
fn bridge_grants() -> Vec<serde_json::Value> {
    bridge::grants()
}

#[tauri::command]
fn bridge_revoke(id: Option<String>) -> Result<(), String> {
    bridge::revoke(id.as_deref())
}

#[tauri::command]
fn list_profiles() -> profiles::Store {
    profiles::load()
}

#[tauri::command]
fn create_profile(name: String) -> Result<profiles::Profile, String> {
    profiles::create(&name)
}

#[tauri::command]
fn save_profile(profile: profiles::Profile) -> Result<profiles::Profile, String> {
    profiles::update(profile)
}

#[tauri::command]
fn delete_profile(running: State<'_, launch::Running>, id: String) -> Result<(), String> {
    if running.any_for_profile(&id) {
        return Err("Close the Roblox clients using this instance first.".into());
    }
    profiles::delete(&id)
}

#[tauri::command]
async fn get_badges(universe_id: u64) -> Result<Vec<extras::Badge>, String> {
    extras::badges(universe_id).await
}

#[tauri::command]
async fn get_passes(universe_id: u64) -> Result<Vec<extras::Pass>, String> {
    extras::passes(universe_id).await
}

#[tauri::command]
async fn get_flag_allowlist() -> Result<Vec<String>, String> {
    extras::flag_allowlist().await
}

#[tauri::command]
fn list_presets() -> Vec<extras::Preset> {
    extras::presets()
}

#[tauri::command]
fn save_preset(preset: extras::Preset) -> Result<extras::Preset, String> {
    extras::save_preset(preset)
}

#[tauri::command]
fn delete_preset(id: String) -> Result<(), String> {
    extras::delete_preset(&id)
}

#[tauri::command]
fn set_game_profile(place_id: u64, profile_id: Option<String>) -> Result<(), String> {
    profiles::set_game(place_id, profile_id)
}

#[tauri::command]
fn set_default_profile(id: String) -> Result<(), String> {
    profiles::set_default(&id)
}

#[tauri::command]
fn list_mods() -> Vec<mods::Mod> {
    mods::list()
}

#[tauri::command]
async fn import_mod(path: String) -> Result<mods::Mod, String> {
    tokio::task::spawn_blocking(move || mods::import(std::path::Path::new(&path)))
        .await
        .map_err(|e| e.to_string())?
}

#[tauri::command]
fn delete_mod(id: String) -> Result<(), String> {
    mods::delete(&id)
}

#[tauri::command]
fn import_font(path: String) -> Result<String, String> {
    mods::import_font(std::path::Path::new(&path))
}

#[tauri::command]
fn close_running(running: State<'_, launch::Running>, pid: u32) -> Result<(), String> {
    running.kill(pid)
}

#[tauri::command]
async fn list_accounts() -> Vec<accounts::Account> {
    accounts::list().await
}

#[tauri::command]
fn remove_account(user_id: u64) -> Result<(), String> {
    accounts::remove(user_id)
}

#[tauri::command]
async fn get_presence() -> Result<Vec<roblox::Presence>, String> {
    let ids = accounts::ids();
    match accounts::any_cookie() {
        Some(c) if !ids.is_empty() => roblox::presence(&c, &ids).await,
        _ => Ok(vec![]),
    }
}

#[tauri::command]
async fn add_account(app: AppHandle) -> Result<accounts::Account, String> {
    if let Some(w) = app.get_webview_window("login") {
        let _ = w.set_focus();
        return Err("The login window is already open.".into());
    }
    let login_url: tauri::Url = "https://www.roblox.com/login".parse().unwrap();
    let win = WebviewWindowBuilder::new(&app, "login", WebviewUrl::External(login_url))
        .title("Log in to Roblox · Umbra")
        .inner_size(480.0, 760.0)
        .center()
        .incognito(true)
        .build()
        .map_err(|e| e.to_string())?;
    let site: tauri::Url = "https://www.roblox.com".parse().unwrap();
    loop {
        tokio::time::sleep(Duration::from_millis(800)).await;
        if app.get_webview_window("login").is_none() {
            return Err("Login cancelled.".into());
        }

        let (w, url) = (win.clone(), site.clone());
        let Ok(Ok(cookies)) = tokio::task::spawn_blocking(move || w.cookies_for_url(url)).await
        else {
            continue;
        };
        if let Some(c) = cookies
            .iter()
            .find(|c| c.name() == ".ROBLOSECURITY" && !c.value().is_empty())
        {
            let value = c.value().to_string();
            let _ = win.close();
            return accounts::add(&value).await;
        }
    }
}

#[tauri::command]
async fn get_charts() -> Result<Vec<discover::Chart>, String> {
    discover::charts(24).await
}

#[tauri::command]
async fn get_banners(universe_ids: Vec<u64>) -> std::collections::HashMap<u64, String> {
    discover::banners(&universe_ids).await
}

#[tauri::command]
async fn search_games(query: String) -> Result<Vec<discover::GameCard>, String> {
    discover::search(&query).await
}

#[tauri::command]
async fn get_game(universe_id: u64) -> Result<discover::GameDetail, String> {
    discover::game(universe_id).await
}

#[tauri::command]
async fn get_servers(
    place_id: u64,
    most_players: bool,
    exclude_full: bool,
    cursor: Option<String>,
) -> Result<discover::ServerPage, String> {
    discover::servers(place_id, most_players, exclude_full, cursor).await
}

#[tauri::command]
async fn get_friends_playing(user_id: u64) -> Result<Vec<discover::FriendGame>, String> {
    discover::friends_playing(user_id, &accounts::cookie(user_id)?).await
}

#[tauri::command]
async fn get_friends(user_id: u64) -> Result<Vec<players::Friend>, String> {
    players::friends(user_id, &accounts::cookie(user_id)?).await
}

#[tauri::command]
async fn search_players(keyword: String) -> Result<Vec<players::Player>, String> {
    players::search(&keyword).await
}

#[tauri::command]
async fn get_player(user_id: u64, as_user: Option<u64>) -> Result<players::Profile, String> {
    let cookie = as_user
        .and_then(|u| accounts::cookie(u).ok())
        .or_else(accounts::any_cookie);
    players::profile(user_id, cookie).await
}

#[tauri::command]
fn list_favorites() -> Vec<favorites::Favorite> {
    favorites::list()
}

#[tauri::command]
fn toggle_favorite(favorite: favorites::Favorite) -> Result<bool, String> {
    favorites::toggle(favorite)
}

#[tauri::command]
async fn get_collection(user_id: u64) -> Result<trades::Collection, String> {
    trades::collection(user_id, accounts::cookie(user_id).ok().as_deref()).await
}

#[tauri::command]
async fn get_trades(user_id: u64, kind: String) -> Result<Vec<trades::TradeSummary>, String> {
    trades::list(&accounts::cookie(user_id)?, &kind).await
}

#[tauri::command]
async fn get_trade(user_id: u64, trade_id: u64) -> Result<trades::TradeDetail, String> {
    trades::detail(user_id, &accounts::cookie(user_id)?, trade_id).await
}

#[tauri::command]
async fn decline_trade(user_id: u64, trade_id: u64) -> Result<(), String> {
    trades::decline(&accounts::cookie(user_id)?, trade_id).await
}

#[tauri::command]
async fn get_item_values(
    asset_ids: Vec<u64>,
) -> Result<std::collections::HashMap<u64, trades::ItemValue>, String> {
    let all = trades::values().await?;
    Ok(asset_ids
        .into_iter()
        .filter_map(|id| Some((id, all.get(&id)?.clone())))
        .collect())
}

#[tauri::command]
async fn get_for_you(user_id: Option<u64>) -> Result<Option<personal::ForYou>, String> {
    let id = match user_id {
        Some(id) => id,
        None => match tokio::task::spawn_blocking(personal::local_players)
            .await
            .map_err(|e| e.to_string())?
            .first()
        {
            Some(id) => *id,
            None => return Ok(None),
        },
    };
    personal::for_you(id, accounts::cookie(id).ok())
        .await
        .map(Some)
}

#[tauri::command]
fn get_settings() -> settings::Settings {
    settings::load()
}

#[tauri::command]
fn set_settings(settings: settings::Settings) -> Result<(), String> {
    settings::save(&settings)
}

#[tauri::command]
fn register_protocol() -> Result<(), String> {
    launch::register_protocol()
}

#[tauri::command]
fn open_data_folder() -> Result<(), String> {
    let _ = std::fs::create_dir_all(paths::base());
    std::process::Command::new("explorer")
        .arg(paths::base())
        .spawn()
        .map(|_| ())
        .map_err(|e| e.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let uri = std::env::args()
        .nth(1)
        .filter(|a| a.starts_with("roblox-player:") || a.starts_with("roblox:"));
    let start_hidden = std::env::args().any(|a| a == "--tray");

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init())
        .manage(launch::Running::default())
        .manage(logwatch::Watcher::default())
        .manage(DiscordState(std::sync::Mutex::new((
            discord::Discord::default(),
            None,
        ))))
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(|app, _shortcut, event| {
                    if event.state() == tauri_plugin_global_shortcut::ShortcutState::Pressed {
                        overlay::toggle(app, &app.state::<launch::Running>());
                    }
                })
                .build(),
        )
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::Focused(false) = event {
                if window.label() == "overlay" {
                    overlay::hide(window.app_handle(), false);
                }
            }
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                if window.label() == "main" && settings::load().close_to_tray {
                    api.prevent_close();
                    let _ = window.hide();
                }
            }
        })
        .setup(move |app| {
            use tauri::menu::{Menu, MenuItem};
            use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
            let open = MenuItem::with_id(app, "open", "Open Umbra", true, None::<&str>)?;
            let quit = MenuItem::with_id(app, "quit", "Quit Umbra", true, None::<&str>)?;
            TrayIconBuilder::with_id("umbra")
                .icon(app.default_window_icon().cloned().expect("app icon"))
                .tooltip("Umbra")
                .menu(&Menu::with_items(app, &[&open, &quit])?)
                .show_menu_on_left_click(false)
                .on_menu_event(|app, e| match e.id().as_ref() {
                    "open" => show_main(app),
                    "quit" => app.exit(0),
                    _ => {}
                })
                .on_tray_icon_event(|tray, e| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = e
                    {
                        show_main(tray.app_handle());
                    }
                })
                .build(app)?;
            if start_hidden {
                if let Some(w) = app.get_webview_window("main") {
                    let _ = w.hide();
                }
            }
            let handle = app.handle().clone();
            std::thread::spawn(move || background_loop(handle));
            let _ = overlay::create(app.handle());
            if let Err(e) = register_hotkey(app.handle(), &settings::load()) {
                let _ = app.handle().emit("launch-error", e);
            }

            if uri.is_none() {
                bridge::start(app.handle().clone());
            }
            if let Some(uri) = uri {
                let handle = app.handle().clone();
                tauri::async_runtime::spawn(async move {
                    let running = handle.state::<launch::Running>();
                    match start_client(&handle, &running, &uri, None, "From website".into(), None)
                        .await
                    {
                        Ok(_) => handle.exit(0),
                        Err(e) => {
                            let _ = handle.emit("launch-error", e);
                        }
                    }
                });
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_status,
            install,
            launch,
            launch_account,
            list_running,
            close_running,
            list_accounts,
            remove_account,
            get_presence,
            add_account,
            set_account_profile,
            get_charts,
            get_banners,
            search_games,
            get_game,
            get_servers,
            get_friends_playing,
            get_friends,
            search_players,
            get_player,
            list_favorites,
            toggle_favorite,
            get_for_you,
            get_collection,
            get_trades,
            get_trade,
            decline_trade,
            get_item_values,
            list_profiles,
            create_profile,
            save_profile,
            delete_profile,
            set_default_profile,
            set_game_profile,
            bridge_answer,
            set_account_rejoin,
            set_client_mute,
            get_spending,
            recent_disconnects,
            trim_client,
            arrange_windows,
            games_updated,
            get_stats,
            bridge_grants,
            bridge_revoke,
            get_badges,
            get_passes,
            list_presets,
            get_flag_allowlist,
            save_preset,
            delete_preset,
            list_mods,
            import_mod,
            delete_mod,
            import_font,
            get_autostart,
            overlay_target,
            overlay_pick,
            overlay_hide,
            set_client_priority,
            set_overlay_hotkey,
            move_client,
            set_autostart,
            discord_status,
            catalog_categories,
            search_catalog,
            avatar_rules,
            get_avatar,
            render_avatar,
            avatar_inventory,
            avatar_owns,
            avatar_outfits,
            avatar_wear,
            avatar_wear_outfit,
            avatar_set_colors,
            avatar_set_scales,
            avatar_set_type,
            get_settings,
            set_settings,
            register_protocol,
            open_data_folder
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
