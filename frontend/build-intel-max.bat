@echo off
REM Meetily Windows Build Script with Vulkan + OpenBLAS + DirectML
REM Uses Vulkan (whisper-rs), OpenBLAS (CPU optimization), and ort-directml (ONNX Runtime)
REM Optimized for Intel Arc GPUs via Vulkan and DirectML backends

setlocal enabledelayedexpansion

REM --- Help ---
if "%~1" == "help" call :_print_help & exit /b 0
if "%~1" == "--help" call :_print_help & exit /b 0
if "%~1" == "-h" call :_print_help & exit /b 0
if "%~1" == "/?" call :_print_help & exit /b 0

echo.
echo ========================================
echo   Meetily Intel Core Ultra Build
echo ========================================
echo.

REM --- Port cleanup ---
echo [1/7] Checking for existing processes on port 3118...
for /f "tokens=5" %%a in ('netstat -aon ^| findstr :3118 2^>nul') do (
    echo    Killing process %%a on port 3118
    taskkill /PID %%a /F >nul 2>&1
)
echo.

REM --- Visual Studio detection ---
REM Order: BuildTools ^> Enterprise ^> 2019 BuildTools ^> Community ^> Professional
echo [2/7] Setting up Visual Studio environment...
set "VS_FOUND=0"

if exist "C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\VC\Auxiliary\Build\vcvars64.bat" (
    echo    Using Visual Studio 2022 Build Tools
    call "C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\VC\Auxiliary\Build\vcvars64.bat" >nul 2>&1
    set "VS_FOUND=1"
)
if "!VS_FOUND!"=="0" if exist "C:\Program Files (x86)\Microsoft Visual Studio\2022\Enterprise\VC\Auxiliary\Build\vcvars64.bat" (
    echo    Using Visual Studio 2022 Enterprise
    call "C:\Program Files (x86)\Microsoft Visual Studio\2022\Enterprise\VC\Auxiliary\Build\vcvars64.bat" >nul 2>&1
    set "VS_FOUND=1"
)
if "!VS_FOUND!"=="0" if exist "C:\Program Files (x86)\Microsoft Visual Studio\2019\BuildTools\VC\Auxiliary\Build\vcvars64.bat" (
    echo    Using Visual Studio 2019 Build Tools
    call "C:\Program Files (x86)\Microsoft Visual Studio\2019\BuildTools\VC\Auxiliary\Build\vcvars64.bat" >nul 2>&1
    set "VS_FOUND=1"
)
if "!VS_FOUND!"=="0" if exist "C:\Program Files (x86)\Microsoft Visual Studio\2022\Community\VC\Auxiliary\Build\vcvars64.bat" (
    echo    Using Visual Studio 2022 Community
    call "C:\Program Files (x86)\Microsoft Visual Studio\2022\Community\VC\Auxiliary\Build\vcvars64.bat" >nul 2>&1
    set "VS_FOUND=1"
)
if "!VS_FOUND!"=="0" if exist "C:\Program Files (x86)\Microsoft Visual Studio\2022\Professional\VC\Auxiliary\Build\vcvars64.bat" (
    echo    Using Visual Studio 2022 Professional
    set "VS_FOUND=1"
    call "C:\Program Files (x86)\Microsoft Visual Studio\2022\Professional\VC\Auxiliary\Build\vcvars64.bat" >nul 2>&1
)

if "!VS_FOUND!"=="0" if exist "C:\Program Files\Microsoft Visual Studio\2022\Community\VC\Auxiliary\Build\vcvars64.bat" (
    echo    Using Visual Studio 2022 Community
    set "VS_FOUND=1"
    call "C:\Program Files\Microsoft Visual Studio\2022\Community\VC\Auxiliary\Build\vcvars64.bat" >nul 2>&1
)
if "!VS_FOUND!"=="0" if exist "C:\Program Files\Microsoft Visual Studio\2022\BuildTools\VC\Auxiliary\Build\vcvars64.bat" (
    echo    Using Visual Studio 2022 Build Tools
    set "VS_FOUND=1"
    call "C:\Program Files\Microsoft Visual Studio\2022\BuildTools\VC\Auxiliary\Build\vcvars64.bat" >nul 2>&1
)

if "!VS_FOUND!"=="0" (
    echo    Warning: Visual Studio environment not found. Using manual SDK setup...
    if exist "C:\Program Files (x86)\Windows Kits\10\Lib\10.0.22621.0\um\x64\kernel32.lib" (
        set "LIB=C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\VC\Tools\MSVC\14.44.35207\lib\x64;C:\Program Files (x86)\Windows Kits\10\Lib\10.0.22621.0\um\x64;C:\Program Files (x86)\Windows Kits\10\Lib\10.0.22621.0\ucrt\x64"
        set "INCLUDE=C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\VC\Tools\MSVC\14.44.35207\include;C:\Program Files (x86)\Windows Kits\10\Include\10.0.22621.0\um;C:\Program Files (x86)\Windows Kits\10\Include\10.0.22621.0\shared;C:\Program Files (x86)\Windows Kits\10\Include\10.0.22621.0\ucrt"
        set "PATH=C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\VC\Tools\MSVC\14.44.35207\bin\HostX64\x64;C:\Program Files (x86)\Windows Kits\10\bin\10.0.22621.0\x64;%PATH%"
    ) else (
        echo.
        echo    Error: Could not find Visual Studio or Windows SDK.
        echo    Please install Visual Studio 2022 Build Tools with C++ workload:
        echo    https://visualstudio.microsoft.com/visual-cpp-build-tools/
        exit /b 1
    )
)
echo.

