#!/usr/bin/env pwsh
# Verify the Windows build is correct after switching sherpa-onnx to its shared
# (DLL) prebuilt: the staged sherpa-onnx-c-api.dll exists, installers bundle it
# with exactly one onnxruntime.dll, and the app_lib.dll actually loads (proving
# the DLL dependency graph resolves at runtime, not just at link time).

$ErrorActionPreference = 'Stop'

$repoRoot = (Resolve-Path (Join-Path $PSScriptRoot '..')).Path
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
function Get-InstallerFileNames([string]$InstallerPath) {
    $names = [System.Collections.Generic.List[string]]::new()
    $tempBase = if ($env:RUNNER_TEMP) { $env:RUNNER_TEMP } else { [IO.Path]::GetTempPath() }
    $tmp = Join-Path $tempBase ("installer-inspect-" + [Guid]::NewGuid().ToString('N'))
    New-Item -ItemType Directory -Path $tmp -Force | Out-Null

    if ($InstallerPath -match '\.msi$') {
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
        } catch {
            Write-Warning "MSI File table query failed for $InstallerPath ($_); falling back to msiexec administrative extract."
            $names.Clear()
            $p = Start-Process msiexec -ArgumentList @('/a', "`"$InstallerPath`"", '/qn', "TARGETDIR=`"$tmp`"") -Wait -PassThru
            if ($p.ExitCode -eq 0) {
                Get-ChildItem -Path $tmp -Recurse -File -ErrorAction SilentlyContinue |
                    ForEach-Object { $names.Add($_.Name) }
            }
        }
    } elseif ($InstallerPath -match '\.exe$') {
        & 7z x -y "-o$tmp" -- $InstallerPath | Out-Null
        if ($LASTEXITCODE -eq 0) {
            Get-ChildItem -Path $tmp -Recurse -File -ErrorAction SilentlyContinue |
                ForEach-Object { $names.Add($_.Name) }
        }
    }

    Remove-Item -Path $tmp -Recurse -Force -ErrorAction SilentlyContinue
    return $names
}

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
            $names = @(Get-InstallerFileNames $installer.FullName)
            $sherpaHits = @($names | Where-Object { $_ -match 'sherpa-onnx-c-api\.dll' })
            if ($sherpaHits.Count -lt 1) {
                Fail "installer $($installer.Name) does not bundle sherpa-onnx-c-api.dll"
            }

            $ortHits = @($names | Where-Object { $_ -match 'onnxruntime\.dll' })
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
