// src-tauri/src/lib.rs
mod config;
mod vita;
mod discord;
mod covers;

use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::{
    AppHandle, Manager, State, Emitter,
    menu::{Menu, MenuItem},
    tray::{TrayIconBuilder, TrayIconEvent},
};
use serde::{Deserialize, Serialize};
use tokio::sync::watch;

// ── State ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct AppState {
    pub status:            String,
    pub current_title:     String,
    pub current_title_id:  String,
    pub platform:          String,
    pub discord_connected: bool,
    pub status_message:    String,
    pub cover_url:         Option<String>,
    pub cover_source:      String,
}

impl AppState {
    fn disconnected() -> Self {
        Self {
            status:         "disconnected".into(),
            status_message: "Desconectado".into(),
            ..Default::default()
        }
    }
}

struct Inner {
    discord:       discord::DiscordRpc,
    app_state:     AppState,
    running:       bool,
    stop_tx:       Option<watch::Sender<bool>>,
    last_title_id: String,
    game_start:    Option<i64>,
    last_seen:     Option<std::time::Instant>,
    manual_update: bool,
    poll_count:    u32,
}

impl Inner {
    fn new() -> Self {
        Self {
            discord:       discord::DiscordRpc::new(String::new()),
            app_state:     AppState::disconnected(),
            running:       false,
            stop_tx:       None,
            last_title_id: String::new(),
            game_start:    None,
            last_seen:     None,
            manual_update: true,
            poll_count:    0,
        }
    }
}

pub struct VitaState(pub Mutex<Inner>);

// ── Helpers ────────────────────────────────────────────────────────────

fn now_ts() -> i64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs() as i64
}

fn fmt_time() -> String {
    let s = now_ts() as u64;
    format!("{:02}:{:02}:{:02}", (s % 86400) / 3600, (s % 3600) / 60, s % 60)
}

fn emit_state(app: &AppHandle, s: &AppState) {
    app.emit("state-update", s).ok();
}

fn emit_log(app: &AppHandle, msg: &str) {
    app.emit("log", format!("[{}] {}", fmt_time(), msg)).ok();
}

fn with_state<F, R>(app: &AppHandle, f: F) -> R
where
    F: FnOnce(&mut Inner) -> R,
{
    let state = app.state::<VitaState>();
    let mut inner = state.inner().0.lock().unwrap();
    f(&mut inner)
}

// ── Commands ───────────────────────────────────────────────────────────

#[tauri::command]
fn get_config() -> config::Config {
    config::load()
}

#[tauri::command]
fn save_config(cfg: config::Config) -> Result<(), String> {
    let errs = cfg.validate();
    if !errs.is_empty() { return Err(errs.join(", ")); }
    covers::clear_cache();
    config::save(&cfg)
}

#[tauri::command]
fn get_state(state: State<VitaState>) -> AppState {
    state.0.lock().unwrap().app_state.clone()
}

#[tauri::command]
fn get_running(state: State<VitaState>) -> bool {
    state.0.lock().unwrap().running
}

#[tauri::command]
async fn connect(app: AppHandle, state: State<'_, VitaState>) -> Result<(), String> {
    let cfg = config::load();
    if let Some(e) = cfg.validate().into_iter().next() { return Err(e); }

    let (stop_tx, stop_rx) = watch::channel(false);

    // Connect Discord + update state (lock scope — dropped before spawn)
    {
        let mut inner = state.0.lock().unwrap();
        if inner.running { return Ok(()); }

        inner.discord = discord::DiscordRpc::new(cfg.client_id.clone());
        inner.discord.connect().map_err(|e| {
            emit_log(&app, &format!("❌ Discord: {}", e));
            e
        })?;

        inner.running       = true;
        inner.stop_tx       = Some(stop_tx);
        inner.manual_update = true;
        inner.app_state = AppState {
            status:            "connecting".into(),
            status_message:    "Conectando ao Vita...".into(),
            discord_connected: true,
            ..Default::default()
        };
        emit_log(&app, "✅ Discord RPC conectado");
        emit_state(&app, &inner.app_state);
    } // <- lock dropped here

    let app2 = app.clone();
    tokio::spawn(async move { poll_loop(app2, stop_rx).await; });
    Ok(())
}

#[tauri::command]
async fn disconnect(app: AppHandle, state: State<'_, VitaState>) -> Result<(), String> {
    let mut inner = state.0.lock().unwrap();
    if let Some(tx) = inner.stop_tx.take() { tx.send(true).ok(); }
    inner.running       = false;
    inner.last_title_id = String::new();
    inner.game_start    = None;
    inner.last_seen     = None;
    inner.manual_update = true;
    inner.discord.disconnect();
    inner.app_state = AppState::disconnected();
    let s = inner.app_state.clone();
    drop(inner); // explicit drop before emit
    emit_state(&app, &s);
    emit_log(&app, "⏹ Desconectado");
    Ok(())
}

