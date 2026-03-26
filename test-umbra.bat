@echo off
setlocal

cd /d "%~dp0"

set "MODE=%~1"
if "%MODE%"=="" set "MODE=app"

if /i "%MODE%"=="help" goto :help

call :check_command npm "npm / node.js"
if errorlevel 1 goto :pause_fail

if not exist "node_modules\" (
  echo [umbra] node_modules fehlt.
  echo [umbra] bitte zuerst einmal npm install ausfuehren.
  goto :pause_fail
)

set "RUST_BACKTRACE=1"

if /i "%MODE%"=="app" (
  call :check_command cargo "cargo / rust toolchain"
  if errorlevel 1 goto :pause_fail

  echo [umbra] starte tauri dev...
  call npm run tauri dev
  goto :finish
)

if /i "%MODE%"=="web" (
  echo [umbra] starte vite dev server auf http://127.0.0.1:4173 ...
  call npm run dev -- --host 127.0.0.1 --port 4173
  goto :finish
)

if /i "%MODE%"=="tests" (
  echo [umbra] starte vitest...
  call npm test
  goto :finish
)

echo [umbra] unbekannter modus: %MODE%
goto :help_fail

:check_command
where %~1 >nul 2>nul
if errorlevel 1 (
  echo [umbra] %~2 wurde nicht gefunden.
  exit /b 1
)
exit /b 0

:help
echo.
echo UMBRA test launcher
echo.
echo 1. test-umbra.bat
echo    startet die tauri-app im dev-modus.
echo.
echo 2. test-umbra.bat web
echo    startet nur den vite-webserver auf port 4173.
echo.
echo 3. test-umbra.bat tests
echo    fuehrt die frontend-tests aus.
echo.
echo 4. test-umbra.bat help
echo    zeigt diese hilfe.
echo.
exit /b 0

:help_fail
call :help

:pause_fail
echo.
pause
endlocal
exit /b 1

:finish
if errorlevel 1 goto :pause_fail
endlocal
exit /b 0
