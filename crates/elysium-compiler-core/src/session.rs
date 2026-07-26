use crate::manifest::{
    is_jsonl_path, portable_relative_path, resolve_manifest_path,
    validate_manifest_collection_descriptors, ManifestCollectionDescriptor, RawManifest,
    RAW_EXPORT_MANIFEST_FILE,
};
use crate::native_ui_export_abi::NativeUiExportAbiValidationReport;
use crate::raw_export_abi::RawExportAbiValidationReport;
use anyhow::{anyhow, Context, Result};
use flate2::read::GzDecoder;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::collections::{btree_map::Entry, BTreeMap};
use std::fs::{self, File};
use std::io::{BufRead, BufReader, Cursor, Read};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::{Instant, UNIX_EPOCH};

pub(crate) const INPUT_GENERATION_POLICY: &str =
    "manifest bytes plus the complete input-tree path/type/length/mtime metadata catalog must remain unchanged for the session lifetime; symlinks are forbidden";
const RAW_EXPORT_POINTER_FILE: &str = "current.json";
const RAW_EXPORT_POINTER_SCHEMA: &str = "nesqlpp/raw-export-generation-pointer/v1";
const RAW_EXPORT_GENERATIONS_DIRECTORY: &str = "generations";

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum RawExportInputAuthority {
    GenerationPointer,
    LegacyDirectManifest,
}

