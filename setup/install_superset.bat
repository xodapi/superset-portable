@echo off
REM Install Apache Superset into Portable Python
REM Run this on a machine WITH internet access

setlocal EnableDelayedExpansion

echo === Portable Superset: Installing Superset ===
echo.

set "SCRIPT_DIR=%~dp0"
set "ROOT_DIR=%SCRIPT_DIR%.."
set "PYTHON_DIR=%ROOT_DIR%\python"
set "PYTHON_EXE=%PYTHON_DIR%\python.exe"
set "SUPERSET_HOME=%ROOT_DIR%\superset_home"

REM Check Python
if not exist "%PYTHON_EXE%" (
    echo ERROR: Python not found at %PYTHON_EXE%
    echo Please run download_python.ps1 first
    pause
    exit /b 1
)

REM Set environment for portable Python
set "PYTHONHOME=%PYTHON_DIR%"
set "PATH=%PYTHON_DIR%;%PYTHON_DIR%\Scripts;%PATH%"

echo [1/5] Upgrading pip...
"%PYTHON_EXE%" -m pip install --upgrade pip setuptools wheel

echo.
echo [2/5] Installing Apache Superset and dependencies...
echo      This may take 5-10 minutes...
"%PYTHON_EXE%" -m pip install apache-superset

echo.
echo [3/5] Creating Superset home directory...
if not exist "%SUPERSET_HOME%" mkdir "%SUPERSET_HOME%"

REM Create superset_config.py
echo [4/5] Creating configuration...
set "CONFIG_FILE=%SUPERSET_HOME%\superset_config.py"
if not exist "%CONFIG_FILE%" (
    echo import os > "%CONFIG_FILE%"
    echo SECRET_KEY = 'portable-superset-secret-key-change-me' >> "%CONFIG_FILE%"
    echo SQLALCHEMY_DATABASE_URI = 'sqlite:///' + os.path.join^(os.path.dirname^(__file__^), 'superset.db'^) >> "%CONFIG_FILE%"
    echo WTF_CSRF_ENABLED = False >> "%CONFIG_FILE%"
    echo FEATURE_FLAGS = {'ALERT_REPORTS': False} >> "%CONFIG_FILE%"
    echo CACHE_CONFIG = {'CACHE_TYPE': 'SimpleCache', 'CACHE_DEFAULT_TIMEOUT': 300} >> "%CONFIG_FILE%"
)

REM Set Superset config path
set "SUPERSET_CONFIG_PATH=%CONFIG_FILE%"

echo.
echo [5/5] Initializing Superset database...
echo      Running migrations...
"%PYTHON_EXE%" -m superset db upgrade

echo      Creating admin user (admin/admin)...
"%PYTHON_EXE%" -m superset fab create-admin --username admin --password admin --firstname Admin --lastname User --email admin@localhost

echo      Loading examples (optional, may take a while)...
REM Uncomment to load example dashboards:
REM "%PYTHON_EXE%" -m superset load_examples

echo      Final initialization...
"%PYTHON_EXE%" -m superset init

echo.
echo [6/6] Setting up RZD Demo Dashboard...
echo      Creating tables and loading data...
"%PYTHON_EXE%" "%SCRIPT_DIR%setup_rzd_dashboard.py"

echo      Configuring database metadata...
"%PYTHON_EXE%" "%SCRIPT_DIR%setup_db_connection.py"

echo      Importing dashboards and charts...
call "%SCRIPT_DIR%import_dashboard.bat"


echo.
echo === Installation Complete ===
echo.
echo To start Superset, run:
echo   superset-launcher.exe start
echo.
echo Or use the batch file:
echo   start_superset.bat
echo.
echo Default login: admin / admin
echo.
pause
