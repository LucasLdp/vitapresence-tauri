// src-tauri/src/covers.rs — Cover art resolver
//
// Fontes por ordem de prioridade:
//   1. IGDB (melhor qualidade, requer Twitch credentials)
//   2. PSMT-Covers/SvenGDK   — GitHub, por title ID (Vita + PSP)
//   3. NutDB/nicoboss         — icon0.png da maioria dos títulos Vita
//   4. xlenore/psp-covers     — fallback PSP
//   5. xlenore/psx-covers     — fallback PS1
//   6. custom_icon_url        — URL manual do usuário (passada por cima de tudo)

use reqwest::Client;
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::Mutex;
use once_cell::sync::Lazy;

static CACHE: Lazy<Mutex<HashMap<String, Option<String>>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));
static TOKEN: Lazy<Mutex<Option<(String, u64)>>> =
    Lazy::new(|| Mutex::new(None));

// ── Types ──────────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct TokenResp { access_token: String, expires_in: u64 }

#[derive(Deserialize)]
struct IgdbGame { name: String, cover: Option<IgdbCover> }

#[derive(Deserialize)]
struct IgdbCover { url: String }

// ── Platform detection ─────────────────────────────────────────────────

fn detect_platform(id: &str) -> &'static str {
    let id = id.to_uppercase();
    if id.starts_with("PC") || id.starts_with("NP")               { return "vita"; }
    if ["ULUS","ULES","ULJM","ULKS","UCUS","UCES","UCJM","NPHG",
        "NPUG","NPEG","NPJG"].iter().any(|p| id.starts_with(p))   { return "psp"; }
    if ["SCUS","SCES","SLUS","SLES","SCPS","SLPS","SLPM","SCED",
        "PAPX"].iter().any(|p| id.starts_with(p))                  { return "ps1"; }
    if ["SLUS-2","SCES-5","SCUS-9"].iter().any(|p| id.starts_with(p)) { return "ps2"; }
    "vita"
}

const IGDB_PLATFORMS: &[(&str, u64)] = &[
    ("vita", 46), ("psp", 38), ("ps1", 7), ("ps2", 8), ("ps3", 9),
];

// ── Title normalization ────────────────────────────────────────────────
// Strips region tags, version numbers, subtitles etc. to improve search hit rate

fn normalize_title(name: &str) -> String {
    let mut s = name.to_string();

    // Remove region/version brackets: [USA], (EUR), (v1.1), [NPEB01234]
    s = regex_strip(&s, r"\s*[\[\(][^\]\)]*[\]\)]");

    // Remove common suffixes that confuse search
    for suffix in &[
        " HD", " Remaster", " Remastered", " Complete Edition",
        " Game of the Year", " GOTY", " Director's Cut",
        " The Complete Story", " Golden", " Portable",
    ] {
        if let Some(pos) = s.to_uppercase().find(&suffix.to_uppercase()) {
            // Only strip if it's a suffix (near end of string)
            if pos > s.len() / 2 {
                s = s[..pos].to_string();
            }
        }
    }

    // Collapse whitespace
    s.split_whitespace().collect::<Vec<_>>().join(" ").trim().to_string()
}

// Simple regex-free bracket stripping
fn regex_strip(s: &str, _pattern: &str) -> String {
    let mut result = String::new();
    let mut depth_sq = 0u32;
    let mut depth_rn = 0u32;
    for c in s.chars() {
        match c {
            '[' => depth_sq += 1,
            ']' => { depth_sq = depth_sq.saturating_sub(1); continue; }
            '(' => depth_rn += 1,
            ')' => { depth_rn = depth_rn.saturating_sub(1); continue; }
            _ if depth_sq > 0 || depth_rn > 0 => continue,
            _ => {}
        }
        if c != '[' && c != '(' {
            result.push(c);
        }
    }
    result.trim().to_string()
}

// ── HTTP helpers ───────────────────────────────────────────────────────

async fn url_exists(client: &Client, url: &str) -> bool {
    client.head(url).send().await
        .map(|r| r.status().is_success())
        .unwrap_or(false)
}

async fn first_valid(client: &Client, urls: &[String]) -> Option<String> {
    for url in urls {
        if url_exists(client, url).await {
            return Some(url.clone());
        }
    }
    None
}

// ── Twitch token ───────────────────────────────────────────────────────

fn now_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default().as_secs()
}

async fn get_token(client: &Client, id: &str, secret: &str) -> Option<String> {
    {
        let c = TOKEN.lock().unwrap();
        if let Some((tok, exp)) = c.as_ref() {
            if now_secs() < exp.saturating_sub(60) { return Some(tok.clone()); }
        }
    }
    let r = client
        .post("https://id.twitch.tv/oauth2/token")
        .form(&[("client_id",id),("client_secret",secret),("grant_type","client_credentials")])
        .send().await.ok()?.json::<TokenResp>().await.ok()?;
    let exp = now_secs() + r.expires_in;
    *TOKEN.lock().unwrap() = Some((r.access_token.clone(), exp));
    Some(r.access_token)
}

// ── IGDB ───────────────────────────────────────────────────────────────
// reqwest async helper
async fn igdb_query(
    client: &Client, endpoint: &str, query: String, id: &str, token: &str,
) -> Option<Vec<IgdbGame>> {
    client.post(endpoint)
        .header("Client-ID",     id)
        .header("Authorization", format!("Bearer {}", token))
        .body(query)
        .send().await.ok()?
        .json::<Vec<IgdbGame>>().await.ok()
}

