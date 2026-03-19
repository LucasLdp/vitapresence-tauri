// src-tauri/src/config.rs
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    pub ip:                 String,
    pub client_id:          String,
    pub update_interval:    u64,
    pub state:              String,
    pub display_timer:      bool,
    pub display_main_menu:  bool,
    pub minimize_to_tray:   bool,
    pub custom_icon_url:    String,   // URL de ícone customizado no Discord
    pub igdb_client_id:     String,
    pub igdb_client_secret: String,
    pub language:           String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            ip:                 String::new(),
            client_id:          String::new(),
            update_interval:    10,
            state:              String::new(),
            display_timer:      true,
            display_main_menu:  true,
            minimize_to_tray:   true,
            custom_icon_url:    String::new(),
            igdb_client_id:     String::new(),
            igdb_client_secret: String::new(),
            language:           "pt".to_string(),
        }
    }
}

impl Config {
    pub fn validate(&self) -> Vec<String> {
        let mut errors = Vec::new();
        if self.ip.trim().is_empty() {
            errors.push("IP ou MAC do Vita é obrigatório".into());
        }
        if self.client_id.trim().is_empty() {
            errors.push("Discord Client ID é obrigatório".into());
        }
        if self.update_interval < 1 {
            errors.push("Intervalo mínimo é 1 segundo".into());
        }
        errors
    }
}

fn config_path() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("vitapresence")
        .join("config.json")
}

pub fn load() -> Config {
    let path = config_path();
    if path.exists() {
        if let Ok(raw) = fs::read_to_string(&path) {
            if let Ok(cfg) = serde_json::from_str::<Config>(&raw) {
                return cfg;
            }
        }
    }
    Config::default()
}

pub fn save(cfg: &Config) -> Result<(), String> {
    let path = config_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let json = serde_json::to_string_pretty(cfg).map_err(|e| e.to_string())?;
    fs::write(&path, json).map_err(|e| e.to_string())
}
