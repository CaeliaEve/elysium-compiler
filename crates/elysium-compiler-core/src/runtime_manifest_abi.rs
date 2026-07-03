//! Runtime manifest ABI catalog.
//!
//! Runtime manifests are the consumer-facing contract for compiled packs. Keep
//! schema names, report paths, entrypoint names, scope capabilities, and path
//! policy in one module so producers, validators, and NeoNEI gates cannot drift.

use crate::cli::CompileScope;
use crate::compiler_scope_catalog::{
    compile_scope_descriptors, compile_scope_runtime_capabilities,
};
use std::collections::BTreeMap;
use std::collections::BTreeSet;

use serde_json::Map;
use serde_json::{json, Value};

pub const RUST_RUNTIME_SCHEMA: &str = "neonei/runtime/current";
pub const RUST_RUNTIME_MANIFEST_SCHEMA_VERSION: &str = "neonei/rust-runtime-manifest/current";
pub const RUST_RUNTIME_SCHEMA_REVISION: u32 = 1;
pub const NATIVE_RUNTIME_DIST_SCHEMA_VERSION: &str = "neonei/native-runtime-dist/current";

pub const RUST_INTEGRITY_SCHEMA_VERSION: &str = "neonei/rust-integrity/current";
pub const RUST_SIZE_REPORT_SCHEMA_VERSION: &str = "neonei/rust-size-report/current";
pub const RUST_MISSING_DATA_REPORT_SCHEMA_VERSION: &str = "neonei/rust-missing-data-report/current";
pub const RUST_MIGRATION_READINESS_SCHEMA_VERSION: &str = "neonei/rust-migration-readiness/current";
pub const RUST_DEPLOYMENT_REPORT_SCHEMA_VERSION: &str = "neonei/rust-deployment-report/current";

pub const RUST_RUNTIME_MANIFEST_PATH: &str = "rust/runtime-manifest.json";
pub const RUST_INTEGRITY_REPORT_PATH: &str = "rust/integrity.json";
pub const RUST_SIZE_REPORT_PATH: &str = "rust/size-report.json";
pub const RUST_MISSING_DATA_REPORT_PATH: &str = "rust/missing-data-report.json";
pub const RUST_MIGRATION_READINESS_REPORT_PATH: &str = "rust/migration-readiness.json";
pub const RUST_DEPLOYMENT_REPORT_PATH: &str = "rust/deployment-report.json";

pub const PATH_POLICY_PORTABLE_RELATIVE_ONLY: &str =
    "portable-relative-runtime-paths-only; no drive letters, UNC paths, or file URLs";
pub const SCHEMA_HASH_RUNTIME_MANIFEST_INPUT: &str =
    "runtime-manifest-abi=neonei/rust-runtime-manifest/current;revision=1";
pub const SCHEMA_HASH_RUNTIME_REPORT_PAYLOAD_POLICY_INPUT: &str =
    "runtime-report-payload-policy=neonei/rust-runtime-manifest/current;payload-policy=v1";

