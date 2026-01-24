# Download Python Embeddable for Portable Superset
# Run this on a machine WITH internet access

param(
    [string]$PythonVersion = "3.11.7"
)

$ErrorActionPreference = "Stop"
$ProgressPreference = "SilentlyContinue"  # Faster downloads

$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$rootDir = Split-Path -Parent $scriptDir
$pythonDir = Join-Path $rootDir "python"

Write-Host "=== Portable Superset: Python Setup ===" -ForegroundColor Cyan
Write-Host "Python Version: $PythonVersion"
Write-Host "Target Directory: $pythonDir"

# Create directories
if (!(Test-Path $pythonDir)) {
    New-Item -ItemType Directory -Path $pythonDir | Out-Null
}

# Download Python Embeddable
$pythonZip = "python-$PythonVersion-embed-amd64.zip"
$pythonUrl = "https://www.python.org/ftp/python/$PythonVersion/$pythonZip"
$zipPath = Join-Path $env:TEMP $pythonZip

Write-Host "`n[1/5] Downloading Python Embeddable..." -ForegroundColor Yellow
if (!(Test-Path $zipPath)) {
    Invoke-WebRequest -Uri $pythonUrl -OutFile $zipPath
}
Write-Host "      Downloaded: $zipPath" -ForegroundColor Green

# Extract Python
Write-Host "`n[2/5] Extracting Python..." -ForegroundColor Yellow
Expand-Archive -Path $zipPath -DestinationPath $pythonDir -Force
Write-Host "      Extracted to: $pythonDir" -ForegroundColor Green

# Enable site-packages by modifying python311._pth
$pthFile = Get-ChildItem -Path $pythonDir -Filter "python*._pth" | Select-Object -First 1
if ($pthFile) {
    Write-Host "`n[3/5] Configuring Python paths..." -ForegroundColor Yellow
    $pthContent = Get-Content $pthFile.FullName
    # Uncomment import site
    $pthContent = $pthContent -replace "^#import site", "import site"
    # Add Lib\site-packages
    $pthContent += "`nLib\site-packages"
    Set-Content -Path $pthFile.FullName -Value $pthContent
    Write-Host "      Updated: $($pthFile.Name)" -ForegroundColor Green
}

# Create Lib and Scripts directories
$libDir = Join-Path $pythonDir "Lib\site-packages"
$scriptsDir = Join-Path $pythonDir "Scripts"
New-Item -ItemType Directory -Path $libDir -Force | Out-Null
New-Item -ItemType Directory -Path $scriptsDir -Force | Out-Null

# Download and install pip
Write-Host "`n[4/5] Installing pip..." -ForegroundColor Yellow
$getPipUrl = "https://bootstrap.pypa.io/get-pip.py"
$getPipPath = Join-Path $env:TEMP "get-pip.py"
Invoke-WebRequest -Uri $getPipUrl -OutFile $getPipPath

$pythonExe = Join-Path $pythonDir "python.exe"
& $pythonExe $getPipPath --no-warn-script-location
Write-Host "      pip installed successfully" -ForegroundColor Green

# Verify
Write-Host "`n[5/5] Verifying installation..." -ForegroundColor Yellow
& $pythonExe -m pip --version
Write-Host "      Python ready!" -ForegroundColor Green

Write-Host "`n=== Python Setup Complete ===" -ForegroundColor Cyan
Write-Host "Next: Run install_superset.bat to install Apache Superset" -ForegroundColor White
