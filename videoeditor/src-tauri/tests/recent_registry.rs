use std::path::Path;

use videoeditor_lib::recent::RecentRegistry;

#[test]
fn end_to_end_touch_save_load() {
    let tmp = tempfile::tempdir().unwrap();
    let registry_path = tmp.path().join("recent.json");

    let r = RecentRegistry::load(&registry_path).unwrap();
    assert!(r.items.is_empty());

    let mut r = RecentRegistry::default();
    r.touch(Path::new("/projects/one.vproj"), "One");
    r.touch(Path::new("/projects/two.vproj"), "Two");
    r.save(&registry_path).unwrap();

    let reloaded = RecentRegistry::load(&registry_path).unwrap();
    assert_eq!(reloaded.items.len(), 2);
    assert_eq!(reloaded.items[0].path, "/projects/two.vproj");
    assert_eq!(reloaded.items[1].path, "/projects/one.vproj");
}
