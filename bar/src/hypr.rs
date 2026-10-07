// Hyprland IPC over its two Unix sockets: `.socket.sock` answers one request
// per connection (`j/<query>` returns JSON, `dispatch <lua>` runs a dispatcher
// exactly as `hyprctl dispatch` does), `.socket2.sock` streams `event>>data`
// lines.

use std::io::{Read, Write};
use std::os::unix::net::UnixStream;
use std::path::PathBuf;

use serde_json::Value;

fn socket_dir() -> PathBuf {
    let runtime = std::env::var("XDG_RUNTIME_DIR").expect("XDG_RUNTIME_DIR not set");
    let sig = std::env::var("HYPRLAND_INSTANCE_SIGNATURE")
        .expect("HYPRLAND_INSTANCE_SIGNATURE not set (not running under Hyprland?)");
    PathBuf::from(runtime).join("hypr").join(sig)
}

pub fn event_stream() -> std::io::Result<UnixStream> {
    UnixStream::connect(socket_dir().join(".socket2.sock"))
}

fn request(msg: &str) -> Option<String> {
    let mut stream = UnixStream::connect(socket_dir().join(".socket.sock")).ok()?;
    stream.write_all(msg.as_bytes()).ok()?;
    let mut out = String::new();
    stream.read_to_string(&mut out).ok()?;
    Some(out)
}

fn query(what: &str) -> Value {
    request(&format!("j/{what}"))
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or(Value::Null)
}

/// Runs a Lua dispatcher expression such as `hl.dsp.focus({ workspace = 3 })`.
pub fn dispatch(lua: &str) {
    let _ = request(&format!("dispatch {lua}"));
}

/// Events that can change what the bar shows. Title changes are deliberately
/// not here: the bar doesn't show titles, and some apps (a spinner in a
/// terminal title) change theirs several times a second.
pub fn is_relevant(event: &str) -> bool {
    matches!(
        event,
        "workspacev2"
            | "focusedmonv2"
            | "activewindowv2"
            | "openwindow"
            | "closewindow"
            | "movewindowv2"
            | "createworkspacev2"
            | "destroyworkspacev2"
            | "moveworkspacev2"
            | "renameworkspace"
            | "activelayout"
            | "monitoraddedv2"
            | "monitorremovedv2"
            | "urgent"
    )
}

#[derive(Clone, PartialEq)]
pub struct Workspace {
    pub id: i64,
    pub monitor: String,
    pub windows: i64,
}

#[derive(Clone, PartialEq)]
pub struct Client {
    pub address: String,
    pub class: String,
    pub workspace: i64,
    pub monitor: String,
}

#[derive(Clone, PartialEq, Default)]
pub struct Snapshot {
    pub workspaces: Vec<Workspace>,
    /// (monitor name, active workspace id)
    pub monitors: Vec<(String, i64)>,
    pub clients: Vec<Client>,
    pub active_window: String,
    pub layout: String,
}

impl Snapshot {
    pub fn fetch() -> Self {
        let monitors_json = query("monitors");
        let monitors_json = monitors_json.as_array().cloned().unwrap_or_default();
        let monitor_name = |id: i64| {
            monitors_json
                .iter()
                .find(|m| m["id"].as_i64() == Some(id))
                .and_then(|m| m["name"].as_str())
                .unwrap_or_default()
                .to_string()
        };
        let monitors = monitors_json
            .iter()
            .map(|m| {
                let name = m["name"].as_str().unwrap_or_default().to_string();
                (name, m["activeWorkspace"]["id"].as_i64().unwrap_or(0))
            })
            .collect();

        let workspaces = query("workspaces")
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(|w| {
                let id = w["id"].as_i64()?;
                (id > 0).then(|| Workspace {
                    id,
                    monitor: w["monitor"].as_str().unwrap_or_default().to_string(),
                    windows: w["windows"].as_i64().unwrap_or(0),
                })
            })
            .collect();

        let clients = query("clients")
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(|c| {
                let workspace = c["workspace"]["id"].as_i64()?;
                (workspace > 0 && c["mapped"].as_bool() != Some(false)).then(|| Client {
                    address: c["address"].as_str().unwrap_or_default().to_string(),
                    class: c["class"].as_str().unwrap_or_default().to_string(),
                    workspace,
                    monitor: monitor_name(c["monitor"].as_i64().unwrap_or(-1)),
                })
            })
            .collect();

        let active_window =
            query("activewindow")["address"].as_str().unwrap_or_default().to_string();

        let devices = query("devices");
        let layout = devices["keyboards"]
            .as_array()
            .into_iter()
            .flatten()
            .find(|k| k["main"].as_bool() == Some(true))
            .and_then(|k| k["active_keymap"].as_str())
            .map(short_layout)
            .unwrap_or_default();

        Self { workspaces, monitors, clients, active_window, layout }
    }
}

/// "English (US)" -> "ENG", "Russian" -> "RUS": the first three letters,
/// matching the old Waybar `hyprland/language` formats.
fn short_layout(keymap: &str) -> String {
    keymap.chars().filter(|c| c.is_alphabetic()).take(3).collect::<String>().to_uppercase()
}
