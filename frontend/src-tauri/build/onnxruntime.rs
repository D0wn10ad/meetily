//! Stages the ONNX Runtime DLL on Windows, fully offline.
//!
//! sherpa-onnx-sys's build script (`copy_windows_runtime_dlls`) already places
//! its prebuilt `onnxruntime.dll` (ORT 1.28.2, DirectML-capable) into the cargo
//! profile directory before this app's build script runs (dependency build
//! scripts run first). We copy that exact DLL into `binaries/onnxruntime/` and
//! verify it against pinned size/SHA-256 constants — no network access.
//!
//! Only `onnxruntime.dll` is staged. `onnxruntime_providers_shared.dll` is
//! deliberately NOT staged: the 1.28.2 DLL does not import it, and CI
//! (.github/verify-windows-dlls.ps1) enforces exactly one `onnxruntime*.dll`.

use bzip2::read::BzDecoder;
use sha2::{Digest, Sha256};
use std::{
    env,
    fs::{self, File},
    io::{self, Read},
    path::{Component, Path, PathBuf},
    process,
};
use tar::Archive;

#[cfg(windows)]
use std::os::windows::fs::MetadataExt;

#[allow(dead_code)]
mod build_env {
    include!("../src/build_env.rs");
}

const WINDOWS_X64_TARGET: &str = "x86_64-pc-windows-msvc";
const DLL_NAME: &str = "onnxruntime.dll";
/// Pinned from the sherpa-onnx v1.13.8 win-x64 shared prebuilt (ORT 1.28.2).
const DLL_SIZE: u64 = 17_799_168;
const DLL_SHA256: &str = "7f66f939a881baf4f46a2216496798edf4a1429878b646d12674aa62f27d8a25";

pub fn ensure_onnxruntime_runtime() {
    if env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        return;
    }
    println!("cargo:rerun-if-changed=binaries/onnxruntime");
    // Re-run on every GitHub Actions run even on a full cache hit: the staged
    // binaries/onnxruntime dir is restored by actions/cache, so the directive
    // above alone would skip this script and leave onnxruntime_providers_shared.dll
    // in the profile dir. GITHUB_RUN_ID is set per Actions run and inert locally.
    println!("cargo:rerun-if-env-changed=GITHUB_RUN_ID");

    let target = env::var("TARGET").expect("TARGET environment variable not set");
    if target != WINDOWS_X64_TARGET {
        panic!("ONNX Runtime is bundled only for {WINDOWS_X64_TARGET}; got {target}");
    }

    let manifest_dir =
        env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR environment variable not set");
    let destination = Path::new(&manifest_dir)
        .join("binaries")
        .join("onnxruntime");

    // sherpa-onnx-sys's copy_windows_runtime_dlls also drops
    // onnxruntime_providers_shared.dll into the cargo profile dir, which Tauri
    // would otherwise bundle as a runtime dependency. The 1.28.2 onnxruntime.dll
    // does not import it, and CI (.github/verify-windows-dlls.ps1) enforces exactly
    // one onnxruntime*.dll, so it must not be bundled. Resolve the profile dir
    // via the same OUT_DIR-ancestor walk as build/sherpa.rs.
    let out_dir = env::var("OUT_DIR").expect("OUT_DIR environment variable not set");
    let profile = build_env::expect_profile(env::var("PROFILE"));
    let profile_dir = build_env::resolve_profile_dir(Path::new(&out_dir), &profile);
    let providers_shared = profile_dir.join("onnxruntime_providers_shared.dll");
    if providers_shared.exists() {
        fs::remove_file(&providers_shared).unwrap_or_else(|error| {
            panic!(
                "Failed to remove {}: {error}",
                providers_shared.display()
            )
        });
        println!(
            "cargo:warning=Removed {} so it is not bundled",
            providers_shared.display()
        );
    }

    match verify_staged_runtime(&destination) {
        Ok(()) => {
            println!(
                "cargo:warning=Using verified bundled ONNX Runtime from {}",
                destination.display()
            );
            return;
        }
        Err(verification_error) => match fs::symlink_metadata(&destination) {
            Ok(metadata) if is_link_or_reparse_point(&metadata) => {
                panic!(
                    "Refusing to replace linked or reparse-point ONNX Runtime stage at {}: {verification_error}",
                    destination.display()
                );
            }
            Ok(metadata) if metadata.is_dir() => {
                println!(
                    "cargo:warning=Replacing invalid bundled ONNX Runtime: {verification_error}"
                );
                fs::remove_dir_all(&destination).unwrap_or_else(|remove_error| {
                    panic!(
                        "Failed to remove invalid ONNX Runtime directory at {}: {remove_error}",
                        destination.display()
                    )
                });
            }
            Ok(metadata) if metadata.is_file() => {
                println!(
                    "cargo:warning=Replacing invalid bundled ONNX Runtime: {verification_error}"
                );
                fs::remove_file(&destination).unwrap_or_else(|remove_error| {
                    panic!(
                        "Failed to remove invalid ONNX Runtime file at {}: {remove_error}",
                        destination.display()
                    )
                });
            }
            Ok(_) => {
                panic!(
                    "Refusing to replace special ONNX Runtime stage at {}: {verification_error}",
                    destination.display()
                );
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => {
                panic!(
                    "Failed to inspect invalid ONNX Runtime stage at {}: {error}",
                    destination.display()
                );
            }
        },
    }

    // Source DLL: prefer the cargo profile-dir copy (populated by
    // sherpa-onnx-sys's copy_windows_runtime_dlls before this script), else fall
    // back to a verified sherpa-onnx prebuilt-cache copy. The profile-dir copy
    // can be missing when a rust-cache restore replays build artifacts without
    // re-running sherpa-onnx-sys's DLL copy.
    let source = resolve_runtime_source(&profile_dir);

    fs::create_dir_all(
        destination
            .parent()
            .expect("ONNX Runtime destination has no parent"),
    )
    .expect("Failed to create ONNX Runtime binaries directory");

    let temporary_destination =
        destination.with_file_name(format!(".onnxruntime-{}-{}", process::id(), target));
    let _ = fs::remove_dir_all(&temporary_destination);

    let result = stage_runtime(&source, &temporary_destination);

    if let Err(error) = result {
        let _ = fs::remove_dir_all(&temporary_destination);
        panic!("Failed to stage bundled ONNX Runtime: {error}");
    }

    fs::rename(&temporary_destination, &destination).unwrap_or_else(|error| {
        let _ = fs::remove_dir_all(&temporary_destination);
        panic!(
            "Failed to finalize bundled ONNX Runtime at {}: {error}",
            destination.display()
        );
    });

    verify_staged_runtime(&destination).unwrap_or_else(|error| {
        panic!("Bundled ONNX Runtime verification failed after staging: {error}")
    });
    println!(
        "cargo:warning=Bundled verified ONNX Runtime at {}",
        destination.display()
    );
}

