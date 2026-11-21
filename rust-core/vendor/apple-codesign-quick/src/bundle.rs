use crate::error::{CodeSignError, Result};
use crate::file_bytes::read_file_bytes;
use crate::macho::{DEFAULT_CMS_BLOB_RESERVATION, MachOSigningConfig, sign_macho_file};
use crate::signature::CmsSigner;
use plist::{Dictionary, Value};
use crate::parallel::prelude::*;
use sha1::Digest as _;
use std::collections::BTreeMap;
#[cfg(not(target_os = "wasi"))]
use std::fs;
#[cfg(target_os = "wasi")]
use rootshell_runtime::vfs as fs;
use std::io::Read;
use std::path::{Path, PathBuf};

const FAIRPLAY_DIR: &str = "SC_Info";
const CODE_SIGNATURE_DIR: &str = "_CodeSignature";
const CODE_RESOURCES_FILE: &str = "CodeResources";

pub struct BundleSigningSettings<'a> {
    pub team_id: String,
    pub main_entitlements: Dictionary,
    pub entitlements_by_bundle_id: BTreeMap<String, Dictionary>,
    pub cms_signer: Option<&'a dyn CmsSigner>,
    pub embedded_mobileprovision: Option<&'a [u8]>,
    pub embedded_mobileprovisions_by_bundle_id: BTreeMap<String, &'a [u8]>,
    pub cms_blob_reservation: usize,
}

impl<'a> BundleSigningSettings<'a> {
    pub fn new(
        team_id: impl Into<String>,
        main_entitlements: Dictionary,
        cms_signer: Option<&'a dyn CmsSigner>,
    ) -> Self {
        Self {
            team_id: team_id.into(),
            main_entitlements,
            entitlements_by_bundle_id: BTreeMap::new(),
            cms_signer,
            embedded_mobileprovision: None,
            embedded_mobileprovisions_by_bundle_id: BTreeMap::new(),
            cms_blob_reservation: DEFAULT_CMS_BLOB_RESERVATION,
        }
    }
}

#[derive(Debug)]
pub struct Bundle {
    path: PathBuf,
    app_info: Dictionary,
}

