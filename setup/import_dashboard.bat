@echo off
set "SCRIPT_DIR=%~dp0"
set "ROOT_DIR=%SCRIPT_DIR%.."
set "PYTHON_EXE=%ROOT_DIR%\python\python.exe"
set "DASHBOARD_DIR=%ROOT_DIR%\docs\demo_dashboard"

echo Importing RZD Dashboard from %DASHBOARD_DIR%...

if not exist "%PYTHON_EXE%" (
    echo [ERROR] Python not found. Please run download_python.ps1
    exit /b 1
)

"%PYTHON_EXE%" -m superset.cli.main import-dashboards -p "%DASHBOARD_DIR%" -u admin

if %ERRORLEVEL% EQU 0 (
    echo [OK] Dashboard imported successfully!
) else (
    echo [ERROR] Failed to import dashboard.
)