/// True only when `entry_path` is exactly `<archive>/lib/onnxruntime.dll`.
/// Rejects absolute paths, parent traversal, and every other member so a
/// malformed archive can never stage `onnxruntime_providers_shared.dll`.
fn archive_member_matches(entry_path: &Path, archive_name: &str) -> bool {
    let mut actual = Vec::new();
    for component in entry_path.components() {
        match component {
            Component::Normal(part) => actual.push(part.to_string_lossy().into_owned()),
            Component::CurDir => {}
            _ => return false,
        }
    }
    let expected: Vec<String> = Path::new(archive_name)
        .join("lib")
        .join(DLL_NAME)
        .components()
        .filter_map(|component| match component {
            Component::Normal(part) => Some(part.to_string_lossy().into_owned()),
            _ => None,
        })
        .collect();
    actual == expected
}

/// Streams the single verified DLL member out of a retained `.tar.bz2` archive
/// into `output`. Never extracts any other member.
fn extract_dll_from_archive(
    archive_path: &Path,
    archive_name: &str,
    output: &Path,
) -> Result<(), String> {
    let file = File::open(archive_path)
        .map_err(|error| format!("failed to open archive {}: {error}", archive_path.display()))?;
    let decoder = BzDecoder::new(file);
    let mut archive = Archive::new(decoder);
    let entries = archive
        .entries()
        .map_err(|error| format!("failed to read archive entries: {error}"))?;

    for entry in entries {
        let mut entry = entry.map_err(|error| format!("failed to read archive entry: {error}"))?;
        let entry_path = entry
            .path()
            .map_err(|error| format!("failed to read archive entry path: {error}"))?
            .into_owned();
        if !archive_member_matches(&entry_path, archive_name) {
            continue;
        }

        if let Some(parent) = output.parent() {
            fs::create_dir_all(parent)
                .map_err(|error| format!("failed to create {}: {error}", parent.display()))?;
        }
        let mut extracted = File::create(output)
            .map_err(|error| format!("failed to create {}: {error}", output.display()))?;
        io::copy(&mut entry, &mut extracted)
            .map_err(|error| format!("failed to extract {}: {error}", entry_path.display()))?;
        return Ok(());
    }

    Err(format!(
        "archive {} does not contain {}",
        archive_path.display(),
        Path::new(archive_name).join("lib").join(DLL_NAME).display()
    ))
}

