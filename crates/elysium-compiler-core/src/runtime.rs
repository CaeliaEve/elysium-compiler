use crate::cli::CompileScope;
use crate::io::{sha256_file, write_json_value};
use crate::pack_abi::{
    collect_text_path_violations, runtime_manifest_file_entries, runtime_pack_artifact_specs,
    PACK_ABI_VALIDATION_REPORT_PATH,
};
use crate::runtime_manifest_abi::{
    runtime_capabilities, NATIVE_RUNTIME_DIST_SCHEMA_VERSION, RUST_DEPLOYMENT_REPORT_PATH,
    RUST_DEPLOYMENT_REPORT_SCHEMA_VERSION, RUST_INTEGRITY_REPORT_PATH,
    RUST_INTEGRITY_SCHEMA_VERSION, RUST_MIGRATION_READINESS_REPORT_PATH,
    RUST_MIGRATION_READINESS_SCHEMA_VERSION, RUST_MISSING_DATA_REPORT_PATH,
    RUST_MISSING_DATA_REPORT_SCHEMA_VERSION, RUST_RUNTIME_ENTRYPOINTS, RUST_RUNTIME_MANIFEST_PATH,
    RUST_RUNTIME_MANIFEST_SCHEMA_VERSION, RUST_RUNTIME_SCHEMA, RUST_RUNTIME_SCHEMA_REVISION,
    RUST_SIZE_REPORT_PATH, RUST_SIZE_REPORT_SCHEMA_VERSION,
};
use crate::version::metadata as compiler_metadata;
use anyhow::{anyhow, Context, Result};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

pub fn purge_debug_json_artifacts(output: &Path) -> Result<()> {
    let rust_dir = output.join("rust");
    for artifact_name in [
        "browser-pack.json",
        "search-pack.json",
        "recipe-pack.json",
        "texture-pack.json",
    ] {
        let path = rust_dir.join(artifact_name);
        if path.exists() {
            fs::remove_file(&path)
                .with_context(|| format!("remove stale debug artifact {}", path.display()))?;
        }
    }
    let payload_shards = rust_dir.join("recipe-ui-payload-shards");
    if payload_shards.exists() {
        fs::remove_dir_all(&payload_shards).with_context(|| {
            format!(
                "remove stale debug shard directory {}",
                payload_shards.display()
            )
        })?;
    }
    Ok(())
}

pub fn rust_manifest_file_entries(
    scope: CompileScope,
    debug_json: bool,
) -> Vec<(&'static str, &'static str)> {
    runtime_manifest_file_entries(scope, debug_json)
}

pub fn runtime_id_from_integrity(integrity: &BTreeMap<String, String>) -> String {
    let mut hasher = Sha256::new();
    for (path, hash) in integrity {
        hasher.update(path.as_bytes());
        hasher.update(b"\0");
        hasher.update(hash.as_bytes());
        hasher.update(b"\n");
    }
    let digest = format!("{:x}", hasher.finalize());
    format!("rust-{}", &digest[..16])
}

pub fn rust_entrypoints_from_integrity(integrity: &BTreeMap<String, String>) -> Value {
    let mut entrypoints = serde_json::Map::new();
    for spec in RUST_RUNTIME_ENTRYPOINTS {
        if integrity.contains_key(spec.path) {
            entrypoints.insert(spec.key.to_string(), Value::String(spec.path.to_string()));
        }
    }
    Value::Object(entrypoints)
}

pub fn rust_capabilities(scope: CompileScope) -> Value {
    json!(runtime_capabilities(scope))
}

