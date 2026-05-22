use std::path::{Path, PathBuf};

pub const THUMB_HEIGHT: u32 = 90;
pub const THUMB_INTERVAL_MS: u64 = 1000;

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
