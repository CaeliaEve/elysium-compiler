//! Runtime manifest ABI catalog.
//!
//! Runtime manifests are the consumer-facing contract for compiled packs. Keep
//! schema names, report paths, entrypoint names, scope capabilities, and path
//! policy in one module so producers, validators, and NeoNEI gates cannot drift.

use crate::cli::CompileScope;
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RuntimeManifestReportDescriptor {
    pub key: &'static str,
    pub path: &'static str,
    pub schema_version: &'static str,
}

impl RuntimeManifestReportDescriptor {
    const fn new(key: &'static str, path: &'static str, schema_version: &'static str) -> Self {
        Self {
            key,
            path,
            schema_version,
        }
    }
}

pub const RUNTIME_MANIFEST_REPORT_DESCRIPTORS: &[RuntimeManifestReportDescriptor] = &[
    RuntimeManifestReportDescriptor::new(
        "integrity",
        RUST_INTEGRITY_REPORT_PATH,
        RUST_INTEGRITY_SCHEMA_VERSION,
    ),
    RuntimeManifestReportDescriptor::new(
        "sizeReport",
        RUST_SIZE_REPORT_PATH,
        RUST_SIZE_REPORT_SCHEMA_VERSION,
    ),
    RuntimeManifestReportDescriptor::new(
        "missingDataReport",
        RUST_MISSING_DATA_REPORT_PATH,
        RUST_MISSING_DATA_REPORT_SCHEMA_VERSION,
    ),
    RuntimeManifestReportDescriptor::new(
        "migrationReadiness",
        RUST_MIGRATION_READINESS_REPORT_PATH,
        RUST_MIGRATION_READINESS_SCHEMA_VERSION,
    ),
    RuntimeManifestReportDescriptor::new(
        "deploymentReport",
        RUST_DEPLOYMENT_REPORT_PATH,
        RUST_DEPLOYMENT_REPORT_SCHEMA_VERSION,
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

const CAPABILITIES_ALL: &[&str] = &[
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
];

const CAPABILITIES_NATIVE_UI: &[&str] = &[
    "groups.collapse",
    "groups.semantic-nbt",
    "recipes.lookup",
    "recipes.native-ui-layout",
    "recipes.ui-pack",
    "search.zh-cn",
    "strings.zh-cn",
    "native-render.webgl2",
];

const CAPABILITIES_SEARCH: &[&str] = &["search.zh-cn", "strings.zh-cn"];

const CAPABILITIES_BROWSER: &[&str] = &[
    "groups.collapse",
    "groups.semantic-nbt",
    "search.zh-cn",
    "strings.zh-cn",
    "native-render.webgl2",
];

const CAPABILITIES_RECIPES: &[&str] = &["recipes.lookup", "recipes.native-ui-layout"];
const CAPABILITIES_UI: &[&str] = &["recipes.ui-pack", "native-render.webgl2"];
const CAPABILITIES_TEXTURES: &[&str] = &["atlas.static", "atlas.animated", "atlas.meta"];

pub fn runtime_capabilities(scope: CompileScope) -> &'static [&'static str] {
    match scope {
        CompileScope::All => CAPABILITIES_ALL,
        CompileScope::NativeUi => CAPABILITIES_NATIVE_UI,
        CompileScope::Search => CAPABILITIES_SEARCH,
        CompileScope::Browser => CAPABILITIES_BROWSER,
        CompileScope::Recipes => CAPABILITIES_RECIPES,
        CompileScope::Ui => CAPABILITIES_UI,
        CompileScope::Textures => CAPABILITIES_TEXTURES,
    }
}

fn scope_capability_catalog() -> Value {
    let scopes = [
        CompileScope::All,
        CompileScope::NativeUi,
        CompileScope::Search,
        CompileScope::Browser,
        CompileScope::Recipes,
        CompileScope::Ui,
        CompileScope::Textures,
    ];
    Value::Object(
        scopes
            .into_iter()
            .map(|scope| {
                (
                    scope.as_str().to_string(),
                    json!(runtime_capabilities(scope)),
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
        "schemaHashInput": SCHEMA_HASH_RUNTIME_MANIFEST_INPUT
    })
}

pub fn validate_runtime_manifest_report_descriptors() {
    if RUNTIME_MANIFEST_REPORT_DESCRIPTORS.is_empty() {
        panic!("runtime manifest report descriptor catalog must not be empty");
    }
    let mut keys = BTreeSet::new();
    let mut paths = BTreeSet::new();
    let mut schemas = BTreeSet::new();
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
    }
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
