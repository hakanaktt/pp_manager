use std::mem;
use winapi::shared::minwindef::{DWORD, FALSE};
use winapi::shared::ntdef::NULL;
use winapi::um::handleapi::{CloseHandle, INVALID_HANDLE_VALUE};
use winapi::um::processthreadsapi::{OpenProcess, SetPriorityClass, GetPriorityClass};
use winapi::um::tlhelp32::{
    CreateToolhelp32Snapshot, Process32First, Process32Next, PROCESSENTRY32, TH32CS_SNAPPROCESS,
};
use winapi::um::winnt::{
    PROCESS_QUERY_INFORMATION, PROCESS_SET_INFORMATION, PROCESS_QUERY_LIMITED_INFORMATION,
};
use winapi::um::winbase::{SetProcessAffinityMask, GetProcessAffinityMask};
use winapi::um::processthreadsapi::GetCurrentProcess;
use winapi::um::securitybaseapi::GetTokenInformation;
use winapi::um::winnt::{TokenElevation, TOKEN_ELEVATION, TOKEN_QUERY};
use winapi::um::sysinfoapi::{GetSystemInfo, SYSTEM_INFO};
use chrono::{DateTime, Local};



#[derive(Debug, Clone, PartialEq)]
pub enum ProcessState {
    NotFound,
    Found(u32), // PID
    FoundAndMonitoring(u32), // PID - process found and being continuously monitored
    SettingsApplied(u32, String), // PID, details of what was changed
    Error(String),
}

#[derive(Debug, Clone)]
pub struct LogEntry {
    pub timestamp: DateTime<Local>,
    pub message: String,
    pub level: String,
}

pub struct ProcessManager {
    last_known_pid: Option<u32>,
    last_applied_affinity: Option<u64>,
    last_applied_priority: Option<u32>,
}

impl ProcessManager {
    pub fn new() -> Self {
        Self {
            last_known_pid: None,
            last_applied_affinity: None,
            last_applied_priority: None,
        }
    }

    pub fn check_and_manage_process(&mut self, process_name: &str, affinity_mask: u64, priority_class: u32) -> ProcessState {
        match self.find_process_by_name(process_name) {
            Ok(Some(pid)) => {
                // Check if this is a new process or PID changed
                let is_new_process = self.last_known_pid != Some(pid);

                if is_new_process {
                    // New process detected, reset tracking
                    self.last_known_pid = Some(pid);
                    self.last_applied_affinity = None;
                    self.last_applied_priority = None;
                }

                // Continuously monitor and reapply settings
                match self.monitor_and_reapply_settings(pid, affinity_mask, priority_class) {
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
                        // Process might have exited, reset tracking
                        self.last_known_pid = None;
                        self.last_applied_affinity = None;
                        self.last_applied_priority = None;
                        ProcessState::Error(format!("Failed to monitor PID {}: {}", pid, err))
                    }
                }
            }
            Ok(None) => {
                // Process not found, reset tracking
                if self.last_known_pid.is_some() {
                    self.last_known_pid = None;
                    self.last_applied_affinity = None;
                    self.last_applied_priority = None;
                }
                ProcessState::NotFound
            }
            Err(err) => ProcessState::Error(format!("Process enumeration failed: {}", err)),
        }
    }

    fn find_process_by_name(&self, process_name: &str) -> Result<Option<u32>, String> {
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

            loop {
                let exe_name = self.get_exe_name_from_entry(&process_entry);
                if exe_name.to_lowercase() == process_name.to_lowercase() {
                    let pid = process_entry.th32ProcessID;
                    CloseHandle(snapshot);
                    return Ok(Some(pid));
                }

                if Process32Next(snapshot, &mut process_entry) == FALSE {
                    break;
                }
            }

            CloseHandle(snapshot);
            Ok(None)
        }
    }

    fn get_exe_name_from_entry(&self, entry: &PROCESSENTRY32) -> String {
        let exe_file = &entry.szExeFile;
        let len = exe_file.iter().position(|&c| c == 0).unwrap_or(exe_file.len());
        let bytes: Vec<u8> = exe_file[..len].iter().map(|&c| c as u8).collect();
        String::from_utf8_lossy(&bytes).to_string()
    }

    fn monitor_and_reapply_settings(&mut self, pid: u32, desired_affinity: u64, desired_priority: u32) -> Result<String, String> {
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

            if GetProcessAffinityMask(process_handle, &mut current_affinity, &mut system_affinity) != 0 {
                let current_affinity_u64 = current_affinity as u64;

                if current_affinity_u64 != desired_affinity || self.last_applied_affinity != Some(desired_affinity) {
                    if SetProcessAffinityMask(process_handle, desired_affinity as u32) != 0 {
                        changes_applied.push(format!("CPU affinity: 0x{:X} → 0x{:X}", current_affinity_u64, desired_affinity));
                        self.last_applied_affinity = Some(desired_affinity);
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

            if current_priority != desired_priority || self.last_applied_priority != Some(desired_priority) {
                if SetPriorityClass(process_handle, desired_priority) != 0 {
                    changes_applied.push(format!("Priority: 0x{:X} → 0x{:X}", current_priority, desired_priority));
                    self.last_applied_priority = Some(desired_priority);
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
}

impl Default for ProcessManager {
    fn default() -> Self {
        Self::new()
    }
}
