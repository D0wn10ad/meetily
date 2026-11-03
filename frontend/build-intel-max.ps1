# Meetily Windows Build Script with Vulkan + OpenBLAS + DirectML (PowerShell)
# Uses Vulkan (whisper-rs), OpenBLAS (CPU optimization), and ort-directml (ONNX Runtime)
# Optimized for Intel Arc GPUs via Vulkan and DirectML backends

$ErrorActionPreference = "Stop"

# --- Help ---
$helpFlags = @("help", "--help", "-h", "/?")
$isHelp = $false
foreach ($arg in $args) {
    if ($helpFlags -contains $arg.ToLower()) {
        $isHelp = $true
        break
    }
}
if ($isHelp) {
    Write-Host ""
    Write-Host "========================================" -ForegroundColor Cyan
    Write-Host "  Meetily Intel Core Ultra Build - Help" -ForegroundColor Cyan
    Write-Host "========================================" -ForegroundColor Cyan
    Write-Host ""
    Write-Host "USAGE:" -ForegroundColor White
    Write-Host "  .\build-intel-max.ps1 [OPTION]"
    Write-Host ""
    Write-Host "OPTIONS:" -ForegroundColor White
    Write-Host "  help, --help, -h, /?    Show this help message"
    Write-Host ""
    Write-Host "DESCRIPTION:" -ForegroundColor White
    Write-Host "  Builds Meetily with GPU hardware acceleration (Vulkan + DirectML),"
    Write-Host "  optimized for Intel Arc GPUs using Vulkan compute and DirectML"
    Write-Host "  backends."
    Write-Host ""
    Write-Host "  - Vulkan    : Vulkan compute backend for GPU acceleration"
    Write-Host "                (whisper-rs Vulkan backend for Intel Arc GPUs)"
    Write-Host "  - OpenBLAS  : Optimized BLAS for Intel CPUs"
    Write-Host "                (whisper-rs matrix operations)"
    Write-Host "  - DirectML  : DirectML execution provider for ONNX Runtime"
    Write-Host "                (ort-directml backend)"
    Write-Host ""
    Write-Host "FEATURE MATRIX:" -ForegroundColor White
    Write-Host "  +------------------+--------------------------------------+"
    Write-Host "  | vulkan           | whisper-rs < Vulkan > (ARC GPU)        |"
    Write-Host "  | openblas         | whisper-rs < OpenBLAS > (if detected)  |"
    Write-Host "  | ort-directml     | ONNX Runtime < DirectML >              |"
    Write-Host "  +------------------+--------------------------------------+"
    Write-Host ""
    Write-Host "REQUIREMENTS:" -ForegroundColor White
    Write-Host "  - Visual Studio 2022 Build Tools (or Enterprise/Community/Professional)"
    Write-Host "  - Windows SDK 10.0.22621.0 or compatible"
    Write-Host "  - Rust toolchain (rustup.rs)"
    Write-Host "  - LLVM 18 at C:\Program Files\LLVM-18\bin or C:\Program Files\LLVM\bin"
    Write-Host "  - Node.js with pnpm (or npm)"
    Write-Host "  - sherpa-onnx native libraries at C:\sherpa-onnx-md-lib"
    Write-Host ""
    Write-Host "OPTIONAL COMPONENTS:" -ForegroundColor White
    Write-Host "  - OpenBLAS at C:\OpenBLAS (extra CPU optimization)"
    Write-Host "  - Vulkan runtime (included with GPU drivers)"
    Write-Host ""
    Write-Host "EXAMPLES:" -ForegroundColor White
    Write-Host "  .\build-intel-max.ps1              Build with Intel acceleration"
    Write-Host "  .\build-intel-max.ps1 --help       Show this help"
    Write-Host ""
    Write-Host "  The build output will be in src-tauri/target/release/"
    Write-Host ""
    exit 0
}

Write-Host ""
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "  Meetily Intel Core Ultra Build" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""

# --- Step counter ---
$step = 0
function Write-Step($text) {
    $script:step++
    Write-Host "[$($script:step)/7] $text" -ForegroundColor Yellow
}