impl Bundle {
    pub fn open(path: impl Into<PathBuf>) -> Result<Self> {
        let path = path.into();
        let info_path = path.join("Info.plist");
        if !fs::exists(&info_path).unwrap_or(false) {
            return Err(CodeSignError::MissingInfoPlist(info_path));
        }
        let root = fs::canonicalize(&path).map_err(|source| CodeSignError::io(&path, source))?;
        let info_target = fs::canonicalize(&info_path).map_err(|source| CodeSignError::io(&info_path, source))?;
        if !info_target.starts_with(root) { return Err(CodeSignError::macho(&info_path, "Info.plist escapes its bundle")); }

        let file =
            fs::File::open(&info_path).map_err(|source| CodeSignError::io(&info_path, source))?;
        let app_info = match Value::from_reader(file)? {
            Value::Dictionary(dict) => dict,
            _ => {
                return Err(CodeSignError::macho(
                    &info_path,
                    "Info.plist is not a dictionary",
                ));
            }
        };

        Ok(Self { path, app_info })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn bundle_identifier(&self) -> Result<&str> {
        self.info_string("CFBundleIdentifier")
    }

    pub fn executable_name(&self) -> Result<&str> {
        let name = self.info_string("CFBundleExecutable")?;
        if name.is_empty() || Path::new(name).components().any(|c| !matches!(c, std::path::Component::Normal(_))) {
            return Err(CodeSignError::macho(&self.path, "invalid executable path"));
        }
        let image = fs::canonicalize(self.path.join(name)).map_err(|source| CodeSignError::io(self.path.join(name), source))?;
        let root = fs::canonicalize(&self.path).map_err(|source| CodeSignError::io(&self.path, source))?;
        if !image.starts_with(root) { return Err(CodeSignError::macho(&self.path, "executable escapes its bundle")); }
        Ok(name)
    }

    pub fn sign(&mut self, settings: &BundleSigningSettings<'_>) -> Result<CodeResourcesMaps> {
        self.sign_inner(settings, true)
    }

    fn sign_inner(
        &mut self,
        settings: &BundleSigningSettings<'_>,
        is_root: bool,
    ) -> Result<CodeResourcesMaps> {
        #[cfg(any(target_os = "wasi", feature = "performance"))]
        let performance = siboot_perf::span("sign.bundle", &[("root", u64::from(is_root))]);
        remove_fairplay_dir(&self.path)?;

        let bundle_id = self.bundle_identifier()?.to_string();
        let executable = self.executable_name()?.to_string();
        let sub_bundles = self.sub_bundles()?;
        let sub_bundles_for_hashing = sub_bundles.clone();

        let code_signature_dir = self.path.join(CODE_SIGNATURE_DIR);
        if code_signature_dir.exists() {
            fs::remove_dir_all(&code_signature_dir).map_err(|source| CodeSignError::io(&code_signature_dir, source))?;
        }
        fs::create_dir_all(&code_signature_dir)
            .map_err(|source| CodeSignError::io(&code_signature_dir, source))?;

        let code_resources_path = code_signature_dir.join(CODE_RESOURCES_FILE);
        if code_resources_path.exists() {
            fs::remove_file(&code_resources_path)
                .map_err(|source| CodeSignError::io(&code_resources_path, source))?;
        }

        let info_data = encode_binary_plist(&self.app_info)?;
        let info_path = self.path.join("Info.plist");
        fs::write(&info_path, &info_data)
            .map_err(|source| CodeSignError::io(&info_path, source))?;
        let mobileprovision = settings
            .embedded_mobileprovisions_by_bundle_id
            .get(&bundle_id)
            .copied()
            .or_else(|| {
                if is_root {
                    settings.embedded_mobileprovision
                } else {
                    None
                }
            });
        if let Some(mobileprovision) = mobileprovision {
            let profile_path = self.path.join("embedded.mobileprovision");
            fs::write(&profile_path, mobileprovision)
                .map_err(|source| CodeSignError::io(&profile_path, source))?;
        }

        let (sub_results, bundle_files) = crate::parallel::join(
            || sign_sub_bundles(&self.path, sub_bundles, settings),
            || collect_own_bundle_files(&self.path, &executable, &sub_bundles_for_hashing),
        );
        let sub_results = sub_results?;
        let bundle_files = bundle_files?;
        sign_libraries(&bundle_files.libraries, settings)?;
        let resource_hashes = hash_resource_files(&self.path, &bundle_files.resource_files)?;

        let mut maps = CodeResourcesMaps::default();

        for sub in &sub_results {
            merge_sub_bundle_resources(&mut maps, &self.path, sub)?;
        }

        for hash in resource_hashes {
            maps.insert_hash(hash);
        }

        let code_resources = encode_code_resources(&maps)?;
        fs::write(&code_resources_path, &code_resources)
            .map_err(|source| CodeSignError::io(&code_resources_path, source))?;

        let empty_entitlements = Dictionary::new();
        let bundle_entitlements = settings
            .entitlements_by_bundle_id
            .get(&bundle_id)
            .or_else(|| is_root.then_some(&settings.main_entitlements))
            .unwrap_or(&empty_entitlements);

        let mut config = MachOSigningConfig::new(
            &bundle_id,
            &settings.team_id,
            bundle_entitlements,
            settings.cms_signer,
        );
        config.cms_blob_reservation = settings.cms_blob_reservation;
        config.info_plist = Some(&info_data);
        config.code_resources = Some(&code_resources);
        sign_macho_file(&self.path.join(&executable), &config)?;

        #[cfg(any(target_os = "wasi", feature = "performance"))]
        performance.finish(siboot_perf::Outcome::Ok, &[("resources", maps.files2.len() as u64)]);
        Ok(maps)
    }

    fn info_string(&self, key: &'static str) -> Result<&str> {
        self.app_info
            .get(key)
            .and_then(Value::as_string)
            .ok_or(CodeSignError::MissingInfoString(key))
    }

    fn sub_bundles(&self) -> Result<Vec<PathBuf>> {
        #[cfg(any(target_os = "wasi", feature = "performance"))]
        let performance = siboot_perf::span("sign.discover_bundles", &[]);
        let mut bundles = Vec::new();
        let mut pending = vec![self.path.clone()];
        while let Some(dir) = pending.pop() {
            for entry in fs::read_dir(&dir).map_err(|source| CodeSignError::io(&dir, source))? {
                let entry = entry.map_err(|source| CodeSignError::io(&dir, source))?;
                let path = entry.path();
                if !entry.file_type().map_err(|source| CodeSignError::io(&path, source))?.is_dir()
                    || path == self.path.join(CODE_SIGNATURE_DIR) || path == self.path.join(FAIRPLAY_DIR) { continue; }
                let info = path.join("Info.plist");
                let known_code_bundle = matches!(path.extension().and_then(|e| e.to_str()), Some("app" | "appex" | "framework" | "xpc"));
                let executable_bundle = if fs::metadata(&info).is_ok_and(|m| m.is_file()) {
                    match plist::from_file::<_, Dictionary>(fs::canonicalize(&info).map_err(|source| CodeSignError::io(&info, source))?) {
                        Ok(dict) => dict.get("CFBundleExecutable").and_then(Value::as_string)
                            .is_some_and(|name| fs::metadata(path.join(name)).is_ok_and(|m| m.is_file())),
                        Err(error) if known_code_bundle => return Err(error.into()),
                        Err(_) => false,
                    }
                } else { false };
                if known_code_bundle || executable_bundle {
                    // The child owns all code and resources beneath it. It
                    // discovers its own children before signing its main image.
                    bundles.push(path);
                } else {
                    pending.push(path);
                }
            }
        }
        bundles.sort();
        #[cfg(any(target_os = "wasi", feature = "performance"))]
        performance.finish(siboot_perf::Outcome::Ok, &[("bundles", bundles.len() as u64)]);
        Ok(bundles)
    }
}

#[derive(Debug)]
struct SubBundleResult {
    path: PathBuf,
    relative_root: String,
    executable: String,
    resources: CodeResourcesMaps,
}

#[derive(Clone, Debug)]
struct FileHash {
    relative_path: String,
    sha1: [u8; 20],
    sha256: [u8; 32],
    optional: bool,
    symlink: Option<String>,
}

#[derive(Debug)]
struct BundleFiles {
    resource_files: Vec<PathBuf>,
    libraries: Vec<PathBuf>,
}

fn merge_sub_bundle_resources(
    maps: &mut CodeResourcesMaps,
    parent_path: &Path,
    sub: &SubBundleResult,
) -> Result<()> {
    maps.merge_rerooted(&sub.relative_root, &sub.resources);
    maps.insert_hash(hash_bundle_file(
        parent_path,
        &sub.path.join(&sub.executable),
    )?);
    let executable = sub.path.join(&sub.executable);
    if fs::symlink_metadata(&executable).map_err(|source| CodeSignError::io(&executable, source))?.file_type().is_symlink() {
        let target = fs::canonicalize(&executable).map_err(|source| CodeSignError::io(&executable, source))?;
        let root = fs::canonicalize(parent_path).map_err(|source| CodeSignError::io(parent_path, source))?;
        let relative = target.strip_prefix(root).map_err(|_| CodeSignError::macho(&target, "nested executable escapes its bundle"))?;
        maps.insert_hash(hash_bundle_file(parent_path, &parent_path.join(relative))?);
    }
    maps.insert_hash(hash_bundle_file(
        parent_path,
        &sub.path.join(CODE_SIGNATURE_DIR).join(CODE_RESOURCES_FILE),
    )?);
    Ok(())
}

#[derive(Clone, Debug, Default)]
pub struct CodeResourcesMaps {
    pub files: Dictionary,
    pub files2: Dictionary,
    hashes: Vec<FileHash>,
}

impl CodeResourcesMaps {
    fn insert_hash(&mut self, hash: FileHash) {
        self.hashes.push(hash.clone());
        if let Some(target) = hash.symlink {
            let mut entry = Dictionary::new();
            entry.insert("symlink".into(), Value::String(target));
            self.files2.insert(hash.relative_path, Value::Dictionary(entry));
            return;
        }
        let sha1 = hash.sha1.to_vec();
        if !omit_from_files(&hash.relative_path) {
            if hash.optional {
                let mut file = Dictionary::new();
                file.insert("hash".to_string(), Value::Data(sha1.clone()));
                file.insert("optional".to_string(), Value::Boolean(true));
                self.files
                    .insert(hash.relative_path.clone(), Value::Dictionary(file));
            } else {
                self.files
                    .insert(hash.relative_path.clone(), Value::Data(sha1.clone()));
            }
        }

        if !omit_from_files2(&hash.relative_path) {
            let mut file2 = Dictionary::new();
            file2.insert("hash".to_string(), Value::Data(sha1));
            file2.insert("hash2".to_string(), Value::Data(hash.sha256.to_vec()));
            if hash.optional {
                file2.insert("optional".to_string(), Value::Boolean(true));
            }
            self.files2
                .insert(hash.relative_path, Value::Dictionary(file2));
        }
    }

