@echo off
setlocal
powershell.exe -NoProfile -ExecutionPolicy RemoteSigned -File "%~dp0Uninstall.ps1" -Portable
if errorlevel 1 (
 echo Registration removal failed.
 pause
 exit /b 1
)
echo Registration removed. Settings and files remain in this folder.
pause
