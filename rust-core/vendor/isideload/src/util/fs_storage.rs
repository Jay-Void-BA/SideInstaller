#[cfg(target_os = "wasi")]
use rootshell_runtime::vfs as storage_fs;
#[cfg(not(target_os = "wasi"))]
use std::fs as storage_fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, OnceLock, RwLock};

use rootcause::prelude::*;

use crate::util::storage::SideloadingStorage;

/// Optional authenticated encryption for a process-owned storage directory.
/// The caller owns its key and format. No key is written by FsStorage.
pub trait DataProtector: Send + Sync {
    fn seal(&self, key: &str, data: &[u8]) -> io::Result<Vec<u8>>;
    fn open(&self, key: &str, data: &[u8]) -> io::Result<Vec<u8>>;
}

type ProtectedDirectories = Vec<(PathBuf, Arc<dyn DataProtector>)>;
static PROTECTORS: OnceLock<RwLock<ProtectedDirectories>> = OnceLock::new();

pub struct FsStorage {
    path: PathBuf,
}

impl FsStorage {
    pub fn new(path: PathBuf) -> Self {
        let path = std::fs::canonicalize(&path).unwrap_or_else(|_| {
            if path.is_absolute() { path } else { std::env::current_dir().unwrap_or_default().join(path) }
        });
        FsStorage { path }
    }

    pub fn protect_directory(path: &Path, protector: Arc<dyn DataProtector>) -> io::Result<()> {
        let path = std::fs::canonicalize(path)?;
        let mut entries = PROTECTORS.get_or_init(Default::default).write().unwrap_or_else(|e| e.into_inner());
        entries.retain(|(root, _)| *root != path);
        entries.push((path, protector));
        Ok(())
    }

    pub fn unprotect_directory(path: &Path) -> io::Result<()> {
        let path = std::fs::canonicalize(path)?;
        if let Some(entries) = PROTECTORS.get() {
            entries.write().unwrap_or_else(|e| e.into_inner()).retain(|(root, _)| *root != path);
        }
        Ok(())
    }

    fn protector(&self, path: &Path) -> Option<(Arc<dyn DataProtector>, String)> {
        let entries = PROTECTORS.get()?.read().unwrap_or_else(|e| e.into_inner());
        entries.iter().filter_map(|(root, protector)| {
            let relative = path.strip_prefix(root).ok()?.to_str()?;
            Some((root.components().count(), Arc::clone(protector), relative.to_string()))
        }).max_by_key(|(depth, _, _)| *depth).map(|(_, protector, relative)| (protector, relative))
    }

    fn data_path(&self, key: &str) -> io::Result<PathBuf> {
        if key.is_empty() || Path::new(key).components().any(|c| !matches!(c, std::path::Component::Normal(_))) {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, "Invalid signing storage key"));
        }
        Ok(self.path.join(key))
    }
}

impl Default for FsStorage {
    fn default() -> Self { Self::new(PathBuf::from(".")) }
}

/// Private, atomic writes also protect newly created pairing and vault files.
pub fn write_private(path: &Path, data: &[u8]) -> io::Result<()> {
    let parent = path.parent().unwrap_or(Path::new("."));
    storage_fs::create_dir_all(parent)?;
    let staging = parent.join(format!(".siboot-{}.tmp", uuid::Uuid::new_v4()));
    let result = (|| {
        let mut options = std::fs::OpenOptions::new();
        options.write(true).create_new(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        let mut file = options.open(&staging)?;
        file.write_all(data)?;
        file.sync_all()?;
        std::fs::rename(&staging, path)
    })();
    if result.is_err() { let _ = std::fs::remove_file(staging); }
    result
}

impl SideloadingStorage for FsStorage {
    fn store_data(&self, key: &str, data: &[u8]) -> Result<(), Report> {
        let path = self.data_path(key).context("Invalid signing storage key")?;
        let encoded = self.protector(&path).map(|(protector, relative)| protector.seal(&relative, data)).transpose()
            .context("Couldn't encrypt signing data")?;
        write_private(&path, encoded.as_deref().unwrap_or(data)).context("Failed to write data to file")?;
        Ok(())
    }

    fn retrieve_data(&self, key: &str) -> Result<Option<Vec<u8>>, Report> {
        let path = self.data_path(key).context("Invalid signing storage key")?;
        match storage_fs::read(&path) {
            Ok(data) => match self.protector(&path) {
                Some((protector, relative)) => Ok(Some(protector.open(&relative, &data).context("Couldn't decrypt signing data")?)),
                None => Ok(Some(data)),
            },
            Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(report!(e).context("Failed to read data from file").into()),
        }
    }

    fn store(&self, key: &str, value: &str) -> Result<(), Report> { self.store_data(key, value.as_bytes()) }

    fn retrieve(&self, key: &str) -> Result<Option<String>, Report> {
        self.retrieve_data(key).map(|value| value.map(|bytes| String::from_utf8_lossy(&bytes).into_owned()))
    }
}
