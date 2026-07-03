use crate::cli::CompileScope;
pub use crate::compiler_scope_catalog::{
    RUNTIME_PACK_PRODUCER_BROWSER, RUNTIME_PACK_PRODUCER_RECIPES, RUNTIME_PACK_PRODUCER_SEARCH,
    RUNTIME_PACK_PRODUCER_TEXTURE, RUNTIME_PACK_PRODUCER_UI,
};
use crate::compiler_scope_catalog::{
    SCOPE_ALL_NATIVE_BROWSER, SCOPE_ALL_NATIVE_RECIPES, SCOPE_ALL_NATIVE_SEARCH_BROWSER,
    SCOPE_ALL_NATIVE_UI, SCOPE_ALL_TEXTURES, SCOPE_EVERY,
};
use crate::io::{normalize_path, sha256_file, write_json_value};
use crate::native_ui_export_abi_catalog::NATIVE_UI_EXPORT_ABI_VALIDATION_REPORT_PATH;
use crate::raw_export_abi::RAW_EXPORT_ABI_VALIDATION_REPORT_PATH;
use crate::runtime_manifest_abi::{
    PATH_POLICY_PORTABLE_RELATIVE_ONLY, RUST_DEPLOYMENT_REPORT_PATH, RUST_INTEGRITY_REPORT_PATH,
    RUST_MIGRATION_READINESS_REPORT_PATH, RUST_MISSING_DATA_REPORT_PATH,
    RUST_RUNTIME_MANIFEST_PATH, RUST_SIZE_REPORT_PATH,
};
use crate::ui_pack_abi::{
    UI_PACK_ABI_VALIDATION_REPORT_PATH, UI_PACK_ABI_VALIDATION_SCHEMA_VERSION,
};
use crate::version::{COMPILED_DIST_SCHEMA_VERSION, PACK_ABI_VERSION};
use anyhow::{anyhow, Result};
use serde::Serialize;
use serde_json::{json, Value};
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

pub const RUNTIME_PACK_PRODUCER_IDS: &[&str] = &[
    RUNTIME_PACK_PRODUCER_BROWSER,
    RUNTIME_PACK_PRODUCER_RECIPES,
    RUNTIME_PACK_PRODUCER_UI,
    RUNTIME_PACK_PRODUCER_TEXTURE,
    RUNTIME_PACK_PRODUCER_SEARCH,
];

const NO_RUNTIME_PACK_PRODUCER: &[&str] = &[];
const PRODUCER_BROWSER: &[&str] = &[RUNTIME_PACK_PRODUCER_BROWSER];
const PRODUCER_RECIPES: &[&str] = &[RUNTIME_PACK_PRODUCER_RECIPES];
const PRODUCER_UI: &[&str] = &[RUNTIME_PACK_PRODUCER_UI];
const PRODUCER_TEXTURE: &[&str] = &[RUNTIME_PACK_PRODUCER_TEXTURE];
const PRODUCERS_BROWSER_SEARCH: &[&str] =
    &[RUNTIME_PACK_PRODUCER_BROWSER, RUNTIME_PACK_PRODUCER_SEARCH];

pub const PACK_ABI_VALIDATION_SCHEMA_VERSION: &str = "elysium-compiler/pack-abi-validation/v1";
pub const PACK_ABI_VALIDATION_REPORT_PATH: &str = "rust/pack-validation-report.json";
pub const RUNTIME_ARTIFACT_CATALOG_SCHEMA_VERSION: &str =
    "elysium-compiler/runtime-artifact-catalog/v1";
pub const SCHEMA_HASH_RUNTIME_ARTIFACT_CATALOG_INPUT: &str =
    "runtime-artifact-catalog=elysium-compiler/runtime-artifact-catalog/v1;producer-map=v1";
pub const RUNTIME_ARTIFACT_STATUS_PRESENT: &str = "present";
pub const RUNTIME_ARTIFACT_STATUS_MISSING: &str = "missing";
pub const PACK_ABI_STATUS_OK: &str = "ok";
pub const PACK_ABI_STATUS_BLOCKED: &str = "blocked";
pub const PACK_ABI_GENERATED_AT: &str = "deterministic-rust-compiler";
pub const PACK_ABI_POLICY_MISSING_REQUIRED_ARTIFACT: &str = "fail-closed";
pub const PACK_ABI_POLICY_LEGACY_FALLBACK: &str = "forbidden";
pub const RUNTIME_DEBUG_DIRECTORY_ARTIFACTS: &[&str] = &["rust/recipe-ui-payload-shards"];

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
    producers: &'static [&'static str],
}

