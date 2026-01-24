@echo off
REM Quick start script for Portable Superset

setlocal EnableDelayedExpansion

set "ROOT_DIR=%~dp0"
set "PYTHON_DIR=%ROOT_DIR%python"
set "PYTHON_EXE=%PYTHON_DIR%\python.exe"
set "SUPERSET_HOME=%ROOT_DIR%superset_home"
set "DOCS_DIR=%ROOT_DIR%docs"

REM Set environment
set "PYTHONHOME=%PYTHON_DIR%"
set "PATH=%PYTHON_DIR%;%PYTHON_DIR%\Scripts;%PATH%"
set "SUPERSET_CONFIG_PATH=%SUPERSET_HOME%\superset_config.py"
set "FLASK_APP=superset"

echo ========================================
echo   Portable Apache Superset
echo ========================================
echo.

REM Check if installed
if not exist "%PYTHON_EXE%" (
    echo ERROR: Python not found!
    echo Please run setup\download_python.ps1 first.
    pause
    exit /b 1
)

if not exist "%SUPERSET_HOME%\superset.db" (
    echo WARNING: Superset not initialized!
    echo Please run setup\install_superset.bat first.
    pause
    exit /b 1
)

echo Starting documentation server on http://localhost:8089
echo Starting Superset on http://localhost:8088
echo Press Ctrl+C to stop
echo.

REM Start documentation server in background
start /b "" "%PYTHON_EXE%" -m http.server 8089 --directory "%DOCS_DIR%"

REM Start browser after 5 seconds
start /b cmd /c "timeout /t 5 >nul && start http://localhost:8088"

REM Start Superset
"%PYTHON_DIR%\Scripts\superset.exe" run -h 127.0.0.1 -p 8088 --with-threads

