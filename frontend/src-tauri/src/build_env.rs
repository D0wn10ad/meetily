// Build-script helpers shared with the library crate.
//
// Build scripts cannot host `#[cfg(test)]` tests that `cargo test -p meetily`
// runs, so the pure logic lives here and is pulled into a build script with
// `include!("../src/build_env.rs");`. Keep this file dependency-free (stdlib
// only) and free of inner attributes so the include works both ways.
//
// Sections:
//   - `build/sherpa.rs` (T3): cargo profile-dir resolution + freshness checks.
//   - `build/onnxruntime.rs` (T4): download retry/backoff schedule.

/// Number of download attempts (the initial attempt plus retries).
pub const DOWNLOAD_ATTEMPTS: usize = 3;

/// Backoff delay, in seconds, used between download attempts.
///
/// Exponential: 1s, 2s, 4s. Only the first `DOWNLOAD_ATTEMPTS - 1` entries are
/// slept through by [`retry_with_backoff`]; the last entry documents the next
/// delay if the attempt budget is ever raised.
pub fn download_backoff_seconds() -> [u64; DOWNLOAD_ATTEMPTS] {
    let mut schedule = [0_u64; DOWNLOAD_ATTEMPTS];
    for (attempt, delay) in schedule.iter_mut().enumerate() {
        *delay = 1_u64 << attempt;
    }
    schedule
}

/// Runs `operation` up to [`DOWNLOAD_ATTEMPTS`] times, sleeping between
/// attempts according to [`download_backoff_seconds`].
///
/// `operation` receives the 1-based attempt number so callers can log progress
/// (and tests can inject a transient failure). `sleep` receives the delay in
/// seconds; production callers pass `std::thread::sleep`, tests pass a plain
/// recorder so the helper stays hermetic.
pub fn retry_with_backoff<F, S, T, E>(mut operation: F, mut sleep: S) -> Result<T, E>
where
    F: FnMut(usize) -> Result<T, E>,
    S: FnMut(u64),
{
    let schedule = download_backoff_seconds();
    let mut last_error: Option<E> = None;

    for attempt in 1..=DOWNLOAD_ATTEMPTS {
        match operation(attempt) {
            Ok(value) => return Ok(value),
            Err(error) => {
                last_error = Some(error);
                if attempt < DOWNLOAD_ATTEMPTS {
                    sleep(schedule[attempt - 1]);
                }
            }
        }
    }

    Err(last_error.expect("Retry loop must attempt at least once"))
}

// --- T3: cargo profile-dir resolution + prebuilt staging helpers -------

use std::ffi::OsStr;
use std::fs::Metadata;
use std::path::{Path, PathBuf};

#[allow(dead_code)]
pub fn expect_profile(profile: Result<String, std::env::VarError>) -> String {
    profile.expect(
        "PROFILE env var not set by cargo; cannot resolve the cargo target profile directory",
    )
}

#[allow(dead_code)]
pub fn resolve_profile_dir(out_dir: &Path, profile: &str) -> PathBuf {
    out_dir
        .ancestors()
        .find(|ancestor| ancestor.file_name() == Some(OsStr::new(profile)))
        .map(Path::to_path_buf)
        .unwrap_or_else(|| {
            panic!(
                "could not resolve the cargo profile directory: no ancestor of OUT_DIR={} is \
                 named after PROFILE={}",
                out_dir.display(),
                profile
            )
        })
}

#[allow(dead_code)]
pub fn find_prebuilt_dir(profile_dir: &Path) -> Option<PathBuf> {
    for ancestor in profile_dir.ancestors() {
        let candidate = ancestor.join("sherpa-onnx-prebuilt");
        match std::fs::metadata(&candidate) {
            Ok(metadata) if metadata.is_dir() => return Some(candidate),
            Ok(_) => continue,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
            Err(error) => panic!(
                "Failed to inspect sherpa-onnx prebuilt cache candidate {}: {}",
                candidate.display(),
                error
            ),
        }
    }
    None
}

