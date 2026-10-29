#!/usr/bin/env pwsh
# Verify the Windows build is correct after switching sherpa-onnx to its shared
# (DLL) prebuilt: the staged sherpa-onnx-c-api.dll exists, installers bundle it
# with exactly one onnxruntime.dll, and the app_lib.dll actually loads (proving
# the DLL dependency graph resolves at runtime, not just at link time).

param([string]$RepoRoot = '')

$ErrorActionPreference = 'Stop'

# Repo root: explicit -RepoRoot, then MEETILY_REPO_ROOT, then git, then the
# historical one-directory-up fallback.
if (-not $RepoRoot) {
    if ($env:MEETILY_REPO_ROOT) {
        $RepoRoot = $env:MEETILY_REPO_ROOT
    } else {
        $gitRoot = $null
        if (Get-Command git -ErrorAction SilentlyContinue) {
            $gitRoot = (& git -C $PSScriptRoot rev-parse --show-toplevel 2>$null)
            if ($LASTEXITCODE -ne 0) { $gitRoot = $null }
        }
        if ($gitRoot -and (Test-Path $gitRoot)) {
            $RepoRoot = $gitRoot
        } else {
            $RepoRoot = (Resolve-Path (Join-Path $PSScriptRoot '..')).Path
        }
    }
}
$repoRoot = $RepoRoot
$rootTarget = Join-Path $repoRoot 'target'
$legacyTarget = Join-Path $repoRoot 'frontend/src-tauri/target'
if (Test-Path $rootTarget) {
    $targetDir = $rootTarget
} else {
    $targetDir = $legacyTarget
}
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

