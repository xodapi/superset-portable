@echo off
rem Start Superset Launcher in LightDocs mode
cd /d "%~dp0"
start "" superset-launcher.exe lightdocs serve
