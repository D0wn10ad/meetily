#!/usr/bin/env pwsh
# Verify the Windows build is correct after switching sherpa-onnx to its shared
# (DLL) prebuilt: the staged sherpa-onnx-c-api.dll exists, installers bundle it
# with exactly one onnxruntime.dll, and the app_lib.dll actually loads (proving
# the DLL dependency graph resolves at runtime, not just at link time).

$ErrorActionPreference = 'Stop'

$repoRoot = (Resolve-Path (Join-Path $PSScriptRoot '..')).Path
$targetDir = Join-Path $repoRoot 'frontend/src-tauri/target'
$sherpaDll = Join-Path $repoRoot 'frontend/src-tauri/binaries/sherpa/sherpa-onnx-c-api.dll'
$ortDll = Join-Path $repoRoot 'frontend/src-tauri/binaries/onnxruntime/onnxruntime.dll'

function Fail([string]$Message) {
    Write-Host "FAIL: $Message" -ForegroundColor Red
    exit 1
}

# --- 1. Staged sherpa DLL must exist -----------------------------------------
if (-not (Test-Path $sherpaDll)) {
    Fail "staged DLL not found: $sherpaDll (the sherpa-onnx 'shared' feature must be active on Windows)"
}
Write-Host "PASS: staged $sherpaDll"

# --- 2. Installer bundles must contain the DLLs ------------------------------
$installers = @()
if (Test-Path $targetDir) {
    $installers = @(
        Get-ChildItem -Path $targetDir -Recurse -File -Include *.exe, *.msi -ErrorAction SilentlyContinue |
            Where-Object { $_.FullName -match '[\\/]bundle[\\/](nsis|msi)[\\/]' }
    )
}

if ($installers.Count -eq 0) {
    Write-Warning "No installer artifacts found under $targetDir — skipping archive content checks (artifact presence is verified elsewhere)."
} else {
    $sevenZip = Get-Command 7z -ErrorAction SilentlyContinue
    if (-not $sevenZip) {
        Write-Warning "7z not available — skipping installer content checks."
    } else {
        foreach ($installer in $installers) {
            Write-Host "Inspecting installer: $($installer.FullName)"
            $listing = (& 7z l -- $installer.FullName 2>&1 | Out-String)
            if ($LASTEXITCODE -ne 0) {
                Fail "7z failed (exit $LASTEXITCODE) listing $($installer.FullName)"
            }
            $lines = $listing -split "`r?`n"

            $sherpaHits = @($lines | Where-Object { $_ -match 'sherpa-onnx-c-api\.dll' })
            if ($sherpaHits.Count -lt 1) {
                Fail "installer $($installer.Name) does not bundle sherpa-onnx-c-api.dll"
            }

            $ortHits = @($lines | Where-Object { $_ -match 'onnxruntime\.dll' })
            if ($ortHits.Count -ne 1) {
                Fail "installer $($installer.Name) must contain exactly 1 onnxruntime.dll, found $($ortHits.Count)"
            }
            Write-Host "PASS: $($installer.Name) bundles sherpa-onnx-c-api.dll and exactly 1 onnxruntime.dll"
        }
    }
}

# --- 3. LoadLibrary smoke test ------------------------------------------------
$appLib = Get-ChildItem -Path $targetDir -Recurse -File -Filter app_lib.dll -ErrorAction SilentlyContinue |
    Sort-Object { if ($_.FullName -match '[\\/]release[\\/]') { 0 } else { 1 } } |
    Select-Object -First 1
if (-not $appLib) {
    Fail "built app_lib.dll not found under $targetDir"
}
if (-not (Test-Path $ortDll)) {
    Fail "bundled onnxruntime.dll not found: $ortDll"
}

$tempBase = if ($env:RUNNER_TEMP) { $env:RUNNER_TEMP } else { [IO.Path]::GetTempPath() }
$tempDir = Join-Path $tempBase ("sherpa-dll-smoke-" + [Guid]::NewGuid().ToString('N'))
New-Item -ItemType Directory -Path $tempDir -Force | Out-Null

Copy-Item $appLib.FullName (Join-Path $tempDir 'app_lib.dll') -Force
Copy-Item $sherpaDll (Join-Path $tempDir 'sherpa-onnx-c-api.dll') -Force
Copy-Item $ortDll (Join-Path $tempDir 'onnxruntime.dll') -Force
# Dependency resolution searches the process dir + PATH, not the loaded DLL's dir,
# so put the smoke dir on PATH before loading.
$env:PATH = "$tempDir;$env:PATH"

Add-Type -TypeDefinition @'
using System;
using System.Runtime.InteropServices;
public static class NativeWin32 {
    [DllImport("kernel32", SetLastError = true, CharSet = CharSet.Unicode)]
    public static extern IntPtr LoadLibraryW(string lpFileName);
    [DllImport("kernel32", SetLastError = true)]
    [return: MarshalAs(UnmanagedType.Bool)]
    public static extern bool FreeLibrary(IntPtr hModule);
    [DllImport("kernel32", SetLastError = true)]
    public static extern uint GetLastError();
}
'@

$appLibFull = (Resolve-Path (Join-Path $tempDir 'app_lib.dll')).Path
$handle = [NativeWin32]::LoadLibraryW($appLibFull)
if ($handle -eq [IntPtr]::Zero) {
    $code = [NativeWin32]::GetLastError()
    $hint = switch ($code) {
        126 { 'missing dependency DLL' }
        127 { 'missing export' }
        default { 'Win32 error code' }
    }
    Fail "LoadLibraryW failed for $appLibFull (Win32 error $code = $hint)"
}
Write-Host "PASS: LoadLibraryW succeeded for app_lib.dll (handle=$handle)"
[void][NativeWin32]::FreeLibrary($handle)

Remove-Item -Path $tempDir -Recurse -Force -ErrorAction SilentlyContinue
Write-Host "PASS: Windows DLL linking verification complete"
exit 0
