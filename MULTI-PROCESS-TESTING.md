# Multi-Process Tracking Testing Guide

This guide explains how to test the multi-process tracking functionality of the Process Priority Manager.

## Features Implemented

### 1. Multi-Process Detection and Tracking
- The app now detects and tracks multiple instances of the same process name
- Each process instance is tracked individually with its own PID
- Settings are applied to all instances simultaneously

### 2. Enhanced UI
- **Tracked Process Instances Section**: Shows all currently monitored process instances
- **Process Details**: Displays PID, applied affinity, and priority for each instance
- **Process Management**: Individual controls for each process instance

### 3. Process Instance Management
- **Details Button**: View detailed information about a specific process instance
- **Exclude Button**: Remove specific process instances from monitoring
- **Show All Button**: Re-include previously excluded processes
- **Instance Counter**: Shows total tracked and excluded process counts

### 4. Enhanced Logging
- Logs now clearly identify when multiple processes are found
- Shows process counts and individual PIDs in log messages
- Distinguishes between single and multi-process scenarios

## Testing Scenarios

### Scenario 1: Basic Multi-Process Detection

1. **Setup**: Run the test script to create multiple notepad instances:
   ```bash
   # PowerShell
   .\test-multiple-processes.ps1
   
   # Or Batch
   .\test-multiple-processes.bat
   ```

2. **Test Steps**:
   - Open the Process Priority Manager
   - Set target process to `notepad.exe`
   - Configure CPU affinity (select some cores)
   - Set priority (e.g., ABOVE_NORMAL)
   - Click "Start Monitoring"

3. **Expected Results**:
   - Logs should show "notepad.exe X instances found (PIDs: ...)"
   - "Tracked Process Instances" section should appear
   - Each notepad instance should be listed with its PID
   - All instances should show the applied affinity and priority settings

### Scenario 2: Process Instance Management

1. **Prerequisites**: Complete Scenario 1

2. **Test Steps**:
   - Click "📊 Details" on any process instance
   - Click "❌ Exclude" on one process instance
   - Observe the instance counter changes
   - Click "🔄 Show All" to restore excluded processes

3. **Expected Results**:
   - Details dialog shows process information
   - Excluded process disappears from the list
   - Counter shows "(X excluded)"
   - "Show All" button restores all processes

### Scenario 3: Dynamic Process Changes

1. **Test Steps**:
   - Start monitoring with multiple notepad instances
   - Close one notepad window manually
   - Open a new notepad instance
   - Observe the tracking updates

2. **Expected Results**:
   - Closed process should disappear from tracked instances
   - New process should be automatically detected and added
   - Logs should reflect the changes

### Scenario 4: Settings Application

1. **Test Steps**:
   - Start monitoring multiple instances
   - Use Task Manager to verify CPU affinity settings
   - Check process priorities in Task Manager
   - Change settings in the app and observe reapplication

2. **Expected Results**:
   - All instances should have the same CPU affinity
   - All instances should have the same priority
   - Changes should be applied to all instances simultaneously

## Verification Checklist

- [ ] Multiple process instances are detected and displayed
- [ ] Each instance shows correct PID
- [ ] Settings are applied to all instances
- [ ] Process exclusion works correctly
- [ ] Excluded processes can be restored
- [ ] Logs clearly identify multi-process scenarios
- [ ] Dynamic process changes are handled correctly
- [ ] UI updates in real-time during monitoring

## Cleanup

To clean up test processes:

```powershell
# PowerShell
Get-Process notepad | Stop-Process

# Or manually close all notepad windows
```

## Known Limitations

1. **Process Exclusion**: Currently only excludes from UI display, not from actual monitoring
2. **Individual Settings**: All instances use the same settings (future enhancement could allow per-instance settings)
3. **Process Identification**: Based on executable name only, not full path

## Future Enhancements

1. **Per-Instance Settings**: Allow different settings for each process instance
2. **Process Filtering**: Filter by command line arguments or working directory
3. **Process Grouping**: Group related processes together
4. **Advanced Exclusion**: Permanently exclude specific PIDs from monitoring
