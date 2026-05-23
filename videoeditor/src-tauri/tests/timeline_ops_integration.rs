use std::path::PathBuf;

use uuid::Uuid;
use videoeditor_lib::media_repo::MediaRepo;
use videoeditor_lib::model::project::{MediaItem, Project, ProxyStatus};
use videoeditor_lib::project_io::{load_project, save_project};

fn make_item(source: &str, proxy: Option<&str>, status: ProxyStatus) -> MediaItem {
    MediaItem {
        id: Uuid::new_v4(),
        source_path: source.into(),
        proxy_path: proxy.map(|s| s.into()),
        proxy_status: status,
        probe: None,
    }
}

fn write_dummy(path: &PathBuf) {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).unwrap();
    }
    std::fs::write(path, b"x").unwrap();
}

#[test]
fn reconcile_from_project_populates_repo_with_media_items() {
    let tmp = tempfile::tempdir().unwrap();
    let proxy_path = tmp.path().join("proxy.mp4");
    write_dummy(&proxy_path);

    let mut project = Project::new("Reconcile Test".into());
    let item = make_item(
        "/in/foo.mp4",
        Some(proxy_path.to_string_lossy().as_ref()),
        ProxyStatus::Ready,
    );
    let item_id = item.id;
    project.media_pool.push(item);

    let repo = MediaRepo::default();
    repo.reconcile_from_project(&project).unwrap();

    let items = repo.list().unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].id, item_id);
    assert_eq!(items[0].source_path, "/in/foo.mp4");
}

#[test]
fn reconcile_marks_ready_when_proxy_file_exists() {
    let tmp = tempfile::tempdir().unwrap();
    let proxy_path = tmp.path().join("proxy.mp4");
    write_dummy(&proxy_path);

    let mut project = Project::new("R".into());
    project.media_pool.push(make_item(
        "/in/foo.mp4",
        Some(proxy_path.to_string_lossy().as_ref()),
        ProxyStatus::Ready,
    ));

    let repo = MediaRepo::default();
    repo.reconcile_from_project(&project).unwrap();

    let items = repo.list().unwrap();
    assert_eq!(items[0].proxy_status, ProxyStatus::Ready);
}

#[test]
fn reconcile_marks_pending_when_proxy_file_missing() {
    let tmp = tempfile::tempdir().unwrap();
    let missing_proxy = tmp.path().join("does_not_exist.mp4");

    let mut project = Project::new("R".into());
    project.media_pool.push(make_item(
        "/in/foo.mp4",
        Some(missing_proxy.to_string_lossy().as_ref()),
        ProxyStatus::Ready,
    ));

    let repo = MediaRepo::default();
    repo.reconcile_from_project(&project).unwrap();

    let items = repo.list().unwrap();
    assert_eq!(items[0].proxy_status, ProxyStatus::Pending);
}

#[test]
fn reconcile_marks_pending_when_no_proxy_path_recorded() {
    let mut project = Project::new("R".into());
    project
        .media_pool
        .push(make_item("/in/foo.mp4", None, ProxyStatus::Pending));

    let repo = MediaRepo::default();
    repo.reconcile_from_project(&project).unwrap();

    let items = repo.list().unwrap();
    assert_eq!(items[0].proxy_status, ProxyStatus::Pending);
}

#[test]
fn save_then_load_then_reconcile_round_trip() {
    let tmp = tempfile::tempdir().unwrap();
    let project_path = tmp.path().join("rt.vproj");
    let proxy_path = tmp.path().join("proxy.mp4");
    write_dummy(&proxy_path);

    let mut project = Project::new("RT".into());
    let item = make_item(
        "/in/foo.mp4",
        Some(proxy_path.to_string_lossy().as_ref()),
        ProxyStatus::Ready,
    );
    let item_id = item.id;
    project.media_pool.push(item);
    save_project(&project, &project_path).unwrap();

    let loaded = load_project(&project_path).unwrap();
    let repo = MediaRepo::default();
    repo.reconcile_from_project(&loaded).unwrap();

    let items = repo.list().unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].id, item_id);
    assert_eq!(items[0].proxy_status, ProxyStatus::Ready);
}
