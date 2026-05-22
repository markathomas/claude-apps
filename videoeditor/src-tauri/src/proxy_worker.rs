use std::path::PathBuf;

use tokio::sync::mpsc;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct ProxyJob {
    pub media_id: Uuid,
    pub source_path: PathBuf,
    pub proxy_path: PathBuf,
    pub thumbnails_dir: PathBuf,
    pub waveform_path: PathBuf,
    pub has_audio: bool,
    pub duration_ms: u64,
}

#[derive(Clone)]
pub struct ProxyWorkerHandle {
    sender: mpsc::UnboundedSender<ProxyJob>,
}

impl ProxyWorkerHandle {
    pub fn new(sender: mpsc::UnboundedSender<ProxyJob>) -> Self {
        Self { sender }
    }

    pub fn enqueue(&self, job: ProxyJob) -> Result<(), String> {
        self.sender.send(job).map_err(|e| e.to_string())
    }
}

pub fn channel() -> (ProxyWorkerHandle, mpsc::UnboundedReceiver<ProxyJob>) {
    let (tx, rx) = mpsc::unbounded_channel();
    (ProxyWorkerHandle::new(tx), rx)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_job() -> ProxyJob {
        ProxyJob {
            media_id: Uuid::new_v4(),
            source_path: PathBuf::from("/in.mp4"),
            proxy_path: PathBuf::from("/cache/p.mp4"),
            thumbnails_dir: PathBuf::from("/cache/thumbs/id"),
            waveform_path: PathBuf::from("/cache/wf/id.json"),
            has_audio: true,
            duration_ms: 5000,
        }
    }

    #[tokio::test]
    async fn enqueue_delivers_job_to_receiver() {
        let (handle, mut rx) = channel();
        let job = sample_job();
        handle.enqueue(job.clone()).unwrap();
        let received = rx.recv().await.unwrap();
        assert_eq!(received.media_id, job.media_id);
        assert_eq!(received.source_path, job.source_path);
    }

    #[tokio::test]
    async fn enqueue_fails_after_receiver_dropped() {
        let (handle, rx) = channel();
        drop(rx);
        let result = handle.enqueue(sample_job());
        assert!(result.is_err());
    }
}
