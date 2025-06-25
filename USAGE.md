# ICAD W11 Runner - Usage Guide

## Quick Start

1. **Build the application**:
   ```bash
   cargo build --release
   ```

2. **Run the application**:
   ```bash
   cargo run --release
   ```
   Or directly run the executable:
   ```bash
   ./target/release/icad_w11_runner.exe
   ```

## Testing the Application

### Option 1: Using the Test Setup
1. Run the test setup batch file:
   ```bash
   test_setup.bat
   ```
   This will build a test process and copy it as `iced.exe`.

2. Start the ICAD W11 Runner GUI application
3. Click "▶ Start Monitoring"
4. In another terminal, run:
   ```bash
   iced.exe
   ```
5. Watch the logs in the GUI to see process detection and settings application

### Option 2: Manual Testing
1. Build the test process manually:
   ```bash
   rustc test_process.rs -o test_process.exe
   copy test_process.exe iced.exe
   ```

2. Follow steps 2-5 from Option 1

## GUI Features

### Configuration Panel
- **Target Process**: Editable field for any process name (e.g., iced.exe, notepad.exe, chrome.exe)
- **CPU Affinity**: Individual checkboxes for cores 0-7 with real-time mask display
- **Priority Class**: Dropdown menu with 6 priority levels from IDLE to REALTIME
- **Quick Presets**: One-click buttons for common configurations
- **Validation**: Real-time warnings for invalid configurations
- **Note**: All settings can only be changed when monitoring is stopped

### Control Panel
- **▶ Start Monitoring**: Begin watching for the target process
- **⏹ Stop Monitoring**: Stop the monitoring loop
- **🗑 Clear Logs**: Clear all log entries
- **Auto-scroll logs**: Checkbox to automatically scroll to newest log entries
- **Log counter**: Shows total number of log entries

### Status Indicators
- **🟢 Monitoring Active**: Green indicator when actively monitoring
- **🔴 Monitoring Stopped**: Red indicator when not monitoring
- **⚠️ Warning**: Yellow warning when trying to change settings while monitoring

### Log Display
- **Color-coded messages**:
  - 🔵 **INFO**: General information (process not found)
  - 🟢 **SUCCESS**: Process found and settings applied successfully
  - 🔴 **ERROR**: Errors during process management
- **Timestamps**: Each log entry shows the time it occurred
- **Auto-scroll**: Optional automatic scrolling to newest entries

## Expected Behavior

When monitoring is active and the target process is detected:

```
[INFO] 14:30:15 iced.exe not running.
[SUCCESS] 14:30:45 iced.exe found (PID: 5824). Settings applied successfully.
[INFO] 14:32:10 iced.exe not running.
[SUCCESS] 14:33:22 iced.exe found (PID: 6120). Settings applied successfully.
```

## Technical Details

### Process Settings Applied (Configurable)
- **CPU Affinity**: User-selected cores with calculated mask (e.g., 0x30 for cores 4-5)
- **Priority**: User-selected priority class from IDLE to REALTIME

### Configuration Examples
1. **Gaming Performance**: Cores 4-5, ABOVE_NORMAL priority
2. **Background Task**: Cores 0-1, BELOW_NORMAL priority
3. **High Performance**: Cores 6-7, HIGH priority
4. **Maximum Resources**: All cores, ABOVE_NORMAL priority

### Quick Preset Configurations

The application includes four preset buttons for common scenarios:

1. **Performance Cores (4-5)**
   - Cores: 4, 5 (Mask: 0x30)
   - Priority: ABOVE_NORMAL (0x00008000)
   - Use case: Gaming, high-performance applications

2. **Efficiency Cores (0-3)**
   - Cores: 0, 1, 2, 3 (Mask: 0x0F)
   - Priority: NORMAL (0x00000020)
   - Use case: Background tasks, energy efficiency

3. **High Performance (6-7)**
   - Cores: 6, 7 (Mask: 0xC0)
   - Priority: HIGH (0x00000080)
   - Use case: Critical applications, real-time processing

4. **All Cores**
   - Cores: 0-7 (Mask: 0xFF)
   - Priority: ABOVE_NORMAL (0x00008000)
   - Use case: CPU-intensive tasks, maximum performance

### Monitoring Behavior
- **Polling Interval**: 1 second
- **State Tracking**: Only logs when process state changes (found ↔ not found)
- **Automatic Re-application**: Settings are re-applied if process restarts with new PID

### Windows API Functions Used
- `CreateToolhelp32Snapshot` - Create process snapshot
- `Process32First/Next` - Enumerate processes
- `OpenProcess` - Get process handle
- `SetProcessAffinityMask` - Set CPU affinity
- `SetPriorityClass` - Set process priority
- `CloseHandle` - Clean up handles

## Troubleshooting

### Common Issues
1. **"Access Denied" errors**: Run as Administrator if targeting system processes
2. **Process not detected**: Ensure the exact process name matches (case-insensitive)
3. **Settings not applied**: Check that the process allows affinity/priority changes

### Debug Tips
- Use the test process (`iced.exe`) to verify functionality
- Check Windows Task Manager to confirm affinity and priority changes
- Monitor the GUI logs for detailed error messages