impl RuntimeArtifactSpec {
    pub const fn new(
        logical_name: &'static str,
        relative_path: &'static str,
        kind: RuntimeArtifactKind,
        debug_only: bool,
        scopes: &'static [CompileScope],
        producers: &'static [&'static str],
    ) -> Self {
        Self {
            logical_name,
            relative_path,
            kind,
            debug_only,
            scopes,
            producers,
        }
    }

    pub fn applies_to(self, scope: CompileScope, debug_json: bool) -> bool {
        (!self.debug_only || debug_json) && self.scopes.contains(&scope)
    }

    pub fn scopes(self) -> &'static [CompileScope] {
        self.scopes
    }

    pub fn producers(self) -> &'static [&'static str] {
        self.producers
    }

    pub fn produced_by(self, producer_id: &str) -> bool {
        self.producers.contains(&producer_id)
    }
}

pub const RUNTIME_FINAL_REPORT_SPECS: &[RuntimeArtifactSpec] = &[
    RuntimeArtifactSpec::new(
        "rustRuntimeManifest",
        RUST_RUNTIME_MANIFEST_PATH,
        RuntimeArtifactKind::Manifest,
        false,
        SCOPE_EVERY,
        NO_RUNTIME_PACK_PRODUCER,
    ),
    RuntimeArtifactSpec::new(
        "rustIntegrity",
        RUST_INTEGRITY_REPORT_PATH,
        RuntimeArtifactKind::Report,
        false,
        SCOPE_EVERY,
        NO_RUNTIME_PACK_PRODUCER,
    ),
    RuntimeArtifactSpec::new(
        "rustSizeReport",
        RUST_SIZE_REPORT_PATH,
        RuntimeArtifactKind::Report,
        false,
        SCOPE_EVERY,
        NO_RUNTIME_PACK_PRODUCER,
    ),
    RuntimeArtifactSpec::new(
        "rustMissingDataReport",
        RUST_MISSING_DATA_REPORT_PATH,
        RuntimeArtifactKind::Report,
        false,
        SCOPE_EVERY,
        NO_RUNTIME_PACK_PRODUCER,
    ),
    RuntimeArtifactSpec::new(
        "rustPackValidationReport",
        PACK_ABI_VALIDATION_REPORT_PATH,
        RuntimeArtifactKind::Report,
        false,
        SCOPE_EVERY,
        NO_RUNTIME_PACK_PRODUCER,
    ),
    RuntimeArtifactSpec::new(
        "rustMigrationReadiness",
        RUST_MIGRATION_READINESS_REPORT_PATH,
        RuntimeArtifactKind::Report,
        false,
        SCOPE_EVERY,
        NO_RUNTIME_PACK_PRODUCER,
    ),
    RuntimeArtifactSpec::new(
        "rustDeploymentReport",
        RUST_DEPLOYMENT_REPORT_PATH,
        RuntimeArtifactKind::Report,
        false,
        SCOPE_EVERY,
        NO_RUNTIME_PACK_PRODUCER,
    ),
];

