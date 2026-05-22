# VideoEditor — Media + Proxy Pipeline Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add media import to VideoEditor — pick video files, probe them with `ffprobe`, generate 540p H.264 proxies + a thumbnail filmstrip + a peak-data waveform in the background, surface progress and ready state in a real MediaPool UI.

**Architecture:** Rust backend gains an `ffmpeg` module (subprocess wrappers for `ffmpeg` and `ffprobe`), a `media_repo` (in-memory store of imported items, scoped per-app-instance), and a `proxy_worker` (single-threaded background job queue). New Tauri commands: `import_media`, `delete_media`, `list_media`. New events: `proxy_progress`, `proxy_ready`, `proxy_failed`. The Svelte frontend gets a `mediaStore` listening to events and renders a redesigned `MediaPool` with thumbnails, status badges, and import affordances.

**Tech Stack:** Rust async via `tokio` (for proxy worker), `tokio::process::Command` for FFmpeg, `serde_json` for ffprobe parsing, Tauri 2 events, Svelte 5 runes + stores.

**Project layout (additions/changes):**

```
videoeditor/
├── src-tauri/
│   ├── Cargo.toml                              (modify: add tokio, regex)
│   ├── src/
│   │   ├── lib.rs                              (modify: register new modules + commands)
│   │   ├── ffmpeg/
│   │   │   ├── mod.rs                          (new: re-exports)
│   │   │   ├── probe.rs                        (new: ffprobe wrapper + parser)
│   │   │   ├── proxy.rs                        (new: proxy generation command)
│   │   │   ├── thumbnails.rs                   (new: filmstrip extraction)
│   │   │   ├── waveform.rs                     (new: audio peaks extraction)
│   │   │   └── progress.rs                     (new: ffmpeg stderr progress parser)
│   │   ├── media_repo.rs                       (new: in-memory MediaItem store)
│   │   ├── proxy_worker.rs                     (new: tokio background job queue)
│   │   └── commands.rs                         (modify: add import/delete/list)
│   └── tests/
│       ├── fixtures/
│       │   ├── probes/
│       │   │   ├── h264_with_audio.json        (new: canned ffprobe output)
│       │   │   ├── h264_no_audio.json          (new: video-only)
│       │   │   ├── vfr.json                    (new: variable framerate)
│       │   │   └── unsupported.json            (new: unsupported codec)
│       │   └── media/
│       │       └── tiny.mp4                    (new: 1-second test video)
│       ├── ffprobe_parsing.rs                  (new: integration tests against fixtures)
│       └── proxy_worker_lifecycle.rs           (new: spawn/cancel)
├── src/
│   ├── lib/
│   │   ├── ipc.ts                              (modify: add import/delete/list calls)
│   │   ├── types.ts                            (already covers MediaItem; no change)
│   │   ├── stores/
│   │   │   └── mediaStore.ts                   (new: media items + event subscriptions)
│   │   ├── components/
│   │   │   └── MediaPool.svelte                (modify: real implementation)
│   │   └── dialogs/
│   │       └── ImportProgressDialog.svelte     (new: optional batch import progress)
└── tests/
    └── frontend/
        └── mediaStore.test.ts                  (new)
```

---

## Prerequisites

- [ ] **Verify `ffmpeg` and `ffprobe` are on PATH.**

Run:
```bash
which ffmpeg ffprobe && ffmpeg -version | head -1
```

Expected: both binaries found, version reports.

- [ ] **Foundation plan is merged.** This plan assumes everything from `2026-05-22-videoeditor-foundation.md` is on `main`.

- [ ] **Create a feature branch.**

Run:
```bash
cd /home/mthomas/data/git/claude-apps
git checkout main && git pull --ff-only origin main 2>/dev/null || true
git checkout -b videoeditor-media
```

Expected: switched to new branch.

---

### Task 1: Add `tokio` and `regex` dependencies

**Files:**
- Modify: `videoeditor/src-tauri/Cargo.toml`

- [ ] **Step 1: Add tokio and regex to `[dependencies]`.**

Edit `videoeditor/src-tauri/Cargo.toml`. Replace the `[dependencies]` block with:

```toml
[dependencies]
tauri = { version = "2", features = [] }
tauri-plugin-dialog = "2"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
uuid = { version = "1", features = ["v4", "serde"] }
chrono = { version = "0.4", features = ["serde"] }
dirs = "5"
thiserror = "1"
tokio = { version = "1", features = ["process", "sync", "rt-multi-thread", "macros", "io-util", "time"] }
regex = "1"
```

- [ ] **Step 2: Verify build.**

Run:
```bash
cd /home/mthomas/data/git/claude-apps/videoeditor/src-tauri && cargo build 2>&1 | tail -3
```

Expected: `Finished `dev` profile`. New crates download.

- [ ] **Step 3: Commit.**

```bash
cd /home/mthomas/data/git/claude-apps
git add videoeditor/src-tauri/Cargo.toml videoeditor/src-tauri/Cargo.lock
git commit -m "chore(videoeditor): add tokio and regex deps"
```

---

### Task 2: ffprobe parser — types and parsing logic

**Files:**
- Create: `videoeditor/src-tauri/src/ffmpeg/mod.rs`
- Create: `videoeditor/src-tauri/src/ffmpeg/probe.rs`

- [ ] **Step 1: Create the module entry point.**

Create `videoeditor/src-tauri/src/ffmpeg/mod.rs`:

```rust
pub mod probe;
```

- [ ] **Step 2: Wire it into `lib.rs`.**

Edit `videoeditor/src-tauri/src/lib.rs`. Replace the entire file with:

```rust
pub mod commands;
pub mod error;
pub mod ffmpeg;
pub mod model;
pub mod paths;
pub mod project_io;
pub mod recent;

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            commands::new_project,
            commands::open_project,
            commands::save_project_cmd,
            commands::get_recent_projects,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

- [ ] **Step 3: Write the failing tests for probe parsing.**

Create `videoeditor/src-tauri/src/ffmpeg/probe.rs`:

```rust
use serde::Deserialize;

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
    let parsed: FfprobeOutput = serde_json::from_str(json)
        .map_err(AppError::Json)?;

    let video = parsed.streams.iter()
        .find(|s| s.codec_type == "video")
        .ok_or_else(|| AppError::InvalidPath("no video stream".into()))?;

    let video_codec = video.codec_name.clone()
        .ok_or_else(|| AppError::InvalidPath("video codec missing".into()))?;

    if !SUPPORTED_VIDEO_CODECS.contains(&video_codec.as_str()) {
        return Err(AppError::InvalidPath(format!(
            "unsupported video codec: {video_codec}"
        )));
    }

    let width = video.width
        .ok_or_else(|| AppError::InvalidPath("width missing".into()))?;
    let height = video.height
        .ok_or_else(|| AppError::InvalidPath("height missing".into()))?;

    let fps = video.avg_frame_rate.as_ref()
        .or(video.r_frame_rate.as_ref())
        .and_then(|s| parse_rational(s))
        .unwrap_or(0.0);

    let duration_secs: f64 = parsed.format.duration.as_deref()
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
    if denom == 0.0 { None } else { Some(num / denom) }
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
```

- [ ] **Step 4: Run unit tests.**

Run:
```bash
cd /home/mthomas/data/git/claude-apps/videoeditor/src-tauri && cargo test --lib ffmpeg::probe 2>&1 | tail -10
```

Expected: 6 tests pass.

- [ ] **Step 5: Commit.**

```bash
cd /home/mthomas/data/git/claude-apps
git add videoeditor/src-tauri/src/ffmpeg/ videoeditor/src-tauri/src/lib.rs
git commit -m "feat(videoeditor): ffprobe json parser"
```

---

### Task 3: ffprobe subprocess invocation

**Files:**
- Modify: `videoeditor/src-tauri/src/ffmpeg/probe.rs`

- [ ] **Step 1: Add the async `probe_file` function below the existing parser.**

Append to `videoeditor/src-tauri/src/ffmpeg/probe.rs` (above `#[cfg(test)]`):

```rust
use std::path::Path;
use tokio::process::Command;

pub async fn probe_file(path: &Path) -> AppResult<Probe> {
    let output = Command::new("ffprobe")
        .args([
            "-v", "error",
            "-print_format", "json",
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
```

- [ ] **Step 2: Add an integration test using a real tiny video fixture.**

The fixture file is created in the next sub-step. First, append to `videoeditor/src-tauri/src/ffmpeg/probe.rs` inside `mod tests`:

```rust
    use std::path::PathBuf;

    fn fixture(name: &str) -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/media")
            .join(name)
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
```

- [ ] **Step 3: Generate the tiny video fixture.**

