//! WASI has no path_symlink in rootshell. Retain ZIP links as metadata while
//! signing and recreate them over AFC. Placeholder bytes never enter a seal.
use std::{
    collections::BTreeMap,
    ffi::OsString,
    fs,
    io::{self, Read, Seek, SeekFrom, Write},
    path::{Component, Path, PathBuf},
    sync::{Mutex, OnceLock},
};

#[derive(Clone)]
struct Link {
    root: PathBuf,
    target: PathBuf,
}
fn links() -> &'static Mutex<BTreeMap<PathBuf, Link>> {
    static LINKS: OnceLock<Mutex<BTreeMap<PathBuf, Link>>> = OnceLock::new();
    LINKS.get_or_init(Default::default)
}
fn invalid() -> io::Error {
    io::Error::new(
        io::ErrorKind::InvalidData,
        "IPA symlink escapes its bundle, is cyclic, or has an invalid target",
    )
}
fn absolute(path: &Path) -> io::Result<PathBuf> {
    let path = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()?.join(path)
    };
    let mut result = PathBuf::new();
    for component in path.components() {
        match component {
            Component::ParentDir => {
                if !result.pop() {
                    return Err(invalid());
                }
            }
            Component::CurDir => {}
            other => result.push(other),
        }
    }
    Ok(result)
}
pub fn register_link(root: &Path, path: &Path, target: &Path) -> io::Result<()> {
    if target.is_absolute() || target.as_os_str().is_empty() {
        return Err(invalid());
    }
    let root = absolute(root)?;
    let path = absolute(path)?;
    if !path.starts_with(&root) || path == root {
        return Err(invalid());
    }
    let destination = absolute(&path.parent().ok_or_else(invalid)?.join(target))?;
    if !destination.starts_with(&root) {
        return Err(invalid());
    }
    links().lock().map_err(|_| invalid())?.insert(
        path,
        Link {
            root,
            target: target.to_path_buf(),
        },
    );
    Ok(())
}
pub fn validate_links(root: &Path) -> io::Result<()> {
    let root = absolute(root)?;
    let paths: Vec<_> = links()
        .lock()
        .map_err(|_| invalid())?
        .keys()
        .filter(|p| p.starts_with(&root))
        .cloned()
        .collect();
    for path in paths {
        let target = resolve(&path)?;
        if !target.starts_with(&root) || !target.exists() {
            return Err(invalid());
        }
    }
    Ok(())
}
pub fn clear_links(root: &Path) -> io::Result<()> {
    let root = absolute(root)?;
    links()
        .lock()
        .map_err(|_| invalid())?
        .retain(|path, _| !path.starts_with(&root));
    Ok(())
}
pub fn resolve(path: &Path) -> io::Result<PathBuf> {
    let mut path = absolute(path)?;
    let table = links().lock().map_err(|_| invalid())?;
    for _ in 0..64 {
        let mut prefix = PathBuf::new();
        let mut replacement = None;
        for component in path.components() {
            prefix.push(component);
            if let Some(link) = table.get(&prefix) {
                let suffix = path.strip_prefix(&prefix).map_err(|_| invalid())?;
                let next = absolute(
                    &prefix
                        .parent()
                        .ok_or_else(invalid)?
                        .join(&link.target)
                        .join(suffix),
                )?;
                if !next.starts_with(&link.root) {
                    return Err(invalid());
                }
                replacement = Some(next);
                break;
            }
        }
        match replacement {
            Some(next) => path = next,
            None => return Ok(path),
        }
    }
    Err(invalid())
}
pub fn read_link(path: impl AsRef<Path>) -> io::Result<PathBuf> {
    links()
        .lock()
        .map_err(|_| invalid())?
        .get(&absolute(path.as_ref())?)
        .map(|l| l.target.clone())
        .ok_or_else(invalid)
}
pub fn canonicalize(path: impl AsRef<Path>) -> io::Result<PathBuf> {
    super::perf::io_result(
        "file.canonicalize",
        |_| 0,
        || fs::canonicalize(resolve(path.as_ref())?),
    )
}
pub fn exists(path: impl AsRef<Path>) -> io::Result<bool> {
    Ok(resolve(path.as_ref())?.exists())
}
pub fn is_file(path: impl AsRef<Path>) -> bool {
    resolve(path.as_ref()).is_ok_and(|p| p.is_file())
}
pub fn read(path: impl AsRef<Path>) -> io::Result<Vec<u8>> {
    super::perf::io_result::<Vec<u8>>(
        "file.read",
        |bytes| bytes.len() as u64,
        || {
            let mut file = File::open(path)?;
            let expected = usize::try_from(file.metadata()?.len()).map_err(io::Error::other)?;
            let mut bytes = Vec::new();
            bytes
                .try_reserve_exact(expected)
                .map_err(io::Error::other)?;
            bytes.resize(expected, 0);
            let mut filled = 0;
            while filled < expected {
                match file.read(&mut bytes[filled..]) {
                    Ok(0) => {
                        bytes.truncate(filled);
                        return Ok(bytes);
                    }
                    Ok(count) => filled += count,
                    Err(error) if error.kind() == io::ErrorKind::Interrupted => continue,
                    Err(error) => return Err(error),
                }
            }
            // Avoid read_to_end's extra allocation when the size hint was
            // exact, while retaining normal read semantics for growing files.
            let mut extra = [0];
            loop {
                match file.read(&mut extra) {
                    Ok(0) => return Ok(bytes),
                    Ok(_) => {
                        bytes.try_reserve(1).map_err(io::Error::other)?;
                        bytes.push(extra[0]);
                        break;
                    }
                    Err(error) if error.kind() == io::ErrorKind::Interrupted => continue,
                    Err(error) => return Err(error),
                }
            }
            let mut tail = vec![0; super::MAX_HOST_READ];
            loop {
                match file.read(&mut tail) {
                    Ok(0) => break,
                    Ok(count) => {
                        bytes.try_reserve(count).map_err(io::Error::other)?;
                        bytes.extend_from_slice(&tail[..count]);
                    }
                    Err(error) if error.kind() == io::ErrorKind::Interrupted => continue,
                    Err(error) => return Err(error),
                }
            }
            Ok(bytes)
        },
    )
}
pub fn write(path: impl AsRef<Path>, bytes: impl AsRef<[u8]>) -> io::Result<()> {
    let bytes = bytes.as_ref();
    super::perf::io_result(
        "file.write",
        |()| bytes.len() as u64,
        || fs::write(resolve(path.as_ref())?, bytes),
    )
}
pub fn create_dir_all(path: impl AsRef<Path>) -> io::Result<()> {
    super::directory::create_all(resolve(path.as_ref())?)
}
pub fn remove_file(path: impl AsRef<Path>) -> io::Result<()> {
    fs::remove_file(resolve(path.as_ref())?)
}
pub fn remove_dir_all(path: impl AsRef<Path>) -> io::Result<()> {
    let path = absolute(path.as_ref())?;
    links()
        .lock()
        .map_err(|_| invalid())?
        .retain(|key, _| !key.starts_with(&path));
    fs::remove_dir_all(path)
}