pub const RUST_RUNTIME_GENERATED_AT: &str = "deterministic-rust-compiler";
pub const RUST_RUNTIME_INTEGRITY_ALGORITHM: &str = "sha256";
pub const NATIVE_RUNTIME_STATUS_READY: &str = "ready";
pub const NATIVE_RUNTIME_AUTHORITY_RUST: &str = "rust";
pub const DIST_MANIFEST_SCHEMA_VERSION: &str = "neonei/dist-data/current";
pub const DIST_MANIFEST_SOURCE: &str = "elysium-compiler";

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum RuntimeManifestReportKind {
    Integrity,
    Size,
    MissingData,
    MigrationReadiness,
    Deployment,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RuntimeManifestReportDescriptor {
    pub key: &'static str,
    pub path: &'static str,
    pub schema_version: &'static str,
    pub kind: RuntimeManifestReportKind,
}

impl RuntimeManifestReportDescriptor {
    const fn new(
        key: &'static str,
        path: &'static str,
        schema_version: &'static str,
        kind: RuntimeManifestReportKind,
    ) -> Self {
        Self {
            key,
            path,
            schema_version,
            kind,
        }
    }
}

pub const RUNTIME_MANIFEST_REPORT_DESCRIPTORS: &[RuntimeManifestReportDescriptor] = &[
    RuntimeManifestReportDescriptor::new(
        "integrity",
        RUST_INTEGRITY_REPORT_PATH,
        RUST_INTEGRITY_SCHEMA_VERSION,
        RuntimeManifestReportKind::Integrity,
    ),
    RuntimeManifestReportDescriptor::new(
        "sizeReport",
        RUST_SIZE_REPORT_PATH,
        RUST_SIZE_REPORT_SCHEMA_VERSION,
        RuntimeManifestReportKind::Size,
    ),
    RuntimeManifestReportDescriptor::new(
        "missingDataReport",
        RUST_MISSING_DATA_REPORT_PATH,
        RUST_MISSING_DATA_REPORT_SCHEMA_VERSION,
        RuntimeManifestReportKind::MissingData,
    ),
    RuntimeManifestReportDescriptor::new(
        "migrationReadiness",
        RUST_MIGRATION_READINESS_REPORT_PATH,
        RUST_MIGRATION_READINESS_SCHEMA_VERSION,
        RuntimeManifestReportKind::MigrationReadiness,
    ),
    RuntimeManifestReportDescriptor::new(
        "deploymentReport",
        RUST_DEPLOYMENT_REPORT_PATH,
        RUST_DEPLOYMENT_REPORT_SCHEMA_VERSION,
        RuntimeManifestReportKind::Deployment,
    ),
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RuntimeEntrypointSpec {
    pub key: &'static str,
    pub path: &'static str,
}

pub const RUST_RUNTIME_ENTRYPOINTS: &[RuntimeEntrypointSpec] = &[
    RuntimeEntrypointSpec {
        key: "browser",
        path: "rust/browser.bin",
    },
    RuntimeEntrypointSpec {
        key: "groups",
        path: "rust/groups.bin",
    },
    RuntimeEntrypointSpec {
        key: "search",
        path: "rust/search.bin",
    },
    RuntimeEntrypointSpec {
        key: "recipes",
        path: "rust/recipes.bin",
    },
    RuntimeEntrypointSpec {
        key: "textures",
        path: "rust/textures.bin",
    },
    RuntimeEntrypointSpec {
        key: "atlasMeta",
        path: "rust/atlas.meta.bin",
    },
    RuntimeEntrypointSpec {
        key: "animations",
        path: "rust/animations.bin",
    },
    RuntimeEntrypointSpec {
        key: "stringsZhCn",
        path: "rust/strings.zh_cn.bin",
    },
    RuntimeEntrypointSpec {
        key: "uiTemplates",
        path: "rust/ui-pack/ui_templates.bin",
    },
    RuntimeEntrypointSpec {
        key: "uiBindings",
        path: "rust/ui-pack/ui_bindings.bin",
    },
    RuntimeEntrypointSpec {
        key: "uiStrings",
        path: "rust/ui-pack/ui_strings.bin",
    },
];

pub struct RuntimeReportPayloadInput<'a> {
    pub integrity: &'a BTreeMap<String, String>,
    pub sizes: &'a BTreeMap<String, u64>,
    pub missing: &'a [String],
    pub path_violations: &'a [String],
    pub runtime_id: &'a str,
    pub total_bytes: u64,
    pub compile_scope: &'a str,
    pub capabilities: &'a Value,
}

pub fn runtime_capabilities(scope: CompileScope) -> &'static [&'static str] {
    compile_scope_runtime_capabilities(scope)
}

fn scope_capability_catalog() -> Value {
    Value::Object(
        compile_scope_descriptors()
            .iter()
            .map(|descriptor| {
                (
                    descriptor.name().to_string(),
                    json!(descriptor.runtime_capabilities()),
                )
            })
            .collect(),
    )
}

pub fn runtime_manifest_paths_catalog() -> Value {
    validate_runtime_manifest_report_descriptors();
    let mut paths = Map::new();
    paths.insert(
        "runtimeManifest".to_string(),
        json!(RUST_RUNTIME_MANIFEST_PATH),
    );
    for descriptor in RUNTIME_MANIFEST_REPORT_DESCRIPTORS {
        paths.insert(descriptor.key.to_string(), json!(descriptor.path));
    }
    Value::Object(paths)
}

pub fn runtime_report_schemas_catalog() -> Value {
    validate_runtime_manifest_report_descriptors();
    let mut schemas = Map::new();
    for descriptor in RUNTIME_MANIFEST_REPORT_DESCRIPTORS {
        schemas.insert(descriptor.key.to_string(), json!(descriptor.schema_version));
    }
    Value::Object(schemas)
}

pub fn runtime_report_payload_policy_catalog() -> Value {
    validate_runtime_report_payload_policy();
    json!({
        "schemaHashInput": SCHEMA_HASH_RUNTIME_REPORT_PAYLOAD_POLICY_INPUT,
        "generatedAt": RUST_RUNTIME_GENERATED_AT,
        "integrityAlgorithm": RUST_RUNTIME_INTEGRITY_ALGORITHM,
        "nativeRuntime": {
            "status": NATIVE_RUNTIME_STATUS_READY,
            "authority": NATIVE_RUNTIME_AUTHORITY_RUST,
            "schemaVersion": NATIVE_RUNTIME_DIST_SCHEMA_VERSION
        },
        "distManifest": {
            "schemaVersion": DIST_MANIFEST_SCHEMA_VERSION,
            "source": DIST_MANIFEST_SOURCE
        },
        "runtimeManifestPathPolicy": runtime_manifest_path_policy(),
        "nativeRuntimePathPolicy": native_runtime_dist_path_policy(),
        "deploymentChecks": [
            "requiredArtifactsPresent",
            "pathPortable",
            "integrityHashesGenerated",
            "sizeReportGenerated",
            "capabilitiesGenerated"
        ]
    })
}

