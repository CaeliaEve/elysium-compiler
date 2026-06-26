use anyhow::{Context, Result};
use flate2::read::GzDecoder;
use serde::Deserialize;
use serde_json::Value;
use std::collections::BTreeMap;
use std::fs::{self, File};
use std::io::{BufRead, BufReader, Read};
use std::path::{Path, PathBuf};

#[derive(Debug, Deserialize)]
pub struct RawManifest {
    #[serde(rename = "schemaVersion")]
    pub schema_version: Option<String>,
    #[serde(default)]
    pub files: BTreeMap<String, String>,
    #[serde(default)]
    pub capabilities: Vec<String>,
    #[serde(rename = "generatedAt")]
    pub generated_at: Option<Value>,
    #[serde(rename = "repositoryName")]
    pub repository_name: Option<String>,
}

pub fn read_manifest(input: &Path) -> Result<RawManifest> {
    let manifest_path = input.join("manifest.json");
    let text = fs::read_to_string(&manifest_path)
        .with_context(|| format!("read manifest {}", manifest_path.display()))?;
    serde_json::from_str(&text)
        .with_context(|| format!("parse manifest {}", manifest_path.display()))
}

pub fn read_manifest_json(
    input: &Path,
    manifest: &RawManifest,
    logical_name: &str,
) -> Result<Option<Value>> {
    let Some(path) = resolve_manifest_path(input, manifest, logical_name) else {
        return Ok(None);
    };
    let text = fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    let value = serde_json::from_str(&text).with_context(|| format!("parse {}", path.display()))?;
    Ok(Some(value))
}

pub fn read_optional_manifest_json(
    input: &Path,
    manifest: &RawManifest,
    logical_name: &str,
) -> Result<Option<Value>> {
    let Some(path) = resolve_manifest_path(input, manifest, logical_name) else {
        return Ok(None);
    };
    if !path.exists() {
        return Ok(None);
    }
    let text = fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    let value = serde_json::from_str(&text).with_context(|| format!("parse {}", path.display()))?;
    Ok(Some(value))
}

pub fn read_jsonl_values(
    input: &Path,
    manifest: &RawManifest,
    logical_name: &str,
) -> Result<Vec<Value>> {
    let Some(path) = resolve_manifest_path(input, manifest, logical_name) else {
        return Ok(Vec::new());
    };
    read_jsonl_file_values(&path)
}

pub fn read_json_collection(
    input: &Path,
    manifest: &RawManifest,
    logical_names: &[&str],
    array_field: Option<&str>,
) -> Result<Vec<Value>> {
    for logical_name in logical_names {
        let Some(path) = resolve_manifest_path(input, manifest, logical_name) else {
            continue;
        };
        if path.extension().and_then(|value| value.to_str()) == Some("jsonl")
            || path.extension().and_then(|value| value.to_str()) == Some("gz")
        {
            return read_jsonl_file_values(&path);
        }
        let text = fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
        let value: Value =
            serde_json::from_str(&text).with_context(|| format!("parse {}", path.display()))?;
        if let Some(rows) = value.as_array() {
            return Ok(rows.clone());
        }
        if let Some(field) = array_field {
            if let Some(rows) = value.get(field).and_then(Value::as_array) {
                return Ok(rows.clone());
            }
        }
        if let Some(rows) = value.get("items").and_then(Value::as_array) {
            return Ok(rows.clone());
        }
        if let Some(rows) = value.get("groups").and_then(Value::as_array) {
            return Ok(rows.clone());
        }
        if let Some(rows) = value.get("animations").and_then(Value::as_array) {
            return Ok(rows.clone());
        }
        return Ok(vec![value]);
    }
    Ok(Vec::new())
}

pub fn read_jsonl_file_values(path: &Path) -> Result<Vec<Value>> {
    let reader: Box<dyn Read> = if path.extension().and_then(|value| value.to_str()) == Some("gz") {
        Box::new(GzDecoder::new(File::open(path)?))
    } else {
        Box::new(File::open(path)?)
    };
    let buf = BufReader::new(reader);
    let mut rows = Vec::new();
    for (index, line) in buf.lines().enumerate() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        rows.push(
            serde_json::from_str(&line)
                .with_context(|| format!("parse {} line {}", path.display(), index + 1))?,
        );
    }
    Ok(rows)
}

pub fn count_jsonl_rows(path: &Path) -> Result<u64> {
    let reader: Box<dyn Read> = if path.extension().and_then(|value| value.to_str()) == Some("gz") {
        Box::new(GzDecoder::new(File::open(path)?))
    } else {
        Box::new(File::open(path)?)
    };
    let buf = BufReader::new(reader);
    let mut count = 0u64;
    for line in buf.lines() {
        if !line?.trim().is_empty() {
            count += 1;
        }
    }
    Ok(count)
}

pub fn resolve_manifest_path(
    input: &Path,
    manifest: &RawManifest,
    logical_name: &str,
) -> Option<PathBuf> {
    let relative_path = manifest.files.get(logical_name)?;
    let normalized = relative_path
        .replace('\\', "/")
        .trim_start_matches('/')
        .to_string();
    Some(input.join(normalized))
}

pub fn portable_relative_path(value: &str) -> Option<PathBuf> {
    let normalized = value.replace('\\', "/").trim_start_matches('/').to_string();
    if normalized.is_empty()
        || normalized.contains("://")
        || normalized.contains(':')
        || normalized
            .split('/')
            .any(|part| part.is_empty() || part == "." || part == "..")
    {
        return None;
    }
    Some(PathBuf::from(normalized))
}

pub fn runtime_file_descriptors(
    input: &Path,
    manifest: &RawManifest,
    logical_pairs: &[(&str, &str)],
) -> Result<Vec<Value>> {
    let mut files = Vec::new();
    for (public_name, manifest_key) in logical_pairs {
        let Some(path) = resolve_manifest_path(input, manifest, manifest_key) else {
            continue;
        };
        if !path.exists() {
            continue;
        }
        files.push(serde_json::json!({
            "logicalName": public_name,
            "manifestKey": manifest_key,
            "path": path.strip_prefix(input).unwrap_or(path.as_path()).to_string_lossy().replace('\\', "/"),
            "bytes": path.metadata()?.len(),
            "sha256": crate::io::sha256_file(&path)?,
        }));
    }
    Ok(files)
}
