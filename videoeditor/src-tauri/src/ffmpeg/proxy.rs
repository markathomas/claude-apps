use std::path::{Path, PathBuf};

pub const PROXY_HEIGHT: u32 = 540;
// VP9 CRF range 0-63; 33 is a good proxy quality/size balance
pub const PROXY_CRF: &str = "33";

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
        "libvpx-vp9".into(),
        "-crf".into(),
        PROXY_CRF.into(),
        "-b:v".into(),
        "0".into(),
        "-deadline".into(),
        "realtime".into(),
        "-cpu-used".into(),
        "8".into(),
        "-c:a".into(),
        "libopus".into(),
        "-b:a".into(),
        "128k".into(),
        output.to_string_lossy().into(),
    ]
}

pub fn proxy_path_for(proxies_dir: &Path, media_id: &str) -> PathBuf {
    proxies_dir.join(format!("{media_id}.webm"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn proxy_args_include_input_and_output() {
        let input = Path::new("/m/in.mp4");
        let output = Path::new("/c/out.webm");
        let args = build_proxy_args(input, output);
        assert!(args.contains(&"/m/in.mp4".to_string()));
        assert!(args.contains(&"/c/out.webm".to_string()));
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
    fn proxy_args_use_vp9_codec() {
        let args = build_proxy_args(Path::new("/in"), Path::new("/out"));
        let cv_idx = args.iter().position(|s| s == "-c:v").unwrap();
        assert_eq!(args[cv_idx + 1], "libvpx-vp9");
    }

    #[test]
    fn proxy_path_uses_webm_extension() {
        let dir = Path::new("/tmp/proxies");
        let path = proxy_path_for(dir, "abc-123");
        assert_eq!(path, PathBuf::from("/tmp/proxies/abc-123.webm"));
    }
}
