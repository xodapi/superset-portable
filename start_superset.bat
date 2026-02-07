@echo off
REM ========================================
REM   Portable Apache Superset - Launcher
REM   Works from USB / any folder / no admin
REM   Uses Waitress WSGI (stable, production)
REM ========================================

chcp 65001 >nul 2>&1
setlocal EnableDelayedExpansion

REM Always work from the directory where this bat file lives
cd /d "%~dp0"

set "ROOT_DIR=%~dp0"
set "PYTHON_DIR=%ROOT_DIR%python"
set "PYTHON_EXE=%PYTHON_DIR%\python.exe"
set "SUPERSET_HOME=%ROOT_DIR%superset_home"
set "DOCS_DIR=%ROOT_DIR%docs"

REM Set environment
set "PYTHONHOME=%PYTHON_DIR%"
set "PYTHONIOENCODING=utf-8"
set "PATH=%PYTHON_DIR%;%PYTHON_DIR%\Scripts;%PATH%"
set "SUPERSET_CONFIG_PATH=%SUPERSET_HOME%\superset_config.py"
set "FLASK_APP=superset"

echo ========================================
echo   Portable Apache Superset
echo   http://localhost:8088
echo ========================================
echo.

REM Check if installed
if not exist "%PYTHON_EXE%" (
    echo [ERROR] Python not found: %PYTHON_EXE%
    echo.
    echo   This is a portable release - python\ folder is required.
    echo   Make sure the archive was fully extracted.
    pause
    exit /b 1
)

if not exist "%SUPERSET_HOME%\superset.db" (
    echo [ERROR] Superset database not found!
    echo   File missing: %SUPERSET_HOME%\superset.db
    pause
    exit /b 1
)

if not exist "%ROOT_DIR%examples.db" (
    echo [WARN] examples.db not found - demo dashboard may not work.
    echo   Run: python\python.exe setup\create_rzd_dashboard.py
    echo.
)

REM Fix database URI to match current installation path
if exist "%ROOT_DIR%setup\fix_db_uri.py" (
    "%PYTHON_EXE%" "%ROOT_DIR%setup\fix_db_uri.py"
    echo.
)

echo Starting documentation server on http://localhost:8089
echo Starting Superset on http://localhost:8088
echo.
echo   Dashboard: http://localhost:8088/superset/dashboard/rzd_analytics/
echo   Login: admin / Password: admin
echo.
echo Press Ctrl+C to stop
echo.

REM Start documentation server in background
start /b "" "%PYTHON_EXE%" -m http.server 8089 --directory "%DOCS_DIR%" >nul 2>&1

REM Open browser after 40 seconds (Superset needs time to load on slow PCs)
start /b cmd /c "timeout /t 40 >nul && start http://localhost:8088/superset/dashboard/rzd_analytics/"

REM Start Superset via Waitress (production WSGI server - stable, multi-threaded)
REM Waitress handles concurrent requests properly unlike Flask dev server
"%PYTHON_EXE%" -c "from waitress import serve; from superset.app import create_app; app = create_app(); print('Superset is READY at http://127.0.0.1:8088', flush=True); serve(app, host='127.0.0.1', port=8088, threads=4, channel_timeout=120, recv_bytes=65536)"