pub fn runtime_manifest_abi_catalog() -> Value {
    json!({
        "name": "neonei.runtime.manifest",
        "schemaVersion": RUST_RUNTIME_MANIFEST_SCHEMA_VERSION,
        "runtimeSchema": RUST_RUNTIME_SCHEMA,
        "schemaRevision": RUST_RUNTIME_SCHEMA_REVISION,
        "nativeRuntimeDistSchemaVersion": NATIVE_RUNTIME_DIST_SCHEMA_VERSION,
        "paths": runtime_manifest_paths_catalog(),
        "reportSchemas": runtime_report_schemas_catalog(),
        "entrypoints": RUST_RUNTIME_ENTRYPOINTS.iter().map(|spec| {
            json!({ "key": spec.key, "path": spec.path })
        }).collect::<Vec<_>>(),
        "capabilitiesByScope": scope_capability_catalog(),
        "pathPolicy": {
            "name": PATH_POLICY_PORTABLE_RELATIVE_ONLY,
            "portableRelativePathsOnly": true,
            "absolutePathsAllowed": false,
            "windowsPathsAllowed": false
        },
        "reportPayloadPolicy": runtime_report_payload_policy_catalog(),
        "schemaHashInput": SCHEMA_HASH_RUNTIME_MANIFEST_INPUT
    })
}

pub fn runtime_manifest_payload(
    input: &RuntimeReportPayloadInput<'_>,
    compiler_metadata: Value,
    files: Vec<Value>,
    entrypoints: Value,
) -> Value {
    validate_runtime_report_payload_policy();
    json!({
        "schema": RUST_RUNTIME_SCHEMA,
        "schemaVersion": RUST_RUNTIME_MANIFEST_SCHEMA_VERSION,
        "schemaRevision": RUST_RUNTIME_SCHEMA_REVISION,
        "runtimeId": input.runtime_id,
        "generatedAt": RUST_RUNTIME_GENERATED_AT,
        "compiler": compiler_metadata,
        "capabilities": input.capabilities,
        "files": files,
        "compileScope": input.compile_scope,
        "entrypoints": entrypoints,
        "pathPolicy": runtime_manifest_path_policy(),
    })
}

pub fn runtime_report_payload(
    descriptor: &RuntimeManifestReportDescriptor,
    input: &RuntimeReportPayloadInput<'_>,
) -> Value {
    validate_runtime_report_payload_policy();
    match descriptor.kind {
        RuntimeManifestReportKind::Integrity => json!({
            "schemaVersion": descriptor.schema_version,
            "algorithm": RUST_RUNTIME_INTEGRITY_ALGORITHM,
            "files": input.integrity,
        }),
        RuntimeManifestReportKind::Size => json!({
            "schemaVersion": descriptor.schema_version,
            "totalBytes": input.total_bytes,
            "files": input.sizes,
        }),
        RuntimeManifestReportKind::MissingData => json!({
            "schemaVersion": descriptor.schema_version,
            "missingFiles": input.missing,
        }),
        RuntimeManifestReportKind::MigrationReadiness => json!({
            "schemaVersion": descriptor.schema_version,
            "ready": input.missing.is_empty() && input.path_violations.is_empty(),
            "checks": {
                "requiredArtifactsPresent": input.missing.is_empty(),
                "pathPortable": input.path_violations.is_empty(),
                "integrityHashesGenerated": true,
                "sizeReportGenerated": true,
            },
            "pathViolations": input.path_violations,
        }),
        RuntimeManifestReportKind::Deployment => json!({
            "schemaVersion": descriptor.schema_version,
            "runtimeId": input.runtime_id,
            "generatedAt": RUST_RUNTIME_GENERATED_AT,
            "compileScope": input.compile_scope,
            "runtimeSize": {
                "totalBytes": input.total_bytes,
                "files": input.sizes,
            },
            "cache": {
                "immutableRuntimeFiles": input.integrity.len(),
                "estimatedRuntimeCacheBytes": input.total_bytes,
                "cacheKeyInputs": {
                    "runtimeId": input.runtime_id,
                    "integrityAlgorithm": RUST_RUNTIME_INTEGRITY_ALGORITHM,
                },
            },
            "missingData": {
                "missingFiles": input.missing,
                "missingFileCount": input.missing.len(),
            },
            "schema": {
                "runtime": RUST_RUNTIME_SCHEMA,
                "schemaRevision": RUST_RUNTIME_SCHEMA_REVISION,
                "capabilities": input.capabilities,
            },
            "deploymentChecks": {
                "requiredArtifactsPresent": input.missing.is_empty(),
                "pathPortable": input.path_violations.is_empty(),
                "integrityHashesGenerated": true,
                "sizeReportGenerated": true,
                "capabilitiesGenerated": true,
            },
            "pathViolations": input.path_violations,
        }),
    }
}

