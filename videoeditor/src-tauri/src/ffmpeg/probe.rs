use serde::Deserialize;
use std::path::Path;
use tokio::process::Command;

use crate::error::{AppError, AppResult};
use crate::model::project::Probe;

#[derive(Debug, Deserialize)]
struct FfprobeOutput {
    streams: Vec<Stream>,
    format: Format,
}

#[derive(Debug, Deserialize)]
struct Stream {
    codec_type: String,
    codec_name: Option<String>,
    width: Option<u32>,
    height: Option<u32>,
    r_frame_rate: Option<String>,
    avg_frame_rate: Option<String>,
}

#[derive(Debug, Deserialize)]
struct Format {
    duration: Option<String>,
}

pub const SUPPORTED_VIDEO_CODECS: &[&str] = &[
    "h264", "hevc", "h265", "vp8", "vp9", "av1", "mpeg4", "prores",
];

pub fn parse_ffprobe_json(json: &str) -> AppResult<Probe> {
    let parsed: FfprobeOutput = serde_json::from_str(json).map_err(AppError::Json)?;

    let video = parsed
        .streams
        .iter()
        .find(|s| s.codec_type == "video")
        .ok_or_else(|| AppError::InvalidPath("no video stream".into()))?;

    let video_codec = video
        .codec_name
        .clone()
        .ok_or_else(|| AppError::InvalidPath("video codec missing".into()))?;

    if !SUPPORTED_VIDEO_CODECS.contains(&video_codec.as_str()) {
        return Err(AppError::InvalidPath(format!(
            "unsupported video codec: {video_codec}"
        )));
    }

    let width = video
        .width
        .ok_or_else(|| AppError::InvalidPath("width missing".into()))?;
    let height = video
        .height
        .ok_or_else(|| AppError::InvalidPath("height missing".into()))?;

    let fps = video
        .avg_frame_rate
        .as_ref()
        .or(video.r_frame_rate.as_ref())
        .and_then(|s| parse_rational(s))
        .unwrap_or(0.0);

    let duration_secs: f64 = parsed
        .format
        .duration
        .as_deref()
        .and_then(|s| s.parse().ok())
        .unwrap_or(0.0);
    let duration_ms = (duration_secs * 1000.0) as u64;

    let audio = parsed.streams.iter().find(|s| s.codec_type == "audio");
    let has_audio = audio.is_some();
    let audio_codec = audio.and_then(|s| s.codec_name.clone());

    Ok(Probe {
        duration_ms,
        width,
        height,
        fps,
        video_codec,
        audio_codec,
        has_audio,
    })
}

fn parse_rational(s: &str) -> Option<f32> {
    let mut parts = s.split('/');
    let num: f32 = parts.next()?.parse().ok()?;
    let denom: f32 = parts.next()?.parse().ok()?;
    if denom == 0.0 {
        None
    } else {
        Some(num / denom)
    }
}

pub async fn probe_file(path: &Path) -> AppResult<Probe> {
    let output = Command::new("ffprobe")
        .args([
            "-v",
            "error",
            "-print_format",
            "json",
            "-show_streams",
            "-show_format",
        ])
        .arg(path)
        .output()
        .await
        .map_err(AppError::Io)?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(AppError::InvalidPath(format!(
            "ffprobe failed: {}",
            stderr.trim()
        )));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    parse_ffprobe_json(&stdout)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn fixture(name: &str) -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/media")
            .join(name)
    }

    #[test]
    fn parse_h264_with_audio() {
        let json = r#"{
            "streams": [
                {"codec_type": "video", "codec_name": "h264", "width": 1920, "height": 1080, "r_frame_rate": "30/1", "avg_frame_rate": "30/1"},
                {"codec_type": "audio", "codec_name": "aac"}
            ],
            "format": {"duration": "12.500000"}
        }"#;
        let probe = parse_ffprobe_json(json).unwrap();
        assert_eq!(probe.width, 1920);
        assert_eq!(probe.height, 1080);
        assert_eq!(probe.video_codec, "h264");
        assert_eq!(probe.audio_codec.as_deref(), Some("aac"));
        assert!(probe.has_audio);
        assert_eq!(probe.duration_ms, 12_500);
        assert!((probe.fps - 30.0).abs() < f32::EPSILON);
    }

    #[test]
    fn parse_video_only() {
        let json = r#"{
            "streams": [
                {"codec_type": "video", "codec_name": "h264", "width": 1280, "height": 720, "r_frame_rate": "24/1"}
            ],
            "format": {"duration": "5.0"}
        }"#;
        let probe = parse_ffprobe_json(json).unwrap();
        assert!(!probe.has_audio);
        assert!(probe.audio_codec.is_none());
    }

    #[test]
    fn rejects_no_video_stream() {
        let json = r#"{
            "streams": [{"codec_type": "audio", "codec_name": "aac"}],
            "format": {"duration": "5.0"}
        }"#;
        let err = parse_ffprobe_json(json).unwrap_err();
        assert!(err.to_string().contains("no video stream"));
    }

    #[test]
    fn rejects_unsupported_codec() {
        let json = r#"{
            "streams": [{"codec_type": "video", "codec_name": "exotic", "width": 640, "height": 480, "r_frame_rate": "30/1"}],
            "format": {"duration": "1.0"}
        }"#;
        let err = parse_ffprobe_json(json).unwrap_err();
        assert!(err.to_string().contains("unsupported video codec"));
    }

    #[test]
    fn vfr_uses_avg_frame_rate() {
        let json = r#"{
            "streams": [
                {"codec_type": "video", "codec_name": "h264", "width": 1920, "height": 1080, "r_frame_rate": "60/1", "avg_frame_rate": "29970/1000"}
            ],
            "format": {"duration": "1.0"}
        }"#;
        let probe = parse_ffprobe_json(json).unwrap();
        assert!((probe.fps - 29.97).abs() < 0.01);
    }

    #[test]
    fn parse_rational_handles_zero_denominator() {
        assert_eq!(parse_rational("30/0"), None);
        assert_eq!(parse_rational("not a rational"), None);
        assert_eq!(parse_rational(""), None);
    }

    #[tokio::test]
    async fn probe_file_returns_metadata_for_real_video() {
        let path = fixture("tiny.mp4");
        let probe = probe_file(&path).await.unwrap();
        assert!(probe.duration_ms > 0);
        assert!(probe.width > 0);
        assert!(probe.height > 0);
        assert!(!probe.video_codec.is_empty());
    }

    #[tokio::test]
    async fn probe_file_errors_on_missing_file() {
        let path = std::path::PathBuf::from("/does/not/exist.mp4");
        let err = probe_file(&path).await.unwrap_err();
        assert!(err.to_string().contains("ffprobe failed") || err.to_string().contains("io"));
    }
}