# Return the human-readable file names contained in an installer. WiX MSIs name
# their cab payload entries with generic keys (PathFile_I<uuid>), so 7-Zip's
# listing is useless — the real names live in the MSI 'File' table. NSIS .exe
# archives store real names, but we still extract + enumerate for robustness.
function Get-InstallerFileNames([string]$InstallerPath, [bool]$MsiComAvailable) {
    $names = [System.Collections.Generic.List[string]]::new()
    $tempBase = if ($env:RUNNER_TEMP) { $env:RUNNER_TEMP } else { [IO.Path]::GetTempPath() }
    $tmp = Join-Path $tempBase ("installer-inspect-" + [Guid]::NewGuid().ToString('N'))
    New-Item -ItemType Directory -Path $tmp -Force | Out-Null
    $extractedDirs = @()

    if ($InstallerPath -match '\.msi$') {
        $comOk = $false
        if ($MsiComAvailable) {
            try {
                $installer = New-Object -ComObject WindowsInstaller.Installer
                $db = $installer.GetType().InvokeMember('OpenDatabase', 'InvokeMethod', $null, $installer, @($InstallerPath, 0))
                $view = $db.GetType().InvokeMember('OpenView', 'InvokeMethod', $null, $db, @('SELECT `FileName` FROM `File`'))
                $view.GetType().InvokeMember('Execute', 'InvokeMethod', $null, $view, $null)
                while ($true) {
                    $rec = $view.GetType().InvokeMember('Fetch', 'InvokeMethod', $null, $view, $null)
                    if ($null -eq $rec) { break }
                    $names.Add([string]$rec.GetType().InvokeMember('StringData', 'InvokeMethod', $null, $rec, @(1)))
                }
                $view.GetType().InvokeMember('Close', 'InvokeMethod', $null, $view, $null)
                $comOk = $true
            } catch {
                Write-Warning "MSI File table query failed for $InstallerPath ($_); falling back to msiexec administrative extract."
                $names.Clear()
            }
        }
        if (-not $comOk) {
            $names.Clear()
            $p = Start-Process msiexec -ArgumentList @('/a', "`"$InstallerPath`"", '/qn', "TARGETDIR=`"$tmp`"") -Wait -PassThru
            if ($p.ExitCode -eq 0) {
                $extractedDirs += $tmp
            }
        }
    } elseif ($InstallerPath -match '\.exe$') {
        & $sevenZip.Source x -y "-o$tmp" -- $InstallerPath | Out-Null
        if ($LASTEXITCODE -eq 0) {
            $extractedDirs += $tmp
        }
    }

    # Only the DLLs under test matter here; enumerating just these patterns
    # avoids walking every file an installer payload extract may contain.
    foreach ($dir in $extractedDirs) {
        foreach ($pattern in @('sherpa-onnx-c-api*.dll', 'onnxruntime*.dll')) {
            foreach ($file in [IO.Directory]::EnumerateFiles($dir, $pattern, [IO.SearchOption]::AllDirectories)) {
                $names.Add([IO.Path]::GetFileName($file))
            }
        }
    }

    Remove-Item -Path $tmp -Recurse -Force -ErrorAction SilentlyContinue
    return $names
}

# --- 2. Installer bundles must contain the DLLs ------------------------------
$installers = @()
if (Test-Path $targetDir) {
    $installers = @(
        foreach ($file in [IO.Directory]::EnumerateFiles($targetDir, '*', [IO.SearchOption]::AllDirectories)) {
            $fi = [IO.FileInfo]::new($file)
            if ($fi.Extension -in @('.exe', '.msi') -and $fi.FullName -match '[\\/]bundle[\\/](nsis|msi)[\\/]') {
                $fi
            }
        }
    )
}

if ($installers.Count -eq 0) {
    Write-Warning "No installer artifacts found under $targetDir — skipping archive content checks (artifact presence is verified elsewhere)."
} else {
    $sevenZip = Get-Command 7z -ErrorAction SilentlyContinue
    if (-not $sevenZip) {
        Write-Warning "7z not available — skipping installer content checks."
    } else {
        # Detect the WindowsInstaller COM type once so its absence is reported
        # loudly instead of silently degrading to the msiexec path per installer.
        $msiComAvailable = $false
        try {
            $null = New-Object -ComObject WindowsInstaller.Installer
            $msiComAvailable = $true
        } catch {
            Write-Warning "WindowsInstaller COM type is unavailable — the MSI File-table check is being SKIPPED; the msiexec administrative-extract fallback will be used."
        }

        foreach ($installer in $installers) {
            Write-Host "Inspecting installer: $($installer.FullName)"
            $names = @(Get-InstallerFileNames -InstallerPath $installer.FullName -MsiComAvailable $msiComAvailable)
            $sherpaHits = @($names | Where-Object { $_ -match 'sherpa-onnx-c-api\.dll' })
            if ($sherpaHits.Count -lt 1) {
                Fail "installer $($installer.Name) does not bundle sherpa-onnx-c-api.dll"
            }

            # Delivery invariant: exactly one onnxruntime*.dll (versioned names such as onnxruntime64_*.dll are allowed).
            $ortHits = @($names | Where-Object { $_ -match '^onnxruntime.*\.dll$' })
            if (($ortHits.Count -lt 1) -or ($ortHits.Count -gt 1)) {
                Fail "installer $($installer.Name) must contain exactly 1 onnxruntime.dll, found $($ortHits.Count)"
            }
            Write-Host "PASS: $($installer.Name) bundles sherpa-onnx-c-api.dll and exactly 1 onnxruntime.dll"
        }
    }
}

# --- 3. LoadLibrary smoke test ------------------------------------------------
$appLibCandidates = @()
if (Test-Path $targetDir) {
    $appLibCandidates = @(
        foreach ($file in [IO.Directory]::EnumerateFiles($targetDir, 'app_lib.dll', [IO.SearchOption]::AllDirectories)) {
            [IO.FileInfo]::new($file)
        }
    )
}
$appLib = $appLibCandidates |
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

Add-Type -TypeDefinition @'
using System;
using System.Runtime.InteropServices;
public static class NativeWin32 {
    [DllImport("kernel32", SetLastError = true, CharSet = CharSet.Unicode)]
    public static extern IntPtr LoadLibraryExW(string lpFileName, IntPtr hFile, uint dwFlags);
    [DllImport("kernel32", SetLastError = true)]
    [return: MarshalAs(UnmanagedType.Bool)]
    public static extern bool FreeLibrary(IntPtr hModule);
    [DllImport("kernel32", SetLastError = true)]
    public static extern uint GetLastError();
}
'@

try {
    New-Item -ItemType Directory -Path $tempDir -Force | Out-Null

    Copy-Item $appLib.FullName (Join-Path $tempDir 'app_lib.dll') -Force
    Copy-Item $sherpaDll (Join-Path $tempDir 'sherpa-onnx-c-api.dll') -Force
    Copy-Item $ortDll (Join-Path $tempDir 'onnxruntime.dll') -Force

    # Restricted search order: the DLL's own directory, the application directory
    # and System32 only — PATH is never consulted for dependency resolution.
    $LOAD_LIBRARY_SEARCH_DLL_LOAD_DIR = 0x00001000
    $LOAD_LIBRARY_SEARCH_SYSTEM32 = 0x00000800
    $LOAD_LIBRARY_SEARCH_APPLICATION_DIR = 0x00002000
    $dllSearchFlags = $LOAD_LIBRARY_SEARCH_DLL_LOAD_DIR -bor $LOAD_LIBRARY_SEARCH_SYSTEM32 -bor $LOAD_LIBRARY_SEARCH_APPLICATION_DIR  # 0x00003800

    $appLibFull = (Resolve-Path (Join-Path $tempDir 'app_lib.dll')).Path
    $handle = [NativeWin32]::LoadLibraryExW($appLibFull, [IntPtr]::Zero, $dllSearchFlags)
    if ($handle -eq [IntPtr]::Zero) {
        $code = [NativeWin32]::GetLastError()
        $hint = switch ($code) {
            126 { 'missing dependency DLL' }
            127 { 'missing export' }
            1114 { 'DLL initialization routine failed' }
            1115 { 'system shutting down or resource load cycle' }
            1261 { 'image cannot have the same TEB size' }
            default { 'Win32 error code' }
        }
        Fail "LoadLibraryExW failed for $appLibFull (Win32 error $code = $hint)"
    }
    Write-Host "PASS: LoadLibraryExW succeeded for app_lib.dll (handle=$handle)"
    [void][NativeWin32]::FreeLibrary($handle)
} finally {
    Remove-Item -Path $tempDir -Recurse -Force -ErrorAction SilentlyContinue
}

Write-Host "PASS: Windows DLL linking verification complete"
exit 0
