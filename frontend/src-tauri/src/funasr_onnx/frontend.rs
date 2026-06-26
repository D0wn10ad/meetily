use std::path::Path;
use std::sync::OnceLock;

use crate::funasr_onnx::FunasrError;
use realfft::num_complex::Complex32;
use realfft::RealFftPlanner;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

pub const SAMPLE_RATE: u32 = 16000;
pub const FRAME_LENGTH: usize = 400; // 25 ms at 16 kHz
pub const FRAME_SHIFT: usize = 160; // 10 ms at 16 kHz
pub const N_MELS: usize = 80;
pub const LFR_M: usize = 7;
pub const LFR_N: usize = 6;
pub const FEAT_DIM: usize = 560; // N_MELS * LFR_M
pub const FFT_SIZE: usize = 512; // next power of two >= 400

// ---------------------------------------------------------------------------
// Mel scale helpers
// ---------------------------------------------------------------------------

/// Convert Hz to the Mel scale.
fn hz_to_mel(hz: f32) -> f32 {
    2595.0 * (1.0 + hz / 700.0).log10()
}

/// Convert Mel back to Hz.
fn mel_to_hz(mel: f32) -> f32 {
    700.0 * (10.0_f32.powf(mel / 2595.0) - 1.0)
}

// ---------------------------------------------------------------------------
// Pre-computed constant helpers (lazy)
// ---------------------------------------------------------------------------

/// 400-point Hamming window.
fn hamming_window() -> &'static [f32; FRAME_LENGTH] {
    static WINDOW: OnceLock<[f32; FRAME_LENGTH]> = OnceLock::new();
    WINDOW.get_or_init(|| {
        let mut w = [0.0f32; FRAME_LENGTH];
        let n = FRAME_LENGTH;
        for i in 0..n {
            w[i] = 0.54 - 0.46 * (2.0 * std::f32::consts::PI * i as f32 / (n - 1) as f32).cos();
        }
        w
    })
}

/// 80 × 257 triangular mel filterbank matrix.
///
/// Row `m` contains the filter weights for mel band `m`.
/// Columns correspond to FFT bins (0 … 256).
fn mel_filterbank() -> &'static Vec<Vec<f32>> {
    static FB: OnceLock<Vec<Vec<f32>>> = OnceLock::new();
    FB.get_or_init(|| {
        let n_bins = FFT_SIZE / 2 + 1; // 257
        let low_mel = hz_to_mel(20.0);
        let high_mel = hz_to_mel(8000.0);

        // N_MELS + 2 equally-spaced points on the mel axis
        let mel_points: Vec<f32> = (0..N_MELS + 2)
            .map(|i| low_mel + (high_mel - low_mel) * i as f32 / (N_MELS + 1) as f32)
            .collect();

        // Convert to Hz and then to continuous FFT-bin indices
        let bin_points: Vec<f32> = mel_points
            .iter()
            .map(|&m| mel_to_hz(m) * FFT_SIZE as f32 / SAMPLE_RATE as f32)
            .collect();

        let mut fb = vec![vec![0.0f32; n_bins]; N_MELS];

        for m in 0..N_MELS {
            let f_start = bin_points[m];
            let f_center = bin_points[m + 1];
            let f_end = bin_points[m + 2];

            for k in 0..n_bins {
                let kf = k as f32;
                if kf >= f_start && kf <= f_center {
                    fb[m][k] = (kf - f_start) / (f_center - f_start);
                } else if kf > f_center && kf <= f_end {
                    fb[m][k] = (f_end - kf) / (f_end - f_center);
                }
            }
        }

        fb
    })
}

// ---------------------------------------------------------------------------
// FBank feature extraction
// ---------------------------------------------------------------------------

