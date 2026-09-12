//! The only on-disk source envelope. Collections and assets are declared and hashed.
use crate::domain::Probe;
use anyhow::{bail, ensure, Context, Result};
use flate2::read::MultiGzDecoder;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::fs::{self, File};
use std::io::{BufRead, BufReader, Read};
use std::path::{Component, Path, PathBuf};

pub const SOURCE_FORMAT: &str = "elysium.source";
pub const SOURCE_REVISION: u32 = 10;
pub const CORE_COLLECTIONS: &[&str] = &[
    "aspects",
    "assets",
    "blocks",
    "builds",
    "categories",
    "circuits",
    "fluids",
    "groups",
    "items",
    "materials",
    "models",
    "mutations",
    "recipes",
    "research",
    "shapes",
    "species",
    "strings",
    "structures",
    "tracks",
    "views",
];
const MANIFEST_LIMIT: u64 = 64 * 1024 * 1024;
const RECORD_LIMIT: u64 = 1024 * 1024;
const SHARD_LIMIT: u64 = 16 * 1024 * 1024;
const ASSET_LIMIT: u64 = 64 * 1024 * 1024;

#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Producer {
    pub name: String,
    pub version: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Scope {
    pub mode: String,
    pub collections: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SourceFile {
    pub path: String,
    pub kind: String,
    pub encoding: String,
    pub bytes: u64,
    pub decoded_bytes: u64,
    pub rows: u64,
    pub sha256: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SourceManifest {
    #[schemars(schema_with = "source_format")]
    pub format: String,
    #[schemars(schema_with = "source_revision")]
    pub revision: u32,
    pub id: String,
    pub producer: Producer,
    pub environment: String,
    pub scope: Scope,
    pub files: Vec<SourceFile>,
}

pub(crate) fn literal_schema(value: impl Into<Value>) -> schemars::schema::Schema {
    schemars::schema::SchemaObject {
        enum_values: Some(vec![value.into()]),
        ..Default::default()
    }
    .into()
}

fn source_format(_: &mut schemars::gen::SchemaGenerator) -> schemars::schema::Schema {
    literal_schema(SOURCE_FORMAT)
}
fn source_revision(_: &mut schemars::gen::SchemaGenerator) -> schemars::schema::Schema {
    literal_schema(SOURCE_REVISION)
}

#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Mod {
    pub id: String,
    pub name: String,
    pub version: String,
    pub sha256: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct EnvironmentFile {
    pub path: String,
    pub sha256: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Environment {
    pub game: String,
    pub loader: String,
    pub locale: String,
    pub mods: Vec<Mod>,
    pub inputs: Vec<EnvironmentFile>,
    /// Selected resource packs, in the game's priority order.
    pub resources: Vec<String>,
    /// Construction parameters, ordered by count then channel name/value pairs.
    #[schemars(length(min = 1, max = 16))]
    pub probes: Vec<Probe>,
    pub settings: BTreeMap<String, String>,
    /// Digests of knowledge states used by adapters; no player names or world locations.
    pub knowledge: BTreeMap<String, String>,
}

impl SourceManifest {
    pub fn digest(&self) -> Result<String> {
        let mut value = serde_json::to_value(self)?;
        value.as_object_mut().expect("manifest object").remove("id");
        Ok(format!("{:x}", Sha256::digest(serde_json::to_vec(&value)?)))
    }

    pub fn validate(&self) -> Result<()> {
        ensure!(
            self.format == SOURCE_FORMAT,
            "expected {SOURCE_FORMAT}, found {}",
            self.format
        );
        ensure!(
            self.revision == SOURCE_REVISION,
            "unsupported source revision: {}",
            self.revision
        );
        ensure!(
            is_digest(&self.id) && is_digest(&self.environment),
            "source ids must be SHA-256 digests"
        );
        ensure!(
            !self.producer.name.is_empty() && !self.producer.version.is_empty(),
            "source producer is required"
        );
        ensure!(
            matches!(self.scope.mode.as_str(), "complete" | "selection"),
            "invalid source scope mode"
        );
        ensure!(
            !self.scope.collections.is_empty(),
            "source has no collections"
        );
        let mut collections = BTreeSet::new();
        for name in &self.scope.collections {
            ensure!(is_collection(name), "invalid collection name: {name}");
            ensure!(
                collections.insert(name.as_str()),
                "duplicate collection: {name}"
            );
        }
        ensure!(
            self.scope
                .collections
                .iter()
                .map(String::as_str)
                .eq(collections.iter().copied()),
            "collections must be sorted"
        );
        if self.scope.mode == "complete" {
            for name in CORE_COLLECTIONS {
                ensure!(
                    collections.contains(name),
                    "complete source is missing collection {name}"
                );
            }
        }
        let mut paths = BTreeSet::new();
        let mut declared = BTreeSet::new();
        let mut previous = "";
        let mut environment = false;
        for file in &self.files {
            source_path(&file.path)?;
            ensure!(
                previous < file.path.as_str(),
                "source files must have unique sorted paths"
            );
            previous = &file.path;
            ensure!(
                paths.insert(file.path.to_ascii_lowercase()),
                "case-colliding source path: {}",
                file.path
            );
            ensure!(
                is_digest(&file.sha256),
                "invalid file digest: {}",
                file.path
            );
            ensure!(
                file.bytes > 0 && file.bytes <= ASSET_LIMIT,
                "invalid file size: {}",
                file.path
            );
            if file.kind == "environment" {
                ensure!(
                    !environment
                        && file.path == "environment.json"
                        && file.encoding == "json"
                        && file.sha256 == self.environment
                        && file.rows == 1
                        && file.bytes == file.decoded_bytes,
                    "invalid environment descriptor"
                );
                environment = true;
            } else if file.kind == "asset" {
                ensure!(
                    matches!(file.encoding.as_str(), "png" | "webp"),
                    "unsupported asset encoding: {}",
                    file.encoding
                );
                ensure!(
                    file.path == format!("assets/{}.{}", file.sha256, file.encoding),
                    "asset path must contain its digest"
                );
                ensure!(
                    file.rows == 0 && file.decoded_bytes == file.bytes,
                    "invalid asset counts: {}",
                    file.path
                );
            } else {
                ensure!(
                    collections.contains(file.kind.as_str()),
                    "undeclared collection: {}",
                    file.kind
                );
                ensure!(
                    file.encoding == "jsonl.gzip",
                    "unsupported record encoding: {}",
                    file.encoding
                );
                let prefix = format!("records/{}/part-", file.kind);
                let part = file
                    .path
                    .strip_prefix(&prefix)
                    .and_then(|path| path.strip_suffix(".jsonl.gz"));
                ensure!(
                    part.is_some_and(
                        |part| part.len() == 6 && part.bytes().all(|byte| byte.is_ascii_digit())
                    ),
                    "invalid shard path: {}",
                    file.path
                );
                ensure!(
                    file.rows <= 4096 && file.decoded_bytes <= SHARD_LIMIT,
                    "source shard exceeds size limit: {}",
                    file.path
                );
                declared.insert(file.kind.as_str());
            }
        }
        ensure!(
            collections == declared,
            "every collection must declare at least one shard, including empty collections"
        );
        ensure!(environment, "source must declare its environment facts");
        ensure!(self.id == self.digest()?, "source manifest digest mismatch");
        Ok(())
    }
}

pub struct Source {
    root: PathBuf,
    pub manifest: SourceManifest,
}

impl Source {
    pub fn open(root: &Path) -> Result<Self> {
        require_plain(root)?;
        ensure!(root.is_dir(), "source root must be a directory");
        let root = fs::canonicalize(root)?;
        let path = root.join("manifest.json");
        require_plain(&path)?;
        let mut bytes = Vec::new();
        File::open(&path)
            .with_context(|| format!("open {}", path.display()))?
            .take(MANIFEST_LIMIT + 1)
            .read_to_end(&mut bytes)?;
        ensure!(
            bytes.len() as u64 <= MANIFEST_LIMIT,
            "source manifest exceeds size limit"
        );
        let manifest: SourceManifest =
            serde_json::from_slice(&bytes).context("decode source manifest")?;
        manifest.validate()?;
        Ok(Self { root, manifest })
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn environment(&self) -> Result<Environment> {
        let file = self
            .files("environment")
            .next()
            .context("environment facts are missing")?;
        let bytes = self.read_file(file)?;
        let value: Value = serde_json::from_slice(&bytes)?;
        ensure!(
            serde_json::to_vec(&value)? == bytes,
            "environment must use canonical JSON"
        );
        let environment: Environment = serde_json::from_value(value)?;
        Probe::validate_all(&environment.probes)?;
        ensure!(
            !environment.game.is_empty()
                && !environment.loader.is_empty()
                && !environment.locale.is_empty(),
            "environment requires game, loader and locale"
        );
        ensure!(
            environment
                .mods
                .windows(2)
                .all(|pair| pair[0].id < pair[1].id),
            "environment mods must be unique and sorted"
        );
        for item in &environment.mods {
            ensure!(
                !item.id.is_empty() && !item.version.is_empty() && is_digest(&item.sha256),
                "invalid environment mod"
            );
        }
        ensure!(
            environment
                .inputs
                .windows(2)
                .all(|pair| pair[0].path < pair[1].path),
            "environment files must be unique and sorted"
        );
        for file in &environment.inputs {
            ensure!(
                !file.path.is_empty()
                    && !file.path.starts_with('/')
                    && !file.path.contains('\\')
                    && !file.path.contains(':')
                    && !file
                        .path
                        .split('/')
                        .any(|part| part == ".." || part == "." || part.is_empty())
                    && is_digest(&file.sha256),
                "invalid environment file fingerprint"
            );
        }
        ensure!(
            environment
                .knowledge
                .values()
                .all(|digest| is_digest(digest)),
            "invalid knowledge fingerprint"
        );
        Ok(environment)
    }

    pub fn files<'a>(&'a self, kind: &'a str) -> impl Iterator<Item = &'a SourceFile> + 'a {
        self.manifest
            .files
            .iter()
            .filter(move |file| file.kind == kind)
    }

    pub fn path(&self, relative: &str) -> Result<PathBuf> {
        let relative = source_path(relative)?;
        let mut path = self.root.clone();
        for component in relative.components() {
            path.push(component);
            require_plain(&path)?;
        }
        ensure!(
            path.is_file(),
            "source entry must be a regular file: {}",
            path.display()
        );
        Ok(path)
    }

    pub fn read_file(&self, descriptor: &SourceFile) -> Result<Vec<u8>> {
        let path = self.path(&descriptor.path)?;
        let mut bytes = Vec::new();
        File::open(&path)?
            .take(descriptor.bytes + 1)
            .read_to_end(&mut bytes)?;
        ensure!(
            bytes.len() as u64 == descriptor.bytes,
            "file size mismatch: {}",
            descriptor.path
        );
        ensure!(
            format!("{:x}", Sha256::digest(&bytes)) == descriptor.sha256,
            "file digest mismatch: {}",
            descriptor.path
        );
        Ok(bytes)
    }

    pub fn visit(&self, kind: &str, mut consume: impl FnMut(Value) -> Result<()>) -> Result<u64> {
        ensure!(
            self.manifest
                .scope
                .collections
                .iter()
                .any(|name| name == kind),
            "source collection is missing: {kind}"
        );
        let mut count = 0;
        let mut previous: Option<String> = None;
        for descriptor in self.files(kind) {
            let bytes = self.read_file(descriptor)?;
            let mut reader = BufReader::new(MultiGzDecoder::new(bytes.as_slice()));
            let mut rows = 0;
            let mut decoded = 0;
            loop {
                let mut line = Vec::new();
                let size = (&mut reader)
                    .take(RECORD_LIMIT + 2)
                    .read_until(b'\n', &mut line)?;
                if size == 0 {
                    break;
                }
                decoded += size as u64;
                ensure!(
                    size as u64 <= RECORD_LIMIT + 1 && line.last() == Some(&b'\n'),
                    "invalid or oversized record in {}",
                    descriptor.path
                );
                ensure!(
                    decoded <= descriptor.decoded_bytes && rows < descriptor.rows,
                    "record counts exceed manifest: {}",
                    descriptor.path
                );
                let value: Value = serde_json::from_slice(&line)
                    .with_context(|| format!("decode {} row {}", descriptor.path, rows + 1))?;
                ensure!(serde_json::to_vec(&value)? == line[..line.len() - 1],
                    "record must use canonical JSON (unique ordered keys and exact numbers): {} row {}",
                    descriptor.path, rows + 1);
                let id = value
                    .get("id")
                    .and_then(Value::as_str)
                    .context("source record requires an id")?;
                ensure!(
                    !id.is_empty() && id.len() <= 256 && id.is_ascii(),
                    "invalid source record id"
                );
                ensure!(
                    previous
                        .as_ref()
                        .is_none_or(|previous| previous.as_str() < id),
                    "duplicate or unordered {kind} id: {id}"
                );
                previous = Some(id.to_owned());
                consume(value)?;
                rows += 1;
            }
            ensure!(
                rows == descriptor.rows && decoded == descriptor.decoded_bytes,
                "record counts differ from manifest: {}",
                descriptor.path
            );
            count += rows;
        }
        Ok(count)
    }

    pub fn verify(&self) -> Result<()> {
        self.environment()?;
        for kind in &self.manifest.scope.collections {
            self.visit(kind, |_| Ok(()))?;
        }
        for file in self.files("asset") {
            self.read_file(file)?;
        }
        Ok(())
    }
}

pub fn is_digest(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn is_collection(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 48
        && value != "asset"
        && value != "environment"
        && value.as_bytes()[0].is_ascii_lowercase()
        && value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
}

pub fn source_path(value: &str) -> Result<PathBuf> {
    ensure!(
        !value.is_empty() && value.is_ascii() && value.len() <= 240,
        "invalid source path: {value}"
    );
    ensure!(
        !value.contains(['\\', ':', '%', '?', '*', '"', '<', '>', '|']),
        "source path is not portable: {value}"
    );
    for segment in value.split('/') {
        ensure!(
            !segment.is_empty()
                && segment != "."
                && segment != ".."
                && !segment.ends_with(['.', ' ']),
            "invalid source path segment: {value}"
        );
        ensure!(
            segment.bytes().all(|byte| (33..127).contains(&byte)),
            "invalid source path byte: {value}"
        );
        let stem = segment
            .split('.')
            .next()
            .unwrap_or_default()
            .to_ascii_uppercase();
        ensure!(
            !matches!(stem.as_str(), "CON" | "PRN" | "AUX" | "NUL")
                && !(stem.len() == 4
                    && (stem.starts_with("COM") || stem.starts_with("LPT"))
                    && matches!(stem.as_bytes()[3], b'1'..=b'9')),
            "reserved source path: {value}"
        );
    }
    let path = PathBuf::from(value);
    ensure!(
        path.components()
            .all(|part| matches!(part, Component::Normal(_))),
        "source path must be relative: {value}"
    );
    Ok(path)
}

pub(crate) fn require_plain(path: &Path) -> Result<()> {
    let metadata =
        fs::symlink_metadata(path).with_context(|| format!("inspect {}", path.display()))?;
    if metadata.file_type().is_symlink() {
        bail!("source path must not be a link: {}", path.display());
    }
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        ensure!(
            metadata.file_attributes() & 0x400 == 0,
            "source path must not be a reparse point: {}",
            path.display()
        );
    }
    Ok(())
}
