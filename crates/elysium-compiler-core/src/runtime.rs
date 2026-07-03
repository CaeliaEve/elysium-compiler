use crate::cli::CompileScope;
use crate::io::{sha256_file, write_json_value};
use crate::pack_abi::{
    collect_text_path_violations, runtime_debug_artifact_specs, runtime_manifest_file_entries,
    runtime_pack_artifact_specs, PACK_ABI_VALIDATION_REPORT_PATH,
    RUNTIME_DEBUG_DIRECTORY_ARTIFACTS,
};
use crate::runtime_manifest_abi::{
    base_dist_manifest_payload, native_runtime_dist_payload, runtime_capabilities,
    runtime_manifest_payload, runtime_report_payload, RuntimeReportPayloadInput,
    RUNTIME_MANIFEST_REPORT_DESCRIPTORS, RUST_RUNTIME_ENTRYPOINTS, RUST_RUNTIME_MANIFEST_PATH,
};
use crate::version::metadata as compiler_metadata;
use anyhow::{anyhow, Context, Result};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

pub fn purge_debug_json_artifacts(output: &Path) -> Result<()> {
    for spec in runtime_debug_artifact_specs() {
        let path = output.join(spec.relative_path);
        if path.exists() {
            fs::remove_file(&path)
                .with_context(|| format!("remove stale debug artifact {}", path.display()))?;
        }
    }
    for relative_path in RUNTIME_DEBUG_DIRECTORY_ARTIFACTS {
        let path = output.join(relative_path);
        if path.exists() {
            fs::remove_dir_all(&path).with_context(|| {
                format!("remove stale debug shard directory {}", path.display())
            })?;
        }
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
    let capabilities = rust_capabilities(scope);
    let report_input = RuntimeReportPayloadInput {
        integrity: &integrity,
        sizes: &sizes,
        missing: &missing,
        path_violations: &path_violations,
        runtime_id: &runtime_id,
        total_bytes,
        compile_scope: scope.as_str(),
        capabilities: &capabilities,
    };
    write_json_value(
        &output.join(RUST_RUNTIME_MANIFEST_PATH),
        &runtime_manifest_payload(
            &report_input,
            compiler_metadata(),
            files,
            rust_entrypoints_from_integrity(&integrity),
        ),
    )?;

    for descriptor in RUNTIME_MANIFEST_REPORT_DESCRIPTORS {
        write_json_value(
            &output.join(descriptor.path),
            &runtime_report_payload(descriptor, &report_input),
        )?;
    }
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
        base_dist_manifest_payload(compiler_metadata())
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
    manifest["nativeRuntime"] = native_runtime_dist_payload(&RuntimeReportPayloadInput {
        integrity,
        sizes,
        missing: &[],
        path_violations: &[],
        runtime_id,
        total_bytes,
        compile_scope: scope.as_str(),
        capabilities: &Value::Null,
    });

    write_json_value(&manifest_path, &manifest)
}
