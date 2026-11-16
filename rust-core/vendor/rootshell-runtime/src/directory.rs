//! Directory creation compatible with rootshell's filesystem broker.
//!
//! The broker reports an existing directory as EIO rather than EEXIST. Check
//! metadata first, and only accept a failed mkdir when a subsequent stat proves
//! that another caller created a directory. Files and real I/O errors survive.
use std::{fs, io, path::Path};

pub fn create_all(path: impl AsRef<Path>) -> io::Result<()> {
    let path = path.as_ref();
    if path.as_os_str().is_empty() {
        return Ok(());
    }
    let mut missing = Vec::new();
    let mut cursor = path;
    loop {
        match fs::metadata(cursor) {
            Ok(info) if info.is_dir() => break,
            Ok(_) => {
                return Err(io::Error::new(
                    io::ErrorKind::AlreadyExists,
                    "A file occupies the requested directory",
                ))
            }
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                missing.push(cursor);
                match cursor.parent() {
                    Some(parent) if !parent.as_os_str().is_empty() => cursor = parent,
                    _ => break,
                }
            }
            Err(error) => return Err(error),
        }
    }
    for directory in missing.into_iter().rev() {
        if let Err(error) = fs::create_dir(directory) {
            if !fs::metadata(directory).is_ok_and(|info| info.is_dir()) {
                return Err(error);
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn reuse_directories_but_never_replace_a_file() {
        let stamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root =
            std::env::temp_dir().join(format!("siboot-directory-{}-{stamp}", std::process::id()));
        let nested = root.join("first/second");
        super::create_all(&nested).unwrap();
        super::create_all(&nested).unwrap();
        let file = root.join("occupied");
        std::fs::write(&file, b"keep me").unwrap();
        assert!(super::create_all(file.join("child")).is_err());
        assert!(super::create_all(&file).is_err());
        assert_eq!(std::fs::read(&file).unwrap(), b"keep me");
        std::fs::remove_dir_all(root).unwrap();
    }
}
