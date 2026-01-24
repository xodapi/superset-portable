@echo off
REM Start documentation server on port 8089
setlocal EnableDelayedExpansion

set "ROOT_DIR=%~dp0"
set "PYTHON_DIR=%ROOT_DIR%python"
set "DOCS_DIR=%ROOT_DIR%docs"

echo Starting documentation server on http://localhost:8089
"%PYTHON_DIR%\python.exe" -m http.server 8089 --directory "%DOCS_DIR%"