    fn merge_rerooted(&mut self, root: &str, other: &Self) {
        for hash in &other.hashes {
            let mut hash = hash.clone();
            hash.relative_path = join_relative(root, &hash.relative_path);
            self.insert_hash(hash);
        }
    }
}

fn omit_from_files(relative_path: &str) -> bool {
    relative_path.ends_with(".lproj/locversion.plist")
}

fn omit_from_files2(relative_path: &str) -> bool {
    omit_from_files(relative_path)
        || relative_path == "Info.plist"
        || relative_path == "PkgInfo"
        || relative_path == ".DS_Store"
        || relative_path.ends_with("/.DS_Store")
}

pub fn sign_bundle(path: impl Into<PathBuf>, settings: &BundleSigningSettings<'_>) -> Result<()> {
    let mut bundle = Bundle::open(path)?;
    bundle.sign(settings)?;
    verify_bundle_resources(bundle.path())?;
    Ok(())
}

fn sign_sub_bundles(
    root_path: &Path,
    sub_bundles: Vec<PathBuf>,
    settings: &BundleSigningSettings<'_>,
) -> Result<Vec<SubBundleResult>> {
    sub_bundles
        .into_par_iter()
        .map(|path| {
            let mut bundle = Bundle::open(&path)?;
            let result = bundle.sign_inner(settings, false)?;
            let relative_root = relative_path_string(root_path, &path)?;
            let executable = bundle.executable_name()?.to_string();
            Ok(SubBundleResult {
                path,
                relative_root,
                executable,
                resources: result,
            })
        })
        .collect()
}

fn collect_own_bundle_files(
    bundle_path: &Path,
    executable: &str,
    sub_bundles: &[PathBuf],
) -> Result<BundleFiles> {
    #[cfg(any(target_os = "wasi", feature = "performance"))]
    let performance = siboot_perf::span("sign.scan", &[]);
    let mut resource_files = Vec::new();
    let mut libraries = Vec::new();
    let main_image = fs::canonicalize(bundle_path.join(executable)).ok();

    walk_files_pruned(
        bundle_path,
        &mut |dir| !sub_bundles.iter().any(|sub_bundle| sub_bundle == dir)
            && dir != bundle_path.join(CODE_SIGNATURE_DIR) && dir != bundle_path.join(FAIRPLAY_DIR),
        &mut |path| {
            let relative = relative_path_string(bundle_path, path)?;
            let canonical = fs::canonicalize(path).map_err(|source| CodeSignError::io(path, source))?;
            let root = fs::canonicalize(bundle_path).map_err(|source| CodeSignError::io(bundle_path, source))?;
            if !canonical.starts_with(&root) { return Err(CodeSignError::macho(path, "symlink escapes the app bundle")); }
            let symbolic = fs::symlink_metadata(path).map_err(|source| CodeSignError::io(path, source))?.file_type().is_symlink();
            let is_main = main_image.as_ref() == Some(&canonical);
            if !is_main && !symbolic && (looks_like_macho(path)? || path.extension().and_then(|value| value.to_str()) == Some("dylib")) {
                libraries.push(path.to_path_buf());
            }
            if !is_main && should_hash_resource(&relative, executable) {
                resource_files.push(path.to_path_buf());
            }
            Ok(())
        },
    )?;

    resource_files.sort();
    libraries.sort();

    #[cfg(any(target_os = "wasi", feature = "performance"))]
    performance.finish(siboot_perf::Outcome::Ok, &[("resources", resource_files.len() as u64), ("libraries", libraries.len() as u64)]);
    Ok(BundleFiles {
        resource_files,
        libraries,
    })
}

fn sign_libraries(libraries: &[PathBuf], settings: &BundleSigningSettings<'_>) -> Result<()> {
    libraries
        .par_iter()
        .map(|path| {
            let identifier = path
                .file_name()
                .and_then(|value| value.to_str())
                .ok_or_else(|| CodeSignError::InvalidBundlePath(path.clone()))?;
            let entitlements = Dictionary::new();
            let mut config = MachOSigningConfig::new(
                identifier,
                &settings.team_id,
                &entitlements,
                settings.cms_signer,
            );
            config.cms_blob_reservation = settings.cms_blob_reservation;
            sign_macho_file(path, &config)
        })
        .collect::<Result<Vec<_>>>()?;
    Ok(())
}

fn hash_resource_files(bundle_path: &Path, resource_files: &[PathBuf]) -> Result<Vec<FileHash>> {
    resource_files
        .par_iter()
        .map(|path| hash_bundle_file(bundle_path, path))
        .collect()
}

fn remove_fairplay_dir(bundle_path: &Path) -> Result<()> {
    let fairplay_path = bundle_path.join(FAIRPLAY_DIR);
    if fairplay_path.exists() {
        fs::remove_dir_all(&fairplay_path)
            .map_err(|source| CodeSignError::io(&fairplay_path, source))?;
    }
    Ok(())
}

fn walk_files_pruned(
    root: &Path,
    should_descend: &mut impl FnMut(&Path) -> bool,
    visitor: &mut impl FnMut(&Path) -> Result<()>,
) -> Result<()> {
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        for entry in fs::read_dir(&dir).map_err(|source| CodeSignError::io(&dir, source))? {
            let entry = entry.map_err(|source| CodeSignError::io(&dir, source))?;
            let path = entry.path();
            let file_type = entry
                .file_type()
                .map_err(|source| CodeSignError::io(&path, source))?;
            if file_type.is_dir() && should_descend(&path) {
                stack.push(path);
            } else if file_type.is_file() || file_type.is_symlink() {
                visitor(&path)?;
            }
        }
    }
    Ok(())
}