pub const RUNTIME_PACK_ARTIFACT_SPECS: &[RuntimeArtifactSpec] = &[
    RuntimeArtifactSpec::new(
        "rustBrowserBin",
        "rust/browser.bin",
        RuntimeArtifactKind::BinaryPack,
        false,
        SCOPE_ALL_NATIVE_BROWSER,
        PRODUCER_BROWSER,
    ),
    RuntimeArtifactSpec::new(
        "rustGroupsBin",
        "rust/groups.bin",
        RuntimeArtifactKind::BinaryPack,
        false,
        SCOPE_ALL_NATIVE_BROWSER,
        PRODUCER_BROWSER,
    ),
    RuntimeArtifactSpec::new(
        "rustSearchBin",
        "rust/search.bin",
        RuntimeArtifactKind::BinaryPack,
        false,
        SCOPE_ALL_NATIVE_SEARCH_BROWSER,
        PRODUCERS_BROWSER_SEARCH,
    ),
    RuntimeArtifactSpec::new(
        "rustRecipeBin",
        "rust/recipes.bin",
        RuntimeArtifactKind::BinaryPack,
        false,
        SCOPE_ALL_NATIVE_RECIPES,
        PRODUCER_RECIPES,
    ),
    RuntimeArtifactSpec::new(
        "rustTextureBin",
        "rust/textures.bin",
        RuntimeArtifactKind::BinaryPack,
        false,
        SCOPE_ALL_TEXTURES,
        PRODUCER_TEXTURE,
    ),
    RuntimeArtifactSpec::new(
        "rustAtlasMetaBin",
        "rust/atlas.meta.bin",
        RuntimeArtifactKind::BinaryPack,
        false,
        SCOPE_ALL_TEXTURES,
        PRODUCER_TEXTURE,
    ),
    RuntimeArtifactSpec::new(
        "rustAnimationBin",
        "rust/animations.bin",
        RuntimeArtifactKind::BinaryPack,
        false,
        SCOPE_ALL_TEXTURES,
        PRODUCER_TEXTURE,
    ),
    RuntimeArtifactSpec::new(
        "rustStringsZhCnBin",
        "rust/strings.zh_cn.bin",
        RuntimeArtifactKind::BinaryPack,
        false,
        SCOPE_ALL_NATIVE_SEARCH_BROWSER,
        PRODUCERS_BROWSER_SEARCH,
    ),
    RuntimeArtifactSpec::new(
        "rustUiTemplatesBin",
        "rust/ui-pack/ui_templates.bin",
        RuntimeArtifactKind::BinaryPack,
        false,
        SCOPE_ALL_NATIVE_UI,
        PRODUCER_UI,
    ),
    RuntimeArtifactSpec::new(
        "rustUiBindingsBin",
        "rust/ui-pack/ui_bindings.bin",
        RuntimeArtifactKind::BinaryPack,
        false,
        SCOPE_ALL_NATIVE_UI,
        PRODUCER_UI,
    ),
    RuntimeArtifactSpec::new(
        "rustUiStringsBin",
        "rust/ui-pack/ui_strings.bin",
        RuntimeArtifactKind::BinaryPack,
        false,
        SCOPE_ALL_NATIVE_UI,
        PRODUCER_UI,
    ),
    RuntimeArtifactSpec::new(
        "rustUiAssetsManifest",
        "rust/ui-pack/ui_assets.manifest.json",
        RuntimeArtifactKind::Manifest,
        false,
        SCOPE_ALL_NATIVE_UI,
        PRODUCER_UI,
    ),
    RuntimeArtifactSpec::new(
        "rustUiTemplateCatalog",
        "rust/ui-pack/ui_template_catalog.json",
        RuntimeArtifactKind::Manifest,
        false,
        SCOPE_ALL_NATIVE_UI,
        PRODUCER_UI,
    ),
    RuntimeArtifactSpec::new(
        "rustUiTemplateBindingIndex",
        "rust/ui-pack/ui_template_binding_index.json",
        RuntimeArtifactKind::Manifest,
        false,
        SCOPE_ALL_NATIVE_UI,
        PRODUCER_UI,
    ),
    RuntimeArtifactSpec::new(
        "rustUiFamilyCensus",
        "rust/ui-pack/ui_family_census.json",
        RuntimeArtifactKind::Report,
        false,
        SCOPE_ALL_NATIVE_UI,
        PRODUCER_UI,
    ),
    RuntimeArtifactSpec::new(
        "rustUiPackReport",
        "rust/ui-pack/ui_pack_report.json",
        RuntimeArtifactKind::Report,
        false,
        SCOPE_ALL_NATIVE_UI,
        PRODUCER_UI,
    ),
    RuntimeArtifactSpec::new(
        "rustRawExportAbiValidationReport",
        RAW_EXPORT_ABI_VALIDATION_REPORT_PATH,
        RuntimeArtifactKind::Report,
        false,
        SCOPE_EVERY,
        NO_RUNTIME_PACK_PRODUCER,
    ),
    RuntimeArtifactSpec::new(
        "rustNativeUiExportAbiValidationReport",
        NATIVE_UI_EXPORT_ABI_VALIDATION_REPORT_PATH,
        RuntimeArtifactKind::Report,
        false,
        SCOPE_EVERY,
        NO_RUNTIME_PACK_PRODUCER,
    ),
    RuntimeArtifactSpec::new(
        "rustUiPackAbiValidationReport",
        UI_PACK_ABI_VALIDATION_REPORT_PATH,
        RuntimeArtifactKind::Report,
        false,
        SCOPE_EVERY,
        NO_RUNTIME_PACK_PRODUCER,
    ),
    RuntimeArtifactSpec::new(
        "rustSemanticValidationReport",
        "rust/semantic-validation-report.json",
        RuntimeArtifactKind::Report,
        false,
        SCOPE_EVERY,
        NO_RUNTIME_PACK_PRODUCER,
    ),
    RuntimeArtifactSpec::new(
        "rustMissingTextureReport",
        "rust/missing-texture-report.json",
        RuntimeArtifactKind::Report,
        false,
        SCOPE_ALL_TEXTURES,
        PRODUCER_TEXTURE,
    ),
    RuntimeArtifactSpec::new(
        "rustSuspiciousTextureReport",
        "rust/suspicious-texture-report.json",
        RuntimeArtifactKind::Report,
        false,
        SCOPE_ALL_TEXTURES,
        PRODUCER_TEXTURE,
    ),
    RuntimeArtifactSpec::new(
        "rustRecipeHandlerMetadataReport",
        "rust/recipe-handler-metadata-report.json",
        RuntimeArtifactKind::Report,
        false,
        SCOPE_ALL_NATIVE_RECIPES,
        PRODUCER_RECIPES,
    ),
    RuntimeArtifactSpec::new(
        "rustRecipeFragmentationReport",
        "rust/recipe-fragmentation-report.json",
        RuntimeArtifactKind::Report,
        false,
        SCOPE_ALL_NATIVE_RECIPES,
        PRODUCER_RECIPES,
    ),
    RuntimeArtifactSpec::new(
        "rustNativeUiLayoutReport",
        "rust/native-ui-layout-report.json",
        RuntimeArtifactKind::Report,
        false,
        SCOPE_ALL_NATIVE_RECIPES,
        NO_RUNTIME_PACK_PRODUCER,
    ),
    RuntimeArtifactSpec::new(
        "rustBrowserPack",
        "rust/browser-pack.json",
        RuntimeArtifactKind::DebugJsonPack,
        true,
        SCOPE_ALL_NATIVE_BROWSER,
        PRODUCER_BROWSER,
    ),
    RuntimeArtifactSpec::new(
        "rustSearchPack",
        "rust/search-pack.json",
        RuntimeArtifactKind::DebugJsonPack,
        true,
        SCOPE_ALL_NATIVE_SEARCH_BROWSER,
        PRODUCERS_BROWSER_SEARCH,
    ),
    RuntimeArtifactSpec::new(
        "rustRecipePack",
        "rust/recipe-pack.json",
        RuntimeArtifactKind::DebugJsonPack,
        true,
        SCOPE_ALL_NATIVE_RECIPES,
        PRODUCER_RECIPES,
    ),
    RuntimeArtifactSpec::new(
        "rustTexturePack",
        "rust/texture-pack.json",
        RuntimeArtifactKind::DebugJsonPack,
        true,
        SCOPE_ALL_TEXTURES,
        PRODUCER_TEXTURE,
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

pub fn runtime_artifact_catalog_specs() -> Vec<&'static RuntimeArtifactSpec> {
    RUNTIME_FINAL_REPORT_SPECS
        .iter()
        .chain(RUNTIME_PACK_ARTIFACT_SPECS.iter())
        .collect()
}

pub fn runtime_pack_artifact_specs_for_producer(
    producer_id: &str,
    include_debug: bool,
) -> Vec<&'static RuntimeArtifactSpec> {
    RUNTIME_PACK_ARTIFACT_SPECS
        .iter()
        .filter(|spec| spec.produced_by(producer_id) && (!spec.debug_only || include_debug))
        .collect()
}

