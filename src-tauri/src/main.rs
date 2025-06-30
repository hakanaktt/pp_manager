// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use chrono::Local;
use serde::{Serialize, Deserialize};
use std::fs;


mod process_manager;
use process_manager::{ProcessManager, ProcessState, LogEntry};

#[derive(Serialize, Deserialize, Clone, Debug)]
struct ProcessConfig {
    pub name: String,
    pub core_selections: Vec<bool>,
    pub priority_class: u32,
    pub enabled: bool,
}

#[derive(Serialize, Deserialize, Clone)]
struct Config {
    // Legacy single process support (for backward compatibility)
    target_process: String,
    core_selections: Vec<bool>,
    priority_class: u32,

    // New multi-process support
    processes: Vec<ProcessConfig>,
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
            core_selections: core_selections.clone(),
            priority_class: 0x00008000, // ABOVE_NORMAL_PRIORITY_CLASS
            processes: vec![
                ProcessConfig {
                    name: "icad.exe".to_string(),
                    core_selections,
                    priority_class: 0x00008000,
                    enabled: true,
                }
            ],
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

    // Migrate legacy single process config to new multi-process format if needed
    if config.processes.is_empty() && !config.target_process.is_empty() {
        config.processes.push(ProcessConfig {
            name: config.target_process.clone(),
            core_selections: config.core_selections.clone(),
            priority_class: config.priority_class,
            enabled: true,
        });
    }

