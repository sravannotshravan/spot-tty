# spot-tty Windows installer
# Run with: irm https://raw.githubusercontent.com/Gaurav-Gali/spot-tty/main/install.ps1 | iex

$ErrorActionPreference = "Stop"

function Write-Header { Write-Host "`n$args" -ForegroundColor White }
function Write-Info    { Write-Host "  -> $args" -ForegroundColor Cyan }
function Write-Success { Write-Host "  v $args" -ForegroundColor Green }
function Write-Warn    { Write-Host "  ! $args" -ForegroundColor Yellow }
function Write-Fail    { Write-Host "  x $args" -ForegroundColor Red; exit 1 }

Write-Host @"

  ███████╗██████╗  ██████╗ ████████╗    ████████╗████████╗██╗   ██╗
  ██╔════╝██╔══██╗██╔═══██╗╚══██╔══╝       ██╔══╝╚══██╔══╝╚██╗ ██╔╝
  ███████╗██████╔╝██║   ██║   ██║   █████╗ ██║      ██║    ╚████╔╝ 
  ╚════██║██╔═══╝ ██║   ██║   ██║   ╚════╝ ██║      ██║     ╚██╔╝  
  ███████║██║     ╚██████╔╝   ██║          ██║      ██║      ██║   
  ╚══════╝╚═╝      ╚═════╝    ╚═╝          ╚═╝      ╚═╝      ╚═╝   

  Spotify TUI for your terminal — Windows installer
"@ -ForegroundColor Cyan

# ── Check Rust ────────────────────────────────────────────────────────────────
Write-Header "Checking Rust toolchain..."
if (Get-Command cargo -ErrorAction SilentlyContinue) {
    $v = (rustc --version)
    Write-Success "Rust already installed: $v"
} else {
    Write-Warn "Rust not found — downloading rustup..."
    $rustup = "$env:TEMP\rustup-init.exe"
    Invoke-WebRequest "https://win.rustup.rs/x86_64" -OutFile $rustup
    & $rustup -y --quiet
    $env:PATH += ";$env:USERPROFILE\.cargo\bin"
    Write-Success "Rust installed"
}

# ── Clone & build ─────────────────────────────────────────────────────────────
Write-Header "Fetching spot-tty..."
$tmp = Join-Path $env:TEMP "spot-tty-build"
if (Test-Path $tmp) { Remove-Item $tmp -Recurse -Force }

if (-not (Get-Command git -ErrorAction SilentlyContinue)) {
    Write-Fail "git is required. Install from https://git-scm.com and re-run."
}
git clone --depth=1 "https://github.com/Gaurav-Gali/spot-tty" $tmp
if ($LASTEXITCODE -ne 0) { Write-Fail "Failed to clone repository" }
Write-Success "Cloned repository"

Write-Header "Building spot-tty (this takes ~1 min on first run)..."
Push-Location $tmp
$env:RUSTFLAGS = "-A warnings"; cargo build --release
if ($LASTEXITCODE -ne 0) { Write-Fail "Build failed" }
Pop-Location
Write-Success "Build complete"

# ── Install binary ────────────────────────────────────────────────────────────
Write-Header "Installing binary..."
$installDir = "$env:USERPROFILE\.local\bin"
New-Item -ItemType Directory -Force -Path $installDir | Out-Null
Copy-Item "$tmp\target\release\spot-tty.exe" "$installDir\spot-tty.exe" -Force
Write-Success "Binary installed to $installDir\spot-tty.exe"

# Add to PATH if not already there
$userPath = [Environment]::GetEnvironmentVariable("PATH", "User")
if ($userPath -notlike "*$installDir*") {
    [Environment]::SetEnvironmentVariable("PATH", "$userPath;$installDir", "User")
    Write-Warn "Added $installDir to your PATH — restart your terminal after install"
}

# ── Spotify credentials ───────────────────────────────────────────────────────
Write-Header "Spotify API credentials..."
$configDir = "$env:APPDATA\spot-tty"
New-Item -ItemType Directory -Force -Path $configDir | Out-Null
$envFile = "$configDir\.env"

if ((Test-Path $envFile) -and (Select-String "RSPOTIFY_CLIENT_ID=" $envFile -Quiet)) {
    Write-Success "Credentials already set at $envFile — skipping"
} else {
    Write-Host ""
    Write-Host "  You need a Spotify Developer app. Steps:" -ForegroundColor White
    Write-Host "    1. Go to https://developer.spotify.com/dashboard"
    Write-Host "    2. Create an app (any name)"
    Write-Host "    3. Settings -> Redirect URIs -> add: http://127.0.0.1:8888/callback"
    Write-Host "    4. Copy your Client ID and Client Secret"
    Write-Host ""

    $clientId     = Read-Host "  Client ID"
    $clientSecret = Read-Host "  Client Secret"

    if (-not $clientId -or -not $clientSecret) {
        Write-Fail "Client ID and Secret cannot be empty"
    }

    @"
RSPOTIFY_CLIENT_ID=$clientId
RSPOTIFY_CLIENT_SECRET=$clientSecret
RSPOTIFY_REDIRECT_URI=http://127.0.0.1:8888/callback
"@ | Set-Content $envFile -Encoding UTF8
    Write-Success "Credentials saved to $envFile"
}

# Cleanup
Remove-Item $tmp -Recurse -Force

# ── Done ──────────────────────────────────────────────────────────────────────
Write-Host ""
Write-Host "  ✓ spot-tty installed!" -ForegroundColor Green
Write-Host ""
Write-Host "  Run: spot-tty"
Write-Host "  On first launch, a browser window opens for Spotify login."
Write-Host ""
Write-Warn "If 'spot-tty' is not found, restart your terminal and try again."
Write-Host ""
