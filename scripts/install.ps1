# Installation script for Inferno AI/ML model runner (Windows PowerShell)
param(
    [string]$InstallDir = "$env:LOCALAPPDATA\Programs\Inferno",
    [string]$Version = "latest",
    [switch]$Force,
    [switch]$Help
)

# Configuration
$RepoUrl = "https://github.com/inferno-ai/inferno"
$BinaryName = "inferno.exe"

function Write-ColorOutput {
    param(
        [string]$Message,
        [string]$Color = "White"
    )
    Write-Host $Message -ForegroundColor $Color
}

function Write-Info {
    param([string]$Message)
    Write-ColorOutput "[INFO] $Message" "Cyan"
}

function Write-Warning {
    param([string]$Message)
    Write-ColorOutput "[WARN] $Message" "Yellow"
}

function Write-Error {
    param([string]$Message)
    Write-ColorOutput "[ERROR] $Message" "Red"
    exit 1
}

function Write-Success {
    param([string]$Message)
    Write-ColorOutput "[SUCCESS] $Message" "Green"
}

function Show-Usage {
    Write-Host "Inferno AI/ML Model Runner Installation Script for Windows"
    Write-Host ""
    Write-Host "Usage: .\install.ps1 [OPTIONS]"
    Write-Host ""
    Write-Host "Parameters:"
    Write-Host "  -InstallDir DIR     Installation directory (default: %LOCALAPPDATA%\Programs\Inferno)"
    Write-Host "  -Version VERSION    Version to install (default: latest)"
    Write-Host "  -Force              Force installation (overwrite existing)"
    Write-Host "  -Help               Show this help"
    Write-Host ""
    Write-Host "Examples:"
    Write-Host "  .\install.ps1"
    Write-Host "  .\install.ps1 -InstallDir 'C:\Tools\Inferno'"
    Write-Host "  .\install.ps1 -Version 'v0.1.0'"
}

function Test-Prerequisites {
    Write-Info "Checking prerequisites..."

    # Check PowerShell version
    if ($PSVersionTable.PSVersion.Major -lt 5) {
        Write-Error "PowerShell 5.0 or higher is required"
    }

    # Check for required .NET classes
    try {
        [System.Net.ServicePointManager]::SecurityProtocol = [System.Net.SecurityProtocolType]::Tls12
    } catch {
        Write-Error "Unable to set TLS 1.2 security protocol"
    }

    # Test internet connectivity
    try {
        $null = Invoke-WebRequest -Uri "https://api.github.com" -UseBasicParsing -TimeoutSec 10
    } catch {
        Write-Error "Unable to connect to the internet"
    }
}

function Get-DownloadUrl {
    param(
        [string]$Version
    )

    if ($Version -eq "latest") {
        Write-Info "Fetching latest release information..."
        $apiUrl = "$RepoUrl/releases/latest".Replace("github.com", "api.github.com/repos")

        try {
            $response = Invoke-RestMethod -Uri $apiUrl -TimeoutSec 30
            $Version = $response.tag_name
            Write-Info "Latest version: $Version"
        } catch {
            Write-Error "Failed to fetch latest version: $_"
        }
    }

    $filename = "inferno-windows-x86_64.exe"
    return "$RepoUrl/releases/download/$Version/$filename"
}

function Install-Inferno {
    Write-Info "Starting Inferno installation..."

    # Create installation directory
    if (-not (Test-Path $InstallDir)) {
        Write-Info "Creating installation directory: $InstallDir"
        try {
            New-Item -ItemType Directory -Path $InstallDir -Force | Out-Null
        } catch {
            Write-Error "Failed to create installation directory: $_"
        }
    }

    $binaryPath = Join-Path $InstallDir $BinaryName

    # Check if already installed
    if ((Test-Path $binaryPath) -and -not $Force) {
        Write-Warning "Inferno is already installed at $binaryPath"
        Write-Warning "Use -Force to overwrite, or uninstall first"

        try {
            $currentVersion = & $binaryPath --version 2>$null
            if ($LASTEXITCODE -eq 0) {
                Write-Host "Current version: $currentVersion"
            }
        } catch {
            # Ignore errors when checking version
        }

        exit 1
    }

    # Get download URL
    $downloadUrl = Get-DownloadUrl -Version $Version
    Write-Info "Download URL: $downloadUrl"

    # Download binary
    Write-Info "Downloading Inferno..."
    $tempFile = [System.IO.Path]::GetTempFileName()

    try {
        Invoke-WebRequest -Uri $downloadUrl -OutFile $tempFile -TimeoutSec 300
    } catch {
        Write-Error "Failed to download Inferno: $_"
    }

    # Install binary
    Write-Info "Installing to $binaryPath..."
    try {
        Move-Item $tempFile $binaryPath -Force
    } catch {
        Write-Error "Failed to install binary: $_"
    }

    Write-Success "Inferno installed successfully!"
}

