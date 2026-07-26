use crate::manifest::portable_relative_path;
use crate::reports::{RawExportSummary, ZeroRecipeDiagnostics};
use crate::session::{RawExportSession, RelativeInputKind};
use anyhow::Result;
use serde_json::Value;
use std::collections::BTreeMap;

pub fn summarize_raw_export(
    session: &RawExportSession,
    warnings: &mut Vec<String>,
    blocked: &mut Vec<String>,
) -> Result<RawExportSummary> {
    let manifest = session.manifest();
    let mut existing_declared_files = 0usize;
    let mut missing_declared_files = Vec::new();
    let mut file_counts = BTreeMap::new();
    let mut file_hashes = BTreeMap::new();

    for (logical_name, relative_path) in &manifest.files {
        let Some(portable_path) = portable_relative_path(relative_path) else {
            blocked.push(format!(
                "manifest path violates portable-relative policy: {}:{}",
                logical_name, relative_path
            ));
            continue;
        };
        let normalized = portable_path.to_string_lossy().replace('\\', "/");
        match session.relative_input_kind(relative_path)? {
            RelativeInputKind::Missing => {
                missing_declared_files.push(format!("{}:{}", logical_name, normalized));
                continue;
            }
            RelativeInputKind::Directory | RelativeInputKind::Unsupported => {
                warnings.push(format!(
                    "manifest path is not a regular file and was not hashed: {}:{}",
                    logical_name, normalized
                ));
                existing_declared_files += 1;
                continue;
            }
            RelativeInputKind::File => {}
        }
        existing_declared_files += 1;
        let descriptor = session
            .describe_relative_file(relative_path)?
            .ok_or_else(|| {
                anyhow::anyhow!("raw-export file disappeared during session: {}", normalized)
            })?;
        file_hashes.insert(logical_name.clone(), descriptor.sha256.clone());
        if let Some(row_count) = descriptor.row_count {
            file_counts.insert(logical_name.clone(), row_count);
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
    let zero_recipe_diagnostics = read_zero_recipe_diagnostics(session, warnings)?;

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
    session: &RawExportSession,
    warnings: &mut Vec<String>,
) -> Result<Option<ZeroRecipeDiagnostics>> {
    let value = if session.manifest().files.contains_key("neiHandlerAnomalies") {
        session.read_optional_manifest_json("neiHandlerAnomalies")?
    } else {
        session.read_relative_json("validation/nei_handler_anomalies.json")?
    };
    let Some(value) = value else {
        warnings.push(
            "zero-recipe diagnostics are missing: validation/nei_handler_anomalies.json"
                .to_string(),
        );
        return Ok(None);
    };
    Ok(zero_recipe_diagnostics_from_value(value.as_ref()))
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