/// Resolves the `onnxruntime.dll` source to stage.
///
/// Prefers the cargo profile-dir copy (populated by sherpa-onnx-sys's
/// `copy_windows_runtime_dlls` before this script runs). When that copy is
/// missing or fails verification — e.g. a rust-cache restore replays build
/// artifacts without re-running sherpa-onnx-sys's DLL copy — falls back to a
/// verified copy inside the sherpa-onnx prebuilt cache
/// (`target/sherpa-onnx-prebuilt/<archive>/lib/onnxruntime.dll`), honoring the
/// `SHERPA_ONNX_PREBUILT` override. If the extracted copy is also absent,
/// recovers the DLL from the retained `<archive>.tar.bz2` (honoring
/// `SHERPA_ONNX_ARCHIVE_DIR`), verifying it before use. Fails closed with a
/// searched-paths diagnostic when no verified source exists.
fn resolve_runtime_source(profile_dir: &Path) -> PathBuf {
    let profile_copy = profile_dir.join(DLL_NAME);
    if verify_file(&profile_copy, DLL_NAME, DLL_SIZE, DLL_SHA256).is_ok() {
        return profile_copy;
    }

    let prebuilt_root = match env::var("SHERPA_ONNX_PREBUILT") {
        Ok(override_dir) if !override_dir.is_empty() => {
            let root = Path::new(&override_dir).to_path_buf();
            if !root.is_dir() {
                panic!(
                    "SHERPA_ONNX_PREBUILT={} is not a directory; cannot resolve a verified ONNX Runtime source",
                    root.display()
                );
            }
            root
        }
        _ => match build_env::find_prebuilt_dir(profile_dir) {
            Some(root) => root,
            None => {
                panic!(
                    "No ONNX Runtime source found: profile-dir copy {} is missing or invalid, and no \
                     sherpa-onnx prebuilt cache was found. Searched ancestors: {}",
                    profile_copy.display(),
                    build_env::describe_prebuilt_search(profile_dir)
                );
            }
        },
    };

    let mut searched = Vec::new();
    searched.push(profile_copy.display().to_string());

    let mut archives: Vec<String> = Vec::new();
    if let Ok(entries) = fs::read_dir(&prebuilt_root) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().into_owned();
            if entry.path().is_dir() {
                archives.push(name);
            }
        }
    }
    let archives = build_env::prefer_shared_archives(&archives);
    let candidates = build_env::prebuilt_dll_candidates(&prebuilt_root, &archives, DLL_NAME);

    for candidate in &candidates {
        searched.push(candidate.display().to_string());
        if verify_file(candidate, DLL_NAME, DLL_SIZE, DLL_SHA256).is_ok() {
            println!(
                "cargo:warning=Staged ONNX Runtime from prebuilt cache: {}",
                candidate.display()
            );
            return candidate.clone();
        }
    }

    let archive_root = match env::var("SHERPA_ONNX_ARCHIVE_DIR") {
        Ok(override_dir) if !override_dir.is_empty() => {
            let root = Path::new(&override_dir).to_path_buf();
            if !root.is_dir() {
                panic!(
                    "SHERPA_ONNX_ARCHIVE_DIR={} is not a directory; cannot recover ONNX Runtime from archive",
                    root.display()
                );
            }
            root
        }
        _ => prebuilt_root.clone(),
    };
    let archive_candidates = build_env::archive_tarball_candidates(&archive_root, &archives);
    let extracted = profile_dir.join(format!(".onnxruntime-archive-{}.dll", process::id()));
    for (archive_path, archive_name) in archive_candidates.iter().zip(archives.iter()) {
        searched.push(archive_path.display().to_string());
        if !archive_path.is_file() {
            continue;
        }
        match extract_dll_from_archive(archive_path, archive_name, &extracted) {
            Ok(()) => {
                if verify_file(&extracted, DLL_NAME, DLL_SIZE, DLL_SHA256).is_ok() {
                    println!(
                        "cargo:warning=Staged ONNX Runtime from archive: {}",
                        extracted.display()
                    );
                    return extracted;
                }
                searched.push(format!(
                    "{} (extracted DLL failed verification)",
                    extracted.display()
                ));
            }
            Err(error) => {
                println!(
                    "cargo:warning=Skipping unusable ONNX Runtime archive {}: {error}",
                    archive_path.display()
                );
            }
        }
    }

    panic!(
        "Failed to stage bundled ONNX Runtime: no verified onnxruntime.dll source found. \
         Searched: {}. If SHERPA_ONNX_PREBUILT is set, point it at a sherpa-onnx prebuilt \
         cache root containing a valid onnxruntime.dll.",
        searched.join(", ")
    );
}