pub struct Metadata {
    inner: fs::Metadata,
    link: bool,
}
pub struct FileType {
    inner: fs::FileType,
    link: bool,
}
impl Metadata {
    pub fn file_type(&self) -> FileType {
        FileType {
            inner: self.inner.file_type(),
            link: self.link,
        }
    }
    pub fn is_dir(&self) -> bool {
        !self.link && self.inner.is_dir()
    }
    pub fn is_file(&self) -> bool {
        !self.link && self.inner.is_file()
    }
    pub fn len(&self) -> u64 {
        self.inner.len()
    }
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}
impl FileType {
    pub fn is_symlink(&self) -> bool {
        self.link
    }
    pub fn is_dir(&self) -> bool {
        !self.link && self.inner.is_dir()
    }
    pub fn is_file(&self) -> bool {
        !self.link && self.inner.is_file()
    }
}
pub fn symlink_metadata(path: impl AsRef<Path>) -> io::Result<Metadata> {
    super::perf::io_result(
        "file.metadata",
        |_| 0,
        || {
            let path = absolute(path.as_ref())?;
            let link = links().lock().map_err(|_| invalid())?.contains_key(&path);
            Ok(Metadata {
                inner: fs::symlink_metadata(path)?,
                link,
            })
        },
    )
}
pub fn metadata(path: impl AsRef<Path>) -> io::Result<Metadata> {
    Ok(Metadata {
        inner: fs::metadata(resolve(path.as_ref())?)?,
        link: false,
    })
}
pub struct DirEntry(fs::DirEntry);
impl DirEntry {
    pub fn path(&self) -> PathBuf {
        self.0.path()
    }
    pub fn file_name(&self) -> OsString {
        self.0.file_name()
    }
    pub fn file_type(&self) -> io::Result<FileType> {
        Ok(symlink_metadata(self.path())?.file_type())
    }
}
pub struct ReadDir(fs::ReadDir);
impl Iterator for ReadDir {
    type Item = io::Result<DirEntry>;
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|r| r.map(DirEntry))
    }
}
pub fn read_dir(path: impl AsRef<Path>) -> io::Result<ReadDir> {
    super::perf::io_result(
        "file.read_dir",
        |_| 0,
        || Ok(ReadDir(fs::read_dir(resolve(path.as_ref())?)?)),
    )
}

pub struct File(fs::File);
impl File {
    pub fn metadata(&self) -> io::Result<fs::Metadata> {
        self.0.metadata()
    }
    pub fn open(path: impl AsRef<Path>) -> io::Result<Self> {
        super::perf::io_result(
            "file.open",
            |_| 0,
            || Ok(Self(fs::File::open(resolve(path.as_ref())?)?)),
        )
    }
}
impl Read for File {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let limit = buf.len().min(super::MAX_HOST_READ);
        super::perf::io_bytes("file.read_chunk", || self.0.read(&mut buf[..limit]))
    }
}
impl Write for File {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        super::perf::io_bytes("file.write_chunk", || self.0.write(buf))
    }
    fn flush(&mut self) -> io::Result<()> {
        self.0.flush()
    }
}
impl Seek for File {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        super::perf::io_result("file.seek", |_| 0, || self.0.seek(pos))
    }
}