// Rewrite search_igdb properly without the block hack
async fn search_igdb_v2(
    client: &Client, name: &str, title_id: &str, cid: &str, secret: &str,
) -> Option<String> {
    let token    = get_token(client, cid, secret).await?;
    let platform = detect_platform(title_id);
    let plat_id  = IGDB_PLATFORMS.iter().find(|&&(p,_)| p == platform)
        .map(|&(_,i)| i).unwrap_or(46);

    let norm  = normalize_title(name);
    let first = norm.split([':','-']).next().unwrap_or(&norm).trim().to_string();
    let variants: Vec<String> = {
        let mut v = vec![norm.clone()];
        if first != norm && first.len() > 3 { v.push(first); }
        if norm.to_uppercase().contains("VITA") {
            v.push(norm.to_uppercase().replace("VITA","").trim().to_string()
                .split_whitespace().collect::<Vec<_>>().join(" "));
        }
        v.into_iter().filter(|s| !s.is_empty()).collect()
    };

    for variant in &variants {
        let clean = variant.replace('"', "").replace('\'', "");
        let query = format!(
            "fields name,cover.url; where platforms=({}) & name~*\"{}\"*; limit 5;",
            plat_id, clean
        );
        if let Some(games) = igdb_query(client, "https://api.igdb.com/v4/games", query, cid, &token).await {
            if let Some(game) = games.into_iter().find(|g| g.cover.is_some()) {
                if let Some(cover) = game.cover {
                    return Some(
                        cover.url.replace("//","https://").replace("t_thumb","t_cover_big")
                    );
                }
            }
        }
    }
    None
}

// ── GitHub fallbacks ───────────────────────────────────────────────────

async fn search_github(client: &Client, title_id: &str) -> Option<String> {
    let id       = title_id.to_uppercase();
    let id_lower = title_id.to_lowercase();
    let platform = detect_platform(&id);

    let urls: Vec<String> = match platform {
        "psp" => vec![
            format!("https://raw.githubusercontent.com/SvenGDK/PSMT-Covers/main/PSP/{}.jpg",  id),
            format!("https://raw.githubusercontent.com/SvenGDK/PSMT-Covers/main/PSP/{}.png",  id),
            format!("https://raw.githubusercontent.com/xlenore/psp-covers/main/covers/default/{}.jpg", id),
            format!("https://raw.githubusercontent.com/xlenore/psp-covers/main/covers/{}.jpg", id),
            format!("https://raw.githubusercontent.com/xlenore/psp-covers/main/covers/default/{}.jpg", id_lower),
        ],
        "ps1" => vec![
            format!("https://raw.githubusercontent.com/xlenore/psx-covers/main/covers/default/{}.jpg", id),
            format!("https://raw.githubusercontent.com/xlenore/psx-covers/main/covers/{}.jpg",         id),
            format!("https://raw.githubusercontent.com/xlenore/psx-covers/main/covers/default/{}.jpg", id_lower),
        ],
        _ => vec![
            // PSMT-Covers — melhor cobertura Vita
            format!("https://raw.githubusercontent.com/SvenGDK/PSMT-Covers/main/PSV/{}.jpg",  id),
            format!("https://raw.githubusercontent.com/SvenGDK/PSMT-Covers/main/PSV/{}.png",  id),
            // NutDB — icon0.png para quase todo título Vita
            format!("https://raw.githubusercontent.com/nicoboss/NutDB/master/titles/{}/icon0.png", id),
            // PSMT lowercase variant
            format!("https://raw.githubusercontent.com/SvenGDK/PSMT-Covers/main/PSV/{}.jpg",  id_lower),
            // Fallback: busca pelo prefixo regional variante
            format!("https://raw.githubusercontent.com/SvenGDK/PSMT-Covers/main/PSV/{}.jpg",
                id.replace("PCSA","PCSE").replace("PCSE","PCSA")),
        ],
    };

    first_valid(client, &urls).await
}

// ── Public API ─────────────────────────────────────────────────────────

pub fn clear_cache() {
    CACHE.lock().unwrap().clear();
    *TOKEN.lock().unwrap() = None;
}

/// Resolve cover URL. Returns custom_icon_url if set, else searches IGDB/GitHub.
pub async fn resolve(
    game_name:      &str,
    title_id:       &str,
    custom_icon:    Option<&str>,
    igdb_id:        Option<&str>,
    igdb_sec:       Option<&str>,
) -> Option<String> {
    // Custom icon always wins
    if let Some(url) = custom_icon {
        if !url.is_empty() { return Some(url.to_string()); }
    }

    if title_id.is_empty() { return None; }

    let key = format!("{}:{}", title_id.to_uppercase(), game_name);
    {
        let cache = CACHE.lock().unwrap();
        if let Some(cached) = cache.get(&key) {
            return cached.clone();
        }
    }

    let client = match Client::builder()
        .timeout(std::time::Duration::from_secs(8))
        .build()
    {
        Ok(c)  => c,
        Err(_) => return None,
    };

    let result = match (igdb_id, igdb_sec) {
        (Some(id), Some(sec)) if !id.is_empty() && !sec.is_empty() => {
            search_igdb_v2(&client, game_name, title_id, id, sec).await
                .or(search_github(&client, title_id).await)
        }
        _ => search_github(&client, title_id).await,
    };

    CACHE.lock().unwrap().insert(key, result.clone());
    result
}