fn should_hash_resource(relative: &str, executable: &str) -> bool {
    relative != executable && relative != FAIRPLAY_DIR && !relative.starts_with("SC_Info/")
        && !relative.starts_with("_CodeSignature/")
}

fn looks_like_macho(path: &Path) -> Result<bool> {
    let mut file = fs::File::open(path).map_err(|source| CodeSignError::io(path, source))?;
    let mut header = [0u8; 12];
    let len = file.read(&mut header).map_err(|source| CodeSignError::io(path, source))?;
    if len < 4 { return Ok(false); }
    let magic = u32::from_be_bytes(header[..4].try_into().unwrap());
    if matches!(magic, 0xfeedface | 0xcefaedfe | 0xfeedfacf | 0xcffaedfe) { return Ok(true); }
    if len < 12 { return Ok(false); }
    let (count, cpu) = match magic {
        0xcafebabe | 0xcafebabf => (u32::from_be_bytes(header[4..8].try_into().unwrap()), u32::from_be_bytes(header[8..12].try_into().unwrap())),
        0xbebafeca | 0xbfbafeca => (u32::from_le_bytes(header[4..8].try_into().unwrap()), u32::from_le_bytes(header[8..12].try_into().unwrap())),
        _ => return Ok(false),
    };
    // CAFEBABE is also Java's magic; validate a real Mach CPU type.
    Ok((1..=32).contains(&count) && matches!(cpu & 0x00ffffff, 7 | 12 | 18))
}

