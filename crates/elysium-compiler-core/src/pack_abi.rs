use crate::cli::CompileScope;
use crate::io::{normalize_path, sha256_file, write_json_value};
use crate::raw_export_abi::RAW_EXPORT_ABI_VALIDATION_REPORT_PATH;
use crate::version::PACK_ABI_VERSION;
use anyhow::{anyhow, Result};
use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};

const SCOPE_EVERY: &[CompileScope] = &[
    CompileScope::All,
    CompileScope::NativeUi,
    CompileScope::Search,
    CompileScope::Browser,
    CompileScope::Recipes,
    CompileScope::Ui,
    CompileScope::Textures,
];
const SCOPE_ALL_NATIVE_BROWSER: &[CompileScope] = &[
    CompileScope::All,
    CompileScope::NativeUi,
    CompileScope::Browser,
];
const SCOPE_ALL_NATIVE_SEARCH_BROWSER: &[CompileScope] = &[
    CompileScope::All,
    CompileScope::NativeUi,
    CompileScope::Search,
    CompileScope::Browser,
];
const SCOPE_ALL_NATIVE_RECIPES: &[CompileScope] = &[
    CompileScope::All,
    CompileScope::NativeUi,
    CompileScope::Recipes,
];
const SCOPE_ALL_NATIVE_UI: &[CompileScope] =
    &[CompileScope::All, CompileScope::NativeUi, CompileScope::Ui];
const SCOPE_ALL_TEXTURES: &[CompileScope] = &[CompileScope::All, CompileScope::Textures];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RuntimeArtifactKind {
    BinaryPack,
    DebugJsonPack,
    Report,
    Manifest,
}

impl RuntimeArtifactKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            RuntimeArtifactKind::BinaryPack => "binary-pack",
            RuntimeArtifactKind::DebugJsonPack => "debug-json-pack",
            RuntimeArtifactKind::Report => "report",
            RuntimeArtifactKind::Manifest => "manifest",
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct RuntimeArtifactSpec {
    pub logical_name: &'static str,
    pub relative_path: &'static str,
    pub kind: RuntimeArtifactKind,
    pub debug_only: bool,
    scopes: &'static [CompileScope],
}

impl RuntimeArtifactSpec {
    pub const fn new(
        logical_name: &'static str,
        relative_path: &'static str,
        kind: RuntimeArtifactKind,
        debug_only: bool,
        scopes: &'static [CompileScope],
    ) -> Self {
        Self {
            logical_name,
            relative_path,
            kind,
            debug_only,
            scopes,
        }
    }

    pub fn applies_to(self, scope: CompileScope, debug_json: bool) -> bool {
        (!self.debug_only || debug_json) && self.scopes.contains(&scope)
    }
}

pub const RUNTIME_FINAL_REPORT_SPECS: &[RuntimeArtifactSpec] = &[
    RuntimeArtifactSpec::new(
        "rustRuntimeManifest",
        "rust/runtime-manifest.json",
        RuntimeArtifactKind::Manifest,
        false,
        SCOPE_EVERY,
    ),
    RuntimeArtifactSpec::new(
        "rustIntegrity",
        "rust/integrity.json",
        RuntimeArtifactKind::Report,
        false,
        SCOPE_EVERY,
    ),
    RuntimeArtifactSpec::new(
        "rustSizeReport",
        "rust/size-report.json",
        RuntimeArtifactKind::Report,
        false,
        SCOPE_EVERY,
    ),
    RuntimeArtifactSpec::new(
        "rustMissingDataReport",
        "rust/missing-data-report.json",
        RuntimeArtifactKind::Report,
        false,
        SCOPE_EVERY,
    ),
    RuntimeArtifactSpec::new(
        "rustPackValidationReport",
        "rust/pack-validation-report.json",
        RuntimeArtifactKind::Report,
        false,
        SCOPE_EVERY,
    ),
    RuntimeArtifactSpec::new(
        "rustMigrationReadiness",
        "rust/migration-readiness.json",
        RuntimeArtifactKind::Report,
        false,
        SCOPE_EVERY,
    ),
    RuntimeArtifactSpec::new(
        "rustDeploymentReport",
        "rust/deployment-report.json",
        RuntimeArtifactKind::Report,
        false,
        SCOPE_EVERY,
    ),
];

