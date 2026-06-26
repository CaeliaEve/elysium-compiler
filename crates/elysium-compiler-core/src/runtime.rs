use crate::cli::CompileScope;
use crate::io::{normalize_path, sha256_file, write_json_value};
use crate::native_ui_report::{compile_native_ui_layout_report, CapturedUiFamilyKeyFn};
use crate::version::metadata as compiler_metadata;
use anyhow::{anyhow, Context, Result};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

pub fn is_text_runtime_artifact(path: &Path) -> bool {
    path.extension()
        .and_then(|value| value.to_str())
        .is_some_and(|extension| matches!(extension, "json" | "txt" | "log"))
}

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
    let mut entries = vec![
        ("rustRuntimeManifest", "rust/runtime-manifest.json"),
        ("rustIntegrity", "rust/integrity.json"),
        ("rustSizeReport", "rust/size-report.json"),
        ("rustMissingDataReport", "rust/missing-data-report.json"),
        (
            "rustSemanticValidationReport",
            "rust/semantic-validation-report.json",
        ),
        (
            "rustMissingTextureReport",
            "rust/missing-texture-report.json",
        ),
        (
            "rustSuspiciousTextureReport",
            "rust/suspicious-texture-report.json",
        ),
        (
            "rustRecipeHandlerMetadataReport",
            "rust/recipe-handler-metadata-report.json",
        ),
        (
            "rustRecipeFragmentationReport",
            "rust/recipe-fragmentation-report.json",
        ),
        (
            "rustNativeUiLayoutReport",
            "rust/native-ui-layout-report.json",
        ),
        ("rustMigrationReadiness", "rust/migration-readiness.json"),
        ("rustDeploymentReport", "rust/deployment-report.json"),
    ];
    match scope {
        CompileScope::All => entries.extend([
            ("rustBrowserBin", "rust/browser.bin"),
            ("rustGroupsBin", "rust/groups.bin"),
            ("rustSearchBin", "rust/search.bin"),
            ("rustRecipeBin", "rust/recipes.bin"),
            ("rustTextureBin", "rust/textures.bin"),
            ("rustAtlasMetaBin", "rust/atlas.meta.bin"),
            ("rustAnimationBin", "rust/animations.bin"),
            ("rustStringsZhCnBin", "rust/strings.zh_cn.bin"),
            ("rustUiTemplatesBin", "rust/ui-pack/ui_templates.bin"),
            ("rustUiBindingsBin", "rust/ui-pack/ui_bindings.bin"),
            ("rustUiStringsBin", "rust/ui-pack/ui_strings.bin"),
            (
                "rustUiAssetsManifest",
                "rust/ui-pack/ui_assets.manifest.json",
            ),
            (
                "rustUiTemplateCatalog",
                "rust/ui-pack/ui_template_catalog.json",
            ),
            (
                "rustUiTemplateBindingIndex",
                "rust/ui-pack/ui_template_binding_index.json",
            ),
            ("rustUiFamilyCensus", "rust/ui-pack/ui_family_census.json"),
            ("rustUiPackReport", "rust/ui-pack/ui_pack_report.json"),
        ]),
        CompileScope::NativeUi => entries.extend([
            ("rustBrowserBin", "rust/browser.bin"),
            ("rustGroupsBin", "rust/groups.bin"),
            ("rustSearchBin", "rust/search.bin"),
            ("rustRecipeBin", "rust/recipes.bin"),
            ("rustStringsZhCnBin", "rust/strings.zh_cn.bin"),
            ("rustUiTemplatesBin", "rust/ui-pack/ui_templates.bin"),
            ("rustUiBindingsBin", "rust/ui-pack/ui_bindings.bin"),
            ("rustUiStringsBin", "rust/ui-pack/ui_strings.bin"),
            (
                "rustUiAssetsManifest",
                "rust/ui-pack/ui_assets.manifest.json",
            ),
            (
                "rustUiTemplateCatalog",
                "rust/ui-pack/ui_template_catalog.json",
            ),
            (
                "rustUiTemplateBindingIndex",
                "rust/ui-pack/ui_template_binding_index.json",
            ),
            ("rustUiFamilyCensus", "rust/ui-pack/ui_family_census.json"),
            ("rustUiPackReport", "rust/ui-pack/ui_pack_report.json"),
        ]),
        CompileScope::Search => entries.extend([
            ("rustSearchBin", "rust/search.bin"),
            ("rustStringsZhCnBin", "rust/strings.zh_cn.bin"),
        ]),
        CompileScope::Browser => entries.extend([
            ("rustBrowserBin", "rust/browser.bin"),
            ("rustGroupsBin", "rust/groups.bin"),
            ("rustSearchBin", "rust/search.bin"),
            ("rustStringsZhCnBin", "rust/strings.zh_cn.bin"),
        ]),
        CompileScope::Recipes => entries.extend([("rustRecipeBin", "rust/recipes.bin")]),
        CompileScope::Ui => entries.extend([
            ("rustUiTemplatesBin", "rust/ui-pack/ui_templates.bin"),
            ("rustUiBindingsBin", "rust/ui-pack/ui_bindings.bin"),
            ("rustUiStringsBin", "rust/ui-pack/ui_strings.bin"),
            (
                "rustUiAssetsManifest",
                "rust/ui-pack/ui_assets.manifest.json",
            ),
            (
                "rustUiTemplateCatalog",
                "rust/ui-pack/ui_template_catalog.json",
            ),
            (
                "rustUiTemplateBindingIndex",
                "rust/ui-pack/ui_template_binding_index.json",
            ),
            ("rustUiFamilyCensus", "rust/ui-pack/ui_family_census.json"),
            ("rustUiPackReport", "rust/ui-pack/ui_pack_report.json"),
        ]),
        CompileScope::Textures => entries.extend([
            ("rustTextureBin", "rust/textures.bin"),
            ("rustAtlasMetaBin", "rust/atlas.meta.bin"),
            ("rustAnimationBin", "rust/animations.bin"),
        ]),
    }
    if debug_json {
        match scope {
            CompileScope::All => entries.extend([
                ("rustBrowserPack", "rust/browser-pack.json"),
                ("rustSearchPack", "rust/search-pack.json"),
                ("rustRecipePack", "rust/recipe-pack.json"),
                ("rustTexturePack", "rust/texture-pack.json"),
            ]),
            CompileScope::NativeUi => entries.extend([
                ("rustBrowserPack", "rust/browser-pack.json"),
                ("rustSearchPack", "rust/search-pack.json"),
                ("rustRecipePack", "rust/recipe-pack.json"),
            ]),
            CompileScope::Search => entries.push(("rustSearchPack", "rust/search-pack.json")),
            CompileScope::Browser => entries.extend([
                ("rustBrowserPack", "rust/browser-pack.json"),
                ("rustSearchPack", "rust/search-pack.json"),
            ]),
            CompileScope::Recipes => entries.push(("rustRecipePack", "rust/recipe-pack.json")),
            CompileScope::Ui => {
                entries.push(("rustUiPackReport", "rust/ui-pack/ui_pack_report.json"))
            }
            CompileScope::Textures => entries.push(("rustTexturePack", "rust/texture-pack.json")),
        }
    }
    entries
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
    for (key, path) in [
        ("browser", "rust/browser.bin"),
        ("groups", "rust/groups.bin"),
        ("search", "rust/search.bin"),
        ("recipes", "rust/recipes.bin"),
        ("textures", "rust/textures.bin"),
        ("atlasMeta", "rust/atlas.meta.bin"),
        ("animations", "rust/animations.bin"),
        ("stringsZhCn", "rust/strings.zh_cn.bin"),
        ("uiTemplates", "rust/ui-pack/ui_templates.bin"),
        ("uiBindings", "rust/ui-pack/ui_bindings.bin"),
        ("uiStrings", "rust/ui-pack/ui_strings.bin"),
    ] {
        if integrity.contains_key(path) {
            entrypoints.insert(key.to_string(), Value::String(path.to_string()));
        }
    }
    Value::Object(entrypoints)
}

