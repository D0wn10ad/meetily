use crate::funasr_onnx::FunasrError;

const BLANK_ID: usize = 0;
const SOS_ID: usize = 1;
const EOS_ID: usize = 2;
const VOCAB_SIZE: usize = 8404;

/// Decode model output logits into text.
///
/// # Arguments
///
/// * `logits` - Flat `Vec<f32>`, shape `(T, vocab_size)` where `vocab_size = 8404`.
/// * `token_num` - Predicted valid token count from the model output.
/// * `token_list` - Vocabulary loaded from `tokens.json`.
///
/// # Returns
///
/// Decoded text string.
pub fn decode(
    logits: &[f32],
    token_num: i32,
    token_list: &[String],
) -> Result<String, FunasrError> {
    // 1. Validate input
    if logits.is_empty() {
        return Ok(String::new());
    }

    if logits.len() % VOCAB_SIZE != 0 {
        return Err(FunasrError::DecodeError(format!(
            "logits length {} is not divisible by vocab_size {}",
            logits.len(),
            VOCAB_SIZE
        )));
    }

    let num_frames = logits.len() / VOCAB_SIZE;

    // 2. Argmax per timestep
    let tokens: Vec<usize> = (0..num_frames)
        .map(|t| {
            let start = t * VOCAB_SIZE;
            let end = start + VOCAB_SIZE;
            let frame = &logits[start..end];
            argmax(frame)
        })
        .collect();

    // 3. Filter special tokens
    let filtered: Vec<usize> = tokens
        .iter()
        .copied()
        .filter(|&id| id != BLANK_ID && id != SOS_ID && id != EOS_ID)
        .collect();

    // 4. Trim to valid length
    // The model reports one extra token, so subtract pred_bias = 1
    let valid_len = (token_num - 1).max(0) as usize;
    let valid_len = valid_len.min(filtered.len());
    let selected = &filtered[..valid_len];

    // 5. Token lookup
    let mut decoded = String::new();
    for &token_id in selected {
        if token_id < token_list.len() {
            decoded.push_str(&token_list[token_id]);
        } else {
            log::warn!("FunASR decode: token {} out of range (vocab size {})", token_id, token_list.len());
        }
    }

    // 6. Post-processing: remove any remaining special token substrings
    let result = decoded
        .replace("<s>", "")
        .replace("</s>", "")
        .replace("<unk>", "")
        .replace("<blank>", "");
    let result = result.trim().to_string();

    log::info!("FunASR decode: {} logit frames, {} raw tokens → {} filtered tokens → '{}'", num_frames, tokens.len(), filtered.len(), result);

    Ok(result)
}

/// Find the index of the maximum value in a slice.
fn argmax(slice: &[f32]) -> usize {
    slice
        .iter()
        .enumerate()
        .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
        .map(|(idx, _)| idx)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_decoding() {
        let mut tokens = vec![String::new(); 8404];
        tokens[0] = "<blank>".to_string();
        tokens[1] = "<s>".to_string();
        tokens[2] = "</s>".to_string();
        tokens[3] = "我".to_string();
        tokens[4] = "是".to_string();
        tokens[5] = "谁".to_string();

        let mut logits = vec![0.0f32; 6 * 8404];
        logits[0 * 8404 + 1] = 10.0; // timestep 0 -> sos
        logits[1 * 8404 + 3] = 10.0; // timestep 1 -> 我
        logits[2 * 8404 + 4] = 10.0; // timestep 2 -> 是
        logits[3 * 8404 + 5] = 10.0; // timestep 3 -> 谁
        logits[4 * 8404 + 2] = 10.0; // timestep 4 -> eos
        logits[5 * 8404 + 0] = 10.0; // timestep 5 -> blank

        let result = decode(&logits, 4, &tokens).unwrap();
        assert_eq!(result, "我是谁");
    }

    #[test]
    fn test_empty_logits() {
        let tokens = vec!["a".to_string(); 8404];
        let result = decode(&[], 0, &tokens).unwrap();
        assert_eq!(result, "");
    }

    #[test]
    fn test_all_blank_decoding() {
        let mut tokens = vec![String::new(); 8404];
        tokens[0] = "<blank>".to_string();
        // 3 frames, uniform values -> argmax picks index 0 (blank)
        let logits = vec![1.0f32; 3 * 8404];
        let result = decode(&logits, 0, &tokens).unwrap();
        assert_eq!(result, "");
    }

    #[test]
    fn test_token_num_zero() {
        let mut tokens = vec![String::new(); 8404];
        tokens[0] = "<blank>".to_string();
        tokens[1] = "<s>".to_string();
        tokens[3] = "test".to_string();
        let mut logits = vec![0.0f32; 3 * 8404];
        logits[0 * 8404 + 1] = 5.0; // sos
        logits[1 * 8404 + 3] = 5.0; // "test"
        logits[2 * 8404 + 0] = 5.0; // blank
        let result = decode(&logits, 0, &tokens).unwrap();
        assert_eq!(result, "");
    }

    #[test]
    fn test_out_of_bounds_token_id() {
        let mut tokens = vec![String::new(); 10]; // small vocab
        tokens[0] = "<blank>".to_string();
        tokens[1] = "<s>".to_string();
        tokens[2] = "</s>".to_string();
        tokens[3] = "hello".to_string();
        let mut logits = vec![0.0f32; 3 * 8404];
        logits[0 * 8404 + 1] = 10.0; // sos
        logits[1 * 8404 + 3] = 10.0; // "hello" (in bounds)
        logits[2 * 8404 + 99] = 10.0; // 99 (out of bounds for vocab of 10)
        let result = decode(&logits, 2, &tokens).unwrap();
        assert_eq!(result, "hello");
    }

    #[test]
    fn test_invalid_logits_length() {
        let tokens = vec![String::new(); 8404];
        // 10 elements is not divisible by 8404
        let logits = vec![0.0f32; 10];
        let result = decode(&logits, 0, &tokens);
        assert!(result.is_err());
        match result {
            Err(FunasrError::DecodeError(msg)) => {
                assert!(msg.contains("not divisible"));
            }
            _ => panic!("Expected DecodeError"),
        }
    }
}