# --- [1/7] Port cleanup ---
Write-Step "Checking for existing processes on port 3118..."
try {
    $connections = netstat -aon | Select-String ":3118"
    foreach ($conn in $connections) {
        $parts = $conn -split '\s+'
        if ($parts.Count -ge 5) {
            $pid = $parts[4]
            Write-Host "   Killing process $pid on port 3118"
            Stop-Process -Id $pid -Force -ErrorAction SilentlyContinue
        }
    }
}
catch {
    Write-Host "   No processes found on port 3118" -ForegroundColor Gray
}
Write-Host ""

# --- [2/7] Visual Studio detection ---
Write-Step "Setting up Visual Studio environment..."

$vsPaths = @(
    "C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\VC\Auxiliary\Build\vcvars64.bat",
    "C:\Program Files (x86)\Microsoft Visual Studio\2022\Enterprise\VC\Auxiliary\Build\vcvars64.bat",
    "C:\Program Files (x86)\Microsoft Visual Studio\2019\BuildTools\VC\Auxiliary\Build\vcvars64.bat",
    "C:\Program Files (x86)\Microsoft Visual Studio\2022\Community\VC\Auxiliary\Build\vcvars64.bat",
    "C:\Program Files (x86)\Microsoft Visual Studio\2022\Professional\VC\Auxiliary\Build\vcvars64.bat"
    "C:\Program Files\Microsoft Visual Studio\2022\Community\VC\Auxiliary\Build\vcvars64.bat"
    "C:\Program Files\Microsoft Visual Studio\2022\BuildTools\VC\Auxiliary\Build\vcvars64.bat"
)

$vsLabels = @{
    "BuildTools" = "Visual Studio 2022 Build Tools"
    "Enterprise" = "Visual Studio 2022 Enterprise"
    "2019BuildTools" = "Visual Studio 2019 Build Tools"
    "Community" = "Visual Studio 2022 Community"
    "Professional" = "Visual Studio 2022 Professional"
}