impl RawExportInputAuthority {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::GenerationPointer => "generation-pointer",
            Self::LegacyDirectManifest => "legacy-direct-manifest",
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawExportGenerationPointer {
    schema_version: String,
    generation_id: String,
    relative_path: String,
}

#[derive(Debug)]
struct ResolvedRawExportInput {
    authority_root: PathBuf,
    generation_root: PathBuf,
    authority: RawExportInputAuthority,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ManifestIoMetrics {
    pub load_open_count: u64,
    pub load_read_count: u64,
    pub parse_count: u64,
    pub load_bytes_read: u64,
    pub verification_open_count: u64,
    pub verification_read_count: u64,
    pub verification_bytes_read: u64,
    pub source_open_count: u64,
    pub source_read_count: u64,
    pub source_bytes_read: u64,
    pub source_decompression_count: u64,
    pub json_parse_count: u64,
    pub jsonl_parse_count: u64,
    pub jsonl_row_parse_count: u64,
    pub cache_hit_count: u64,
    pub cache_miss_count: u64,
    pub source_cache_release_count: u64,
    pub source_cache_release_ms: u64,
    pub raw_export_abi_validation_count: u64,
    pub native_ui_export_abi_validation_count: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum RelativeInputKind {
    Missing,
    File,
    Directory,
    Unsupported,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct SessionFileDescriptor {
    pub bytes: u64,
    pub sha256: String,
    pub row_count: Option<u64>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
struct InputFileIdentity {
    exists: bool,
    is_file: bool,
    is_dir: bool,
    len: u64,
    modified_unix_nanos: Option<u128>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct InputGenerationSnapshot {
    id: String,
    input_files: BTreeMap<String, InputFileIdentity>,
}

#[derive(Default, Debug)]
struct SourceIoCounters {
    open_count: AtomicU64,
    read_count: AtomicU64,
    bytes_read: AtomicU64,
    decompression_count: AtomicU64,
    json_parse_count: AtomicU64,
    jsonl_parse_count: AtomicU64,
    jsonl_row_parse_count: AtomicU64,
    cache_hit_count: AtomicU64,
    cache_miss_count: AtomicU64,
    cache_release_count: AtomicU64,
    cache_release_ms: AtomicU64,
}

#[derive(Clone, Debug)]
struct CachedLoadError(Arc<str>);

impl std::fmt::Display for CachedLoadError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.0)
    }
}

type CachedLoad<T> = std::result::Result<Arc<T>, CachedLoadError>;
type LazyCache<K, T> = Mutex<BTreeMap<K, Arc<OnceLock<CachedLoad<T>>>>>;

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct CollectionCacheKey {
    path: PathBuf,
    array_fields: Vec<&'static str>,
}

#[derive(Debug)]
pub(crate) struct RawExportSession {
    authority_input: PathBuf,
    input_authority: RawExportInputAuthority,
    input: PathBuf,
    manifest: RawManifest,
    manifest_io: ManifestIoMetrics,
    verification_open_count: AtomicU64,
    verification_read_count: AtomicU64,
    verification_bytes_read: AtomicU64,
    source_io: SourceIoCounters,
    source_bytes_cache: LazyCache<PathBuf, Vec<u8>>,
    json_cache: LazyCache<PathBuf, Value>,
    jsonl_cache: LazyCache<PathBuf, Vec<Value>>,
    collection_cache: LazyCache<CollectionCacheKey, Vec<Value>>,
    descriptor_cache: LazyCache<PathBuf, SessionFileDescriptor>,
    raw_export_abi_cache: OnceLock<CachedLoad<RawExportAbiValidationReport>>,
    native_ui_export_abi_cache: OnceLock<CachedLoad<NativeUiExportAbiValidationReport>>,
    raw_export_abi_validation_count: AtomicU64,
    native_ui_export_abi_validation_count: AtomicU64,
    generation: InputGenerationSnapshot,
}

impl RawExportSession {
    pub(crate) fn open(input: &Path) -> Result<Self> {
        let resolved = resolve_raw_export_input(input)?;
        let manifest_path = resolved.generation_root.join(RAW_EXPORT_MANIFEST_FILE);
        let mut manifest_io = ManifestIoMetrics::default();
        let mut manifest_file = File::open(&manifest_path)
            .with_context(|| format!("open manifest {}", manifest_path.display()))?;
        manifest_io.load_open_count += 1;
        let mut bytes = Vec::new();
        manifest_file
            .read_to_end(&mut bytes)
            .with_context(|| format!("read manifest {}", manifest_path.display()))?;
        manifest_io.load_read_count += 1;
        manifest_io.load_bytes_read += bytes.len() as u64;
        let manifest = serde_json::from_slice(&bytes)
            .with_context(|| format!("parse manifest {}", manifest_path.display()))?;
        manifest_io.parse_count += 1;
        let manifest_digest = format!("{:x}", Sha256::digest(&bytes));
        let generation = capture_generation(&resolved.generation_root, &manifest_digest)?;
        Ok(Self {
            authority_input: resolved.authority_root,
            input_authority: resolved.authority,
            input: resolved.generation_root,
            manifest,
            manifest_io,
            verification_open_count: AtomicU64::new(0),
            verification_read_count: AtomicU64::new(0),
            verification_bytes_read: AtomicU64::new(0),
            source_io: SourceIoCounters::default(),
            source_bytes_cache: Mutex::new(BTreeMap::new()),
            json_cache: Mutex::new(BTreeMap::new()),
            jsonl_cache: Mutex::new(BTreeMap::new()),
            collection_cache: Mutex::new(BTreeMap::new()),
            descriptor_cache: Mutex::new(BTreeMap::new()),
            raw_export_abi_cache: OnceLock::new(),
            native_ui_export_abi_cache: OnceLock::new(),
            raw_export_abi_validation_count: AtomicU64::new(0),
            native_ui_export_abi_validation_count: AtomicU64::new(0),
            generation,
        })
    }

    pub(crate) fn input(&self) -> &Path {
        &self.input
    }

    pub(crate) fn authority_input(&self) -> &Path {
        &self.authority_input
    }

    pub(crate) fn input_authority(&self) -> RawExportInputAuthority {
        self.input_authority
    }

    pub(crate) fn manifest(&self) -> &RawManifest {
        &self.manifest
    }

    pub(crate) fn manifest_io_metrics(&self) -> ManifestIoMetrics {
        ManifestIoMetrics {
            verification_open_count: self.verification_open_count.load(Ordering::Relaxed),
            verification_read_count: self.verification_read_count.load(Ordering::Relaxed),
            verification_bytes_read: self.verification_bytes_read.load(Ordering::Relaxed),
            source_open_count: self.source_io.open_count.load(Ordering::Relaxed),
            source_read_count: self.source_io.read_count.load(Ordering::Relaxed),
            source_bytes_read: self.source_io.bytes_read.load(Ordering::Relaxed),
            source_decompression_count: self.source_io.decompression_count.load(Ordering::Relaxed),
            json_parse_count: self.source_io.json_parse_count.load(Ordering::Relaxed),
            jsonl_parse_count: self.source_io.jsonl_parse_count.load(Ordering::Relaxed),
            jsonl_row_parse_count: self.source_io.jsonl_row_parse_count.load(Ordering::Relaxed),
            cache_hit_count: self.source_io.cache_hit_count.load(Ordering::Relaxed),
            cache_miss_count: self.source_io.cache_miss_count.load(Ordering::Relaxed),
            source_cache_release_count: self.source_io.cache_release_count.load(Ordering::Relaxed),
            source_cache_release_ms: self.source_io.cache_release_ms.load(Ordering::Relaxed),
            raw_export_abi_validation_count: self
                .raw_export_abi_validation_count
                .load(Ordering::Relaxed),
            native_ui_export_abi_validation_count: self
                .native_ui_export_abi_validation_count
                .load(Ordering::Relaxed),
            ..self.manifest_io
        }
    }

    pub(crate) fn read_manifest_json(&self, logical_name: &str) -> Result<Option<Arc<Value>>> {
        let Some(path) = resolve_manifest_path(&self.input, &self.manifest, logical_name) else {
            return Ok(None);
        };
        self.load_json(path).map(Some)
    }

    pub(crate) fn read_manifest_jsonl(&self, logical_name: &str) -> Result<Arc<Vec<Value>>> {
        let Some(path) = resolve_manifest_path(&self.input, &self.manifest, logical_name) else {
            return Ok(Arc::new(Vec::new()));
        };
        anyhow::ensure!(
            is_jsonl_path(&path),
            "raw-export manifest entry is not JSONL or JSONL.GZ: {}",
            logical_name
        );
        self.load_jsonl(path)
    }

    pub(crate) fn read_optional_manifest_json(
        &self,
        logical_name: &str,
    ) -> Result<Option<Arc<Value>>> {
        let Some(path) = resolve_manifest_path(&self.input, &self.manifest, logical_name) else {
            return Ok(None);
        };
        self.verify_path_generation(&path)?;
        if !path.exists() {
            return Ok(None);
        }
        self.read_manifest_json(logical_name)
    }

    pub(crate) fn read_manifest_collection(
        &self,
        descriptor: ManifestCollectionDescriptor,
    ) -> Result<Arc<Vec<Value>>> {
        validate_manifest_collection_descriptors();
        for logical_name in descriptor.logical_names {
            let Some(path) = resolve_manifest_path(&self.input, &self.manifest, logical_name)
            else {
                continue;
            };
            if is_jsonl_path(&path) {
                return self.load_jsonl(path);
            }
            let key = CollectionCacheKey {
                path: path.clone(),
                array_fields: descriptor.array_fields.to_vec(),
            };
            return cached_load(&self.collection_cache, key, &self.source_io, || {
                let value = self.load_json(path)?;
                if let Some(rows) = value.as_array() {
                    return Ok(rows.clone());
                }
                for field in descriptor.array_fields {
                    if let Some(rows) = value.get(field).and_then(Value::as_array) {
                        return Ok(rows.clone());
                    }
                }
                Ok(vec![value.as_ref().clone()])
            });
        }
        Ok(Arc::new(Vec::new()))
    }

    #[cfg(test)]
    pub(crate) fn read_relative_jsonl(&self, relative_path: &str) -> Result<Arc<Vec<Value>>> {
        let portable = portable_relative_path(relative_path).ok_or_else(|| {
            anyhow!(
                "raw-export JSONL path must be portable and relative: {}",
                relative_path
            )
        })?;
        let path = self.input.join(portable);
        anyhow::ensure!(
            is_jsonl_path(&path),
            "raw-export collection path must be JSONL or JSONL.GZ: {}",
            relative_path
        );
        self.load_jsonl(path)
    }

    pub(crate) fn visit_relative_jsonl<F>(&self, relative_path: &str, mut visitor: F) -> Result<u64>
    where
        F: FnMut(Value) -> Result<()>,
    {
        let portable = portable_relative_path(relative_path).ok_or_else(|| {
            anyhow!(
                "raw-export JSONL path must be portable and relative: {}",
                relative_path
            )
        })?;
        let path = self.input.join(portable);
        anyhow::ensure!(
            is_jsonl_path(&path),
            "raw-export collection path must be JSONL or JSONL.GZ: {}",
            relative_path
        );
        self.verify_path_generation(&path)?;
        let compressed_bytes = fs::metadata(&path)
            .with_context(|| format!("inspect {}", path.display()))?
            .len();
        let file = File::open(&path).with_context(|| format!("open {}", path.display()))?;
        self.source_io.open_count.fetch_add(1, Ordering::Relaxed);
        self.source_io
            .cache_miss_count
            .fetch_add(1, Ordering::Relaxed);
        let mut reader: Box<dyn BufRead> =
            if path.extension().and_then(|value| value.to_str()) == Some("gz") {
                self.source_io
                    .decompression_count
                    .fetch_add(1, Ordering::Relaxed);
                Box::new(BufReader::new(GzDecoder::new(file)))
            } else {
                Box::new(BufReader::new(file))
            };
        let mut line = String::new();
        let mut row_count = 0u64;
        loop {
            line.clear();
            let bytes = reader
                .read_line(&mut line)
                .with_context(|| format!("read {} line {}", path.display(), row_count + 1))?;
            if bytes == 0 {
                break;
            }
            if line.trim().is_empty() {
                continue;
            }
            let value = serde_json::from_str(&line)
                .with_context(|| format!("parse {} line {}", path.display(), row_count + 1))?;
            visitor(value)?;
            row_count += 1;
        }
        self.source_io.read_count.fetch_add(1, Ordering::Relaxed);
        self.source_io
            .bytes_read
            .fetch_add(compressed_bytes, Ordering::Relaxed);
        self.source_io
            .jsonl_parse_count
            .fetch_add(1, Ordering::Relaxed);
        self.source_io
            .jsonl_row_parse_count
            .fetch_add(row_count, Ordering::Relaxed);
        self.verify_path_generation(&path)?;
        Ok(row_count)
    }

    pub(crate) fn read_relative_json(&self, relative_path: &str) -> Result<Option<Arc<Value>>> {
        let portable = portable_relative_path(relative_path).ok_or_else(|| {
            anyhow!(
                "raw-export JSON path must be portable and relative: {}",
                relative_path
            )
        })?;
        let path = self.input.join(portable);
        self.verify_path_generation(&path)?;
        if !path.is_file() {
            return Ok(None);
        }
        self.load_json(path).map(Some)
    }

    pub(crate) fn relative_input_kind(&self, relative_path: &str) -> Result<RelativeInputKind> {
        let portable = portable_relative_path(relative_path).ok_or_else(|| {
            anyhow!(
                "raw-export path must be portable and relative: {}",
                relative_path
            )
        })?;
        let path = self.input.join(portable);
        self.verify_path_generation(&path)?;
        let metadata = match fs::symlink_metadata(&path) {
            Ok(metadata) => metadata,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                return Ok(RelativeInputKind::Missing)
            }
            Err(error) => {
                return Err(error).with_context(|| format!("inspect input path {}", path.display()))
            }
        };
        reject_link_or_reparse_metadata(&path, "raw-export input path", &metadata)?;
        Ok(if metadata.is_file() {
            RelativeInputKind::File
        } else if metadata.is_dir() {
            RelativeInputKind::Directory
        } else {
            RelativeInputKind::Unsupported
        })
    }

    pub(crate) fn describe_relative_file(
        &self,
        relative_path: &str,
    ) -> Result<Option<Arc<SessionFileDescriptor>>> {
        if self.relative_input_kind(relative_path)? != RelativeInputKind::File {
            return Ok(None);
        }
        let portable = portable_relative_path(relative_path).ok_or_else(|| {
            anyhow!(
                "raw-export file path must be portable and relative: {}",
                relative_path
            )
        })?;
        let path = self.input.join(portable);
        let descriptor = cached_load_unmetered(&self.descriptor_cache, path.clone(), || {
            self.verify_path_generation(&path)?;
            let mut file = File::open(&path).with_context(|| format!("open {}", path.display()))?;
            self.source_io.open_count.fetch_add(1, Ordering::Relaxed);
            let mut bytes = Vec::new();
            file.read_to_end(&mut bytes)
                .with_context(|| format!("read {}", path.display()))?;
            self.source_io.read_count.fetch_add(1, Ordering::Relaxed);
            self.source_io
                .bytes_read
                .fetch_add(bytes.len() as u64, Ordering::Relaxed);
            let row_count = if is_jsonl_path(&path) {
                Some(self.count_jsonl_rows_from_bytes(&path, &bytes)?)
            } else {
                None
            };
            let descriptor = SessionFileDescriptor {
                bytes: bytes.len() as u64,
                sha256: format!("{:x}", Sha256::digest(bytes.as_slice())),
                row_count,
            };
            self.verify_path_generation(&path)?;
            Ok(descriptor)
        })?;
        Ok(Some(descriptor))
    }

    fn count_jsonl_rows_from_bytes(&self, path: &Path, bytes: &[u8]) -> Result<u64> {
        let reader: Box<dyn BufRead> =
            if path.extension().and_then(|value| value.to_str()) == Some("gz") {
                self.source_io
                    .decompression_count
                    .fetch_add(1, Ordering::Relaxed);
                Box::new(BufReader::new(GzDecoder::new(bytes)))
            } else {
                Box::new(BufReader::new(Cursor::new(bytes)))
            };
        let mut row_count = 0u64;
        for (index, line) in reader.lines().enumerate() {
            let line =
                line.with_context(|| format!("read {} line {}", path.display(), index + 1))?;
            if line.trim().is_empty() {
                continue;
            }
            let _: Value = serde_json::from_str(&line)
                .with_context(|| format!("parse {} line {}", path.display(), index + 1))?;
            row_count += 1;
        }
        self.source_io
            .jsonl_parse_count
            .fetch_add(1, Ordering::Relaxed);
        self.source_io
            .jsonl_row_parse_count
            .fetch_add(row_count, Ordering::Relaxed);
        Ok(row_count)
    }

    pub(crate) fn read_relative_bytes(&self, relative_path: &str) -> Result<Option<Arc<Vec<u8>>>> {
        let portable = portable_relative_path(relative_path).ok_or_else(|| {
            anyhow!(
                "raw-export file path must be portable and relative: {}",
                relative_path
            )
        })?;
        let path = self.input.join(portable);
        self.verify_path_generation(&path)?;
        if !path.is_file() {
            return Ok(None);
        }
        self.load_source_bytes(path).map(Some)
    }

    pub(crate) fn runtime_file_descriptors(
        &self,
        logical_pairs: &[(&str, &str)],
    ) -> Result<Vec<Value>> {
        let mut files = Vec::new();
        for (public_name, manifest_key) in logical_pairs {
            let Some(path) = resolve_manifest_path(&self.input, &self.manifest, manifest_key)
            else {
                continue;
            };
            let relative_path = path
                .strip_prefix(&self.input)
                .with_context(|| format!("relativize input path {}", path.display()))?
                .to_string_lossy()
                .replace('\\', "/");
            let Some(descriptor) = self.describe_relative_file(&relative_path)? else {
                continue;
            };
            files.push(serde_json::json!({
                "logicalName": public_name,
                "manifestKey": manifest_key,
                "path": relative_path,
                "bytes": descriptor.bytes,
                "sha256": descriptor.sha256,
            }));
        }
        Ok(files)
    }

    pub(crate) fn read_optional_manifest_bytes(
        &self,
        logical_name: &str,
    ) -> Result<Option<Arc<Vec<u8>>>> {
        let Some(path) = resolve_manifest_path(&self.input, &self.manifest, logical_name) else {
            return Ok(None);
        };
        self.verify_path_generation(&path)?;
        if !path.is_file() {
            return Ok(None);
        }
        self.load_source_bytes(path).map(Some)
    }

    pub(crate) fn generation_id(&self) -> &str {
        &self.generation.id
    }

    pub(crate) fn raw_export_abi_validation<F>(
        &self,
        validator: F,
    ) -> Result<Arc<RawExportAbiValidationReport>>
    where
        F: FnOnce() -> Result<RawExportAbiValidationReport>,
    {
        cached_validation(
            &self.raw_export_abi_cache,
            &self.raw_export_abi_validation_count,
            validator,
        )
    }

    pub(crate) fn native_ui_export_abi_validation<F>(
        &self,
        validator: F,
    ) -> Result<Arc<NativeUiExportAbiValidationReport>>
    where
        F: FnOnce() -> Result<NativeUiExportAbiValidationReport>,
    {
        cached_validation(
            &self.native_ui_export_abi_cache,
            &self.native_ui_export_abi_validation_count,
            validator,
        )
    }

    pub(crate) fn verify_input_generation(&self) -> Result<()> {
        let current_manifest_digest = self.read_current_manifest_digest()?;
        let current = capture_generation(&self.input, &current_manifest_digest)?;
        anyhow::ensure!(
            current == self.generation,
            "raw-export input generation changed during compilation (expected {}, found {}); policy: {}",
            self.generation.id,
            current.id,
            INPUT_GENERATION_POLICY
        );
        Ok(())
    }

    pub(crate) fn release_source_caches(&self) {
        let started = Instant::now();
        let json_cache = std::mem::take(
            &mut *self
                .json_cache
                .lock()
                .expect("raw-export JSON cache lock poisoned"),
        );
        let source_bytes_cache = std::mem::take(
            &mut *self
                .source_bytes_cache
                .lock()
                .expect("raw-export source bytes cache lock poisoned"),
        );
        let jsonl_cache = std::mem::take(
            &mut *self
                .jsonl_cache
                .lock()
                .expect("raw-export JSONL cache lock poisoned"),
        );
        let collection_cache = std::mem::take(
            &mut *self
                .collection_cache
                .lock()
                .expect("raw-export collection cache lock poisoned"),
        );
        let descriptor_cache = std::mem::take(
            &mut *self
                .descriptor_cache
                .lock()
                .expect("raw-export descriptor cache lock poisoned"),
        );
        std::thread::scope(|scope| {
            scope.spawn(move || drop(source_bytes_cache));
            scope.spawn(move || drop(json_cache));
            scope.spawn(move || drop(jsonl_cache));
            scope.spawn(move || drop(collection_cache));
            scope.spawn(move || drop(descriptor_cache));
        });
        self.source_io
            .cache_release_count
            .fetch_add(1, Ordering::Relaxed);
        self.source_io.cache_release_ms.fetch_add(
            started.elapsed().as_millis().min(u64::MAX as u128) as u64,
            Ordering::Relaxed,
        );
    }

    fn read_current_manifest_digest(&self) -> Result<String> {
        let manifest_path = self.input.join(RAW_EXPORT_MANIFEST_FILE);
        let mut file = File::open(&manifest_path).with_context(|| {
            format!(
                "open manifest for generation verification {}",
                manifest_path.display()
            )
        })?;
        self.verification_open_count.fetch_add(1, Ordering::Relaxed);
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes).with_context(|| {
            format!(
                "read manifest for generation verification {}",
                manifest_path.display()
            )
        })?;
        self.verification_read_count.fetch_add(1, Ordering::Relaxed);
        self.verification_bytes_read
            .fetch_add(bytes.len() as u64, Ordering::Relaxed);
        Ok(format!("{:x}", Sha256::digest(bytes)))
    }