    // Ensure all process configs have correct core count
    for process_config in &mut config.processes {
        if process_config.core_selections.len() != max_cores {
            process_config.core_selections.resize(max_cores, false);
        }
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

    // Check if we should use multi-process mode or legacy single process mode
    let use_multi_process = !config.processes.is_empty() &&
                           config.processes.iter().any(|p| p.enabled);

    if use_multi_process {
        // Multi-process mode
        let process_configs: Vec<(String, u64, u32)> = config.processes
            .iter()
            .filter(|p| p.enabled)
            .map(|p| (
                p.name.clone(),
                calculate_affinity_mask(&p.core_selections),
                p.priority_class
            ))
            .collect();

        thread::spawn(move || {
            let mut last_state = ProcessState::NotFound;

            while *is_running_clone.lock().unwrap() {
                let current_state = {
                    let mut pm = process_manager.lock().unwrap();
                    pm.check_and_manage_multiple_processes(&process_configs)
                };

                let should_log = current_state != last_state;

                if should_log {
                    let log_entry = LogEntry {
                        timestamp: Local::now(),
                        message: match &current_state {
                            ProcessState::NotFound => "No configured processes running.".to_string(),
                            ProcessState::MultiProcessFound(process_map) => {
                                let details: Vec<String> = process_map.iter()
                                    .map(|(name, pids)| format!("{}: {} instances", name, pids.len()))
                                    .collect();
                                format!("Processes found: {}", details.join(", "))
                            },
                            ProcessState::MultiProcessMonitoring(process_map) => {
                                let total_pids: usize = process_map.values().map(|pids| pids.len()).sum();
                                format!("Monitoring {} processes with {} total instances.", process_map.len(), total_pids)
                            },
                            ProcessState::MultiProcessSettingsApplied(process_map) => {
                                let total_changes: usize = process_map.values().map(|changes| changes.len()).sum();
                                format!("Settings reapplied to {} processes ({} instances).", process_map.len(), total_changes)
                            },
                            ProcessState::Error(err) => format!("Error: {}", err),
                            _ => "Unexpected state in multi-process mode".to_string(),
                        },
                        level: match &current_state {
                            ProcessState::Error(_) => "ERROR".to_string(),
                            ProcessState::MultiProcessFound(_) => "SUCCESS".to_string(),
                            ProcessState::MultiProcessSettingsApplied(_) => "REAPPLY".to_string(),
                            ProcessState::MultiProcessMonitoring(_) => "MONITOR".to_string(),
                            ProcessState::NotFound => "INFO".to_string(),
                            _ => "INFO".to_string(),
                        },
                    };

                    logs_clone.lock().unwrap().push(log_entry);
                }

                last_state = current_state;
                thread::sleep(Duration::from_secs(2));
            }
        });
    } else {
        // Legacy single process mode
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
                (ProcessState::NotFound, ProcessState::SettingsApplied(_, _)) |
                (ProcessState::NotFound, ProcessState::MultipleFound(_)) |
                (ProcessState::NotFound, ProcessState::MultipleMonitoring(_)) |
                (ProcessState::NotFound, ProcessState::MultipleSettingsApplied(_)) => true,
                (ProcessState::Found(_), ProcessState::NotFound) |
                (ProcessState::FoundAndMonitoring(_), ProcessState::NotFound) |
                (ProcessState::SettingsApplied(_, _), ProcessState::NotFound) |
                (ProcessState::MultipleFound(_), ProcessState::NotFound) |
                (ProcessState::MultipleMonitoring(_), ProcessState::NotFound) |
                (ProcessState::MultipleSettingsApplied(_), ProcessState::NotFound) => true,
                // Log when process is first found
                (ProcessState::Found(_), _) if last_state == ProcessState::NotFound => true,
                (ProcessState::MultipleFound(_), _) if last_state == ProcessState::NotFound => true,
                // Log when settings are reapplied
                (ProcessState::SettingsApplied(_, _), _) => true,
                (ProcessState::MultipleSettingsApplied(_), _) => true,
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
                        ProcessState::MultipleFound(pids) => format!("{} {} instances found (PIDs: {}). Initial settings applied.", target_process, pids.len(), format_pids(pids)),
                        ProcessState::MultipleMonitoring(pids) => format!("{} {} instances (PIDs: {}) monitoring active.", target_process, pids.len(), format_pids(pids)),
                        ProcessState::MultipleSettingsApplied(pid_changes) => {
                            let changes_str = pid_changes.iter()
                                .map(|(pid, changes)| format!("PID {}: {}", pid, changes))
                                .collect::<Vec<_>>()
                                .join("; ");
                            format!("{} {} instances settings reapplied: {}", target_process, pid_changes.len(), changes_str)
                        },
                        ProcessState::MultiProcessFound(process_map) => {
                            let total_pids: usize = process_map.values().map(|pids| pids.len()).sum();
                            format!("Multi-process monitoring: {} processes, {} total instances found.", process_map.len(), total_pids)
                        },
                        ProcessState::MultiProcessMonitoring(process_map) => {
                            let total_pids: usize = process_map.values().map(|pids| pids.len()).sum();
                            format!("Multi-process monitoring: {} processes, {} total instances active.", process_map.len(), total_pids)
                        },
                        ProcessState::MultiProcessSettingsApplied(process_map) => {
                            let total_changes: usize = process_map.values().map(|changes| changes.len()).sum();
                            format!("Multi-process monitoring: {} processes, {} settings reapplied.", process_map.len(), total_changes)
                        },
                        ProcessState::Error(err) => format!("Error: {}", err),
                    },
                    level: match &current_state {
                        ProcessState::Error(_) => "ERROR".to_string(),
                        ProcessState::Found(_) | ProcessState::MultipleFound(_) | ProcessState::MultiProcessFound(_) => "SUCCESS".to_string(),
                        ProcessState::SettingsApplied(_, _) | ProcessState::MultipleSettingsApplied(_) | ProcessState::MultiProcessSettingsApplied(_) => "REAPPLY".to_string(),
                        ProcessState::FoundAndMonitoring(_) | ProcessState::MultipleMonitoring(_) | ProcessState::MultiProcessMonitoring(_) => "MONITOR".to_string(),
                        ProcessState::NotFound => "INFO".to_string(),
                    },
                };

                logs_clone.lock().unwrap().push(log_entry);
            }

                last_state = current_state;
                thread::sleep(Duration::from_secs(2));
            }
        });
    }

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

#[tauri::command]
fn get_tracked_processes(state: tauri::State<AppState>) -> Vec<serde_json::Value> {
    let process_manager = state.process_manager.lock().unwrap();
    process_manager.get_tracked_processes()
        .values()
        .map(|instance| serde_json::json!({
            "pid": instance.pid,
            "last_applied_affinity": instance.last_applied_affinity,
            "last_applied_priority": instance.last_applied_priority
        }))
        .collect()
}

