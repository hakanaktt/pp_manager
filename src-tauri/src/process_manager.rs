use std::mem;
use std::collections::HashMap;
use winapi::shared::minwindef::{DWORD, FALSE};
use winapi::shared::ntdef::NULL;
use winapi::um::handleapi::{CloseHandle, INVALID_HANDLE_VALUE};
use winapi::um::processthreadsapi::{OpenProcess, SetPriorityClass, GetPriorityClass, TerminateProcess};
use winapi::um::tlhelp32::{
    CreateToolhelp32Snapshot, Process32First, Process32Next, PROCESSENTRY32, TH32CS_SNAPPROCESS,
};
use winapi::um::winnt::{
    PROCESS_QUERY_INFORMATION, PROCESS_SET_INFORMATION, PROCESS_QUERY_LIMITED_INFORMATION, PROCESS_TERMINATE,
};
use winapi::um::winbase::{SetProcessAffinityMask, GetProcessAffinityMask};
use winapi::um::processthreadsapi::GetCurrentProcess;
use winapi::um::securitybaseapi::GetTokenInformation;
use winapi::um::winnt::{TokenElevation, TOKEN_ELEVATION, TOKEN_QUERY};
use winapi::um::sysinfoapi::{GetSystemInfo, SYSTEM_INFO};
use chrono::{DateTime, Local};
use serde::{Serialize, Deserialize};



#[derive(Debug, Clone, PartialEq)]
pub enum ProcessState {
    NotFound,
    Found(u32), // Single PID
    FoundAndMonitoring(u32), // Single PID - process found and being continuously monitored
    SettingsApplied(u32, String), // Single PID, details of what was changed
    MultipleFound(Vec<u32>), // Multiple PIDs found
    MultipleMonitoring(Vec<u32>), // Multiple PIDs being monitored
    MultipleSettingsApplied(Vec<(u32, String)>), // Multiple PIDs with their respective changes

