use std::path::PathBuf;

use videoeditor_lib::project_io::{load_project, save_project};

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/projects")
        .join(name)
}

#[test]
fn loads_empty_v1_fixture() {
    let p = load_project(&fixture("empty_v1.vproj")).unwrap();
    assert_eq!(p.version, "1");
    assert_eq!(p.name, "Empty");
    assert!(p.media_pool.is_empty());
    assert!(p.timeline.video_track.is_empty());
}

#[test]
fn corrupt_fixture_is_rejected() {
    let err = load_project(&fixture("corrupt.vproj")).unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("corrupt"), "expected corrupt error, got: {msg}");
}

#[test]
fn save_then_load_round_trip_with_real_file() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("rt.vproj");
    let original = load_project(&fixture("empty_v1.vproj")).unwrap();
    save_project(&original, &path).unwrap();
    let loaded = load_project(&path).unwrap();
    assert_eq!(loaded, original);
}
