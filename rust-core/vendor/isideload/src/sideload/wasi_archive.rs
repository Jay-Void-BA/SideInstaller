//! Streaming IPA extraction with virtual symlinks for rootshell's WASI host.
use rootshell_runtime::vfs;
use std::{
    fs,
    io::{self, Read},
    path::{Path, PathBuf},
};

pub(super) fn extract<R: Read + io::Seek>(
    archive: &mut zip::ZipArchive<R>,
    root: &Path,
) -> io::Result<()> {
    let invalid = || {
        io::Error::new(
            io::ErrorKind::InvalidData,
            "invalid, oversized, or conflicting IPA archive entry",
        )
    };
    vfs::clear_links(root)?;
    if archive.len() > 100_000 {
        return Err(invalid());
    }
    let mut links = Vec::<(PathBuf, PathBuf)>::new();
    let mut expanded = 0u64;
    for index in 0..archive.len() {
        let mut file = archive.by_index(index).map_err(io::Error::other)?;
        let relative = file.enclosed_name().ok_or_else(invalid)?;
        if relative.as_os_str().is_empty() {
            return Err(invalid());
        }
        let output = root.join(relative);
        expanded = expanded.checked_add(file.size()).ok_or_else(invalid)?;
        if expanded > 4 * 1024 * 1024 * 1024 {
            return Err(invalid());
        }
        if file.is_dir() {
            fs::create_dir_all(output)?;
            continue;
        }
        if let Some(parent) = output.parent() {
            fs::create_dir_all(parent)?;
        }
        if file.is_symlink() {
            if file.size() == 0 || file.size() > 4096 {
                return Err(invalid());
            }
            let mut bytes = Vec::new();
            file.take(4097).read_to_end(&mut bytes)?;
            if bytes.len() > 4096 || bytes.contains(&0) {
                return Err(invalid());
            }
            let target = String::from_utf8(bytes).map_err(|_| invalid())?;
            links.push((output, PathBuf::from(target)));
        } else {
            let mut destination = fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(output)?;
            io::copy(&mut file, &mut destination)?;
        }
    }
    for (path, target) in &links {
        let mut placeholder = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(path)?;
        // No code path reads this placeholder as file contents. Retaining it
        // makes directory enumeration match the original IPA exactly.
        std::io::Write::write_all(&mut placeholder, b"")?;
        vfs::register_link(root, path, target)?;
    }
    vfs::validate_links(root)
}