fn hash_bundle_file(bundle_path: &Path, path: &Path) -> Result<FileHash> {
    #[cfg(any(target_os = "wasi", feature = "performance"))]
    let performance = siboot_perf::span("sign.resource", &[]);
    let relative_path = relative_path_string(bundle_path, path)?;
    if fs::symlink_metadata(path).map_err(|source| CodeSignError::io(path, source))?.file_type().is_symlink() {
        let target = fs::read_link(path).map_err(|source| CodeSignError::io(path, source))?;
        #[cfg(any(target_os = "wasi", feature = "performance"))]
        performance.finish(siboot_perf::Outcome::Ok, &[("symlink", 1)]);
        return Ok(FileHash { relative_path, sha1: [0; 20], sha256: [0; 32], optional: false, symlink: Some(path_to_forward_slashes(&target)?) });
    }
    let data = read_file_bytes(path)?;
    let (sha1, sha256) = hash_file_data(data.as_slice());
    #[cfg(any(target_os = "wasi", feature = "performance"))]
    performance.finish(siboot_perf::Outcome::Ok, &[("bytes", data.as_slice().len() as u64)]);
    Ok(FileHash {
        optional: relative_path.contains(".lproj"),
        relative_path,
        sha1,
        sha256,
        symlink: None,
    })
}

/// Audit the final tree, after every nested image and resource has been signed.
/// This catches omitted .config files and stale hashes before installation.
pub fn verify_bundle_resources(path: &Path) -> Result<()> {
    #[cfg(any(target_os = "wasi", feature = "performance"))]
    let performance = siboot_perf::span("sign.verify_seals", &[]);
    let bundle = Bundle::open(path)?;
    let executable = bundle.executable_name()?;
    let main = fs::canonicalize(path.join(executable)).map_err(|source| CodeSignError::io(path, source))?;
    let sealed: Dictionary = plist::from_file(path.join(CODE_SIGNATURE_DIR).join(CODE_RESOURCES_FILE))?;
    let files = sealed.get("files2").and_then(Value::as_dictionary).ok_or_else(|| CodeSignError::macho(path, "missing resource seal"))?;
    walk_files_pruned(path,
        &mut |dir| dir != path.join(CODE_SIGNATURE_DIR) && dir != path.join(FAIRPLAY_DIR),
        &mut |file| {
            let relative = relative_path_string(path, file)?;
            if omit_from_files2(&relative) || fs::canonicalize(file).ok().as_ref() == Some(&main) { return Ok(()); }
            let entry = files.get(&relative).and_then(Value::as_dictionary)
                .ok_or_else(|| CodeSignError::macho(file, "resource was not sealed"))?;
            let actual = hash_bundle_file(path, file)?;
            let matches = match actual.symlink {
                Some(target) => entry.get("symlink").and_then(Value::as_string) == Some(target.as_str()),
                None => entry.get("hash2").and_then(Value::as_data) == Some(actual.sha256.as_slice()),
            };
            if !matches { return Err(CodeSignError::macho(file, "resource seal does not match its final contents")); }
            Ok(())
        })?;
    #[cfg(any(target_os = "wasi", feature = "performance"))]
    performance.finish(siboot_perf::Outcome::Ok, &[]);
    Ok(())
}

fn hash_file_data(data: &[u8]) -> ([u8; 20], [u8; 32]) {
    #[cfg(any(target_os = "wasi", feature = "performance"))]
    return siboot_perf::work("sign.resource.hash", data.len() as u64, || hash_file_data_inner(data));
    #[cfg(not(any(target_os = "wasi", feature = "performance")))]
    hash_file_data_inner(data)
}

fn hash_file_data_inner(data: &[u8]) -> ([u8; 20], [u8; 32]) {
    if data.len() >= 1024 * 1024 {
        crate::parallel::join(
            || sha1::Sha1::digest(data).into(),
            || sha2::Sha256::digest(data).into(),
        )
    } else {
        (
            sha1::Sha1::digest(data).into(),
            sha2::Sha256::digest(data).into(),
        )
    }
}

fn encode_code_resources(maps: &CodeResourcesMaps) -> Result<Vec<u8>> {
    let mut code_resources = Dictionary::new();
    code_resources.insert("files".to_string(), Value::Dictionary(maps.files.clone()));
    code_resources.insert("files2".to_string(), Value::Dictionary(maps.files2.clone()));
    code_resources.insert("rules".to_string(), rules());
    code_resources.insert("rules2".to_string(), rules2());

    let mut data = Vec::new();
    plist::to_writer_xml(&mut data, &Value::Dictionary(code_resources))?;
    Ok(data)
}

