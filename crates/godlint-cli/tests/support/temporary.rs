use std::{
    fs,
    io::ErrorKind,
    path::{Path, PathBuf},
    process,
    sync::atomic::{AtomicU64, Ordering},
};

static NEXT_ID: AtomicU64 = AtomicU64::new(0);

pub struct TemporaryDirectory {
    path: PathBuf,
}

impl TemporaryDirectory {
    pub fn new(prefix: &str) -> Self {
        loop {
            let candidate = candidate_path(prefix);

            match fs::create_dir(&candidate) {
                Ok(()) => return Self { path: candidate },
                Err(error) if error.kind() == ErrorKind::AlreadyExists => (),
                Err(error) => panic!("creates temporary directory: {error}"),
            }
        }
    }

    #[allow(dead_code)]
    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn write(&self, relative_path: &str, contents: &str) -> PathBuf {
        let path = self.path.join(relative_path);

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap_or_else(|error| panic!("creates parent: {error}"));
        }

        fs::write(&path, contents)
            .unwrap_or_else(|error| panic!("writes {relative_path}: {error}"));

        path
    }
}

impl Drop for TemporaryDirectory {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

fn candidate_path(prefix: &str) -> PathBuf {
    let process_id = process::id();
    let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);

    std::env::temp_dir().join(format!("godlint-{prefix}-{process_id}-{id}"))
}
