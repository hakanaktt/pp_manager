# Process Priority Manager

A lightweight Rust-based Windows application with minimal GUI that continuously monitors and manages the processes, automatically setting its CPU affinity and priority.

## Features

- **Minimal GUI**: Clean, simple interface built with egui
- **Continuous Process Monitoring**: Watches for any configurable target process every 1 second
- **Automatic Settings Reapplication**: Detects and corrects when Windows or applications reset affinity/priority
- **Dynamic CPU Detection**: Automatically detects and supports all available CPU cores on your system
- **Configurable Priority**: Choose from 6 priority levels (IDLE to REALTIME)
- **Quick Presets**: Pre-configured settings for common scenarios
- **Real-time Logging**: Shows process detection, monitoring, and settings reapplication
- **Smart State Tracking**: Only logs important events to reduce spam
- **PID Change Detection**: Automatically handles process restarts with new PIDs
- **Input Validation**: Prevents invalid configurations and provides helpful warnings
- **Resource Safe**: Proper Windows API handle management to prevent resource leaks

## Requirements

- Windows 10+ x64
- Rust toolchain (for building from source)

## Building

```bash
cargo build --release
```

## Running

```bash
cargo run --release
```

Or run the compiled executable:
```bash
./target/release/icad_w11_runner.exe
```

## Usage

1. **Start the Application**: Launch the executable to open the minimal GUI
2. **Configure Target**: The default target process is "iced.exe" (configurable in the GUI)
3. **Start Monitoring**: Click "▶ Start Monitoring" to begin watching for the process
4. **View Logs**: Monitor the activity logs to see when the process is detected and settings applied
5. **Stop Monitoring**: Click "⏹ Stop Monitoring" to stop the watcher

## GUI Features

### Configuration Panel
- **Target Process**: Editable field for any process name (e.g., iced.exe, notepad.exe)
- **CPU Affinity**: Individual checkboxes for all available CPU cores with real-time mask calculation
- **Priority Class**: Dropdown selection from 6 priority levels
- **System Information**: Shows detected CPU count and system affinity mask
- **Quick Presets**: One-click configurations optimized for your system:
  - ⚡ Performance Cores: Middle-high performance cores + ABOVE_NORMAL priority
  - 🔋 Efficiency Cores: First quarter of cores + NORMAL priority
  - 🏆 High Performance: Highest performance cores + HIGH priority
  - 📊 Half Cores: Use half of available cores + ABOVE_NORMAL priority
  - 🌐 All Cores: Use all available cores + ABOVE_NORMAL priority
  - 🚀 Maximum Performance: All cores + HIGH priority

### Control Panel
- **Start/Stop Monitoring**: With validation to prevent invalid configurations
- **Clear Logs**: Remove all log entries
- **Auto-scroll**: Toggle automatic scrolling to newest entries
- **Log Counter**: Shows total number of log entries

### Status Indicators
- **🟢 Monitoring Active**: Green indicator when actively monitoring
- **🔴 Monitoring Stopped**: Red indicator when not monitoring
- **⚠️ Configuration Warnings**: Yellow warnings for invalid settings
- **🔴 Validation Errors**: Red errors preventing startup

### Activity Logs
- **Color-coded messages**:
  - 🔵 **INFO**: General information (process not found)
  - 🟢 **SUCCESS**: Process found and initial settings applied
  - 🟠 **REAPPLY**: Settings were reapplied due to changes detected
  - 🔘 **MONITOR**: Process is being continuously monitored
  - 🔴 **ERROR**: Errors during process management
- **Timestamps**: Each log entry shows the time it occurred
- **Detailed Change Tracking**: Shows exactly what settings were changed and their values
- **Smart Logging**: Only logs important events to prevent spam

## Technical Details

### CPU Affinity (Dynamic Detection)
- **Auto-Detection**: Automatically detects all available CPU cores on your system
- **Selectable Cores**: Any combination of detected cores (e.g., 0-11 for 12-core systems)
- **Real-time Mask**: Automatically calculated from selected cores
- **System Information**: Shows CPU count and system affinity mask
- **Examples for 12-core system**:
  - Performance cores (6-9) → Mask 0x3C0 (binary: 1111000000)
  - Efficiency cores (0-3) → Mask 0x0F (binary: 1111)
  - All cores (0-11) → Mask 0xFFF (binary: 111111111111)

### Process Priority (Configurable)
- **Available Classes**:
  - IDLE (0x00000040) - Lowest priority
  - BELOW_NORMAL (0x00004000) - Below normal priority
  - NORMAL (0x00000020) - Standard priority
  - ABOVE_NORMAL (0x00008000) - Above normal priority
  - HIGH (0x00000080) - High priority
  - REALTIME (0x00000100) - Highest priority (use with caution)

### Windows API Usage
The application uses direct Windows API calls:
- `CreateToolhelp32Snapshot` - Create process snapshot
- `Process32First/Next` - Enumerate running processes
- `OpenProcess` - Get process handle with query and set permissions
- `GetProcessAffinityMask` - Check current CPU affinity
- `SetProcessAffinityMask` - Set CPU affinity
- `GetPriorityClass` - Check current process priority
- `SetPriorityClass` - Set process priority
- `CloseHandle` - Proper resource cleanup (every cycle)

## Behavior

```
[INFO] 14:30:15 iced.exe not running.
[SUCCESS] 14:30:45 iced.exe found (PID: 5824). Initial settings applied.
[REAPPLY] 14:30:52 iced.exe (PID: 5824) settings reapplied: CPU affinity: 0xFF → 0x30
[INFO] 14:32:10 iced.exe not running.
[SUCCESS] 14:33:22 iced.exe found (PID: 6120). Initial settings applied.
```

### Enhanced Monitoring Behavior
The application will:
- **Continuously monitor** every 1 second for the target process
- **Apply settings immediately** when process is first detected
- **Verify and reapply** settings every cycle to counter:
  - Windows automatic affinity resets
  - Application launcher interference
  - Other process management tools
  - Process restarts with new PIDs
- **Smart logging** only when important events occur:
  - Process state changes (found ↔ not found)
  - Settings reapplied due to external changes
  - Errors during monitoring
- **Resource efficient** with proper handle management
- **Continue running** until manually stopped

## Testing

For testing purposes, a test setup is provided:

1. **Build the test process**:
   ```bash
   rustc test_process.rs -o test_process.exe
   copy test_process.exe iced.exe
   ```

2. **Or use the provided batch script**:
   ```bash
   test_setup.bat
   ```

3. **Test the application**:
   - Start the ICAD W11 Runner GUI
   - Click "▶ Start Monitoring"
   - Run `iced.exe` in another terminal
   - Observe the logs in the GUI showing process detection and settings application

## Screenshots

The minimal GUI provides:
- Configuration panel showing target process and CPU affinity settings
- Start/Stop monitoring controls with visual status indicators
- Real-time activity logs with color-coded messages
- Auto-scroll option for continuous monitoring

## License

This project is provided as-is for educational and utility purposes.