fn rules() -> Value {
    let mut rules = Dictionary::new();
    rules.insert("^.*".to_string(), Value::Boolean(true));
    rules.insert(
        "^.*\\.lproj/".to_string(),
        weighted_rule([("optional", Value::Boolean(true))], 1000.0),
    );
    rules.insert(
        "^.*\\.lproj/locversion.plist$".to_string(),
        weighted_rule([("omit", Value::Boolean(true))], 1100.0),
    );
    rules.insert("^Base\\.lproj/".to_string(), weight_only_rule(1010.0));
    rules.insert("^version.plist$".to_string(), Value::Boolean(true));
    Value::Dictionary(rules)
}

fn rules2() -> Value {
    let mut rules = Dictionary::new();
    rules.insert(".*\\.dSYM($|/)".to_string(), weight_only_rule(11.0));
    rules.insert(
        "^(.*/)?\\.DS_Store$".to_string(),
        weighted_rule([("omit", Value::Boolean(true))], 2000.0),
    );
    rules.insert("^.*".to_string(), Value::Boolean(true));
    rules.insert(
        "^.*\\.lproj/".to_string(),
        weighted_rule([("optional", Value::Boolean(true))], 1000.0),
    );
    rules.insert(
        "^.*\\.lproj/locversion.plist$".to_string(),
        weighted_rule([("omit", Value::Boolean(true))], 1100.0),
    );
    rules.insert("^Base\\.lproj/".to_string(), weight_only_rule(1010.0));
    rules.insert(
        "^Info\\.plist$".to_string(),
        weighted_rule([("omit", Value::Boolean(true))], 20.0),
    );
    rules.insert(
        "^PkgInfo$".to_string(),
        weighted_rule([("omit", Value::Boolean(true))], 20.0),
    );
    rules.insert(
        "^embedded\\.provisionprofile$".to_string(),
        weight_only_rule(20.0),
    );
    rules.insert("^version\\.plist$".to_string(), weight_only_rule(20.0));
    Value::Dictionary(rules)
}

fn weighted_rule<const N: usize>(entries: [(&str, Value); N], weight: f64) -> Value {
    let mut dict = Dictionary::new();
    for (key, value) in entries {
        dict.insert(key.to_string(), value);
    }
    dict.insert("weight".to_string(), Value::Real(weight));
    Value::Dictionary(dict)
}

fn weight_only_rule(weight: f64) -> Value {
    weighted_rule::<0>([], weight)
}

fn encode_binary_plist(dict: &Dictionary) -> Result<Vec<u8>> {
    let mut data = Vec::new();
    plist::to_writer_binary(&mut data, &Value::Dictionary(dict.clone()))?;
    Ok(data)
}

fn relative_path_string(root: &Path, path: &Path) -> Result<String> {
    let relative = path
        .strip_prefix(root)
        .map_err(|_| CodeSignError::InvalidBundlePath(path.to_path_buf()))?;
    path_to_forward_slashes(relative)
}

fn path_to_forward_slashes(path: &Path) -> Result<String> {
    let mut parts = Vec::new();
    for component in path.components() {
        parts.push(
            component
                .as_os_str()
                .to_str()
                .ok_or_else(|| CodeSignError::InvalidBundlePath(path.to_path_buf()))?,
        );
    }
    Ok(parts.join("/"))
}