    fn load_json(&self, path: PathBuf) -> Result<Arc<Value>> {
        self.verify_path_generation(&path)?;
        let value = cached_load(&self.json_cache, path.clone(), &self.source_io, || {
            self.verify_path_generation(&path)?;
            let bytes = self.load_source_bytes(path.clone())?;
            let value = serde_json::from_slice(&bytes)
                .with_context(|| format!("parse {}", path.display()))?;
            self.source_io
                .json_parse_count
                .fetch_add(1, Ordering::Relaxed);
            self.verify_path_generation(&path)?;
            Ok(value)
        })?;
        self.verify_path_generation(&path)?;
        Ok(value)
    }

    fn load_jsonl(&self, path: PathBuf) -> Result<Arc<Vec<Value>>> {
        self.verify_path_generation(&path)?;
        let rows = cached_load(&self.jsonl_cache, path.clone(), &self.source_io, || {
            self.verify_path_generation(&path)?;
            let source_bytes = self.load_source_bytes(path.clone())?;
            let decoded_bytes = if path.extension().and_then(|value| value.to_str()) == Some("gz") {
                let mut decoded = Vec::new();
                GzDecoder::new(source_bytes.as_slice())
                    .read_to_end(&mut decoded)
                    .with_context(|| format!("decompress {}", path.display()))?;
                self.source_io
                    .decompression_count
                    .fetch_add(1, Ordering::Relaxed);
                decoded
            } else {
                source_bytes.as_ref().clone()
            };
            let reader = BufReader::new(Cursor::new(decoded_bytes));
            let mut rows = Vec::new();
            for (index, line) in reader.lines().enumerate() {
                let line =
                    line.with_context(|| format!("read {} line {}", path.display(), index + 1))?;
                if line.trim().is_empty() {
                    continue;
                }
                rows.push(
                    serde_json::from_str(&line)
                        .with_context(|| format!("parse {} line {}", path.display(), index + 1))?,
                );
            }
            self.source_io
                .jsonl_parse_count
                .fetch_add(1, Ordering::Relaxed);
            self.source_io
                .jsonl_row_parse_count
                .fetch_add(rows.len() as u64, Ordering::Relaxed);
            self.verify_path_generation(&path)?;
            Ok(rows)
        })?;
        self.verify_path_generation(&path)?;
        Ok(rows)
    }