pub fn rust_capabilities(scope: CompileScope) -> Value {
    match scope {
        CompileScope::All => json!([
            "atlas.static",
            "atlas.animated",
            "atlas.meta",
            "groups.collapse",
            "groups.semantic-nbt",
            "recipes.lookup",
            "recipes.native-ui-layout",
            "search.zh-cn",
            "strings.zh-cn",
            "native-render.webgl2",
            "recipes.ui-pack",
        ]),
        CompileScope::NativeUi => json!([
            "groups.collapse",
            "groups.semantic-nbt",
            "recipes.lookup",
            "recipes.native-ui-layout",
            "recipes.ui-pack",
            "search.zh-cn",
            "strings.zh-cn",
            "native-render.webgl2"
        ]),
        CompileScope::Search => json!(["search.zh-cn", "strings.zh-cn"]),
        CompileScope::Browser => json!([
            "groups.collapse",
            "groups.semantic-nbt",
            "search.zh-cn",
            "strings.zh-cn",
            "native-render.webgl2"
        ]),
        CompileScope::Recipes => json!(["recipes.lookup", "recipes.native-ui-layout"]),
        CompileScope::Ui => json!(["recipes.ui-pack", "native-render.webgl2"]),
        CompileScope::Textures => json!(["atlas.static", "atlas.animated", "atlas.meta"]),
    }
}