function Test-Installation {
    $binaryPath = Join-Path $InstallDir $BinaryName

    Write-Info "Verifying installation..."

    if (-not (Test-Path $binaryPath)) {
        Write-Error "Binary not found at $binaryPath"
    }

    try {
        $versionOutput = & $binaryPath --version 2>$null
        if ($LASTEXITCODE -ne 0) {
            Write-Error "Binary is not executable or corrupted"
        }
        Write-Success "Installation verified: $versionOutput"
    } catch {
        Write-Error "Binary test failed: $_"
    }
}

function Update-Environment {
    Write-Info "Updating environment..."

    # Add to PATH if not already there
    $currentPath = [Environment]::GetEnvironmentVariable("PATH", "User")
    if ($currentPath -notlike "*$InstallDir*") {
        Write-Info "Adding $InstallDir to user PATH..."

        try {
            [Environment]::SetEnvironmentVariable(
                "PATH",
                "$InstallDir;$currentPath",
                "User"
            )

            # Update current session PATH
            $env:PATH = "$InstallDir;$env:PATH"

            Write-Success "PATH updated successfully"
        } catch {
            Write-Warning "Failed to update PATH automatically: $_"
            Write-Warning "Please add $InstallDir to your PATH manually"
        }
    }
}

function Initialize-Configuration {
    Write-Info "Setting up initial configuration..."

    $configDir = Join-Path $env:APPDATA "inferno"
    $modelsDir = Join-Path $env:LOCALAPPDATA "inferno\models"

    # Create directories
    New-Item -ItemType Directory -Path $configDir -Force | Out-Null
    New-Item -ItemType Directory -Path $modelsDir -Force | Out-Null

    $configFile = Join-Path $configDir "config.toml"

    if (-not (Test-Path $configFile)) {
        $configContent = @"
# Inferno AI/ML Model Runner Configuration

models_dir = "$($modelsDir -replace '\\', '\\')"
cache_dir = "$($env:LOCALAPPDATA -replace '\\', '\\')\\inferno\\cache"
log_level = "info"
log_format = "pretty"

[backend_config]
gpu_enabled = false
context_size = 2048
batch_size = 32
memory_map = true

[server]
bind_address = "127.0.0.1"
port = 8080
max_concurrent_requests = 10
request_timeout_seconds = 300

[security]
verify_checksums = true
allowed_model_extensions = ["gguf", "onnx"]
max_model_size_gb = 50.0
sandbox_enabled = true
"@

        try {
            Set-Content -Path $configFile -Value $configContent -Encoding UTF8
            Write-Success "Configuration created at $configFile"
        } catch {
            Write-Warning "Failed to create configuration file: $_"
        }
    }

    Write-Info "Models directory: $modelsDir"
    Write-Info "Configuration directory: $configDir"
}

function Show-GettingStarted {
    Write-Host ""
    Write-Success "🎉 Inferno is now installed!"
    Write-Host ""
    Write-Host "Getting Started:" -ForegroundColor White
    Write-Host "================" -ForegroundColor White
    Write-Host ""
    Write-Host "1. Restart your PowerShell/Command Prompt to refresh PATH"
    Write-Host ""
    Write-Host "2. Check installation:"
    Write-Host "   inferno --version" -ForegroundColor Yellow
    Write-Host ""
    Write-Host "3. View help:"
    Write-Host "   inferno --help" -ForegroundColor Yellow
    Write-Host ""
    Write-Host "4. List models (initially empty):"
    Write-Host "   inferno models list" -ForegroundColor Yellow
    Write-Host ""
    Write-Host "5. Launch TUI:"
    Write-Host "   inferno tui" -ForegroundColor Yellow
    Write-Host ""
    Write-Host "6. Place model files (.gguf or .onnx) in:" -ForegroundColor Cyan
    Write-Host "   $env:LOCALAPPDATA\inferno\models\"
    Write-Host ""
    Write-Host "7. Run inference:"
    Write-Host "   inferno run --model MODEL_NAME --prompt `"Hello, world!`"" -ForegroundColor Yellow
    Write-Host ""
    Write-Host "For more information, visit: $RepoUrl" -ForegroundColor Blue
    Write-Host ""
}

# Main execution
function Main {
    if ($Help) {
        Show-Usage
        exit 0
    }

    Write-Info "Installing Inferno AI/ML Model Runner for Windows"

    Test-Prerequisites
    Install-Inferno
    Test-Installation
    Update-Environment
    Initialize-Configuration
    Show-GettingStarted
}

# Error handling
trap {
    Write-Error "Installation failed: $_"
}

# Run main function
Main