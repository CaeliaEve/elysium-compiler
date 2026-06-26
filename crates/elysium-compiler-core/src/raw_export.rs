use crate::io::sha256_file;
use crate::manifest::{count_jsonl_rows, resolve_manifest_path, RawManifest};
use crate::reports::{RawExportSummary, ZeroRecipeDiagnostics};
use anyhow::{Context, Result};
use serde_json::Value;
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

pub fn summarize_raw_export(
    input: &Path,
    manifest: &RawManifest,
    warnings: &mut Vec<String>,
    blocked: &mut Vec<String>,
) -> Result<RawExportSummary> {
    let mut existing_declared_files = 0usize;
    let mut missing_declared_files = Vec::new();
    let mut file_counts = BTreeMap::new();
    let mut file_hashes = BTreeMap::new();

    for (logical_name, relative_path) in &manifest.files {
        let normalized = relative_path
            .replace('\\', "/")
            .trim_start_matches('/')
            .to_string();
        let path = input.join(&normalized);
        if !path.exists() {
            missing_declared_files.push(format!("{}:{}", logical_name, normalized));
            continue;
        }
        if path.is_dir() {
            warnings.push(format!(
                "manifest path is a directory and was not hashed as a file: {}:{}",
                logical_name, normalized
            ));
            existing_declared_files += 1;
            continue;
        }
        existing_declared_files += 1;
        file_hashes.insert(logical_name.clone(), sha256_file(&path)?);
        if normalized.ends_with(".jsonl") || normalized.ends_with(".jsonl.gz") {
            file_counts.insert(logical_name.clone(), count_jsonl_rows(&path)?);
        }
    }

    for required in ["items", "fluids", "recipeIndex", "browserAtlasIndex"] {
        if !manifest.files.contains_key(required) {
            blocked.push(format!("missing required manifest file key: {}", required));
        }
    }
    if missing_declared_files.is_empty() {
        warnings.push("all declared manifest files exist".to_string());
    }
    let zero_recipe_diagnostics = read_zero_recipe_diagnostics(input, manifest, warnings)?;

    Ok(RawExportSummary {
        manifest_schema_version: manifest.schema_version.clone(),
        repository_name: manifest.repository_name.clone(),
        generated_at: manifest.generated_at.clone(),
        capabilities: manifest.capabilities.clone(),
        declared_files: manifest.files.len(),
        existing_declared_files,
        missing_declared_files,
        file_counts,
        file_hashes,
        zero_recipe_diagnostics,
    })
}

fn read_zero_recipe_diagnostics(
    input: &Path,
    manifest: &RawManifest,
    warnings: &mut Vec<String>,
) -> Result<Option<ZeroRecipeDiagnostics>> {
    let path = resolve_manifest_path(input, manifest, "neiHandlerAnomalies").or_else(|| {
        let candidate = input.join("validation").join("nei_handler_anomalies.json");
        candidate.exists().then_some(candidate)
    });
    let Some(path) = path else {
        warnings.push(
            "zero-recipe diagnostics are missing: validation/nei_handler_anomalies.json"
                .to_string(),
        );
        return Ok(None);
    };
    let text = fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    let value: Value =
        serde_json::from_str(&text).with_context(|| format!("parse {}", path.display()))?;
    Ok(zero_recipe_diagnostics_from_value(&value))
}

pub fn zero_recipe_diagnostics_from_value(value: &Value) -> Option<ZeroRecipeDiagnostics> {
    let summary = value.get("summary").and_then(Value::as_object)?;
    let expected_empty_handlers = object_u64(summary, "expectedEmptyHandlers");
    let native_covered_zero_exports = object_u64(summary, "nativeCoveredZeroExports");
    let non_recipe_info_zero_exports = object_u64(summary, "nonRecipeInfoZeroExports");
    let legal_zero_recipe_handlers = object_u64(summary, "legalZeroRecipeHandlers")
        .max(expected_empty_handlers + native_covered_zero_exports + non_recipe_info_zero_exports);
    Some(ZeroRecipeDiagnostics {
        status: summary
            .get("status")
            .and_then(Value::as_str)
            .map(str::to_string),
        total_handlers: object_u64(summary, "totalHandlers"),
        handlers_with_loaded_recipes: object_u64(summary, "handlersWithLoadedRecipes"),
        handlers_with_exported_recipes: object_u64(summary, "handlersWithExportedRecipes"),
        legal_zero_recipe_handlers,
        expected_empty_handlers,
        native_covered_zero_exports,
        non_recipe_info_zero_exports,
        suspicious_zero_exports: object_u64(summary, "suspiciousZeroExports"),
        partial_exports: object_u64(summary, "partialExports"),
    })
}

fn object_u64(map: &serde_json::Map<String, Value>, key: &str) -> u64 {
    map.get(key).and_then(Value::as_u64).unwrap_or(0)
}