    fn load_source_bytes(&self, path: PathBuf) -> Result<Arc<Vec<u8>>> {
        self.verify_path_generation(&path)?;
        let bytes = cached_load_unmetered(&self.source_bytes_cache, path.clone(), || {
            self.verify_path_generation(&path)?;
            let mut file = File::open(&path).with_context(|| format!("open {}", path.display()))?;
            self.source_io.open_count.fetch_add(1, Ordering::Relaxed);
            let mut bytes = Vec::new();
            file.read_to_end(&mut bytes)
                .with_context(|| format!("read {}", path.display()))?;
            self.source_io.read_count.fetch_add(1, Ordering::Relaxed);
            self.source_io
                .bytes_read
                .fetch_add(bytes.len() as u64, Ordering::Relaxed);
            self.verify_path_generation(&path)?;
            Ok(bytes)
        })?;
        self.verify_path_generation(&path)?;
        Ok(bytes)
    }

    fn verify_path_generation(&self, path: &Path) -> Result<()> {
        let relative_path = path
            .strip_prefix(&self.input)
            .with_context(|| format!("relativize input path {}", path.display()))?
            .to_string_lossy()
            .replace('\\', "/");
        let expected = self
            .generation
            .input_files
            .get(&relative_path)
            .cloned()
            .unwrap_or_default();
        let current = capture_file_identity(path)?;
        anyhow::ensure!(
            current == expected,
            "raw-export input file changed during compilation: {}; expected generation {}; policy: {}",
            relative_path,
            self.generation.id,
            INPUT_GENERATION_POLICY
        );
        Ok(())
    }
}

fn resolve_raw_export_input(input: &Path) -> Result<ResolvedRawExportInput> {
    let input_metadata = fs::symlink_metadata(input)
        .with_context(|| format!("inspect raw-export authority {}", input.display()))?;
    reject_link_or_reparse_metadata(input, "raw-export authority", &input_metadata)?;
    anyhow::ensure!(
        input_metadata.is_dir(),
        "raw-export authority must be a directory: {}",
        input.display()
    );
    let authority_root = fs::canonicalize(input)
        .with_context(|| format!("canonicalize raw-export authority {}", input.display()))?;
    let pointer_path = authority_root.join(RAW_EXPORT_POINTER_FILE);
    let pointer_metadata = match fs::symlink_metadata(&pointer_path) {
        Ok(metadata) => Some(metadata),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => None,
        Err(error) => {
            return Err(error)
                .with_context(|| format!("inspect raw-export pointer {}", pointer_path.display()));
        }
    };
    if let Some(pointer_metadata) = pointer_metadata {
        reject_link_or_reparse_metadata(
            &pointer_path,
            "raw-export current pointer",
            &pointer_metadata,
        )?;
        anyhow::ensure!(
            pointer_metadata.is_file(),
            "raw-export current pointer must be a regular file: {}",
            pointer_path.display()
        );
        let pointer_bytes = fs::read(&pointer_path)
            .with_context(|| format!("read raw-export pointer {}", pointer_path.display()))?;
        let pointer: RawExportGenerationPointer = serde_json::from_slice(&pointer_bytes)
            .with_context(|| format!("parse raw-export pointer {}", pointer_path.display()))?;
        anyhow::ensure!(
            pointer.schema_version == RAW_EXPORT_POINTER_SCHEMA,
            "unsupported raw-export pointer schema: {}",
            pointer.schema_version
        );
        validate_generation_id(&pointer.generation_id)?;
        let expected_relative_path = format!(
            "{RAW_EXPORT_GENERATIONS_DIRECTORY}/{}",
            pointer.generation_id
        );
        anyhow::ensure!(
            pointer.relative_path == expected_relative_path,
            "raw-export pointer relativePath must equal {} (found {})",
            expected_relative_path,
            pointer.relative_path
        );
        let generation_path = authority_root.join(&pointer.relative_path);
        let generation_metadata = fs::symlink_metadata(&generation_path).with_context(|| {
            format!(
                "inspect raw-export current generation {}",
                generation_path.display()
            )
        })?;
        reject_link_or_reparse_metadata(
            &generation_path,
            "raw-export current generation",
            &generation_metadata,
        )?;
        anyhow::ensure!(
            generation_metadata.is_dir(),
            "raw-export current generation must be a directory: {}",
            generation_path.display()
        );
        let generations_path = authority_root.join(RAW_EXPORT_GENERATIONS_DIRECTORY);
        let generations_metadata = fs::symlink_metadata(&generations_path).with_context(|| {
            format!(
                "inspect raw-export generations directory under {}",
                authority_root.display()
            )
        })?;
        reject_link_or_reparse_metadata(
            &generations_path,
            "raw-export generations directory",
            &generations_metadata,
        )?;
        anyhow::ensure!(
            generations_metadata.is_dir(),
            "raw-export generations path must be a directory: {}",
            generations_path.display()
        );
        let generations_root = fs::canonicalize(&generations_path).with_context(|| {
            format!(
                "canonicalize raw-export generations directory under {}",
                authority_root.display()
            )
        })?;
        let generation_root = fs::canonicalize(&generation_path).with_context(|| {
            format!(
                "canonicalize raw-export current generation {}",
                generation_path.display()
            )
        })?;
        anyhow::ensure!(
            generation_root.parent() == Some(generations_root.as_path()),
            "raw-export current generation escapes generations directory: {}",
            generation_root.display()
        );
        anyhow::ensure!(
            generation_root.join(RAW_EXPORT_MANIFEST_FILE).is_file(),
            "raw-export current generation is missing manifest: {}",
            generation_root.join(RAW_EXPORT_MANIFEST_FILE).display()
        );
        return Ok(ResolvedRawExportInput {
            authority_root,
            generation_root,
            authority: RawExportInputAuthority::GenerationPointer,
        });
    }

    let manifest_path = authority_root.join(RAW_EXPORT_MANIFEST_FILE);
    anyhow::ensure!(
        manifest_path.is_file(),
        "raw-export authority has neither {} nor a legacy direct {}: {}",
        RAW_EXPORT_POINTER_FILE,
        RAW_EXPORT_MANIFEST_FILE,
        authority_root.display()
    );
    Ok(ResolvedRawExportInput {
        generation_root: authority_root.clone(),
        authority_root,
        authority: RawExportInputAuthority::LegacyDirectManifest,
    })
}

fn validate_generation_id(generation_id: &str) -> Result<()> {
    let bytes = generation_id.as_bytes();
    let valid_length = (9..=81).contains(&bytes.len());
    let valid_first = bytes
        .first()
        .is_some_and(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit());
    let valid_rest = bytes
        .iter()
        .skip(1)
        .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || *byte == b'-');
    anyhow::ensure!(
        valid_length && valid_first && valid_rest,
        "invalid raw-export generation id: {}",
        generation_id
    );
    Ok(())
}

