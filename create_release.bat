@echo off
REM Create release package for offline deployment
REM Создание релиза для закрытого контура

setlocal EnableDelayedExpansion

set "ROOT_DIR=%~dp0"
set "RELEASE_DIR=%ROOT_DIR%release"
set "RELEASE_NAME=superset-portable-v6.0-ru"

echo ========================================
echo   Creating Superset Portable Release
echo ========================================
echo.

REM Create release directory
if exist "%RELEASE_DIR%" rmdir /s /q "%RELEASE_DIR%"
mkdir "%RELEASE_DIR%\%RELEASE_NAME%"

echo [1/6] Copying Python environment...
xcopy /E /I /Q "%ROOT_DIR%python" "%RELEASE_DIR%\%RELEASE_NAME%\python" >nul

echo [2/6] Copying Superset home...
xcopy /E /I /Q "%ROOT_DIR%superset_home" "%RELEASE_DIR%\%RELEASE_NAME%\superset_home" >nul

echo [3/6] Copying documentation...
xcopy /E /I /Q "%ROOT_DIR%docs" "%RELEASE_DIR%\%RELEASE_NAME%\docs" >nul

echo [4/6] Copying launcher and scripts...
xcopy /E /I /Q "%ROOT_DIR%setup" "%RELEASE_DIR%\%RELEASE_NAME%\setup" >nul
copy "%ROOT_DIR%start_superset.bat" "%RELEASE_DIR%\%RELEASE_NAME%\" >nul
copy "%ROOT_DIR%start_docs.bat" "%RELEASE_DIR%\%RELEASE_NAME%\" >nul
if exist "%ROOT_DIR%superset-launcher.exe" copy "%ROOT_DIR%superset-launcher.exe" "%RELEASE_DIR%\%RELEASE_NAME%\" >nul

echo [5/6] Copying license and docs...
copy "%ROOT_DIR%LICENSE" "%RELEASE_DIR%\%RELEASE_NAME%\" >nul
copy "%ROOT_DIR%NOTICE" "%RELEASE_DIR%\%RELEASE_NAME%\" >nul
copy "%ROOT_DIR%QUICKSTART.md" "%RELEASE_DIR%\%RELEASE_NAME%\README.txt" >nul

echo [6/6] Creating archive...
cd "%RELEASE_DIR%"
powershell -Command "Compress-Archive -Path '%RELEASE_NAME%' -DestinationPath '%RELEASE_NAME%.zip' -Force"

echo.
echo ========================================
echo   Release created successfully!
echo ========================================
echo.
echo Location: %RELEASE_DIR%\%RELEASE_NAME%.zip
echo.

pause