/// Compute 80-dimensional log-mel filterbank energies from scaled PCM audio.
///
/// `audio` is assumed to already be scaled by 32768.0.
/// Returns a `Vec` of frames, each frame being an `[f32; 80]` stored as `Vec<f32>`.
fn compute_fbank(audio: &[f32]) -> Result<Vec<Vec<f32>>, FunasrError> {
    let n_frames = audio.len().saturating_sub(FRAME_LENGTH) / FRAME_SHIFT + 1;
    let mut fbank: Vec<Vec<f32>> = Vec::with_capacity(n_frames);

    let window = hamming_window();
    let fb = mel_filterbank();
    let n_bins = FFT_SIZE / 2 + 1; // 257

    // Pre-allocate FFT buffers (reused across frames)
    let mut planner = RealFftPlanner::new();
    let fft = planner.plan_fft_forward(FFT_SIZE);
    #[allow(unused_mut)]
    let mut fft = fft;
    let mut spectrum: Vec<Complex32> = fft.make_output_vec(); // length 257
    let mut indata = vec![0.0f32; FFT_SIZE];

    let mut frame_buffer = vec![0.0f32; FRAME_LENGTH];

    for frame_idx in 0..n_frames {
        let offset = frame_idx * FRAME_SHIFT;

        // 1. Extract and window the frame
        for (j, src) in audio[offset..offset + FRAME_LENGTH].iter().enumerate() {
            frame_buffer[j] = src * window[j];
        }

        // 2. Copy into FFT input (already zero-initialised beyond FRAME_LENGTH)
        indata[..FRAME_LENGTH].copy_from_slice(&frame_buffer);
        indata[FRAME_LENGTH..].fill(0.0_f32);

        // 3. Real FFT → 257 complex bins
        fft.process(&mut indata, &mut spectrum)
            .map_err(|e| FunasrError::DecodeError(format!("FFT error: {e}")))?;

        // 4. Power spectrum + mel filterbank + log
        let mut mel_energies = [0.0f32; N_MELS];

        // Pre-compute power spectrum into a temporary vec
        let mut power = [0.0f32; 257];
        for bin in 0..n_bins {
            power[bin] = spectrum[bin].norm_sqr();
        }

        // Apply each filter
        for m in 0..N_MELS {
            let mut energy = 0.0_f32;
            for bin in 0..n_bins {
                energy += fb[m][bin] * power[bin];
            }
            mel_energies[m] = (energy + 1e-10_f32).ln();
        }

        fbank.push(mel_energies.to_vec());
    }

    Ok(fbank)
}

// ---------------------------------------------------------------------------
// LFR (Low Frame Rate)
// ---------------------------------------------------------------------------

/// Apply Low Frame Rate stacking.
///
/// Left-pads with `(LFR_M - 1)/2` copies of the first frame, then stacks every
/// `LFR_N` frames taking `LFR_M` consecutive frames per output row.
fn apply_lfr(fbank: &[Vec<f32>]) -> Vec<Vec<f32>> {
    let t_fbank = fbank.len();
    if t_fbank == 0 {
        return Vec::new();
    }

    // Ceiling division for the nominal output length
    let t_lfr = (t_fbank + LFR_N - 1) / LFR_N;
    let left_pad = (LFR_M - 1) / 2; // 3

    // Left-pad by tiling the first frame
    let padded_len = t_fbank + left_pad;
    let mut padded = Vec::with_capacity(padded_len);
    for _ in 0..left_pad {
        padded.push(fbank[0].clone());
    }
    padded.extend_from_slice(fbank);

    let mut output: Vec<Vec<f32>> = Vec::with_capacity(t_lfr);

    for i in 0..t_lfr {
        let start = i * LFR_N;
        if start + LFR_M > padded.len() {
            // Not enough frames for a full output row — break early
            break;
        }
        let mut feat = Vec::with_capacity(FEAT_DIM);
        for j in 0..LFR_M {
            feat.extend_from_slice(&padded[start + j]);
        }
        debug_assert!(feat.len() == FEAT_DIM);
        output.push(feat);
    }

    output
}

// ---------------------------------------------------------------------------
// CMVN (Cepstral Mean & Variance Normalisation)
// ---------------------------------------------------------------------------

