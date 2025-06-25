#![windows_subsystem = "windows"]

use eframe::egui;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use chrono::Local;
use tray_icon::{TrayIconBuilder, menu::{Menu, MenuItem, PredefinedMenuItem, MenuEvent}};
use serde::{Serialize, Deserialize};
use std::fs;

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



fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = load_config();

    // Create the application state
    let app_state = Arc::new(Mutex::new(AppState::new()));
    let show_window = Arc::new(Mutex::new(false)); // Start minimized to tray
    let should_exit = Arc::new(Mutex::new(false));

    // Create tray menu with unique IDs
    let tray_menu = Menu::new();
    let show_item = MenuItem::with_id("show", "Show Settings", true, None);
    let hide_item = MenuItem::with_id("hide", "Hide Settings", true, None);
    let separator = PredefinedMenuItem::separator();
    let quit_item = MenuItem::with_id("exit", "Exit", true, None);

    tray_menu.append_items(&[
        &show_item,
        &hide_item,
        &separator,
        &quit_item,
    ])?;

    // Load icon
    let icon = load_icon();

    // Create tray icon
    let _tray_icon = TrayIconBuilder::new()
        .with_menu(Box::new(tray_menu))
        .with_tooltip("Process Priority Manager")
        .with_icon(icon)
        .build()?;

    println!("Tray icon created successfully. Look for the blue 'P' icon in your system tray.");

    // Implement tray menu event handling using event handlers
    let show_window_clone = Arc::clone(&show_window);
    let should_exit_clone = Arc::clone(&should_exit);

    MenuEvent::set_event_handler(Some(move |event: MenuEvent| {
        println!("Menu event received: {:?}", event.id().as_ref());
        match event.id().as_ref() {
            "show" => {
                println!("Setting show_window to true");
                let mut show = show_window_clone.lock().unwrap();
                *show = true;
                println!("show_window is now: {}", *show);
            }
            "hide" => {
                println!("Setting show_window to false");
                let mut show = show_window_clone.lock().unwrap();
                *show = false;
                println!("show_window is now: {}", *show);
            }
            "exit" => {
                println!("Setting should_exit to true");
                let mut exit = should_exit_clone.lock().unwrap();
                *exit = true;
                println!("should_exit is now: {}", *exit);
            }
            _ => {
                println!("Unknown menu event: {}", event.id().as_ref());
            }
        }
    }));

    // Start the GUI application
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([400.0, 500.0])
            .with_min_inner_size([350.0, 400.0])
            .with_resizable(true)
            .with_title("Process Priority Manager")
            .with_visible(false), // Start hidden in tray
        ..Default::default()
    };

    eframe::run_native(
        "Process Priority Manager",
        options,
        Box::new(move |cc| {
            Box::new(PPManagerApp::new(cc, app_state, show_window, should_exit, config))
        }),
    ).map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
}

fn load_icon() -> tray_icon::Icon {
    // Create a simple 16x16 icon with a blue background and white "P" for Process
    let mut rgba = vec![0u8; 16 * 16 * 4];

    for y in 0..16 {
        for x in 0..16 {
            let idx = (y * 16 + x) * 4;

            // Create a simple "P" pattern
            let is_p_pixel = match (x, y) {
                // Left vertical line
                (2, 2..=13) => true,
                // Top horizontal line
                (2..=10, 2) => true,
                // Middle horizontal line
                (2..=8, 7) => true,
                // Right vertical line (top part)
                (10, 2..=7) => true,
                _ => false,
            };

            if is_p_pixel {
                // White "P"
                rgba[idx] = 255;     // R
                rgba[idx + 1] = 255; // G
                rgba[idx + 2] = 255; // B
                rgba[idx + 3] = 255; // A
            } else {
                // Blue background
                rgba[idx] = 0;       // R
                rgba[idx + 1] = 100; // G
                rgba[idx + 2] = 200; // B
                rgba[idx + 3] = 255; // A
            }
        }
    }

    tray_icon::Icon::from_rgba(rgba, 16, 16).unwrap_or_else(|_| {
        // Ultimate fallback - solid blue
        let mut fallback_rgba = Vec::new();
        for _ in 0..(16 * 16) {
            fallback_rgba.extend_from_slice(&[0u8, 100u8, 200u8, 255u8]);
        }
        tray_icon::Icon::from_rgba(fallback_rgba, 16, 16).unwrap()
    })
}