#[allow(dead_code)]
pub fn describe_prebuilt_search(profile_dir: &Path) -> String {
    profile_dir
        .ancestors()
        .map(|ancestor| ancestor.display().to_string())
        .collect::<Vec<_>>()
        .join(", ")
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StagedFileState {
    Fresh,
    NeedsCopy,
}

#[allow(dead_code)]
pub fn staged_file_state(src: &Metadata, dst: &Metadata) -> StagedFileState {
    if src.len() != dst.len() {
        return StagedFileState::NeedsCopy;
    }
    match (src.modified(), dst.modified()) {
        (Ok(src_mtime), Ok(dst_mtime)) if src_mtime == dst_mtime => StagedFileState::Fresh,
        _ => StagedFileState::NeedsCopy,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn backoff_schedule_is_one_two_four_seconds() {
        assert_eq!(
            download_backoff_seconds(),
            [1, 2, 4],
            "download retries must back off exponentially: 1s, 2s, 4s"
        );
        assert_eq!(DOWNLOAD_ATTEMPTS, 3, "exactly 3 download attempts");
    }

    #[test]
    fn retry_helper_succeeds_on_third_attempt() {
        let mut attempts = 0_usize;
        let mut delays = Vec::new();

        let result: Result<&'static str, &'static str> = retry_with_backoff(
            |attempt| {
                attempts = attempt;
                if attempt < DOWNLOAD_ATTEMPTS {
                    Err("transient transport error")
                } else {
                    Ok("downloaded")
                }
            },
            |seconds| delays.push(seconds),
        );

        assert_eq!(result, Ok("downloaded"));
        assert_eq!(attempts, DOWNLOAD_ATTEMPTS);
        assert_eq!(delays, vec![1, 2], "backoff before retries must be 1s, 2s");
    }

    #[test]
    fn retry_helper_returns_last_error_after_all_attempts() {
        let mut attempts = 0_usize;
        let mut delays = Vec::new();

        let result: Result<(), &'static str> = retry_with_backoff(
            |attempt| {
                attempts = attempt;
                Err("still failing")
            },
            |seconds| delays.push(seconds),
        );

        assert_eq!(result, Err("still failing"));
        assert_eq!(attempts, DOWNLOAD_ATTEMPTS);
        assert_eq!(delays, vec![1, 2]);
    }

    // --- T3: sherpa prebuilt DLL staging helpers -------------------------

    use std::time::{Duration, SystemTime};

    /// Self-cleaning temp directory so a failed assertion never leaves files
    /// behind (and two concurrent test runs never share state).
    struct TempDir(PathBuf);

    impl TempDir {
        fn new(label: &str) -> Self {
            let unique = SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .expect("system clock before UNIX_EPOCH")
                .as_nanos();
            let dir = std::env::temp_dir().join(format!(
                "meetily-build-env-{}-{}-{}",
                std::process::id(),
                label,
                unique
            ));
            let _ = std::fs::remove_dir_all(&dir);
            std::fs::create_dir_all(&dir).expect("create temp dir");
            Self(dir)
        }

        fn path(&self) -> &Path {
            &self.0
        }
    }

    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    fn set_mtime(path: &Path, seconds_since_epoch: u64) {
        let file = std::fs::OpenOptions::new()
            .write(true)
            .open(path)
            .expect("open file to set mtime");
        file.set_modified(SystemTime::UNIX_EPOCH + Duration::from_secs(seconds_since_epoch))
            .expect("set file mtime");
    }

    #[test]
    fn same_size_different_content_is_stale() {
        let dir = TempDir::new("stale");
        let src = dir.path().join("sherpa-onnx-c-api.dll");
        let dst = dir.path().join("staged-sherpa-onnx-c-api.dll");
        // Same length, different bytes: exactly the case a size-only check
        // silently treats as fresh and therefore ships a stale DLL.
        std::fs::write(&src, b"AAAA").expect("write source");
        std::fs::write(&dst, b"BBBB").expect("write destination");
        // Distinct mtimes model a rebuilt source that has not been staged yet.
        set_mtime(&src, 2_000_000_000);
        set_mtime(&dst, 1_000_000_000);

        let src_meta = std::fs::metadata(&src).expect("source metadata");
        let dst_meta = std::fs::metadata(&dst).expect("destination metadata");
        assert_eq!(
            src_meta.len(),
            dst_meta.len(),
            "test precondition: both files must have the same size"
        );
        assert_eq!(
            staged_file_state(&src_meta, &dst_meta),
            StagedFileState::NeedsCopy,
            "same-size/different-content prebuilt DLL must be re-copied"
        );
    }

    #[test]
    fn identical_size_and_mtime_is_fresh() {
        let dir = TempDir::new("fresh");
        let src = dir.path().join("sherpa-onnx-c-api.dll");
        let dst = dir.path().join("staged-sherpa-onnx-c-api.dll");
        std::fs::write(&src, b"same content").expect("write source");
        std::fs::write(&dst, b"same content").expect("write destination");
        set_mtime(&src, 1_700_000_000);
        set_mtime(&dst, 1_700_000_000);

        let src_meta = std::fs::metadata(&src).expect("source metadata");
        let dst_meta = std::fs::metadata(&dst).expect("destination metadata");
        assert_eq!(
            staged_file_state(&src_meta, &dst_meta),
            StagedFileState::Fresh,
            "an unchanged staged copy must be left alone (DLL may be locked)"
        );
    }

    #[test]
    fn ancestor_search_finds_prebuilt_dir_for_target_debug_layout() {
        let root = TempDir::new("ancestors-target-debug");
        let profile_dir = root.path().join("target").join("debug");
        std::fs::create_dir_all(&profile_dir).expect("create profile dir");
        let prebuilt = root.path().join("target").join("sherpa-onnx-prebuilt");
        std::fs::create_dir_all(&prebuilt).expect("create prebuilt dir");

        assert_eq!(
            find_prebuilt_dir(&profile_dir),
            Some(prebuilt),
            "ancestor search must find <target>/sherpa-onnx-prebuilt from <target>/debug"
        );
    }

    #[test]
    fn ancestor_search_finds_prebuilt_dir_for_triple_profile_layout() {
        let root = TempDir::new("ancestors-triple");
        let profile_dir = root
            .path()
            .join("target")
            .join("x86_64-pc-windows-msvc")
            .join("release");
        std::fs::create_dir_all(&profile_dir).expect("create profile dir");
        let prebuilt = root.path().join("target").join("sherpa-onnx-prebuilt");
        std::fs::create_dir_all(&prebuilt).expect("create prebuilt dir");

        assert_eq!(
            find_prebuilt_dir(&profile_dir),
            Some(prebuilt),
            "ancestor search must walk past <target>/<triple> to <target>"
        );
    }

    #[test]
    fn resolve_profile_dir_finds_profile_ancestor_of_out_dir() {
        let out_dir = Path::new("/tmp/custom-target/debug/build/sherpa-onnx-sys-abc/out");
        assert_eq!(
            resolve_profile_dir(out_dir, "debug"),
            PathBuf::from("/tmp/custom-target/debug")
        );
    }

    #[test]
    fn missing_profile_env_panics_with_message_naming_profile() {
        let result = std::panic::catch_unwind(|| {
            expect_profile(Err(std::env::VarError::NotPresent))
        });
        let payload = result.expect_err("a missing PROFILE env var must panic");
        let message = payload
            .downcast_ref::<String>()
            .cloned()
            .or_else(|| payload.downcast_ref::<&str>().map(|text| text.to_string()))
            .unwrap_or_default();
        assert!(
            message.contains("PROFILE"),
            "panic message must name PROFILE, got: {message}"
        );
        println!("captured panic message: {message}");
    }
}