Run:
```bash
cd /home/mthomas/data/git/claude-apps
mkdir -p videoeditor/src-tauri/tests/fixtures/media
ffmpeg -y -f lavfi -i "testsrc=duration=1:size=320x240:rate=24" -f lavfi -i "sine=frequency=440:duration=1" -c:v libx264 -preset ultrafast -crf 28 -c:a aac -shortest videoeditor/src-tauri/tests/fixtures/media/tiny.mp4 2>&1 | tail -5
ls -la videoeditor/src-tauri/tests/fixtures/media/tiny.mp4
```

Expected: file written, ~30–60 KB.

- [ ] **Step 4: Run the new tests.**

Run:
```bash
cd /home/mthomas/data/git/claude-apps/videoeditor/src-tauri && cargo test --lib ffmpeg::probe 2>&1 | tail -10
```

Expected: 8 tests pass (6 unit + 2 integration).

- [ ] **Step 5: Commit.**

```bash
cd /home/mthomas/data/git/claude-apps
git add videoeditor/src-tauri/src/ffmpeg/probe.rs videoeditor/src-tauri/tests/fixtures/media/tiny.mp4
git commit -m "feat(videoeditor): ffprobe subprocess wrapper"
```

---

### Task 4: FFmpeg progress parser

**Files:**
- Create: `videoeditor/src-tauri/src/ffmpeg/progress.rs`
- Modify: `videoeditor/src-tauri/src/ffmpeg/mod.rs`

- [ ] **Step 1: Add the parser with tests.**

Create `videoeditor/src-tauri/src/ffmpeg/progress.rs`:

```rust
use regex::Regex;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Progress {
    pub out_time_ms: u64,
    pub fps: Option<f32>,
}

pub struct ProgressParser {
    out_time_re: Regex,
    fps_re: Regex,
    last_out_time_ms: u64,
    last_fps: Option<f32>,
}

impl Default for ProgressParser {
    fn default() -> Self {
        Self {
            out_time_re: Regex::new(r"out_time_ms=([0-9]+)").unwrap(),
            fps_re: Regex::new(r"\bfps=\s*([0-9.]+)").unwrap(),
            last_out_time_ms: 0,
            last_fps: None,
        }
    }
}

impl ProgressParser {
    pub fn feed(&mut self, line: &str) -> Option<Progress> {
        let mut updated = false;
        if let Some(caps) = self.out_time_re.captures(line) {
            if let Ok(us) = caps[1].parse::<u64>() {
                self.last_out_time_ms = us / 1000;
                updated = true;
            }
        }
        if let Some(caps) = self.fps_re.captures(line) {
            if let Ok(f) = caps[1].parse::<f32>() {
                self.last_fps = Some(f);
            }
        }
        if updated {
            Some(Progress {
                out_time_ms: self.last_out_time_ms,
                fps: self.last_fps,
            })
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_out_time_ms_microseconds_to_ms() {
        let mut p = ProgressParser::default();
        let progress = p.feed("out_time_ms=1500000").unwrap();
        assert_eq!(progress.out_time_ms, 1500);
    }

    #[test]
    fn parses_fps_from_combined_line() {
        let mut p = ProgressParser::default();
        let line = "frame=  120 fps= 30 q=28.0 size=    256kB time=00:00:04.00 bitrate=...";
        // No out_time_ms in this line — no Progress emitted
        assert!(p.feed(line).is_none());
        // But fps was captured for next time
        assert_eq!(p.last_fps, Some(30.0));
    }

    #[test]
    fn returns_none_for_irrelevant_lines() {
        let mut p = ProgressParser::default();
        assert!(p.feed("Stream #0:0 -> #0:0").is_none());
        assert!(p.feed("[libx264] frame I").is_none());
    }

    #[test]
    fn carries_fps_into_subsequent_progress() {
        let mut p = ProgressParser::default();
        p.feed("frame=10 fps=24 time=00:00:00.40");
        let progress = p.feed("out_time_ms=400000").unwrap();
        assert_eq!(progress.out_time_ms, 400);
        assert_eq!(progress.fps, Some(24.0));
    }
}
```

- [ ] **Step 2: Wire the module.**

Replace `videoeditor/src-tauri/src/ffmpeg/mod.rs`:

```rust
pub mod probe;
pub mod progress;
```

- [ ] **Step 3: Run tests.**

Run:
```bash
cd /home/mthomas/data/git/claude-apps/videoeditor/src-tauri && cargo test --lib ffmpeg::progress 2>&1 | tail -10
```

Expected: 4 tests pass.

- [ ] **Step 4: Commit.**

```bash
cd /home/mthomas/data/git/claude-apps
git add videoeditor/src-tauri/src/ffmpeg/progress.rs videoeditor/src-tauri/src/ffmpeg/mod.rs
git commit -m "feat(videoeditor): ffmpeg stderr progress parser"
```

---

### Task 5: Proxy generation command builder

**Files:**
- Create: `videoeditor/src-tauri/src/ffmpeg/proxy.rs`
- Modify: `videoeditor/src-tauri/src/ffmpeg/mod.rs`

- [ ] **Step 1: Write the proxy module with command-builder tests.**

Create `videoeditor/src-tauri/src/ffmpeg/proxy.rs`:

```rust
use std::path::{Path, PathBuf};

pub const PROXY_HEIGHT: u32 = 540;
pub const PROXY_PRESET: &str = "veryfast";
pub const PROXY_CRF: &str = "28";

pub fn build_proxy_args(input: &Path, output: &Path) -> Vec<String> {
    vec![
        "-y".into(),
        "-progress".into(), "pipe:2".into(),
        "-nostats".into(),
        "-i".into(), input.to_string_lossy().into(),
        "-vf".into(), format!("scale=-2:{PROXY_HEIGHT}"),
        "-c:v".into(), "libx264".into(),
        "-preset".into(), PROXY_PRESET.into(),
        "-crf".into(), PROXY_CRF.into(),
        "-pix_fmt".into(), "yuv420p".into(),
        "-c:a".into(), "aac".into(),
        "-b:a".into(), "128k".into(),
        "-movflags".into(), "+faststart".into(),
        output.to_string_lossy().into(),
    ]
}

pub fn proxy_path_for(proxies_dir: &Path, media_id: &str) -> PathBuf {
    proxies_dir.join(format!("{media_id}.mp4"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn proxy_args_include_input_and_output() {
        let input = Path::new("/m/in.mp4");
        let output = Path::new("/c/out.mp4");
        let args = build_proxy_args(input, output);
        assert!(args.contains(&"/m/in.mp4".to_string()));
        assert!(args.contains(&"/c/out.mp4".to_string()));
    }

    #[test]
    fn proxy_args_use_progress_pipe() {
        let args = build_proxy_args(Path::new("/in"), Path::new("/out"));
        let pos = args.iter().position(|s| s == "-progress").unwrap();
        assert_eq!(args[pos + 1], "pipe:2");
    }

    #[test]
    fn proxy_args_target_540p_height() {
        let args = build_proxy_args(Path::new("/in"), Path::new("/out"));
        let vf_idx = args.iter().position(|s| s == "-vf").unwrap();
        assert_eq!(args[vf_idx + 1], "scale=-2:540");
    }

    #[test]
    fn proxy_path_uses_media_id_as_filename() {
        let dir = Path::new("/tmp/proxies");
        let path = proxy_path_for(dir, "abc-123");
        assert_eq!(path, PathBuf::from("/tmp/proxies/abc-123.mp4"));
    }
}
```

- [ ] **Step 2: Wire the module.**

Replace `videoeditor/src-tauri/src/ffmpeg/mod.rs`:

```rust
pub mod probe;
pub mod progress;
pub mod proxy;
```

- [ ] **Step 3: Run tests.**

Run:
```bash
cd /home/mthomas/data/git/claude-apps/videoeditor/src-tauri && cargo test --lib ffmpeg::proxy 2>&1 | tail -10
```

Expected: 4 tests pass.

- [ ] **Step 4: Commit.**

```bash
cd /home/mthomas/data/git/claude-apps
git add videoeditor/src-tauri/src/ffmpeg/proxy.rs videoeditor/src-tauri/src/ffmpeg/mod.rs
git commit -m "feat(videoeditor): proxy command builder"
```

---

### Task 6: Thumbnail filmstrip extraction

**Files:**
- Create: `videoeditor/src-tauri/src/ffmpeg/thumbnails.rs`
- Modify: `videoeditor/src-tauri/src/ffmpeg/mod.rs`

The thumbnail strip is a folder of small JPEGs sampled at a fixed interval, indexed by sequence number.

- [ ] **Step 1: Write the thumbnails module.**

Create `videoeditor/src-tauri/src/ffmpeg/thumbnails.rs`:

```rust
use std::path::{Path, PathBuf};

pub const THUMB_HEIGHT: u32 = 90;
pub const THUMB_INTERVAL_MS: u64 = 1000;

pub fn build_thumbnails_args(input: &Path, out_dir: &Path) -> Vec<String> {
    let pattern = out_dir.join("thumb_%05d.jpg");
    let fps_expr = 1000.0 / THUMB_INTERVAL_MS as f64;
    vec![
        "-y".into(),
        "-i".into(), input.to_string_lossy().into(),
        "-vf".into(), format!("fps={fps_expr},scale=-2:{THUMB_HEIGHT}"),
        "-q:v".into(), "5".into(),
        pattern.to_string_lossy().into(),
    ]
}

pub fn thumbnails_dir_for(thumbnails_root: &Path, media_id: &str) -> PathBuf {
    thumbnails_root.join(media_id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn thumbnails_args_use_fps_filter_and_height() {
        let args = build_thumbnails_args(Path::new("/in.mp4"), Path::new("/t"));
        let vf_idx = args.iter().position(|s| s == "-vf").unwrap();
        assert!(args[vf_idx + 1].contains("fps="));
        assert!(args[vf_idx + 1].contains("scale=-2:90"));
    }

    #[test]
    fn thumbnails_args_include_input_and_pattern() {
        let args = build_thumbnails_args(Path::new("/in.mp4"), Path::new("/t"));
        assert!(args.contains(&"/in.mp4".to_string()));
        assert!(args.iter().any(|s| s.ends_with("thumb_%05d.jpg")));
    }

    #[test]
    fn thumbnails_dir_uses_media_id() {
        let path = thumbnails_dir_for(Path::new("/cache/thumbnails"), "id-1");
        assert_eq!(path, PathBuf::from("/cache/thumbnails/id-1"));
    }
}
```

- [ ] **Step 2: Wire the module.**

Replace `videoeditor/src-tauri/src/ffmpeg/mod.rs`:

```rust
pub mod probe;
pub mod progress;
pub mod proxy;
pub mod thumbnails;
```

- [ ] **Step 3: Run tests.**

Run:
```bash
cd /home/mthomas/data/git/claude-apps/videoeditor/src-tauri && cargo test --lib ffmpeg::thumbnails 2>&1 | tail -10
```

Expected: 3 tests pass.

- [ ] **Step 4: Commit.**

```bash
cd /home/mthomas/data/git/claude-apps
git add videoeditor/src-tauri/src/ffmpeg/thumbnails.rs videoeditor/src-tauri/src/ffmpeg/mod.rs
git commit -m "feat(videoeditor): thumbnail filmstrip extractor"
```

---

### Task 7: Waveform peak extraction

**Files:**
- Create: `videoeditor/src-tauri/src/ffmpeg/waveform.rs`
- Modify: `videoeditor/src-tauri/src/ffmpeg/mod.rs`

The waveform is precomputed peak values stored as JSON (one peak per ~100ms window), so the frontend can render it cheaply at any zoom level.

- [ ] **Step 1: Write the waveform module.**

Create `videoeditor/src-tauri/src/ffmpeg/waveform.rs`:

```rust
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
```

- [ ] **Step 2: Wire the module.**

Replace `videoeditor/src-tauri/src/ffmpeg/mod.rs`:

```rust
pub mod probe;
pub mod progress;
pub mod proxy;
pub mod thumbnails;
pub mod waveform;
```

- [ ] **Step 3: Run tests.**

Run:
```bash
cd /home/mthomas/data/git/claude-apps/videoeditor/src-tauri && cargo test --lib ffmpeg::waveform 2>&1 | tail -10
```

Expected: 5 tests pass.

- [ ] **Step 4: Commit.**

```bash
cd /home/mthomas/data/git/claude-apps
git add videoeditor/src-tauri/src/ffmpeg/waveform.rs videoeditor/src-tauri/src/ffmpeg/mod.rs
git commit -m "feat(videoeditor): waveform peak extraction"
```

---

### Task 8: Media repository (in-memory store)

**Files:**
- Create: `videoeditor/src-tauri/src/media_repo.rs`

The media repo holds `MediaItem`s in a `Mutex<HashMap>` that lives in Tauri's managed state. It's per-app-instance — the project file persists `media_pool` separately on save.

- [ ] **Step 1: Write the repo with tests.**

Create `videoeditor/src-tauri/src/media_repo.rs`:

```rust
use std::collections::HashMap;
use std::sync::Mutex;

use uuid::Uuid;

use crate::error::{AppError, AppResult};
use crate::model::project::{MediaItem, Probe, ProxyStatus};

#[derive(Default)]
pub struct MediaRepo {
    inner: Mutex<HashMap<Uuid, MediaItem>>,
}

impl MediaRepo {
    pub fn add_pending(&self, source_path: String) -> AppResult<MediaItem> {
        let id = Uuid::new_v4();
        let item = MediaItem {
            id,
            source_path,
            proxy_path: None,
            proxy_status: ProxyStatus::Pending,
            probe: None,
        };
        let mut map = self.inner.lock().map_err(|_| poisoned())?;
        map.insert(id, item.clone());
        Ok(item)
    }

    pub fn list(&self) -> AppResult<Vec<MediaItem>> {
        let map = self.inner.lock().map_err(|_| poisoned())?;
        let mut items: Vec<MediaItem> = map.values().cloned().collect();
        items.sort_by(|a, b| a.id.cmp(&b.id));
        Ok(items)
    }

    pub fn get(&self, id: Uuid) -> AppResult<Option<MediaItem>> {
        let map = self.inner.lock().map_err(|_| poisoned())?;
        Ok(map.get(&id).cloned())
    }

    pub fn remove(&self, id: Uuid) -> AppResult<Option<MediaItem>> {
        let mut map = self.inner.lock().map_err(|_| poisoned())?;
        Ok(map.remove(&id))
    }

    pub fn set_probe(&self, id: Uuid, probe: Probe) -> AppResult<()> {
        let mut map = self.inner.lock().map_err(|_| poisoned())?;
        if let Some(item) = map.get_mut(&id) {
            item.probe = Some(probe);
        }
        Ok(())
    }

    pub fn set_proxy_status(&self, id: Uuid, status: ProxyStatus, proxy_path: Option<String>) -> AppResult<()> {
        let mut map = self.inner.lock().map_err(|_| poisoned())?;
        if let Some(item) = map.get_mut(&id) {
            item.proxy_status = status;
            if proxy_path.is_some() {
                item.proxy_path = proxy_path;
            }
        }
        Ok(())
    }
}

fn poisoned() -> AppError {
    AppError::InvalidPath("media repo lock poisoned".into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_pending_returns_item_with_pending_status() {
        let repo = MediaRepo::default();
        let item = repo.add_pending("/tmp/x.mp4".into()).unwrap();
        assert_eq!(item.source_path, "/tmp/x.mp4");
        assert_eq!(item.proxy_status, ProxyStatus::Pending);
        assert!(item.proxy_path.is_none());
        assert!(item.probe.is_none());
    }

    #[test]
    fn list_returns_all_items() {
        let repo = MediaRepo::default();
        repo.add_pending("/a.mp4".into()).unwrap();
        repo.add_pending("/b.mp4".into()).unwrap();
        let items = repo.list().unwrap();
        assert_eq!(items.len(), 2);
    }

    #[test]
    fn get_returns_some_for_known_id() {
        let repo = MediaRepo::default();
        let added = repo.add_pending("/x.mp4".into()).unwrap();
        let got = repo.get(added.id).unwrap();
        assert!(got.is_some());
        assert_eq!(got.unwrap().source_path, "/x.mp4");
    }

    #[test]
    fn get_returns_none_for_unknown_id() {
        let repo = MediaRepo::default();
        let got = repo.get(Uuid::new_v4()).unwrap();
        assert!(got.is_none());
    }

    #[test]
    fn remove_drops_item_and_returns_it() {
        let repo = MediaRepo::default();
        let added = repo.add_pending("/x.mp4".into()).unwrap();
        let removed = repo.remove(added.id).unwrap();
        assert!(removed.is_some());
        assert!(repo.get(added.id).unwrap().is_none());
    }

    #[test]
    fn set_probe_updates_item() {
        let repo = MediaRepo::default();
        let added = repo.add_pending("/x.mp4".into()).unwrap();
        let probe = Probe {
            duration_ms: 1000, width: 1920, height: 1080, fps: 30.0,
            video_codec: "h264".into(), audio_codec: None, has_audio: false,
        };
        repo.set_probe(added.id, probe.clone()).unwrap();
        let got = repo.get(added.id).unwrap().unwrap();
        assert_eq!(got.probe, Some(probe));
    }

    #[test]
    fn set_proxy_status_updates_status_and_path() {
        let repo = MediaRepo::default();
        let added = repo.add_pending("/x.mp4".into()).unwrap();
        repo.set_proxy_status(added.id, ProxyStatus::Ready, Some("/cache/p.mp4".into())).unwrap();
        let got = repo.get(added.id).unwrap().unwrap();
        assert_eq!(got.proxy_status, ProxyStatus::Ready);
        assert_eq!(got.proxy_path.as_deref(), Some("/cache/p.mp4"));
    }
}
```