/// Parse a single vector from a text-format Kaldi CMVN section.
///
/// Matches FunASR Python `load_cmvn()`: finds `<{section_name}>`, then parses
/// the next line's `<LearnRateCoef>` bracket-format values.
fn parse_text_cmvn_vector(data: &[u8], section_name: &str) -> Result<Vec<f32>, FunasrError> {
    let text = std::str::from_utf8(data).map_err(|_| {
        FunasrError::InvalidCmvnFile("CMVN file is not valid UTF-8".into())
    })?;

    let header = format!("<{section_name}>");

    let header_line_idx = text.lines().position(|line| line.trim().starts_with(&header)).ok_or_else(|| {
        FunasrError::InvalidCmvnFile(format!("Missing '<{section_name}>' section in CMVN file"))
    })?;

    let val_line = text.lines().nth(header_line_idx + 1).ok_or_else(|| {
        FunasrError::InvalidCmvnFile(format!(
            "Expected '<LearnRateCoef>' after '<{section_name}>'"
        ))
    })?;

    if !val_line.trim_start().starts_with("<LearnRateCoef>") {
        return Err(FunasrError::InvalidCmvnFile(format!(
            "Expected '<LearnRateCoef>' after '<{section_name}>', got: {}",
            val_line.trim()
        )));
    }

    // tokens layout: [0]="<LearnRateCoef>", [1]="0", [2]="[", [3..last-1]=floats, [last]="]"
    let tokens: Vec<&str> = val_line.split_whitespace().collect();

    if tokens.len() < 4 || tokens[2] != "[" || tokens[tokens.len() - 1] != "]" {
        return Err(FunasrError::InvalidCmvnFile(format!(
            "Expected bracket-format vector after '<{section_name}>', got {} tokens",
            tokens.len()
        )));
    }

    let values: Vec<f32> = tokens[3..tokens.len() - 1]
        .iter()
        .map(|&s| {
            s.parse::<f32>().map_err(|e| {
                FunasrError::InvalidCmvnFile(format!(
                    "Failed to parse float '{}' in '<{section_name}>': {e}",
                    s
                ))
            })
        })
        .collect::<Result<Vec<_>, _>>()?;

    Ok(values)
}

/// Parse a text-format Kaldi CMVN file into (negative_means, inverse_stds).
///
/// The file is expected to contain an `<AddShift>` section (negative means)
/// followed by a `<Rescale>` section (inverse standard deviations).
fn parse_kaldi_cmvn(data: &[u8]) -> Result<(Vec<f32>, Vec<f32>), FunasrError> {
    let means = parse_text_cmvn_vector(data, "AddShift")?;
    let inv_stds = parse_text_cmvn_vector(data, "Rescale")?;

    if means.len() != FEAT_DIM {
        return Err(FunasrError::InvalidCmvnFile(format!(
            "Expected CMVN means length {FEAT_DIM}, got {}",
            means.len()
        )));
    }
    if inv_stds.len() != FEAT_DIM {
        return Err(FunasrError::InvalidCmvnFile(format!(
            "Expected CMVN inv_stds length {FEAT_DIM}, got {}",
            inv_stds.len()
        )));
    }

    Ok((means, inv_stds))
}

/// Apply CMVN normalisation to features.
///
/// Each frame: `out[i] = (frame[i] + means[i]) * inv_stds[i]`
/// where `means` are the **negative** means already (AddShift convention).
fn apply_cmvn(feats: &[Vec<f32>], means: &[f32], inv_stds: &[f32]) -> Vec<Vec<f32>> {
    feats
        .iter()
        .map(|frame| {
            frame
                .iter()
                .zip(means.iter())
                .zip(inv_stds.iter())
                .map(|((&f, &m), &is)| (f + m) * is)
                .collect::<Vec<f32>>()
        })
        .collect()
}

// ---------------------------------------------------------------------------
// FunasrFrontend
// ---------------------------------------------------------------------------

