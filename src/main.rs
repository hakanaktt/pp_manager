// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use chrono::Local;
use serde::{Serialize, Deserialize};
use std::fs;
use tauri::{Manager, SystemTray, SystemTrayEvent, SystemTrayMenu, CustomMenuItem, SystemTrayMenuItem};

mod process_manager;
use process_manager::{ProcessManager, ProcessState, LogEntry};

#[derive(Serialize, Deserialize, Clone)]
struct Config {
    target_process: String,
    core_selections: Vec<bool>,
    priority_class: u32,
}

impl Default for Config {
    fn default() -> Self {
        let cpu_count = ProcessManager::get_system_cpu_count() as usize;
        let max_cores = cpu_count.max(8); // Ensure at least 8 cores for compatibility

        // Initialize with performance cores selected (typically cores 4-5 on hybrid CPUs)
        let mut core_selections = vec![false; max_cores];
        if max_cores > 4 {
            core_selections[4] = true;
        }
        if max_cores > 5 {
            core_selections[5] = true;
        }

        Self {
            target_process: "icad.exe".to_string(),
            core_selections,
            priority_class: 0x00008000, // ABOVE_NORMAL_PRIORITY_CLASS
        }
    }
}

// Application state that will be shared across Tauri commands
#[derive(Clone)]
struct AppState {
    process_manager: Arc<Mutex<ProcessManager>>,
    is_running: Arc<Mutex<bool>>,
    logs: Arc<Mutex<Vec<LogEntry>>>,
    config: Arc<Mutex<Config>>,
}

impl AppState {
    fn new() -> Self {
        Self {
            process_manager: Arc::new(Mutex::new(ProcessManager::new())),
            is_running: Arc::new(Mutex::new(false)),
            logs: Arc::new(Mutex::new(Vec::new())),
            config: Arc::new(Mutex::new(load_config())),
        }
    }
}

fn save_config(config: &Config) {
    if let Ok(data) = toml::to_string(config) {
        let _ = fs::write("config.toml", data);
    }
}

fn load_config() -> Config {
    let mut config: Config = fs::read_to_string("config.toml")
        .ok()
        .and_then(|d| toml::from_str(&d).ok())
        .unwrap_or_default();

    // Ensure core_selections vector matches current system CPU count
    let cpu_count = ProcessManager::get_system_cpu_count() as usize;
    let max_cores = cpu_count.max(8); // Ensure at least 8 cores for compatibility

    if config.core_selections.len() != max_cores {
        // Resize the vector, preserving existing selections where possible
        config.core_selections.resize(max_cores, false);
    }

    config
}

// Tauri commands
#[tauri::command]
fn get_system_info() -> serde_json::Value {
    serde_json::json!({
        "cpu_count": ProcessManager::get_system_cpu_count(),
        "affinity_mask": format!("0x{:X}", ProcessManager::get_system_affinity_mask()),
        "is_admin": ProcessManager::is_running_as_administrator()
    })
}

#[tauri::command]
fn get_config(state: tauri::State<AppState>) -> Config {
    state.config.lock().unwrap().clone()
}

#[tauri::command]
fn save_config_cmd(config: Config, state: tauri::State<AppState>) -> Result<(), String> {
    // Update the state
    *state.config.lock().unwrap() = config.clone();

    // Save to file
    if let Ok(data) = toml::to_string(&config) {
        fs::write("config.toml", data).map_err(|e| e.to_string())?;
        Ok(())
    } else {
        Err("Failed to serialize config".to_string())
    }
}

