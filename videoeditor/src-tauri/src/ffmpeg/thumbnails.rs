use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::{AppError, AppResult};

pub const THUMB_HEIGHT: u32 = 90;
pub const THUMB_INTERVAL_MS: u64 = 1000;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ThumbEntry {
    pub time_ms: u64,
    pub path: String,
}

pub fn build_thumbnails_args(input: &Path, out_dir: &Path) -> Vec<String> {
    let pattern = out_dir.join("thumb_%05d.jpg");
    let fps_expr = 1000.0 / THUMB_INTERVAL_MS as f64;
    vec![
        "-y".into(),
        "-i".into(),
        input.to_string_lossy().into(),
        "-vf".into(),
        format!("fps={fps_expr},scale=-2:{THUMB_HEIGHT}"),
        "-q:v".into(),
        "5".into(),
        pattern.to_string_lossy().into(),
    ]
}

pub fn thumbnails_dir_for(thumbnails_root: &Path, media_id: &str) -> PathBuf {
    thumbnails_root.join(media_id)
}

/// Parse a thumbnail filename like `thumb_00001.jpg` into a 1-based frame index.
/// Returns `None` if the name does not match the expected pattern.
fn parse_thumb_index(file_name: &str) -> Option<u64> {
    let stem = file_name.strip_suffix(".jpg")?;
    let digits = stem.strip_prefix("thumb_")?;
    if digits.is_empty() || !digits.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }
    digits.parse::<u64>().ok()
}

/// List thumbnails in a directory previously populated by `build_thumbnails_args`.
///
/// Each entry's `time_ms` is derived from the 1-based frame index in the filename
/// (`thumb_NNNNN.jpg`) so frame 1 maps to `0 ms`, frame 2 to `THUMB_INTERVAL_MS`, etc.
/// Returns an empty list if the directory does not exist (proxy may not be ready yet).
pub fn list_thumbnails_in_dir(dir: &Path) -> AppResult<Vec<ThumbEntry>> {
    if !dir.exists() {
        return Ok(Vec::new());
    }
    let mut entries: Vec<ThumbEntry> = Vec::new();
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        if !entry.file_type()?.is_file() {
            continue;
        }
        let name = entry.file_name();
        let name = match name.to_str() {
            Some(n) => n,
            None => continue,
        };
        let Some(index) = parse_thumb_index(name) else {
            continue;
        };
        if index == 0 {
            continue;
        }
        let time_ms = (index - 1) * THUMB_INTERVAL_MS;
        let path = entry.path();
        let path_str = path
            .to_str()
            .ok_or_else(|| AppError::InvalidPath(format!("non-utf8 thumbnail path: {path:?}")))?
            .to_string();
        entries.push(ThumbEntry { time_ms, path: path_str });
    }
    entries.sort_by_key(|e| e.time_ms);
    Ok(entries)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use tempfile::tempdir;

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

    #[test]
    fn parse_thumb_index_accepts_valid_pattern() {
        assert_eq!(parse_thumb_index("thumb_00001.jpg"), Some(1));
        assert_eq!(parse_thumb_index("thumb_00042.jpg"), Some(42));
        assert_eq!(parse_thumb_index("thumb_99999.jpg"), Some(99999));
    }

    #[test]
    fn parse_thumb_index_rejects_invalid_patterns() {
        assert_eq!(parse_thumb_index("thumb_.jpg"), None);
        assert_eq!(parse_thumb_index("thumb_abc.jpg"), None);
        assert_eq!(parse_thumb_index("preview_00001.jpg"), None);
        assert_eq!(parse_thumb_index("thumb_00001.png"), None);
        assert_eq!(parse_thumb_index("thumb_00001"), None);
        assert_eq!(parse_thumb_index("00001.jpg"), None);
    }

    #[test]
    fn list_thumbnails_returns_empty_for_missing_dir() {
        let dir = tempdir().unwrap();
        let missing = dir.path().join("does-not-exist");
        let entries = list_thumbnails_in_dir(&missing).unwrap();
        assert!(entries.is_empty());
    }

    #[test]
    fn list_thumbnails_returns_empty_for_empty_dir() {
        let dir = tempdir().unwrap();
        let entries = list_thumbnails_in_dir(dir.path()).unwrap();
        assert!(entries.is_empty());
    }

    #[test]
    fn list_thumbnails_sorts_by_time_and_maps_to_ms() {
        let dir = tempdir().unwrap();
        // Create files out of order to verify sorting.
        for name in ["thumb_00003.jpg", "thumb_00001.jpg", "thumb_00002.jpg"] {
            File::create(dir.path().join(name)).unwrap();
        }
        let entries = list_thumbnails_in_dir(dir.path()).unwrap();
        assert_eq!(entries.len(), 3);
        assert_eq!(entries[0].time_ms, 0);
        assert_eq!(entries[1].time_ms, THUMB_INTERVAL_MS);
        assert_eq!(entries[2].time_ms, 2 * THUMB_INTERVAL_MS);
        assert!(entries[0].path.ends_with("thumb_00001.jpg"));
        assert!(entries[1].path.ends_with("thumb_00002.jpg"));
        assert!(entries[2].path.ends_with("thumb_00003.jpg"));
    }

    #[test]
    fn list_thumbnails_ignores_non_matching_files() {
        let dir = tempdir().unwrap();
        for name in [
            "thumb_00001.jpg",
            "preview.jpg",
            "thumb_abc.jpg",
            "readme.txt",
        ] {
            File::create(dir.path().join(name)).unwrap();
        }
        let entries = list_thumbnails_in_dir(dir.path()).unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].time_ms, 0);
    }
}
