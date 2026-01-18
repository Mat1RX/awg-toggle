use anyhow::{Context, Result};
use serde::Serialize;
use std::env;
use std::fs;
use std::path::{PathBuf};
use std::process::{Command, Stdio};

// Путь к конфигам AmneziaWG
const WG_CONFIG_DIR: &str = "/etc/amnezia/amneziawg";

#[derive(Serialize)]
struct WaybarOutput {
    text: String,
    alt: String,
    tooltip: String,
    class: String,
    percentage: u8,
}

struct State {
    selected_config: String,
}

impl State {
    fn load() -> Self {
        let path = get_state_path();
        if let Ok(content) = fs::read_to_string(&path) {
            return State {
                selected_config: content.trim().to_string(),
            };
        }
        State {
            selected_config: String::new(),
        }
    }

    fn save(&self) -> Result<()> {
        let path = get_state_path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(path, &self.selected_config)?;
        Ok(())
    }
}

fn get_state_path() -> PathBuf {
    dirs::cache_dir()
        .unwrap_or_else(|| PathBuf::from("/tmp"))
        .join("wg-toggle-state")
}

fn get_available_configs() -> Result<Vec<String>> {
    let mut configs = Vec::new();
    let entries = fs::read_dir(WG_CONFIG_DIR)
        .context(format!("Не удалось прочитать {}", WG_CONFIG_DIR))?;

    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) == Some("conf") {
            if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                configs.push(stem.to_string());
            }
        }
    }
    configs.sort();
    Ok(configs)
}

fn get_active_interface() -> Option<String> {
    // Здесь output() перехватывает stdout сам, поэтому утечки в консоль не будет.
    // Но stderr лучше заглушить на случай ошибок прав доступа.
    let output = Command::new("sudo")
        .args(["awg", "show", "interfaces"])
        .stderr(Stdio::null()) 
        .output()
        .ok()?;

    if output.status.success() {
        let out_str = String::from_utf8_lossy(&output.stdout);
        if let Some(first) = out_str.split_whitespace().next() {
            return Some(first.to_string());
        }
    }
    None
}

fn handle_status() -> Result<()> {
    let active = get_active_interface();
    let mut state = State::load();
    let configs = get_available_configs().unwrap_or_default();

    if state.selected_config.is_empty() && !configs.is_empty() {
        state.selected_config = configs[0].clone();
        let _ = state.save();
    }

    let (text, class, tooltip, alt) = match active {
        Some(ref name) => (
            format!(" {}", name),
            "connected".to_string(),
            format!("AmneziaWG Connected: {}", name),
            "connected".to_string(),
        ),
        None => (
            format!(" {}", state.selected_config),
            "disconnected".to_string(),
            format!("AmneziaWG Disconnected. Selected: {}", state.selected_config),
            "disconnected".to_string(),
        ),
    };

    let output = WaybarOutput {
        text,
        alt,
        tooltip,
        class,
        percentage: if active.is_some() { 100 } else { 0 },
    };

    println!("{}", serde_json::to_string(&output)?);
    Ok(())
}

fn toggle_vpn() -> Result<()> {
    let active = get_active_interface();
    let state = State::load();

    if let Some(active_name) = active {
        Command::new("sudo")
            .args(["awg-quick", "down", &active_name])
            .stdout(Stdio::null()) // Глушим вывод
            .stderr(Stdio::null()) // Глушим ошибки
            .status()?;
    } else {
        if !state.selected_config.is_empty() {
            Command::new("sudo")
                .args(["awg-quick", "up", &state.selected_config])
                .stdout(Stdio::null()) // Глушим вывод
                .stderr(Stdio::null()) // Глушим ошибки
                .status()?;
        }
    }
    Ok(())
}

fn cycle_config(direction: i32) -> Result<()> {
    let configs = get_available_configs()?;
    if configs.is_empty() {
        return Ok(());
    }

    let mut state = State::load();
    let active = get_active_interface();

    let current_index = configs
        .iter()
        .position(|c| c == &state.selected_config)
        .unwrap_or(0);

    let len = configs.len() as i32;
    let new_index = (current_index as i32 + direction).rem_euclid(len) as usize;
    
    let new_config = configs[new_index].clone();
    state.selected_config = new_config.clone();
    state.save()?;

    if let Some(active_name) = active {
        Command::new("sudo")
            .args(["awg-quick", "down", &active_name])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()?;

        Command::new("sudo")
            .args(["awg-quick", "up", &new_config])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()?;
    }

    handle_status()?;
    Ok(())
}

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        toggle_vpn()?;
        handle_status()?; 
        return Ok(());
    }

    match args[1].as_str() {
        "--status" => handle_status()?,
        "next" => cycle_config(1)?,
        "previous" => cycle_config(-1)?,
        _ => {
            toggle_vpn()?;
            handle_status()?;
        }
    }

    Ok(())
}