pub fn runtime_pack_artifact_paths_for_producer(
    producer_id: &str,
    include_debug: bool,
) -> Vec<&'static str> {
    runtime_pack_artifact_specs_for_producer(producer_id, include_debug)
        .into_iter()
        .map(|spec| spec.relative_path)
        .collect()
}

pub fn runtime_debug_artifact_specs() -> Vec<&'static RuntimeArtifactSpec> {
    RUNTIME_PACK_ARTIFACT_SPECS
        .iter()
        .filter(|spec| spec.debug_only)
        .collect()
}

pub fn runtime_summary_artifact_specs() -> Vec<(&'static str, &'static str)> {
    let mut specs = vec![("manifest", "manifest.json")];
    specs.extend(
        runtime_artifact_catalog_specs()
            .into_iter()
            .map(|spec| (spec.logical_name, spec.relative_path)),
    );
    specs.push(("runtimeValidationReport", "validation/report.json"));
    specs
}

pub fn runtime_artifact_catalog() -> Value {
    validate_runtime_artifact_catalog_descriptors();
    json!({
        "schemaVersion": RUNTIME_ARTIFACT_CATALOG_SCHEMA_VERSION,
        "packAbiVersion": PACK_ABI_VERSION,
        "schemaHashInput": SCHEMA_HASH_RUNTIME_ARTIFACT_CATALOG_INPUT,
        "policy": {
            "missingRequiredArtifact": PACK_ABI_POLICY_MISSING_REQUIRED_ARTIFACT,
            "pathPortability": PATH_POLICY_PORTABLE_RELATIVE_ONLY,
            "legacyFallback": PACK_ABI_POLICY_LEGACY_FALLBACK,
            "registration": "static-runtime-artifact-descriptor-table",
            "producerOwnership": "runtime pack compiler catalogs derive outputs from artifact producer descriptors"
        },
        "producers": RUNTIME_PACK_PRODUCER_IDS,
        "artifacts": runtime_artifact_catalog_specs()
            .into_iter()
            .map(runtime_artifact_descriptor_catalog)
            .collect::<Vec<_>>()
    })
}