    // New states for multi-process management
    MultiProcessFound(HashMap<String, Vec<u32>>), // Process name -> PIDs
    MultiProcessMonitoring(HashMap<String, Vec<u32>>), // Process name -> PIDs being monitored
    MultiProcessSettingsApplied(HashMap<String, Vec<(u32, String)>>), // Process name -> (PID, changes)
    Error(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessInstance {
    pub pid: u32,
    pub last_applied_affinity: Option<u64>,
    pub last_applied_priority: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessDetails {
    pub pid: u32,
    pub name: String,
    pub current_priority: Option<u32>,
    pub current_affinity: Option<u64>,
    pub last_applied_priority: Option<u32>,
    pub last_applied_affinity: Option<u64>,
    pub is_tracked: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub timestamp: DateTime<Local>,
    pub message: String,
    pub level: String,
}

pub struct ProcessManager {
    tracked_processes: HashMap<u32, ProcessInstance>,
}

impl ProcessManager {
    pub fn new() -> Self {
        Self {
            tracked_processes: HashMap::new(),
        }
    }

    pub fn check_and_manage_process(&mut self, process_name: &str, affinity_mask: u64, priority_class: u32) -> ProcessState {
        match self.find_all_processes_by_name(process_name) {
            Ok(pids) => {
                if pids.is_empty() {
                    // No processes found, clear tracking
                    self.tracked_processes.clear();
                    ProcessState::NotFound
                } else if pids.len() == 1 {
                    // Single process - maintain backward compatibility
                    let pid = pids[0];
                    let is_new_process = !self.tracked_processes.contains_key(&pid);

                    if is_new_process {
                        self.tracked_processes.insert(pid, ProcessInstance {
                            pid,
                            last_applied_affinity: None,
                            last_applied_priority: None,
                        });
                    }

                    match self.monitor_and_reapply_settings_for_pid(pid, affinity_mask, priority_class) {
                        Ok(changes_applied) => {
                            if is_new_process {
                                ProcessState::Found(pid)
                            } else if !changes_applied.is_empty() {
                                ProcessState::SettingsApplied(pid, changes_applied)
                            } else {
                                ProcessState::FoundAndMonitoring(pid)
                            }
                        }
                        Err(err) => {
                            self.tracked_processes.remove(&pid);
                            ProcessState::Error(format!("Failed to monitor PID {}: {}", pid, err))
                        }
                    }
                } else {
                    // Multiple processes found
                    self.handle_multiple_processes(pids, affinity_mask, priority_class)
                }
            }
            Err(err) => ProcessState::Error(format!("Process enumeration failed: {}", err)),
        }
    }

    fn handle_multiple_processes(&mut self, pids: Vec<u32>, affinity_mask: u64, priority_class: u32) -> ProcessState {
        let mut new_processes = Vec::new();
        let mut monitoring_processes = Vec::new();
        let mut settings_applied = Vec::new();
        let mut errors = Vec::new();

        // Remove processes that are no longer running
        let current_pids: std::collections::HashSet<u32> = pids.iter().cloned().collect();
        self.tracked_processes.retain(|&pid, _| current_pids.contains(&pid));

        for pid in pids {
            let is_new_process = !self.tracked_processes.contains_key(&pid);

            if is_new_process {
                self.tracked_processes.insert(pid, ProcessInstance {
                    pid,
                    last_applied_affinity: None,
                    last_applied_priority: None,
                });
                new_processes.push(pid);
            }

            match self.monitor_and_reapply_settings_for_pid(pid, affinity_mask, priority_class) {
                Ok(changes_applied) => {
                    if !changes_applied.is_empty() {
                        settings_applied.push((pid, changes_applied));
                    } else {
                        monitoring_processes.push(pid);
                    }
                }
                Err(err) => {
                    self.tracked_processes.remove(&pid);
                    errors.push(format!("PID {}: {}", pid, err));
                }
            }
        }

        // Return appropriate state based on what happened
        if !errors.is_empty() {
            ProcessState::Error(format!("Errors: {}", errors.join("; ")))
        } else if !new_processes.is_empty() {
            ProcessState::MultipleFound(new_processes)
        } else if !settings_applied.is_empty() {
            ProcessState::MultipleSettingsApplied(settings_applied)
        } else {
            ProcessState::MultipleMonitoring(monitoring_processes)
        }
    }



    fn find_all_processes_by_name(&self, process_name: &str) -> Result<Vec<u32>, String> {
        unsafe {
            let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);
            if snapshot == INVALID_HANDLE_VALUE {
                return Err("Failed to create process snapshot".to_string());
            }

            let mut process_entry: PROCESSENTRY32 = mem::zeroed();
            process_entry.dwSize = mem::size_of::<PROCESSENTRY32>() as DWORD;

            if Process32First(snapshot, &mut process_entry) == FALSE {
                CloseHandle(snapshot);
                return Err("Failed to get first process".to_string());
            }

            let mut matching_pids = Vec::new();

            loop {
                let exe_name = self.get_exe_name_from_entry(&process_entry);
                if exe_name.to_lowercase() == process_name.to_lowercase() {
                    matching_pids.push(process_entry.th32ProcessID);
                }

                if Process32Next(snapshot, &mut process_entry) == FALSE {
                    break;
                }
            }

            CloseHandle(snapshot);
            Ok(matching_pids)
        }
    }

    fn get_exe_name_from_entry(&self, entry: &PROCESSENTRY32) -> String {
        let exe_file = &entry.szExeFile;
        let len = exe_file.iter().position(|&c| c == 0).unwrap_or(exe_file.len());
        let bytes: Vec<u8> = exe_file[..len].iter().map(|&c| c as u8).collect();
        String::from_utf8_lossy(&bytes).to_string()
    }

    fn monitor_and_reapply_settings_for_pid(&mut self, pid: u32, desired_affinity: u64, desired_priority: u32) -> Result<String, String> {
        unsafe {
            // Try with full permissions first
            let mut process_handle = OpenProcess(
                PROCESS_QUERY_INFORMATION | PROCESS_SET_INFORMATION,
                FALSE,
                pid,
            );

            // If that fails, try with limited permissions for read-only access
            if process_handle == NULL {
                process_handle = OpenProcess(
                    PROCESS_QUERY_LIMITED_INFORMATION,
                    FALSE,
                    pid,
                );

                if process_handle == NULL {
                    let is_admin = Self::is_running_as_administrator();
                    let process_info = Self::get_process_owner_info(pid);

                    if is_admin {
                        return Err(format!("Failed to open process PID {}: {}. Process may be protected or have exited.", pid, process_info));
                    } else {
                        return Err(format!("Access denied to PID {}: {}. Right-click the application and 'Run as Administrator'.", pid, process_info));
                    }
                } else {
                    CloseHandle(process_handle);
                    return Err(format!("Insufficient permissions for PID {}. Can read process but cannot modify settings. Run as Administrator.", pid));
                }
            }

            let mut changes_applied = Vec::new();

            // Check and reapply CPU affinity
            let mut current_affinity: usize = 0;
            let mut system_affinity: usize = 0;

            // Get the process instance for tracking
            let process_instance = self.tracked_processes.get(&pid).cloned();
            let last_applied_affinity = process_instance.as_ref().and_then(|p| p.last_applied_affinity);
            let last_applied_priority = process_instance.as_ref().and_then(|p| p.last_applied_priority);

            if GetProcessAffinityMask(process_handle, &mut current_affinity, &mut system_affinity) != 0 {
                let current_affinity_u64 = current_affinity as u64;

                if current_affinity_u64 != desired_affinity || last_applied_affinity != Some(desired_affinity) {
                    if SetProcessAffinityMask(process_handle, desired_affinity as u32) != 0 {
                        changes_applied.push(format!("CPU affinity: 0x{:X} → 0x{:X}", current_affinity_u64, desired_affinity));
                        // Update the tracked process instance
                        if let Some(instance) = self.tracked_processes.get_mut(&pid) {
                            instance.last_applied_affinity = Some(desired_affinity);
                        }
                    } else {
                        CloseHandle(process_handle);
                        return Err("Failed to set process affinity".to_string());
                    }
                }
            } else {
                CloseHandle(process_handle);
                return Err("Failed to get current process affinity".to_string());
            }

            // Check and reapply priority class
            let current_priority = GetPriorityClass(process_handle);
            if current_priority == 0 {
                CloseHandle(process_handle);
                return Err("Failed to get current process priority".to_string());
            }

            if current_priority != desired_priority || last_applied_priority != Some(desired_priority) {
                if SetPriorityClass(process_handle, desired_priority) != 0 {
                    changes_applied.push(format!("Priority: 0x{:X} → 0x{:X}", current_priority, desired_priority));
                    // Update the tracked process instance
                    if let Some(instance) = self.tracked_processes.get_mut(&pid) {
                        instance.last_applied_priority = Some(desired_priority);
                    }
                } else {
                    CloseHandle(process_handle);
                    return Err("Failed to set process priority".to_string());
                }
            }

            CloseHandle(process_handle);
            Ok(changes_applied.join(", "))
        }
    }

    pub fn is_running_as_administrator() -> bool {
        unsafe {
            use winapi::um::processthreadsapi::OpenProcessToken;

            let current_process = GetCurrentProcess();
            let mut token_handle = NULL;

            if OpenProcessToken(current_process, TOKEN_QUERY, &mut token_handle) == 0 {
                return false;
            }

            let mut elevation = TOKEN_ELEVATION { TokenIsElevated: 0 };
            let mut return_length = 0u32;

            let result = GetTokenInformation(
                token_handle,
                TokenElevation,
                &mut elevation as *mut _ as *mut _,
                std::mem::size_of::<TOKEN_ELEVATION>() as u32,
                &mut return_length,
            );

            CloseHandle(token_handle);

            result != 0 && elevation.TokenIsElevated != 0
        }
    }

    pub fn get_process_owner_info(pid: u32) -> String {
        // Try to get more information about why we can't access the process
        unsafe {
            let process_handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, FALSE, pid);
            if process_handle != NULL {
                CloseHandle(process_handle);
                "Process exists but requires elevated permissions to modify"
            } else {
                "Process may have exited or is a protected system process"
            }
        }.to_string()
    }

    /// Get the number of logical processors (CPU cores) available on the system
    pub fn get_system_cpu_count() -> u32 {
        unsafe {
            let mut system_info: SYSTEM_INFO = std::mem::zeroed();
            GetSystemInfo(&mut system_info);
            system_info.dwNumberOfProcessors
        }
    }

    /// Get the system's processor affinity mask to determine available cores
    pub fn get_system_affinity_mask() -> u64 {
        unsafe {
            let current_process = GetCurrentProcess();
            let mut process_affinity: usize = 0;
            let mut system_affinity: usize = 0;

            if GetProcessAffinityMask(current_process, &mut process_affinity, &mut system_affinity) != 0 {
                system_affinity as u64
            } else {
                // Fallback: assume all cores up to CPU count are available
                let cpu_count = Self::get_system_cpu_count();
                if cpu_count <= 64 {
                    (1u64 << cpu_count) - 1
                } else {
                    u64::MAX // All 64 bits set for systems with more than 64 cores
                }
            }
        }
    }

    /// Check and manage multiple different processes with individual settings
    pub fn check_and_manage_multiple_processes(&mut self, process_configs: &[(String, u64, u32)]) -> ProcessState {
        if process_configs.is_empty() {
            self.tracked_processes.clear();
            return ProcessState::NotFound;
        }

        let mut process_results: HashMap<String, Vec<u32>> = HashMap::new();
        let mut new_processes: HashMap<String, Vec<u32>> = HashMap::new();
        let mut monitoring_processes: HashMap<String, Vec<u32>> = HashMap::new();
        let mut settings_applied: HashMap<String, Vec<(u32, String)>> = HashMap::new();
        let mut errors = Vec::new();

        // Get all currently running PIDs for all target processes
        let mut all_current_pids = std::collections::HashSet::new();
        for (process_name, _affinity_mask, _priority_class) in process_configs {
            match self.find_all_processes_by_name(process_name) {
                Ok(pids) => {
                    if !pids.is_empty() {
                        process_results.insert(process_name.clone(), pids.clone());
                        all_current_pids.extend(pids.iter().cloned());
                    }
                }
                Err(err) => {
                    errors.push(format!("{}: {}", process_name, err));
                }
            }
        }

        // Remove tracked processes that are no longer running
        self.tracked_processes.retain(|&pid, _| all_current_pids.contains(&pid));

        // Process each target process
        for (process_name, affinity_mask, priority_class) in process_configs {
            if let Some(pids) = process_results.get(process_name) {
                let mut process_new = Vec::new();
                let mut process_monitoring = Vec::new();
                let mut process_settings = Vec::new();

                for &pid in pids {
                    let is_new_process = !self.tracked_processes.contains_key(&pid);

                    if is_new_process {
                        self.tracked_processes.insert(pid, ProcessInstance {
                            pid,
                            last_applied_affinity: None,
                            last_applied_priority: None,
                        });
                        process_new.push(pid);
                    }

                    match self.monitor_and_reapply_settings_for_pid(pid, *affinity_mask, *priority_class) {
                        Ok(changes_applied) => {
                            if !changes_applied.is_empty() {
                                process_settings.push((pid, changes_applied));
                            } else {
                                process_monitoring.push(pid);
                            }
                        }
                        Err(err) => {
                            self.tracked_processes.remove(&pid);
                            errors.push(format!("{} PID {}: {}", process_name, pid, err));
                        }
                    }
                }

                if !process_new.is_empty() {
                    new_processes.insert(process_name.clone(), process_new);
                } else if !process_settings.is_empty() {
                    settings_applied.insert(process_name.clone(), process_settings);
                } else if !process_monitoring.is_empty() {
                    monitoring_processes.insert(process_name.clone(), process_monitoring);
                }
            }
        }

        // Return appropriate state based on what happened
        if !errors.is_empty() {
            ProcessState::Error(format!("Errors: {}", errors.join("; ")))
        } else if !new_processes.is_empty() {
            ProcessState::MultiProcessFound(new_processes)
        } else if !settings_applied.is_empty() {
            ProcessState::MultiProcessSettingsApplied(settings_applied)
        } else if !monitoring_processes.is_empty() {
            ProcessState::MultiProcessMonitoring(monitoring_processes)
        } else {
            ProcessState::NotFound
        }
    }

    /// Get all currently tracked process instances
    pub fn get_tracked_processes(&self) -> &HashMap<u32, ProcessInstance> {
        &self.tracked_processes
    }

    /// Kill a process by PID
    pub fn kill_process(&mut self, pid: u32) -> Result<(), String> {
        unsafe {
            let process_handle = OpenProcess(
                PROCESS_TERMINATE,
                FALSE,
                pid,
            );

            if process_handle == NULL {
                let is_admin = Self::is_running_as_administrator();
                if is_admin {
                    return Err(format!("Failed to open process PID {} for termination. Process may have already exited or is protected.", pid));
                } else {
                    return Err(format!("Access denied to terminate PID {}. Run as Administrator to terminate processes.", pid));
                }
            }

            let result = TerminateProcess(process_handle, 1);
            CloseHandle(process_handle);

            if result == 0 {
                Err(format!("Failed to terminate process PID {}", pid))
            } else {
                // Remove from tracked processes since it's been terminated
                self.tracked_processes.remove(&pid);
                Ok(())
            }
        }
    }

    /// Get detailed process information by PID
    pub fn get_process_details(&self, pid: u32) -> Result<ProcessDetails, String> {
        unsafe {
            let process_handle = OpenProcess(
                PROCESS_QUERY_INFORMATION,
                FALSE,
                pid,
            );

            if process_handle == NULL {
                // Try with limited permissions
                let process_handle = OpenProcess(
                    PROCESS_QUERY_LIMITED_INFORMATION,
                    FALSE,
                    pid,
                );

                if process_handle == NULL {
                    return Err(format!("Failed to open process PID {} for information", pid));
                }
            }

            // Get process name
            let process_name = self.get_process_name_by_pid(pid).unwrap_or_else(|_| "Unknown".to_string());

            // Get current priority
            let current_priority = GetPriorityClass(process_handle);

            // Get current affinity
            let mut process_affinity: usize = 0;
            let mut system_affinity: usize = 0;
            let affinity_result = GetProcessAffinityMask(
                process_handle,
                &mut process_affinity,
                &mut system_affinity,
            );

            CloseHandle(process_handle);

            let tracked_info = self.tracked_processes.get(&pid);

            Ok(ProcessDetails {
                pid,
                name: process_name,
                current_priority: if current_priority != 0 { Some(current_priority) } else { None },
                current_affinity: if affinity_result != 0 { Some(process_affinity as u64) } else { None },
                last_applied_priority: tracked_info.and_then(|t| t.last_applied_priority),
                last_applied_affinity: tracked_info.and_then(|t| t.last_applied_affinity),
                is_tracked: tracked_info.is_some(),
            })
        }
    }

    /// Get process name by PID
    fn get_process_name_by_pid(&self, target_pid: u32) -> Result<String, String> {
        unsafe {
            let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);
            if snapshot == INVALID_HANDLE_VALUE {
                return Err("Failed to create process snapshot".to_string());
            }

            let mut process_entry: PROCESSENTRY32 = mem::zeroed();
            process_entry.dwSize = mem::size_of::<PROCESSENTRY32>() as DWORD;

            if Process32First(snapshot, &mut process_entry) != 0 {
                loop {
                    if process_entry.th32ProcessID == target_pid {
                        let name = std::ffi::CStr::from_ptr(process_entry.szExeFile.as_ptr())
                            .to_string_lossy()
                            .to_string();
                        CloseHandle(snapshot);
                        return Ok(name);
                    }

                    if Process32Next(snapshot, &mut process_entry) == 0 {
                        break;
                    }
                }
            }

            CloseHandle(snapshot);
            Err(format!("Process with PID {} not found", target_pid))
        }
    }
}

impl Default for ProcessManager {
    fn default() -> Self {
        Self::new()
    }
}
