use std::path::PathBuf;

use crate::error::{AppError, AppResult};

pub const APP_DIR_NAME: &str = "videoeditor";

pub fn config_dir() -> AppResult<PathBuf> {
    let base = dirs::config_dir()
        .ok_or_else(|| AppError::InvalidPath("config dir unavailable".into()))?;
    Ok(base.join(APP_DIR_NAME))
}

pub fn cache_dir() -> AppResult<PathBuf> {
    let base = dirs::cache_dir()
        .ok_or_else(|| AppError::InvalidPath("cache dir unavailable".into()))?;
    Ok(base.join(APP_DIR_NAME))
}

pub fn ensure_dir(path: &PathBuf) -> AppResult<()> {
    if !path.exists() {
        std::fs::create_dir_all(path)?;
    }
    Ok(())
}

pub fn recent_file_path() -> AppResult<PathBuf> {
    Ok(config_dir()?.join("recent.json"))
}

pub fn proxies_dir() -> AppResult<PathBuf> {
    Ok(cache_dir()?.join("proxies"))
}

pub fn thumbnails_dir() -> AppResult<PathBuf> {
    Ok(cache_dir()?.join("thumbnails"))
}

pub fn waveforms_dir() -> AppResult<PathBuf> {
    Ok(cache_dir()?.join("waveforms"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_dir_ends_with_app_name() {
        let p = config_dir().unwrap();
        assert!(p.ends_with(APP_DIR_NAME), "expected dir ending with {APP_DIR_NAME}, got {p:?}");
    }

    #[test]
    fn cache_subdirs_are_under_app_cache() {
        let cache = cache_dir().unwrap();
        assert!(proxies_dir().unwrap().starts_with(&cache));
        assert!(thumbnails_dir().unwrap().starts_with(&cache));
        assert!(waveforms_dir().unwrap().starts_with(&cache));
    }

    #[test]
    fn recent_file_path_is_under_config_dir() {
        let recent = recent_file_path().unwrap();
        let config = config_dir().unwrap();
        assert!(recent.starts_with(&config));
        assert_eq!(recent.file_name().unwrap(), "recent.json");
    }

    #[test]
    fn ensure_dir_creates_missing_dir() {
        let tmp = tempfile::tempdir().unwrap();
        let target = tmp.path().join("a/b/c");
        assert!(!target.exists());
        ensure_dir(&target).unwrap();
        assert!(target.exists());
    }
}