pub fn pack_abi_catalog() -> Value {
    json!({
        "name": "elysium.pack",
        "version": PACK_ABI_VERSION,
        "status": "testing",
        "legacyCompiledDistSchemaVersion": COMPILED_DIST_SCHEMA_VERSION,
        "root": "elysium.pack.v1/",
        "requiredFiles": [
            "manifest.json",
            "abi.json"
        ],
        "requiredDirectories": [
            "packs/",
            "reports/"
        ],
        "validationReport": {
            "path": PACK_ABI_VALIDATION_REPORT_PATH,
            "schemaVersion": PACK_ABI_VALIDATION_SCHEMA_VERSION,
            "policy": "missing required runtime artifacts, path leaks, and legacy fallback are compile blockers"
        },
        "runtimeArtifacts": runtime_artifact_catalog(),
        "runtimePacks": [
            "packs/items.pack",
            "packs/recipes.pack",
            "packs/search.pack",
            "packs/native-ui.pack",
            "packs/textures.pack",
            "packs/browser.pack",
            "packs/bootstrap.pack"
        ],
        "nativeUiPack": {
            "path": "packs/native-ui.pack",
            "encoding": "binary",
            "validationReport": {
                "path": UI_PACK_ABI_VALIDATION_REPORT_PATH,
                "schemaVersion": UI_PACK_ABI_VALIDATION_SCHEMA_VERSION,
                "policy": "native UI binary envelope, section layout, string references, and sidecar schemaVersion are compile blockers"
            },
            "sections": [
                "header",
                "stringTable",
                "surfaceTable",
                "slotRectTable",
                "interactionTable",
                "textureRegionTable",
                "animationTable",
                "checksumTable"
            ],
            "layoutPolicy": "renderer consumes exported design-space coordinates; no frontend reflow"
        },
        "hotPathPolicy": "JSON is for manifests, schemas, reports, and diagnostics; runtime hot paths prefer binary packs."
    })
}

fn runtime_artifact_descriptor_catalog(spec: &'static RuntimeArtifactSpec) -> Value {
    json!({
        "logicalName": spec.logical_name,
        "path": spec.relative_path,
        "kind": spec.kind.as_str(),
        "required": true,
        "debugOnly": spec.debug_only,
        "producers": spec.producers(),
        "scopes": spec.scopes()
            .iter()
            .map(|scope| scope.as_str())
            .collect::<Vec<_>>()
    })
}