$vsFound = $false
foreach ($path in $vsPaths) {
    if (Test-Path $path) {
        $label = "Visual Studio"
        foreach ($key in $vsLabels.Keys) {
            if ($path -match [regex]::Escape($key)) {
                $label = $vsLabels[$key]
                break
            }
        }
        Write-Host "   Using $label"
        cmd /c "`"$path`" >nul 2>&1 && set" | ForEach-Object {
            if ($_ -match '^([^=]+)=(.*)') {
                Set-Content "env:\$($matches[1])" $matches[2]
            }
        }
        $vsFound = $true
        break
    }
}

if (-not $vsFound) {
    Write-Host "   Warning: Visual Studio environment not found. Using manual SDK setup..." -ForegroundColor Yellow
    $sdkLib = "C:\Program Files (x86)\Windows Kits\10\Lib\10.0.22621.0\um\x64\kernel32.lib"
    if (Test-Path $sdkLib) {
        $env:LIB = "C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\VC\Tools\MSVC\14.44.35207\lib\x64;C:\Program Files (x86)\Windows Kits\10\Lib\10.0.22621.0\um\x64;C:\Program Files (x86)\Windows Kits\10\Lib\10.0.22621.0\ucrt\x64"
        $env:INCLUDE = "C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\VC\Tools\MSVC\14.44.35207\include;C:\Program Files (x86)\Windows Kits\10\Include\10.0.22621.0\um;C:\Program Files (x86)\Windows Kits\10\Include\10.0.22621.0\shared;C:\Program Files (x86)\Windows Kits\10\Include\10.0.22621.0\ucrt"
        $env:PATH = "C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\VC\Tools\MSVC\14.44.35207\bin\HostX64\x64;C:\Program Files (x86)\Windows Kits\10\bin\10.0.22621.0\x64;$env:PATH"
    }
    else {
        Write-Host ""
        Write-Host "   Error: Could not find Visual Studio or Windows SDK." -ForegroundColor Red
        Write-Host "   Please install Visual Studio 2022 Build Tools with C++ workload:" -ForegroundColor Red
        Write-Host "   https://visualstudio.microsoft.com/visual-cpp-build-tools/" -ForegroundColor Red
        exit 1
    }
}
Write-Host ""

# --- [3/7] Environment variables ---
Write-Step "Setting environment variables..."

$llvmPaths = @(
    @{ Path = "C:\Program Files\LLVM-18\bin"; Strict = $true },
    @{ Path = "C:\Program Files\LLVM\bin"; Strict = $true },
    @{ Path = "C:\Program Files\LLVM-18\bin"; Strict = $false },
    @{ Path = "C:\Program Files\LLVM\bin"; Strict = $false }
)

$llvmFound = $false
foreach ($llvm in $llvmPaths) {
    $dirExists = Test-Path $llvm.Path
    $dllExists = if ($llvm.Strict) { Test-Path "$($llvm.Path)\clang.dll" } else { $true }
    if ($dirExists -and $dllExists) {
        $env:LIBCLANG_PATH = $llvm.Path
        $suffix = if (-not $dllExists) { " (no clang.dll check)" } else { "" }
        Write-Host "   LIBCLANG_PATH: $($llvm.Path)$suffix"
        $llvmFound = $true
        break
    }
}

if (-not $llvmFound) {
    Write-Host ""
    Write-Host "   Error: LLVM not found." -ForegroundColor Red
    Write-Host "   Install LLVM 18 from: https://github.com/llvm/llvm-project/releases" -ForegroundColor Red
    exit 1
}

$env:BINDGEN_EXTRA_CLANG_ARGS = "--target=x86_64-pc-windows-msvc"
Write-Host "   BINDGEN_EXTRA_CLANG_ARGS: $env:BINDGEN_EXTRA_CLANG_ARGS"

# Forward INCLUDE to BINDGEN_EXTRA_CLANG_ARGS so bindgen finds Windows SDK headers
if ($env:INCLUDE) {
    $env:INCLUDE -split ';' | ForEach-Object {
        if ($_) {
            $env:BINDGEN_EXTRA_CLANG_ARGS += " -imsvc $_"
        }
    }
}

if ($env:CL) {
    $env:CL = "$env:CL /utf-8"
}
else {
    $env:CL = " /utf-8"
}
Write-Host "   CL: appended /utf-8"

if (Test-Path "C:\sherpa-onnx-md-lib") {
    $env:SHERPA_ONNX_LIB_DIR = "C:\sherpa-onnx-md-lib"
    Write-Host "   SHERPA_ONNX_LIB_DIR: C:\sherpa-onnx-md-lib"
}
else {
    Write-Host ""
    Write-Host "   Error: C:\sherpa-onnx-md-lib directory not found." -ForegroundColor Red
    Write-Host "   This directory is required for sherpa-onnx native libraries." -ForegroundColor Red
    Write-Host "   Please ensure the sherpa-onnx native library package is installed" -ForegroundColor Red
    Write-Host "   at C:\sherpa-onnx-md-lib (sherpa-onnx-md-lib)." -ForegroundColor Red
    exit 1
}
Write-Host ""

# --- Development features detection ---
$buildFeatures = @("vulkan", "ort-directml")

if (Test-Path "C:\OpenBLAS\include\cblas.h") {
    Write-Host "   OpenBLAS detected at C:\OpenBLAS" -ForegroundColor Green
    Write-Host "   Enabling OpenBLAS feature for whisper-rs" -ForegroundColor Green
    $buildFeatures += "openblas"
    $env:BLAS_INCLUDE_DIRS = "C:\OpenBLAS\include"
    $env:LIB = "$env:LIB;C:\OpenBLAS\lib"
    $env:INCLUDE = "$env:INCLUDE;C:\OpenBLAS\include"
    $env:PATH = "$env:PATH;C:\OpenBLAS\bin"
}
else {
    Write-Host "   OpenBLAS not found at C:\OpenBLAS\include\cblas.h" -ForegroundColor Gray
    Write-Host "   Proceeding with default CPU optimizations" -ForegroundColor Gray
}

$env:CMAKE_GENERATOR = "Ninja"
Write-Host "   CMAKE_GENERATOR: Ninja (fixes ExternalProject race)"

Write-Host ""

# --- [4/7] Find project root ---
Write-Step "Locating project files..."
if (Test-Path "package.json") {
    Write-Host "   Found package.json in current directory"
}
elseif (Test-Path "frontend\package.json") {
    Write-Host "   Found package.json in frontend directory"
    Set-Location "frontend"
}
else {
    Write-Host ""
    Write-Host "   Error: Could not find package.json" -ForegroundColor Red
    Write-Host "   Make sure you're in the project root or frontend directory" -ForegroundColor Red
    exit 1
}

if (Test-Path "src-tauri") {
    Write-Host "   Found src-tauri directory"
}
elseif (Test-Path "..\src-tauri") {
    Write-Host "   Found src-tauri in parent directory"
}
else {
    Write-Host "   Warning: Could not find src-tauri directory" -ForegroundColor Yellow
}
Write-Host ""

# --- [5/7] Check toolchain ---
Write-Step "Checking toolchain..."

$hasPnpm = $null -ne (Get-Command "pnpm" -ErrorAction SilentlyContinue)
$hasNpm = $null -ne (Get-Command "npm" -ErrorAction SilentlyContinue)

if ($hasPnpm) {
    Write-Host "   Using pnpm package manager"
    $usePnpm = $true
}
elseif ($hasNpm) {
    Write-Host "   Using npm package manager"
    $usePnpm = $false
}
else {
    Write-Host ""
    Write-Host "   Error: Neither npm nor pnpm found" -ForegroundColor Red
    Write-Host "   Install Node.js from: https://nodejs.org/" -ForegroundColor Red
    exit 1
}

$rustVersion = rustc --version
if ($LASTEXITCODE -ne 0) {
    Write-Host "   Error: Rust toolchain not found" -ForegroundColor Red
    Write-Host "   Install from: https://rustup.rs/" -ForegroundColor Red
    exit 1
}

$targetTriple = (rustc -vV | Select-String "host:").ToString().Split(':')[1].Trim()
Write-Host "   Rust toolchain: $targetTriple"
Write-Host ""

# --- [6/7] Build llama-helper sidecar ---
Write-Step "Building llama-helper sidecar..."

$helperDir = "..\llama-helper"
if (-not (Test-Path "$helperDir\Cargo.toml")) {
    $helperDir = "..\..\llama-helper"
}

if (Test-Path "$helperDir\Cargo.toml") {
    Push-Location $helperDir
    try {
        Write-Host "   Building llama-helper from $helperDir..."
        cargo build --release --features vulkan
        if ($LASTEXITCODE -ne 0) {
            Write-Host ""
            Write-Host "   Error: Failed to build llama-helper" -ForegroundColor Red
            exit 1
        }
        Write-Host "   llama-helper built successfully" -ForegroundColor Green
    }
    finally {
        Pop-Location
    }
}
else {
    Write-Host "   Could not find llama-helper Cargo.toml, skipping sidecar build" -ForegroundColor Yellow
}

$binariesDir = "src-tauri\binaries"
if ($targetTriple -match "windows") {
    $srcBinary = "..\target\release\llama-helper.exe"
    $destBinary = "$binariesDir\llama-helper-$targetTriple.exe"
    if ((Test-Path $binariesDir) -and (Test-Path $srcBinary)) {
        Copy-Item -Path $srcBinary -Destination $destBinary -Force
        Write-Host "   Copied llama-helper to binaries directory" -ForegroundColor Green
    }
}
Write-Host ""

# --- [7/7] Build Tauri application ---
Write-Step "Building Meetily with hardware acceleration..."

$featuresStr = $buildFeatures -join ","
Write-Host ""
Write-Host "   Cargo features: $featuresStr" -ForegroundColor Magenta
Write-Host ""

if ($usePnpm) {
    & pnpm exec tauri build -- --features $featuresStr
}
else {
    & npx tauri build -- --features $featuresStr
}

if ($LASTEXITCODE -ne 0) {
    Write-Host ""
    Write-Host "   Error: Build failed" -ForegroundColor Red
    Write-Host "   Check the output above for compilation errors." -ForegroundColor Red
    exit 1
}

Write-Host ""
Write-Host "========================================" -ForegroundColor Green
  Write-Host "  Build completed successfully!"
Write-Host "========================================" -ForegroundColor Green
Write-Host ""
  Write-Host "  Meetily built with GPU acceleration" -ForegroundColor White
Write-Host "  Features: $featuresStr" -ForegroundColor Cyan
Write-Host ""
