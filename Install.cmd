@echo off
setlocal
powershell.exe -NoProfile -ExecutionPolicy RemoteSigned -File "%~dp0Install.ps1" %*
set "RC=%ERRORLEVEL%"
echo.
if not "%RC%"=="0" echo Installation did not complete. Read the error above.
if "%RC%"=="0" echo Installation complete. ReplyNoti is available in Start.
pause
exit /b %RC%