pub fn runtime_pack_artifact_specs(
    scope: CompileScope,
    debug_json: bool,
) -> Vec<&'static RuntimeArtifactSpec> {
    validate_runtime_artifact_catalog_descriptors();
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
    output.join(PACK_ABI_VALIDATION_REPORT_PATH)
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
                status: RUNTIME_ARTIFACT_STATUS_PRESENT,
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
                status: RUNTIME_ARTIFACT_STATUS_MISSING,
                bytes: None,
                sha256: None,
            });
        }
    }
    let path_violations = collect_text_path_violations(&output.join("rust"))?;
    let status = if missing_required_artifacts.is_empty() && path_violations.is_empty() {
        PACK_ABI_STATUS_OK
    } else {
        PACK_ABI_STATUS_BLOCKED
    };
    let present_artifact_count = artifacts
        .iter()
        .filter(|artifact| artifact.status == RUNTIME_ARTIFACT_STATUS_PRESENT)
        .count();
    Ok(PackAbiValidationReport {
        schema_version: PACK_ABI_VALIDATION_SCHEMA_VERSION,
        pack_abi_version: PACK_ABI_VERSION,
        generated_at: PACK_ABI_GENERATED_AT,
        status,
        compile_scope: scope.as_str().to_string(),
        debug_json,
        expected_artifact_count: artifacts.len(),
        present_artifact_count,
        missing_required_artifacts,
        path_violations,
        artifacts,
        policy: PackAbiPolicy {
            missing_required_artifact: PACK_ABI_POLICY_MISSING_REQUIRED_ARTIFACT,
            path_portability: PATH_POLICY_PORTABLE_RELATIVE_ONLY,
            legacy_fallback: PACK_ABI_POLICY_LEGACY_FALLBACK,
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

pub fn validate_runtime_artifact_catalog_descriptors() {
    if RUNTIME_PACK_ARTIFACT_SPECS.is_empty() {
        panic!("runtime pack artifact descriptor catalog must not be empty");
    }

    let mut logical_names = BTreeSet::new();
    let mut paths = BTreeSet::new();
    for spec in runtime_artifact_catalog_specs() {
        require_non_empty("runtime artifact logical name", spec.logical_name);
        require_portable_runtime_path(spec.logical_name, spec.relative_path);
        if !logical_names.insert(spec.logical_name) {
            panic!(
                "duplicate runtime artifact logical name: {}",
                spec.logical_name
            );
        }
        if !paths.insert(spec.relative_path) {
            panic!("duplicate runtime artifact path: {}", spec.relative_path);
        }
        if matches!(
            spec.kind,
            RuntimeArtifactKind::BinaryPack | RuntimeArtifactKind::DebugJsonPack
        ) && spec.producers.is_empty()
        {
            panic!(
                "runtime pack artifact must declare a producer: {}",
                spec.logical_name
            );
        }
        if spec.debug_only && spec.kind != RuntimeArtifactKind::DebugJsonPack {
            panic!(
                "debug-only runtime artifact must use debug-json-pack kind: {}",
                spec.logical_name
            );
        }
        for producer in spec.producers {
            require_non_empty("runtime artifact producer id", producer);
            if !RUNTIME_PACK_PRODUCER_IDS.contains(producer) {
                panic!(
                    "unknown runtime artifact producer {} for {}",
                    producer, spec.logical_name
                );
            }
        }
    }

    for producer in RUNTIME_PACK_PRODUCER_IDS {
        let owned_specs = runtime_pack_artifact_specs_for_producer(producer, false);
        if owned_specs.is_empty() {
            panic!("runtime pack producer has no production artifacts: {producer}");
        }
    }
}

fn require_portable_runtime_path(logical_name: &str, path: &str) {
    require_non_empty("runtime artifact path", path);
    if path.starts_with('/')
        || path.starts_with('\\')
        || path.contains('\\')
        || path.contains(':')
        || path.contains("://")
        || path.contains("..")
    {
        panic!("runtime artifact path must be portable-relative: {logical_name}");
    }
}

fn require_non_empty(label: &str, value: &str) {
    if value.trim().is_empty() {
        panic!("{label} must be non-empty");
    }
}
