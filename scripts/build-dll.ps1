# Safe DLL build — handles Windows Defender / antivirus lock
# Pattern: rename locked file -> build new -> cleanup old locked files

param(
    [string]$Config = "release"
)

$ErrorActionPreference = "Stop"
$dllName = "context_menu_dll.dll"
$targetDir = "target\$Config"
$dllPath = "$targetDir\$dllName"

Write-Host "=== Safe DLL Build ($Config) ===" -ForegroundColor Cyan

# Ensure target directory exists
if (!(Test-Path $targetDir)) {
    New-Item -ItemType Directory -Path $targetDir -Force | Out-Null
}

# Check if DLL exists and try to detect lock
if (Test-Path $dllPath) {
    $locked = $false
    try {
        $stream = [System.IO.File]::Open($dllPath, 'Open', 'ReadWrite', 'None')
        $stream.Close()
        $stream.Dispose()
    } catch {
        $locked = $true
    }

    if ($locked) {
        $timestamp = Get-Date -Format "yyyyMMdd_HHmmss"
        $renameTo = "$dllPath.$timestamp.locked.delete"
        Write-Host "DLL is LOCKED — renaming to: $renameTo" -ForegroundColor Yellow
        Rename-Item -Path $dllPath -NewName "$dllName.$timestamp.locked.delete" -Force
    } else {
        Write-Host "DLL exists and is accessible — removing before rebuild" -ForegroundColor DarkGray
        Remove-Item -Path $dllPath -Force
    }
}

# Cleanup any leftover .locked.delete files
$lockedFiles = Get-ChildItem -Path $targetDir -Filter "*$dllName*.locked.delete" -ErrorAction SilentlyContinue
if ($lockedFiles) {
    foreach ($f in $lockedFiles) {
        try {
            Remove-Item -Path $f.FullName -Force -ErrorAction Stop
            Write-Host "Cleaned up: $($f.Name)" -ForegroundColor DarkGray
        } catch {
            Write-Host "Cannot remove (still locked): $($f.Name) — will be cleaned later" -ForegroundColor DarkYellow
        }
    }
}

# Create placeholder so src-tauri build.rs passes the resource check
New-Item -ItemType File -Path $dllPath -Force | Out-Null
Write-Host "Created placeholder DLL" -ForegroundColor DarkGray

# Build the real DLL (overwrites placeholder)
Write-Host "Building context-menu-dll ($Config)..." -ForegroundColor Green
cargo build -p context-menu-dll --$Config
if ($LASTEXITCODE -ne 0) {
    Write-Error "DLL build failed with exit code $LASTEXITCODE"
    exit $LASTEXITCODE
}

# Verify
if (Test-Path $dllPath) {
    $size = (Get-Item $dllPath).Length / 1KB
    Write-Host "DLL built: $([math]::Round($size)) KB" -ForegroundColor Green
} else {
    Write-Error "DLL not found after build: $dllPath"
    exit 1
}