#[tauri::command]
#[allow(non_snake_case)]
fn add_process_config(name: String, coreSelections: Vec<bool>, priorityClass: u32, state: tauri::State<AppState>) -> Result<(), String> {
    let mut config = state.config.lock().unwrap();

    // Check if process already exists
    if config.processes.iter().any(|p| p.name == name) {
        return Err(format!("Process '{}' already exists", name));
    }

    config.processes.push(ProcessConfig {
        name,
        core_selections: coreSelections,
        priority_class: priorityClass,
        enabled: true,
    });

    // Save to file
    if let Ok(data) = toml::to_string(&*config) {
        fs::write("config.toml", data).map_err(|e| e.to_string())?;
        Ok(())
    } else {
        Err("Failed to serialize config".to_string())
    }
}

#[tauri::command]
fn remove_process_config(name: String, state: tauri::State<AppState>) -> Result<(), String> {
    let mut config = state.config.lock().unwrap();

    let initial_len = config.processes.len();
    config.processes.retain(|p| p.name != name);

    if config.processes.len() == initial_len {
        return Err(format!("Process '{}' not found", name));
    }

    // Save to file
    if let Ok(data) = toml::to_string(&*config) {
        fs::write("config.toml", data).map_err(|e| e.to_string())?;
        Ok(())
    } else {
        Err("Failed to serialize config".to_string())
    }
}

#[tauri::command]
#[allow(non_snake_case)]
fn update_process_config(name: String, coreSelections: Vec<bool>, priorityClass: u32, enabled: bool, state: tauri::State<AppState>) -> Result<(), String> {
    let mut config = state.config.lock().unwrap();

    if let Some(process_config) = config.processes.iter_mut().find(|p| p.name == name) {
        process_config.core_selections = coreSelections;
        process_config.priority_class = priorityClass;
        process_config.enabled = enabled;

        // Save to file
        if let Ok(data) = toml::to_string(&*config) {
            fs::write("config.toml", data).map_err(|e| e.to_string())?;
            Ok(())
        } else {
            Err("Failed to serialize config".to_string())
        }
    } else {
        Err(format!("Process '{}' not found", name))
    }
}

#[tauri::command]
fn get_process_configs(state: tauri::State<AppState>) -> Vec<ProcessConfig> {
    let config = state.config.lock().unwrap();
    config.processes.clone()
}

#[tauri::command]
fn kill_process(pid: u32, state: tauri::State<AppState>) -> Result<(), String> {
    let mut process_manager = state.process_manager.lock().unwrap();
    process_manager.kill_process(pid)
}

#[tauri::command]
fn get_process_details(pid: u32, state: tauri::State<AppState>) -> Result<serde_json::Value, String> {
    let process_manager = state.process_manager.lock().unwrap();
    match process_manager.get_process_details(pid) {
        Ok(details) => Ok(serde_json::json!({
            "pid": details.pid,
            "name": details.name,
            "current_priority": details.current_priority,
            "current_affinity": details.current_affinity,
            "last_applied_priority": details.last_applied_priority,
            "last_applied_affinity": details.last_applied_affinity,
            "is_tracked": details.is_tracked
        })),
        Err(err) => Err(err)
    }
}

fn calculate_affinity_mask(core_selections: &[bool]) -> u64 {
    core_selections
        .iter()
        .enumerate()
        .fold(0u64, |acc, (i, &selected)| {
            if selected { acc | (1u64 << i) } else { acc }
        })
}

fn format_pids(pids: &[u32]) -> String {
    pids.iter()
        .map(|pid| pid.to_string())
        .collect::<Vec<_>>()
        .join(", ")
}

fn main() {
    println!("Starting Process Priority Manager...");

    // Create application state
    let app_state = AppState::new();
    println!("Application state created successfully");

    tauri::Builder::default()
        .manage(app_state)
        .setup(|_app| {
            println!("Setting up application...");

            // For now, let's skip the system tray to avoid complications
            // We can add it back once the basic app is working

            println!("Application setup completed successfully");
            Ok(())
        })
        .on_window_event(|_window, event| match event {
            tauri::WindowEvent::CloseRequested { .. } => {
                // For now, let the window close normally
                // We can add hide-to-tray behavior later
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
            clear_logs,
            get_tracked_processes,
            add_process_config,
            remove_process_config,
            update_process_config,
            get_process_configs,
            kill_process,
            get_process_details
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
