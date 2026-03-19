// src-tauri/src/vita.rs — PS Vita TCP packet parser
use std::net::SocketAddr;
use tokio::net::TcpStream;
use tokio::io::AsyncReadExt;
use serde::Serialize;

pub const VITA_PORT:   u16 = 0xCAFE;
pub const MAGIC:       u32 = 0xCAFECAFE;
pub const PACKET_SIZE: usize = 146;

const PSP_PREFIXES: &[&str] = &[
    "ULUS", "ULES", "ULJM", "ULKS", "UCUS", "UCES", "UCJM",
    "NPUG", "NPEG", "NPJG",
];

const SYSTEM_IDS: &[&str] = &["NPXS10079", "NPXS10063"];

const VITA_EMULATORS: &[(&str, &str, &str)] = &[
    ("MGBA00001", "mGBA",         "Game Boy Advance"),
    ("MGBA00002", "mGBA",         "Game Boy Advance"),
    ("SNES9XVIT", "Snes9x",       "Super Nintendo"),
    ("SNES00001", "Snes9x",       "Super Nintendo"),
    ("RETROARCH0","RetroArch",    "Multi-sistema"),
    ("FCEUX00001","FCEUXVita",    "Nintendo"),
    ("DSVITA001", "DeSmuME Vita", "Nintendo DS"),
    ("MELONDS01", "melonDS Vita", "Nintendo DS"),
    ("PICODRIVE", "PicoDrive",    "Mega Drive"),
    ("FBALPHA01", "FB Alpha",     "Arcade"),
    ("MAME4VITA", "MAME4Vita",    "Arcade"),
    ("SCUMMVM01", "ScummVM",      "Adventure Games"),
    ("DOSBOX001", "DOSBox Vita",  "DOS"),
];

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TitleInfo {
    pub title_id:    String,
    pub title_name:  String,
    pub platform:    String,   // "vita" | "psp" | "emulator" | "liverea"
    pub display:     String,   // friendly display name
    pub state_text:  String,   // auto state (empty = use custom)
    pub is_live_area: bool,
}

fn parse_str(bytes: &[u8]) -> String {
    let end = bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len());
    String::from_utf8_lossy(&bytes[..end]).trim().to_string()
}

pub fn parse_packet(buf: &[u8]) -> Option<TitleInfo> {
    if buf.len() < PACKET_SIZE { return None; }

    let magic = u32::from_le_bytes(buf[0..4].try_into().ok()?);
    if magic != MAGIC { return None; }

    let index     = i32::from_le_bytes(buf[4..8].try_into().ok()?);
    let title_id  = parse_str(&buf[8..18]);
    let title_name = parse_str(&buf[18..146]);

    // Skip system processes
    if SYSTEM_IDS.contains(&title_id.as_str()) { return None; }

    // LiveArea
    if index == 0 {
        return Some(TitleInfo {
            title_id:    String::new(),
            title_name:  "LiveArea".into(),
            platform:    "liverea".into(),
            display:     "No LiveArea".into(),
            state_text:  String::new(),
            is_live_area: true,
        });
    }

    // Adrenaline XMB
    if title_id == "XMB" {
        return Some(TitleInfo {
            title_id,
            title_name:  "Adrenaline XMB Menu".into(),
            platform:    "psp".into(),
            display:     "Adrenaline XMB Menu".into(),
            state_text:  "No menu do PSP".into(),
            is_live_area: false,
        });
    }

    // PSP via Adrenaline
    let id_upper = title_id.to_uppercase();
    if PSP_PREFIXES.iter().any(|p| id_upper.starts_with(p)) {
        let display = if !title_name.is_empty() { title_name.clone() } else { title_id.clone() };
        return Some(TitleInfo {
            title_id,
            title_name: display.clone(),
            platform:   "psp".into(),
            display,
            state_text: "via Adrenaline (PSP)".into(),
            is_live_area: false,
        });
    }

    // Vita emulator
    if let Some(&(_, name, platform)) = VITA_EMULATORS.iter().find(|&&(id, _, _)| id == id_upper) {
        return Some(TitleInfo {
            title_id,
            title_name: name.into(),
            platform:   "emulator".into(),
            display:    name.into(),
            state_text: format!("via {} ({})", name, platform),
            is_live_area: false,
        });
    }

    // Native Vita game
    let display = if !title_name.is_empty() { title_name.clone() } else { title_id.clone() };
    Some(TitleInfo {
        title_id,
        title_name: display.clone(),
        platform:   "vita".into(),
        display,
        state_text: String::new(),
        is_live_area: false,
    })
}

pub async fn poll(ip: &str) -> Result<Option<TitleInfo>, String> {
    let addr: SocketAddr = format!("{}:{}", ip, VITA_PORT)
        .parse()
        .map_err(|e: std::net::AddrParseError| e.to_string())?;

    let mut stream = tokio::time::timeout(
        std::time::Duration::from_millis(2000),
        TcpStream::connect(addr),
    )
    .await
    .map_err(|_| "Timeout de conexão".to_string())?
    .map_err(|e| e.to_string())?;

    let mut buf = vec![0u8; PACKET_SIZE + 64];
    let n = tokio::time::timeout(
        std::time::Duration::from_millis(5500),
        stream.read(&mut buf),
    )
    .await
    .map_err(|_| "Timeout de recebimento".to_string())?
    .map_err(|e| e.to_string())?;

    if n < PACKET_SIZE {
        return Ok(None);
    }

    Ok(parse_packet(&buf[..n]))
}
