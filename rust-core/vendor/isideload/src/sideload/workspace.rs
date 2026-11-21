//! A private extraction directory shared by the signer and its caller.
//! Keeping it in an Arc also keeps background extraction alive on cancellation.
use std::{
    fs, io,
    path::{Path, PathBuf},
    sync::Arc,
};

#[derive(Debug)]
pub struct ExtractionDirectory(PathBuf);

impl ExtractionDirectory {
    pub fn new() -> io::Result<Arc<Self>> {
        let path = std::env::temp_dir().join(format!("siboot-sign-{}", uuid::Uuid::new_v4()));
        let mut builder = fs::DirBuilder::new();
        #[cfg(unix)]
        {
            use std::os::unix::fs::DirBuilderExt;
            builder.mode(0o700);
        }
        builder.create(&path)?;
        Ok(Arc::new(Self(path)))
    }

    pub fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for ExtractionDirectory {
    fn drop(&mut self) {
        // This exact directory was exclusively created above. remove_dir_all
        // does not traverse a replacement symlink to another directory.
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn private_workspace_lives_until_the_last_owner_drops() {
        let owner = ExtractionDirectory::new().unwrap();
        let path = owner.path().to_path_buf();
        fs::write(path.join("keep"), b"signed app").unwrap();
        let background = owner.clone();
        drop(owner);
        assert!(path.join("keep").exists());
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            assert_eq!(
                fs::metadata(&path).unwrap().permissions().mode() & 0o777,
                0o700
            );
        }
        drop(background);
        assert!(!path.exists());
    }
}
