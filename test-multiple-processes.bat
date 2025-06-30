@echo off
echo Creating multiple notepad instances for testing...
echo You can use 'notepad.exe' as the target process in the Process Priority Manager
echo.

REM Create 3 instances of notepad
start notepad.exe
echo Started notepad instance 1
timeout /t 1 /nobreak >nul

start notepad.exe
echo Started notepad instance 2
timeout /t 1 /nobreak >nul

start notepad.exe
echo Started notepad instance 3
timeout /t 1 /nobreak >nul

echo.
echo Created 3 notepad instances. You can now:
echo 1. Set target process to 'notepad.exe' in the app
echo 2. Configure CPU affinity and priority settings
echo 3. Start monitoring to see multi-process tracking in action
echo 4. Check the 'Tracked Process Instances' section to see all PIDs
echo.
echo To clean up, close all notepad windows manually
pause