fn join_relative(root: &str, child: &str) -> String {
    if root.is_empty() {
        child.to_string()
    } else if child.is_empty() {
        root.to_string()
    } else {
        format!("{root}/{child}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn complete_signing_covers_deep_images_hidden_configs_and_nested_bundles() {
        let root = std::env::temp_dir().join(format!("complete-signing-{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
        let bundle = |relative: &str, executable: &str| {
            let folder = root.join(relative);
            fs::create_dir_all(&folder).unwrap();
            let mut info = Dictionary::new();
            info.insert("CFBundleIdentifier".into(), format!("example.{}", executable).into());
            info.insert("CFBundleExecutable".into(), executable.into());
            plist::to_file_xml(folder.join("Info.plist"), &info).unwrap();
            fs::write(folder.join(executable), crate::macho::tests::minimal_macho_with_empty_signature()).unwrap();
        };
        bundle("", "Main");
        bundle("Resources/Deep/Helper.appex", "Helper");
        bundle("Frameworks/Nested/Engine.framework", "Engine");
        for relative in ["Frameworks/Deep/libHidden.dylib", "PlugIns/Tools/helper", "Resources/Disguised.config"] {
            let file = root.join(relative);
            fs::create_dir_all(file.parent().unwrap()).unwrap();
            fs::write(file, crate::macho::tests::minimal_macho_with_empty_signature()).unwrap();
        }
        for relative in ["Frameworks/Deep/settings.config", "PlugIns/Tools/.config", "Resources/.config/settings.json"] {
            let file = root.join(relative);
            fs::create_dir_all(file.parent().unwrap()).unwrap();
            fs::write(file, b"config bytes must be sealed").unwrap();
        }
        fs::create_dir_all(root.join("_CodeSignature")).unwrap();
        fs::write(root.join("_CodeSignature/old-resource-rule"), b"old").unwrap();
        let settings = BundleSigningSettings::new("COMPLETETEAM", Dictionary::new(), None);
        sign_bundle(&root, &settings).unwrap();
        assert!(!root.join("_CodeSignature/old-resource-rule").exists());
        for relative in ["Main", "Resources/Deep/Helper.appex/Helper", "Frameworks/Nested/Engine.framework/Engine", "Frameworks/Deep/libHidden.dylib", "PlugIns/Tools/helper", "Resources/Disguised.config"] {
            let bytes = fs::read(root.join(relative)).unwrap();
            assert!(bytes.windows(b"COMPLETETEAM".len()).any(|b| b == b"COMPLETETEAM"), "{relative} was not signed");
        }
        verify_bundle_resources(&root).unwrap();
        fs::write(root.join("PlugIns/Tools/.config"), b"tampered after signing").unwrap();
        assert!(verify_bundle_resources(&root).is_err(), "config changes must invalidate the resource seal");
        fs::remove_dir_all(root).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn complete_signing_seals_symlinks_and_their_actual_code_without_following_external_paths() {
        use std::os::unix::fs::symlink;
        let root = std::env::temp_dir().join(format!("complete-symlinks-{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
        fs::create_dir_all(root.join("Resources")).unwrap();
        let mut info = Dictionary::new();
        info.insert("CFBundleIdentifier".into(), "example.main".into());
        info.insert("CFBundleExecutable".into(), "Main".into());
        plist::to_file_xml(root.join("Info.plist"), &info).unwrap();
        fs::write(root.join("Main"), crate::macho::tests::minimal_macho_with_empty_signature()).unwrap();
        fs::write(root.join("Resources/real.dylib"), crate::macho::tests::minimal_macho_with_empty_signature()).unwrap();
        symlink("real.dylib", root.join("Resources/alias.dylib")).unwrap();
        let settings = BundleSigningSettings::new("LINKTEAM", Dictionary::new(), None);
        sign_bundle(&root, &settings).unwrap();
        verify_bundle_resources(&root).unwrap();
        let sealed: Dictionary = plist::from_file(root.join("_CodeSignature/CodeResources")).unwrap();
        assert_eq!(sealed["files2"].as_dictionary().unwrap()["Resources/alias.dylib"].as_dictionary().unwrap()["symlink"].as_string(), Some("real.dylib"));
        symlink(std::env::temp_dir(), root.join("Resources/outside")).unwrap();
        assert!(sign_bundle(&root, &settings).is_err());
        fs::remove_dir_all(root).unwrap();
    }

    fn test_hash(relative_path: &str) -> FileHash {
        FileHash {
            relative_path: relative_path.to_string(),
            sha1: [1; 20],
            sha256: [2; 32],
            optional: false,
            symlink: None,
        }
    }

    #[test]
    fn files2_omits_only_the_current_bundles_info_and_pkginfo() {
        let mut child = CodeResourcesMaps::default();
        child.insert_hash(test_hash("Info.plist"));
        child.insert_hash(test_hash("PkgInfo"));

        assert!(child.files.contains_key("Info.plist"));
        assert!(!child.files2.contains_key("Info.plist"));
        assert!(!child.files2.contains_key("PkgInfo"));

        let mut parent = CodeResourcesMaps::default();
        parent.merge_rerooted("PlugIns/Widget.appex", &child);

        assert!(
            parent
                .files2
                .contains_key("PlugIns/Widget.appex/Info.plist")
        );
        assert!(parent.files2.contains_key("PlugIns/Widget.appex/PkgInfo"));
    }

    /// SideInstaller: an ExtensionKit extension in Extensions gets a signature
    /// of its own, with the profile and entitlements listed for its bundle ID,
    /// and the app seals it as it seals one in PlugIns. Before, its files were
    /// sealed as the app's own and its executable was never signed. A bundle
    /// in Extensions that isn't an .appex is still sealed as plain files.
    #[test]
    fn signs_app_extensions_in_extensions() {
        let root = std::env::temp_dir().join(format!(
            "apple-codesign-extensionkit-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let write_bundle = |relative: &str, bundle_id: &str, executable: Option<&str>| {
            let path = root.join(relative);
            fs::create_dir_all(&path).unwrap();
            let mut info = Dictionary::new();
            info.insert("CFBundleIdentifier".to_string(), bundle_id.into());
            if let Some(executable) = executable {
                info.insert("CFBundleExecutable".to_string(), executable.into());
                let macho = crate::macho::tests::minimal_macho_with_empty_signature();
                fs::write(path.join(executable), macho).unwrap();
            }
            plist::to_file_xml(path.join("Info.plist"), &info).unwrap();
        };
        write_bundle("", "com.example.app", Some("App"));
        write_bundle("PlugIns/Widget.appex", "com.example.app.Widget", Some("Widget"));
        write_bundle("Extensions/Kit.appex", "com.example.app.Kit", Some("Kit"));
        write_bundle("Extensions/Strings.bundle", "org.example.Strings", None);

        let entitlements = |bundle_id: &str| {
            let mut entitlements = Dictionary::new();
            let app_id = format!("TEAMID.{bundle_id}");
            entitlements.insert("application-identifier".to_string(), app_id.into());
            entitlements
        };
        let profile = b"extension profile".as_slice();
        let mut settings =
            BundleSigningSettings::new("TEAMID", entitlements("com.example.app"), None);
        for bundle_id in ["com.example.app.Widget", "com.example.app.Kit"] {
            settings
                .entitlements_by_bundle_id
                .insert(bundle_id.to_string(), entitlements(bundle_id));
            settings
                .embedded_mobileprovisions_by_bundle_id
                .insert(bundle_id.to_string(), profile);
        }
        sign_bundle(&root, &settings).unwrap();

        let kit = root.join("Extensions/Kit.appex");
        let kit_executable = fs::read(kit.join("Kit")).unwrap();
        let contains = |needle: &[u8]| {
            kit_executable
                .windows(needle.len())
                .any(|window| window == needle)
        };
        assert!(contains(b"com.example.app.Kit\0"), "Kit isn't signed as itself");
        assert!(contains(b"<string>TEAMID.com.example.app.Kit</string>"));
        assert_eq!(fs::read(kit.join("embedded.mobileprovision")).unwrap(), profile);
        assert!(kit.join("_CodeSignature/CodeResources").is_file());

        let sealed: Dictionary =
            plist::from_file(root.join("_CodeSignature/CodeResources")).unwrap();
        let files2 = sealed["files2"].as_dictionary().unwrap();
        let sealed_sha256 = |relative: &str| {
            files2
                .get(relative)
                .and_then(Value::as_dictionary)
                .and_then(|entry| entry.get("hash2"))
                .and_then(Value::as_data)
                .map(<[u8]>::to_vec)
        };
        let signed_sha256: [u8; 32] = sha2::Sha256::digest(&kit_executable).into();
        assert_eq!(
            sealed_sha256("Extensions/Kit.appex/Kit"),
            Some(signed_sha256.to_vec())
        );
        for relative in [
            "Extensions/Kit.appex/Info.plist",
            "Extensions/Kit.appex/embedded.mobileprovision",
            "Extensions/Kit.appex/_CodeSignature/CodeResources",
            "Extensions/Strings.bundle/Info.plist",
            "PlugIns/Widget.appex/Widget",
        ] {
            assert!(files2.contains_key(relative), "{relative} isn't sealed");
        }
        assert!(!root.join("Extensions/Strings.bundle/_CodeSignature").exists());

        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn direct_dylibs_are_hashed_from_their_final_signed_contents() {
        let test_root = std::env::temp_dir().join(format!(
            "apple-codesign-dylib-resources-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let dylib = test_root.join("Frameworks/Patch.dylib");
        fs::create_dir_all(dylib.parent().unwrap()).unwrap();
        fs::write(&dylib, b"unsigned").unwrap();

        let bundle_files = collect_own_bundle_files(&test_root, "Main", &[]).unwrap();
        assert_eq!(bundle_files.libraries, vec![dylib.clone()]);
        assert_eq!(bundle_files.resource_files, vec![dylib.clone()]);

        fs::write(&dylib, b"signed").unwrap();
        let hashes = hash_resource_files(&test_root, &bundle_files.resource_files).unwrap();
        let expected_sha256: [u8; 32] = sha2::Sha256::digest(b"signed").into();
        assert_eq!(hashes[0].sha256, expected_sha256);

        fs::remove_dir_all(test_root).unwrap();
    }

    #[test]
    fn parent_seals_nested_bundle_code_resources() {
        let test_root = std::env::temp_dir().join(format!(
            "apple-codesign-nested-resources-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let child_path = test_root.join("Frameworks/Example.framework");
        let signature_path = child_path.join(CODE_SIGNATURE_DIR);
        fs::create_dir_all(&signature_path).unwrap();
        fs::write(child_path.join("Example"), b"executable").unwrap();
        fs::write(signature_path.join(CODE_RESOURCES_FILE), b"resources").unwrap();

        let mut child_resources = CodeResourcesMaps::default();
        child_resources.insert_hash(test_hash("Resource.txt"));
        let child = SubBundleResult {
            path: child_path,
            relative_root: "Frameworks/Example.framework".to_string(),
            executable: "Example".to_string(),
            resources: child_resources,
        };
        let mut parent = CodeResourcesMaps::default();

        merge_sub_bundle_resources(&mut parent, &test_root, &child).unwrap();

        assert!(
            parent
                .files2
                .contains_key("Frameworks/Example.framework/Resource.txt")
        );
        assert!(
            parent
                .files2
                .contains_key("Frameworks/Example.framework/Example")
        );
        assert!(
            parent
                .files2
                .contains_key("Frameworks/Example.framework/_CodeSignature/CodeResources")
        );

        fs::remove_dir_all(test_root).unwrap();
    }
}
