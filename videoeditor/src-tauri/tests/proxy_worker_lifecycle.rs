use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use videoeditor_lib::media_repo::MediaRepo;
use videoeditor_lib::model::project::ProxyStatus;
use videoeditor_lib::proxy_worker::{channel, run_worker_loop, EventEmitter, ProxyJob};

struct Capture(Mutex<Vec<(String, serde_json::Value)>>);

impl EventEmitter for Capture {
    fn emit(&self, name: &str, payload: serde_json::Value) {
        self.0.lock().unwrap().push((name.to_string(), payload));
    }
}

fn fixture_video() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/media/tiny.mp4")
}

#[tokio::test]
async fn worker_processes_one_job_end_to_end() {
    let tmp = tempfile::tempdir().unwrap();
    let repo = Arc::new(MediaRepo::default());
    let item = repo
        .add_pending(fixture_video().to_string_lossy().to_string())
        .unwrap();
    let captured = Arc::new(Capture(Mutex::new(Vec::new())));

    let (handle, rx) = channel();
    let job = ProxyJob {
        media_id: item.id,
        source_path: fixture_video(),
        proxy_path: tmp.path().join("proxy.mp4"),
        thumbnails_dir: tmp.path().join("thumbs"),
        waveform_path: tmp.path().join("wf.json"),
        has_audio: true,
        duration_ms: 1000,
    };
    handle.enqueue(job.clone()).unwrap();
    drop(handle); // close sender so loop exits when queue drains

    let captured_for_loop: Arc<dyn EventEmitter> = captured.clone();
    run_worker_loop(rx, repo.clone(), captured_for_loop).await;

    assert!(job.proxy_path.exists(), "proxy file missing");
    assert!(job.waveform_path.exists(), "waveform file missing");

    let updated = repo.get(item.id).unwrap().unwrap();
    assert_eq!(updated.proxy_status, ProxyStatus::Ready);

    let events = captured.0.lock().unwrap();
    assert!(events.iter().any(|(n, _)| n == "proxy_ready"));
}
