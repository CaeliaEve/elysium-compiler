use anyhow::{Context, Result};
use serde::Serialize;
use serde_json::Value;
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Serialize)]
pub struct CompilerReport {
    pub schema_version: &'static str,
    pub mode: String,
    pub input: String,
    pub output: Option<String>,
    pub elapsed_ms: u128,
    pub raw_export: RawExportSummary,
    pub runtime: RuntimeSummary,
    pub warnings: Vec<String>,
    pub blocked: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct RuntimeSummary {
    pub counts: BTreeMap<String, u64>,
    pub sizes: BTreeMap<String, u64>,
}

#[derive(Debug, Serialize)]
pub struct RawExportSummary {
    pub manifest_schema_version: Option<String>,
    pub repository_name: Option<String>,
    pub generated_at: Option<Value>,
    pub capabilities: Vec<String>,
    pub declared_files: usize,
    pub existing_declared_files: usize,
    pub missing_declared_files: Vec<String>,
    pub file_counts: BTreeMap<String, u64>,
    pub file_hashes: BTreeMap<String, String>,
    #[serde(rename = "zeroRecipeDiagnostics")]
    pub zero_recipe_diagnostics: Option<ZeroRecipeDiagnostics>,
}

#[derive(Debug, Serialize)]
pub struct ZeroRecipeDiagnostics {
    pub status: Option<String>,
    #[serde(rename = "totalHandlers")]
    pub total_handlers: u64,
    #[serde(rename = "handlersWithLoadedRecipes")]
    pub handlers_with_loaded_recipes: u64,
    #[serde(rename = "handlersWithExportedRecipes")]
    pub handlers_with_exported_recipes: u64,
    #[serde(rename = "legalZeroRecipeHandlers")]
    pub legal_zero_recipe_handlers: u64,
    #[serde(rename = "expectedEmptyHandlers")]
    pub expected_empty_handlers: u64,
    #[serde(rename = "nativeCoveredZeroExports")]
    pub native_covered_zero_exports: u64,
    #[serde(rename = "nonRecipeInfoZeroExports")]
    pub non_recipe_info_zero_exports: u64,
    #[serde(rename = "suspiciousZeroExports")]
    pub suspicious_zero_exports: u64,
    #[serde(rename = "partialExports")]
    pub partial_exports: u64,
}

pub fn summarize_runtime_output(output: Option<&Path>) -> Result<RuntimeSummary> {
    let mut counts = BTreeMap::new();
    let mut sizes = BTreeMap::new();
    let Some(output) = output else {
        return Ok(RuntimeSummary { counts, sizes });
    };

    let validation_report = output.join("validation").join("report.json");
    if validation_report.exists() {
        let text = fs::read_to_string(&validation_report).with_context(|| {
            format!(
                "read runtime validation report {}",
                validation_report.display()
            )
        })?;
        let value: Value = serde_json::from_str(&text).with_context(|| {
            format!(
                "parse runtime validation report {}",
                validation_report.display()
            )
        })?;
        if let Some(report_counts) = value.get("counts").and_then(Value::as_object) {
            for (key, value) in report_counts {
                if let Some(count) = value.as_u64() {
                    counts.insert(key.clone(), count);
                }
            }
        }
    }

    for (logical_name, relative_path) in [
        ("manifest", "manifest.json"),
        ("browserItemCatalog", "browser/item-catalog.json"),
        ("browserGroupIndex", "browser/group-index.json"),
        ("searchPack", "search/all.json"),
        ("recipeItemIndex", "recipes/item-index.json"),
        ("recipeHandlerIndex", "recipes/handler-index.json"),
        ("browserAtlasIndex", "textures/browser-atlas-index.json"),
        ("animationTable", "textures/animation-table.json"),
        ("uiTemplatesBin", "rust/ui-pack/ui_templates.bin"),
        ("uiBindingsBin", "rust/ui-pack/ui_bindings.bin"),
        ("uiStringsBin", "rust/ui-pack/ui_strings.bin"),
        ("uiAssetsManifest", "rust/ui-pack/ui_assets.manifest.json"),
        ("uiPackReport", "rust/ui-pack/ui_pack_report.json"),
        ("nativeUiLayoutReport", "rust/native-ui-layout-report.json"),
        ("runtimeValidationReport", "validation/report.json"),
    ] {
        let path = output.join(relative_path);
        if path.exists() {
            sizes.insert(logical_name.to_string(), path.metadata()?.len());
        }
    }

    Ok(RuntimeSummary { counts, sizes })
}

pub fn write_report(path: &Path, report: &CompilerReport) -> Result<()> {
    let text = serde_json::to_string_pretty(report)?;
    fs::write(path, text).with_context(|| format!("write report {}", path.display()))
}