#[tauri::command]
fn start_monitoring(state: tauri::State<AppState>) -> Result<(), String> {
    let mut is_running = state.is_running.lock().unwrap();
    if *is_running {
        return Err("Monitoring is already running".to_string());
    }
    *is_running = true;
    drop(is_running);

    let process_manager = Arc::clone(&state.process_manager);
    let is_running_clone = Arc::clone(&state.is_running);
    let logs_clone = Arc::clone(&state.logs);
    let config = state.config.lock().unwrap().clone();

    let target_process = config.target_process.clone();
    let affinity_mask = calculate_affinity_mask(&config.core_selections);
    let priority_class = config.priority_class;

    thread::spawn(move || {
        let mut last_state = ProcessState::NotFound;

        while *is_running_clone.lock().unwrap() {
            let current_state = {
                let mut pm = process_manager.lock().unwrap();
                pm.check_and_manage_process(&target_process, affinity_mask, priority_class)
            };

            // Log based on state changes and important events
            let should_log = match (&current_state, &last_state) {
                // Always log state changes
                (ProcessState::NotFound, ProcessState::Found(_)) |
                (ProcessState::NotFound, ProcessState::FoundAndMonitoring(_)) |
                (ProcessState::NotFound, ProcessState::SettingsApplied(_, _)) => true,
                (ProcessState::Found(_), ProcessState::NotFound) |
                (ProcessState::FoundAndMonitoring(_), ProcessState::NotFound) |
                (ProcessState::SettingsApplied(_, _), ProcessState::NotFound) => true,
                // Log when process is first found
                (ProcessState::Found(_), _) if last_state == ProcessState::NotFound => true,
                // Log when settings are reapplied
                (ProcessState::SettingsApplied(_, _), _) => true,
                // Log errors
                (ProcessState::Error(_), _) => true,
                // Don't spam logs for continuous monitoring
                _ => false,
            };

            if should_log {
                let log_entry = LogEntry {
                    timestamp: Local::now(),
                    message: match &current_state {
                        ProcessState::NotFound => format!("{} not running.", target_process),
                        ProcessState::Found(pid) => format!("{} found (PID: {}). Initial settings applied.", target_process, pid),
                        ProcessState::FoundAndMonitoring(pid) => format!("{} (PID: {}) monitoring active.", target_process, pid),
                        ProcessState::SettingsApplied(pid, changes) => format!("{} (PID: {}) settings reapplied: {}", target_process, pid, changes),
                        ProcessState::Error(err) => format!("Error: {}", err),
                    },
                    level: match &current_state {
                        ProcessState::Error(_) => "ERROR".to_string(),
                        ProcessState::Found(_) => "SUCCESS".to_string(),
                        ProcessState::SettingsApplied(_, _) => "REAPPLY".to_string(),
                        ProcessState::FoundAndMonitoring(_) => "MONITOR".to_string(),
                        ProcessState::NotFound => "INFO".to_string(),
                    },
                };

                logs_clone.lock().unwrap().push(log_entry);
            }

            last_state = current_state;
            thread::sleep(Duration::from_secs(2));
        }
    });

    Ok(())
}

#[tauri::command]
fn stop_monitoring(state: tauri::State<AppState>) -> Result<(), String> {
    let mut is_running = state.is_running.lock().unwrap();
    *is_running = false;
    Ok(())
}

#[tauri::command]
fn get_monitoring_status(state: tauri::State<AppState>) -> bool {
    *state.is_running.lock().unwrap()
}

#[tauri::command]
fn get_logs(state: tauri::State<AppState>) -> Vec<LogEntry> {
    state.logs.lock().unwrap().clone()
}

#[tauri::command]
fn clear_logs(state: tauri::State<AppState>) -> Result<(), String> {
    state.logs.lock().unwrap().clear();
    Ok(())
}

fn calculate_affinity_mask(core_selections: &[bool]) -> u64 {
    core_selections
        .iter()
        .enumerate()
        .fold(0u64, |acc, (i, &selected)| {
            if selected { acc | (1u64 << i) } else { acc }
        })
}



fn main() {
    // Create system tray
    let show = CustomMenuItem::new("show".to_string(), "Show Settings");
    let hide = CustomMenuItem::new("hide".to_string(), "Hide Settings");
    let quit = CustomMenuItem::new("quit".to_string(), "Exit");
    let tray_menu = SystemTrayMenu::new()
        .add_item(show)
        .add_item(hide)
        .add_native_item(SystemTrayMenuItem::Separator)
        .add_item(quit);

    let system_tray = SystemTray::new()
        .with_menu(tray_menu)
        .with_tooltip("Process Priority Manager");

    // Create application state
    let app_state = AppState::new();

    tauri::Builder::default()
        .manage(app_state)
        .system_tray(system_tray)
        .on_system_tray_event(|app, event| match event {
            SystemTrayEvent::LeftClick {
                position: _,
                size: _,
                ..
            } => {
                let window = app.get_window("main").unwrap();
                window.show().unwrap();
                window.set_focus().unwrap();
            }
            SystemTrayEvent::MenuItemClick { id, .. } => match id.as_str() {
                "show" => {
                    let window = app.get_window("main").unwrap();
                    window.show().unwrap();
                    window.set_focus().unwrap();
                }
                "hide" => {
                    let window = app.get_window("main").unwrap();
                    window.hide().unwrap();
                }
                "quit" => {
                    std::process::exit(0);
                }
                _ => {}
            },
            _ => {}
        })
        .on_window_event(|event| match event.event() {
            tauri::WindowEvent::CloseRequested { api, .. } => {
                event.window().hide().unwrap();
                api.prevent_close();
            }
            _ => {}
        })
        .invoke_handler(tauri::generate_handler![
            get_system_info,
            get_config,
            save_config_cmd,
            start_monitoring,
            stop_monitoring,
            get_monitoring_status,
            get_logs,
            clear_logs
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}






