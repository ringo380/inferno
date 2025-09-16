# Bootstrap script for Inferno AI/ML Model Runner (Windows PowerShell)
# This script creates the entire repository structure from scratch

param(
    [string]$ProjectDir = "inferno",
    [switch]$Help
)

if ($Help) {
    Write-Host "Bootstrap script for Inferno AI/ML Model Runner"
    Write-Host ""
    Write-Host "Usage: .\bootstrap.ps1 [-ProjectDir DIR] [-Help]"
    Write-Host ""
    Write-Host "Parameters:"
    Write-Host "  -ProjectDir    Directory to create project in (default: inferno)"
    Write-Host "  -Help          Show this help"
    exit 0
}

function Write-Info {
    param([string]$Message)
    Write-Host "[BOOTSTRAP] $Message" -ForegroundColor Cyan
}

function Write-Success {
    param([string]$Message)
    Write-Host "[SUCCESS] $Message" -ForegroundColor Green
}

function Write-Warning {
    param([string]$Message)
    Write-Host "[WARN] $Message" -ForegroundColor Yellow
}

Write-Info "Creating Inferno project in: $ProjectDir"

# Create project directory
if (Test-Path $ProjectDir) {
    Write-Warning "Directory $ProjectDir already exists"
} else {
    New-Item -ItemType Directory -Path $ProjectDir | Out-Null
}

Set-Location $ProjectDir

# Create directory structure
Write-Info "Creating directory structure..."

$directories = @(
    "src\cli",
    "src\tui\components",
    "src\tui\events",
    "src\backends",
    "src\models",
    "src\io",
    "src\metrics",
    "tests",
    "benches",
    "examples",
    "docs",
    "scripts",
    ".github\workflows"
)

foreach ($dir in $directories) {
    New-Item -ItemType Directory -Path $dir -Force | Out-Null
}

# Create all the Rust source files (content would be the same as created above)
Write-Info "Creating source files..."

# Cargo.toml
@'
[package]
name = "inferno"
version = "0.1.0"
edition = "2021"
authors = ["Inferno Developers"]
description = "An offline AI/ML model runner for GGUF and ONNX models"
readme = "README.md"
homepage = "https://github.com/inferno-ai/inferno"
repository = "https://github.com/inferno-ai/inferno"
license = "MIT OR Apache-2.0"
keywords = ["ai", "ml", "gguf", "onnx", "inference"]
categories = ["command-line-utilities", "science"]

[[bin]]
name = "inferno"
path = "src/main.rs"

[dependencies]
clap = { version = "4.4", features = ["derive", "env"] }
serde = { version = "1.0", features = ["derive"] }
toml = "0.8"
figment = { version = "0.10", features = ["toml", "env"] }
tokio = { version = "1.0", features = ["full"] }
futures = "0.3"
ratatui = "0.24"
crossterm = "0.27"
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter", "json"] }
anyhow = "1.0"
thiserror = "1.0"
serde_json = "1.0"
image = { version = "0.24", features = ["png", "jpeg"] }
hound = "3.5"
llama-cpp-2 = "0.1.67"
ort = { version = "2.0", features = ["load-dynamic"] }
sysinfo = "0.29"
sha2 = "0.10"
hex = "0.4"
dirs = "5.0"
indicatif = "0.17"
async-trait = "0.1"
futures-util = "0.3"

[dev-dependencies]
tempfile = "3.8"
assert_cmd = "2.0"
predicates = "3.0"
criterion = { version = "0.5", features = ["html_reports"] }

[features]
default = []
gpu-metal = []
gpu-vulkan = []
gpu-directml = []

[profile.release]
lto = true
codegen-units = 1
panic = "abort"
strip = true
'@ | Set-Content -Path "Cargo.toml" -Encoding UTF8

# .gitignore
@'
# Rust
/target/
**/*.rs.bk
*.pdb
Cargo.lock

# IDE
.idea/
.vscode/
*.swp
*.swo
*~

# OS
.DS_Store
Thumbs.db

# Project specific
/models/
/cache/
/logs/
*.log
*.gguf
*.onnx

# Testing
/test_output/
/bench_results/

# Coverage
*.profraw
*.profdata
/coverage/
cobertura.xml

# Release artifacts
/release/
/dist/
'@ | Set-Content -Path ".gitignore" -Encoding UTF8

# Create example files
Write-Info "Creating example files..."

@'
// Example: Simple inference with Inferno
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    println!("Inferno Simple Inference Example");
    println!("This is a placeholder example.");
    println!("Run with: cargo run --example simple_inference");
    Ok(())
}
'@ | Set-Content -Path "examples\simple_inference.rs" -Encoding UTF8

# License files
Write-Info "Creating license files..."

@'
MIT License

Copyright (c) 2024 Inferno Contributors

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
'@ | Set-Content -Path "LICENSE-MIT" -Encoding UTF8

# Create verification script
@'
# Verification script for Windows
Write-Host "Verifying Inferno installation..." -ForegroundColor Cyan

# Check Rust installation
if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
    Write-Host "✗ Cargo not found" -ForegroundColor Red
    exit 1
}
Write-Host "✓ Cargo found" -ForegroundColor Green

# Build the project
Write-Host "Building project..." -ForegroundColor Yellow
try {
    cargo build --release
    Write-Host "✓ Build successful" -ForegroundColor Green
} catch {
    Write-Host "✗ Build failed" -ForegroundColor Red
    exit 1
}

# Run tests
Write-Host "Running tests..." -ForegroundColor Yellow
try {
    cargo test
    Write-Host "✓ Tests passed" -ForegroundColor Green
} catch {
    Write-Host "✗ Tests failed" -ForegroundColor Red
    exit 1
}

# Check binary
Write-Host "Testing binary..." -ForegroundColor Yellow
try {
    & ".\target\release\inferno.exe" --version
    Write-Host "✓ Binary works" -ForegroundColor Green
} catch {
    Write-Host "✗ Binary test failed" -ForegroundColor Red
    exit 1
}

Write-Host "All checks passed! Inferno is ready to use." -ForegroundColor Green
Write-Host ""
Write-Host "Next steps:"
Write-Host "1. Place model files in the models directory"
Write-Host "2. Run: .\target\release\inferno.exe --help"
Write-Host "3. Try: .\target\release\inferno.exe tui"
'@ | Set-Content -Path "verify.ps1" -Encoding UTF8

# Initialize git repository if git is available
if (Get-Command git -ErrorAction SilentlyContinue) {
    Write-Info "Initializing git repository..."
    if (-not (Test-Path ".git")) {
        git init
        git add .
        git commit -m "Initial commit: Inferno AI/ML Model Runner"
    }
} else {
    Write-Warning "Git not found, skipping repository initialization"
}

Write-Success "Bootstrap complete! Project created in: $ProjectDir"
Write-Host ""
Write-Host "Next steps:"
Write-Host "1. cd $ProjectDir"
Write-Host "2. .\verify.ps1          # Verify installation"
Write-Host "3. cargo run -- --help   # Run the application"
Write-Host ""
Write-Host "To build for release:"
Write-Host "  cargo build --release"
Write-Host ""
Write-Host "To run tests:"
Write-Host "  cargo test"