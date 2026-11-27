@echo off
setlocal
powershell.exe -NoProfile -ExecutionPolicy RemoteSigned -File "%~dp0Install.ps1" -Portable
if errorlevel 1 (
 echo Portable registration failed. Review the message above.
 pause
 exit /b 1
)
exit /b 0