fn cached_validation<T, F>(
    cell: &OnceLock<CachedLoad<T>>,
    count: &AtomicU64,
    validator: F,
) -> Result<Arc<T>>
where
    F: FnOnce() -> Result<T>,
{
    cell.get_or_init(|| {
        count.fetch_add(1, Ordering::Relaxed);
        validator()
            .map(Arc::new)
            .map_err(|error| CachedLoadError(Arc::from(format!("{error:#}"))))
    })
    .clone()
    .map_err(|error| anyhow!(error))
}

fn cached_load<K, T, F>(
    cache: &LazyCache<K, T>,
    key: K,
    metrics: &SourceIoCounters,
    loader: F,
) -> Result<Arc<T>>
where
    K: Ord,
    F: FnOnce() -> Result<T>,
{
    let (cell, inserted) = {
        let mut cache = cache
            .lock()
            .map_err(|_| anyhow!("raw-export session cache lock poisoned"))?;
        match cache.entry(key) {
            Entry::Occupied(entry) => (entry.get().clone(), false),
            Entry::Vacant(entry) => {
                let cell = Arc::new(OnceLock::new());
                entry.insert(cell.clone());
                (cell, true)
            }
        }
    };
    if inserted {
        metrics.cache_miss_count.fetch_add(1, Ordering::Relaxed);
    } else {
        metrics.cache_hit_count.fetch_add(1, Ordering::Relaxed);
    }
    cell.get_or_init(|| {
        loader()
            .map(Arc::new)
            .map_err(|error| CachedLoadError(Arc::from(format!("{error:#}"))))
    })
    .clone()
    .map_err(|error| anyhow!(error))
}