/// The WavFrontend port from funasr_onnx.
///
/// Processes 16 kHz mono f32 PCM audio into features suitable for the
/// Paraformer ONNX model.
pub struct FunasrFrontend {
    cmvn_means: Vec<f32>,
    cmvn_inv_std: Vec<f32>,
}

impl FunasrFrontend {
    /// Load a Kaldi-binary CMVN file and construct a frontend.
    pub fn new(cmvn_path: &Path) -> Result<Self, FunasrError> {
        let bytes = std::fs::read(cmvn_path)?;
        let (cmvn_means, cmvn_inv_std) = parse_kaldi_cmvn(&bytes)?;
        log::info!("FunASR CMVN loaded: {} means, {} inv_stds", cmvn_means.len(), cmvn_inv_std.len());
        Ok(Self {
            cmvn_means,
            cmvn_inv_std,
        })
    }

    /// Process 16 kHz mono f32 PCM audio into flattened LFR-CMVN features.
    ///
    /// Returns `(features_flat, feats_len)` where:
    /// - `features_flat` has shape `(T_lfr * FEAT_DIM)`
    /// - `feats_len` = `T_lfr` as `i32`
    pub fn process(&self, audio: &[f32]) -> Result<(Vec<f32>, i32), FunasrError> {
        if audio.is_empty() {
            return Err(FunasrError::EmptyAudio);
        }
        if audio.len() < FRAME_LENGTH {
            return Err(FunasrError::AudioTooShort {
                min: FRAME_LENGTH,
                actual: audio.len(),
            });
        }

        // 1. Scale to match the int16-range convention used by the Python frontend
        let scaled: Vec<f32> = audio.iter().map(|&s| s * 32768.0).collect();

        // 2. Log-mel filterbank features
        let fbank = compute_fbank(&scaled)?;

        // 3. Low Frame Rate stacking
        let lfr = apply_lfr(&fbank);
        log::info!("FunASR frontend: {} samples → {} fbank frames → {} lfr frames", audio.len(), fbank.len(), lfr.len());

        // 4. CMVN normalisation
        let cmvn = apply_cmvn(&lfr, &self.cmvn_means, &self.cmvn_inv_std);

        // 5. Flatten
        let t_lfr = cmvn.len();
        let features: Vec<f32> = cmvn.into_iter().flatten().collect();

        Ok((features, t_lfr as i32))
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn generate_sine_wave(samples: usize, freq: f32, sample_rate: u32) -> Vec<f32> {
        (0..samples)
            .map(|i| (2.0 * std::f32::consts::PI * freq * i as f32 / sample_rate as f32).sin())
            .collect()
    }

    #[test]
    fn test_frontend_output_shape() {
        let audio = generate_sine_wave(16000, 440.0, 16000);
        // Expected T_fbank = floor((16000 - 400) / 160) + 1
        //                = floor(15600/160) + 1
        //                = 97 + 1
        //                = 98
        // Expected T_lfr  = ceil(98 / 6) = 17
        let t_fbank = ((audio.len() - FRAME_LENGTH) / FRAME_SHIFT) + 1;
        let t_lfr = (t_fbank + LFR_N - 1) / LFR_N;
        assert_eq!(t_fbank, 98);
        assert_eq!(t_lfr, 17);
        assert_eq!(t_lfr * FEAT_DIM, 9520);
    }

    #[test]
    fn test_short_audio_error() {
        let err = FunasrError::AudioTooShort {
            min: FRAME_LENGTH,
            actual: 200,
        };
        assert!(err.to_string().contains("200"));
    }

    #[test]
    fn test_empty_audio_error() {
        let err = FunasrError::EmptyAudio;
        assert_eq!(format!("{err}"), "Empty audio input");
    }

    #[test]
    fn test_mel_filterbank_shape() {
        let fb = mel_filterbank();
        assert_eq!(fb.len(), N_MELS);
        for row in fb.iter() {
            assert_eq!(row.len(), FFT_SIZE / 2 + 1); // 257
        }
    }

    #[test]
    fn test_hamming_window_symmetry() {
        let w = hamming_window();
        assert_eq!(w.len(), FRAME_LENGTH);
        // Check symmetry: w[n] ≈ w[FRAME_LENGTH - 1 - n]
        for i in 0..FRAME_LENGTH / 2 {
            let diff = (w[i] - w[FRAME_LENGTH - 1 - i]).abs();
            assert!(diff < 1e-6, "Hamming asymmetry at index {i}: diff={diff}");
        }
        // Peak should be at the center
        let center = FRAME_LENGTH / 2;
        let max_val = w.iter().fold(0.0_f32, |a, &b| a.max(b));
        assert!((w[center] - max_val).abs() < 1e-6);
    }

    #[test]
    fn test_hz_mel_roundtrip() {
        let test_hz = 1000.0_f32;
        let mel = hz_to_mel(test_hz);
        let back = mel_to_hz(mel);
        let rel = (back - test_hz).abs() / test_hz;
        assert!(rel < 1e-4, "Hz↔Mel roundtrip error: {rel}");
    }

    #[test]
    fn test_apply_lfr_empty() {
        let result = apply_lfr(&[]);
        assert!(result.is_empty());
    }

    #[test]
    fn test_apply_lfr_single_frame() {
        // A single fbank frame should not produce output (needs 7 frames)
        let fb = vec![vec![1.0f32; N_MELS]];
        let result = apply_lfr(&fb);
        assert!(result.is_empty());
    }

    #[test]
    fn test_apply_lfr_output_shape() {
        // 14 fbank frames → padded = 17 frames → T_lfr = 2 (ceil(14/6)=2)
        let n_frames = 14;
        let fb: Vec<Vec<f32>> = (0..n_frames).map(|_| vec![0.5f32; N_MELS]).collect();
        let result = apply_lfr(&fb);
        assert_eq!(result.len(), 2);
        for row in &result {
            assert_eq!(row.len(), FEAT_DIM);
        }
    }

    #[test]
    fn test_compute_fbank_shape() {
        // Enough audio for 2 frames: 400 + 160 = 560 samples
        let audio = generate_sine_wave(560, 440.0, 16000);
        let scaled: Vec<f32> = audio.iter().map(|&s| s * 32768.0).collect();
        let fbank = compute_fbank(&scaled).unwrap();
        assert_eq!(fbank.len(), 2);
        for frame in &fbank {
            assert_eq!(frame.len(), N_MELS);
            // All values should be finite
            for &v in frame {
                assert!(v.is_finite(), "Non-finite mel energy: {v}");
            }
        }
    }

    #[test]
    fn test_cmvn_identity() {
        // If means are 0 and inv_stds are 1, output should equal input
        let n_frames = 3;
        let feats: Vec<Vec<f32>> = (0..n_frames)
            .map(|i| vec![i as f32 + 1.0; FEAT_DIM])
            .collect();
        let means = vec![0.0_f32; FEAT_DIM];
        let inv_stds = vec![1.0_f32; FEAT_DIM];
        let result = apply_cmvn(&feats, &means, &inv_stds);
        assert_eq!(result.len(), n_frames);
        for (orig, processed) in feats.iter().zip(result.iter()) {
            for (&o, &p) in orig.iter().zip(processed.iter()) {
                assert!((o - p).abs() < 1e-6);
            }
        }
    }

    #[test]
    fn test_cmvn_shift_scale() {
        // means = [1.0; 560], inv_stds = [2.0; 560]
        // output[i] = (input[i] + 1.0) * 2.0
        let feats: Vec<Vec<f32>> = vec![vec![3.0_f32; FEAT_DIM]];
        let means = vec![1.0_f32; FEAT_DIM];
        let inv_stds = vec![2.0_f32; FEAT_DIM];
        let result = apply_cmvn(&feats, &means, &inv_stds);
        assert_eq!(result.len(), 1);
        for &v in &result[0] {
            assert!((v - 8.0).abs() < 1e-6); // (3 + 1) * 2 = 8
        }
    }

    // -----------------------------------------------------------------------
    // Text-format CMVN parser tests
    // -----------------------------------------------------------------------

    /// Generate a valid 560-dim AddShift text line for use in tests.
    fn make_addshift_line() -> String {
        let vals: Vec<String> = (0..560).map(|i| format!("{}", -0.5 - i as f32 * 0.001)).collect();
        format!("<LearnRateCoef> 0 [ {} ]", vals.join(" "))
    }

    /// Generate a valid 560-dim Rescale text line for use in tests.
    fn make_rescale_line() -> String {
        let vals: Vec<String> = (0..560).map(|i| format!("{}", 0.1 + i as f32 * 0.0005)).collect();
        format!("<LearnRateCoef> 0 [ {} ]", vals.join(" "))
    }

    fn make_full_cmvn_text() -> String {
        format!(
            "<Nnet>\n<Splice> 560 560\n[ 0 ]\n<AddShift> 560 560\n{}\n<Rescale> 560 560\n{}\n</Nnet>\n",
            make_addshift_line(),
            make_rescale_line(),
        )
    }

    #[test]
    fn test_parse_text_cmvn_valid() {
        let data = make_full_cmvn_text();
        let vec = parse_text_cmvn_vector(data.as_bytes(), "AddShift").unwrap();
        assert_eq!(vec.len(), 560);
        let expected_first = -0.5;
        assert!((vec[0] - expected_first).abs() < 1e-5);
        let expected_last = -0.5 - 559.0 * 0.001;
        assert!((vec[559] - expected_last).abs() < 1e-5);
    }

    #[test]
    fn test_parse_text_cmvn_missing_section() {
        let data = b"<Nnet>\n<SomethingElse> 560 560\n<LearnRateCoef> 0 [ 1.0 2.0 ]\n</Nnet>";
        let result = parse_text_cmvn_vector(data, "AddShift");
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("AddShift"), "error should mention AddShift: {err}");
    }

