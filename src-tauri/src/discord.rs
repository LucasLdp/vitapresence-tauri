// src-tauri/src/discord.rs — Discord RPC via native Unix IPC (zero deps)
use std::io::{Read, Write};
use std::os::unix::net::UnixStream;
use std::time::Duration;
use serde_json::{json, Value};

fn get_uid() -> u32 {
    #[cfg(unix)]
    { unsafe { libc::getuid() } }
    #[cfg(not(unix))]
    { 1000 }
}

fn get_socket_path() -> Option<String> {
    let runtime = std::env::var("XDG_RUNTIME_DIR")
        .unwrap_or_else(|_| format!("/run/user/{}", get_uid()));

    let tmp = std::env::temp_dir().to_string_lossy().to_string();

    let bases = vec![
        runtime.clone(),
        format!("{}/app/com.discordapp.Discord", runtime),
        format!("{}/snap.discord", runtime),
        tmp,
    ];

    for base in &bases {
        for i in 0..10u8 {
            let path = format!("{}/discord-ipc-{}", base, i);
            if std::path::Path::new(&path).exists() {
                return Some(path);
            }
        }
    }
    None
}

fn encode(op: u32, data: &Value) -> Vec<u8> {
    let json  = serde_json::to_string(data).unwrap_or_default();
    let bytes = json.as_bytes();
    let len   = bytes.len() as u32;
    let mut buf = Vec::with_capacity(8 + bytes.len());
    buf.extend_from_slice(&op.to_le_bytes());
    buf.extend_from_slice(&len.to_le_bytes());
    buf.extend_from_slice(bytes);
    buf
}

fn decode(buf: &[u8]) -> Option<Value> {
    if buf.len() < 8 { return None; }
    let len = u32::from_le_bytes(buf[4..8].try_into().ok()?) as usize;
    if buf.len() < 8 + len { return None; }
    serde_json::from_slice(&buf[8..8 + len]).ok()
}

pub struct DiscordRpc {
    socket:    Option<UnixStream>,
    client_id: String,
    nonce:     u32,
}

impl DiscordRpc {
    pub fn new(client_id: String) -> Self {
        Self { socket: None, client_id, nonce: 0 }
    }

    pub fn is_connected(&self) -> bool {
        self.socket.is_some()
    }

    pub fn connect(&mut self) -> Result<(), String> {
        let path = get_socket_path()
            .ok_or_else(|| "Discord IPC socket não encontrado. Discord está aberto?".to_string())?;

        let mut stream = UnixStream::connect(&path)
            .map_err(|e| format!("Falha ao conectar ao socket Discord: {}", e))?;

        stream.set_read_timeout(Some(Duration::from_secs(5))).ok();
        stream.set_write_timeout(Some(Duration::from_secs(5))).ok();

        let handshake = encode(0, &json!({
            "v":         1,
            "client_id": self.client_id
        }));
        stream.write_all(&handshake)
            .map_err(|e| format!("Falha no handshake Discord: {}", e))?;

        let mut buf = vec![0u8; 4096];
        let n = stream.read(&mut buf)
            .map_err(|e| format!("Falha ao ler resposta Discord: {}", e))?;

        let msg = decode(&buf[..n])
            .ok_or_else(|| "Resposta inválida do Discord".to_string())?;

        if msg.get("evt").and_then(|v| v.as_str()) == Some("READY") {
            self.socket = Some(stream);
            Ok(())
        } else {
            Err(format!("Handshake falhou: {:?}", msg))
        }
    }

    pub fn disconnect(&mut self) {
        let _ = self.clear_presence();
        self.socket = None;
    }

    pub fn set_presence(
        &mut self,
        details:    &str,
        state:      Option<&str>,
        start_time: Option<i64>,
        cover_url:  Option<&str>,
    ) -> Result<(), String> {
        let socket = self.socket.as_mut()
            .ok_or_else(|| "Não conectado ao Discord".to_string())?;

        let mut activity = json!({ "details": details, "type": 0 });

        if let Some(s) = state {
            if !s.is_empty() {
                activity["state"] = json!(s);
            }
        }
        if let Some(ts) = start_time {
            activity["timestamps"] = json!({ "start": ts });
        }
        if let Some(url) = cover_url {
            if !url.is_empty() {
                activity["assets"] = json!({
                    "large_image": url,
                    "large_text":  details,
                });
            }
        }

        self.nonce += 1;
        let payload = encode(1, &json!({
            "cmd":   "SET_ACTIVITY",
            "args":  { "pid": std::process::id(), "activity": activity },
            "nonce": self.nonce.to_string(),
        }));

        socket.write_all(&payload).map_err(|e| {
            self.socket = None;
            e.to_string()
        })
    }

    pub fn clear_presence(&mut self) -> Result<(), String> {
        let socket = self.socket.as_mut()
            .ok_or_else(|| "Não conectado".to_string())?;
        self.nonce += 1;
        let payload = encode(1, &json!({
            "cmd":   "SET_ACTIVITY",
            "args":  { "pid": std::process::id(), "activity": null },
            "nonce": self.nonce.to_string(),
        }));
        socket.write_all(&payload).map_err(|e| e.to_string())
    }
}