pub fn compile_runtime_reports(
    output: &Path,
    scope: CompileScope,
    strict: bool,
    debug_json: bool,
) -> Result<()> {
    let rust_dir = output.join("rust");
    fs::create_dir_all(&rust_dir)?;
    let mut files = Vec::new();
    let mut integrity = BTreeMap::new();
    let mut sizes = BTreeMap::new();
    let mut missing = Vec::new();

    let mut artifact_paths = runtime_pack_artifact_specs(scope, debug_json)
        .into_iter()
        .map(|spec| spec.relative_path.to_string())
        .collect::<Vec<_>>();
    artifact_paths.push(PACK_ABI_VALIDATION_REPORT_PATH.to_string());

    for relative in artifact_paths {
        let path = output.join(&relative);
        if !path.exists() {
            missing.push(relative.clone());
            continue;
        }
        let hash = sha256_file(&path)?;
        let size = path.metadata()?.len();
        integrity.insert(relative.clone(), hash);
        sizes.insert(relative.clone(), size);
        files.push(json!({
        "path": relative,
        "bytes": size,
        }));
    }

    let should_hash_texture_assets = matches!(scope, CompileScope::All | CompileScope::Textures);
    let texture_asset_dir = output.join("textures");
    if should_hash_texture_assets && texture_asset_dir.exists() {
        for entry in walkdir::WalkDir::new(&texture_asset_dir)
            .into_iter()
            .filter_map(std::result::Result::ok)
            .filter(|entry| entry.file_type().is_file())
        {
            let path = entry.path();
            let relative_path = path
                .strip_prefix(output)
                .unwrap_or(path)
                .to_string_lossy()
                .replace('\\', "/");
            let hash = sha256_file(path)?;
            let size = path.metadata()?.len();
            integrity.insert(relative_path.clone(), hash);
            sizes.insert(relative_path.clone(), size);
            files.push(json!({
                "path": relative_path,
                "bytes": size,
            }));
        }
    }

    let path_violations = collect_text_path_violations(&rust_dir)?;

    if strict && (!missing.is_empty() || !path_violations.is_empty()) {
        return Err(anyhow!(
            "runtime report blocked: missing={}, path violations={}",
            missing.len(),
            path_violations.len()
        ));
    }

    let total_bytes = sizes.values().sum::<u64>();
    let runtime_id = runtime_id_from_integrity(&integrity);
    let generated_at = "deterministic-rust-compiler";
    let capabilities = rust_capabilities(scope);
    write_json_value(
        &output.join(RUST_RUNTIME_MANIFEST_PATH),
        &json!({
            "schema": RUST_RUNTIME_SCHEMA,
            "schemaVersion": RUST_RUNTIME_MANIFEST_SCHEMA_VERSION,
            "schemaRevision": RUST_RUNTIME_SCHEMA_REVISION,
            "runtimeId": runtime_id,
            "generatedAt": generated_at,
            "compiler": compiler_metadata(),
            "capabilities": capabilities,
            "files": files,
            "compileScope": scope.as_str(),
            "entrypoints": rust_entrypoints_from_integrity(&integrity),
            "pathPolicy": {
                "portableRelativePathsOnly": true,
                "absolutePathsAllowed": false,
                "windowsPathsAllowed": false,
            },
        }),
    )?;
    write_json_value(
        &output.join(RUST_INTEGRITY_REPORT_PATH),
        &json!({
            "schemaVersion": RUST_INTEGRITY_SCHEMA_VERSION,
            "algorithm": "sha256",
            "files": integrity,
        }),
    )?;
    write_json_value(
        &output.join(RUST_SIZE_REPORT_PATH),
        &json!({
            "schemaVersion": RUST_SIZE_REPORT_SCHEMA_VERSION,
            "totalBytes": total_bytes,
            "files": sizes,
        }),
    )?;
    write_json_value(
        &output.join(RUST_MISSING_DATA_REPORT_PATH),
        &json!({
            "schemaVersion": RUST_MISSING_DATA_REPORT_SCHEMA_VERSION,
            "missingFiles": missing,
        }),
    )?;
    write_json_value(
        &output.join(RUST_MIGRATION_READINESS_REPORT_PATH),
        &json!({
            "schemaVersion": RUST_MIGRATION_READINESS_SCHEMA_VERSION,
            "ready": missing.is_empty() && path_violations.is_empty(),
            "checks": {
                "requiredArtifactsPresent": missing.is_empty(),
                "pathPortable": path_violations.is_empty(),
                "integrityHashesGenerated": true,
                "sizeReportGenerated": true,
            },
            "pathViolations": path_violations,
        }),
    )?;
    write_json_value(
        &output.join(RUST_DEPLOYMENT_REPORT_PATH),
        &json!({
            "schemaVersion": RUST_DEPLOYMENT_REPORT_SCHEMA_VERSION,
            "runtimeId": runtime_id,
            "generatedAt": generated_at,
            "compileScope": scope.as_str(),
            "runtimeSize": {
                "totalBytes": total_bytes,
                "files": sizes,
            },
            "cache": {
                "immutableRuntimeFiles": integrity.len(),
                "estimatedRuntimeCacheBytes": total_bytes,
                "cacheKeyInputs": {
                    "runtimeId": runtime_id,
                    "integrityAlgorithm": "sha256",
                },
            },
            "missingData": {
                "missingFiles": missing,
                "missingFileCount": missing.len(),
            },
            "schema": {
                "runtime": RUST_RUNTIME_SCHEMA,
                "schemaRevision": RUST_RUNTIME_SCHEMA_REVISION,
                "capabilities": capabilities,
            },
            "deploymentChecks": {
                "requiredArtifactsPresent": missing.is_empty(),
                "pathPortable": path_violations.is_empty(),
                "integrityHashesGenerated": true,
                "sizeReportGenerated": true,
                "capabilitiesGenerated": true,
            },
            "pathViolations": path_violations,
        }),
    )?;
    update_dist_manifest_with_rust_runtime(
        output,
        scope,
        debug_json,
        &integrity,
        &sizes,
        &runtime_id,
        total_bytes,
    )?;
    Ok(())
}

