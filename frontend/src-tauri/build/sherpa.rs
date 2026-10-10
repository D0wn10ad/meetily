//! Stages the sherpa-onnx SHARED runtime DLL on Windows.
//!
//! sherpa-onnx's prebuilt Windows STATIC libraries embed the /MT CRT, which
//! collides with the /MD CRT used by the Rust app (and whisper), producing
//! LNK2038/LNK2005/LNK1169 link errors. On Windows we therefore build against
//! the `shared` feature and stage ONLY `sherpa-onnx-c-api.dll` next to the app.
//! It imports just `OrtGetApiBase@ordinal1` from onnxruntime, satisfied by the
//! app's own sha-verified onnxruntime.dll — so no onnxruntime DLL is copied here.

use std::ffi::OsStr;
use std::path::Path;

/// Copies `sherpa-onnx-c-api.dll` from the cargo profile dir into
/// `binaries/sherpa/` so Tauri bundles it as a resource. Windows-only;
/// a no-op on every other target OS.
pub fn stage_sherpa_dlls() {
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        return;
    }

    // OUT_DIR looks like target/<triple>/<profile>/build/<crate>/out (with
    // --target) or target/<profile>/build/<crate>/out (without); walking
    // ancestors finds the profile dir in both layouts.
    let out_dir = std::env::var("OUT_DIR").expect("OUT_DIR not set by cargo");
    let profile = std::env::var("PROFILE").unwrap_or_default();
    let profile_dir = Path::new(&out_dir)
        .ancestors()
        .find(|p| p.file_name() == Some(OsStr::new(&profile)))
        .expect("could not resolve cargo profile dir from OUT_DIR");

    let src = profile_dir.join("sherpa-onnx-c-api.dll");
    let dst_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("binaries/sherpa");
    std::fs::create_dir_all(&dst_dir)
        .expect("failed to create binaries/sherpa staging directory");
    let dst = dst_dir.join("sherpa-onnx-c-api.dll");

    // Skip identical copies: under `tauri dev` the DLL may be loaded and
    // locked by a running app, so a redundant copy would fail for no reason.
    if dst.exists() {
        if let (Ok(src_meta), Ok(dst_meta)) = (std::fs::metadata(&src), std::fs::metadata(&dst))
        {
            if src_meta.len() == dst_meta.len() {
                return;
            }
        }
    }

    // Stage ONLY sherpa-onnx-c-api.dll. Never glob *.dll: sherpa's build dir
    // also contains its own onnxruntime.dll / onnxruntime_providers_shared.dll,
    // which must NOT be staged (the app ships a single verified onnxruntime.dll).
    if let Err(err) = std::fs::copy(&src, &dst) {
        if dst.exists() {
            println!(
                "cargo:warning=failed to update staged sherpa-onnx-c-api.dll ({}); keeping existing copy at {}",
                err,
                dst.display()
            );
        } else if profile == "debug" {
            println!(
                "cargo:warning=debug build: no sherpa-onnx-c-api.dll at {} and no staged copy at {}; skipping sherpa staging",
                src.display(),
                dst.display()
            );
        } else {
            panic!(
                "failed to stage sherpa-onnx-c-api.dll: source {} not found ({}). \
                 The sherpa-onnx `shared` feature appears to be inactive on Windows \
                 (expected it in Cargo.toml target cfg for windows).",
                src.display(),
                err
            );
        }
    }
}
