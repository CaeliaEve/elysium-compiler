use crate::cli::CompileScope;
use crate::io::{normalize_path, sha256_file};
use crate::session::RawExportSession;
use crate::version::metadata as compiler_metadata;
use anyhow::{anyhow, bail, Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
#[cfg(unix)]
use std::fs::File;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Component, Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};
use walkdir::WalkDir;

pub const OUTPUT_GENERATION_POINTER_SCHEMA_VERSION: &str =
    "elysium-compiler/output-generation-pointer/v1";
pub const OUTPUT_GENERATION_SEAL_SCHEMA_VERSION: &str =
    "elysium-compiler/output-generation-seal/v1";
pub const OUTPUT_GENERATION_SEAL_FILE: &str = "generation-seal.json";
pub const OUTPUT_GENERATIONS_DIRECTORY: &str = "generations";

static UNIQUE_SUFFIX: AtomicU64 = AtomicU64::new(0);

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct OutputGenerationPointer {
    pub schema_version: String,
    pub generation_id: String,
    pub relative_path: String,
    pub seal_sha256: String,
    pub runtime_id: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct OutputGenerationFile {
    pub path: String,
    pub bytes: u64,
    pub sha256: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct OutputInputGeneration {
    pub authority: String,
    pub authority_input: String,
    pub resolved_input: String,
    pub generation_id: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct OutputGenerationSeal {
    pub schema_version: String,
    pub generation_id: String,
    pub compiler: Value,
    pub input_generation: OutputInputGeneration,
    pub scope: String,
    pub strict: bool,
    pub debug_json: bool,
    pub runtime_id: String,
    pub file_count: usize,
    pub total_bytes: u64,
    pub files: Vec<OutputGenerationFile>,
}

#[derive(Clone, Debug)]
pub struct ResolvedOutputGeneration {
    authority: PathBuf,
    path: PathBuf,
    pointer: OutputGenerationPointer,
    seal: OutputGenerationSeal,
}

impl ResolvedOutputGeneration {
    pub fn authority(&self) -> &Path {
        &self.authority
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn pointer(&self) -> &OutputGenerationPointer {
        &self.pointer
    }

    pub fn seal(&self) -> &OutputGenerationSeal {
        &self.seal
    }
}

#[derive(Debug)]
pub(crate) struct OutputGeneration {
    authority: PathBuf,
    generations: PathBuf,
    generation_id: String,
    staging_path: PathBuf,
}

#[derive(Debug)]
pub(crate) struct SealedOutputGeneration {
    authority: PathBuf,
    generations: PathBuf,
    generation_id: String,
    sealed_path: PathBuf,
    pointer: OutputGenerationPointer,
    published: bool,
}

pub(crate) struct PreparedReceipt {
    write: PreparedAtomicWrite,
}

struct PreparedAtomicWrite {
    target: PathBuf,
    temp: PathBuf,
    committed: bool,
}

#[derive(Debug)]
enum AtomicWriteCommit {
    Durable,
    CommittedWithDurabilityWarning(anyhow::Error),
}

impl OutputGeneration {
    pub(crate) fn begin(authority: &Path) -> Result<Self> {
        fs::create_dir_all(authority)
            .with_context(|| format!("create output authority {}", authority.display()))?;
        reject_link_or_reparse(authority, "output authority")?;
        let authority = fs::canonicalize(authority)
            .with_context(|| format!("canonicalize output authority {}", authority.display()))?;
        let metadata = fs::metadata(&authority)
            .with_context(|| format!("inspect output authority {}", authority.display()))?;
        if !metadata.is_dir() {
            bail!(
                "output authority must be a directory: {}",
                authority.display()
            );
        }

        let generations_path = authority.join(OUTPUT_GENERATIONS_DIRECTORY);
        fs::create_dir_all(&generations_path)
            .with_context(|| format!("create output generations {}", generations_path.display()))?;
        reject_link_or_reparse(&generations_path, "output generations directory")?;
        let generations = fs::canonicalize(&generations_path).with_context(|| {
            format!(
                "canonicalize output generations {}",
                generations_path.display()
            )
        })?;
        if generations.parent() != Some(authority.as_path())
            || generations.file_name().and_then(|name| name.to_str())
                != Some(OUTPUT_GENERATIONS_DIRECTORY)
        {
            bail!(
                "output generations directory escaped authority: authority={}, generations={}",
                authority.display(),
                generations.display()
            );
        }
        let current_pointer = authority.join("current.json");
        if current_pointer.exists() {
            resolve_current_generation(&authority).with_context(|| {
                format!(
                    "refuse to replace invalid current output authority {}",
                    authority.display()
                )
            })?;
        }

        for _ in 0..1024 {
            let generation_id = next_generation_id()?;
            let staging_path = generations.join(format!("{generation_id}.staging"));
            match fs::create_dir(&staging_path) {
                Ok(()) => {
                    return Ok(Self {
                        authority,
                        generations,
                        generation_id,
                        staging_path,
                    });
                }
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
                Err(error) => {
                    return Err(error).with_context(|| {
                        format!(
                            "create staging output generation {}",
                            staging_path.display()
                        )
                    });
                }
            }
        }

        bail!("could not allocate a unique output generation id")
    }

    pub(crate) fn staging_path(&self) -> &Path {
        &self.staging_path
    }

    pub(crate) fn require_external_receipt_path(&self, report: &Path) -> Result<()> {
        let report = canonicalize_with_missing_tail(report)?;
        if path_is_within(&report, &self.authority) {
            bail!(
                "compile --report must be outside output authority {}; receipt path was {}",
                self.authority.display(),
                report.display()
            );
        }
        Ok(())
    }

    pub(crate) fn finalize_generation_report(&self, report: &Path) -> Result<()> {
        let mut value: Value = serde_json::from_slice(
            &fs::read(report)
                .with_context(|| format!("read generation report {}", report.display()))?,
        )
        .with_context(|| format!("parse generation report {}", report.display()))?;
        let object = value.as_object_mut().ok_or_else(|| {
            anyhow!(
                "generation report must be a JSON object: {}",
                report.display()
            )
        })?;
        let sealed_path = self.generations.join(&self.generation_id);
        object.insert(
            "output".to_string(),
            Value::String(public_path(&sealed_path)),
        );
        object.insert(
            "outputAuthority".to_string(),
            Value::String(public_path(&self.authority)),
        );
        object.insert(
            "outputGenerationId".to_string(),
            Value::String(self.generation_id.clone()),
        );
        object.insert(
            "outputPointer".to_string(),
            Value::String(public_path(&self.authority.join("current.json"))),
        );
        write_existing_json_file(report, &value)
    }

    pub(crate) fn seal(
        self,
        session: &RawExportSession,
        scope: CompileScope,
        strict: bool,
        debug_json: bool,
    ) -> Result<SealedOutputGeneration> {
        self.seal_with_fault(session, scope, strict, debug_json, SealFault::None)
    }

    fn seal_with_fault(
        self,
        session: &RawExportSession,
        scope: CompileScope,
        strict: bool,
        debug_json: bool,
        fault: SealFault,
    ) -> Result<SealedOutputGeneration> {
        session.verify_input_generation()?;
        let runtime_id = read_runtime_id(&self.staging_path)?;
        let files = collect_generation_files(&self.staging_path)?;
        let total_bytes = files.iter().map(|file| file.bytes).sum();
        let seal = OutputGenerationSeal {
            schema_version: OUTPUT_GENERATION_SEAL_SCHEMA_VERSION.to_string(),
            generation_id: self.generation_id.clone(),
            compiler: compiler_metadata(),
            input_generation: OutputInputGeneration {
                authority: session.input_authority().as_str().to_string(),
                authority_input: normalize_path(session.authority_input()),
                resolved_input: normalize_path(session.input()),
                generation_id: session.generation_id().to_string(),
            },
            scope: scope.as_str().to_string(),
            strict,
            debug_json,
            runtime_id: runtime_id.clone(),
            file_count: files.len(),
            total_bytes,
            files,
        };
        let seal_path = self.staging_path.join(OUTPUT_GENERATION_SEAL_FILE);
        write_new_json_file(&seal_path, &seal)?;
        let verified_seal =
            verify_generation_tree(&self.staging_path, &self.generation_id, Some(&runtime_id))?;
        if verified_seal != seal {
            bail!(
                "output generation seal changed during verification: {}",
                self.staging_path.display()
            );
        }
        let seal_sha256 = sha256_file(&seal_path)
            .with_context(|| format!("hash output generation seal {}", seal_path.display()))?;

        let sealed_path = self
            .authority
            .join(OUTPUT_GENERATIONS_DIRECTORY)
            .join(&self.generation_id);
        if sealed_path.exists() {
            bail!(
                "sealed output generation already exists: {}",
                sealed_path.display()
            );
        }
        verify_owned_directory(
            &self.staging_path,
            &self.generations,
            "staging output generation",
        )?;
        #[cfg(test)]
        if fault == SealFault::GenerationDurabilityFailure {
            bail!("injected output generation durability barrier failure");
        }
        #[cfg(not(test))]
        let _ = fault;
        sync_generation_tree(&self.staging_path).with_context(|| {
            format!(
                "sync output generation before publication {}",
                self.staging_path.display()
            )
        })?;
        durable_rename_directory(&self.staging_path, &sealed_path).with_context(|| {
            format!(
                "seal output generation {} -> {}",
                self.staging_path.display(),
                sealed_path.display()
            )
        })?;
        verify_owned_directory(&sealed_path, &self.generations, "sealed output generation")?;
        if let Err(error) = sync_directory(&self.generations).with_context(|| {
            format!(
                "sync output generations directory after sealing {}",
                sealed_path.display()
            )
        }) {
            let _ = remove_owned_directory(&sealed_path, &self.generations);
            return Err(error);
        }

        Ok(SealedOutputGeneration {
            authority: self.authority.clone(),
            generations: self.generations.clone(),
            generation_id: self.generation_id.clone(),
            sealed_path,
            pointer: OutputGenerationPointer {
                schema_version: OUTPUT_GENERATION_POINTER_SCHEMA_VERSION.to_string(),
                generation_id: self.generation_id.clone(),
                relative_path: format!("{OUTPUT_GENERATIONS_DIRECTORY}/{}", self.generation_id),
                seal_sha256,
                runtime_id,
            },
            published: false,
        })
    }
}

impl Drop for OutputGeneration {
    fn drop(&mut self) {
        let _ = remove_owned_directory(&self.staging_path, &self.generations);
    }
}

impl SealedOutputGeneration {
    pub(crate) fn prepare_receipt(&self, report: &Path) -> Result<PreparedReceipt> {
        let source = self.sealed_path.join("compiler-report.json");
        let bytes = fs::read(&source)
            .with_context(|| format!("read compiler receipt {}", source.display()))?;
        Ok(PreparedReceipt {
            write: prepare_atomic_write(report, &bytes).with_context(|| {
                format!("prepare external compiler receipt {}", report.display())
            })?,
        })
    }

    pub(crate) fn publish(self) -> Result<Self> {
        self.publish_with_fault(PublishFault::None)
    }

    fn publish_with_fault(mut self, fault: PublishFault) -> Result<Self> {
        let commit = self.publish_inner(fault)?;
        self.published = true;
        if let AtomicWriteCommit::CommittedWithDurabilityWarning(error) = commit {
            eprintln!(
                "warning: output generation {} is published, but pointer parent durability could not be confirmed: {error:#}",
                self.generation_id
            );
        }
        Ok(self)
    }

    pub(crate) fn commit_receipt_after_publish(&self, receipt: PreparedReceipt) {
        self.commit_receipt_after_publish_with_fault(receipt, AtomicWriteFault::None);
    }

    fn commit_receipt_after_publish_with_fault(
        &self,
        receipt: PreparedReceipt,
        fault: AtomicWriteFault,
    ) {
        match receipt.write.commit_with_fault(fault) {
            Ok(AtomicWriteCommit::Durable) => {}
            Ok(AtomicWriteCommit::CommittedWithDurabilityWarning(error)) => {
                eprintln!(
                    "warning: output generation {} is published and the non-authoritative compiler receipt was replaced, but receipt parent durability could not be confirmed: {error:#}",
                    self.generation_id
                );
            }
            Err(error) => {
                eprintln!(
                    "warning: output generation {} is published, but the non-authoritative compiler receipt could not be committed: {error:#}",
                    self.generation_id
                );
            }
        }
    }

    fn publish_inner(&self, fault: PublishFault) -> Result<AtomicWriteCommit> {
        verify_generation_tree(
            &self.sealed_path,
            &self.generation_id,
            Some(&self.pointer.runtime_id),
        )?;
        let actual_seal_sha256 = sha256_file(&self.sealed_path.join(OUTPUT_GENERATION_SEAL_FILE))?;
        if actual_seal_sha256 != self.pointer.seal_sha256 {
            bail!(
                "sealed output generation hash changed before publication: {}",
                self.sealed_path.display()
            );
        }
        match fault {
            PublishFault::None => {
                atomic_write_json(&self.authority.join("current.json"), &self.pointer)
                    .context("publish output generation pointer")
            }
            #[cfg(test)]
            PublishFault::BeforePointerReplace => {
                bail!("injected output generation failure before pointer replacement")
            }
            #[cfg(test)]
            PublishFault::PointerReplaceFailure => atomic_write_json_with_fault(
                &self.authority.join("current.json"),
                &self.pointer,
                AtomicWriteFault::BeforeReplace,
            )
            .context("publish output generation pointer"),
            #[cfg(test)]
            PublishFault::AfterPointerReplaceBeforeParentSync => atomic_write_json_with_fault(
                &self.authority.join("current.json"),
                &self.pointer,
                AtomicWriteFault::AfterReplaceBeforeParentSync,
            )
            .context("publish output generation pointer"),
        }
    }
}

impl Drop for SealedOutputGeneration {
    fn drop(&mut self) {
        if !self.published {
            let _ = remove_owned_directory(&self.sealed_path, &self.generations);
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum PublishFault {
    None,
    #[cfg(test)]
    BeforePointerReplace,
    #[cfg(test)]
    PointerReplaceFailure,
    #[cfg(test)]
    AfterPointerReplaceBeforeParentSync,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum SealFault {
    None,
    #[cfg(test)]
    GenerationDurabilityFailure,
}

pub fn resolve_current_generation(authority: &Path) -> Result<ResolvedOutputGeneration> {
    reject_link_or_reparse(authority, "output authority")?;
    let authority = fs::canonicalize(authority)
        .with_context(|| format!("canonicalize output authority {}", authority.display()))?;
    let pointer_path = authority.join("current.json");
    let pointer_metadata = fs::symlink_metadata(&pointer_path).with_context(|| {
        format!(
            "inspect output generation pointer {}",
            pointer_path.display()
        )
    })?;
    reject_link_or_reparse_metadata(&pointer_path, "output current pointer", &pointer_metadata)?;
    if !pointer_metadata.is_file() {
        bail!(
            "output current pointer must be a regular file: {}",
            pointer_path.display()
        );
    }
    let pointer: OutputGenerationPointer =
        serde_json::from_slice(&fs::read(&pointer_path).with_context(|| {
            format!("read output generation pointer {}", pointer_path.display())
        })?)
        .with_context(|| format!("parse output generation pointer {}", pointer_path.display()))?;
    validate_pointer(&pointer)?;

    let generations_path = authority.join(OUTPUT_GENERATIONS_DIRECTORY);
    reject_link_or_reparse(&generations_path, "output generations directory")?;
    let generations =
        fs::canonicalize(&generations_path).context("canonicalize output generations directory")?;
    if generations.parent() != Some(authority.as_path())
        || generations.file_name().and_then(|name| name.to_str())
            != Some(OUTPUT_GENERATIONS_DIRECTORY)
    {
        bail!(
            "output generations directory escaped authority: {}",
            generations.display()
        );
    }
    let unresolved_path = authority.join(path_from_forward_slashes(&pointer.relative_path));
    reject_link_or_reparse(&unresolved_path, "current output generation")?;
    let path = fs::canonicalize(&unresolved_path).with_context(|| {
        format!(
            "resolve current output generation {}",
            pointer.relative_path
        )
    })?;
    if path.parent() != Some(generations.as_path())
        || path.file_name().and_then(|name| name.to_str()) != Some(pointer.generation_id.as_str())
    {
        bail!(
            "output generation pointer escaped generations directory: {}",
            pointer.relative_path
        );
    }
    let metadata = fs::metadata(&path)
        .with_context(|| format!("inspect current output generation {}", path.display()))?;
    if !metadata.is_dir() {
        bail!(
            "current output generation is not a directory: {}",
            path.display()
        );
    }

    let seal_path = path.join(OUTPUT_GENERATION_SEAL_FILE);
    let seal_sha256 = sha256_file(&seal_path).with_context(|| {
        format!(
            "hash current output generation seal {}",
            seal_path.display()
        )
    })?;
    if seal_sha256 != pointer.seal_sha256 {
        bail!(
            "output generation seal hash mismatch for {}: pointer={}, actual={}",
            pointer.generation_id,
            pointer.seal_sha256,
            seal_sha256
        );
    }
    let seal = verify_generation_tree(&path, &pointer.generation_id, Some(&pointer.runtime_id))?;
    Ok(ResolvedOutputGeneration {
        authority,
        path,
        pointer,
        seal,
    })
}

fn validate_pointer(pointer: &OutputGenerationPointer) -> Result<()> {
    if pointer.schema_version != OUTPUT_GENERATION_POINTER_SCHEMA_VERSION {
        bail!(
            "output generation pointer schemaVersion must be {}, got {}",
            OUTPUT_GENERATION_POINTER_SCHEMA_VERSION,
            pointer.schema_version
        );
    }
    validate_generation_id(&pointer.generation_id)?;
    let expected = format!("{OUTPUT_GENERATIONS_DIRECTORY}/{}", pointer.generation_id);
    if pointer.relative_path != expected {
        bail!(
            "output generation pointer relativePath must equal {expected}, got {}",
            pointer.relative_path
        );
    }
    validate_sha256("output generation pointer sealSha256", &pointer.seal_sha256)?;
    if pointer.runtime_id.trim().is_empty() {
        bail!("output generation pointer runtimeId must be non-empty");
    }
    Ok(())
}

fn verify_generation_tree(
    root: &Path,
    expected_generation_id: &str,
    expected_runtime_id: Option<&str>,
) -> Result<OutputGenerationSeal> {
    let seal_path = root.join(OUTPUT_GENERATION_SEAL_FILE);
    let seal: OutputGenerationSeal = serde_json::from_slice(
        &fs::read(&seal_path)
            .with_context(|| format!("read output generation seal {}", seal_path.display()))?,
    )
    .with_context(|| format!("parse output generation seal {}", seal_path.display()))?;
    if seal.schema_version != OUTPUT_GENERATION_SEAL_SCHEMA_VERSION {
        bail!(
            "output generation seal schemaVersion must be {}, got {}",
            OUTPUT_GENERATION_SEAL_SCHEMA_VERSION,
            seal.schema_version
        );
    }
    if seal.generation_id != expected_generation_id {
        bail!(
            "output generation seal id mismatch: expected {}, got {}",
            expected_generation_id,
            seal.generation_id
        );
    }
    if let Some(expected_runtime_id) = expected_runtime_id {
        if seal.runtime_id != expected_runtime_id {
            bail!(
                "output generation runtimeId mismatch: expected {}, got {}",
                expected_runtime_id,
                seal.runtime_id
            );
        }
    }
    let runtime_id = read_runtime_id(root)?;
    if runtime_id != seal.runtime_id {
        bail!(
            "runtime manifest runtimeId differs from output generation seal: seal={}, manifest={}",
            seal.runtime_id,
            runtime_id
        );
    }
    let actual_files = collect_generation_files(root)?;
    let actual_total_bytes = actual_files.iter().map(|file| file.bytes).sum::<u64>();
    if seal.file_count != actual_files.len()
        || seal.total_bytes != actual_total_bytes
        || seal.files != actual_files
    {
        bail!(
            "output generation tree does not match seal: {}",
            root.display()
        );
    }
    Ok(seal)
}

fn collect_generation_files(root: &Path) -> Result<Vec<OutputGenerationFile>> {
    let mut files = Vec::new();
    for entry in WalkDir::new(root).follow_links(false).into_iter() {
        let entry = entry.with_context(|| format!("walk output generation {}", root.display()))?;
        if entry.path() == root {
            continue;
        }
        let file_type = entry.file_type();
        reject_link_or_reparse(entry.path(), "output generation entry")?;
        if file_type.is_dir() {
            continue;
        }
        if !file_type.is_file() {
            bail!(
                "output generation contains unsupported filesystem entry: {}",
                entry.path().display()
            );
        }
        let relative = entry.path().strip_prefix(root).with_context(|| {
            format!(
                "derive output generation relative path for {}",
                entry.path().display()
            )
        })?;
        let relative = normalize_path(relative);
        if relative == OUTPUT_GENERATION_SEAL_FILE {
            continue;
        }
        let bytes = entry
            .metadata()
            .with_context(|| format!("inspect output artifact {}", entry.path().display()))?
            .len();
        files.push(OutputGenerationFile {
            path: relative,
            bytes,
            sha256: sha256_file(entry.path())
                .with_context(|| format!("hash output artifact {}", entry.path().display()))?,
        });
    }
    files.sort_by(|left, right| left.path.cmp(&right.path));
    for pair in files.windows(2) {
        if pair[0].path == pair[1].path {
            bail!("duplicate output generation path: {}", pair[0].path);
        }
    }
    Ok(files)
}

fn read_runtime_id(root: &Path) -> Result<String> {
    let path = root.join("rust/runtime-manifest.json");
    let value: Value = serde_json::from_slice(
        &fs::read(&path).with_context(|| format!("read runtime manifest {}", path.display()))?,
    )
    .with_context(|| format!("parse runtime manifest {}", path.display()))?;
    let runtime_id = value
        .get("runtimeId")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| {
            anyhow!(
                "runtime manifest runtimeId must be non-empty: {}",
                path.display()
            )
        })?;
    Ok(runtime_id.to_string())
}

fn write_new_json_file<T: Serialize + ?Sized>(path: &Path, value: &T) -> Result<()> {
    let mut bytes = serde_json::to_vec_pretty(value)?;
    bytes.push(b'\n');
    let mut file = OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(path)
        .with_context(|| format!("create immutable file {}", path.display()))?;
    file.write_all(&bytes)
        .with_context(|| format!("write immutable file {}", path.display()))?;
    file.sync_all()
        .with_context(|| format!("sync immutable file {}", path.display()))
}

fn write_existing_json_file<T: Serialize + ?Sized>(path: &Path, value: &T) -> Result<()> {
    let mut bytes = serde_json::to_vec_pretty(value)?;
    bytes.push(b'\n');
    let mut file = OpenOptions::new()
        .write(true)
        .truncate(true)
        .open(path)
        .with_context(|| format!("open existing JSON file {}", path.display()))?;
    file.write_all(&bytes)
        .with_context(|| format!("write existing JSON file {}", path.display()))?;
    file.sync_all()
        .with_context(|| format!("sync existing JSON file {}", path.display()))
}

fn atomic_write_json<T: Serialize + ?Sized>(path: &Path, value: &T) -> Result<AtomicWriteCommit> {
    atomic_write_json_with_fault(path, value, AtomicWriteFault::None)
}

fn atomic_write_json_with_fault<T: Serialize + ?Sized>(
    path: &Path,
    value: &T,
    fault: AtomicWriteFault,
) -> Result<AtomicWriteCommit> {
    let mut bytes = serde_json::to_vec_pretty(value)?;
    bytes.push(b'\n');
    atomic_write_with_fault(path, &bytes, fault)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum AtomicWriteFault {
    None,
    #[cfg(test)]
    BeforeReplace,
    #[cfg(test)]
    AfterReplaceBeforeParentSync,
}

fn atomic_write_with_fault(
    path: &Path,
    bytes: &[u8],
    fault: AtomicWriteFault,
) -> Result<AtomicWriteCommit> {
    prepare_atomic_write(path, bytes)?.commit_with_fault(fault)
}

fn prepare_atomic_write(path: &Path, bytes: &[u8]) -> Result<PreparedAtomicWrite> {
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    fs::create_dir_all(parent)
        .with_context(|| format!("create atomic write parent {}", parent.display()))?;
    let temp = allocate_atomic_temp_path(path)?;
    let mut file = OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(&temp)
        .with_context(|| format!("create atomic temp file {}", temp.display()))?;
    let result = (|| -> Result<()> {
        file.write_all(bytes)
            .with_context(|| format!("write atomic temp file {}", temp.display()))?;
        file.sync_all()
            .with_context(|| format!("sync atomic temp file {}", temp.display()))
    })();
    if let Err(error) = result {
        drop(file);
        let _ = fs::remove_file(&temp);
        return Err(error);
    }
    drop(file);
    Ok(PreparedAtomicWrite {
        target: path.to_path_buf(),
        temp,
        committed: false,
    })
}

impl PreparedAtomicWrite {
    fn commit_with_fault(mut self, fault: AtomicWriteFault) -> Result<AtomicWriteCommit> {
        let parent = self.target.parent().unwrap_or_else(|| Path::new("."));
        #[cfg(test)]
        if fault == AtomicWriteFault::BeforeReplace {
            bail!("injected atomic replacement failure");
        }
        #[cfg(not(test))]
        let _ = fault;
        atomic_replace(&self.temp, &self.target)?;
        self.committed = true;
        #[cfg(test)]
        if fault == AtomicWriteFault::AfterReplaceBeforeParentSync {
            return Ok(AtomicWriteCommit::CommittedWithDurabilityWarning(anyhow!(
                "injected parent directory sync failure after atomic replacement"
            )));
        }
        match sync_parent_directory(parent) {
            Ok(()) => Ok(AtomicWriteCommit::Durable),
            Err(error) => Ok(AtomicWriteCommit::CommittedWithDurabilityWarning(error)),
        }
    }
}

impl Drop for PreparedAtomicWrite {
    fn drop(&mut self) {
        if !self.committed && self.temp.exists() {
            let _ = fs::remove_file(&self.temp);
        }
    }
}

fn allocate_atomic_temp_path(target: &Path) -> Result<PathBuf> {
    let parent = target.parent().unwrap_or_else(|| Path::new("."));
    let name = target
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("output");
    for _ in 0..1024 {
        let suffix = UNIQUE_SUFFIX.fetch_add(1, Ordering::Relaxed);
        let candidate = parent.join(format!(
            ".{name}.{:08x}.{suffix:016x}.tmp",
            std::process::id()
        ));
        if !candidate.exists() {
            return Ok(candidate);
        }
    }
    bail!(
        "could not allocate atomic temp path for {}",
        target.display()
    )
}

fn sync_generation_tree(root: &Path) -> Result<()> {
    let mut directories = Vec::new();
    for entry in WalkDir::new(root).follow_links(false).sort_by_file_name() {
        let entry = entry.with_context(|| format!("walk output generation {}", root.display()))?;
        let path = entry.path();
        let metadata = fs::symlink_metadata(path)
            .with_context(|| format!("inspect output generation entry {}", path.display()))?;
        reject_link_or_reparse_metadata(path, "output generation durability entry", &metadata)?;
        if metadata.is_dir() {
            directories.push(path.to_path_buf());
            continue;
        }
        if !metadata.is_file() {
            bail!(
                "output generation durability entry must be a regular file or directory: {}",
                path.display()
            );
        }
        OpenOptions::new()
            .read(true)
            .write(true)
            .open(path)
            .with_context(|| format!("open output generation file for sync {}", path.display()))?
            .sync_all()
            .with_context(|| format!("sync output generation file {}", path.display()))?;
    }
    directories.sort_by_key(|path| std::cmp::Reverse(path.components().count()));
    for directory in directories {
        sync_directory(&directory)
            .with_context(|| format!("sync output generation directory {}", directory.display()))?;
    }
    Ok(())
}

#[cfg(windows)]
fn durable_rename_directory(source: &Path, target: &Path) -> Result<()> {
    use std::os::windows::ffi::OsStrExt;

    const MOVEFILE_WRITE_THROUGH: u32 = 0x0000_0008;

    #[link(name = "Kernel32")]
    extern "system" {
        fn MoveFileExW(
            lp_existing_file_name: *const u16,
            lp_new_file_name: *const u16,
            dw_flags: u32,
        ) -> i32;
    }

    let source_wide = source
        .as_os_str()
        .encode_wide()
        .chain(std::iter::once(0))
        .collect::<Vec<_>>();
    let target_wide = target
        .as_os_str()
        .encode_wide()
        .chain(std::iter::once(0))
        .collect::<Vec<_>>();
    let moved = unsafe {
        MoveFileExW(
            source_wide.as_ptr(),
            target_wide.as_ptr(),
            MOVEFILE_WRITE_THROUGH,
        )
    };
    if moved == 0 {
        return Err(std::io::Error::last_os_error()).context("durably rename output generation");
    }
    Ok(())
}

#[cfg(not(windows))]
fn durable_rename_directory(source: &Path, target: &Path) -> Result<()> {
    fs::rename(source, target).with_context(|| {
        format!(
            "rename output generation {} -> {}",
            source.display(),
            target.display()
        )
    })
}

#[cfg(windows)]
fn sync_directory(directory: &Path) -> Result<()> {
    use std::ffi::c_void;
    use std::os::windows::ffi::OsStrExt;

    const GENERIC_READ: u32 = 0x8000_0000;
    const GENERIC_WRITE: u32 = 0x4000_0000;
    const FILE_SHARE_READ: u32 = 0x0000_0001;
    const FILE_SHARE_WRITE: u32 = 0x0000_0002;
    const FILE_SHARE_DELETE: u32 = 0x0000_0004;
    const OPEN_EXISTING: u32 = 3;
    const FILE_FLAG_BACKUP_SEMANTICS: u32 = 0x0200_0000;
    const INVALID_HANDLE_VALUE: isize = -1;

    #[link(name = "Kernel32")]
    extern "system" {
        fn CreateFileW(
            lp_file_name: *const u16,
            dw_desired_access: u32,
            dw_share_mode: u32,
            lp_security_attributes: *mut c_void,
            dw_creation_disposition: u32,
            dw_flags_and_attributes: u32,
            h_template_file: isize,
        ) -> isize;
        fn FlushFileBuffers(h_file: isize) -> i32;
        fn CloseHandle(h_object: isize) -> i32;
    }

    let directory_wide = directory
        .as_os_str()
        .encode_wide()
        .chain(std::iter::once(0))
        .collect::<Vec<_>>();
    let handle = unsafe {
        CreateFileW(
            directory_wide.as_ptr(),
            GENERIC_READ | GENERIC_WRITE,
            FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE,
            std::ptr::null_mut(),
            OPEN_EXISTING,
            FILE_FLAG_BACKUP_SEMANTICS,
            0,
        )
    };
    if handle == INVALID_HANDLE_VALUE {
        return Err(std::io::Error::last_os_error()).with_context(|| {
            format!("open directory for durability sync {}", directory.display())
        });
    }
    let flushed = unsafe { FlushFileBuffers(handle) };
    let flush_error = if flushed == 0 {
        Some(std::io::Error::last_os_error())
    } else {
        None
    };
    unsafe {
        CloseHandle(handle);
    }
    if let Some(error) = flush_error {
        return Err(error)
            .with_context(|| format!("flush directory durability {}", directory.display()));
    }
    Ok(())
}

#[cfg(unix)]
fn sync_directory(directory: &Path) -> Result<()> {
    File::open(directory)
        .with_context(|| format!("open directory for durability sync {}", directory.display()))?
        .sync_all()
        .with_context(|| format!("sync directory durability {}", directory.display()))
}

#[cfg(not(any(windows, unix)))]
fn sync_directory(_directory: &Path) -> Result<()> {
    Ok(())
}

#[cfg(windows)]
fn atomic_replace(source: &Path, target: &Path) -> Result<()> {
    use std::os::windows::ffi::OsStrExt;

    const MOVEFILE_REPLACE_EXISTING: u32 = 0x0000_0001;
    const MOVEFILE_WRITE_THROUGH: u32 = 0x0000_0008;

    #[link(name = "Kernel32")]
    extern "system" {
        fn MoveFileExW(
            lp_existing_file_name: *const u16,
            lp_new_file_name: *const u16,
            dw_flags: u32,
        ) -> i32;
    }

    let source = source
        .as_os_str()
        .encode_wide()
        .chain(std::iter::once(0))
        .collect::<Vec<_>>();
    let target = target
        .as_os_str()
        .encode_wide()
        .chain(std::iter::once(0))
        .collect::<Vec<_>>();
    let replaced = unsafe {
        MoveFileExW(
            source.as_ptr(),
            target.as_ptr(),
            MOVEFILE_REPLACE_EXISTING | MOVEFILE_WRITE_THROUGH,
        )
    };
    if replaced == 0 {
        return Err(std::io::Error::last_os_error()).context("atomically replace file");
    }
    Ok(())
}

#[cfg(not(windows))]
fn atomic_replace(source: &Path, target: &Path) -> Result<()> {
    fs::rename(source, target).with_context(|| {
        format!(
            "atomically replace {} with {}",
            target.display(),
            source.display()
        )
    })
}

#[cfg(unix)]
fn sync_parent_directory(parent: &Path) -> Result<()> {
    File::open(parent)
        .with_context(|| format!("open atomic write parent {}", parent.display()))?
        .sync_all()
        .with_context(|| format!("sync atomic write parent {}", parent.display()))
}

#[cfg(not(unix))]
fn sync_parent_directory(_parent: &Path) -> Result<()> {
    Ok(())
}

fn next_generation_id() -> Result<String> {
    let elapsed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .context("system clock is before the Unix epoch")?;
    let suffix = UNIQUE_SUFFIX.fetch_add(1, Ordering::Relaxed);
    Ok(format!(
        "gen-{:016x}{:08x}-{:08x}-{suffix:016x}",
        elapsed.as_secs(),
        elapsed.subsec_nanos(),
        std::process::id()
    ))
}

fn validate_generation_id(generation_id: &str) -> Result<()> {
    if generation_id.len() < 16
        || generation_id.len() > 128
        || !generation_id
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
    {
        bail!("invalid output generation id: {generation_id}");
    }
    Ok(())
}

fn validate_sha256(label: &str, value: &str) -> Result<()> {
    if value.len() != 64 || !value.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        bail!("{label} must be a 64-character hexadecimal SHA-256 digest");
    }
    Ok(())
}

fn path_from_forward_slashes(path: &str) -> PathBuf {
    path.split('/').collect()
}

fn canonicalize_with_missing_tail(path: &Path) -> Result<PathBuf> {
    let mut absolute = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()?.join(path)
    };
    absolute = normalize_lexical(&absolute);

    let mut existing = absolute.as_path();
    let mut missing = Vec::new();
    while !existing.exists() {
        let name = existing.file_name().ok_or_else(|| {
            anyhow!(
                "cannot resolve existing ancestor for path {}",
                absolute.display()
            )
        })?;
        missing.push(name.to_os_string());
        existing = existing.parent().ok_or_else(|| {
            anyhow!(
                "cannot resolve existing ancestor for path {}",
                absolute.display()
            )
        })?;
    }
    let mut resolved = fs::canonicalize(existing)
        .with_context(|| format!("canonicalize path ancestor {}", existing.display()))?;
    for component in missing.iter().rev() {
        resolved.push(component);
    }
    Ok(resolved)
}

fn normalize_lexical(path: &Path) -> PathBuf {
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                normalized.pop();
            }
            _ => normalized.push(component.as_os_str()),
        }
    }
    normalized
}

fn path_is_within(candidate: &Path, authority: &Path) -> bool {
    #[cfg(windows)]
    {
        let candidate = normalize_path(candidate).to_ascii_lowercase();
        let authority = normalize_path(authority).to_ascii_lowercase();
        candidate == authority || candidate.starts_with(&(authority + "/"))
    }
    #[cfg(not(windows))]
    {
        candidate == authority || candidate.starts_with(authority)
    }
}

fn public_path(path: &Path) -> String {
    let normalized = normalize_path(path);
    #[cfg(windows)]
    {
        if let Some(path) = normalized.strip_prefix("//?/UNC/") {
            return format!("//{path}");
        }
        if let Some(path) = normalized.strip_prefix("//?/") {
            return path.to_string();
        }
    }
    normalized
}

fn reject_link_or_reparse(path: &Path, label: &str) -> Result<()> {
    let metadata = fs::symlink_metadata(path)
        .with_context(|| format!("inspect {label} {}", path.display()))?;
    reject_link_or_reparse_metadata(path, label, &metadata)
}

fn reject_link_or_reparse_metadata(
    path: &Path,
    label: &str,
    metadata: &fs::Metadata,
) -> Result<()> {
    if metadata.file_type().is_symlink() || metadata_is_windows_reparse_point(metadata) {
        bail!(
            "{label} must not be a symlink or reparse point: {}",
            path.display()
        );
    }
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

fn verify_owned_directory(path: &Path, parent: &Path, label: &str) -> Result<()> {
    reject_link_or_reparse(path, label)?;
    let resolved = fs::canonicalize(path)
        .with_context(|| format!("canonicalize {label} {}", path.display()))?;
    if resolved.parent() != Some(parent) {
        bail!(
            "{label} escaped owned parent: parent={}, path={}",
            parent.display(),
            resolved.display()
        );
    }
    let metadata = fs::metadata(&resolved)
        .with_context(|| format!("inspect {label} {}", resolved.display()))?;
    if !metadata.is_dir() {
        bail!("{label} must be a directory: {}", resolved.display());
    }
    Ok(())
}

fn remove_owned_directory(path: &Path, parent: &Path) -> Result<()> {
    let metadata = match fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(error) => {
            return Err(error).with_context(|| format!("inspect cleanup path {}", path.display()))
        }
    };
    if path.parent() != Some(parent) {
        bail!(
            "refuse to clean output generation outside owned parent: {}",
            path.display()
        );
    }
    if metadata.file_type().is_symlink() || metadata_is_windows_reparse_point(&metadata) {
        return fs::remove_dir(path)
            .or_else(|_| fs::remove_file(path))
            .with_context(|| format!("remove output generation link {}", path.display()));
    }
    verify_owned_directory(path, parent, "output generation cleanup target")?;
    fs::remove_dir_all(path).with_context(|| format!("remove output generation {}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn test_session() -> (tempfile::TempDir, RawExportSession) {
        let input = tempfile::tempdir().unwrap();
        fs::write(
            input.path().join("manifest.json"),
            br#"{"schemaVersion":"fixture/v1","files":{}}"#,
        )
        .unwrap();
        let session = RawExportSession::open(input.path()).unwrap();
        (input, session)
    }

    fn populate_generation(generation: &OutputGeneration, runtime_id: &str) {
        fs::create_dir_all(generation.staging_path().join("rust")).unwrap();
        fs::write(generation.staging_path().join("rust/data.bin"), b"payload").unwrap();
        fs::write(
            generation.staging_path().join("rust/runtime-manifest.json"),
            serde_json::to_vec_pretty(&json!({ "runtimeId": runtime_id })).unwrap(),
        )
        .unwrap();
        fs::write(
            generation.staging_path().join("compiler-report.json"),
            b"{}\n",
        )
        .unwrap();
    }

    fn publish_fixture(authority: &Path, runtime_id: &str) -> ResolvedOutputGeneration {
        let (_input, session) = test_session();
        let generation = OutputGeneration::begin(authority).unwrap();
        populate_generation(&generation, runtime_id);
        generation
            .seal(&session, CompileScope::All, true, false)
            .unwrap()
            .publish()
            .unwrap();
        resolve_current_generation(authority).unwrap()
    }

    #[test]
    fn pointer_replacement_keeps_previous_generation_and_has_no_flat_manifest_mirror() {
        let authority = tempfile::tempdir().unwrap();
        let first = publish_fixture(authority.path(), "runtime-first");
        let first_path = first.path().to_path_buf();
        let first_pointer = fs::read(authority.path().join("current.json")).unwrap();
        let first_files = collect_generation_files(&first_path).unwrap();
        assert_eq!(
            first.pointer().schema_version,
            OUTPUT_GENERATION_POINTER_SCHEMA_VERSION
        );
        assert_eq!(
            first.pointer().relative_path,
            format!(
                "{OUTPUT_GENERATIONS_DIRECTORY}/{}",
                first.pointer().generation_id
            )
        );
        assert_eq!(first.pointer().runtime_id, "runtime-first");
        assert_eq!(
            first.seal().schema_version,
            OUTPUT_GENERATION_SEAL_SCHEMA_VERSION
        );
        assert_eq!(first.seal().file_count, first.seal().files.len());
        assert_eq!(
            first.seal().total_bytes,
            first
                .seal()
                .files
                .iter()
                .map(|file| file.bytes)
                .sum::<u64>()
        );

        let second = publish_fixture(authority.path(), "runtime-second");
        assert_ne!(second.path(), first_path);
        assert!(
            first_path.is_dir(),
            "the previous generation must remain rollbackable"
        );
        assert_eq!(collect_generation_files(&first_path).unwrap(), first_files);
        assert_ne!(
            fs::read(authority.path().join("current.json")).unwrap(),
            first_pointer
        );
        assert!(!authority.path().join("manifest.json").exists());
        assert!(!authority.path().join("rust/runtime-manifest.json").exists());
    }

    #[test]
    fn injected_pre_publish_failure_preserves_pointer_and_previous_generation_hashes() {
        let authority = tempfile::tempdir().unwrap();
        let current = publish_fixture(authority.path(), "runtime-current");
        let pointer_before = fs::read(authority.path().join("current.json")).unwrap();
        let current_files_before = collect_generation_files(current.path()).unwrap();

        let (_input, session) = test_session();
        let generation = OutputGeneration::begin(authority.path()).unwrap();
        populate_generation(&generation, "runtime-candidate");
        let candidate_id = generation.generation_id.clone();
        let sealed = generation
            .seal(&session, CompileScope::All, true, false)
            .unwrap();
        let error = sealed
            .publish_inner(PublishFault::BeforePointerReplace)
            .unwrap_err();
        assert!(error
            .to_string()
            .contains("injected output generation failure"));
        drop(sealed);

        assert_eq!(
            fs::read(authority.path().join("current.json")).unwrap(),
            pointer_before
        );
        assert_eq!(
            collect_generation_files(current.path()).unwrap(),
            current_files_before
        );
        assert!(!authority
            .path()
            .join(OUTPUT_GENERATIONS_DIRECTORY)
            .join(candidate_id)
            .exists());
    }

    #[test]
    fn generation_durability_barrier_failure_preserves_previous_pointer_and_generation() {
        let authority = tempfile::tempdir().unwrap();
        let current = publish_fixture(authority.path(), "runtime-current");
        let pointer_before = fs::read(authority.path().join("current.json")).unwrap();
        let current_files_before = collect_generation_files(current.path()).unwrap();

        let (_input, session) = test_session();
        let generation = OutputGeneration::begin(authority.path()).unwrap();
        populate_generation(&generation, "runtime-candidate");
        let candidate_id = generation.generation_id.clone();
        let error = generation
            .seal_with_fault(
                &session,
                CompileScope::All,
                true,
                false,
                SealFault::GenerationDurabilityFailure,
            )
            .unwrap_err();
        assert!(format!("{error:#}").contains("durability barrier failure"));

        assert_eq!(
            fs::read(authority.path().join("current.json")).unwrap(),
            pointer_before
        );
        assert_eq!(
            collect_generation_files(current.path()).unwrap(),
            current_files_before
        );
        assert!(!authority
            .path()
            .join(OUTPUT_GENERATIONS_DIRECTORY)
            .join(&candidate_id)
            .exists());
        assert!(!authority
            .path()
            .join(OUTPUT_GENERATIONS_DIRECTORY)
            .join(format!("{candidate_id}.staging"))
            .exists());
    }

    #[test]
    fn injected_pointer_replace_failure_leaves_pointer_and_receipt_untouched() {
        let authority = tempfile::tempdir().unwrap();
        let receipt_dir = tempfile::tempdir().unwrap();
        let receipt = receipt_dir.path().join("compiler-report.json");
        let current = publish_fixture(authority.path(), "runtime-current");
        let pointer_before = fs::read(authority.path().join("current.json")).unwrap();
        let current_files_before = collect_generation_files(current.path()).unwrap();

        let (_input, session) = test_session();
        let generation = OutputGeneration::begin(authority.path()).unwrap();
        populate_generation(&generation, "runtime-candidate");
        let candidate_id = generation.generation_id.clone();
        let sealed = generation
            .seal(&session, CompileScope::All, true, false)
            .unwrap();
        let prepared_receipt = sealed.prepare_receipt(&receipt).unwrap();
        let publish_result = sealed.publish_inner(PublishFault::PointerReplaceFailure);
        let error = publish_result.unwrap_err();
        assert!(format!("{error:#}").contains("injected atomic replacement failure"));
        drop(prepared_receipt);
        drop(sealed);

        assert!(!receipt.exists());
        assert_eq!(
            fs::read(authority.path().join("current.json")).unwrap(),
            pointer_before
        );
        assert_eq!(
            collect_generation_files(current.path()).unwrap(),
            current_files_before
        );
        assert!(!authority
            .path()
            .join(OUTPUT_GENERATIONS_DIRECTORY)
            .join(candidate_id)
            .exists());
        assert!(!fs::read_dir(authority.path()).unwrap().any(|entry| entry
            .unwrap()
            .file_name()
            .to_string_lossy()
            .ends_with(".tmp")));
    }

    #[test]
    fn pointer_parent_sync_failure_after_replace_keeps_published_generation() {
        let authority = tempfile::tempdir().unwrap();
        let previous = publish_fixture(authority.path(), "runtime-previous");
        let previous_path = previous.path().to_path_buf();

        let (_input, session) = test_session();
        let generation = OutputGeneration::begin(authority.path()).unwrap();
        populate_generation(&generation, "runtime-published");
        let sealed = generation
            .seal(&session, CompileScope::All, true, false)
            .unwrap();
        let published_id = sealed.generation_id.clone();
        let published_path = sealed.sealed_path.clone();
        let published = sealed
            .publish_with_fault(PublishFault::AfterPointerReplaceBeforeParentSync)
            .unwrap();
        drop(published);

        let current = resolve_current_generation(authority.path()).unwrap();
        assert_eq!(current.pointer().generation_id, published_id);
        assert_eq!(current.pointer().runtime_id, "runtime-published");
        assert!(published_path.is_dir());
        assert!(previous_path.is_dir());
    }

    #[cfg(windows)]
    #[test]
    fn real_windows_pointer_replace_failure_preserves_current_and_receipt() {
        let authority = tempfile::tempdir().unwrap();
        let receipt_dir = tempfile::tempdir().unwrap();
        let receipt = receipt_dir.path().join("compiler-report.json");
        let current = publish_fixture(authority.path(), "runtime-current");
        let pointer_path = authority.path().join("current.json");
        let pointer_before = fs::read(&pointer_path).unwrap();
        let current_files_before = collect_generation_files(current.path()).unwrap();
        let pointer_lock = lock_file_without_delete_share(&pointer_path);

        let (_input, session) = test_session();
        let generation = OutputGeneration::begin(authority.path()).unwrap();
        populate_generation(&generation, "runtime-candidate");
        let candidate_id = generation.generation_id.clone();
        let sealed = generation
            .seal(&session, CompileScope::All, true, false)
            .unwrap();
        let prepared_receipt = sealed.prepare_receipt(&receipt).unwrap();
        let error = sealed.publish_inner(PublishFault::None).unwrap_err();
        assert!(format!("{error:#}").contains("atomically replace file"));
        drop(prepared_receipt);
        drop(sealed);
        drop(pointer_lock);

        assert_eq!(fs::read(&pointer_path).unwrap(), pointer_before);
        assert_eq!(
            collect_generation_files(current.path()).unwrap(),
            current_files_before
        );
        assert!(!receipt.exists());
        assert!(!authority
            .path()
            .join(OUTPUT_GENERATIONS_DIRECTORY)
            .join(candidate_id)
            .exists());
    }

    #[test]
    fn receipt_commit_failure_after_publication_does_not_report_transaction_failure() {
        let authority = tempfile::tempdir().unwrap();
        let receipt_dir = tempfile::tempdir().unwrap();
        let receipt = receipt_dir.path().join("compiler-report.json");
        let previous = publish_fixture(authority.path(), "runtime-previous");
        let previous_path = previous.path().to_path_buf();

        let (_input, session) = test_session();
        let generation = OutputGeneration::begin(authority.path()).unwrap();
        populate_generation(&generation, "runtime-published");
        let sealed = generation
            .seal(&session, CompileScope::All, true, false)
            .unwrap();
        let published_id = sealed.generation_id.clone();
        let prepared_receipt = sealed.prepare_receipt(&receipt).unwrap();
        let published = sealed.publish().unwrap();

        published.commit_receipt_after_publish_with_fault(
            prepared_receipt,
            AtomicWriteFault::BeforeReplace,
        );

        let current = resolve_current_generation(authority.path()).unwrap();
        assert_eq!(current.pointer().generation_id, published_id);
        assert_eq!(current.pointer().runtime_id, "runtime-published");
        assert!(previous_path.is_dir());
        assert!(!receipt.exists());
    }

    #[cfg(windows)]
    #[test]
    fn real_windows_receipt_replace_failure_keeps_published_pointer_successful() {
        let authority = tempfile::tempdir().unwrap();
        let receipt_dir = tempfile::tempdir().unwrap();
        let receipt = receipt_dir.path().join("compiler-report.json");
        fs::write(&receipt, b"previous receipt\n").unwrap();
        let receipt_before = fs::read(&receipt).unwrap();
        let receipt_lock = lock_file_without_delete_share(&receipt);

        let (_input, session) = test_session();
        let generation = OutputGeneration::begin(authority.path()).unwrap();
        populate_generation(&generation, "runtime-published");
        let sealed = generation
            .seal(&session, CompileScope::All, true, false)
            .unwrap();
        let published_id = sealed.generation_id.clone();
        let prepared_receipt = sealed.prepare_receipt(&receipt).unwrap();
        let published = sealed.publish().unwrap();
        published.commit_receipt_after_publish(prepared_receipt);
        drop(receipt_lock);

        let current = resolve_current_generation(authority.path()).unwrap();
        assert_eq!(current.pointer().generation_id, published_id);
        assert_eq!(current.pointer().runtime_id, "runtime-published");
        assert_eq!(fs::read(&receipt).unwrap(), receipt_before);
    }

    #[test]
    fn abandoned_staging_generation_is_removed_without_touching_current_authority() {
        let authority = tempfile::tempdir().unwrap();
        let current = publish_fixture(authority.path(), "runtime-current");
        let pointer_before = fs::read(authority.path().join("current.json")).unwrap();
        let current_files_before = collect_generation_files(current.path()).unwrap();

        let staging_path = {
            let generation = OutputGeneration::begin(authority.path()).unwrap();
            fs::write(generation.staging_path().join("partial.bin"), b"partial").unwrap();
            generation.staging_path().to_path_buf()
        };

        assert!(!staging_path.exists());
        assert_eq!(
            fs::read(authority.path().join("current.json")).unwrap(),
            pointer_before
        );
        assert_eq!(
            collect_generation_files(current.path()).unwrap(),
            current_files_before
        );
    }

    #[test]
    fn resolver_rejects_tree_mutation_and_never_falls_back_to_flat_root() {
        let authority = tempfile::tempdir().unwrap();
        let current = publish_fixture(authority.path(), "runtime-current");
        fs::write(current.path().join("rust/data.bin"), b"tampered").unwrap();
        fs::write(authority.path().join("manifest.json"), b"legacy-flat-root").unwrap();

        let error = resolve_current_generation(authority.path()).unwrap_err();
        assert!(
            error.to_string().contains("does not match seal"),
            "{error:#}"
        );
    }

    #[test]
    fn report_path_inside_authority_is_rejected() {
        let authority = tempfile::tempdir().unwrap();
        let generation = OutputGeneration::begin(authority.path()).unwrap();
        let error = generation
            .require_external_receipt_path(&authority.path().join("compiler-report.json"))
            .unwrap_err();
        assert!(error
            .to_string()
            .contains("must be outside output authority"));
    }

    #[test]
    fn begin_refuses_to_overwrite_an_invalid_existing_pointer() {
        let authority = tempfile::tempdir().unwrap();
        fs::create_dir_all(authority.path().join(OUTPUT_GENERATIONS_DIRECTORY)).unwrap();
        let pointer_path = authority.path().join("current.json");
        fs::write(&pointer_path, b"{not-json").unwrap();
        let pointer_before = fs::read(&pointer_path).unwrap();

        let error = OutputGeneration::begin(authority.path()).unwrap_err();
        assert!(format!("{error:#}").contains("refuse to replace invalid current output authority"));
        assert_eq!(fs::read(pointer_path).unwrap(), pointer_before);
        assert_eq!(
            fs::read_dir(authority.path().join(OUTPUT_GENERATIONS_DIRECTORY))
                .unwrap()
                .count(),
            0
        );
    }

    #[test]
    fn begin_rejects_generations_directory_link_escape() {
        let authority = tempfile::tempdir().unwrap();
        let outside = tempfile::tempdir().unwrap();
        let marker = outside.path().join("outside-marker.txt");
        fs::write(&marker, b"outside").unwrap();
        let generations = authority.path().join(OUTPUT_GENERATIONS_DIRECTORY);
        create_directory_link(outside.path(), &generations);

        let error = OutputGeneration::begin(authority.path()).unwrap_err();
        assert!(
            format!("{error:#}").contains("must not be a symlink or reparse point"),
            "{error:#}"
        );
        assert_eq!(fs::read(marker).unwrap(), b"outside");
        remove_directory_link(&generations);
    }

    #[test]
    fn begin_rejects_authority_link_escape() {
        let link_parent = tempfile::tempdir().unwrap();
        let outside = tempfile::tempdir().unwrap();
        let authority = link_parent.path().join("output-authority-link");
        create_directory_link(outside.path(), &authority);
        let error = OutputGeneration::begin(&authority).unwrap_err();
        assert!(
            format!("{error:#}").contains("must not be a symlink or reparse point"),
            "{error:#}"
        );
        remove_directory_link(&authority);
    }

    #[test]
    fn resolver_rejects_authority_link_escape() {
        let link_parent = tempfile::tempdir().unwrap();
        let outside = tempfile::tempdir().unwrap();
        publish_fixture(outside.path(), "runtime-current");
        let authority = link_parent.path().join("output-authority-link");
        create_directory_link(outside.path(), &authority);

        let error = resolve_current_generation(&authority).unwrap_err();
        assert!(
            format!("{error:#}")
                .contains("output authority must not be a symlink or reparse point"),
            "{error:#}"
        );
        remove_directory_link(&authority);
    }

    #[test]
    fn resolver_rejects_current_pointer_reparse_point() {
        let authority = tempfile::tempdir().unwrap();
        fs::create_dir_all(authority.path().join(OUTPUT_GENERATIONS_DIRECTORY)).unwrap();
        let outside = tempfile::tempdir().unwrap();
        let pointer_path = authority.path().join("current.json");
        create_directory_link(outside.path(), &pointer_path);

        let error = resolve_current_generation(authority.path()).unwrap_err();
        assert!(
            format!("{error:#}")
                .contains("output current pointer must not be a symlink or reparse point"),
            "{error:#}"
        );
        remove_directory_link(&pointer_path);
    }

    #[test]
    fn seal_rejects_nested_generation_link_escape() {
        let authority = tempfile::tempdir().unwrap();
        let outside = tempfile::tempdir().unwrap();
        let marker = outside.path().join("outside-marker.txt");
        fs::write(&marker, b"outside").unwrap();
        let (_input, session) = test_session();
        let generation = OutputGeneration::begin(authority.path()).unwrap();
        populate_generation(&generation, "runtime-candidate");
        let nested = generation.staging_path().join("nested-link");
        create_directory_link(outside.path(), &nested);
        let error = generation
            .seal(&session, CompileScope::All, true, false)
            .unwrap_err();
        assert!(
            format!("{error:#}").contains("must not be a symlink or reparse point"),
            "{error:#}"
        );
        assert_eq!(fs::read(marker).unwrap(), b"outside");
    }

    #[cfg(windows)]
    struct LockedFile(isize);

    #[cfg(windows)]
    impl Drop for LockedFile {
        fn drop(&mut self) {
            #[link(name = "Kernel32")]
            extern "system" {
                fn CloseHandle(handle: isize) -> i32;
            }
            unsafe {
                CloseHandle(self.0);
            }
        }
    }

    #[cfg(windows)]
    fn lock_file_without_delete_share(path: &Path) -> LockedFile {
        use std::os::windows::ffi::OsStrExt;
        const GENERIC_READ: u32 = 0x8000_0000;
        const FILE_SHARE_READ: u32 = 0x0000_0001;
        const OPEN_EXISTING: u32 = 3;
        const FILE_ATTRIBUTE_NORMAL: u32 = 0x0000_0080;
        const INVALID_HANDLE_VALUE: isize = -1;
        #[link(name = "Kernel32")]
        extern "system" {
            fn CreateFileW(
                file_name: *const u16,
                desired_access: u32,
                share_mode: u32,
                security_attributes: *mut std::ffi::c_void,
                creation_disposition: u32,
                flags_and_attributes: u32,
                template_file: isize,
            ) -> isize;
        }
        let wide = path
            .as_os_str()
            .encode_wide()
            .chain(std::iter::once(0))
            .collect::<Vec<_>>();
        let handle = unsafe {
            CreateFileW(
                wide.as_ptr(),
                GENERIC_READ,
                FILE_SHARE_READ,
                std::ptr::null_mut(),
                OPEN_EXISTING,
                FILE_ATTRIBUTE_NORMAL,
                0,
            )
        };
        assert_ne!(
            handle,
            INVALID_HANDLE_VALUE,
            "failed to lock {}",
            path.display()
        );
        LockedFile(handle)
    }

    #[cfg(unix)]
    fn create_directory_link(target: &Path, link: &Path) {
        std::os::unix::fs::symlink(target, link).unwrap();
    }

    #[cfg(unix)]
    fn remove_directory_link(link: &Path) {
        fs::remove_file(link).unwrap();
    }

    #[cfg(windows)]
    fn create_directory_link(target: &Path, link: &Path) {
        let status = std::process::Command::new("cmd")
            .args(["/d", "/s", "/c", "mklink", "/J"])
            .arg(link)
            .arg(target)
            .status()
            .unwrap();
        assert!(status.success(), "mklink /J failed: {status}");
    }

    #[cfg(windows)]
    fn remove_directory_link(link: &Path) {
        fs::remove_dir(link).unwrap();
    }
}
