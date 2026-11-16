//! Streaming IPA extraction with virtual symlinks for rootshell's WASI host.
use rootshell_runtime::vfs;
use std::{
    collections::BTreeSet,
    fs,
    io::{self, Read, Write},
    path::{Path, PathBuf},
};

pub(super) fn extract<R: Read + io::Seek>(
    archive: &mut zip::ZipArchive<R>,
    root: &Path,
) -> io::Result<()> {
    let performance =
        rootshell_runtime::perf::span("ipa.extract", &[("entries", archive.len() as u64)]);
    rootshell_runtime::perf::checkpoint();
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
    // This extraction owns its fresh temporary tree. Reuse parents already
    // checked here instead of one WebKit metadata round trip for every file.
    // Never cache failures; create_new still rejects file/link conflicts.
    let mut directories = BTreeSet::new();
    let mut directory_checks_saved = 0u64;
    let mut ensure_directory = |directory: &Path| -> io::Result<()> {
        if directories.contains(directory) {
            directory_checks_saved += 1;
            return Ok(());
        }
        vfs::create_dir_all(directory)?;
        directories.insert(directory.to_path_buf());
        Ok(())
    };
    for index in 0..archive.len() {
        let mut file = archive.by_index(index).map_err(io::Error::other)?;
        let relative = file.enclosed_name().ok_or_else(invalid)?;
        if relative.as_os_str().is_empty() {
            return Err(invalid());
        }
        // Bound traversal depth and reject ambiguous separators before any
        // extraction or signing work is performed on an untrusted ZIP entry.
        if relative.components().count() > 128 || relative.to_string_lossy().contains('\\') {
            return Err(invalid());
        }
        let output = root.join(relative);
        expanded = expanded.checked_add(file.size()).ok_or_else(invalid)?;
        if expanded > 4 * 1024 * 1024 * 1024 {
            return Err(invalid());
        }
        if file.is_dir() {
            ensure_directory(&output)?;
            continue;
        }
        if let Some(parent) = output.parent() {
            ensure_directory(parent)?;
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
            let destination = fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(output)?;
            // Deflate emits small writes. Coalesce them before crossing the
            // WebKit bridge; write replies contain only a byte count.
            let mut destination = io::BufWriter::with_capacity(64 * 1024, destination);
            io::copy(&mut file, &mut destination)?;
            destination.flush()?;
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
    vfs::validate_links(root)?;
    performance.finish(
        rootshell_runtime::perf::Outcome::Ok,
        &[
            ("expanded_bytes", expanded),
            ("links", links.len() as u64),
            ("directories", directories.len() as u64),
            ("directory_checks_saved", directory_checks_saved),
        ],
    );
    Ok(())
}