- [ ] **Step 2: Wire it into `lib.rs`.**

Edit `videoeditor/src-tauri/src/lib.rs`. Replace the file with:

```rust
pub mod commands;
pub mod error;
pub mod ffmpeg;
pub mod media_repo;
pub mod model;
pub mod paths;
pub mod project_io;
pub mod recent;

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(media_repo::MediaRepo::default())
        .invoke_handler(tauri::generate_handler![
            commands::new_project,
            commands::open_project,
            commands::save_project_cmd,
            commands::get_recent_projects,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

- [ ] **Step 3: Run tests.**

Run:
```bash
cd /home/mthomas/data/git/claude-apps/videoeditor/src-tauri && cargo test --lib media_repo 2>&1 | tail -10
```

Expected: 7 tests pass.

- [ ] **Step 4: Commit.**

```bash
cd /home/mthomas/data/git/claude-apps
git add videoeditor/src-tauri/src/media_repo.rs videoeditor/src-tauri/src/lib.rs
git commit -m "feat(videoeditor): in-memory media repository"
```

---

### Task 9: Proxy worker — types and channel

**Files:**
- Create: `videoeditor/src-tauri/src/proxy_worker.rs`

The worker consumes jobs over an `mpsc::UnboundedSender`. One job per media item; the worker runs them sequentially. Splitting this across two tasks: this one defines the job type and channel, the next one wires up the actual FFmpeg execution and event emission.

- [ ] **Step 1: Write the skeleton with channel-only tests.**

Create `videoeditor/src-tauri/src/proxy_worker.rs`:

```rust
use std::path::PathBuf;

use tokio::sync::mpsc;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct ProxyJob {
    pub media_id: Uuid,
    pub source_path: PathBuf,
    pub proxy_path: PathBuf,
    pub thumbnails_dir: PathBuf,
    pub waveform_path: PathBuf,
    pub has_audio: bool,
    pub duration_ms: u64,
}

#[derive(Clone)]
pub struct ProxyWorkerHandle {
    sender: mpsc::UnboundedSender<ProxyJob>,
}

impl ProxyWorkerHandle {
    pub fn new(sender: mpsc::UnboundedSender<ProxyJob>) -> Self {
        Self { sender }
    }

    pub fn enqueue(&self, job: ProxyJob) -> Result<(), String> {
        self.sender.send(job).map_err(|e| e.to_string())
    }
}

