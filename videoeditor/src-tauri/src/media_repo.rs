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

    pub fn set_proxy_status(
        &self,
        id: Uuid,
        status: ProxyStatus,
        proxy_path: Option<String>,
    ) -> AppResult<()> {
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
            duration_ms: 1000,
            width: 1920,
            height: 1080,
            fps: 30.0,
            video_codec: "h264".into(),
            audio_codec: None,
            has_audio: false,
        };
        repo.set_probe(added.id, probe.clone()).unwrap();
        let got = repo.get(added.id).unwrap().unwrap();
        assert_eq!(got.probe, Some(probe));
    }

    #[test]
    fn set_proxy_status_updates_status_and_path() {
        let repo = MediaRepo::default();
        let added = repo.add_pending("/x.mp4".into()).unwrap();
        repo.set_proxy_status(added.id, ProxyStatus::Ready, Some("/cache/p.mp4".into()))
            .unwrap();
        let got = repo.get(added.id).unwrap().unwrap();
        assert_eq!(got.proxy_status, ProxyStatus::Ready);
        assert_eq!(got.proxy_path.as_deref(), Some("/cache/p.mp4"));
    }
}