#[derive(Clone)]
struct AppState {
    process_manager: Arc<Mutex<ProcessManager>>,
    is_running: Arc<Mutex<bool>>,
    logs: Arc<Mutex<Vec<LogEntry>>>,
}

impl AppState {
    fn new() -> Self {
        Self {
            process_manager: Arc::new(Mutex::new(ProcessManager::new())),
            is_running: Arc::new(Mutex::new(false)),
            logs: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

struct PPManagerApp {
    app_state: Arc<Mutex<AppState>>,
    show_window: Arc<Mutex<bool>>,
    should_exit: Arc<Mutex<bool>>,
    config: Config,
    auto_scroll: bool,
    priority_selection: usize,
    window_visible: bool,
}

impl PPManagerApp {
    fn new(_cc: &eframe::CreationContext<'_>, app_state: Arc<Mutex<AppState>>, show_window: Arc<Mutex<bool>>, should_exit: Arc<Mutex<bool>>, config: Config) -> Self {
        // Determine priority selection index based on config
        let priority_selection = match config.priority_class {
            0x00000040 => 0, // IDLE
            0x00004000 => 1, // BELOW_NORMAL
            0x00000020 => 2, // NORMAL
            0x00008000 => 3, // ABOVE_NORMAL
            0x00000080 => 4, // HIGH
            0x00000100 => 5, // REALTIME
            _ => 3, // Default to ABOVE_NORMAL
        };

        Self {
            app_state,
            show_window,
            should_exit,
            config,
            auto_scroll: true,
            priority_selection,
            window_visible: false, // Start hidden
        }
    }

    fn start_monitoring(&self) {
        let app_state = self.app_state.lock().unwrap();
        let mut is_running = app_state.is_running.lock().unwrap();
        if *is_running {
            return;
        }
        *is_running = true;
        drop(is_running);

        let process_manager = Arc::clone(&app_state.process_manager);
        let is_running_clone = Arc::clone(&app_state.is_running);
        let logs_clone = Arc::clone(&app_state.logs);
        drop(app_state);

        let target_process = self.config.target_process.clone();
        let affinity_mask = self.calculate_affinity_mask();
        let priority_class = self.config.priority_class;

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
    }

    fn stop_monitoring(&self) {
        let app_state = self.app_state.lock().unwrap();
        let mut is_running = app_state.is_running.lock().unwrap();
        *is_running = false;
    }

    fn clear_logs(&self) {
        let app_state = self.app_state.lock().unwrap();
        app_state.logs.lock().unwrap().clear();
    }

    fn calculate_affinity_mask(&self) -> u64 {
        self.config.core_selections
            .iter()
            .enumerate()
            .fold(0u64, |acc, (i, &selected)| {
                if selected { acc | (1u64 << i) } else { acc }
            })
    }

    fn get_priority_options() -> Vec<(&'static str, u32)> {
        vec![
            ("IDLE", 0x00000040),
            ("BELOW_NORMAL", 0x00004000),
            ("NORMAL", 0x00000020),
            ("ABOVE_NORMAL", 0x00008000),
            ("HIGH", 0x00000080),
            ("REALTIME", 0x00000100),
        ]
    }

    fn get_selected_cores_text(&self) -> String {
        let selected: Vec<String> = self.config.core_selections
            .iter()
            .enumerate()
            .filter_map(|(i, &selected)| if selected { Some(i.to_string()) } else { None })
            .collect();

        if selected.is_empty() {
            "None".to_string()
        } else {
            selected.join(", ")
        }
    }

    fn apply_preset(&mut self, preset_name: &str) {
        let core_count = self.config.core_selections.len();

        match preset_name {
            "Performance Cores" => {
                self.config.core_selections.fill(false);
                // For 12-core systems, select cores 6-9 (middle-high performance cores)
                // For other systems, select middle cores
                let perf_start = if core_count >= 12 { 6 } else if core_count >= 8 { 4 } else { core_count / 2 };
                let perf_count = if core_count >= 12 { 4 } else { 2 };
                let perf_end = (perf_start + perf_count).min(core_count);
                for i in perf_start..perf_end {
                    self.config.core_selections[i] = true;
                }
                self.priority_selection = 3; // ABOVE_NORMAL
                self.config.priority_class = 0x00008000;
            }
            "Efficiency Cores" => {
                self.config.core_selections.fill(false);
                // For 12-core systems, select cores 0-3 (efficiency cores)
                // For other systems, select first quarter
                let eff_count = if core_count >= 12 { 4 } else if core_count >= 8 { 4 } else { (core_count / 2).max(1) };
                for i in 0..eff_count.min(core_count) {
                    self.config.core_selections[i] = true;
                }
                self.priority_selection = 2; // NORMAL
                self.config.priority_class = 0x00000020;
            }
            "High Performance" => {
                self.config.core_selections.fill(false);
                // For 12-core systems, select cores 10-11 (highest performance cores)
                // For other systems, select last 2 cores
                let high_count = if core_count >= 12 { 2 } else { 2 };
                let high_start = if core_count >= high_count { core_count - high_count } else { 0 };
                for i in high_start..core_count {
                    self.config.core_selections[i] = true;
                }
                self.priority_selection = 4; // HIGH
                self.config.priority_class = 0x00000080;
            }
            "All Cores" => {
                self.config.core_selections.fill(true);
                self.priority_selection = 3; // ABOVE_NORMAL
                self.config.priority_class = 0x00008000;
            }
            "Maximum Performance" => {
                // Use all available cores with highest safe priority
                self.config.core_selections.fill(true);
                self.priority_selection = 4; // HIGH
                self.config.priority_class = 0x00000080;
            }
            "Half Cores" => {
                // New preset: Use half of the available cores (good for leaving resources for other tasks)
                self.config.core_selections.fill(false);
                let half_count = (core_count / 2).max(1);
                for i in 0..half_count {
                    self.config.core_selections[i] = true;
                }
                self.priority_selection = 3; // ABOVE_NORMAL
                self.config.priority_class = 0x00008000;
            }
            _ => {}
        }
    }
}

impl eframe::App for PPManagerApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        // Request repaint every second to update the UI
        ctx.request_repaint_after(Duration::from_secs(1));

        // Check if we should exit the application
        if *self.should_exit.lock().unwrap() {
            // Stop monitoring first
            self.stop_monitoring();
            // Close the application
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            return;
        }

        // Handle window visibility
        let should_show = *self.show_window.lock().unwrap();
        if should_show != self.window_visible {
            self.window_visible = should_show;
            if should_show {
                // Show and focus the window
                println!("Attempting to show window...");
                ctx.send_viewport_cmd(egui::ViewportCommand::Visible(true));
                ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(false));
                ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
                // Also try to bring to front
                ctx.send_viewport_cmd(egui::ViewportCommand::WindowLevel(egui::WindowLevel::AlwaysOnTop));
                ctx.send_viewport_cmd(egui::ViewportCommand::WindowLevel(egui::WindowLevel::Normal));
                println!("Window show commands sent");
            } else {
                // Hide the window
                println!("Attempting to hide window...");
                ctx.send_viewport_cmd(egui::ViewportCommand::Visible(false));
                println!("Window hide commands sent");
            }
        }

        // Handle window close event - minimize to tray instead of exit
        if ctx.input(|i| i.viewport().close_requested()) {
            *self.show_window.lock().unwrap() = false;
            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
        }

        // Handle keyboard shortcut to toggle window (Ctrl+Shift+P)
        if ctx.input(|i| i.key_pressed(egui::Key::P) && i.modifiers.ctrl && i.modifiers.shift) {
            let mut show = self.show_window.lock().unwrap();
            *show = !*show;
            println!("Keyboard shortcut pressed - show_window is now: {}", *show);
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("🎯 Process Priority Manager");

            // System and Administrator status indicators
            ui.horizontal(|ui| {
                let cpu_count = ProcessManager::get_system_cpu_count();
                let affinity_mask = ProcessManager::get_system_affinity_mask();
                ui.colored_label(egui::Color32::BLUE, format!("🖥️ System: {} CPU cores (Mask: 0x{:X})", cpu_count, affinity_mask));
            });

            ui.horizontal(|ui| {
                let is_admin = process_manager::ProcessManager::is_running_as_administrator();
                if is_admin {
                    ui.colored_label(egui::Color32::GREEN, "🛡️ Running as Administrator");
                } else {
                    ui.colored_label(egui::Color32::YELLOW, "⚠️ Not running as Administrator");
                    ui.label("(May have limited access to system processes)");
                }
            });

            ui.separator();

            // Configuration section
            ui.group(|ui| {
                ui.label("⚙️ Configuration:");

                let app_state = self.app_state.lock().unwrap();
                let is_running = *app_state.is_running.lock().unwrap();
                drop(app_state);

                ui.add_enabled_ui(!is_running, |ui| {
                    // Target process configuration
                    ui.horizontal(|ui| {
                        ui.label("Target Process:");
                        ui.text_edit_singleline(&mut self.config.target_process);
                        ui.label("(e.g., icad.exe, notepad.exe)");
                    });

                    ui.separator();

                    // CPU Affinity configuration
                    ui.label("🖥️ CPU Affinity (Select cores):");
                    ui.horizontal_wrapped(|ui| {
                        for (i, selected) in self.config.core_selections.iter_mut().enumerate() {
                            ui.checkbox(selected, format!("Core {}", i));
                        }
                    });

                    let affinity_mask = self.calculate_affinity_mask();
                    ui.horizontal(|ui| {
                        ui.label("Selected cores:");
                        ui.colored_label(egui::Color32::BLUE, self.get_selected_cores_text());
                        ui.label(format!("(Mask: 0x{:X})", affinity_mask));
                    });

                    ui.separator();

                    // Priority configuration
                    ui.label("⚡ Process Priority:");
                    let priority_options = Self::get_priority_options();
                    egui::ComboBox::from_label("Priority Class")
                        .selected_text(priority_options[self.priority_selection].0)
                        .show_ui(ui, |ui| {
                            for (i, (name, value)) in priority_options.iter().enumerate() {
                                let response = ui.selectable_value(&mut self.priority_selection, i, *name);
                                if response.clicked() {
                                    self.config.priority_class = *value;
                                }
                            }
                        });

                    ui.horizontal(|ui| {
                        ui.label("Priority value:");
                        ui.colored_label(egui::Color32::BLUE, format!("0x{:08X}", self.config.priority_class));
                    });

                    ui.separator();

                    // Preset configurations
                    ui.label("🎯 Quick Presets:");
                    ui.horizontal_wrapped(|ui| {
                        if ui.button("⚡ Performance Cores").clicked() {
                            self.apply_preset("Performance Cores");
                        }
                        if ui.button("🔋 Efficiency Cores").clicked() {
                            self.apply_preset("Efficiency Cores");
                        }
                        if ui.button("🏆 High Performance").clicked() {
                            self.apply_preset("High Performance");
                        }
                    });
                    ui.horizontal_wrapped(|ui| {
                        if ui.button("📊 Half Cores").clicked() {
                            self.apply_preset("Half Cores");
                        }
                        if ui.button("🌐 All Cores").clicked() {
                            self.apply_preset("All Cores");
                        }
                        if ui.button("🚀 Maximum Performance").clicked() {
                            self.apply_preset("Maximum Performance");
                        }
                    });
                });

                let affinity_mask = self.calculate_affinity_mask();
                if is_running {
                    ui.colored_label(egui::Color32::YELLOW, "⚠️ Stop monitoring to change configuration");
                } else if affinity_mask == 0 {
                    ui.colored_label(egui::Color32::RED, "⚠️ Warning: No CPU cores selected!");
                }

                // Show administrator recommendation for certain processes
                if !process_manager::ProcessManager::is_running_as_administrator() {
                    ui.separator();
                    ui.colored_label(egui::Color32::YELLOW, "💡 Tip: Run as Administrator for full access to system processes");
                }
            });

            ui.separator();

            // Control buttons
            ui.horizontal(|ui| {
                let app_state = self.app_state.lock().unwrap();
                let is_running = *app_state.is_running.lock().unwrap();
                drop(app_state);

                let affinity_mask = self.calculate_affinity_mask();
                let can_start = !is_running && affinity_mask > 0 && !self.config.target_process.trim().is_empty();

                if is_running {
                    if ui.button("⏹ Stop Monitoring").clicked() {
                        self.stop_monitoring();
                    }
                    ui.colored_label(egui::Color32::GREEN, "🟢 Monitoring Active");
                } else {
                    ui.add_enabled_ui(can_start, |ui| {
                        if ui.button("▶ Start Monitoring").clicked() {
                            self.start_monitoring();
                            save_config(&self.config);
                        }
                    });

                    if !can_start && affinity_mask == 0 {
                        ui.colored_label(egui::Color32::RED, "🔴 Cannot start: No cores selected");
                    } else if !can_start && self.config.target_process.trim().is_empty() {
                        ui.colored_label(egui::Color32::RED, "🔴 Cannot start: No target process");
                    } else {
                        ui.colored_label(egui::Color32::RED, "🔴 Monitoring Stopped");
                    }
                }
            });

            ui.horizontal(|ui| {
                if ui.button("🗑 Clear Logs").clicked() {
                    self.clear_logs();
                }

                if ui.button("📱 Minimize to Tray").clicked() {
                    *self.show_window.lock().unwrap() = false;
                }

                ui.checkbox(&mut self.auto_scroll, "Auto-scroll logs");

                let app_state = self.app_state.lock().unwrap();
                let logs = app_state.logs.lock().unwrap();
                ui.label(format!("📊 {} log entries", logs.len()));
                drop(logs);
                drop(app_state);
            });

            ui.separator();

            // Tray information
            ui.horizontal(|ui| {
                ui.label("💡 Tip:");
                ui.label("Use Ctrl+Shift+P to show/hide this window, or use the system tray icon.");
            });

            ui.separator();

            // Logs section
            ui.label("📋 Activity Logs:");

            egui::ScrollArea::vertical()
                .auto_shrink([false; 2])
                .stick_to_bottom(self.auto_scroll)
                .show(ui, |ui| {
                    let app_state = self.app_state.lock().unwrap();
                    let logs = app_state.logs.lock().unwrap();

                    if logs.is_empty() {
                        ui.label("No logs yet. Start monitoring to see activity.");
                    } else {
                        // Show last 20 logs to prevent UI slowdown
                        let start_idx = if logs.len() > 20 { logs.len() - 20 } else { 0 };
                        for log in logs.iter().skip(start_idx) {
                            ui.horizontal(|ui| {
                                let color = match log.level.as_str() {
                                    "ERROR" => egui::Color32::RED,
                                    "SUCCESS" => egui::Color32::GREEN,
                                    "REAPPLY" => egui::Color32::from_rgb(255, 165, 0), // Orange
                                    "MONITOR" => egui::Color32::from_rgb(128, 128, 128), // Gray
                                    "INFO" => egui::Color32::from_rgb(100, 150, 255),
                                    _ => egui::Color32::GRAY,
                                };

                                ui.colored_label(color, &format!("[{}]", log.level));
                                ui.label(log.timestamp.format("%H:%M:%S").to_string());
                                ui.label(&log.message);
                            });
                        }
                    }
                });
        });
    }
}
