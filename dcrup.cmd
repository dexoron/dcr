@echo off
setlocal
set "SCRIPT_DIR=%~dp0"
if exist "%SCRIPT_DIR%dcrup.ps1" (
  powershell -NoProfile -ExecutionPolicy Bypass -File "%SCRIPT_DIR%dcrup.ps1" %*
  exit /b %ERRORLEVEL%
)
if exist "%USERPROFILE%\.dcr\bin\dcrup.ps1" (
  powershell -NoProfile -ExecutionPolicy Bypass -File "%USERPROFILE%\.dcr\bin\dcrup.ps1" %*
  exit /b %ERRORLEVEL%
)
echo [error] dcrup.ps1 not found near dcrup.cmd >&2
exit /b 1
