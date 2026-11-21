//! Bounded native IPA extraction. Files are written before any symlink exists,
//! then the complete link graph is checked before links are materialized.
use std::{
    collections::{BTreeMap, BTreeSet, VecDeque},
    fs,
    io::{self, Read, Write},
    path::{Component, Path, PathBuf},
};

fn invalid() -> io::Error {
    io::Error::new(
        io::ErrorKind::InvalidData,
        "Invalid, oversized, conflicting or escaping IPA entry",
    )
}

fn normalize(path: &Path) -> io::Result<PathBuf> {
    let mut result = PathBuf::new();
    for part in path.components() {
        match part {
            Component::Normal(name) => {
                if name.to_string_lossy().contains(['\\', '\0']) {
                    return Err(invalid());
                }
                result.push(name);
            }
            Component::CurDir => {}
            Component::ParentDir if result.pop() => {}
            _ => return Err(invalid()),
        }
    }
    Ok(result)
}

fn resolve(path: &Path, links: &BTreeMap<PathBuf, PathBuf>) -> io::Result<PathBuf> {
    let mut pending: VecDeque<_> = path
        .components()
        .map(|part| part.as_os_str().to_os_string())
        .collect();
    let mut resolved = PathBuf::new();
    let mut followed = 0;
    while let Some(part) = pending.pop_front() {
        if part == "." {
            continue;
        }
        if part == ".." {
            if !resolved.pop() {
                return Err(invalid());
            }
            continue;
        }
        let candidate = resolved.join(&part);
        if let Some(target) = links.get(&candidate) {
            followed += 1;
            if followed > 64 {
                return Err(invalid());
            }
            // Resolve a symlink before interpreting a following `..`, exactly
            // as the filesystem does. Lexically normalizing first can escape.
            for component in target.components().rev() {
                match component {
                    Component::Normal(_) | Component::ParentDir | Component::CurDir => {
                        pending.push_front(component.as_os_str().to_os_string());
                    }
                    _ => return Err(invalid()),
                }
            }
        } else {
            resolved.push(part);
        }
    }
    Ok(resolved)
}