pub fn base_dist_manifest_payload(compiler_metadata: Value) -> Value {
    validate_runtime_report_payload_policy();
    json!({
        "schemaVersion": DIST_MANIFEST_SCHEMA_VERSION,
        "source": DIST_MANIFEST_SOURCE,
        "compiler": compiler_metadata,
        "files": {},
    })
}

pub fn native_runtime_dist_payload(input: &RuntimeReportPayloadInput<'_>) -> Value {
    validate_runtime_report_payload_policy();
    json!({
        "schemaVersion": NATIVE_RUNTIME_DIST_SCHEMA_VERSION,
        "runtimeId": input.runtime_id,
        "compileScope": input.compile_scope,
        "status": NATIVE_RUNTIME_STATUS_READY,
        "authority": NATIVE_RUNTIME_AUTHORITY_RUST,
        "runtimeManifest": RUST_RUNTIME_MANIFEST_PATH,
        "totalBytes": input.total_bytes,
        "files": input.sizes,
        "hashes": input.integrity,
        "pathPolicy": native_runtime_dist_path_policy(),
    })
}

pub fn runtime_manifest_path_policy() -> Value {
    json!({
        "portableRelativePathsOnly": true,
        "absolutePathsAllowed": false,
        "windowsPathsAllowed": false,
    })
}

pub fn native_runtime_dist_path_policy() -> Value {
    json!({
        "portableRelativePathsOnly": true,
        "absolutePathsAllowed": false,
    })
}

pub fn validate_runtime_manifest_report_descriptors() {
    if RUNTIME_MANIFEST_REPORT_DESCRIPTORS.is_empty() {
        panic!("runtime manifest report descriptor catalog must not be empty");
    }
    let mut keys = BTreeSet::new();
    let mut paths = BTreeSet::new();
    let mut schemas = BTreeSet::new();
    let mut kinds = BTreeSet::new();
    for descriptor in RUNTIME_MANIFEST_REPORT_DESCRIPTORS {
        require_non_empty("runtime manifest report descriptor key", descriptor.key);
        require_relative_json_path(descriptor.key, descriptor.path);
        require_non_empty(
            "runtime manifest report descriptor schema version",
            descriptor.schema_version,
        );
        if !keys.insert(descriptor.key) {
            panic!(
                "duplicate runtime manifest report descriptor key: {}",
                descriptor.key
            );
        }
        if !paths.insert(descriptor.path) {
            panic!(
                "duplicate runtime manifest report descriptor path: {}",
                descriptor.path
            );
        }
        if !schemas.insert(descriptor.schema_version) {
            panic!(
                "duplicate runtime manifest report descriptor schema version: {}",
                descriptor.schema_version
            );
        }
        if !kinds.insert(descriptor.kind) {
            panic!(
                "duplicate runtime manifest report descriptor kind: {:?}",
                descriptor.kind
            );
        }
    }
}

pub fn validate_runtime_report_payload_policy() {
    require_non_empty("runtime report generatedAt", RUST_RUNTIME_GENERATED_AT);
    require_non_empty(
        "runtime report integrity algorithm",
        RUST_RUNTIME_INTEGRITY_ALGORITHM,
    );
    require_non_empty("native runtime status", NATIVE_RUNTIME_STATUS_READY);
    require_non_empty("native runtime authority", NATIVE_RUNTIME_AUTHORITY_RUST);
    require_non_empty("dist manifest schema version", DIST_MANIFEST_SCHEMA_VERSION);
    require_non_empty("dist manifest source", DIST_MANIFEST_SOURCE);
    require_non_empty(
        "runtime report payload policy schema hash input",
        SCHEMA_HASH_RUNTIME_REPORT_PAYLOAD_POLICY_INPUT,
    );
}

fn require_relative_json_path(key: &str, path: &str) {
    require_non_empty("runtime manifest report descriptor path", path);
    if path.starts_with('/')
        || path.starts_with('\\')
        || path.contains('\\')
        || path.contains("..")
        || !path.ends_with(".json")
    {
        panic!("runtime manifest report descriptor path must be portable JSON: {key}");
    }
}

fn require_non_empty(label: &str, value: &str) {
    if value.trim().is_empty() {
        panic!("{label} must be non-empty");
    }
}
