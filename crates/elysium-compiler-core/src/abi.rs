use crate::raw_export_abi::RAW_EXPORT_ABI_VALIDATION_SCHEMA_VERSION;
use crate::ui_pack_abi::UI_PACK_ABI_VALIDATION_SCHEMA_VERSION;
use crate::version::{
    COMPILED_DIST_SCHEMA_VERSION, EXPORT_ABI_VERSION, PACK_ABI_VERSION, RAW_EXPORT_SCHEMA_VERSION,
    RUNTIME_ABI_VERSION,
};
use serde_json::{json, Value};

pub fn abi_catalog() -> Value {
    json!({
        "stabilityLevels": {
            "stable": "Consumers may depend on this ABI until a new version is declared.",
            "testing": "The ABI is visible for fixtures and integration work but may change before stabilization.",
            "obsolete": "The ABI or path is scheduled for removal after the replacement is authoritative.",
            "removed": "Historical record only; implementations must not depend on it."
        },
        "exportAbi": export_abi_catalog(),
        "packAbi": pack_abi_catalog(),
        "runtimeAbi": runtime_abi_catalog(),
        "controlAbi": control_abi_catalog(),
        "debugAbi": debug_abi_catalog(),
        "featureMatrix": feature_matrix_catalog(),
        "traceEvents": trace_event_catalog(),
        "policy": {
            "legacyFallback": "forbidden",
            "missingCapability": "fail-fast",
            "hotPathEncoding": "binary-pack-preferred",
            "fullExportValidation": "milestone-gate-only"
        }
    })
}

fn export_abi_catalog() -> Value {
    json!({
        "name": "elysium.export",
        "version": EXPORT_ABI_VERSION,
        "status": "testing",
        "legacyRawExportSchemaVersion": RAW_EXPORT_SCHEMA_VERSION,
        "root": "elysium.export.v2/",
        "requiredFiles": [
            "manifest.json",
            "capabilities.json",
            "modules.json",
            "drivers.json",
            "kconfig.json"
        ],
        "requiredDirectories": [
            "raw/",
            "native-ui/",
            "reports/"
        ],
        "nativeUi": {
            "requiredCapabilities": [
                "native_ui.surface",
                "native_ui.design_space_coordinates",
                "native_ui.background_asset"
            ],
            "requiredFiles": [
                "native-ui/families.jsonl.zst",
                "native-ui/surfaces.jsonl.zst",
                "native-ui/slots.bin"
            ],
            "coordinateSpace": "source-design-pixels",
            "runtimeTransform": "uniform-scale-to-fit-only",
            "fallbackPolicy": "missing required native UI capture is a validation error"
        },
        "fileContract": {
            "requiredMetadata": ["schemaVersion", "producer", "checksum", "rowCountOrSize"],
            "pathPolicy": "portable-relative-only; no drive letters, file URLs, absolute paths, '.', or '..' segments"
        },
        "validationReport": {
            "path": "rust/raw-export-abi-validation-report.json",
            "schemaVersion": RAW_EXPORT_ABI_VALIDATION_SCHEMA_VERSION,
            "policy": "missing required raw files, missing declared raw files, and path violations are compile blockers"
        }
    })
}

fn pack_abi_catalog() -> Value {
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
            "path": "rust/pack-validation-report.json",
            "schemaVersion": "elysium-compiler/pack-abi-validation/v1",
            "policy": "missing required runtime artifacts, path leaks, and legacy fallback are compile blockers"
        },
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
                "path": "rust/ui-pack-abi-validation-report.json",
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

fn runtime_abi_catalog() -> Value {
    json!({
        "name": "elysium.runtime",
        "version": RUNTIME_ABI_VERSION,
        "status": "testing",
        "snapshot": {
            "requiredFields": [
                "id",
                "exportAbiVersion",
                "packAbiVersion",
                "compilerVersion",
                "manifestPath",
                "checksum",
                "createdAt"
            ],
            "optionalFields": [
                "dbPath",
                "binaryPackPaths",
                "textureAtlasPaths",
                "nativeUiSurfaceIndex"
            ],
            "lifecycle": "RCU-style acquire/read/release with atomic snapshot promotion"
        }
    })
}

fn control_abi_catalog() -> Value {
    json!({
        "name": "elysium.control",
        "status": "stable-contract-target",
        "endpoints": [
            "/control/abi",
            "/control/capabilities",
            "/control/modules",
            "/control/drivers",
            "/control/snapshot/current",
            "/control/health",
            "/control/version"
        ],
        "policy": "Machine-readable and intended for consumers."
    })
}

fn debug_abi_catalog() -> Value {
    json!({
        "name": "elysium.debug",
        "status": "unstable-debug-only",
        "endpoints": [
            "/debug/trace/latest",
            "/debug/export/timing",
            "/debug/export/probe-failures",
            "/debug/compiler/intermediate",
            "/debug/runtime/readers"
        ],
        "policy": "Diagnostics only; not a stable consumer ABI."
    })
}

fn feature_matrix_catalog() -> Value {
    json!({
        "features": [
            { "name": "CONFIG_EXPORT_NEI", "provides": ["nei.handler.registry"] },
            { "name": "CONFIG_EXPORT_GT5U", "dependsOn": ["CONFIG_EXPORT_NEI"], "provides": ["gt5u.recipe_maps"] },
            { "name": "CONFIG_EXPORT_IC2", "dependsOn": ["CONFIG_EXPORT_NEI"], "provides": ["ic2.recipe_handlers"] },
            { "name": "CONFIG_EXPORT_NATIVE_UI_CAPTURE", "dependsOn": ["CONFIG_EXPORT_NEI"], "provides": ["native_ui.surface"] },
            { "name": "CONFIG_EXPORT_ANGELICA_FRAMEBUFFER", "provides": ["render.framebuffer.capture"] },
            { "name": "CONFIG_COMPILER_BINARY_PACK", "provides": ["compiler.binary_pack"] },
            { "name": "CONFIG_COMPILER_NATIVE_UI_PACK", "dependsOn": ["CONFIG_COMPILER_BINARY_PACK"], "provides": ["compiler.native_ui_pack"] },
            { "name": "CONFIG_COMPILER_WASM_LAYOUT_PACK", "dependsOn": ["CONFIG_COMPILER_NATIVE_UI_PACK"], "provides": ["compiler.wasm_layout_pack"] },
            { "name": "CONFIG_RUNTIME_RCU_SNAPSHOT", "provides": ["runtime.rcu_snapshot"] },
            { "name": "CONFIG_DEBUG_TRACEPOINTS", "provides": ["debug.tracepoints"] }
        ],
        "requiredFeatureFields": ["name", "dependsOn", "provides", "owner", "tests", "failureMode"]
    })
}

fn trace_event_catalog() -> Value {
    json!({
        "events": {
            "export.driver.probe.start": { "fields": ["driver_id", "device_id"] },
            "export.driver.probe.fail": { "fields": ["driver_id", "device_id", "reason"] },
            "export.native_ui.capture.surface": { "fields": ["family", "surface_id", "width", "height", "slots", "duration_ms"] },
            "compiler.pack.emit.done": { "fields": ["pack_id", "bytes", "rows", "duration_ms"] },
            "runtime.snapshot.promote": { "fields": ["from", "to", "readers", "duration_ms"] }
        }
    })
}