pub fn channel() -> (ProxyWorkerHandle, mpsc::UnboundedReceiver<ProxyJob>) {
    let (tx, rx) = mpsc::unbounded_channel();
    (ProxyWorkerHandle::new(tx), rx)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_job() -> ProxyJob {
        ProxyJob {
            media_id: Uuid::new_v4(),
            source_path: PathBuf::from("/in.mp4"),
            proxy_path: PathBuf::from("/cache/p.mp4"),
            thumbnails_dir: PathBuf::from("/cache/thumbs/id"),
            waveform_path: PathBuf::from("/cache/wf/id.json"),
            has_audio: true,
            duration_ms: 5000,
        }
    }

    #[tokio::test]
    async fn enqueue_delivers_job_to_receiver() {
        let (handle, mut rx) = channel();
        let job = sample_job();
        handle.enqueue(job.clone()).unwrap();
        let received = rx.recv().await.unwrap();
        assert_eq!(received.media_id, job.media_id);
        assert_eq!(received.source_path, job.source_path);
    }

    #[tokio::test]
    async fn enqueue_fails_after_receiver_dropped() {
        let (handle, rx) = channel();
        drop(rx);
        let result = handle.enqueue(sample_job());
        assert!(result.is_err());
    }
}
```

- [ ] **Step 2: Wire it into `lib.rs`.**

Edit `videoeditor/src-tauri/src/lib.rs`. Replace with:

```rust
pub mod commands;
pub mod error;
pub mod ffmpeg;
pub mod media_repo;
pub mod model;
pub mod paths;
pub mod project_io;
pub mod proxy_worker;
pub mod recent;

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(media_repo::MediaRepo::default())
        .invoke_handler(tauri::generate_handler![
            commands::new_project,
            commands::open_project,
            commands::save_project_cmd,
            commands::get_recent_projects,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

- [ ] **Step 3: Run tests.**

Run:
```bash
cd /home/mthomas/data/git/claude-apps/videoeditor/src-tauri && cargo test --lib proxy_worker 2>&1 | tail -10
```

Expected: 2 tests pass.

- [ ] **Step 4: Commit.**

```bash
cd /home/mthomas/data/git/claude-apps
git add videoeditor/src-tauri/src/proxy_worker.rs videoeditor/src-tauri/src/lib.rs
git commit -m "feat(videoeditor): proxy worker channel skeleton"
```

---

### Task 10: Proxy worker — execution + event emission

**Files:**
- Modify: `videoeditor/src-tauri/src/proxy_worker.rs`
- Create: `videoeditor/src-tauri/tests/proxy_worker_lifecycle.rs`

This task wires the worker loop: pop a job, run FFmpeg for proxy + thumbnails + waveform (sequentially per job), update the repo, emit events. Events are emitted via a generic emitter trait so tests can capture them without a real Tauri app.

- [ ] **Step 1: Add the worker loop and emitter trait.**

Append to `videoeditor/src-tauri/src/proxy_worker.rs` (above `#[cfg(test)]`):

```rust
use std::sync::Arc;

use serde::Serialize;
use tokio::process::Command;
use tokio::io::{AsyncBufReadExt, BufReader};

use crate::ffmpeg::progress::ProgressParser;
use crate::ffmpeg::proxy::build_proxy_args;
use crate::ffmpeg::thumbnails::build_thumbnails_args;
use crate::ffmpeg::waveform::{build_waveform_args, compute_peaks, Waveform, WAVEFORM_BUCKET_MS};
use crate::media_repo::MediaRepo;
use crate::model::project::ProxyStatus;

pub trait EventEmitter: Send + Sync + 'static {
    fn emit(&self, name: &str, payload: serde_json::Value);
}

#[derive(Serialize)]
struct ProxyProgressPayload {
    media_id: String,
    percent: f32,
}

#[derive(Serialize)]
struct ProxyReadyPayload {
    media_id: String,
    proxy_path: String,
    thumbnails_dir: String,
    waveform_path: String,
}

#[derive(Serialize)]
struct ProxyFailedPayload {
    media_id: String,
    reason: String,
}

pub async fn run_worker_loop(
    mut rx: mpsc::UnboundedReceiver<ProxyJob>,
    repo: Arc<MediaRepo>,
    emitter: Arc<dyn EventEmitter>,
) {
    while let Some(job) = rx.recv().await {
        let _ = repo.set_proxy_status(job.media_id, ProxyStatus::Generating, None);

        if let Err(e) = process_job(&job, &repo, emitter.as_ref()).await {
            let _ = repo.set_proxy_status(job.media_id, ProxyStatus::Failed, None);
            emitter.emit(
                "proxy_failed",
                serde_json::to_value(ProxyFailedPayload {
                    media_id: job.media_id.to_string(),
                    reason: e,
                }).unwrap(),
            );
        }
    }
}

async fn process_job(
    job: &ProxyJob,
    repo: &MediaRepo,
    emitter: &dyn EventEmitter,
) -> Result<(), String> {
    if let Some(parent) = job.proxy_path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    std::fs::create_dir_all(&job.thumbnails_dir).map_err(|e| e.to_string())?;
    if let Some(parent) = job.waveform_path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }

    run_proxy(job, emitter).await?;
    run_thumbnails(job).await?;
    if job.has_audio {
        run_waveform(job).await?;
    } else {
        std::fs::write(
            &job.waveform_path,
            serde_json::to_string(&Waveform { bucket_ms: WAVEFORM_BUCKET_MS, peaks: vec![] }).unwrap(),
        ).map_err(|e| e.to_string())?;
    }

    repo.set_proxy_status(
        job.media_id,
        ProxyStatus::Ready,
        Some(job.proxy_path.to_string_lossy().to_string()),
    ).map_err(|e| e.to_string())?;

    emitter.emit(
        "proxy_ready",
        serde_json::to_value(ProxyReadyPayload {
            media_id: job.media_id.to_string(),
            proxy_path: job.proxy_path.to_string_lossy().to_string(),
            thumbnails_dir: job.thumbnails_dir.to_string_lossy().to_string(),
            waveform_path: job.waveform_path.to_string_lossy().to_string(),
        }).unwrap(),
    );
    Ok(())
}

async fn run_proxy(job: &ProxyJob, emitter: &dyn EventEmitter) -> Result<(), String> {
    let args = build_proxy_args(&job.source_path, &job.proxy_path);
    let mut cmd = Command::new("ffmpeg");
    cmd.args(&args);
    cmd.stderr(std::process::Stdio::piped());
    let mut child = cmd.spawn().map_err(|e| e.to_string())?;
    let stderr = child.stderr.take().ok_or_else(|| "no stderr".to_string())?;
    let mut reader = BufReader::new(stderr).lines();
    let mut parser = ProgressParser::default();

    while let Ok(Some(line)) = reader.next_line().await {
        if let Some(progress) = parser.feed(&line) {
            let percent = if job.duration_ms == 0 {
                0.0
            } else {
                (progress.out_time_ms as f32 / job.duration_ms as f32 * 100.0).clamp(0.0, 100.0)
            };
            emitter.emit(
                "proxy_progress",
                serde_json::to_value(ProxyProgressPayload {
                    media_id: job.media_id.to_string(),
                    percent,
                }).unwrap(),
            );
        }
    }

    let status = child.wait().await.map_err(|e| e.to_string())?;
    if !status.success() {
        return Err(format!("proxy ffmpeg exited with {status}"));
    }
    Ok(())
}

async fn run_thumbnails(job: &ProxyJob) -> Result<(), String> {
    let args = build_thumbnails_args(&job.source_path, &job.thumbnails_dir);
    let status = Command::new("ffmpeg").args(&args).status().await.map_err(|e| e.to_string())?;
    if !status.success() {
        return Err(format!("thumbnails ffmpeg exited with {status}"));
    }
    Ok(())
}

async fn run_waveform(job: &ProxyJob) -> Result<(), String> {
    let tmp = job.waveform_path.with_extension("pcm");
    let args = build_waveform_args(&job.source_path, &tmp);
    let status = Command::new("ffmpeg").args(&args).status().await.map_err(|e| e.to_string())?;
    if !status.success() {
        return Err(format!("waveform ffmpeg exited with {status}"));
    }
    let pcm = std::fs::read(&tmp).map_err(|e| e.to_string())?;
    let _ = std::fs::remove_file(&tmp);
    let peaks = compute_peaks(&pcm, 8000, WAVEFORM_BUCKET_MS);
    let waveform = Waveform { bucket_ms: WAVEFORM_BUCKET_MS, peaks };
    std::fs::write(
        &job.waveform_path,
        serde_json::to_string(&waveform).map_err(|e| e.to_string())?,
    ).map_err(|e| e.to_string())?;
    Ok(())
}
```

- [ ] **Step 2: Add a stub emitter and integration test.**

Create `videoeditor/src-tauri/tests/proxy_worker_lifecycle.rs`:

```rust
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use videoeditor_lib::media_repo::MediaRepo;
use videoeditor_lib::model::project::ProxyStatus;
use videoeditor_lib::proxy_worker::{channel, run_worker_loop, EventEmitter, ProxyJob};

struct Capture(Mutex<Vec<(String, serde_json::Value)>>);

impl EventEmitter for Capture {
    fn emit(&self, name: &str, payload: serde_json::Value) {
        self.0.lock().unwrap().push((name.to_string(), payload));
    }
}

fn fixture_video() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/media/tiny.mp4")
}

#[tokio::test]
async fn worker_processes_one_job_end_to_end() {
    let tmp = tempfile::tempdir().unwrap();
    let repo = Arc::new(MediaRepo::default());
    let item = repo.add_pending(fixture_video().to_string_lossy().to_string()).unwrap();
    let captured = Arc::new(Capture(Mutex::new(Vec::new())));

    let (handle, rx) = channel();
    let job = ProxyJob {
        media_id: item.id,
        source_path: fixture_video(),
        proxy_path: tmp.path().join("proxy.mp4"),
        thumbnails_dir: tmp.path().join("thumbs"),
        waveform_path: tmp.path().join("wf.json"),
        has_audio: true,
        duration_ms: 1000,
    };
    handle.enqueue(job.clone()).unwrap();
    drop(handle); // close sender so loop exits when queue drains

    let captured_for_loop: Arc<dyn EventEmitter> = captured.clone();
    run_worker_loop(rx, repo.clone(), captured_for_loop).await;

    assert!(job.proxy_path.exists(), "proxy file missing");
    assert!(job.waveform_path.exists(), "waveform file missing");

    let updated = repo.get(item.id).unwrap().unwrap();
    assert_eq!(updated.proxy_status, ProxyStatus::Ready);

    let events = captured.0.lock().unwrap();
    assert!(events.iter().any(|(n, _)| n == "proxy_ready"));
}
```

- [ ] **Step 3: Run the tests.**

Run:
```bash
cd /home/mthomas/data/git/claude-apps/videoeditor/src-tauri && cargo test --test proxy_worker_lifecycle 2>&1 | tail -10
```

Expected: 1 test passes (~3–8s — actually runs FFmpeg).

- [ ] **Step 4: Commit.**

```bash
cd /home/mthomas/data/git/claude-apps
git add videoeditor/src-tauri/src/proxy_worker.rs videoeditor/src-tauri/tests/proxy_worker_lifecycle.rs
git commit -m "feat(videoeditor): proxy worker FFmpeg execution and events"
```

---

### Task 11: Tauri event emitter implementation

**Files:**
- Modify: `videoeditor/src-tauri/src/proxy_worker.rs` (add a Tauri-specific impl)
- Modify: `videoeditor/src-tauri/src/lib.rs` (start the worker loop on app startup)

- [ ] **Step 1: Add the Tauri emitter struct.**

Append to `videoeditor/src-tauri/src/proxy_worker.rs` (above `#[cfg(test)]`). The `use tauri::Emitter;` import brings the `emit` method into scope.

```rust
use tauri::Emitter as _;

pub struct TauriEmitter {
    app: tauri::AppHandle,
}

impl TauriEmitter {
    pub fn new(app: tauri::AppHandle) -> Self { Self { app } }
}

impl EventEmitter for TauriEmitter {
    fn emit(&self, name: &str, payload: serde_json::Value) {
        let _ = self.app.emit(name, payload);
    }
}
```

- [ ] **Step 2: Spawn the worker on app startup.**

Replace `videoeditor/src-tauri/src/lib.rs` with:

```rust
use std::sync::Arc;

pub mod commands;
pub mod error;
pub mod ffmpeg;
pub mod media_repo;
pub mod model;
pub mod paths;
pub mod project_io;
pub mod proxy_worker;
pub mod recent;

pub fn run() {
    let (handle, rx) = proxy_worker::channel();
    let repo = Arc::new(media_repo::MediaRepo::default());

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(repo.clone())
        .manage(handle)
        .setup(move |app| {
            let app_handle = app.handle().clone();
            let repo = repo.clone();
            tauri::async_runtime::spawn(async move {
                let emitter: Arc<dyn proxy_worker::EventEmitter> =
                    Arc::new(proxy_worker::TauriEmitter::new(app_handle));
                proxy_worker::run_worker_loop(rx, repo, emitter).await;
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::new_project,
            commands::open_project,
            commands::save_project_cmd,
            commands::get_recent_projects,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

- [ ] **Step 3: Note about `manage()` and `Arc<MediaRepo>`.**

The repo is now managed as `Arc<MediaRepo>` instead of `MediaRepo`. Any state extractor `State<MediaRepo>` becomes `State<Arc<MediaRepo>>`. Commands added in Task 12 will use the Arc form.

- [ ] **Step 4: Verify build.**

Run:
```bash
cd /home/mthomas/data/git/claude-apps/videoeditor/src-tauri && cargo build 2>&1 | tail -3
```

Expected: build succeeds.

- [ ] **Step 5: Commit.**

```bash
cd /home/mthomas/data/git/claude-apps
git add videoeditor/src-tauri/src/proxy_worker.rs videoeditor/src-tauri/src/lib.rs
git commit -m "feat(videoeditor): wire proxy worker into tauri app"
```

---

### Task 12: Tauri commands — `import_media`, `delete_media`, `list_media`

**Files:**
- Modify: `videoeditor/src-tauri/src/commands.rs`
- Modify: `videoeditor/src-tauri/src/lib.rs` (register new commands)

- [ ] **Step 1: Add imports and command functions.**

Edit `videoeditor/src-tauri/src/commands.rs`. Replace the entire file with:

```rust
use std::path::PathBuf;
use std::sync::Arc;

use tauri::State;
use uuid::Uuid;

use crate::error::{AppError, AppResult};
use crate::ffmpeg::probe::probe_file;
use crate::ffmpeg::proxy::proxy_path_for;
use crate::ffmpeg::thumbnails::thumbnails_dir_for;
use crate::ffmpeg::waveform::waveform_path_for;
use crate::media_repo::MediaRepo;
use crate::model::project::{MediaItem, Project};
use crate::paths::{ensure_dir, proxies_dir, recent_file_path, thumbnails_dir, waveforms_dir};
use crate::project_io::{load_project, save_project};
use crate::proxy_worker::{ProxyJob, ProxyWorkerHandle};
use crate::recent::{RecentProject, RecentRegistry};

#[tauri::command]
pub fn new_project(name: String) -> AppResult<Project> {
    Ok(Project::new(name))
}

#[tauri::command]
pub fn open_project(path: String) -> AppResult<Project> {
    let path_buf = PathBuf::from(&path);
    if !path_buf.is_absolute() {
        return Err(AppError::InvalidPath(format!("not absolute: {path}")));
    }
    let project = load_project(&path_buf)?;

    let registry_path = recent_file_path()?;
    if let Some(parent) = registry_path.parent() {
        ensure_dir(&parent.to_path_buf())?;
    }
    let mut registry = RecentRegistry::load(&registry_path)?;
    registry.touch(&path_buf, &project.name);
    registry.save(&registry_path)?;

    Ok(project)
}

#[tauri::command]
pub fn save_project_cmd(project: Project, path: String) -> AppResult<()> {
    let path_buf = PathBuf::from(&path);
    if !path_buf.is_absolute() {
        return Err(AppError::InvalidPath(format!("not absolute: {path}")));
    }
    save_project(&project, &path_buf)?;

    let registry_path = recent_file_path()?;
    if let Some(parent) = registry_path.parent() {
        ensure_dir(&parent.to_path_buf())?;
    }
    let mut registry = RecentRegistry::load(&registry_path)?;
    registry.touch(&path_buf, &project.name);
    registry.save(&registry_path)?;

    Ok(())
}

#[tauri::command]
pub fn get_recent_projects() -> AppResult<Vec<RecentProject>> {
    let registry_path = recent_file_path()?;
    let mut registry = RecentRegistry::load(&registry_path)?;
    registry.prune_missing();
    Ok(registry.items)
}

#[tauri::command]
pub async fn import_media(
    paths: Vec<String>,
    repo: State<'_, Arc<MediaRepo>>,
    worker: State<'_, ProxyWorkerHandle>,
) -> AppResult<Vec<MediaItem>> {
    let proxies_root = proxies_dir()?;
    let thumbs_root = thumbnails_dir()?;
    let waves_root = waveforms_dir()?;
    ensure_dir(&proxies_root)?;
    ensure_dir(&thumbs_root)?;
    ensure_dir(&waves_root)?;

    let mut imported = Vec::with_capacity(paths.len());
    for path_str in paths {
        let path_buf = PathBuf::from(&path_str);
        if !path_buf.is_absolute() {
            return Err(AppError::InvalidPath(format!("not absolute: {path_str}")));
        }

        let probe = probe_file(&path_buf).await?;
        let mut item = repo.add_pending(path_str.clone())?;
        repo.set_probe(item.id, probe.clone())?;
        item.probe = Some(probe.clone());

        let job = ProxyJob {
            media_id: item.id,
            source_path: path_buf,
            proxy_path: proxy_path_for(&proxies_root, &item.id.to_string()),
            thumbnails_dir: thumbnails_dir_for(&thumbs_root, &item.id.to_string()),
            waveform_path: waveform_path_for(&waves_root, &item.id.to_string()),
            has_audio: probe.has_audio,
            duration_ms: probe.duration_ms,
        };
        worker.enqueue(job).map_err(|e| AppError::InvalidPath(e))?;

        imported.push(item);
    }
    Ok(imported)
}

#[tauri::command]
pub fn delete_media(id: String, repo: State<'_, Arc<MediaRepo>>) -> AppResult<()> {
    let uuid = Uuid::parse_str(&id)
        .map_err(|e| AppError::InvalidPath(format!("invalid uuid: {e}")))?;
    repo.remove(uuid)?;
    Ok(())
}

#[tauri::command]
pub fn list_media(repo: State<'_, Arc<MediaRepo>>) -> AppResult<Vec<MediaItem>> {
    repo.list()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_project_command_returns_named_project() {
        let p = new_project("Hello".into()).unwrap();
        assert_eq!(p.name, "Hello");
        assert_eq!(p.version, "1");
    }

    #[test]
    fn open_project_rejects_relative_path() {
        let err = open_project("relative/path.vproj".into()).unwrap_err();
        assert!(matches!(err, AppError::InvalidPath(_)));
    }

    #[test]
    fn save_project_rejects_relative_path() {
        let p = Project::new("X".into());
        let err = save_project_cmd(p, "relative/path.vproj".into()).unwrap_err();
        assert!(matches!(err, AppError::InvalidPath(_)));
    }
}
```

- [ ] **Step 2: Register the new commands.**

Edit `videoeditor/src-tauri/src/lib.rs`. In the `invoke_handler` macro, add the three new commands. Replace the macro call with:

```rust
        .invoke_handler(tauri::generate_handler![
            commands::new_project,
            commands::open_project,
            commands::save_project_cmd,
            commands::get_recent_projects,
            commands::import_media,
            commands::delete_media,
            commands::list_media,
        ])
```

- [ ] **Step 3: Run all Rust tests.**

Run:
```bash
cd /home/mthomas/data/git/claude-apps/videoeditor/src-tauri && cargo test 2>&1 | grep "test result"
```

Expected: every line shows `ok`.

- [ ] **Step 4: Commit.**

```bash
cd /home/mthomas/data/git/claude-apps
git add videoeditor/src-tauri/src/commands.rs videoeditor/src-tauri/src/lib.rs
git commit -m "feat(videoeditor): import_media, delete_media, list_media commands"
```

---

### Task 13: Frontend IPC — import / delete / list

**Files:**
- Modify: `videoeditor/src/lib/ipc.ts`
- Modify: `videoeditor/tests/frontend/ipc.test.ts`

- [ ] **Step 1: Add tests for the new methods.**

Edit `videoeditor/tests/frontend/ipc.test.ts`. Append to the `describe('ipc', ...)` block:

```ts
  it('importMedia calls import_media with paths', async () => {
    mockInvoke.mockResolvedValueOnce([]);
    await ipc.importMedia(['/abs/a.mp4', '/abs/b.mp4']);
    expect(mockInvoke).toHaveBeenCalledWith('import_media', { paths: ['/abs/a.mp4', '/abs/b.mp4'] });
  });

  it('deleteMedia calls delete_media with id', async () => {
    mockInvoke.mockResolvedValueOnce(null);
    await ipc.deleteMedia('abc-123');
    expect(mockInvoke).toHaveBeenCalledWith('delete_media', { id: 'abc-123' });
  });

  it('listMedia calls list_media', async () => {
    mockInvoke.mockResolvedValueOnce([]);
    const r = await ipc.listMedia();
    expect(mockInvoke).toHaveBeenCalledWith('list_media', undefined);
    expect(r).toEqual([]);
  });
```

- [ ] **Step 2: Add the methods to `ipc.ts`.**

Edit `videoeditor/src/lib/ipc.ts`. Replace the entire file with:

```ts
import { invoke } from '@tauri-apps/api/core';
import type { MediaItem, Project, RecentProject } from './types';

export const ipc = {
  newProject(name: string): Promise<Project> {
    return invoke('new_project', { name });
  },
  openProject(path: string): Promise<Project> {
    return invoke('open_project', { path });
  },
  saveProject(project: Project, path: string): Promise<void> {
    return invoke('save_project_cmd', { project, path });
  },
  getRecentProjects(): Promise<RecentProject[]> {
    return invoke('get_recent_projects', undefined);
  },
  importMedia(paths: string[]): Promise<MediaItem[]> {
    return invoke('import_media', { paths });
  },
  deleteMedia(id: string): Promise<void> {
    return invoke('delete_media', { id });
  },
  listMedia(): Promise<MediaItem[]> {
    return invoke('list_media', undefined);
  },
};
```

- [ ] **Step 3: Run frontend tests.**

Run:
```bash
cd /home/mthomas/data/git/claude-apps/videoeditor && npm test -- ipc 2>&1 | tail -10
```

Expected: 7 tests pass.

- [ ] **Step 4: Commit.**

```bash
cd /home/mthomas/data/git/claude-apps
git add videoeditor/src/lib/ipc.ts videoeditor/tests/frontend/ipc.test.ts
git commit -m "feat(videoeditor): frontend ipc for media operations"
```

---

### Task 14: Frontend media store

**Files:**
- Create: `videoeditor/src/lib/stores/mediaStore.ts`
- Create: `videoeditor/tests/frontend/mediaStore.test.ts`

The store mirrors the backend's media list. It listens to Tauri events for status updates and exposes actions for import/delete.

- [ ] **Step 1: Write the failing tests.**

Create `videoeditor/tests/frontend/mediaStore.test.ts`:

```ts
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { get } from 'svelte/store';

const mockInvoke = vi.fn();
const mockListen = vi.fn();
const eventCallbacks: Record<string, Array<(payload: unknown) => void>> = {};

vi.mock('@tauri-apps/api/core', () => ({
  invoke: (cmd: string, args?: unknown) => mockInvoke(cmd, args),
}));

vi.mock('@tauri-apps/api/event', () => ({
  listen: (name: string, cb: (e: { payload: unknown }) => void) => {
    eventCallbacks[name] ??= [];
    eventCallbacks[name].push((p) => cb({ payload: p }));
    mockListen(name);
    return Promise.resolve(() => {});
  },
}));

import { mediaStore, mediaActions } from '$lib/stores/mediaStore';

const mediaItem = {
  id: 'm1',
  source_path: '/a.mp4',
  proxy_path: null,
  proxy_status: 'pending' as const,
  probe: null,
};

function fireEvent(name: string, payload: unknown) {
  for (const cb of eventCallbacks[name] ?? []) cb(payload);
}

describe('mediaStore', () => {
  beforeEach(() => {
    mockInvoke.mockReset();
    mockListen.mockReset();
    for (const k of Object.keys(eventCallbacks)) delete eventCallbacks[k];
    mediaActions.reset();
  });

  it('starts empty', () => {
    expect(get(mediaStore).items).toEqual([]);
  });

  it('initialize subscribes to events and loads list', async () => {
    mockInvoke.mockResolvedValueOnce([mediaItem]);
    await mediaActions.initialize();
    expect(mockListen).toHaveBeenCalledWith('proxy_progress');
    expect(mockListen).toHaveBeenCalledWith('proxy_ready');
    expect(mockListen).toHaveBeenCalledWith('proxy_failed');
    expect(get(mediaStore).items).toHaveLength(1);
  });

  it('importMedia appends items', async () => {
    mockInvoke.mockResolvedValueOnce([mediaItem]);
    await mediaActions.importMedia(['/a.mp4']);
    expect(mockInvoke).toHaveBeenCalledWith('import_media', { paths: ['/a.mp4'] });
    expect(get(mediaStore).items).toEqual([mediaItem]);
  });

  it('proxy_progress updates progress for matching item', async () => {
    mockInvoke.mockResolvedValueOnce([mediaItem]);
    await mediaActions.initialize();
    fireEvent('proxy_progress', { media_id: 'm1', percent: 42 });
    const item = get(mediaStore).items[0];
    expect(item.progress).toBe(42);
  });

  it('proxy_ready updates status to ready and stores paths', async () => {
    mockInvoke.mockResolvedValueOnce([mediaItem]);
    await mediaActions.initialize();
    fireEvent('proxy_ready', {
      media_id: 'm1',
      proxy_path: '/p.mp4',
      thumbnails_dir: '/t',
      waveform_path: '/w.json',
    });
    const item = get(mediaStore).items[0];
    expect(item.proxy_status).toBe('ready');
    expect(item.proxy_path).toBe('/p.mp4');
    expect(item.thumbnails_dir).toBe('/t');
    expect(item.waveform_path).toBe('/w.json');
  });

  it('proxy_failed updates status to failed', async () => {
    mockInvoke.mockResolvedValueOnce([mediaItem]);
    await mediaActions.initialize();
    fireEvent('proxy_failed', { media_id: 'm1', reason: 'boom' });
    const item = get(mediaStore).items[0];
    expect(item.proxy_status).toBe('failed');
    expect(item.error).toBe('boom');
  });

  it('deleteMedia removes from store', async () => {
    mockInvoke.mockResolvedValueOnce([mediaItem]);
    await mediaActions.initialize();
    mockInvoke.mockResolvedValueOnce(null);
    await mediaActions.deleteMedia('m1');
    expect(mockInvoke).toHaveBeenCalledWith('delete_media', { id: 'm1' });
    expect(get(mediaStore).items).toEqual([]);
  });
});
```

- [ ] **Step 2: Write the store.**

Create `videoeditor/src/lib/stores/mediaStore.ts`:

```ts
import { writable, get } from 'svelte/store';
import { listen } from '@tauri-apps/api/event';
import { ipc } from '$lib/ipc';
import type { MediaItem } from '$lib/types';

export interface MediaItemView extends MediaItem {
  progress?: number;
  thumbnails_dir?: string;
  waveform_path?: string;
  error?: string;
}

interface MediaState {
  items: MediaItemView[];
  initialized: boolean;
}

const initial: MediaState = { items: [], initialized: false };

export const mediaStore = writable<MediaState>(initial);

interface ProxyProgressPayload { media_id: string; percent: number }
interface ProxyReadyPayload {
  media_id: string;
  proxy_path: string;
  thumbnails_dir: string;
  waveform_path: string;
}
interface ProxyFailedPayload { media_id: string; reason: string }

function update(id: string, patch: Partial<MediaItemView>) {
  mediaStore.update((s) => ({
    ...s,
    items: s.items.map((i) => (i.id === id ? { ...i, ...patch } : i)),
  }));
}

export const mediaActions = {
  reset(): void {
    mediaStore.set(initial);
  },

  async initialize(): Promise<void> {
    if (get(mediaStore).initialized) return;

    await listen<ProxyProgressPayload>('proxy_progress', (e) => {
      update(e.payload.media_id, { progress: e.payload.percent });
    });
    await listen<ProxyReadyPayload>('proxy_ready', (e) => {
      update(e.payload.media_id, {
        proxy_status: 'ready',
        proxy_path: e.payload.proxy_path,
        thumbnails_dir: e.payload.thumbnails_dir,
        waveform_path: e.payload.waveform_path,
        progress: 100,
      });
    });
    await listen<ProxyFailedPayload>('proxy_failed', (e) => {
      update(e.payload.media_id, {
        proxy_status: 'failed',
        error: e.payload.reason,
      });
    });

    const items = await ipc.listMedia();
    mediaStore.set({ items, initialized: true });
  },

  async importMedia(paths: string[]): Promise<void> {
    const newItems = await ipc.importMedia(paths);
    mediaStore.update((s) => ({ ...s, items: [...s.items, ...newItems] }));
  },

  async deleteMedia(id: string): Promise<void> {
    await ipc.deleteMedia(id);
    mediaStore.update((s) => ({ ...s, items: s.items.filter((i) => i.id !== id) }));
  },
};
```

- [ ] **Step 3: Run tests.**

Run:
```bash
cd /home/mthomas/data/git/claude-apps/videoeditor && npm test -- mediaStore 2>&1 | tail -15
```

Expected: 7 tests pass.

- [ ] **Step 4: Commit.**

```bash
cd /home/mthomas/data/git/claude-apps
git add videoeditor/src/lib/stores/mediaStore.ts videoeditor/tests/frontend/mediaStore.test.ts
git commit -m "feat(videoeditor): media store with event subscriptions"
```

---

### Task 15: MediaPool UI — list with status

**Files:**
- Modify: `videoeditor/src/lib/components/MediaPool.svelte`
- Modify: `videoeditor/src/App.svelte` (initialize the store on mount)

- [ ] **Step 1: Initialize the media store on app mount.**

Edit `videoeditor/src/App.svelte`. Replace the entire file with:

```svelte
<script lang="ts">
  import { onMount } from 'svelte';
  import MenuBar from '$lib/components/MenuBar.svelte';
  import PaneLayout from '$lib/components/PaneLayout.svelte';
  import { mediaActions } from '$lib/stores/mediaStore';

  onMount(() => {
    void mediaActions.initialize();
  });
</script>

<div class="app">
  <MenuBar />
  <PaneLayout />
</div>

<style>
  .app {
    height: 100vh;
    display: grid;
    grid-template-rows: auto 1fr;
  }
</style>
```

- [ ] **Step 2: Build out the MediaPool with import button + item list.**

Replace `videoeditor/src/lib/components/MediaPool.svelte` with:

```svelte
<script lang="ts">
  import { open as openDialog } from '@tauri-apps/plugin-dialog';
  import { mediaStore, mediaActions } from '$lib/stores/mediaStore';

  async function handleImport() {
    const picked = await openDialog({
      multiple: true,
      filters: [{ name: 'Video', extensions: ['mp4', 'mov', 'mkv', 'webm', 'm4v'] }],
    });
    if (!picked) return;
    const paths = Array.isArray(picked) ? picked : [picked];
    if (paths.length === 0) return;
    await mediaActions.importMedia(paths);
  }

  async function handleDelete(id: string) {
    await mediaActions.deleteMedia(id);
  }

  function basename(p: string): string {
    const i = Math.max(p.lastIndexOf('/'), p.lastIndexOf('\\'));
    return i >= 0 ? p.slice(i + 1) : p;
  }

  function formatDuration(ms: number | undefined): string {
    if (!ms) return '--:--';
    const total = Math.round(ms / 1000);
    const m = Math.floor(total / 60);
    const s = total % 60;
    return `${m}:${s.toString().padStart(2, '0')}`;
  }
</script>

<aside class="pane media-pool">
  <header>
    <h2>Media</h2>
    <button type="button" onclick={handleImport}>+ Import</button>
  </header>

  {#if $mediaStore.items.length === 0}
    <p class="placeholder">No media imported yet.</p>
  {:else}
    <ul>
      {#each $mediaStore.items as item (item.id)}
        <li class="item" data-status={item.proxy_status}>
          <div class="row">
            <span class="name" title={item.source_path}>{basename(item.source_path)}</span>
            <button type="button" class="delete" onclick={() => handleDelete(item.id)} aria-label="Remove">×</button>
          </div>
          <div class="meta">
            {#if item.probe}
              <span>{item.probe.width}×{item.probe.height}</span>
              <span>{formatDuration(item.probe.duration_ms)}</span>
              {#if !item.probe.has_audio}<span class="muted-tag">no audio</span>{/if}
            {/if}
          </div>
          <div class="status">
            {#if item.proxy_status === 'pending'}
              <span class="badge pending">queued</span>
            {:else if item.proxy_status === 'generating'}
              <span class="badge generating">proxy {Math.round(item.progress ?? 0)}%</span>
            {:else if item.proxy_status === 'ready'}
              <span class="badge ready">ready</span>
            {:else if item.proxy_status === 'failed'}
              <span class="badge failed" title={item.error ?? ''}>failed</span>
            {/if}
          </div>
        </li>
      {/each}
    </ul>
  {/if}
</aside>

<style>
  .pane { padding: 0.5rem 0.75rem; overflow: auto; }
  header { display: flex; align-items: center; justify-content: space-between; margin-bottom: 0.5rem; }
  h2 { font-size: 0.85rem; text-transform: uppercase; letter-spacing: 0.05em; opacity: 0.7; margin: 0; }
  header button {
    background: #2563eb; color: white; border: 0; padding: 0.3rem 0.6rem;
    border-radius: 4px; font-size: 0.8rem; cursor: pointer;
  }
  header button:hover { background: #1d4ed8; }
  .placeholder { font-size: 0.875rem; opacity: 0.6; }
  ul { list-style: none; padding: 0; margin: 0; display: flex; flex-direction: column; gap: 0.4rem; }
  .item {
    background: #1f1f1f; padding: 0.5rem; border-radius: 4px;
    border-left: 3px solid #555;
  }
  .item[data-status="ready"] { border-left-color: #22c55e; }
  .item[data-status="failed"] { border-left-color: #ef4444; }
  .item[data-status="generating"] { border-left-color: #2563eb; }
  .row { display: flex; align-items: center; justify-content: space-between; gap: 0.5rem; }
  .name { font-size: 0.875rem; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .delete {
    background: transparent; border: 0; color: inherit; opacity: 0.5;
    font-size: 1rem; line-height: 1; cursor: pointer; padding: 0 0.25rem;
  }
  .delete:hover { opacity: 1; }
  .meta { display: flex; gap: 0.5rem; font-size: 0.75rem; opacity: 0.6; margin-top: 0.25rem; }
  .muted-tag { color: #f59e0b; }
  .status { margin-top: 0.4rem; }
  .badge {
    display: inline-block; font-size: 0.7rem; padding: 0.1rem 0.4rem;
    border-radius: 2px; background: #2a2a2a;
  }
  .badge.ready { background: #15803d; }
  .badge.failed { background: #991b1b; }
  .badge.generating { background: #1d4ed8; }
</style>
```

- [ ] **Step 3: Verify svelte-check passes.**

Run:
```bash
cd /home/mthomas/data/git/claude-apps/videoeditor && npx svelte-check 2>&1 | tail -3
```

Expected: 0 errors.

- [ ] **Step 4: Run all tests.**

Run:
```bash
cd /home/mthomas/data/git/claude-apps/videoeditor && npm test 2>&1 | tail -10
```

Expected: every test file green.

- [ ] **Step 5: Commit.**

```bash
cd /home/mthomas/data/git/claude-apps
git add videoeditor/src/lib/components/MediaPool.svelte videoeditor/src/App.svelte
git commit -m "feat(videoeditor): MediaPool with import, list, and status"
```

---

### Task 16: Manual smoke test

**Files:** none — verification only.

- [ ] **Step 1: Run the dev server.**

Run:
```bash
cd /home/mthomas/data/git/claude-apps/videoeditor && npm run tauri dev
```

Expected: window opens.

- [ ] **Step 2: Smoke test media import.**

In the running app:

1. Click **+ Import** in the Media pane. Select one or two video files (any MP4 you have lying around, or use `videoeditor/src-tauri/tests/fixtures/media/tiny.mp4`).
2. Observe that the items appear immediately with `queued` status, then transition to `proxy N%` (generating) with a progress bar update, then `ready`.
3. Verify:
   ```bash
   ls -la ~/.cache/videoeditor/proxies/
   ls -la ~/.cache/videoeditor/thumbnails/
   ls -la ~/.cache/videoeditor/waveforms/
   ```
   Expected: a proxy `.mp4`, a thumbnails subdir with several JPEGs, and a waveform `.json` file per imported item.
4. Open one of the waveform JSONs:
   ```bash
   cat ~/.cache/videoeditor/waveforms/*.json | python3 -m json.tool | head -10
   ```
   Expected: `{"bucket_ms":100,"peaks":[...]}` with peak values between 0 and 1.
5. Click the × on an item. Verify it disappears from the pool.

- [ ] **Step 3: Test the "no audio" path.**

Generate a video-only file:
```bash
ffmpeg -y -f lavfi -i "testsrc=duration=2:size=320x240:rate=24" -c:v libx264 -preset ultrafast /tmp/silent.mp4
```

Import it. Verify the item shows `no audio` tag and proxy still completes.

- [ ] **Step 4: Test the unsupported codec path.**

If you have a file with an unsupported codec (or one that isn't actually a video), try importing it. Verify a clear error surfaces (currently in the DevTools console — UI error display lands in a future plan).

If you don't have such a file handy, this step can be skipped — the unit test covers the rejection path.

- [ ] **Step 5: Close the app.**

Ctrl+C in the terminal.

- [ ] **Step 6: No commit needed.**

---

### Task 17: Final verification + plan close-out

**Files:** none — verification only.

- [ ] **Step 1: Run all Rust tests.**

Run:
```bash
cd /home/mthomas/data/git/claude-apps/videoeditor/src-tauri && cargo test 2>&1 | grep "test result"
```

Expected: every line `ok`. Approximate counts: 60+ unit tests, 5+ integration tests.

- [ ] **Step 2: Run all frontend tests.**

Run:
```bash
cd /home/mthomas/data/git/claude-apps/videoeditor && npm test 2>&1 | tail -10
```

Expected: 17+ tests passing across 3 files.

- [ ] **Step 3: Run TypeScript check.**

Run:
```bash
cd /home/mthomas/data/git/claude-apps/videoeditor && npx svelte-check 2>&1 | tail -3
```

Expected: 0 errors.

- [ ] **Step 4: Run release build.**

Run:
```bash
cd /home/mthomas/data/git/claude-apps/videoeditor && npm run build && cd src-tauri && cargo build --release 2>&1 | tail -3
```

Expected: both succeed.

- [ ] **Step 5: No commit needed.**

---

## What ships at the end of this plan

- Click **+ Import** to pick one or more video files (MP4/MOV/MKV/WebM/M4V)
- Each imported file is probed with `ffprobe` and metadata appears in the MediaPool
- Background worker generates a 540p H.264 proxy + thumbnail filmstrip + audio waveform peaks
- Status surfaces as colored badges with live progress %
- Files persist in `$XDG_CACHE_HOME/videoeditor/{proxies,thumbnails,waveforms}/`
- Unsupported codecs are rejected at import with a clear message
- Video-only files skip the waveform step gracefully

**Test counts at end of plan:**
- Rust: ~62 unit + 5 integration (including a real-FFmpeg lifecycle test)
- Frontend: 17 Vitest tests across 3 files

## What's NOT in this plan (deferred)

- Drag-and-drop import (only file picker for now)
- Thumbnail rendering in the UI (filmstrip is generated; rendering is part of Plan 3 timeline work)
- Waveform rendering in the UI (data is generated; rendering is part of Plan 3)
- Error UX (failures surface in DevTools console; modal UX lands in Plan 7)
- Reflecting `media_pool` from the project file into the repo on `open_project` — currently the repo is fresh per app launch and the project's `media_pool` field isn't yet populated. Plan 3 reconciles this when timeline editing actually consumes media.
- Cancellation of in-flight proxy jobs (jobs run to completion or fail)

## Next plan

**Plan 3 — Timeline core.** Adds the timeline reducer, drag-from-MediaPool-to-timeline, trim/split/delete, undo/redo, snap-to-edge, preview playback using the proxies we now generate. With Plan 3, the editor reaches "I can cut a real video end-to-end inside the app" — the first genuinely useful state.
