use std::path::{Path, PathBuf};

pub const PROXY_HEIGHT: u32 = 540;
// x264 CRF range 0-51; 20 is visually transparent for proxy use.
pub const PROXY_CRF: &str = "20";
// Short GOP for responsive scrubbing without blowing up file size.
pub const PROXY_GOP: &str = "30";

pub fn build_proxy_args(input: &Path, output: &Path) -> Vec<String> {
    vec![
        "-y".into(),
        "-progress".into(),
        "pipe:2".into(),
        "-nostats".into(),
        "-i".into(),
        input.to_string_lossy().into(),
        "-vf".into(),
        format!("scale=-2:{PROXY_HEIGHT}"),
        "-c:v".into(),
        "libx264".into(),
        "-preset".into(),
        "veryfast".into(),
        "-crf".into(),
        PROXY_CRF.into(),
        "-pix_fmt".into(),
        "yuv420p".into(),
        "-g".into(),
        PROXY_GOP.into(),
        "-keyint_min".into(),
        PROXY_GOP.into(),
        "-movflags".into(),
        "+faststart".into(),
        "-c:a".into(),
        "aac".into(),
        "-b:a".into(),
        "128k".into(),
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
    fn proxy_args_use_h264_codec() {
        let args = build_proxy_args(Path::new("/in"), Path::new("/out"));
        let cv_idx = args.iter().position(|s| s == "-c:v").unwrap();
        assert_eq!(args[cv_idx + 1], "libx264");
    }

    #[test]
    fn proxy_args_use_yuv420p_pix_fmt() {
        let args = build_proxy_args(Path::new("/in"), Path::new("/out"));
        let idx = args.iter().position(|s| s == "-pix_fmt").unwrap();
        assert_eq!(args[idx + 1], "yuv420p");
    }

    #[test]
    fn proxy_args_set_short_gop_for_scrubbing() {
        let args = build_proxy_args(Path::new("/in"), Path::new("/out"));
        let g_idx = args.iter().position(|s| s == "-g").unwrap();
        assert_eq!(args[g_idx + 1], PROXY_GOP);
        let kmin_idx = args.iter().position(|s| s == "-keyint_min").unwrap();
        assert_eq!(args[kmin_idx + 1], PROXY_GOP);
    }

    #[test]
    fn proxy_args_enable_faststart() {
        let args = build_proxy_args(Path::new("/in"), Path::new("/out"));
        let idx = args.iter().position(|s| s == "-movflags").unwrap();
        assert_eq!(args[idx + 1], "+faststart");
    }

    #[test]
    fn proxy_path_uses_mp4_extension() {
        let dir = Path::new("/tmp/proxies");
        let path = proxy_path_for(dir, "abc-123");
        assert_eq!(path, PathBuf::from("/tmp/proxies/abc-123.mp4"));
    }
}