/// Copies the single prebuilt `onnxruntime.dll` from the cargo profile dir
/// into the temporary staging directory and verifies it before the atomic
/// rename. Never copies `onnxruntime_providers_shared.dll` or any other file.
fn stage_runtime(source: &Path, destination: &Path) -> Result<(), String> {
    let source_metadata = fs::symlink_metadata(source).map_err(|error| {
        format!(
            "failed to inspect source {} (sherpa-onnx-sys must stage it before this build script runs): {error}",
            source.display()
        )
    })?;
    if is_link_or_reparse_point(&source_metadata) {
        return Err(format!(
            "source {} is a link or reparse point; refusing to follow it",
            source.display()
        ));
    }
    if !source_metadata.is_file() {
        return Err(format!("source {} is not a regular file", source.display()));
    }

    fs::create_dir_all(destination)
        .map_err(|error| format!("failed to create {}: {error}", destination.display()))?;

    let output = destination.join(DLL_NAME);
    fs::copy(source, &output).map_err(|error| {
        format!(
            "failed to copy {} to {}: {error}",
            source.display(),
            output.display()
        )
    })?;
    verify_file(&output, DLL_NAME, DLL_SIZE, DLL_SHA256)?;

    Ok(())
}

fn is_link_or_reparse_point(metadata: &fs::Metadata) -> bool {
    if metadata.file_type().is_symlink() {
        return true;
    }

    #[cfg(windows)]
    {
        return metadata.file_attributes() & 0x0000_0400 != 0;
    }

    #[cfg(not(windows))]
    false
}

fn verify_staged_runtime(destination: &Path) -> Result<(), String> {
    let metadata = fs::symlink_metadata(destination)
        .map_err(|error| format!("failed to inspect runtime stage: {error}"))?;
    if is_link_or_reparse_point(&metadata) {
        return Err("runtime stage is a link or reparse point".to_string());
    }
    if !metadata.file_type().is_dir() {
        return Err("runtime stage is not a directory".to_string());
    }

    let entries = fs::read_dir(destination)
        .map_err(|error| format!("failed to enumerate runtime stage: {error}"))?;
    for entry in entries {
        let entry =
            entry.map_err(|error| format!("failed to enumerate runtime stage entry: {error}"))?;
        let entry_metadata = fs::symlink_metadata(entry.path()).map_err(|error| {
            format!(
                "failed to inspect runtime stage entry {}: {error}",
                entry.path().display()
            )
        })?;
        if is_link_or_reparse_point(&entry_metadata) {
            return Err(format!(
                "runtime stage entry {} is a link or reparse point",
                entry.path().display()
            ));
        }
        if !entry_metadata.file_type().is_file() {
            return Err(format!(
                "runtime stage entry {} is not a regular file",
                entry.path().display()
            ));
        }

        // Exactly one staged file is allowed: onnxruntime.dll. Anything else
        // (e.g. a stray onnxruntime_providers_shared.dll) invalidates the stage.
        if entry.file_name().as_os_str() != DLL_NAME {
            return Err(format!(
                "runtime stage contains undeclared entry {}",
                entry.path().display()
            ));
        }
    }

    verify_file(&destination.join(DLL_NAME), DLL_NAME, DLL_SIZE, DLL_SHA256)
}

fn verify_file(
    path: &Path,
    name: &str,
    expected_size: u64,
    expected_sha256: &str,
) -> Result<(), String> {
    let metadata =
        fs::metadata(path).map_err(|error| format!("failed to inspect {name}: {error}"))?;
    if metadata.len() != expected_size {
        return Err(format!(
            "{name} has size {}, expected {expected_size}",
            metadata.len()
        ));
    }

    let mut file = File::open(path).map_err(|error| format!("failed to open {name}: {error}"))?;
    let mut hasher = Sha256::new();
    let mut buffer = [0_u8; 64 * 1024];
    loop {
        let read = file
            .read(&mut buffer)
            .map_err(|error| format!("failed to hash {name}: {error}"))?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }

    let actual_sha256 = format!("{:x}", hasher.finalize());
    if actual_sha256 != expected_sha256 {
        return Err(format!(
            "{name} has SHA-256 {actual_sha256}, expected {expected_sha256}"
        ));
    }

    Ok(())
}