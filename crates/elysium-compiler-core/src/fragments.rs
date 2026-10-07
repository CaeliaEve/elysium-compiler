//! Restores the exact Source file set; diagnostic findings and unfinished captures cannot publish.
use crate::catalog::store::destination;
use crate::domain::Domain;
use crate::provenance::Provenance;
use crate::source::{is_digest, require_plain, Source, SourceFile, SourceManifest, MANIFEST_LIMIT};
use anyhow::{ensure, Context, Result};
use fs2::FileExt;
use serde::Deserialize;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    path::Path,
};

// v1 repeats the descriptor list inside source; the Source itself stays <=64 MiB.
const LIMIT: u64 = 2 * MANIFEST_LIMIT + 1024 * 1024;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Capture {
    format: String,
    revision: u32,
    id: String,
    state: String,
    provenance: Value,
    request: Value,
    files: Vec<SourceFile>,
    source: SourceManifest,
}

fn hash(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
fn read(path: &Path, limit: u64) -> Result<Vec<u8>> {
    require_plain(path)?;
    let file = File::open(path)?;
    ensure!(
        file.metadata()?.is_file(),
        "capture input is not a regular file"
    );
    let mut bytes = Vec::new();
    file.take(limit + 1).read_to_end(&mut bytes)?;
    ensure!(
        bytes.len() as u64 <= limit,
        "capture input exceeds size limit"
    );
    Ok(bytes)
}

pub fn assemble(input: &Path, expected: &str, output: &Path) -> Result<Value> {
    let input = destination(input)?;
    let output = destination(output)?;
    ensure!(
        input.is_dir() && !output.starts_with(&input) && !input.starts_with(&output),
        "Source output must be separate from its capture"
    );
    ensure!(
        is_digest(expected),
        "expected capture manifest digest is required"
    );
    let bytes = read(&input.join("manifest.json"), LIMIT)?;
    ensure!(hash(&bytes) == expected, "capture manifest digest mismatch");
    let mut raw: Value = serde_json::from_slice(&bytes)?;
    ensure!(raw["state"] == "complete", "capture is not complete");
    ensure!(
        serde_json::to_vec(&raw["source"])?.len() as u64 <= MANIFEST_LIMIT,
        "source manifest exceeds size limit"
    );
    let capture: Capture = serde_json::from_value(raw.clone())?;
    raw.as_object_mut()
        .context("invalid capture manifest")?
        .remove("id");
    ensure!(
        capture.format == "elysium.capture"
            && capture.revision == 1
            && capture.state == "complete"
            && is_digest(&capture.id)
            && capture.id == hash(&serde_json::to_vec(&raw)?),
        "invalid capture identity"
    );
    capture.source.validate()?;
    ensure!(
        capture.request["scope"].is_null() || capture.source.scope.mode == "selection",
        "scoped capture cannot claim complete Source coverage"
    );
    ensure!(
        capture.source.producer.name == "nesql" && capture.request["check"].is_null(),
        "capture is not a native export"
    );
    ensure!(
        serde_json::to_value(&capture.files)? == serde_json::to_value(&capture.source.files)?,
        "capture must contain exactly the declared Source files"
    );
    let mut total = 0_u64;
    for file in &capture.files {
        total = total
            .checked_add(file.bytes)
            .context("capture size overflow")?;
        ensure!(total <= 128 * 1024 * 1024 * 1024, "capture exceeds 128 GiB");
    }
    require_plain(&input.join("blobs"))?;
    let parent = output.parent().context("Source output needs a parent")?;
    fs::create_dir_all(parent)?;
    require_plain(parent)?;
    let lock_path = destination(&parent.join(".source.lock"))?;
    let lock = OpenOptions::new()
        .create(true)
        .truncate(false)
        .read(true)
        .write(true)
        .open(lock_path)?;
    lock.try_lock_exclusive()
        .context("another Source assembly is using this directory")?;
    let reused = output.exists();
    if reused {
        let source = Source::open(&output)?;
        ensure!(
            source.manifest.id == capture.source.id,
            "output already contains a different Source"
        );
    }
    let stage = if reused {
        None
    } else {
        Some(tempfile::TempDir::new_in(parent)?)
    };
    let mut environment = None;
    for descriptor in &capture.files {
        let bytes = read(
            &input.join("blobs").join(&descriptor.sha256),
            descriptor.bytes,
        )?;
        ensure!(
            bytes.len() as u64 == descriptor.bytes && hash(&bytes) == descriptor.sha256,
            "capture fragment mismatch: {}",
            descriptor.path
        );
        if descriptor.kind == "environment" {
            environment = Some(serde_json::from_slice::<Value>(&bytes)?);
        }
        if let Some(stage) = &stage {
            let path = stage.path().join(&descriptor.path);
            fs::create_dir_all(path.parent().context("Source file needs a parent")?)?;
            let mut file = OpenOptions::new().write(true).create_new(true).open(path)?;
            file.write_all(&bytes)?;
            file.sync_all()?;
        }
    }
    Provenance::verify(
        &capture.provenance,
        &environment.context("capture environment is missing")?,
        &capture.request,
    )?;
    if let Some(stage) = &stage {
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(stage.path().join("manifest.json"))?;
        // Canonical Value ordering matches the native Source envelope.
        file.write_all(&serde_json::to_vec(&serde_json::to_value(
            &capture.source,
        )?)?)?;
        file.sync_all()?;
    }
    let assembled = stage
        .as_ref()
        .map_or(output.as_path(), |stage| stage.path());
    Domain::load(&Source::open(assembled)?)?;
    if let Some(stage) = stage {
        ensure!(!output.exists(), "Source output appeared during assembly");
        fs::rename(stage.path(), &output).context("publish assembled Source")?;
    }
    Ok(
        json!({"id":capture.source.id,"environment":capture.source.environment,"scope":capture.source.scope,
        "capture":capture.id,"sha256":expected,"output":output,"files":capture.files.len(),"bytes":total,"reused":reused}),
    )
}