#[tauri::command]
fn hide_window(app: AppHandle) {
    if let Some(w) = app.get_webview_window("main") { w.hide().ok(); }
}

// ── Poll loop ──────────────────────────────────────────────────────────

async fn poll_loop(app: AppHandle, mut stop_rx: watch::Receiver<bool>) {
    let cfg = config::load();
    emit_log(&app, &format!("▶ Monitorando {} ({}s)", cfg.ip, cfg.update_interval));

    loop {
        if *stop_rx.borrow() { break; }

        let cfg = config::load();
        let ip  = resolve_ip(&cfg.ip).await;

        match ip {
            None => set_connecting(&app, "Resolvendo endereço..."),
            Some(ref ip) => match vita::poll(ip).await {
                Err(e)      => on_disconnect(&app, &e).await,
                Ok(None)    => on_disconnect(&app, "Packet inválido").await,
                Ok(Some(t)) => on_title(&app, &cfg, t).await,
            }
        }

        tokio::select! {
            _ = tokio::time::sleep(tokio::time::Duration::from_secs(cfg.update_interval)) => {}
            _ = stop_rx.changed() => { break; }
        }
    }
}

fn set_connecting(app: &AppHandle, msg: &str) {
    with_state(app, |inner| {
        inner.app_state.status         = "connecting".into();
        inner.app_state.status_message = msg.into();
    });
    let s = with_state(app, |inner| inner.app_state.clone());
    emit_state(app, &s);
}

async fn on_title(app: &AppHandle, cfg: &config::Config, title: vita::TitleInfo) {
    // --- Step 1: compute changes under lock, then DROP lock ---
    let (changed, should_update, start_time) = {
        let state = app.state::<VitaState>();
        let mut inner = state.inner().0.lock().unwrap();

        inner.last_seen = Some(std::time::Instant::now());
        let changed     = inner.last_title_id != title.title_id;

        if changed {
            emit_log(app, &format!("🎮 {} → {}", inner.last_title_id, title.display));
            inner.last_title_id  = title.title_id.clone();
            inner.game_start     = if cfg.display_timer { Some(now_ts()) } else { None };
            inner.manual_update  = true;
            inner.poll_count     = 0;
        }
        inner.poll_count += 1;
        let should  = inner.manual_update || changed || inner.poll_count % 5 == 0;
        let start   = inner.game_start;
        (changed, should, start)
        // lock dropped here
    };

    // --- Step 2: handle LiveArea suppression ---
    if title.is_live_area && !cfg.display_main_menu {
        let new_state = {
            let state = app.state::<VitaState>();
            let mut inner = state.inner().0.lock().unwrap();
            inner.discord.clear_presence().ok();
            inner.manual_update = false;
            inner.app_state = AppState {
                status:            "connected".into(),
                status_message:    "Conectado (LiveArea oculta)".into(),
                discord_connected: true,
                ..Default::default()
            };
            inner.app_state.clone()
            // lock dropped
        };
        emit_state(app, &new_state);
        return;
    }

    if !should_update {
        let s = with_state(app, |inner| inner.app_state.clone());
        emit_state(app, &s);
        return;
    }

    // --- Step 3: resolve cover ASYNC (no lock held) ---
    let cover = if !title.is_live_area && title.title_id != "XMB" {
        let icon = if cfg.custom_icon_url.is_empty() { None } else { Some(cfg.custom_icon_url.as_str()) };
        let id   = if cfg.igdb_client_id.is_empty()  { None } else { Some(cfg.igdb_client_id.as_str()) };
        let sec  = if cfg.igdb_client_secret.is_empty() { None } else { Some(cfg.igdb_client_secret.as_str()) };
        covers::resolve(&title.display, &title.title_id, icon, id, sec).await
    } else {
        None
    };

    let cover_source = cover.as_ref().map(|u| {
        if u.contains("igdb.com") { "IGDB".to_string() } else { "GitHub".to_string() }
    }).unwrap_or_default();

    if cover.is_some() {
        emit_log(app, &format!("🖼 Capa via {}", cover_source));
    }

    let state_str = if !cfg.state.is_empty() { cfg.state.clone() } else { title.state_text.clone() };

    // --- Step 4: set Discord presence under lock, then DROP lock ---
    let (new_state, failed) = {
        let state = app.state::<VitaState>();
        let mut inner = state.inner().0.lock().unwrap();

        let ok = inner.discord.set_presence(
            &title.display,
            if state_str.is_empty() { None } else { Some(&state_str) },
            start_time,
            cover.as_deref(),
        );

        if ok.is_ok() { inner.manual_update = false; }
        else          { inner.manual_update = true; }

        inner.app_state = AppState {
            status:            "connected".into(),
            current_title:     title.display,
            current_title_id:  title.title_id,
            platform:          title.platform,
            discord_connected: true,
            status_message:    "Conectado".into(),
            cover_url:         cover,
            cover_source,
        };
        (inner.app_state.clone(), ok.is_err())
        // lock dropped here
    };

    emit_state(app, &new_state);

    // --- Step 5: reconnect if needed ASYNC (no lock held) ---
    if failed {
        emit_log(app, "⚠ Falha Discord — reconectando...");
        reconnect_discord(app).await;
    }
}