pub const RUNTIME_PACK_ARTIFACT_SPECS: &[RuntimeArtifactSpec] = &[
    RuntimeArtifactSpec::new(
        "rustBrowserBin",
        "rust/browser.bin",
        RuntimeArtifactKind::BinaryPack,
        false,
        SCOPE_ALL_NATIVE_BROWSER,
    ),
    RuntimeArtifactSpec::new(
        "rustGroupsBin",
        "rust/groups.bin",
        RuntimeArtifactKind::BinaryPack,
        false,
        SCOPE_ALL_NATIVE_BROWSER,
    ),
    RuntimeArtifactSpec::new(
        "rustSearchBin",
        "rust/search.bin",
        RuntimeArtifactKind::BinaryPack,
        false,
        SCOPE_ALL_NATIVE_SEARCH_BROWSER,
    ),
    RuntimeArtifactSpec::new(
        "rustRecipeBin",
        "rust/recipes.bin",
        RuntimeArtifactKind::BinaryPack,
        false,
        SCOPE_ALL_NATIVE_RECIPES,
    ),
    RuntimeArtifactSpec::new(
        "rustTextureBin",
        "rust/textures.bin",
        RuntimeArtifactKind::BinaryPack,
        false,
        SCOPE_ALL_TEXTURES,
    ),
    RuntimeArtifactSpec::new(
        "rustAtlasMetaBin",
        "rust/atlas.meta.bin",
        RuntimeArtifactKind::BinaryPack,
        false,
        SCOPE_ALL_TEXTURES,
    ),
    RuntimeArtifactSpec::new(
        "rustAnimationBin",
        "rust/animations.bin",
        RuntimeArtifactKind::BinaryPack,
        false,
        SCOPE_ALL_TEXTURES,
    ),
    RuntimeArtifactSpec::new(
        "rustStringsZhCnBin",
        "rust/strings.zh_cn.bin",
        RuntimeArtifactKind::BinaryPack,
        false,
        SCOPE_ALL_NATIVE_SEARCH_BROWSER,
    ),
    RuntimeArtifactSpec::new(
        "rustUiTemplatesBin",
        "rust/ui-pack/ui_templates.bin",
        RuntimeArtifactKind::BinaryPack,
        false,
        SCOPE_ALL_NATIVE_UI,
    ),
    RuntimeArtifactSpec::new(
        "rustUiBindingsBin",
        "rust/ui-pack/ui_bindings.bin",
        RuntimeArtifactKind::BinaryPack,
        false,
        SCOPE_ALL_NATIVE_UI,
    ),
    RuntimeArtifactSpec::new(
        "rustUiStringsBin",
        "rust/ui-pack/ui_strings.bin",
        RuntimeArtifactKind::BinaryPack,
        false,
        SCOPE_ALL_NATIVE_UI,
    ),
    RuntimeArtifactSpec::new(
        "rustUiAssetsManifest",
        "rust/ui-pack/ui_assets.manifest.json",
        RuntimeArtifactKind::Manifest,
        false,
        SCOPE_ALL_NATIVE_UI,
    ),
    RuntimeArtifactSpec::new(
        "rustUiTemplateCatalog",
        "rust/ui-pack/ui_template_catalog.json",
        RuntimeArtifactKind::Manifest,
        false,
        SCOPE_ALL_NATIVE_UI,
    ),
    RuntimeArtifactSpec::new(
        "rustUiTemplateBindingIndex",
        "rust/ui-pack/ui_template_binding_index.json",
        RuntimeArtifactKind::Manifest,
        false,
        SCOPE_ALL_NATIVE_UI,
    ),
    RuntimeArtifactSpec::new(
        "rustUiFamilyCensus",
        "rust/ui-pack/ui_family_census.json",
        RuntimeArtifactKind::Report,
        false,
        SCOPE_ALL_NATIVE_UI,
    ),
    RuntimeArtifactSpec::new(
        "rustUiPackReport",
        "rust/ui-pack/ui_pack_report.json",
        RuntimeArtifactKind::Report,
        false,
        SCOPE_ALL_NATIVE_UI,
    ),
    RuntimeArtifactSpec::new(
        "rustRawExportAbiValidationReport",
        RAW_EXPORT_ABI_VALIDATION_REPORT_PATH,
        RuntimeArtifactKind::Report,
        false,
        SCOPE_EVERY,
    ),
    RuntimeArtifactSpec::new(
        "rustSemanticValidationReport",
        "rust/semantic-validation-report.json",
        RuntimeArtifactKind::Report,
        false,
        SCOPE_EVERY,
    ),
    RuntimeArtifactSpec::new(
        "rustMissingTextureReport",
        "rust/missing-texture-report.json",
        RuntimeArtifactKind::Report,
        false,
        SCOPE_ALL_TEXTURES,
    ),
    RuntimeArtifactSpec::new(
        "rustSuspiciousTextureReport",
        "rust/suspicious-texture-report.json",
        RuntimeArtifactKind::Report,
        false,
        SCOPE_ALL_TEXTURES,
    ),
    RuntimeArtifactSpec::new(
        "rustRecipeHandlerMetadataReport",
        "rust/recipe-handler-metadata-report.json",
        RuntimeArtifactKind::Report,
        false,
        SCOPE_ALL_NATIVE_RECIPES,
    ),
    RuntimeArtifactSpec::new(
        "rustRecipeFragmentationReport",
        "rust/recipe-fragmentation-report.json",
        RuntimeArtifactKind::Report,
        false,
        SCOPE_ALL_NATIVE_RECIPES,
    ),
    RuntimeArtifactSpec::new(
        "rustNativeUiLayoutReport",
        "rust/native-ui-layout-report.json",
        RuntimeArtifactKind::Report,
        false,
        SCOPE_ALL_NATIVE_RECIPES,
    ),
    RuntimeArtifactSpec::new(
        "rustBrowserPack",
        "rust/browser-pack.json",
        RuntimeArtifactKind::DebugJsonPack,
        true,
        SCOPE_ALL_NATIVE_BROWSER,
    ),
    RuntimeArtifactSpec::new(
        "rustSearchPack",
        "rust/search-pack.json",
        RuntimeArtifactKind::DebugJsonPack,
        true,
        SCOPE_ALL_NATIVE_SEARCH_BROWSER,
    ),
    RuntimeArtifactSpec::new(
        "rustRecipePack",
        "rust/recipe-pack.json",
        RuntimeArtifactKind::DebugJsonPack,
        true,
        SCOPE_ALL_NATIVE_RECIPES,
    ),
    RuntimeArtifactSpec::new(
        "rustTexturePack",
        "rust/texture-pack.json",
        RuntimeArtifactKind::DebugJsonPack,
        true,
        SCOPE_ALL_TEXTURES,
    ),
];

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeArtifactRecord {
    pub logical_name: String,
    pub path: String,
    pub kind: &'static str,
    pub required: bool,
    pub debug_only: bool,
    pub status: &'static str,
    pub bytes: Option<u64>,
    pub sha256: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PackAbiValidationReport {
    pub schema_version: &'static str,
    pub pack_abi_version: &'static str,
    pub generated_at: &'static str,
    pub status: &'static str,
    pub compile_scope: String,
    pub debug_json: bool,
    pub expected_artifact_count: usize,
    pub present_artifact_count: usize,
    pub missing_required_artifacts: Vec<String>,
    pub path_violations: Vec<String>,
    pub artifacts: Vec<RuntimeArtifactRecord>,
    pub policy: PackAbiPolicy,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PackAbiPolicy {
    pub missing_required_artifact: &'static str,
    pub path_portability: &'static str,
    pub legacy_fallback: &'static str,
}

impl PackAbiValidationReport {
    pub fn is_blocked(&self) -> bool {
        !self.missing_required_artifacts.is_empty() || !self.path_violations.is_empty()
    }
}

pub fn runtime_pack_artifact_specs(
    scope: CompileScope,
    debug_json: bool,
) -> Vec<&'static RuntimeArtifactSpec> {
    RUNTIME_PACK_ARTIFACT_SPECS
        .iter()
        .filter(|spec| spec.applies_to(scope, debug_json))
        .collect()
}

pub fn runtime_manifest_file_entries(
    scope: CompileScope,
    debug_json: bool,
) -> Vec<(&'static str, &'static str)> {
    let mut entries = RUNTIME_FINAL_REPORT_SPECS
        .iter()
        .filter(|spec| spec.applies_to(scope, debug_json))
        .map(|spec| (spec.logical_name, spec.relative_path))
        .collect::<Vec<_>>();
    entries.extend(
        runtime_pack_artifact_specs(scope, debug_json)
            .into_iter()
            .map(|spec| (spec.logical_name, spec.relative_path)),
    );
    entries
}

pub fn pack_validation_report_path(output: &Path) -> PathBuf {
    output.join("rust").join("pack-validation-report.json")
}

pub fn purge_out_of_scope_runtime_artifacts(
    output: &Path,
    scope: CompileScope,
    debug_json: bool,
) -> Result<()> {
    for spec in RUNTIME_PACK_ARTIFACT_SPECS {
        if spec.applies_to(scope, debug_json) {
            continue;
        }
        let path = output.join(spec.relative_path);
        if path.exists() {
            fs::remove_file(&path)?;
        }
    }
    Ok(())
}

pub fn write_pack_abi_validation_report(
    output: &Path,
    scope: CompileScope,
    debug_json: bool,
    strict: bool,
) -> Result<PackAbiValidationReport> {
    let report = validate_runtime_pack_abi(output, scope, debug_json)?;
    let path = pack_validation_report_path(output);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    write_json_value(&path, &serde_json::to_value(&report)?)?;
    if strict && report.is_blocked() {
        return Err(anyhow!(
            "pack ABI validation blocked: missing={}, path violations={}",
            report.missing_required_artifacts.len(),
            report.path_violations.len()
        ));
    }
    Ok(report)
}

pub fn validate_runtime_pack_abi(
    output: &Path,
    scope: CompileScope,
    debug_json: bool,
) -> Result<PackAbiValidationReport> {
    let specs = runtime_pack_artifact_specs(scope, debug_json);
    let mut artifacts = Vec::with_capacity(specs.len());
    let mut missing_required_artifacts = Vec::new();
    for spec in specs {
        let path = output.join(spec.relative_path);
        if path.exists() {
            let bytes = path.metadata()?.len();
            artifacts.push(RuntimeArtifactRecord {
                logical_name: spec.logical_name.to_string(),
                path: spec.relative_path.to_string(),
                kind: spec.kind.as_str(),
                required: true,
                debug_only: spec.debug_only,
                status: "present",
                bytes: Some(bytes),
                sha256: Some(sha256_file(&path)?),
            });
        } else {
            missing_required_artifacts.push(spec.relative_path.to_string());
            artifacts.push(RuntimeArtifactRecord {
                logical_name: spec.logical_name.to_string(),
                path: spec.relative_path.to_string(),
                kind: spec.kind.as_str(),
                required: true,
                debug_only: spec.debug_only,
                status: "missing",
                bytes: None,
                sha256: None,
            });
        }
    }
    let path_violations = collect_text_path_violations(&output.join("rust"))?;
    let status = if missing_required_artifacts.is_empty() && path_violations.is_empty() {
        "ok"
    } else {
        "blocked"
    };
    let present_artifact_count = artifacts
        .iter()
        .filter(|artifact| artifact.status == "present")
        .count();
    Ok(PackAbiValidationReport {
        schema_version: "elysium-compiler/pack-abi-validation/v1",
        pack_abi_version: PACK_ABI_VERSION,
        generated_at: "deterministic-rust-compiler",
        status,
        compile_scope: scope.as_str().to_string(),
        debug_json,
        expected_artifact_count: artifacts.len(),
        present_artifact_count,
        missing_required_artifacts,
        path_violations,
        artifacts,
        policy: PackAbiPolicy {
            missing_required_artifact: "fail-closed",
            path_portability:
                "portable-relative-runtime-paths-only; no drive letters, UNC paths, or file URLs",
            legacy_fallback: "forbidden",
        },
    })
}

pub fn collect_text_path_violations(root: &Path) -> Result<Vec<String>> {
    if !root.exists() {
        return Ok(Vec::new());
    }
    let mut violations = Vec::new();
    for entry in walkdir::WalkDir::new(root)
        .into_iter()
        .filter_map(std::result::Result::ok)
        .filter(|entry| entry.file_type().is_file())
        .filter(|entry| is_text_runtime_artifact(entry.path()))
    {
        let path = entry.path();
        let text = fs::read_to_string(path)?;
        for needle in ["E:\\", "C:\\", "\\\\", "file://"] {
            if text.contains(needle) {
                violations.push(format!("{} contains {}", normalize_path(path), needle));
            }
        }
    }
    Ok(violations)
}

pub fn is_text_runtime_artifact(path: &Path) -> bool {
    path.extension()
        .and_then(|value| value.to_str())
        .is_some_and(|extension| matches!(extension, "json" | "txt" | "log"))
}