fn update_dist_manifest_with_rust_runtime(
    output: &Path,
    scope: CompileScope,
    debug_json: bool,
    integrity: &BTreeMap<String, String>,
    sizes: &BTreeMap<String, u64>,
    runtime_id: &str,
    total_bytes: u64,
) -> Result<()> {
    let manifest_path = output.join("manifest.json");
    let mut manifest = if manifest_path.exists() {
        let text = fs::read_to_string(&manifest_path)
            .with_context(|| format!("read dist manifest {}", manifest_path.display()))?;
        serde_json::from_str::<Value>(&text)
            .with_context(|| format!("parse dist manifest {}", manifest_path.display()))?
    } else {
        json!({
            "schemaVersion": "neonei/dist-data/current",
            "source": "elysium-compiler",
            "compiler": compiler_metadata(),
            "files": {},
        })
    };

    if !manifest.is_object() {
        return Err(anyhow!(
            "dist manifest must be a JSON object: {}",
            manifest_path.display()
        ));
    }
    if manifest.get("files").and_then(Value::as_object).is_none() {
        manifest["files"] = json!({});
    }
    let files = manifest["files"]
        .as_object_mut()
        .ok_or_else(|| anyhow!("dist manifest files must be a JSON object"))?;

    if !debug_json {
        for key in [
            "rustBrowserPack",
            "rustSearchPack",
            "rustRecipePack",
            "rustTexturePack",
        ] {
            files.remove(key);
        }
    }

    for (key, relative_path) in rust_manifest_file_entries(scope, debug_json) {
        if integrity.contains_key(relative_path) || output.join(relative_path).exists() {
            files.insert(key.to_string(), Value::String(relative_path.to_string()));
        }
    }

    manifest["compiler"] = compiler_metadata();
    manifest["nativeRuntime"] = json!({
        "schemaVersion": NATIVE_RUNTIME_DIST_SCHEMA_VERSION,
        "runtimeId": runtime_id,
        "compileScope": scope.as_str(),
        "status": "ready",
        "authority": "rust",
        "runtimeManifest": RUST_RUNTIME_MANIFEST_PATH,
        "totalBytes": total_bytes,
        "files": sizes,
        "hashes": integrity,
        "pathPolicy": {
            "portableRelativePathsOnly": true,
            "absolutePathsAllowed": false,
        },
    });

    write_json_value(&manifest_path, &manifest)
}