fn cached_load_unmetered<K, T, F>(cache: &LazyCache<K, T>, key: K, loader: F) -> Result<Arc<T>>
where
    K: Ord,
    F: FnOnce() -> Result<T>,
{
    let cell = {
        let mut cache = cache
            .lock()
            .map_err(|_| anyhow!("raw-export session cache lock poisoned"))?;
        match cache.entry(key) {
            Entry::Occupied(entry) => entry.get().clone(),
            Entry::Vacant(entry) => {
                let cell = Arc::new(OnceLock::new());
                entry.insert(cell.clone());
                cell
            }
        }
    };
    cell.get_or_init(|| {
        loader()
            .map(Arc::new)
            .map_err(|error| CachedLoadError(Arc::from(format!("{error:#}"))))
    })
    .clone()
    .map_err(|error| anyhow!(error))
}

fn capture_generation(input: &Path, manifest_digest: &str) -> Result<InputGenerationSnapshot> {
    let mut input_files = BTreeMap::new();
    for entry in walkdir::WalkDir::new(input).follow_links(false) {
        let entry = entry.with_context(|| format!("walk raw-export input {}", input.display()))?;
        if entry.path() == input {
            continue;
        }
        let metadata = fs::symlink_metadata(entry.path()).with_context(|| {
            format!("inspect raw-export input entry {}", entry.path().display())
        })?;
        reject_link_or_reparse_metadata(entry.path(), "raw-export input entry", &metadata)?;
        let relative_path = entry
            .path()
            .strip_prefix(input)
            .with_context(|| format!("relativize input path {}", entry.path().display()))?
            .to_string_lossy()
            .replace('\\', "/");
        let identity = capture_file_identity(entry.path())?;
        input_files.insert(relative_path, identity);
    }
    let mut hasher = Sha256::new();
    hasher.update(manifest_digest.as_bytes());
    for (relative_path, identity) in &input_files {
        hasher.update(relative_path.as_bytes());
        hasher.update([
            identity.exists as u8,
            identity.is_file as u8,
            identity.is_dir as u8,
        ]);
        hasher.update(identity.len.to_le_bytes());
        hasher.update(
            identity
                .modified_unix_nanos
                .unwrap_or_default()
                .to_le_bytes(),
        );
    }
    Ok(InputGenerationSnapshot {
        id: format!("{:x}", hasher.finalize()),
        input_files,
    })
}