REM --- Environment variables ---
echo [3/7] Setting environment variables...

if exist "C:\Program Files\LLVM-18\bin\clang.dll" (
    set "LIBCLANG_PATH=C:\Program Files\LLVM-18\bin"
    echo    LIBCLANG_PATH: C:\Program Files\LLVM-18\bin
) else if exist "C:\Program Files\LLVM\bin\clang.dll" (
    set "LIBCLANG_PATH=C:\Program Files\LLVM\bin"
    echo    LIBCLANG_PATH: C:\Program Files\LLVM\bin
) else if exist "C:\Program Files\LLVM-18\bin" (
    set "LIBCLANG_PATH=C:\Program Files\LLVM-18\bin"
    echo    LIBCLANG_PATH: C:\Program Files\LLVM-18\bin (no clang.dll check)
) else if exist "C:\Program Files\LLVM\bin" (
    set "LIBCLANG_PATH=C:\Program Files\LLVM\bin"
    echo    LIBCLANG_PATH: C:\Program Files\LLVM\bin (no clang.dll check)
) else (
    echo.
    echo    Error: LLVM not found.
    echo    Install LLVM 18 from: https://github.com/llvm/llvm-project/releases
    echo    Or from: https://releases.llvm.org/
    exit /b 1
)

set "BINDGEN_EXTRA_CLANG_ARGS=--target=x86_64-pc-windows-msvc"
echo    BINDGEN_EXTRA_CLANG_ARGS: --target=x86_64-pc-windows-msvc

set "CL=%CL% /utf-8"
echo    CL: appended /utf-8

REM Forward INCLUDE to BINDGEN_EXTRA_CLANG_ARGS so bindgen finds Windows SDK headers
if defined INCLUDE (
    for %%i in ("%INCLUDE:;=","%") do if not "%%~i"=="" call set "BINDGEN_EXTRA_CLANG_ARGS=%%BINDGEN_EXTRA_CLANG_ARGS%% -imsvc %%~i"
)

if exist "C:\sherpa-onnx-md-lib" (
    set "SHERPA_ONNX_LIB_DIR=C:\sherpa-onnx-md-lib"
    echo    SHERPA_ONNX_LIB_DIR: C:\sherpa-onnx-md-lib
) else (
    echo.
    echo    Error: C:\sherpa-onnx-md-lib directory not found.
    echo    This directory is required for sherpa-onnx native libraries.
    echo    Please ensure the sherpa-onnx native library package is installed
    echo    at C:\sherpa-onnx-md-lib (sherpa-onnx-md-lib).
    exit /b 1
)
echo.

REM --- Build features detection ---
set "BUILD_FEATURES=vulkan ort-directml"

if exist "C:\OpenBLAS\include\cblas.h" (
    echo    OpenBLAS detected at C:\OpenBLAS
    echo    Enabling OpenBLAS feature for whisper-rs
    set "BUILD_FEATURES=vulkan openblas ort-directml"
    set "BLAS_INCLUDE_DIRS=C:\OpenBLAS\include"
    set "LIB=C:\OpenBLAS\lib;%LIB%"
    set "INCLUDE=C:\OpenBLAS\include;%INCLUDE%"
    set "PATH=C:\OpenBLAS\bin;%PATH%"
) else (
    echo    OpenBLAS not found at C:\OpenBLAS\include\cblas.h
    echo    Proceeding with default CPU optimizations
)
set "CMAKE_GENERATOR=Ninja"
echo    CMAKE_GENERATOR: Ninja (fixes ExternalProject race)
echo.

REM --- Find project root ---
echo [4/7] Locating project files...
set "SRC_PATH=."
if exist "package.json" (
    echo    Found package.json in current directory
) else if exist "frontend\package.json" (
    echo    Found package.json in frontend directory
    cd frontend
    set "SRC_PATH=frontend"
) else (
    echo.
    echo    Error: Could not find package.json
    echo    Make sure you're in the project root or frontend directory
    exit /b 1
)

if exist "src-tauri" (
    echo    Found src-tauri directory
) else if exist "..\src-tauri" (
    echo    Found src-tauri in parent directory
) else (
    echo    Warning: Could not find src-tauri directory
)
echo.

REM --- Check package manager ---
echo [5/7] Checking package manager...
where pnpm >nul 2>&1
if %errorlevel% equ 0 (
    echo    Using pnpm package manager
    set "USE_PNPM=1"
) else (
    where npm >nul 2>&1
    if %errorlevel% equ 0 (
        echo    Using npm package manager
        set "USE_PNPM=0"
    ) else (
        echo.
        echo    Error: Neither npm nor pnpm found
        echo    Install Node.js from: https://nodejs.org/
        exit /b 1
    )
)