async fn on_disconnect(app: &AppHandle, reason: &str) {
    const CLEAR: u64 = 60;

    // All lock work done synchronously, dropped before any await
    let new_state = {
        let state = app.state::<VitaState>();
        let mut inner = state.inner().0.lock().unwrap();
        let elapsed = inner.last_seen.map(|t| t.elapsed().as_secs()).unwrap_or(u64::MAX);
        if elapsed > CLEAR {
            inner.last_title_id = String::new();
            inner.game_start    = None;
            inner.last_seen     = None;
            inner.manual_update = true;
            inner.discord.clear_presence().ok();
        }
        inner.app_state.status         = "connecting".into();
        inner.app_state.status_message = "Reconectando...".into();
        inner.app_state.clone()
        // lock dropped
    };

    emit_state(app, &new_state);
    emit_log(app, &format!("Vita inacessível ({}) — tentando...", reason));
}

async fn reconnect_discord(app: &AppHandle) {
    let client_id = config::load().client_id;

    // Lock, do work, drop lock — then no await while holding lock
    let result = {
        let state = app.state::<VitaState>();
        let mut inner = state.inner().0.lock().unwrap();
        inner.discord = discord::DiscordRpc::new(client_id);
        let r = inner.discord.connect();
        if r.is_ok() { inner.manual_update = true; }
        r
        // lock dropped
    };

    match result {
        Ok(_)  => emit_log(app, "✅ Discord reconectado"),
        Err(e) => emit_log(app, &format!("❌ Discord: {}", e)),
    }
}

// ── IP resolution ──────────────────────────────────────────────────────

async fn resolve_ip(target: &str) -> Option<String> {
    let t = target.trim();
    if t.split('.').count() == 4 && t.split('.').all(|o| o.parse::<u8>().is_ok()) {
        return Some(t.to_string());
    }
    get_ip_by_mac(t).await
}

async fn get_ip_by_mac(mac: &str) -> Option<String> {
    let m = mac.to_lowercase().replace('-', ":");
    if let Ok(out) = tokio::process::Command::new("ip").args(["neigh"]).output().await {
        for line in String::from_utf8_lossy(&out.stdout).lines() {
            if line.to_lowercase().contains(&m) {
                return line.split_whitespace().next().map(str::to_string);
            }
        }
    }
    if let Ok(out) = tokio::process::Command::new("arp").args(["-a"]).output().await {
        for line in String::from_utf8_lossy(&out.stdout).lines() {
            if line.to_lowercase().contains(&m) {
                return line.split(['(', ')']).nth(1).map(str::to_string);
            }
        }
    }
    None
}

// ── Tray ───────────────────────────────────────────────────────────────

fn setup_tray(app: &AppHandle) -> tauri::Result<()> {
    let show = MenuItem::with_id(app, "show", "Mostrar", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Sair",    true, None::<&str>)?;
    let sep  = tauri::menu::PredefinedMenuItem::separator(app)?;
    let menu = Menu::with_items(app, &[&show, &sep, &quit])?;

    TrayIconBuilder::new()
        .icon(app.default_window_icon().unwrap().clone())
        .menu(&menu)
        .tooltip("VitaPresence")
        .on_menu_event(|app, event| match event.id.as_ref() {
            "show" => {
                if let Some(w) = app.get_webview_window("main") {
                    w.show().ok();
                    w.set_focus().ok();
                }
            }
            "quit" => { app.exit(0); }
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click { .. } = event {
                let app = tray.app_handle();
                if let Some(win) = app.get_webview_window("main") {
                    if win.is_visible().unwrap_or(false) {
                        win.hide().ok();
                    } else {
                        win.show().ok();
                        win.set_focus().ok();
                    }
                }
            }
        })
        .build(app)?;
    Ok(())
}

// ── Entry point ────────────────────────────────────────────────────────

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(VitaState(Mutex::new(Inner::new())))
        .plugin(tauri_plugin_shell::init())
        .setup(|app| { setup_tray(app.handle())?; Ok(()) })
        .invoke_handler(tauri::generate_handler![
            get_config, save_config, get_state, get_running,
            connect, disconnect, hide_window,
        ])
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                if config::load().minimize_to_tray {
                    api.prevent_close();
                    window.hide().ok();
                }
            }
        })
        .run(tauri::generate_context!())
        .expect("error running VitaPresence");
}
