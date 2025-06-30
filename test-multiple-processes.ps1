# Test script to create multiple instances of a process for testing multi-process tracking
# This script creates multiple notepad instances to test the multi-process functionality

Write-Host "Creating multiple notepad instances for testing..."
Write-Host "You can use 'notepad.exe' as the target process in the Process Priority Manager"
Write-Host ""

# Create 3 instances of notepad
for ($i = 1; $i -le 3; $i++) {
    Start-Process notepad.exe
    Write-Host "Started notepad instance $i"
    Start-Sleep -Seconds 1
}

Write-Host ""
Write-Host "Created 3 notepad instances. You can now:"
Write-Host "1. Set target process to 'notepad.exe' in the app"
Write-Host "2. Configure CPU affinity and priority settings"
Write-Host "3. Start monitoring to see multi-process tracking in action"
Write-Host "4. Check the 'Tracked Process Instances' section to see all PIDs"
Write-Host ""
Write-Host "To clean up, close all notepad windows or run: Get-Process notepad | Stop-Process"