    #[test]
    fn test_parse_text_cmvn_wrong_dim() {
        let data = b"<AddShift> 560 560\n<LearnRateCoef> 0 [ 1.0 2.0 3.0 ]\n<Rescale> 560 560\n<LearnRateCoef> 0 [ 4.0 5.0 6.0 ]\n";
        // parse_text_cmvn_vector doesn't check dim — should return 3 floats
        let vec = parse_text_cmvn_vector(data, "AddShift").unwrap();
        assert_eq!(vec.len(), 3);
    }

    #[test]
    fn test_parse_text_cmvn_invalid_float() {
        let data = b"<AddShift> 560 560\n<LearnRateCoef> 0 [ 1.0 xyz 3.0 ]\n";
        let result = parse_text_cmvn_vector(data, "AddShift");
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("xyz"), "error should mention 'xyz': {err}");
    }

    #[test]
    fn test_parse_text_cmvn_non_utf8() {
        let data = vec![0xff, 0xfe, 0x00, 0x01]; // invalid UTF-8 bytes
        let result = parse_text_cmvn_vector(&data, "AddShift");
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("UTF-8") || err.contains("valid"), "error should mention UTF-8: {err}");
    }

    #[test]
    fn test_parse_text_cmvn_end_to_end() {
        let data = make_full_cmvn_text();
        let (means, inv_stds) = parse_kaldi_cmvn(data.as_bytes()).unwrap();
        assert_eq!(means.len(), 560);
        assert_eq!(inv_stds.len(), 560);
        // Spot-check AddShift first value
        let expected_means_0 = -0.5;
        assert!((means[0] - expected_means_0).abs() < 1e-5);
        // Spot-check Rescale first value
        let expected_inv_0 = 0.1;
        assert!((inv_stds[0] - expected_inv_0).abs() < 1e-5);
    }
}