pub(super) fn extract<R: Read + io::Seek>(
    archive: &mut zip::ZipArchive<R>,
    root: &Path,
) -> io::Result<()> {
    #[cfg(feature = "performance")]
    let performance = siboot_perf::span("ipa.extract", &[("entries", archive.len() as u64)]);
    if archive.len() > 100_000 {
        return Err(invalid());
    }
    let canonical_root = root.canonicalize()?;
    let mut directories = BTreeSet::new();
    let mut saved = 0u64;
    let mut ensure_directory = |path: &Path| -> io::Result<()> {
        if directories.contains(path) {
            saved += 1;
            return Ok(());
        }
        fs::create_dir_all(path)?;
        directories.insert(path.to_path_buf());
        Ok(())
    };
    let mut links = BTreeMap::new();
    let mut expanded = 0u64;
    for index in 0..archive.len() {
        let file = archive.by_index(index).map_err(io::Error::other)?;
        let enclosed = file.enclosed_name().ok_or_else(invalid)?;
        if enclosed.components().count() > 128 {
            return Err(invalid());
        }
        let relative = normalize(&enclosed)?;
        if relative.as_os_str().is_empty() {
            return Err(invalid());
        }
        let output = root.join(&relative);
        let size = file.size();
        expanded = expanded.checked_add(size).ok_or_else(invalid)?;
        if expanded > 4 * 1024 * 1024 * 1024 {
            return Err(invalid());
        }
        let kind = file.unix_mode().unwrap_or(0) & 0o170000;
        if !matches!(kind, 0 | 0o040000 | 0o100000 | 0o120000) {
            return Err(invalid());
        }
        if file.is_dir() {
            ensure_directory(&output)?;
            continue;
        }
        ensure_directory(output.parent().ok_or_else(invalid)?)?;
        if file.is_symlink() {
            if size == 0 || size > 4096 {
                return Err(invalid());
            }
            let mut bytes = Vec::new();
            file.take(4097).read_to_end(&mut bytes)?;
            if bytes.len() as u64 != size || bytes.contains(&0) {
                return Err(invalid());
            }
            let target = PathBuf::from(String::from_utf8(bytes).map_err(|_| invalid())?);
            if target.is_absolute() || target.to_string_lossy().contains('\\') {
                return Err(invalid());
            }
            if links.insert(relative, target).is_some() {
                return Err(invalid());
            }
        } else {
            let executable = file.unix_mode().unwrap_or(0) & 0o111 != 0;
            let destination = fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&output)?;
            let mut destination = io::BufWriter::with_capacity(256 * 1024, destination);
            let count = io::copy(&mut file.take(size + 1), &mut destination)?;
            if count != size {
                return Err(invalid());
            }
            destination.flush()?;
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                fs::set_permissions(
                    &output,
                    fs::Permissions::from_mode(if executable { 0o700 } else { 0o600 }),
                )?;
            }
            #[cfg(not(unix))]
            let _ = executable;
        }
    }
    for path in links.keys() {
        let target = root.join(resolve(path, &links)?).canonicalize()?;
        if !target.starts_with(&canonical_root) {
            return Err(invalid());
        }
    }
    for (path, target) in &links {
        #[cfg(unix)]
        std::os::unix::fs::symlink(target, root.join(path))?;
        #[cfg(not(unix))]
        {
            let _ = (path, target);
            return Err(io::Error::new(
                io::ErrorKind::Unsupported,
                "IPA symlinks require a Unix filesystem",
            ));
        }
    }
    #[cfg(feature = "performance")]
    performance.finish(
        siboot_perf::Outcome::Ok,
        &[
            ("expanded_bytes", expanded),
            ("links", links.len() as u64),
            ("directories", directories.len() as u64),
            ("directory_checks_saved", saved),
        ],
    );
    #[cfg(not(feature = "performance"))]
    let _ = saved;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sideload::workspace::ExtractionDirectory;
    fn archive(
        files: &[(&str, &[u8])],
        links: &[(&str, &str)],
    ) -> zip::ZipArchive<io::Cursor<Vec<u8>>> {
        let mut writer = zip::ZipWriter::new(io::Cursor::new(Vec::new()));
        for (name, bytes) in files {
            writer
                .start_file(*name, zip::write::SimpleFileOptions::default())
                .unwrap();
            writer.write_all(bytes).unwrap();
        }
        for (name, target) in links {
            writer
                .add_symlink(*name, *target, zip::write::SimpleFileOptions::default())
                .unwrap();
        }
        zip::ZipArchive::new(writer.finish().unwrap()).unwrap()
    }
    #[test]
    fn preserves_hidden_resources_and_resolves_versioned_framework_links() {
        let workspace = ExtractionDirectory::new().unwrap();
        let mut zip = archive(
            &[("Framework/Versions/A/.config", b"resource")],
            &[
                ("Framework/Versions/Current", "A"),
                ("Framework/Resources", "Versions/Current"),
            ],
        );
        extract(&mut zip, workspace.path()).unwrap();
        assert_eq!(
            fs::read(workspace.path().join("Framework/Resources/.config")).unwrap(),
            b"resource"
        );
        assert_eq!(
            fs::read_link(workspace.path().join("Framework/Versions/Current")).unwrap(),
            Path::new("A")
        );
    }
    #[test]
    fn never_follows_an_archive_link_while_writing_files() {
        let outside = ExtractionDirectory::new().unwrap();
        let sentinel = outside.path().join("sentinel");
        fs::write(&sentinel, b"preserved").unwrap();
        let workspace = ExtractionDirectory::new().unwrap();
        let target = outside.path().to_str().unwrap();
        let mut zip = archive(&[("link/sentinel", b"malicious")], &[("link", target)]);
        assert!(extract(&mut zip, workspace.path()).is_err());
        assert_eq!(fs::read(sentinel).unwrap(), b"preserved");
        for links in [vec![("a", "b"), ("b", "a")], vec![("a", "../../outside")]] {
            let workspace = ExtractionDirectory::new().unwrap();
            assert!(extract(&mut archive(&[], &links), workspace.path()).is_err());
        }
    }

    #[test]
    fn resolves_links_before_parent_components() {
        let workspace = ExtractionDirectory::new().unwrap();
        let mut zip = archive(
            &[("outside", b"lexical decoy"), ("dir/keep", b"directory")],
            &[("dir/deep", ".."), ("escape", "dir/deep/../../outside")],
        );
        assert!(extract(&mut zip, workspace.path()).is_err());
        assert!(!workspace.path().join("escape").exists());
    }
}
