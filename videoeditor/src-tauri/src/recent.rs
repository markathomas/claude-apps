use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::error::AppResult;

pub const MAX_RECENT: usize = 10;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RecentProject {
    pub path: String,
    pub name: String,
    pub last_opened: DateTime<Utc>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct RecentRegistry {
    pub items: Vec<RecentProject>,
}

impl RecentRegistry {
    pub fn touch(&mut self, path: &Path, name: &str) {
        let path_str = path.to_string_lossy().to_string();
        self.items.retain(|r| r.path != path_str);
        self.items.insert(0, RecentProject {
            path: path_str,
            name: name.to_string(),
            last_opened: Utc::now(),
        });
        self.items.truncate(MAX_RECENT);
    }

    pub fn remove(&mut self, path: &Path) {
        let path_str = path.to_string_lossy().to_string();
        self.items.retain(|r| r.path != path_str);
    }

    pub fn load(path: &Path) -> AppResult<Self> {
        if !path.exists() {
            return Ok(Self::default());
        }
        let bytes = std::fs::read(path)?;
        let registry: Self = serde_json::from_slice(&bytes).unwrap_or_default();
        Ok(registry)
    }

    pub fn save(&self, path: &Path) -> AppResult<()> {
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() && !parent.exists() {
                std::fs::create_dir_all(parent)?;
            }
        }
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(path, json)?;
        Ok(())
    }

    pub fn prune_missing(&mut self) {
        self.items.retain(|r| PathBuf::from(&r.path).exists());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn touch_inserts_at_front_and_dedupes() {
        let mut r = RecentRegistry::default();
        r.touch(Path::new("/a/p.vproj"), "A");
        r.touch(Path::new("/b/p.vproj"), "B");
        r.touch(Path::new("/a/p.vproj"), "A again");
        assert_eq!(r.items.len(), 2);
        assert_eq!(r.items[0].path, "/a/p.vproj");
        assert_eq!(r.items[0].name, "A again");
        assert_eq!(r.items[1].path, "/b/p.vproj");
    }

    #[test]
    fn touch_caps_at_max_recent() {
        let mut r = RecentRegistry::default();
        for i in 0..(MAX_RECENT + 5) {
            r.touch(&PathBuf::from(format!("/p{i}.vproj")), &format!("P{i}"));
        }
        assert_eq!(r.items.len(), MAX_RECENT);
        assert_eq!(r.items[0].path, format!("/p{}.vproj", MAX_RECENT + 4));
    }

    #[test]
    fn remove_drops_entry() {
        let mut r = RecentRegistry::default();
        r.touch(Path::new("/a/p.vproj"), "A");
        r.touch(Path::new("/b/p.vproj"), "B");
        r.remove(Path::new("/a/p.vproj"));
        assert_eq!(r.items.len(), 1);
        assert_eq!(r.items[0].path, "/b/p.vproj");
    }

    #[test]
    fn load_returns_empty_when_file_missing() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("nope.json");
        let r = RecentRegistry::load(&path).unwrap();
        assert!(r.items.is_empty());
    }

    #[test]
    fn save_then_load_round_trip() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("recent.json");
        let mut r = RecentRegistry::default();
        r.touch(Path::new("/a/p.vproj"), "A");
        r.save(&path).unwrap();
        let loaded = RecentRegistry::load(&path).unwrap();
        assert_eq!(loaded, r);
    }

    #[test]
    fn load_corrupt_file_yields_empty_registry() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("recent.json");
        std::fs::write(&path, "garbage").unwrap();
        let r = RecentRegistry::load(&path).unwrap();
        assert!(r.items.is_empty());
    }

    #[test]
    fn prune_missing_drops_paths_that_dont_exist() {
        let tmp = tempfile::tempdir().unwrap();
        let exists = tmp.path().join("real.vproj");
        std::fs::write(&exists, "{}").unwrap();
        let mut r = RecentRegistry::default();
        r.touch(&exists, "Real");
        r.touch(Path::new("/does/not/exist.vproj"), "Ghost");
        r.prune_missing();
        assert_eq!(r.items.len(), 1);
        assert_eq!(r.items[0].path, exists.to_string_lossy());
    }
}
