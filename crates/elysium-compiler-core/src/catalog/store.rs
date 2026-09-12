use super::{Table, FILE_LIMIT, FORMAT, IMAGE_LIMIT, REVISION, TABLE_ROWS};
use crate::source::{is_digest, require_plain, source_path, Source};
use anyhow::{ensure, Context, Result};
use fs2::FileExt;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::fs::{self, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use tempfile::{NamedTempFile, TempDir};

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct File {
    pub path: String,
    pub kind: String,
    pub encoding: String,
    pub bytes: u64,
    pub rows: u64,
    pub sha256: String,
    pub first: Option<String>,
    pub last: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Manifest {
    #[schemars(schema_with = "catalog_format")]
    pub format: String,
    #[schemars(schema_with = "catalog_revision")]
    pub revision: u32,
    pub id: String,
    pub source: String,
    pub environment: String,
    pub compiler: String,
    pub scope: String,
    pub files: Vec<File>,
    pub counts: BTreeMap<String, u64>,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Pointer {
    #[schemars(schema_with = "pointer_format")]
    pub format: String,
    #[schemars(schema_with = "catalog_revision")]
    pub revision: u32,
    pub id: String,
}

fn catalog_format(_: &mut schemars::gen::SchemaGenerator) -> schemars::schema::Schema {
    crate::source::literal_schema(FORMAT)
}
fn catalog_revision(_: &mut schemars::gen::SchemaGenerator) -> schemars::schema::Schema {
    crate::source::literal_schema(REVISION)
}
fn pointer_format(_: &mut schemars::gen::SchemaGenerator) -> schemars::schema::Schema {
    crate::source::literal_schema("elysium.catalog-pointer")
}

#[derive(Clone, Debug, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Publication {
    pub id: String,
    pub source: String,
    pub path: PathBuf,
    pub counts: BTreeMap<String, u64>,
    pub bytes: u64,
}

impl Manifest {
    pub fn digest(&self) -> Result<String> {
        let mut value = serde_json::to_value(self)?;
        value.as_object_mut().expect("manifest").remove("id");
        Ok(hash(&serde_json::to_vec(&value)?))
    }

    pub fn validate(&self) -> Result<()> {
        ensure!(
            self.format == FORMAT && self.revision == REVISION,
            "unsupported catalog format or revision"
        );
        ensure!(
            is_digest(&self.id) && is_digest(&self.source) && is_digest(&self.environment),
            "invalid catalog identity"
        );
        ensure!(
            self.id == self.digest()?,
            "catalog manifest digest mismatch"
        );
        ensure!(
            matches!(self.scope.as_str(), "complete" | "selection"),
            "invalid catalog scope"
        );
        ensure!(
            !self.compiler.is_empty(),
            "catalog compiler version is required"
        );
        let mut previous = "";
        let mut paths = BTreeSet::new();
        let mut counts = BTreeMap::new();
        let mut ranges = BTreeMap::<&str, &str>::new();
        for file in &self.files {
            source_path(&file.path)?;
            ensure!(
                previous < file.path.as_str() && paths.insert(file.path.to_ascii_lowercase()),
                "catalog paths must be sorted and unique"
            );
            previous = &file.path;
            ensure!(
                file.bytes > 0
                    && file.bytes
                        <= if file.kind == "image" {
                            IMAGE_LIMIT
                        } else {
                            FILE_LIMIT
                        } as u64
                    && is_digest(&file.sha256),
                "invalid catalog file descriptor: {}",
                file.path
            );
            if file.kind == "image" {
                ensure!(
                    file.encoding == "webp"
                        && file.path == format!("textures/{}.webp", file.sha256),
                    "invalid texture page path"
                );
                ensure!(
                    file.rows == 0 && file.first.is_none() && file.last.is_none(),
                    "texture page cannot declare record counts"
                );
            } else {
                ensure!(
                    Table::KINDS.contains(&file.kind.as_str()) && file.encoding == "msgpack",
                    "unsupported catalog table"
                );
                let prefix = format!("tables/{}/part-", file.kind);
                let part = file
                    .path
                    .strip_prefix(&prefix)
                    .and_then(|part| part.strip_suffix(".msgpack"));
                ensure!(
                    part.is_some_and(
                        |part| part.len() == 6 && part.bytes().all(|c| c.is_ascii_digit())
                    ),
                    "invalid catalog table path"
                );
                ensure!(
                    file.rows <= TABLE_ROWS as u64,
                    "catalog table exceeds record limit"
                );
                match (file.rows, &file.first, &file.last) {
                    (0, None, None) => {}
                    (1.., Some(first), Some(last)) => {
                        ensure!(
                            !first.is_empty()
                                && first.is_ascii()
                                && last.is_ascii()
                                && first <= last
                                && last.len() <= 256,
                            "invalid catalog table range"
                        );
                        if let Some(previous) = ranges.insert(&file.kind, last) {
                            ensure!(
                                previous < first.as_str(),
                                "overlapping catalog table ranges"
                            );
                        }
                    }
                    _ => anyhow::bail!("record range does not match row count"),
                }
                *counts.entry(file.kind.clone()).or_insert(0) += file.rows;
            }
        }
        ensure!(
            counts.len() == Table::KINDS.len()
                && Table::KINDS.iter().all(|kind| counts.contains_key(*kind))
                && counts == self.counts,
            "catalog collection counts do not match file descriptors"
        );
        Ok(())
    }
}

pub struct Catalog {
    pub root: PathBuf,
    pub manifest: Manifest,
}

impl Catalog {
    pub fn open(root: &Path) -> Result<Self> {
        require_plain(root)?;
        let root = fs::canonicalize(root)?;
        require_plain(&root.join("manifest.json"))?;
        let manifest: Manifest =
            serde_json::from_slice(&read(&root.join("manifest.json"), 64 * 1024 * 1024)?)?;
        manifest.validate()?;
        Ok(Self { root, manifest })
    }

    pub fn current(root: &Path) -> Result<Self> {
        require_plain(root)?;
        let path = root.join("current.json");
        require_plain(&path)?;
        let pointer: Pointer = serde_json::from_slice(&read(&path, 4096)?)?;
        ensure!(
            pointer.format == "elysium.catalog-pointer"
                && pointer.revision == REVISION
                && is_digest(&pointer.id),
            "invalid catalog pointer"
        );
        require_plain(&root.join("catalogs"))?;
        let catalog = Self::open(&root.join("catalogs").join(&pointer.id))?;
        ensure!(
            catalog.manifest.id == pointer.id,
            "catalog pointer identity mismatch"
        );
        Ok(catalog)
    }

    pub fn read(&self, file: &File) -> Result<Vec<u8>> {
        let mut path = self.root.clone();
        for component in source_path(&file.path)?.components() {
            path.push(component);
            require_plain(&path)?;
        }
        let bytes = read(&path, file.bytes as usize)?;
        ensure!(
            bytes.len() as u64 == file.bytes && hash(&bytes) == file.sha256,
            "catalog file integrity mismatch: {}",
            file.path
        );
        Ok(bytes)
    }

    pub fn verify(&self) -> Result<()> {
        for file in &self.manifest.files {
            let bytes = self.read(file)?;
            if file.kind == "image" {
                continue;
            }
            let table: Table =
                rmp_serde::from_slice(&bytes).with_context(|| format!("decode {}", file.path))?;
            let (kind, ids) = table.keys();
            ensure!(
                kind == file.kind
                    && ids.len() as u64 == file.rows
                    && ids.first().copied() == file.first.as_deref()
                    && ids.last().copied() == file.last.as_deref()
                    && ids.windows(2).all(|pair| pair[0] < pair[1]),
                "catalog table range or count mismatch: {}",
                file.path
            );
        }
        Ok(())
    }
}

pub(super) struct Writer {
    root: PathBuf,
    staging: TempDir,
    _lock: std::fs::File,
    manifest: Manifest,
}

impl Writer {
    pub fn new(root: &Path, source: &Source) -> Result<Self> {
        let root = destination(root)?;
        ensure!(
            !root.starts_with(source.root()),
            "catalog output cannot be inside a source dataset"
        );
        fs::create_dir_all(&root)?;
        require_plain(&root)?;
        let lock_path = root.join(".lock");
        if lock_path.exists() {
            require_plain(&lock_path)?;
        }
        let lock = OpenOptions::new()
            .create(true)
            .truncate(false)
            .read(true)
            .write(true)
            .open(lock_path)?;
        lock.try_lock_exclusive()
            .context("another compiler is publishing to this catalog directory")?;
        let catalogs = destination(&root.join("catalogs"))?;
        fs::create_dir_all(&catalogs)?;
        require_plain(&catalogs)?;
        if root.join("current.json").exists() {
            Catalog::current(&root)?.verify()?;
        }
        let staging = tempfile::Builder::new()
            .prefix(".capture-")
            .tempdir_in(&catalogs)?;
        Ok(Self {
            root,
            staging,
            _lock: lock,
            manifest: Manifest {
                format: FORMAT.to_owned(),
                revision: REVISION,
                id: String::new(),
                source: source.manifest.id.clone(),
                environment: source.manifest.environment.clone(),
                compiler: env!("CARGO_PKG_VERSION").to_owned(),
                scope: source.manifest.scope.mode.clone(),
                files: Vec::new(),
                counts: BTreeMap::new(),
            },
        })
    }

    fn file(&mut self, descriptor: File, bytes: &[u8]) -> Result<()> {
        ensure!(
            bytes.len()
                <= if descriptor.kind == "image" {
                    IMAGE_LIMIT
                } else {
                    FILE_LIMIT
                },
            "catalog file exceeds its size budget: {}",
            descriptor.path
        );
        source_path(&descriptor.path)?;
        let path = self.staging.path().join(&descriptor.path);
        fs::create_dir_all(path.parent().context("file requires a parent")?)?;
        let mut output = OpenOptions::new().create_new(true).write(true).open(path)?;
        output.write_all(bytes)?;
        output.sync_all()?;
        self.manifest.files.push(descriptor);
        Ok(())
    }

    pub fn image(&mut self, bytes: &[u8]) -> Result<String> {
        let sha256 = hash(bytes);
        let path = format!("textures/{sha256}.webp");
        if self.manifest.files.iter().any(|file| file.path == path) {
            return Ok(path);
        }
        self.file(
            File {
                path: path.clone(),
                kind: "image".to_owned(),
                encoding: "webp".to_owned(),
                bytes: bytes.len() as u64,
                rows: 0,
                sha256,
                first: None,
                last: None,
            },
            bytes,
        )?;
        Ok(path)
    }

    pub fn table<T>(
        &mut self,
        kind: &str,
        rows: &[T],
        wrap: impl Fn(&[T]) -> Table,
        id: impl Fn(&T) -> &str,
    ) -> Result<()> {
        let mut start = 0;
        let mut part = 0;
        loop {
            let mut end = rows.len().min(start + TABLE_ROWS);
            let bytes = loop {
                let bytes = rmp_serde::to_vec_named(&wrap(&rows[start..end]))?;
                if bytes.len() <= FILE_LIMIT {
                    break bytes;
                }
                ensure!(
                    end - start > 1,
                    "single {kind} record exceeds catalog page limit"
                );
                end = start + (end - start) / 2;
            };
            let count = end - start;
            self.file(
                File {
                    path: format!("tables/{kind}/part-{part:06}.msgpack"),
                    kind: kind.to_owned(),
                    encoding: "msgpack".to_owned(),
                    bytes: bytes.len() as u64,
                    rows: count as u64,
                    sha256: hash(&bytes),
                    first: rows
                        .get(start)
                        .filter(|_| count > 0)
                        .map(|row| id(row).to_owned()),
                    last: end
                        .checked_sub(1)
                        .and_then(|last| rows.get(last))
                        .filter(|_| count > 0)
                        .map(|row| id(row).to_owned()),
                },
                &bytes,
            )?;
            start = end;
            part += 1;
            if start == rows.len() {
                break;
            }
        }
        self.manifest
            .counts
            .insert(kind.to_owned(), rows.len() as u64);
        Ok(())
    }

    pub fn publish(mut self) -> Result<Publication> {
        self.manifest
            .files
            .sort_by(|left, right| left.path.cmp(&right.path));
        self.manifest.id = self.manifest.digest()?;
        self.manifest.validate()?;
        atomic_json(&self.staging.path().join("manifest.json"), &self.manifest)?;
        Catalog::open(self.staging.path())?.verify()?;
        let target = self.root.join("catalogs").join(&self.manifest.id);
        if target.exists() {
            let existing = Catalog::open(&target)?;
            ensure!(
                existing.manifest.id == self.manifest.id,
                "catalog directory contains different content"
            );
            existing.verify()?;
        } else {
            fs::rename(self.staging.path(), &target)
                .context("publish immutable catalog directory")?;
        }
        atomic_json(
            &self.root.join("current.json"),
            &Pointer {
                format: "elysium.catalog-pointer".to_owned(),
                revision: REVISION,
                id: self.manifest.id.clone(),
            },
        )?;
        Ok(Publication {
            id: self.manifest.id,
            source: self.manifest.source,
            path: target,
            counts: self.manifest.counts,
            bytes: self.manifest.files.iter().map(|file| file.bytes).sum(),
        })
    }
}

pub fn atomic_json(path: &Path, value: &impl Serialize) -> Result<()> {
    let path = destination(path)?;
    let parent = path
        .parent()
        .context("output file requires a parent directory")?;
    fs::create_dir_all(parent)?;
    require_plain(parent)?;
    if path.exists() {
        require_plain(&path)?;
    }
    let mut temporary = NamedTempFile::new_in(parent)?;
    let value = serde_json::to_value(value)?;
    temporary.write_all(&serde_json::to_vec(&value)?)?;
    temporary.as_file().sync_all()?;
    temporary
        .persist(&path)
        .context("atomically replace JSON file")?;
    Ok(())
}

/// Validate existing ancestors before creating any output, including Windows junctions.
pub(super) fn destination(path: &Path) -> Result<PathBuf> {
    ensure!(
        !path
            .components()
            .any(|part| matches!(part, std::path::Component::ParentDir)),
        "output path must not contain parent-directory segments"
    );
    let absolute = std::path::absolute(path)?;
    let mut existing = None;
    for ancestor in absolute.ancestors() {
        match fs::symlink_metadata(ancestor) {
            Ok(_) => {
                require_plain(ancestor)?;
                if existing.is_none() {
                    existing = Some(ancestor);
                }
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(error.into()),
        }
    }
    let existing = existing.context("output path has no existing ancestor")?;
    Ok(fs::canonicalize(existing)?.join(absolute.strip_prefix(existing)?))
}

fn read(path: &Path, limit: usize) -> Result<Vec<u8>> {
    let mut bytes = Vec::new();
    std::fs::File::open(path)?
        .take(limit as u64 + 1)
        .read_to_end(&mut bytes)?;
    ensure!(
        bytes.len() <= limit,
        "file exceeds size limit: {}",
        path.display()
    );
    Ok(bytes)
}

fn hash(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
