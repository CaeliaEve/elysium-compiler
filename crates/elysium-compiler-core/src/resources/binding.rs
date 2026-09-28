use super::provenance::{CaptureInput, Provenance, SourceProof};
use super::{
    logical_path, publish, selected, staging, Entry, ARCHIVE_LIMIT, ENTRY_LIMIT, FORMAT,
    OUTPUT_LIMIT, REVISION, ROW_LIMIT,
};
use crate::catalog::store::destination;
use crate::source::{is_digest, source_path};
use anyhow::{ensure, Context, Result};
use serde::Deserialize;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Origin {
    key: String,
    bytes: u64,
    sha256: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Inventory {
    format: String,
    revision: u32,
    id: String,
    resolution: String,
    archives: Vec<Origin>,
    entries: Vec<Entry>,
}

pub(super) fn hash(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

pub(super) fn read(path: &Path, limit: u64) -> Result<Vec<u8>> {
    let path = destination(path)?;
    let file = File::open(&path)?;
    ensure!(
        file.metadata()?.is_file(),
        "resource input is not a regular file"
    );
    let mut bytes = Vec::new();
    file.take(limit + 1).read_to_end(&mut bytes)?;
    ensure!(bytes.len() as u64 <= limit, "resource input exceeds limit");
    Ok(bytes)
}

fn location_path(location: &str) -> Result<String> {
    let (namespace, path) = location
        .split_once(':')
        .context("resource namespace is required")?;
    ensure!(
        !namespace.is_empty()
            && namespace
                .bytes()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || b"_.-".contains(&c)),
        "invalid resource namespace"
    );
    ensure!(location.len() <= 4096, "resource location exceeds limit");
    let path = format!("assets/{namespace}/{path}");
    logical_path(&path)?;
    ensure!(selected(&path), "unsupported resource kind");
    Ok(path)
}

/// A bindings inventory proves selected resource bytes, never recipe or render coverage.
pub fn resolve(
    resources: &Path,
    report: &Path,
    report_hash: &str,
    environment: &str,
    output: &Path,
    capture: Option<&CaptureInput>,
) -> Result<Value> {
    let resources = destination(resources)?;
    let target = destination(output)?;
    ensure!(
        !target.starts_with(&resources),
        "bindings output must be outside the resource inventory"
    );
    ensure!(
        is_digest(report_hash) && is_digest(environment),
        "expected report and environment digests are required"
    );
    let evidence = read(report, 16 * 1024 * 1024)?;
    ensure!(
        hash(&evidence) == report_hash,
        "resource report digest mismatch"
    );
    let report: Value = serde_json::from_slice(&evidence)?;
    ensure!(
        report["format"] == "nesql.check"
            && report["status"] == "complete"
            && report.get("error").is_none(),
        "resource report is not complete"
    );
    ensure!(
        report["request"]["check"]["domain"] == "resources",
        "report does not contain resource observations"
    );
    ensure!(
        report["environment"].is_object()
            && hash(&serde_json::to_vec(&report["environment"])?) == environment,
        "resource environment mismatch"
    );
    if let Some(proof) = report.get("provenance") {
        Provenance::verify(proof, &report["environment"], &report["request"])?;
    }
    let mut source = capture
        .map(|input| SourceProof::open(input, &report, &target))
        .transpose()?;
    let requested = report["request"]["check"]["resources"]
        .as_array()
        .context("missing resource selection")?;
    let world = report["request"]["world"]
        .as_str()
        .context("resource report is not bound to a world")?;
    ensure!(
        !world.is_empty()
            && world.len() <= 512
            && world != "."
            && world != ".."
            && !world
                .chars()
                .any(|c| c.is_control() || c == '/' || c == '\\'),
        "invalid resource world"
    );
    ensure!(
        !requested.is_empty() && requested.len() <= 128,
        "invalid resource selection size"
    );
    let mut pending = BTreeSet::new();
    for resource in requested {
        let name = resource.as_str().context("invalid resource selection")?;
        location_path(name)?;
        ensure!(pending.insert(name), "duplicate resource selection");
    }
    let rows = report["rows"]
        .as_array()
        .context("missing resource observations")?;
    ensure!(
        rows.len() == pending.len(),
        "resource target count mismatch"
    );

    // An archive with matching pixels on disk is insufficient: its bytes must
    // belong to this loaded mod set or an enabled resource-pack file.
    let mut loaded = BTreeSet::new();
    for row in report["environment"]["mods"]
        .as_array()
        .context("missing mod environment")?
    {
        let digest = row["sha256"].as_str().context("mod digest missing")?;
        ensure!(is_digest(digest), "invalid mod digest");
        loaded.insert(digest);
    }
    let packs: BTreeSet<_> = report["environment"]["resources"]
        .as_array()
        .context("missing enabled resource packs")?
        .iter()
        .map(|p| {
            p.as_str()
                .context("invalid pack name")
                .map(|p| format!("resourcepacks/{p}"))
        })
        .collect::<Result<_>>()?;
    for row in report["environment"]["inputs"]
        .as_array()
        .context("missing environment inputs")?
    {
        if row["path"].as_str().is_some_and(|p| packs.contains(p)) {
            let digest = row["sha256"].as_str().context("pack digest missing")?;
            ensure!(is_digest(digest), "invalid pack digest");
            loaded.insert(digest);
        }
    }
    let bytes = read(&resources.join("manifest.json"), 128 * 1024 * 1024)?;
    let mut raw: Value = serde_json::from_slice(&bytes)?;
    let inventory: Inventory = serde_json::from_value(raw.clone())?;
    raw.as_object_mut()
        .context("invalid resource manifest")?
        .remove("id");
    ensure!(
        inventory.format == FORMAT
            && inventory.revision == REVISION
            && inventory.resolution == "unverified"
            && is_digest(&inventory.id)
            && hash(&serde_json::to_vec(&raw)?) == inventory.id,
        "resource inventory identity mismatch"
    );
    ensure!(
        !inventory.archives.is_empty()
            && inventory.archives.len() <= 1024
            && inventory.entries.len() <= ROW_LIMIT,
        "resource inventory exceeds count limits"
    );
    let mut origins = BTreeMap::new();
    for origin in &inventory.archives {
        source_path(&origin.key)?;
        ensure!(
            origin.bytes > 0
                && origin.bytes <= ARCHIVE_LIMIT
                && is_digest(&origin.sha256)
                && origins
                    .insert(origin.key.as_str(), origin.sha256.as_str())
                    .is_none(),
            "invalid resource origin"
        );
    }
    let mut previous = None;
    let mut total = 0;
    let mut by_path: BTreeMap<&str, Vec<&Entry>> = BTreeMap::new();
    for entry in &inventory.entries {
        logical_path(&entry.path)?;
        let key = (&entry.path, &entry.archive);
        ensure!(
            previous.is_none_or(|p| p < key)
                && selected(&entry.path)
                && origins.contains_key(entry.archive.as_str())
                && is_digest(&entry.sha256)
                && entry.bytes <= ENTRY_LIMIT
                && entry.blob == format!("blobs/{}", entry.sha256),
            "invalid resource entry"
        );
        previous = Some(key);
        by_path.entry(&entry.path).or_default().push(entry);
        total += entry.bytes;
        ensure!(
            total <= OUTPUT_LIMIT,
            "resource inventory exceeds byte limit"
        );
    }

    let (output, stage, _lock) = staging(output)?;
    let mut bindings = Vec::new();
    for row in rows {
        let resource = row["resource"].as_str().context("missing resource name")?;
        ensure!(
            pending.remove(resource) && row["status"] == "passed" && row.get("error").is_none(),
            "resource observation failed or duplicated"
        );
        let path = location_path(resource)?;
        ensure!(row["path"] == path, "native resource path mismatch");
        let count = row["bytes"]
            .as_str()
            .context("resource byte count must be a decimal string")?;
        let count_value: u64 = count.parse()?;
        ensure!(
            count_value <= ENTRY_LIMIT && count_value.to_string() == count,
            "invalid native resource byte count"
        );
        let digest = row["sha256"]
            .as_str()
            .context("missing native resource digest")?;
        ensure!(is_digest(digest), "invalid native resource digest");
        let matched: Vec<_> = by_path
            .get(path.as_str())
            .into_iter()
            .flatten()
            .filter(|e| {
                e.path == path
                    && e.sha256 == digest
                    && e.bytes == count_value
                    && origins
                        .get(e.archive.as_str())
                        .is_some_and(|sha| loaded.contains(sha))
            })
            .collect();
        ensure!(
            !matched.is_empty(),
            "no loaded resource candidate matches {resource}"
        );
        let blob = &matched[0].blob;
        let bytes = read(&resources.join(blob), ENTRY_LIMIT)?;
        ensure!(
            bytes.len() as u64 == count_value && hash(&bytes) == digest,
            "resource blob mismatch: {resource}"
        );
        if let Some(source) = &mut source {
            source.compare(resource, &bytes)?;
        }
        if !stage.path().join(blob).exists() {
            let mut file = File::create(stage.path().join(blob))?;
            file.write_all(&bytes)?;
            file.sync_all()?;
        }
        bindings.push(
            json!({"resource":resource,"path":path,"bytes":count,"sha256":digest,"blob":blob,
            "origins":matched.iter().map(|e| &e.archive).collect::<Vec<_>>()}),
        );
    }
    ensure!(pending.is_empty(), "resource observations are incomplete");
    bindings.sort_by(|a, b| a["resource"].as_str().cmp(&b["resource"].as_str()));
    let mut manifest = json!({"format":"elysium.bindings","revision":1,"environment":environment,"world":world,
        "inventory":inventory.id,"report":report_hash,"scope":"resources","entries":bindings});
    if let Some(proof) = report.get("provenance") {
        manifest["provenance"] = proof.clone();
    }
    if let Some(source) = source {
        manifest["source"] = source.finish()?;
    }
    let id = publish(&output, stage, manifest)?;
    Ok(json!({"id":id,"output":output,"resources":bindings.len(),"scope":"resources"}))
}