pub fn compile_runtime_reports(
    output: &Path,
    scope: CompileScope,
    strict: bool,
    debug_json: bool,
    captured_ui_family_key: CapturedUiFamilyKeyFn,
) -> Result<()> {
    let rust_dir = output.join("rust");
    fs::create_dir_all(&rust_dir)?;
    compile_native_ui_layout_report(output, captured_ui_family_key)?;

    let mut artifact_names = match scope {
        CompileScope::All => vec![
            "browser.bin",
            "groups.bin",
            "search.bin",
            "recipes.bin",
            "textures.bin",
            "atlas.meta.bin",
            "animations.bin",
            "strings.zh_cn.bin",
            "missing-texture-report.json",
            "suspicious-texture-report.json",
            "semantic-validation-report.json",
            "recipe-handler-metadata-report.json",
            "recipe-fragmentation-report.json",
            "native-ui-layout-report.json",
            "ui-pack/ui_templates.bin",
            "ui-pack/ui_bindings.bin",
            "ui-pack/ui_strings.bin",
            "ui-pack/ui_assets.manifest.json",
            "ui-pack/ui_template_catalog.json",
            "ui-pack/ui_template_binding_index.json",
            "ui-pack/ui_family_census.json",
            "ui-pack/ui_pack_report.json",
        ],
        CompileScope::NativeUi => vec![
            "browser.bin",
            "groups.bin",
            "search.bin",
            "recipes.bin",
            "strings.zh_cn.bin",
            "semantic-validation-report.json",
            "recipe-handler-metadata-report.json",
            "recipe-fragmentation-report.json",
            "native-ui-layout-report.json",
            "ui-pack/ui_templates.bin",
            "ui-pack/ui_bindings.bin",
            "ui-pack/ui_strings.bin",
            "ui-pack/ui_assets.manifest.json",
            "ui-pack/ui_template_catalog.json",
            "ui-pack/ui_template_binding_index.json",
            "ui-pack/ui_family_census.json",
            "ui-pack/ui_pack_report.json",
        ],
        CompileScope::Search => vec![
            "search.bin",
            "strings.zh_cn.bin",
            "semantic-validation-report.json",
        ],
        CompileScope::Browser => vec![
            "browser.bin",
            "groups.bin",
            "search.bin",
            "strings.zh_cn.bin",
            "semantic-validation-report.json",
        ],
        CompileScope::Recipes => vec![
            "recipes.bin",
            "semantic-validation-report.json",
            "recipe-handler-metadata-report.json",
            "recipe-fragmentation-report.json",
            "native-ui-layout-report.json",
        ],
        CompileScope::Ui => vec![
            "semantic-validation-report.json",
            "ui-pack/ui_templates.bin",
            "ui-pack/ui_bindings.bin",
            "ui-pack/ui_strings.bin",
            "ui-pack/ui_assets.manifest.json",
            "ui-pack/ui_template_catalog.json",
            "ui-pack/ui_template_binding_index.json",
            "ui-pack/ui_family_census.json",
            "ui-pack/ui_pack_report.json",
        ],
        CompileScope::Textures => vec![
            "textures.bin",
            "atlas.meta.bin",
            "animations.bin",
            "missing-texture-report.json",
            "suspicious-texture-report.json",
            "semantic-validation-report.json",
        ],
    };
    if debug_json {
        match scope {
            CompileScope::All => artifact_names.extend([
                "browser-pack.json",
                "search-pack.json",
                "recipe-pack.json",
                "texture-pack.json",
            ]),
            CompileScope::NativeUi => {
                artifact_names.extend(["browser-pack.json", "search-pack.json", "recipe-pack.json"])
            }
            CompileScope::Search => artifact_names.push("search-pack.json"),
            CompileScope::Browser => {
                artifact_names.extend(["browser-pack.json", "search-pack.json"])
            }
            CompileScope::Recipes => artifact_names.push("recipe-pack.json"),
            CompileScope::Ui => {}
            CompileScope::Textures => artifact_names.push("texture-pack.json"),
        }
    }
    if !matches!(scope, CompileScope::All) {
        for artifact_name in [
            "browser.bin",
            "groups.bin",
            "search.bin",
            "recipes.bin",
            "textures.bin",
            "atlas.meta.bin",
            "animations.bin",
            "strings.zh_cn.bin",
            "ui-pack/ui_templates.bin",
            "ui-pack/ui_bindings.bin",
            "ui-pack/ui_strings.bin",
            "ui-pack/ui_assets.manifest.json",
            "ui-pack/ui_template_catalog.json",
            "ui-pack/ui_template_binding_index.json",
            "ui-pack/ui_family_census.json",
            "native-ui-layout-report.json",
        ] {
            if !artifact_names.contains(&artifact_name) && rust_dir.join(artifact_name).exists() {
                artifact_names.push(artifact_name);
            }
        }
    }
    let mut files = Vec::new();
    let mut integrity = BTreeMap::new();
    let mut sizes = BTreeMap::new();
    let mut missing = Vec::new();
    let mut path_violations = Vec::new();

    for artifact_name in artifact_names {
        let path = rust_dir.join(artifact_name);
        let relative = format!("rust/{artifact_name}");
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

    for entry in walkdir::WalkDir::new(&rust_dir)
        .into_iter()
        .filter_map(std::result::Result::ok)
        .filter(|entry| entry.file_type().is_file())
        .filter(|entry| is_text_runtime_artifact(entry.path()))
    {
        let path = entry.path();
        let text = fs::read_to_string(path).unwrap_or_default();
        for needle in ["E:\\", "C:\\", "\\\\", "file://"] {
            if text.contains(needle) {
                path_violations.push(format!("{} contains {}", normalize_path(path), needle));
            }
        }
    }

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
        &rust_dir.join("runtime-manifest.json"),
        &json!({
            "schema": "neonei/runtime/current",
            "schemaVersion": "neonei/rust-runtime-manifest/current",
            "schemaRevision": 1,
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
        &rust_dir.join("integrity.json"),
        &json!({
            "schemaVersion": "neonei/rust-integrity/current",
            "algorithm": "sha256",
            "files": integrity,
        }),
    )?;
    write_json_value(
        &rust_dir.join("size-report.json"),
        &json!({
            "schemaVersion": "neonei/rust-size-report/current",
            "totalBytes": total_bytes,
            "files": sizes,
        }),
    )?;
    write_json_value(
        &rust_dir.join("missing-data-report.json"),
        &json!({
            "schemaVersion": "neonei/rust-missing-data-report/current",
            "missingFiles": missing,
        }),
    )?;
    write_json_value(
        &rust_dir.join("migration-readiness.json"),
        &json!({
            "schemaVersion": "neonei/rust-migration-readiness/current",
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
        &rust_dir.join("deployment-report.json"),
        &json!({
            "schemaVersion": "neonei/rust-deployment-report/current",
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
                "runtime": "neonei/runtime/current",
                "schemaRevision": 1,
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
        if integrity.contains_key(relative_path) || relative_path.ends_with("runtime-manifest.json")
        {
            files.insert(key.to_string(), Value::String(relative_path.to_string()));
        }
    }

    manifest["compiler"] = compiler_metadata();
    manifest["nativeRuntime"] = json!({
        "schemaVersion": "neonei/native-runtime-dist/current",
        "runtimeId": runtime_id,
        "compileScope": scope.as_str(),
        "status": "ready",
        "authority": "rust",
        "runtimeManifest": "rust/runtime-manifest.json",
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
