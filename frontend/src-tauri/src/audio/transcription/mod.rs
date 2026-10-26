// audio/transcription/mod.rs
//
// Transcription module: provider abstraction, engine lifecycle, and the
// transcription worker.
//
// Purpose
// -------
// This module defines the `TranscriptionProvider` trait — the single seam every
// engine implementation (Whisper, Parakeet, FunASR, Sherpa-ONNX) plugs into —
// plus the `TranscriptionEngine` that owns the selected provider behind
// `Arc<dyn TranscriptionProvider>`, and the worker task that polls recorded
// audio and emits transcript updates to the frontend.
//
// Lifecycle
// ---------
// `model load → engine init → worker poll → transcript emit`:
// 1. The caller validates/downloads the model files.
// 2. `get_or_init_transcription_engine` / `get_or_init_whisper` lazily build
//    the engine (process-global caches); they are idempotent and MUST be
//    awaited by the caller before the engine is used.
// 3. `start_transcription_task` spawns the worker, which polls audio chunks,
//    feeds them to the provider, and emits `TranscriptUpdate` events.
//
// Thread safety
// --------------
// Providers are always used behind `Arc<dyn TranscriptionProvider>`; the init
// functions take `&AppHandle` and are `async`. Engine caches are process-global
// and internally guarded, so concurrent callers share a single initialization.
//
// Visibility
// -----------
// Submodules are `pub(crate)`: in-crate consumers go through the re-exports
// below (the trait and its supporting types), never through a concrete
// provider type.

pub(crate) mod engine;
pub(crate) mod funasr_provider;
pub(crate) mod parakeet_provider;
pub(crate) mod provider;
pub(crate) mod sherpa_onnx_provider;
pub(crate) mod whisper_provider;
pub(crate) mod worker;

// Engine lifecycle + engine type (crate-internal API). Some items have no
// re-export consumer yet (worker.rs and the whisper/parakeet engines reach
// them via direct paths; T19/T23–T26 consume them next) — the re-export is
// the module's declared internal surface, so unused_imports is intentional.
#[allow(unused_imports)]
pub(crate) use engine::{
    get_or_init_transcription_engine, get_or_init_whisper, validate_transcription_model_ready,
    TranscriptionEngine,
};
// Concrete providers are NOT part of the public surface: crate consumers that
// genuinely need a concrete type use these `pub(crate)` aliases; everything
// else depends on the `TranscriptionProvider` trait re-export below.
// (`ParakeetProvider` / `WhisperProvider` have no concrete-type consumers and
// are no longer re-exported at all.)
pub(crate) use funasr_provider::FunasrProvider;
pub(crate) use provider::TranscriptionProvider;
pub(crate) use sherpa_onnx_provider::SherpaOnnxProvider;
// Kept `pub` on purpose: the backward-compat re-export chain
// `recording_commands::TranscriptUpdate` (pub use) → `audio::TranscriptUpdate`
// (pub use, owned by T22 and out of this todo's scope) requires this item to
// stay publicly visible; narrowing it would break the build (E0364).
pub use worker::{reset_speech_detected_flag, start_transcription_task, TranscriptUpdate};
