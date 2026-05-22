use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

pub const WAVEFORM_BUCKET_MS: u64 = 100;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Waveform {
    pub bucket_ms: u64,
    pub peaks: Vec<f32>,
}

pub fn build_waveform_args(input: &Path, raw_pcm_out: &Path) -> Vec<String> {
    vec![
        "-y".into(),
        "-i".into(), input.to_string_lossy().into(),
        "-vn".into(),
        "-ac".into(), "1".into(),
        "-ar".into(), "8000".into(),
        "-f".into(), "s16le".into(),
        raw_pcm_out.to_string_lossy().into(),
    ]
}

pub fn compute_peaks(pcm_s16le: &[u8], sample_rate: u32, bucket_ms: u64) -> Vec<f32> {
    let samples_per_bucket = (sample_rate as u64 * bucket_ms / 1000).max(1) as usize;
    let mut peaks = Vec::new();
    let total_samples = pcm_s16le.len() / 2;
    let mut i = 0;
    while i < total_samples {
        let end = (i + samples_per_bucket).min(total_samples);
        let mut max: i16 = 0;
        for s in i..end {
            let lo = pcm_s16le[s * 2];
            let hi = pcm_s16le[s * 2 + 1];
            let sample = i16::from_le_bytes([lo, hi]);
            let abs_sample = sample.saturating_abs();
            if abs_sample > max { max = abs_sample; }
        }
        peaks.push(max as f32 / i16::MAX as f32);
        i = end;
    }
    peaks
}

pub fn waveform_path_for(waveforms_root: &Path, media_id: &str) -> PathBuf {
    waveforms_root.join(format!("{media_id}.json"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn waveform_args_force_mono_8khz_s16le() {
        let args = build_waveform_args(Path::new("/in.mp4"), Path::new("/out.pcm"));
        assert!(args.iter().any(|s| s == "-ac"));
        let ac_idx = args.iter().position(|s| s == "-ac").unwrap();
        assert_eq!(args[ac_idx + 1], "1");
        let ar_idx = args.iter().position(|s| s == "-ar").unwrap();
        assert_eq!(args[ar_idx + 1], "8000");
        let f_idx = args.iter().position(|s| s == "-f").unwrap();
        assert_eq!(args[f_idx + 1], "s16le");
    }

    #[test]
    fn compute_peaks_normalized_against_i16_max() {
        // 100ms at 8000Hz = 800 samples per bucket
        // Build 1 bucket of constant max-amplitude samples
        let mut buf = Vec::new();
        for _ in 0..800 {
            buf.extend_from_slice(&i16::MAX.to_le_bytes());
        }
        let peaks = compute_peaks(&buf, 8000, 100);
        assert_eq!(peaks.len(), 1);
        assert!((peaks[0] - 1.0).abs() < 0.001);
    }

    #[test]
    fn compute_peaks_handles_empty_input() {
        let peaks = compute_peaks(&[], 8000, 100);
        assert!(peaks.is_empty());
    }

    #[test]
    fn compute_peaks_silent_input_yields_zeros() {
        let buf = vec![0u8; 1600]; // 800 samples of silence
        let peaks = compute_peaks(&buf, 8000, 100);
        assert_eq!(peaks.len(), 1);
        assert!(peaks[0].abs() < 0.001);
    }

    #[test]
    fn waveform_path_uses_media_id_with_json_ext() {
        let p = waveform_path_for(Path::new("/cache/waveforms"), "abc");
        assert_eq!(p, PathBuf::from("/cache/waveforms/abc.json"));
    }
}
