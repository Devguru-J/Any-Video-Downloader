use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::sync::oneshot;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadJob {
    pub id: String,
    pub url: String,
    pub title: String,
    pub format_id: String,
    pub output_dir: String,
    pub thumbnail: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum JobStatus {
    Queued,
    Running {
        downloaded: u64,
        total: u64,
        speed: f64,
        eta: f64,
    },
    Done {
        path: String,
    },
    Error {
        message: String,
    },
    Cancelled,
}

#[derive(Debug, Clone, Serialize)]
pub struct JobState {
    #[serde(flatten)]
    pub job: DownloadJob,
    pub status: JobStatus,
    pub started_at: u64,
    pub ended_at: Option<u64>,
}

/// Cheaply cloneable: only an Arc<Mutex<…>> inside.
#[derive(Clone)]
pub struct JobRegistry {
    inner: Arc<Mutex<RegistryInner>>,
}

struct RegistryInner {
    jobs: HashMap<String, JobState>,
    cancel: HashMap<String, oneshot::Sender<()>>,
}

impl JobRegistry {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(RegistryInner {
                jobs: HashMap::new(),
                cancel: HashMap::new(),
            })),
        }
    }

    pub fn list(&self) -> Vec<JobState> {
        self.inner.lock().unwrap().jobs.values().cloned().collect()
    }

    pub fn upsert(&self, state: JobState) {
        self.inner
            .lock()
            .unwrap()
            .jobs
            .insert(state.job.id.clone(), state);
    }

    pub fn register_cancel(&self, id: &str, tx: oneshot::Sender<()>) {
        self.inner
            .lock()
            .unwrap()
            .cancel
            .insert(id.to_string(), tx);
    }

    pub fn cancel(&self, id: &str) -> bool {
        if let Some(tx) = self.inner.lock().unwrap().cancel.remove(id) {
            let _ = tx.send(());
            true
        } else {
            false
        }
    }

    pub fn drop_cancel(&self, id: &str) {
        self.inner.lock().unwrap().cancel.remove(id);
    }

    pub fn clear_finished(&self) {
        let mut inner = self.inner.lock().unwrap();
        inner.jobs.retain(|_, state| {
            matches!(
                state.status,
                JobStatus::Queued | JobStatus::Running { .. }
            )
        });
    }
}

pub fn now_secs() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}
