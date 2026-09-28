//! Offline resource candidates. This inventory is neither a Source nor proof of
//! game resource precedence; native resolution is required before binding assets.
mod directory;

use crate::catalog::store::destination;
use crate::source::{is_digest, source_path};
use anyhow::{ensure, Context, Result};
use fs2::FileExt;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use tempfile::{NamedTempFile, TempDir};
use zip::ZipArchive;

const FORMAT: &str = "elysium.resources";
const REVISION: u32 = 1;
const ARCHIVE_LIMIT: u64 = 2 * 1024 * 1024 * 1024;
const INPUT_LIMIT: u64 = 16 * ARCHIVE_LIMIT;
const ENTRY_LIMIT: u64 = 64 * 1024 * 1024;
const OUTPUT_LIMIT: u64 = 8 * 1024 * 1024 * 1024;
const ROW_LIMIT: usize = 500_000;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Input {
    format: String,
    revision: u32,
    archives: Vec<Archive>,
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Archive {
    key: String,
    #[serde(skip_serializing)]
    path: PathBuf,
    bytes: u64,
    sha256: String,
}

#[derive(Serialize)]
struct Entry {
    archive: String,
    path: String,
    bytes: u64,
    sha256: String,
    blob: String,
}

/// Publish a fresh portable inventory after all archives and resources pass.
pub fn import(input: &Path, output: &Path) -> Result<Value> {
    let input = destination(input)?;
    ensure!(input.is_file(), "resource input must be a regular file");
    let mut bytes = Vec::new();
    File::open(&input)?
        .take(4 * 1024 * 1024 + 1)
        .read_to_end(&mut bytes)?;
    ensure!(
        bytes.len() <= 4 * 1024 * 1024,
        "resource input exceeds limit"
    );
    let mut manifest: Input = serde_json::from_slice(&bytes)?;
    ensure!(
        manifest.format == FORMAT && manifest.revision == REVISION,
        "unsupported resource input"
    );
    ensure!(
        !manifest.archives.is_empty() && manifest.archives.len() <= 1024,
        "invalid archive count"
    );
    let mut keys = BTreeSet::new();
    let mut input_bytes = 0;
    for archive in &manifest.archives {
        source_path(&archive.key)?;
        ensure!(
            keys.insert(archive.key.clone()),
            "duplicate archive key: {}",
            archive.key
        );
        ensure!(
            archive.bytes > 0 && archive.bytes <= ARCHIVE_LIMIT && is_digest(&archive.sha256),
            "invalid archive descriptor: {}",
            archive.key
        );
        input_bytes += archive.bytes;
        ensure!(
            input_bytes <= INPUT_LIMIT,
            "archive inputs exceed total limit"
        );
    }
    manifest.archives.sort_by(|a, b| a.key.cmp(&b.key));

    let output = destination(output)?;
    ensure!(!output.exists(), "resource output already exists");
    let parent = output.parent().context("resource output has no parent")?;
    fs::create_dir_all(parent)?;
    // Serialize publication by cooperating importers without changing existing results.
    let lock_path = destination(&parent.join(".resources.lock"))?;
    let lock = OpenOptions::new()
        .create(true)
        .truncate(false)
        .read(true)
        .write(true)
        .open(lock_path)?;
    lock.try_lock_exclusive()
        .context("another resource import is using this directory")?;
    ensure!(!output.exists(), "resource output already exists");
    let stage = TempDir::new_in(parent)?;
    fs::create_dir(stage.path().join("blobs"))?;
    let mut entries = Vec::new();
    let mut output_bytes = 0_u64;
    let base = input.parent().context("resource input has no parent")?;
    for archive in &manifest.archives {
        capture(base, stage.path(), archive, &mut entries, &mut output_bytes)
            .with_context(|| format!("archive {}", archive.key))?;
    }
    entries.sort_by(|a, b| (&a.path, &a.archive).cmp(&(&b.path, &b.archive)));
    let mut result = json!({"format":FORMAT,"revision":REVISION,"resolution":"unverified",
        "archives":manifest.archives,"entries":entries});
    let id = format!("{:x}", Sha256::digest(serde_json::to_vec(&result)?));
    result["id"] = json!(id);
    let mut file = File::create(stage.path().join("manifest.json"))?;
    file.write_all(&serde_json::to_vec_pretty(&result)?)?;
    file.write_all(b"\n")?;
    file.sync_all()?;
    drop(file);
    ensure!(!output.exists(), "resource output already exists");
    fs::rename(stage.path(), &output).context("publish resource inventory")?;
    Ok(
        json!({"id":id,"output":output,"archives":manifest.archives.len(),
        "entries":entries.len(),"resourceBytes":output_bytes,"resolution":"unverified"}),
    )
}

fn capture(
    base: &Path,
    stage: &Path,
    archive: &Archive,
    entries: &mut Vec<Entry>,
    output_bytes: &mut u64,
) -> Result<()> {
    let path = destination(&base.join(&archive.path))?;
    let mut source = File::open(&path)?;
    ensure!(
        source.metadata()?.is_file() && source.metadata()?.len() == archive.bytes,
        "archive size or file type mismatch"
    );
    // Parse only a verified private snapshot. Reopening the original after hashing
    // would allow a changed jar to be indexed under the old digest.
    let mut snapshot = NamedTempFile::new_in(stage)?;
    let mut hasher = Sha256::new();
    let mut total = 0;
    let mut buffer = [0_u8; 65536];
    loop {
        let count = source.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        total += count as u64;
        ensure!(total <= archive.bytes, "archive grew during capture");
        hasher.update(&buffer[..count]);
        snapshot.write_all(&buffer[..count])?;
    }
    ensure!(
        total == archive.bytes && format!("{:x}", hasher.finalize()) == archive.sha256,
        "archive digest mismatch"
    );
    snapshot.flush()?;
    let count = directory::check(snapshot.as_file_mut(), total)?;
    snapshot.seek(SeekFrom::Start(0))?;
    let mut zip = ZipArchive::new(snapshot.as_file_mut())?;
    ensure!(zip.len() == count, "archive directory entry count mismatch");
    for index in 0..zip.len() {
        let mut file = zip.by_index(index)?;
        let name = std::str::from_utf8(file.name_raw())
            .context("archive name is not UTF-8")?
            .to_owned();
        if !name.starts_with("assets/") {
            continue;
        }
        logical_path(&name)?;
        if file.is_dir() || !selected(&name) {
            continue;
        }
        ensure!(
            !file.encrypted()
                && file.is_file()
                && file
                    .unix_mode()
                    .is_none_or(|mode| mode & 0o170000 == 0 || mode & 0o170000 == 0o100000),
            "resource must be an unencrypted regular file: {name}"
        );
        ensure!(
            file.size() <= ENTRY_LIMIT,
            "resource exceeds size limit: {name}"
        );
        ensure!(
            entries.len() < ROW_LIMIT && *output_bytes + file.size() <= OUTPUT_LIMIT,
            "resource inventory exceeds limit"
        );
        let size = file.size();
        let mut bytes = Vec::new();
        (&mut file)
            .take(ENTRY_LIMIT + 1)
            .read_to_end(&mut bytes)
            .with_context(|| format!("read resource {name}"))?;
        ensure!(bytes.len() as u64 == size, "resource size mismatch: {name}");
        *output_bytes += size;
        let sha256 = format!("{:x}", Sha256::digest(&bytes));
        let blob = format!("blobs/{sha256}");
        if !stage.join(&blob).exists() {
            let mut output = OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(stage.join(&blob))?;
            output.write_all(&bytes)?;
            output.sync_all()?;
        }
        entries.push(Entry {
            archive: archive.key.clone(),
            path: name,
            bytes: size,
            sha256,
            blob,
        });
    }
    Ok(())
}

fn logical_path(path: &str) -> Result<()> {
    ensure!(
        path.len() <= 4096
            && !path
                .chars()
                .any(|c| c.is_control() || c == '\\' || c == ':'),
        "invalid resource path: {path}"
    );
    ensure!(
        path.trim_end_matches('/')
            .split('/')
            .all(|part| !part.is_empty() && part != "." && part != ".."),
        "invalid resource path: {path}"
    );
    Ok(())
}

fn selected(path: &str) -> bool {
    let parts: Vec<_> = path.split('/').collect();
    parts.len() >= 4
        && match parts[2] {
            "textures" => path.ends_with(".png") || path.ends_with(".png.mcmeta"),
            "lang" => path.ends_with(".lang") || path.ends_with(".json"),
            _ => false,
        }
}