where rustc >nul 2>&1
if %errorlevel% neq 0 (
    echo    Error: Rust toolchain not found
    echo    Install from: https://rustup.rs/
    exit /b 1
) else (
    for /f "tokens=2" %%i in ('rustc -vV ^| findstr "host:"') do set "TARGET_TRIPLE=%%i"
    echo    Rust toolchain: !TARGET_TRIPLE!
)
echo.

REM --- Build llama-helper sidecar ---
echo [6/7] Building llama-helper sidecar...
set "HELPER_DIR=..\llama-helper"
if not exist "!HELPER_DIR!\Cargo.toml" (
    set "HELPER_DIR=..\..\llama-helper"
)
if exist "!HELPER_DIR!\Cargo.toml" (
    pushd "!HELPER_DIR!"
    echo    Building llama-helper from !HELPER_DIR!...
    cargo build --release --features vulkan
    if errorlevel 1 (
        echo.
        echo    Error: Failed to build llama-helper
        popd
        exit /b 1
    )
    echo    llama-helper built successfully
    popd
) else (
    echo    Could not find llama-helper Cargo.toml, skipping sidecar build
)

REM --- Copy llama-helper binary ---
set "BINARIES_DIR=src-tauri\binaries"
if exist "!BINARIES_DIR!" (
    set "TARGET_DIR=..\target"
    if "!TARGET_TRIPLE:windows=!" neq "!TARGET_TRIPLE!" (
        set "SRC_BINARY=..\target\release\llama-helper.exe"
        set "DEST_BINARY=!BINARIES_DIR!\llama-helper-!TARGET_TRIPLE!.exe"
        if exist "!SRC_BINARY!" (
            copy /y "!SRC_BINARY!" "!DEST_BINARY!" >nul
            echo    Copied llama-helper to binaries directory
        )
    )
)
echo.

REM --- Build Tauri application ---
echo [7/7] Building Meetily with hardware acceleration...
echo.
echo    Cargo features: !BUILD_FEATURES!
echo.

if "%USE_PNPM%"=="1" (
    pnpm exec tauri build -- --features "!BUILD_FEATURES!"
) else (
    npx tauri build -- --features "!BUILD_FEATURES!"
)

if errorlevel 1 (
    echo.
    echo    Error: Build failed
    echo    Check the output above for compilation errors.
    exit /b 1
)

echo.
echo ========================================
echo   Build completed successfully!
echo ========================================
echo.
echo   Meetily built with GPU acceleration
echo   Features: !BUILD_FEATURES!
echo.
goto :eof

REM --- Help text ---
:_print_help
echo.
echo ========================================
echo   Meetily Intel Core Ultra Build - Help
echo ========================================
echo.
echo USAGE:
echo   build-intel-max.bat [OPTION]
echo.
echo OPTIONS:
echo   help, --help, -h, /?    Show this help message
echo.
echo DESCRIPTION:
echo   Builds Meetily with GPU hardware acceleration (Vulkan + DirectML),
echo   optimized for Intel Arc GPUs using Vulkan compute and DirectML
echo   processors with Vulkan + DirectML acceleration.
echo.
echo   - Vulkan    : Vulkan compute backend for GPU acceleration
echo                  (whisper-rs Vulkan backend for Intel Arc GPUs)
echo   - OpenBLAS  : Optimized BLAS for Intel CPUs
echo                  (whisper-rs matrix operations)
echo   - DirectML  : DirectML execution provider for ONNX Runtime
echo                  (ort-directml backend)
echo.
echo FEATURE MATRIX:
echo   +------------------+--------------------------------------+
echo   | vulkan           | whisper-rs ^< Vulkan ^> (ARC GPU)        |
echo   | openblas         | whisper-rs ^< OpenBLAS ^> (if detected)  |
echo   | ort-directml     | ONNX Runtime ^< DirectML ^>              |
echo   +------------------+--------------------------------------+
echo.
echo REQUIREMENTS:
echo   - Visual Studio 2022 Build Tools (or Enterprise/Community/Professional)
echo   - Windows SDK 10.0.22621.0 or compatible
echo   - Rust toolchain (rustup.rs)
echo   - LLVM 18 at C:\Program Files\LLVM-18\bin or C:\Program Files\LLVM\bin
echo   - Node.js with pnpm (or npm)
echo   - sherpa-onnx native libraries at C:\sherpa-onnx-md-lib
echo.
echo OPTIONAL COMPONENTS:
echo   - OpenBLAS at C:\OpenBLAS (extra CPU optimization)
echo   - Vulkan runtime (included with GPU drivers)
echo.
echo EXAMPLES:
echo   build-intel-max.bat              Build with Intel acceleration
echo   build-intel-max.bat --help       Show this help
echo.
echo   The build output will be in src-tauri/target/release/
echo.
goto :eof