fn capture_file_identity(path: &Path) -> Result<InputFileIdentity> {
    let metadata = match std::fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Ok(InputFileIdentity::default())
        }
        Err(error) => {
            return Err(error).with_context(|| format!("stat input file {}", path.display()))
        }
    };
    Ok(InputFileIdentity {
        exists: true,
        is_file: metadata.is_file(),
        is_dir: metadata.is_dir(),
        len: metadata.len(),
        modified_unix_nanos: metadata.modified().ok().and_then(|modified| {
            modified
                .duration_since(UNIX_EPOCH)
                .ok()
                .map(|duration| duration.as_nanos())
        }),
    })
}

fn reject_link_or_reparse_metadata(
    path: &Path,
    label: &str,
    metadata: &fs::Metadata,
) -> Result<()> {
    anyhow::ensure!(
        !metadata.file_type().is_symlink() && !metadata_is_windows_reparse_point(metadata),
        "{} must not be a symlink or reparse point: {}",
        label,
        path.display()
    );
    Ok(())
}

#[cfg(windows)]
fn metadata_is_windows_reparse_point(metadata: &fs::Metadata) -> bool {
    use std::os::windows::fs::MetadataExt;

    const FILE_ATTRIBUTE_REPARSE_POINT: u32 = 0x0000_0400;
    metadata.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0
}

#[cfg(not(windows))]
fn metadata_is_windows_reparse_point(_metadata: &fs::Metadata) -> bool {
    false
}
