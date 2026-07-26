use crate::atlas_repair::select_group_representative;
use crate::cli::{Cli, Command, CompileScope};
use crate::commands::run_command;
use crate::compiler_capability_abi::{
    COMPILER_COMMANDS, COMPILE_SCOPES, NATIVE_UI_COORDINATE_SPACE, NATIVE_UI_FALLBACK_POLICY,
    NATIVE_UI_REQUIRED_CAPABILITIES, NATIVE_UI_REQUIRED_FILES, NATIVE_UI_RUNTIME_TRANSFORM,
    REQUIRED_COMPILER_COMMANDS, SCHEMA_HASH_COMPILER_CAPABILITY_INPUT,
};
use crate::compiler_command_catalog::{
    command_name, compiler_command_catalog, compiler_command_descriptors,
    COMPILER_COMMAND_CATALOG_SCHEMA_VERSION,
};
use crate::compiler_scope_catalog::{
    compile_scope_catalog, compile_scope_descriptors, COMPILE_SCOPE_CATALOG_SCHEMA_VERSION,
};
use crate::diagnostics::{run_diagnostics_report_with_session, DiagnosticsMode};
use crate::io::{normalize_path, write_json_value};
use crate::json_ext::{value_string, value_u64};
use crate::kernel::{COMPILE_KERNEL_TRACE_REPORT_PATH, COMPILE_KERNEL_TRACE_SCHEMA_VERSION};
use crate::manifest::{
    manifest_collection_catalog, validate_manifest_collection_descriptors,
    COLLECTION_ANIMATION_FRAME_MATERIALIZATIONS, COLLECTION_BROWSER_GROUPS,
    COLLECTION_BROWSER_ITEMS, COLLECTION_FACADE_RESOLUTIONS, COLLECTION_HANDLER_LAYOUTS,
    COLLECTION_SEARCH_ITEMS, COLLECTION_TEXTURE_ROWS, COLLECTION_TEXTURE_ROWS_WITH_MANIFEST,
    MANIFEST_COLLECTION_CATALOG_SCHEMA_VERSION, MANIFEST_COLLECTION_DESCRIPTORS,
    SCHEMA_HASH_MANIFEST_COLLECTION_INPUT,
};
use crate::native_ui_export_abi;
use crate::native_ui_pack_abi::{
    UI_BINDING_MAGIC, UI_BINDING_PAYLOAD_VERSION, UI_BINDING_ROW_STRIDE_U32,
    UI_PRIMITIVE_ROW_STRIDE_U32, UI_RECT_ROW_STRIDE_U32, UI_SLOT_ROW_STRIDE_U32, UI_STRING_MAGIC,
    UI_TEMPLATE_MAGIC, UI_TEMPLATE_PAYLOAD_VERSION, UI_TEMPLATE_ROW_STRIDE_U32,
    UI_TEXT_ROW_STRIDE_U32,
};
use crate::native_ui_report;
use crate::pack_abi::{
    runtime_artifact_catalog, runtime_artifact_catalog_specs, runtime_pack_artifact_specs,
    runtime_summary_artifact_specs, validate_runtime_artifact_catalog_descriptors,
    validate_runtime_pack_abi, PACK_ABI_VALIDATION_REPORT_PATH, PACK_ABI_VALIDATION_SCHEMA_VERSION,
    RUNTIME_ARTIFACT_CATALOG_SCHEMA_VERSION, RUNTIME_PACK_PRODUCER_IDS,
    SCHEMA_HASH_RUNTIME_ARTIFACT_CATALOG_INPUT,
};
use crate::packs::browser::build_compact_group_payload_from_groups;
use crate::packs::recipe::build_compact_recipe_payload_from_pack;
use crate::packs::search::{
    build_compact_search_payload_from_items, build_compact_string_payload_from_items,
};
use crate::packs::texture::{
    build_compact_animation_payload_from_table, build_compact_atlas_meta_payload_from_atlas_items,
    build_compact_texture_payload_from_atlas_items, compile_texture_pack,
    copy_runtime_atlas_assets, normalize_runtime_atlas_file_path, normalize_timeline,
};
use crate::packs::ui::{
    build_compact_ui_binding_payload, build_compact_ui_string_payload,
    build_compact_ui_template_payload,
};
use crate::raw_export;
use crate::raw_export_abi;
use crate::raw_ui_schema_catalog::{
    RAW_UI_FAMILY_CENSUS_SCHEMA_VERSION, RAW_UI_TEMPLATE_CATALOG_SCHEMA_VERSION,
};
use crate::recipe_domain::{
    captured_ui_family_key, collect_recipe_item_ids, public_recipe_layout,
    should_skip_redundant_nei_workbench_recipe, RecipeHandlerContext,
};
use crate::recipe_ui_payload::rust_recipe_ui_payload_relative_path;
use crate::reports;
use crate::runtime;
use crate::runtime_manifest_abi::{
    runtime_capabilities, runtime_manifest_abi_catalog, runtime_manifest_path_policy,
    runtime_manifest_paths_catalog, runtime_report_payload_policy_catalog,
    runtime_report_schemas_catalog, validate_runtime_manifest_report_descriptors,
    validate_runtime_report_payload_policy, NATIVE_RUNTIME_AUTHORITY_RUST,
    NATIVE_RUNTIME_STATUS_READY, RUNTIME_MANIFEST_REPORT_DESCRIPTORS, RUST_RUNTIME_ENTRYPOINTS,
    RUST_RUNTIME_GENERATED_AT, RUST_RUNTIME_INTEGRITY_ALGORITHM,
    RUST_RUNTIME_MANIFEST_SCHEMA_VERSION, RUST_RUNTIME_SCHEMA, RUST_RUNTIME_SCHEMA_REVISION,
    SCHEMA_HASH_RUNTIME_MANIFEST_INPUT, SCHEMA_HASH_RUNTIME_REPORT_PAYLOAD_POLICY_INPUT,
};
use crate::runtime_pack_plan::{
    runtime_pack_compilers, runtime_pack_execution_batches, runtime_pack_execution_waves,
    runtime_pack_ownership_conflicts, CompilerThreadPool,
};
use crate::schema_catalog::{
    dist_data_schema_section, raw_export_optional_manifest_files,
    raw_export_required_manifest_files, raw_export_schema_section, runtime_entrypoint_paths,
    validate_schema_catalog_descriptors, DIST_DATA_SCHEMA_DESCRIPTORS,
    RAW_EXPORT_MANIFEST_FILE_DESCRIPTORS, SCHEMA_CATALOG_SCHEMA_VERSION,
    UI_PACK_SCHEMA_DESCRIPTORS,
};
use crate::schemas;
use crate::stages::{compile_kernel_catalog, compile_kernel_modules, run_compile_kernel};
use crate::texture_animation::{
    animation_materialization_diagnostic, expected_animated_item, expected_animation_reason,
    promote_animation_facts_to_animated_atlas_items,
};
use crate::ui_pack_abi;
use crate::ui_presentation_catalog::enrich_ui_templates_with_presentation;
use crate::ui_templates::{build_ui_assets_manifest, build_ui_template_bindings};
use crate::validation::{validate_atlas_bounds, validate_frame_bounds};
use serde_json::json;
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::fs;
use std::path::Path;

fn copy_test_tree(source: &Path, target: &Path) {
    fs::create_dir_all(target).unwrap();
    for entry in fs::read_dir(source).unwrap() {
        let entry = entry.unwrap();
        let source_path = entry.path();
        let target_path = target.join(entry.file_name());
        if entry.file_type().unwrap().is_dir() {
            copy_test_tree(&source_path, &target_path);
        } else {
            fs::copy(source_path, target_path).unwrap();
        }
    }
}

fn write_generation_pointer(authority: &Path, generation_id: &str, relative_path: &str) {
    fs::write(
        authority.join("current.json"),
        serde_json::to_vec_pretty(&json!({
            "schemaVersion": "nesqlpp/raw-export-generation-pointer/v1",
            "generationId": generation_id,
            "relativePath": relative_path,
        }))
        .unwrap(),
    )
    .unwrap();
}

fn compiler_fixture_path(name: &str) -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures")
        .join(name)
}

fn core_source_path(relative_path: &str) -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join(relative_path)
}

fn read_fixture_json(path: impl AsRef<Path>) -> serde_json::Value {
    serde_json::from_str(&fs::read_to_string(path).unwrap()).unwrap()
}

#[test]
fn raw_export_session_resolves_pointer_authority_and_pins_generation() {
    let authority = tempfile::tempdir().unwrap();
    let generations = authority.path().join("generations");
    let old_id = "old-generation-0001";
    let new_id = "new-generation-0002";
    let old_generation = generations.join(old_id);
    let new_generation = generations.join(new_id);
    copy_test_tree(
        &compiler_fixture_path("raw-export-minimal"),
        &old_generation,
    );
    copy_test_tree(
        &compiler_fixture_path("raw-export-minimal"),
        &new_generation,
    );

    for (path, name) in [(&old_generation, "old"), (&new_generation, "new")] {
        let manifest_path = path.join("manifest.json");
        let mut manifest = read_fixture_json(&manifest_path);
        manifest["repositoryName"] = json!(name);
        fs::write(manifest_path, serde_json::to_vec_pretty(&manifest).unwrap()).unwrap();
    }

    write_generation_pointer(authority.path(), old_id, &format!("generations/{old_id}"));
    let old_session = crate::session::RawExportSession::open(authority.path()).unwrap();
    assert_eq!(
        old_session.input_authority(),
        crate::session::RawExportInputAuthority::GenerationPointer
    );
    assert_eq!(
        old_session.input(),
        fs::canonicalize(&old_generation).unwrap()
    );
    assert_eq!(
        old_session.manifest().repository_name.as_deref(),
        Some("old")
    );

    write_generation_pointer(authority.path(), new_id, &format!("generations/{new_id}"));
    old_session.verify_input_generation().unwrap();
    assert_eq!(
        old_session.manifest().repository_name.as_deref(),
        Some("old")
    );

    let new_session = crate::session::RawExportSession::open(authority.path()).unwrap();
    assert_eq!(
        new_session.input(),
        fs::canonicalize(&new_generation).unwrap()
    );
    assert_eq!(
        new_session.manifest().repository_name.as_deref(),
        Some("new")
    );
}

#[test]
fn raw_export_session_keeps_legacy_direct_manifest_explicit_and_observable() {
    let input = compiler_fixture_path("raw-export-minimal");
    let session = crate::session::RawExportSession::open(&input).unwrap();
    assert_eq!(
        session.input_authority(),
        crate::session::RawExportInputAuthority::LegacyDirectManifest
    );
    assert_eq!(session.input_authority().as_str(), "legacy-direct-manifest");
    assert_eq!(session.authority_input(), fs::canonicalize(input).unwrap());
    assert_eq!(session.input(), session.authority_input());
}

#[test]
fn raw_export_pointer_failures_are_fail_closed() {
    for (name, pointer, expected) in [
        (
            "malformed",
            "{not-json".to_string(),
            "parse raw-export pointer",
        ),
        (
            "traversal",
            serde_json::to_string(&json!({
                "schemaVersion": "nesqlpp/raw-export-generation-pointer/v1",
                "generationId": "valid-generation-0001",
                "relativePath": "generations/../outside",
            }))
            .unwrap(),
            "relativePath must equal",
        ),
        (
            "missing",
            serde_json::to_string(&json!({
                "schemaVersion": "nesqlpp/raw-export-generation-pointer/v1",
                "generationId": "missing-generation-01",
                "relativePath": "generations/missing-generation-01",
            }))
            .unwrap(),
            "inspect raw-export current generation",
        ),
    ] {
        let authority = tempfile::tempdir().unwrap();
        fs::create_dir_all(authority.path().join("generations")).unwrap();
        fs::write(authority.path().join("current.json"), pointer).unwrap();
        let error = crate::session::RawExportSession::open(authority.path()).unwrap_err();
        assert!(
            format!("{error:#}").contains(expected),
            "{name} pointer error should contain {expected}: {error:#}"
        );
    }
}

#[test]
fn stable_pointer_root_completes_compile_and_diagnostics_handshake() {
    let authority = tempfile::tempdir().unwrap();
    let generation_id = "compile-generation-01";
    let generation = authority.path().join("generations").join(generation_id);
    copy_test_tree(&compiler_fixture_path("raw-export-minimal"), &generation);
    write_generation_pointer(
        authority.path(),
        generation_id,
        &format!("generations/{generation_id}"),
    );
    let output = tempfile::tempdir().unwrap();
    let report = output.path().join("compiler-report.json");
    let session = run_compile_kernel(
        authority.path(),
        output.path(),
        CompileScope::NativeUi,
        Some(1),
        true,
        false,
    )
    .unwrap();
    run_diagnostics_report_with_session(
        &session,
        Some(output.path()),
        &report,
        true,
        DiagnosticsMode::Compile,
    )
    .unwrap();
    let report = read_fixture_json(report);
    assert_eq!(report["inputAuthority"], json!("generation-pointer"));
    assert_eq!(
        report["input"],
        json!(normalize_path(&fs::canonicalize(authority.path()).unwrap()))
    );
    assert_eq!(
        report["resolvedInput"],
        json!(normalize_path(&fs::canonicalize(&generation).unwrap()))
    );
}

#[test]
fn all_compile_reuses_one_manifest_open_and_parse_through_diagnostics() {
    let input = compiler_fixture_path("raw-export-texture-atlas");
    let output = tempfile::tempdir().unwrap();
    let report = output.path().join("compiler-report.json");
    let session = run_compile_kernel(
        &input,
        output.path(),
        CompileScope::All,
        Some(1),
        true,
        false,
    )
    .unwrap();

    run_diagnostics_report_with_session(
        &session,
        Some(output.path()),
        &report,
        true,
        DiagnosticsMode::Compile,
    )
    .unwrap();

    let metrics = session.manifest_io_metrics();
    assert_eq!(metrics.load_open_count, 1);
    assert_eq!(metrics.load_read_count, 1);
    assert_eq!(metrics.parse_count, 1);
    assert_eq!(metrics.raw_export_abi_validation_count, 1);
    assert_eq!(metrics.native_ui_export_abi_validation_count, 1);
    assert_eq!(metrics.verification_open_count, 2);
    assert_eq!(metrics.verification_read_count, 2);
    assert_eq!(
        metrics.load_bytes_read,
        fs::metadata(input.join("manifest.json")).unwrap().len()
    );
}

#[test]
fn raw_export_session_fails_closed_when_declared_input_changes() {
    let input = tempfile::tempdir().unwrap();
    fs::write(
        input.path().join("manifest.json"),
        r#"{"schemaVersion":"fixture/v1","files":{"items":"items.jsonl"}}"#,
    )
    .unwrap();
    fs::write(input.path().join("items.jsonl"), "{\"id\":\"a\"}\n").unwrap();
    let session = crate::session::RawExportSession::open(input.path()).unwrap();

    fs::write(
        input.path().join("items.jsonl"),
        "{\"id\":\"a\"}\n{\"id\":\"b\"}\n",
    )
    .unwrap();

    let error = session.verify_input_generation().unwrap_err().to_string();
    assert!(error.contains("input generation changed"), "{error}");
}

#[test]
fn raw_export_session_detects_added_transitive_input_files() {
    let input = tempfile::tempdir().unwrap();
    fs::write(
        input.path().join("manifest.json"),
        r#"{"schemaVersion":"fixture/v1","files":{}}"#,
    )
    .unwrap();
    let session = crate::session::RawExportSession::open(input.path()).unwrap();
    fs::create_dir_all(input.path().join("recipes/shards")).unwrap();
    fs::write(input.path().join("recipes/shards/00.json"), "[]").unwrap();

    let error = session.verify_input_generation().unwrap_err().to_string();
    assert!(error.contains("input generation changed"), "{error}");
}

#[test]
fn failed_compile_cannot_leave_stale_runtime_authority_manifests() {
    let input = tempfile::tempdir().unwrap();
    fs::write(
        input.path().join("manifest.json"),
        r#"{"schemaVersion":"fixture/v1","files":{}}"#,
    )
    .unwrap();
    let output = tempfile::tempdir().unwrap();
    fs::create_dir_all(output.path().join("rust")).unwrap();
    fs::write(output.path().join("manifest.json"), "stale").unwrap();
    fs::write(output.path().join("rust/runtime-manifest.json"), "stale").unwrap();

    assert!(run_compile_kernel(
        input.path(),
        output.path(),
        CompileScope::All,
        Some(1),
        true,
        false,
    )
    .is_err());
    assert!(!output.path().join("manifest.json").exists());
    assert!(!output.path().join("rust/runtime-manifest.json").exists());
}

#[test]
fn pre_session_failure_cannot_leave_stale_runtime_authority_manifests() {
    let input = tempfile::tempdir().unwrap();
    let output = tempfile::tempdir().unwrap();
    fs::create_dir_all(output.path().join("rust")).unwrap();
    fs::write(output.path().join("manifest.json"), "stale").unwrap();
    fs::write(output.path().join("rust/runtime-manifest.json"), "stale").unwrap();

    assert!(run_compile_kernel(
        input.path(),
        output.path(),
        CompileScope::All,
        Some(1),
        true,
        false,
    )
    .is_err());
    assert!(!output.path().join("manifest.json").exists());
    assert!(!output.path().join("rust/runtime-manifest.json").exists());
}

#[test]
fn raw_export_session_detects_manifest_bytes_changed_with_same_length() {
    let input = tempfile::tempdir().unwrap();
    let manifest_path = input.path().join("manifest.json");
    let before = r#"{"schemaVersion":"fixture/a","files":{}}"#;
    let after = r#"{"schemaVersion":"fixture/b","files":{}}"#;
    assert_eq!(before.len(), after.len());
    fs::write(&manifest_path, before).unwrap();
    let session = crate::session::RawExportSession::open(input.path()).unwrap();
    fs::write(&manifest_path, after).unwrap();

    let error = session.verify_input_generation().unwrap_err().to_string();
    assert!(error.contains("input generation changed"), "{error}");
}

#[test]
fn raw_export_session_shares_json_collection_and_value_cache_by_resolved_path() {
    let input = tempfile::tempdir().unwrap();
    fs::write(
        input.path().join("manifest.json"),
        r#"{"schemaVersion":"fixture/v1","files":{"items":"items.json"}}"#,
    )
    .unwrap();
    fs::write(
        input.path().join("items.json"),
        r#"{"items":[{"itemId":"a"},{"itemId":"b"}]}"#,
    )
    .unwrap();
    let session = crate::session::RawExportSession::open(input.path()).unwrap();

    let browser_items = session
        .read_manifest_collection(COLLECTION_BROWSER_ITEMS)
        .unwrap();
    let search_items = session
        .read_manifest_collection(COLLECTION_SEARCH_ITEMS)
        .unwrap();
    let raw_json = session.read_manifest_json("items").unwrap().unwrap();

    assert_eq!(browser_items.len(), 2);
    assert!(std::sync::Arc::ptr_eq(&browser_items, &search_items));
    assert_eq!(raw_json["items"].as_array().unwrap().len(), 2);
    let metrics = session.manifest_io_metrics();
    assert_eq!(metrics.source_open_count, 1);
    assert_eq!(metrics.source_read_count, 1);
    assert_eq!(metrics.json_parse_count, 1);
    assert_eq!(metrics.jsonl_parse_count, 0);
    assert_eq!(metrics.cache_miss_count, 2);
    assert_eq!(metrics.cache_hit_count, 2);
}

#[test]
fn raw_export_session_shares_texture_rows_between_browser_and_texture_producers() {
    let input = tempfile::tempdir().unwrap();
    fs::write(
        input.path().join("manifest.json"),
        r#"{"schemaVersion":"fixture/v1","files":{"textures":"textures.jsonl"}}"#,
    )
    .unwrap();
    fs::write(
        input.path().join("textures.jsonl"),
        "{\"assetId\":\"a\"}\n{\"assetId\":\"b\"}\n",
    )
    .unwrap();
    let session = crate::session::RawExportSession::open(input.path()).unwrap();

    let browser_rows = session
        .read_manifest_collection(COLLECTION_TEXTURE_ROWS)
        .unwrap();
    let texture_rows = session
        .read_manifest_collection(COLLECTION_TEXTURE_ROWS_WITH_MANIFEST)
        .unwrap();

    assert_eq!(browser_rows.len(), 2);
    assert!(std::sync::Arc::ptr_eq(&browser_rows, &texture_rows));
    let metrics = session.manifest_io_metrics();
    assert_eq!(metrics.source_open_count, 1);
    assert_eq!(metrics.source_read_count, 1);
    assert_eq!(metrics.jsonl_parse_count, 1);
    assert_eq!(metrics.jsonl_row_parse_count, 2);
    assert_eq!(metrics.cache_miss_count, 1);
    assert_eq!(metrics.cache_hit_count, 1);
}

#[test]
fn raw_export_session_initializes_gzip_jsonl_cache_once_across_threads() {
    use flate2::write::GzEncoder;
    use flate2::Compression;
    use std::io::Write;
    use std::sync::{Arc, Barrier};

    let input = tempfile::tempdir().unwrap();
    fs::write(
        input.path().join("manifest.json"),
        r#"{"schemaVersion":"fixture/v1","files":{"items":"items.jsonl.gz"}}"#,
    )
    .unwrap();
    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    encoder
        .write_all(b"{\"itemId\":\"a\"}\n{\"itemId\":\"b\"}\n")
        .unwrap();
    fs::write(
        input.path().join("items.jsonl.gz"),
        encoder.finish().unwrap(),
    )
    .unwrap();
    let session = Arc::new(crate::session::RawExportSession::open(input.path()).unwrap());
    let barrier = Arc::new(Barrier::new(8));
    let threads = (0..8)
        .map(|_| {
            let session = session.clone();
            let barrier = barrier.clone();
            std::thread::spawn(move || {
                barrier.wait();
                session
                    .read_manifest_collection(COLLECTION_SEARCH_ITEMS)
                    .unwrap()
            })
        })
        .collect::<Vec<_>>();
    let rows = threads
        .into_iter()
        .map(|thread| thread.join().unwrap())
        .collect::<Vec<_>>();

    assert!(rows.iter().all(|rows| rows.len() == 2));
    assert!(rows.windows(2).all(|pair| Arc::ptr_eq(&pair[0], &pair[1])));
    let metrics = session.manifest_io_metrics();
    assert_eq!(metrics.source_open_count, 1);
    assert_eq!(metrics.source_read_count, 1);
    assert_eq!(metrics.source_decompression_count, 1);
    assert_eq!(metrics.jsonl_parse_count, 1);
    assert_eq!(metrics.jsonl_row_parse_count, 2);
    assert_eq!(metrics.cache_miss_count, 1);
    assert_eq!(metrics.cache_hit_count, 7);
}

#[test]
fn raw_export_session_can_release_large_source_caches_after_pack_compilation() {
    let input = tempfile::tempdir().unwrap();
    fs::write(
        input.path().join("manifest.json"),
        r#"{"schemaVersion":"fixture/v1","files":{"items":"items.jsonl"}}"#,
    )
    .unwrap();
    fs::write(
        input.path().join("items.jsonl"),
        "{\"itemId\":\"a\"}\n{\"itemId\":\"b\"}\n",
    )
    .unwrap();
    let session = crate::session::RawExportSession::open(input.path()).unwrap();

    assert_eq!(
        session
            .read_manifest_collection(COLLECTION_BROWSER_ITEMS)
            .unwrap()
            .len(),
        2
    );
    assert_eq!(session.manifest_io_metrics().source_open_count, 1);

    session.release_source_caches();

    assert_eq!(
        session
            .read_manifest_collection(COLLECTION_BROWSER_ITEMS)
            .unwrap()
            .len(),
        2
    );
    let metrics = session.manifest_io_metrics();
    assert_eq!(metrics.source_open_count, 2);
    assert_eq!(metrics.source_read_count, 2);
    assert_eq!(metrics.jsonl_parse_count, 2);
    assert_eq!(metrics.cache_miss_count, 2);
    assert_eq!(metrics.source_cache_release_count, 1);
}

#[test]
fn raw_export_session_can_stream_jsonl_without_retaining_a_cache_entry() {
    let input = tempfile::tempdir().unwrap();
    fs::write(
        input.path().join("manifest.json"),
        r#"{"schemaVersion":"fixture/v1","files":{}}"#,
    )
    .unwrap();
    fs::write(
        input.path().join("rows.jsonl"),
        "{\"itemId\":\"a\"}\n{\"itemId\":\"b\"}\n",
    )
    .unwrap();
    let session = crate::session::RawExportSession::open(input.path()).unwrap();

    for _ in 0..2 {
        let mut item_ids = Vec::new();
        let rows = session
            .visit_relative_jsonl("rows.jsonl", |row| {
                item_ids.push(row["itemId"].as_str().unwrap().to_string());
                Ok(())
            })
            .unwrap();
        assert_eq!(rows, 2);
        assert_eq!(item_ids, vec!["a", "b"]);
    }

    let metrics = session.manifest_io_metrics();
    assert_eq!(metrics.source_open_count, 2);
    assert_eq!(metrics.source_read_count, 2);
    assert_eq!(metrics.jsonl_parse_count, 2);
    assert_eq!(metrics.jsonl_row_parse_count, 4);
    assert_eq!(metrics.cache_miss_count, 2);
    assert_eq!(metrics.cache_hit_count, 0);
}

#[test]
fn raw_export_session_rejects_changed_file_before_serving_cached_collection() {
    let input = tempfile::tempdir().unwrap();
    fs::write(
        input.path().join("manifest.json"),
        r#"{"schemaVersion":"fixture/v1","files":{"items":"items.jsonl"}}"#,
    )
    .unwrap();
    let items_path = input.path().join("items.jsonl");
    fs::write(&items_path, "{\"itemId\":\"a\"}\n").unwrap();
    let session = crate::session::RawExportSession::open(input.path()).unwrap();
    assert_eq!(
        session
            .read_manifest_collection(COLLECTION_BROWSER_ITEMS)
            .unwrap()
            .len(),
        1
    );

    fs::write(&items_path, "{\"itemId\":\"a\"}\n{\"itemId\":\"b\"}\n").unwrap();
    let error = session
        .read_manifest_collection(COLLECTION_BROWSER_ITEMS)
        .unwrap_err()
        .to_string();

    assert!(error.contains("input file changed"), "{error}");
    assert_eq!(session.manifest_io_metrics().source_open_count, 1);
}

#[test]
fn browser_and_search_producers_share_items_and_groups_cache() {
    let input = tempfile::tempdir().unwrap();
    fs::write(
        input.path().join("manifest.json"),
        r#"{"schemaVersion":"fixture/v1","files":{"items":"items.jsonl","groups":"groups.jsonl"}}"#,
    )
    .unwrap();
    fs::write(
        input.path().join("items.jsonl"),
        r#"{"itemId":"mod:item","localizedName":"Item","modId":"mod","internalName":"item"}
"#,
    )
    .unwrap();
    fs::write(
        input.path().join("groups.jsonl"),
        r#"{"groupKey":"group","groupLabel":"Group","groupSize":1,"representativeItemId":"mod:item","memberItemIds":["mod:item"]}
"#,
    )
    .unwrap();
    let output = tempfile::tempdir().unwrap();
    let session = crate::session::RawExportSession::open(input.path()).unwrap();

    crate::packs::browser::compile_browser_pack(&session, output.path(), false, false).unwrap();
    crate::packs::search::compile_search_pack(&session, output.path(), false, false).unwrap();

    let metrics = session.manifest_io_metrics();
    assert_eq!(metrics.source_open_count, 2);
    assert_eq!(metrics.source_read_count, 2);
    assert_eq!(metrics.jsonl_parse_count, 2);
    assert_eq!(metrics.jsonl_row_parse_count, 2);
    assert!(metrics.cache_hit_count >= 3, "{metrics:?}");
    assert_eq!(
        session
            .read_manifest_collection(COLLECTION_BROWSER_GROUPS)
            .unwrap()
            .len(),
        1
    );
    assert_eq!(session.manifest_io_metrics().source_open_count, 2);
}

#[test]
fn browser_pack_merges_semantic_groups_and_preserves_source_order_fallback() {
    let input = tempfile::tempdir().unwrap();
    fs::write(
        input.path().join("manifest.json"),
        r#"{"schemaVersion":"fixture/v1","files":{"items":"items.jsonl","groups":"groups.jsonl","neiOrder":"order.jsonl","semanticItems":"semantic-items.jsonl","itemIdentityMap":"identity-map.jsonl"}}"#,
    )
    .unwrap();
    fs::write(
        input.path().join("items.jsonl"),
        concat!(
            "{\"itemId\":\"a\",\"localizedName\":\"A\",\"modId\":\"mod\",\"internalName\":\"a\"}\n",
            "{\"itemId\":\"b\",\"localizedName\":\"B\",\"modId\":\"mod\",\"internalName\":\"b\"}\n",
            "{\"itemId\":\"c\",\"localizedName\":\"\\u9519\\u4f4d\\u5bb9\\u5668\",\"modId\":\"mod\",\"internalName\":\"c\",\"searchTerms\":\"tooltip \\u901a\\u5173Minecraft should not be runtime search authority\"}\n",
            "{\"itemId\":\"d\",\"localizedName\":\"D\",\"modId\":\"mod\",\"internalName\":\"d\"}\n",
        ),
    )
    .unwrap();
    fs::write(
        input.path().join("groups.jsonl"),
        "{\"groupKey\":\"nei:0\",\"groupLabel\":\"Native\",\"groupSize\":2,\"groupSortOrder\":0,\"representativeItemId\":\"a\",\"memberItemIds\":[\"a\",\"b\"],\"groupSource\":\"nativeNei\"}\n",
    )
    .unwrap();
    fs::write(
        input.path().join("order.jsonl"),
        concat!(
            "{\"entryOrder\":0,\"entryKind\":\"group-collapsed\",\"itemId\":\"a\",\"groupKey\":\"nei:0\"}\n",
            "{\"entryOrder\":1,\"entryKind\":\"item\",\"itemId\":\"c\"}\n",
            "{\"entryOrder\":2,\"entryKind\":\"item\",\"itemId\":\"d\"}\n",
        ),
    )
    .unwrap();
    fs::write(
        input.path().join("semantic-items.jsonl"),
        "{\"publicItemId\":\"semantic:tool.gregtech:mod~tool\",\"representativeLegacyItemId\":\"c\",\"localizedName\":\"GregTech Tools\",\"family\":\"tool.gregtech\",\"classification\":\"classified\"}\n",
    )
    .unwrap();
    fs::write(
        input.path().join("identity-map.jsonl"),
        concat!(
            "{\"legacyItemId\":\"c\",\"publicItemId\":\"semantic:tool.gregtech:mod~tool\",\"family\":\"tool.gregtech\",\"classification\":\"classified\",\"facetSummary\":\"material=iron\"}\n",
            "{\"legacyItemId\":\"d\",\"publicItemId\":\"semantic:tool.gregtech:mod~tool\",\"family\":\"tool.gregtech\",\"classification\":\"classified\",\"facetSummary\":\"material=steel\"}\n",
        ),
    )
    .unwrap();

    let output = tempfile::tempdir().unwrap();
    let session = crate::session::RawExportSession::open(input.path()).unwrap();
    crate::packs::browser::compile_browser_pack(&session, output.path(), true, true).unwrap();

    let browser = read_fixture_json(output.path().join("rust/browser-pack.json"));
    let items = browser["items"].as_array().unwrap();
    let item = |item_id: &str| {
        items
            .iter()
            .find(|item| item["itemId"] == json!(item_id))
            .unwrap()
    };
    assert_eq!(item("b")["browserOrder"], json!(1));
    assert_eq!(
        item("c")["groupKey"],
        json!("semantic:tool.gregtech:mod~tool")
    );
    assert_eq!(
        item("d")["groupKey"],
        json!("semantic:tool.gregtech:mod~tool")
    );
    assert!(browser["groups"]
        .as_array()
        .unwrap()
        .iter()
        .any(|group| group["groupKey"] == json!("semantic:tool.gregtech:mod~tool")));

    let search = read_fixture_json(output.path().join("rust/search-pack.json"));
    let search_c = search["items"]
        .as_array()
        .unwrap()
        .iter()
        .find(|item| item["itemId"] == json!("c"))
        .unwrap();
    assert_eq!(search_c["searchRank"], json!(1));
    assert!(search_c["normalizedSearchTerms"]
        .as_str()
        .unwrap()
        .contains("gregtech tool"));
    assert!(!search_c["normalizedSearchTerms"]
        .as_str()
        .unwrap()
        .contains("minecraft"));
    assert_eq!(search_c["pinyinFull"], json!("cuoweirongqi"));
    assert!(!search_c["pinyinFull"].as_str().unwrap().starts_with("iron"));
}

#[test]
fn recipe_category_ids_preserve_unicode_without_collisions() {
    let steam = crate::recipe_domain::recipe_category_id_from_display_name(
        "GregTech - 大型蒸汽涡轮 ULV",
        "gt.recipe.steam-turbine",
    );
    let gas = crate::recipe_domain::recipe_category_id_from_display_name(
        "GregTech - 大型燃气涡轮 ULV",
        "gt.recipe.gas-turbine",
    );

    assert_ne!(steam, gas);
    assert!(steam.starts_with("display~gregtech%20-%20"), "{steam}");
    assert!(gas.starts_with("display~gregtech%20-%20"), "{gas}");
    assert!(steam.contains('%'), "{steam}");
    assert!(gas.contains('%'), "{gas}");
}

fn compile_fixture(fixture: &str, scope: CompileScope, strict: bool) -> tempfile::TempDir {
    let output = tempfile::tempdir().unwrap();
    let report = output.path().join("compiler-report.json");
    let session = run_compile_kernel(
        &compiler_fixture_path(fixture),
        output.path(),
        scope,
        Some(1),
        strict,
        false,
    )
    .unwrap();
    run_diagnostics_report_with_session(
        &session,
        Some(output.path()),
        &report,
        strict,
        DiagnosticsMode::Compile,
    )
    .unwrap();
    output
}

#[test]
fn ui_pack_requires_compiled_recipe_payload_index() {
    let input = compiler_fixture_path("raw-export-minimal");
    let output = tempfile::tempdir().unwrap();
    let session = crate::session::RawExportSession::open(&input).unwrap();

    let error = crate::packs::ui::compile_ui_pack(&session, output.path(), false, false)
        .unwrap_err()
        .to_string();
    assert!(
        error.contains("compiled recipe UI payload index is missing"),
        "{error}"
    );

    crate::packs::recipe::compile_recipe_pack(&session, output.path(), false, false).unwrap();
    crate::packs::ui::compile_ui_pack(&session, output.path(), false, false).unwrap();
    assert!(output.path().join("recipes/ui-payload-index.json").exists());
    assert!(output.path().join("rust/ui-pack/ui_templates.bin").exists());
}

enum CensusSchemaCase<'a> {
    Undeclared,
    MissingFile,
    Document(Option<&'a str>),
}

fn compile_ui_schema_case(
    template_schema: Option<&str>,
    embedded_census_schema: Option<&str>,
    census_schema: CensusSchemaCase<'_>,
) -> anyhow::Result<()> {
    let input = tempfile::tempdir().unwrap();
    copy_test_tree(&compiler_fixture_path("raw-export-minimal"), input.path());

    let template_catalog_path = input.path().join("recipes/ui-template-catalog.json");
    let mut template_catalog = read_fixture_json(&template_catalog_path);
    let template_catalog_object = template_catalog.as_object_mut().unwrap();
    match template_schema {
        Some(schema) => {
            template_catalog_object.insert("schemaVersion".to_string(), json!(schema));
        }
        None => {
            template_catalog_object.remove("schemaVersion");
        }
    }
    match embedded_census_schema {
        Some(schema) => {
            template_catalog_object
                .insert("source".to_string(), json!({"censusSchemaVersion": schema}));
        }
        None => {
            template_catalog_object.remove("source");
        }
    }
    fs::write(
        &template_catalog_path,
        serde_json::to_vec_pretty(&template_catalog).unwrap(),
    )
    .unwrap();

    if !matches!(&census_schema, CensusSchemaCase::Undeclared) {
        let manifest_path = input.path().join("manifest.json");
        let mut manifest = read_fixture_json(&manifest_path);
        manifest["files"]["uiFamilyCensus"] = json!("validation/ui-family-census.json");
        fs::write(manifest_path, serde_json::to_vec_pretty(&manifest).unwrap()).unwrap();
    }

    if let CensusSchemaCase::Document(census_schema) = &census_schema {
        let census_path = input.path().join("validation/ui-family-census.json");
        fs::create_dir_all(census_path.parent().unwrap()).unwrap();
        let census = match census_schema {
            Some(schema) => json!({"schemaVersion": schema}),
            None => json!({}),
        };
        fs::write(&census_path, serde_json::to_vec_pretty(&census).unwrap()).unwrap();
    }

    let output = tempfile::tempdir().unwrap();
    let session = crate::session::RawExportSession::open(input.path()).unwrap();
    crate::packs::recipe::compile_recipe_pack(&session, output.path(), false, false)?;
    crate::packs::ui::compile_ui_pack(&session, output.path(), false, false)
}

#[test]
fn ui_pack_accepts_exact_raw_ui_schema_v2() {
    compile_ui_schema_case(
        Some(RAW_UI_TEMPLATE_CATALOG_SCHEMA_VERSION),
        Some(RAW_UI_FAMILY_CENSUS_SCHEMA_VERSION),
        CensusSchemaCase::Document(Some(RAW_UI_FAMILY_CENSUS_SCHEMA_VERSION)),
    )
    .unwrap();
}

#[test]
fn ui_pack_rejects_missing_raw_ui_schema_version() {
    let error = compile_ui_schema_case(
        None,
        Some(RAW_UI_FAMILY_CENSUS_SCHEMA_VERSION),
        CensusSchemaCase::Undeclared,
    )
    .unwrap_err()
    .to_string();
    assert!(
        error.contains("uiTemplateCatalog schemaVersion must be")
            && error.contains("but was <missing>"),
        "{error}"
    );
}

#[test]
fn ui_pack_rejects_raw_ui_schema_v1() {
    let error = compile_ui_schema_case(
        Some("nesqlpp/raw-export/alpha1/ui-template-catalog/v1"),
        Some(RAW_UI_FAMILY_CENSUS_SCHEMA_VERSION),
        CensusSchemaCase::Undeclared,
    )
    .unwrap_err()
    .to_string();
    assert!(
        error.contains(RAW_UI_TEMPLATE_CATALOG_SCHEMA_VERSION)
            && error.contains("ui-template-catalog/v1"),
        "{error}"
    );
}

#[test]
fn ui_pack_rejects_wrong_raw_ui_schema() {
    let error = compile_ui_schema_case(
        Some("wrong/ui-template-schema"),
        Some(RAW_UI_FAMILY_CENSUS_SCHEMA_VERSION),
        CensusSchemaCase::Undeclared,
    )
    .unwrap_err()
    .to_string();
    assert!(
        error.contains(RAW_UI_TEMPLATE_CATALOG_SCHEMA_VERSION)
            && error.contains("wrong/ui-template-schema"),
        "{error}"
    );
}

#[test]
fn ui_pack_rejects_wrong_manifest_declared_census_schema() {
    let error = compile_ui_schema_case(
        Some(RAW_UI_TEMPLATE_CATALOG_SCHEMA_VERSION),
        Some(RAW_UI_FAMILY_CENSUS_SCHEMA_VERSION),
        CensusSchemaCase::Document(Some("nesqlpp/raw-export/alpha1/ui-family-census/v1")),
    )
    .unwrap_err()
    .to_string();
    assert!(
        error.contains("uiFamilyCensus schemaVersion must be")
            && error.contains(RAW_UI_FAMILY_CENSUS_SCHEMA_VERSION)
            && error.contains("ui-family-census/v1"),
        "{error}"
    );
}

#[test]
fn ui_pack_rejects_missing_embedded_census_schema() {
    let error = compile_ui_schema_case(
        Some(RAW_UI_TEMPLATE_CATALOG_SCHEMA_VERSION),
        None,
        CensusSchemaCase::Undeclared,
    )
    .unwrap_err()
    .to_string();
    assert!(
        error.contains("source.censusSchemaVersion must be") && error.contains("but was <missing>"),
        "{error}"
    );
}

#[test]
fn ui_pack_rejects_embedded_census_schema_v1() {
    let error = compile_ui_schema_case(
        Some(RAW_UI_TEMPLATE_CATALOG_SCHEMA_VERSION),
        Some("nesqlpp/raw-export/alpha1/ui-family-census/v1"),
        CensusSchemaCase::Undeclared,
    )
    .unwrap_err()
    .to_string();
    assert!(
        error.contains(RAW_UI_FAMILY_CENSUS_SCHEMA_VERSION)
            && error.contains("ui-family-census/v1"),
        "{error}"
    );
}

#[test]
fn ui_pack_rejects_wrong_embedded_census_schema() {
    let error = compile_ui_schema_case(
        Some(RAW_UI_TEMPLATE_CATALOG_SCHEMA_VERSION),
        Some("wrong/ui-family-census-schema"),
        CensusSchemaCase::Undeclared,
    )
    .unwrap_err()
    .to_string();
    assert!(
        error.contains(RAW_UI_FAMILY_CENSUS_SCHEMA_VERSION)
            && error.contains("wrong/ui-family-census-schema"),
        "{error}"
    );
}

#[test]
fn ui_pack_rejects_manifest_declared_missing_census_file() {
    let error = compile_ui_schema_case(
        Some(RAW_UI_TEMPLATE_CATALOG_SCHEMA_VERSION),
        Some(RAW_UI_FAMILY_CENSUS_SCHEMA_VERSION),
        CensusSchemaCase::MissingFile,
    )
    .unwrap_err()
    .to_string();
    assert!(
        error.contains("manifest-declared uiFamilyCensus is unreadable or missing"),
        "{error}"
    );
}

#[test]
fn ui_pack_rejects_manifest_declared_census_missing_schema_version() {
    let error = compile_ui_schema_case(
        Some(RAW_UI_TEMPLATE_CATALOG_SCHEMA_VERSION),
        Some(RAW_UI_FAMILY_CENSUS_SCHEMA_VERSION),
        CensusSchemaCase::Document(None),
    )
    .unwrap_err()
    .to_string();
    assert!(
        error.contains("uiFamilyCensus schemaVersion must be")
            && error.contains("but was <missing>"),
        "{error}"
    );
}

#[test]
fn ui_pack_rejects_manifest_declared_census_wrong_schema() {
    let error = compile_ui_schema_case(
        Some(RAW_UI_TEMPLATE_CATALOG_SCHEMA_VERSION),
        Some(RAW_UI_FAMILY_CENSUS_SCHEMA_VERSION),
        CensusSchemaCase::Document(Some("wrong/ui-family-census-schema")),
    )
    .unwrap_err()
    .to_string();
    assert!(
        error.contains(RAW_UI_FAMILY_CENSUS_SCHEMA_VERSION)
            && error.contains("wrong/ui-family-census-schema"),
        "{error}"
    );
}

fn assert_expected_json_matches(fixture: &str, output: &Path, relative_path: &str) {
    let expected = compiler_fixture_path("expected")
        .join(fixture)
        .join(relative_path);
    let actual = output.join(relative_path);
    assert_eq!(
        read_fixture_json(actual),
        read_fixture_json(expected),
        "expected output mismatch for {fixture}:{relative_path}"
    );
}

#[test]
fn normalize_path_uses_forward_slashes() {
    assert!(normalize_path(Path::new("a/b")).contains('/'));
}

#[test]
fn empty_runtime_summary_without_output() {
    let summary = reports::summarize_runtime_output(None).unwrap();
    assert!(summary.counts.is_empty());
    assert!(summary.sizes.is_empty());
}

#[test]
fn compiler_command_catalog_is_cli_dispatch_authority() {
    let catalog = compiler_command_catalog();
    let descriptors = compiler_command_descriptors();
    let descriptor_names = descriptors
        .iter()
        .map(|descriptor| descriptor.name())
        .collect::<Vec<_>>();
    let required_names = REQUIRED_COMPILER_COMMANDS
        .iter()
        .copied()
        .filter(|name| {
            descriptors
                .iter()
                .any(|descriptor| descriptor.name() == *name && descriptor.required())
        })
        .collect::<Vec<_>>();

    assert_eq!(
        catalog["schemaVersion"],
        json!(COMPILER_COMMAND_CATALOG_SCHEMA_VERSION)
    );
    assert_eq!(
        catalog["dispatchPolicy"],
        json!("static-command-descriptor-ops-table")
    );
    assert_eq!(catalog["legacyFallback"], json!("forbidden"));
    assert_eq!(descriptor_names, COMPILER_COMMANDS);
    assert_eq!(required_names, REQUIRED_COMPILER_COMMANDS);
    assert!(descriptors.iter().any(|descriptor| {
        descriptor.name() == "compile"
            && descriptor
                .capabilities()
                .contains(&"compiler.compile_kernel")
            && descriptor.outputs().contains(&"runtime-packs")
    }));
    assert!(descriptors.iter().any(|descriptor| {
        descriptor.name() == "schemas"
            && descriptor.required()
            && descriptor
                .capabilities()
                .contains(&"compiler.schema_catalog")
    }));

    let command = Command::Compile {
        input: compiler_fixture_path("raw-export-minimal"),
        output: tempfile::tempdir().unwrap().path().to_path_buf(),
        report: tempfile::tempdir()
            .unwrap()
            .path()
            .join("compiler-report.json"),
        scope: CompileScope::NativeUi,
        threads: Some(1),
        strict: false,
        debug_json: false,
    };
    assert_eq!(command_name(&command), "compile");

    let commands_rs = fs::read_to_string(core_source_path("commands.rs")).unwrap();
    assert!(commands_rs.contains("run_compiler_command(&cli.command)"));
    assert!(!commands_rs.contains("match cli.command"));
    assert!(!commands_rs.contains("Command::Compile"));
}

#[test]
fn compile_scope_catalog_is_runtime_pack_authority() {
    let catalog = compile_scope_catalog();
    let descriptors = compile_scope_descriptors();
    let descriptor_names = descriptors
        .iter()
        .map(|descriptor| descriptor.name())
        .collect::<Vec<_>>();

    assert_eq!(
        catalog["schemaVersion"],
        json!(COMPILE_SCOPE_CATALOG_SCHEMA_VERSION)
    );
    assert_eq!(
        catalog["selectionPolicy"],
        json!("static-compile-scope-descriptor-table")
    );
    assert_eq!(catalog["legacyFallback"], json!("forbidden"));
    assert_eq!(descriptor_names, COMPILE_SCOPES);

    for descriptor in descriptors {
        assert_eq!(
            descriptor.scope().as_str(),
            descriptor.name(),
            "CompileScope::as_str must be catalog-owned for {}",
            descriptor.name()
        );
        assert_eq!(
            runtime_capabilities(descriptor.scope()),
            descriptor.runtime_capabilities(),
            "runtime capabilities drifted for {}",
            descriptor.name()
        );
        let active_producers = runtime_pack_compilers(descriptor.scope())
            .map(|compiler| compiler.id())
            .collect::<Vec<_>>();
        assert_eq!(
            active_producers,
            descriptor.runtime_pack_producers(),
            "runtime pack producer selection drifted for {}",
            descriptor.name()
        );
    }

    let native_ui = descriptors
        .iter()
        .find(|descriptor| descriptor.scope() == CompileScope::NativeUi)
        .unwrap();
    assert_eq!(
        native_ui.runtime_pack_producers(),
        &["browser", "recipes", "ui"]
    );
    assert!(native_ui
        .runtime_capabilities()
        .contains(&"recipes.ui-pack"));
    assert!(!native_ui.runtime_pack_producers().contains(&"texture"));
}

#[test]
fn compact_group_pack_uses_native_binary_payload() {
    let groups = vec![json!({
        "groupKey": "thaumcraft:wands",
        "groupLabel": "??",
        "groupSize": 2,
        "representativeItemId": "i~thaumcraft~wand~0",
        "memberItemIds": ["i~thaumcraft~wand~0", "i~thaumcraft~wand~1"]
    })];
    let payload = build_compact_group_payload_from_groups(&groups).unwrap();
    assert_eq!(&payload[0..8], b"NEIGRP1\0");
    assert_eq!(u32::from_le_bytes(payload[8..12].try_into().unwrap()), 1);
    assert_eq!(u32::from_le_bytes(payload[12..16].try_into().unwrap()), 1);
    assert_eq!(u32::from_le_bytes(payload[20..24].try_into().unwrap()), 2);
    assert_eq!(u32::from_le_bytes(payload[24..28].try_into().unwrap()), 6);
}

#[test]
fn compact_animation_pack_uses_native_binary_payload() {
    let animations = vec![json!({
        "itemId": "i~botania~manaResource~4",
        "atlasFile": "textures/atlas/animated-main.webp",
        "frameDurationMs": 50,
        "timeline": [{ "frameIndex": 0, "durationMs": 50 }, { "frameIndex": 1, "durationMs": 75 }]
    })];
    let payload = build_compact_animation_payload_from_table(&animations).unwrap();
    assert_eq!(&payload[0..8], b"NEIANM1\0");
    assert_eq!(u32::from_le_bytes(payload[8..12].try_into().unwrap()), 1);
    assert_eq!(u32::from_le_bytes(payload[12..16].try_into().unwrap()), 1);
    assert_eq!(u32::from_le_bytes(payload[20..24].try_into().unwrap()), 2);
    assert_eq!(u32::from_le_bytes(payload[24..28].try_into().unwrap()), 5);
}

#[test]
fn compact_texture_pack_uses_native_binary_payload() {
    let items = vec![json!({
        "itemId": "minecraft:iron_ingot",
        "staticAtlas": { "atlasFile": "textures/atlas/static-main.webp", "x": 1, "y": 2, "width": 16, "height": 16 },
        "animatedAtlas": {
            "atlasFile": "textures/atlas/animated-main.webp",
            "frameDurationMs": 50,
            "frames": [{ "x": 3, "y": 4, "width": 16, "height": 16 }],
            "timeline": [{ "frameIndex": 0, "durationMs": 50 }]
        }
    })];
    let payload = build_compact_texture_payload_from_atlas_items(&items).unwrap();
    assert_eq!(&payload[0..8], b"NEITEX1\0");
    assert_eq!(u32::from_le_bytes(payload[8..12].try_into().unwrap()), 1);
    assert_eq!(u32::from_le_bytes(payload[12..16].try_into().unwrap()), 1);
    assert_eq!(u32::from_le_bytes(payload[20..24].try_into().unwrap()), 1);
    assert_eq!(u32::from_le_bytes(payload[24..28].try_into().unwrap()), 10);
}

#[test]
fn runtime_atlas_assets_copy_each_declared_source_once() {
    let input = tempfile::tempdir().unwrap();
    let output = tempfile::tempdir().unwrap();
    let source_dir = input.path().join("assets/textures");
    fs::create_dir_all(&source_dir).unwrap();
    fs::write(source_dir.join("a.bin"), b"atlas-a").unwrap();
    fs::write(source_dir.join("b.bin"), b"atlas-b").unwrap();
    let atlas_items = vec![
        json!({ "staticAtlas": { "atlasFile": "assets/textures/a.bin" } }),
        json!({ "animatedAtlas": { "atlasFile": "assets/textures/b.bin" } }),
    ];
    let mut missing = Vec::new();

    copy_runtime_atlas_assets(input.path(), output.path(), &atlas_items, &mut missing).unwrap();

    assert!(missing.is_empty());
    assert_eq!(
        fs::read(output.path().join("textures/a.bin")).unwrap(),
        b"atlas-a"
    );
    assert_eq!(
        fs::read(output.path().join("textures/b.bin")).unwrap(),
        b"atlas-b"
    );
}

#[test]
fn group_representative_prefers_drawable_variant_over_tiny_native_sprite() {
    let atlas_items = BTreeMap::from([
        (
            "i~ae2fc~wireless_fluid_terminal~0".to_string(),
            json!({
            "itemId": "i~ae2fc~wireless_fluid_terminal~0",
            "resolutionMode": "native_sprite",
            "staticAtlas": { "atlasFile": "textures/atlas/static.png", "x": 0, "y": 0, "width": 16, "height": 16 }
            }),
        ),
        (
            "i~ae2fc~wireless_fluid_terminal~0~charged".to_string(),
            json!({
            "itemId": "i~ae2fc~wireless_fluid_terminal~0~charged",
            "resolutionMode": "native_sprite",
            "staticAtlas": { "atlasFile": "textures/atlas/static.png", "x": 16, "y": 0, "width": 64, "height": 64 }
            }),
        ),
    ]);
    let atlas_by_item = atlas_items
        .iter()
        .map(|(item_id, value)| (item_id.clone(), value))
        .collect();
    let selected = select_group_representative(
        Some("i~ae2fc~wireless_fluid_terminal~0".to_string()),
        &[
            "i~ae2fc~wireless_fluid_terminal~0".to_string(),
            "i~ae2fc~wireless_fluid_terminal~0~charged".to_string(),
        ],
        &atlas_by_item,
    );
    assert_eq!(
        selected.as_deref(),
        Some("i~ae2fc~wireless_fluid_terminal~0~charged")
    );
}

#[test]
fn runtime_atlas_paths_are_dist_data_relative() {
    assert_eq!(
        normalize_runtime_atlas_file_path(Some(
            "assets/textures/atlas-assets/atlases/item-native-static.png".to_string()
        ))
        .as_deref(),
        Some("textures/atlas-assets/atlases/item-native-static.png")
    );
    assert_eq!(
        normalize_runtime_atlas_file_path(Some(
            "textures/atlas-assets/atlases/item-native-static.png".to_string()
        ))
        .as_deref(),
        Some("textures/atlas-assets/atlases/item-native-static.png")
    );
}

#[test]
fn compact_atlas_meta_pack_summarizes_atlas_files() {
    let items = vec![
        json!({
            "itemId": "minecraft:iron_ingot",
            "staticAtlas": { "atlasFile": "textures/atlas/static-main.webp", "atlasWidth": 2048, "atlasHeight": 2048, "x": 1, "y": 2, "width": 16, "height": 16 },
        }),
        json!({
            "itemId": "i~AWWayofTime~lifeEssence~0",
            "animatedAtlas": {
                "atlasFile": "textures/atlas/animated-main.webp",
                "atlasWidth": { "value": "2048" },
                "atlasHeight": { "value": "4096" },
                "frames": [
                    { "x": { "value": "0" }, "y": { "value": "0" }, "width": { "value": "16" }, "height": { "value": "16" } },
                    { "x": { "value": "16" }, "y": { "value": "0" }, "width": { "value": "16" }, "height": { "value": "16" } }
                ]
            }
        }),
    ];
    let payload = build_compact_atlas_meta_payload_from_atlas_items(&items).unwrap();
    assert_eq!(&payload[0..8], b"NEIATM1\0");
    assert_eq!(u32::from_le_bytes(payload[8..12].try_into().unwrap()), 1);
    assert_eq!(u32::from_le_bytes(payload[12..16].try_into().unwrap()), 2);
    assert_eq!(u32::from_le_bytes(payload[20..24].try_into().unwrap()), 6);
}

#[test]
fn atlas_bounds_validation_blocks_out_of_bounds_static_rects() {
    let valid = json!({
        "atlasFile": "textures/atlas/static-main.webp",
        "atlasWidth": 64,
        "atlasHeight": 64,
        "x": 48,
        "y": 48,
        "width": 16,
        "height": 16,
    });
    let invalid = json!({
        "atlasFile": "textures/atlas/static-main.webp",
        "atlasWidth": 64,
        "atlasHeight": 64,
        "x": 60,
        "y": 48,
        "width": 16,
        "height": 16,
    });
    let zero_sized = json!({
        "atlasFile": "textures/atlas/static-main.webp",
        "atlasWidth": 64,
        "atlasHeight": 64,
        "x": 0,
        "y": 0,
        "width": 0,
        "height": 16,
    });
    let mut invalid_bounds = Vec::new();
    validate_atlas_bounds("valid-item", "static", Some(&valid), &mut invalid_bounds);
    assert!(invalid_bounds.is_empty());

    validate_atlas_bounds("bad-item", "static", Some(&invalid), &mut invalid_bounds);
    validate_atlas_bounds(
        "zero-item",
        "static",
        Some(&zero_sized),
        &mut invalid_bounds,
    );
    assert_eq!(
        invalid_bounds,
        vec![
            "bad-item:static:out-of-bounds".to_string(),
            "zero-item:static:out-of-bounds".to_string(),
        ]
    );
}

#[test]
fn atlas_bounds_validation_allows_animated_frame_only_atlas() {
    let frame_only = json!({
        "atlasFile": "textures/atlas/animated-main.webp",
        "atlasWidth": 16,
        "atlasHeight": 128,
        "frameCount": 8,
        "frames": [[0, 0, 0, 16, 16], [1, 0, 16, 16, 16]],
    });
    let animated_with_bad_rect = json!({
        "atlasFile": "textures/atlas/animated-main.webp",
        "atlasWidth": 16,
        "atlasHeight": 128,
        "x": 8,
        "y": 120,
        "width": 16,
        "height": 16,
        "frames": [[0, 0, 0, 16, 16]],
    });
    let mut invalid_bounds = Vec::new();
    validate_atlas_bounds(
        "frame-only-item",
        "animated",
        Some(&frame_only),
        &mut invalid_bounds,
    );
    assert!(invalid_bounds.is_empty());

    validate_atlas_bounds(
        "bad-animated-item",
        "animated",
        Some(&animated_with_bad_rect),
        &mut invalid_bounds,
    );
    assert_eq!(
        invalid_bounds,
        vec!["bad-animated-item:animated:out-of-bounds".to_string()]
    );
}

#[test]
fn missing_texture_report_classifies_expected_animated_static_items() {
    let animation = json!({
        "assetId": "avaritia-singularity",
        "frameCount": 4,
        "frameDurationMs": 50,
        "timeline": [{ "frameIndex": 0, "durationMs": 50 }]
    });
    let native_sprite = json!({
        "assetId": "avaritia-singularity",
        "spriteMetadataFile": "assets/minecraft/textures/items/singularity.png.mcmeta"
    });
    assert!(expected_animated_item(
        Some(&animation),
        Some(&native_sprite)
    ));
    assert_eq!(
        expected_animation_reason(Some(&animation), Some(&native_sprite)),
        "native sprite metadata exists"
    );
    assert!(!expected_animated_item(None, None));
}

#[test]
fn strict_texture_blockers_fail_after_reports_while_non_strict_remains_auditable() {
    let input = tempfile::tempdir().unwrap();
    copy_test_tree(
        &compiler_fixture_path("raw-export-texture-atlas"),
        input.path(),
    );
    let manifest_path = input.path().join("manifest.json");
    let mut manifest = read_fixture_json(&manifest_path);
    manifest["files"]["animations"] = json!("animations.jsonl");
    write_json_value(&manifest_path, &manifest).unwrap();
    fs::write(
        input.path().join("animations.jsonl"),
        b"{\"assetId\":\"nesqlpp:item/i~minecraft~iron_ingot~0\",\"frameCount\":20,\"frameDurationMs\":100}\n",
    )
    .unwrap();
    let session = crate::session::RawExportSession::open(input.path()).unwrap();

    let non_strict_output = tempfile::tempdir().unwrap();
    compile_texture_pack(&session, non_strict_output.path(), false, false).unwrap();
    let non_strict_report = read_fixture_json(
        non_strict_output
            .path()
            .join("rust/missing-texture-report.json"),
    );
    assert_eq!(non_strict_report["status"], json!("blocked"));
    assert_eq!(
        non_strict_report["counts"]["blockingActionableIssues"],
        json!(1)
    );
    assert_eq!(
        non_strict_report["issues"][0]["code"],
        json!("EXPECTED_ANIMATED_BUT_STATIC")
    );
    assert!(non_strict_output.path().join("rust/textures.bin").is_file());

    let strict_output = tempfile::tempdir().unwrap();
    let error = compile_texture_pack(&session, strict_output.path(), true, false).unwrap_err();
    assert!(
        format!("{error:#}").contains("blocking actionable issues=1"),
        "{error:#}"
    );
    let strict_report = read_fixture_json(
        strict_output
            .path()
            .join("rust/missing-texture-report.json"),
    );
    assert_eq!(
        strict_report["counts"]["blockingActionableIssues"],
        json!(1)
    );
    assert!(!strict_output.path().join("rust/textures.bin").exists());
}

#[test]
fn native_sprite_snapshot_animation_facts_do_not_fabricate_animated_atlas_frames() {
    let atlas = json!({
        "items": [{
            "itemId": "i~Railcraft~cart.redstone.flux~0",
            "assetId": "nesqlpp:item/i~Railcraft~cart.redstone.flux~0",
            "hasStaticAtlas": true,
            "hasAnimatedAtlas": false,
            "staticAtlas": {
                "atlasFile": "assets/textures/atlas-assets/atlases/item-native-static-011.png",
                "atlasWidth": 2048,
                "atlasHeight": 2048,
                "x": 512,
                "y": 128,
                "width": 64,
                "height": 64
            }
        }]
    });
    let animation = json!({
        "assetId": "nesqlpp:item/i~Railcraft~cart.redstone.flux~0",
        "mode": "native_sprite_snapshot",
        "animationMode": "none",
        "frameCount": 20,
        "frameDurationMs": 100,
        "timeline": [
            { "timelineIndex": 0.0, "frameIndex": 0.0, "durationMs": 100.0 },
            { "timelineIndex": 1.0, "frameIndex": 1.0, "durationMs": 100.0 }
        ]
    });
    let mut animations = BTreeMap::new();
    animations.insert(
        "nesqlpp:item/i~Railcraft~cart.redstone.flux~0".to_string(),
        animation,
    );
    let mut items = atlas["items"].as_array().cloned().unwrap();
    promote_animation_facts_to_animated_atlas_items(
        &mut items,
        &animations,
        &BTreeMap::new(),
        &BTreeMap::new(),
    );
    let item = &items[0];
    assert_eq!(item["hasAnimatedAtlas"], json!(false));
    assert!(item.get("animatedAtlas").is_none());
}

#[test]
fn materialized_animation_frames_can_promote_static_atlas() {
    let mut items = vec![json!({
        "itemId": "animated",
        "assetId": "asset:animated",
        "hasStaticAtlas": true,
        "hasAnimatedAtlas": false,
    })];
    let mut animations = BTreeMap::new();
    animations.insert(
        "asset:animated".to_string(),
        json!({
            "assetId": "asset:animated",
            "runtimeFrameCount": 2,
            "declaredFrameCount": 2,
            "materializedFrameCount": 2,
            "distinctFrameCount": 2,
            "materializationStatus": "materialized",
            "materializationReason": "two distinct runtime frames were captured",
            "atlasFile": "assets/textures/atlas-assets/animated.png",
            "atlasWidth": 16,
            "atlasHeight": 32,
            "frameDurationMs": 75,
            "frames": [
                {
                    "materializationIndex": 0,
                    "frameIndex": 0,
                    "contentHash": "sha256:frame-0",
                    "rect": { "x": 0, "y": 0, "width": 16, "height": 16 }
                },
                {
                    "materializationIndex": 1,
                    "frameIndex": 1,
                    "contentHash": "sha256:frame-1",
                    "rect": { "x": 0, "y": 16, "width": 16, "height": 16 }
                }
            ]
        }),
    );

    promote_animation_facts_to_animated_atlas_items(
        &mut items,
        &animations,
        &BTreeMap::new(),
        &BTreeMap::new(),
    );

    assert_eq!(items[0]["hasAnimatedAtlas"], json!(true));
    assert_eq!(items[0]["animatedAtlas"]["frameCount"], json!(2));
    assert_eq!(
        items[0]["animatedAtlas"]["animationSource"],
        json!("authoritative_animation_frame_materialization")
    );
    assert_ne!(
        items[0]["animatedAtlas"]["frames"][0]["y"],
        items[0]["animatedAtlas"]["frames"][1]["y"]
    );
}

#[test]
fn materialized_animation_inherits_raw_animation_duration_when_fact_omits_it() {
    let mut items = vec![json!({
        "itemId": "animated",
        "assetId": "asset:animated",
        "hasStaticAtlas": true,
        "hasAnimatedAtlas": false,
    })];
    let mut materializations = BTreeMap::new();
    materializations.insert(
        "asset:animated".to_string(),
        json!({
            "assetId": "asset:animated",
            "runtimeFrameCount": 2,
            "declaredFrameCount": 2,
            "materializedFrameCount": 2,
            "distinctFrameCount": 2,
            "materializationStatus": "materialized",
            "materializationReason": "two distinct runtime frames were captured",
            "atlasFile": "assets/animations/materialization-atlases/item/animated.sprite-atlas.png",
            "frames": [
                {
                    "materializationIndex": 0,
                    "frameIndex": 0,
                    "contentHash": "sha256:frame-0",
                    "rect": { "x": 0, "y": 0, "width": 16, "height": 16 }
                },
                {
                    "materializationIndex": 1,
                    "frameIndex": 1,
                    "contentHash": "sha256:frame-1",
                    "rect": { "x": 0, "y": 16, "width": 16, "height": 16 }
                }
            ]
        }),
    );
    let mut animations = BTreeMap::new();
    animations.insert(
        "asset:animated".to_string(),
        json!({
            "assetId": "asset:animated",
            "frameDurationMs": 50,
        }),
    );

    promote_animation_facts_to_animated_atlas_items(
        &mut items,
        &materializations,
        &animations,
        &BTreeMap::new(),
    );

    assert_eq!(items[0]["animatedAtlas"]["frameDurationMs"], json!(50));
    assert_eq!(
        items[0]["animatedAtlas"]["frameDurationSource"],
        json!("raw_animation_index")
    );
    assert_eq!(
        items[0]["animatedAtlas"]["timeline"],
        json!([
            { "frameIndex": 0, "durationMs": 50 },
            { "frameIndex": 1, "durationMs": 50 }
        ])
    );
}

#[test]
fn materialized_animation_inherits_raw_animation_duration_when_fact_duration_is_zero() {
    let mut items = vec![json!({
        "itemId": "animated",
        "assetId": "asset:animated",
        "hasStaticAtlas": true,
        "hasAnimatedAtlas": false,
    })];
    let mut materializations = BTreeMap::new();
    materializations.insert(
        "asset:animated".to_string(),
        json!({
            "assetId": "asset:animated",
            "runtimeFrameCount": 2,
            "declaredFrameCount": 2,
            "materializedFrameCount": 2,
            "distinctFrameCount": 2,
            "materializationStatus": "materialized",
            "materializationReason": "two distinct runtime frames were captured",
            "atlasFile": "assets/animations/materialization-atlases/item/animated.sprite-atlas.png",
            "frameDurationMs": 0,
            "frames": [
                {
                    "materializationIndex": 0,
                    "frameIndex": 0,
                    "contentHash": "sha256:frame-0",
                    "rect": { "x": 0, "y": 0, "width": 16, "height": 16 }
                },
                {
                    "materializationIndex": 1,
                    "frameIndex": 1,
                    "contentHash": "sha256:frame-1",
                    "rect": { "x": 0, "y": 16, "width": 16, "height": 16 }
                }
            ]
        }),
    );
    let mut animations = BTreeMap::new();
    animations.insert(
        "asset:animated".to_string(),
        json!({
            "assetId": "asset:animated",
            "frameDurationMs": 50,
        }),
    );

    promote_animation_facts_to_animated_atlas_items(
        &mut items,
        &materializations,
        &animations,
        &BTreeMap::new(),
    );

    assert_eq!(items[0]["animatedAtlas"]["frameDurationMs"], json!(50));
    assert_eq!(
        items[0]["animatedAtlas"]["frameDurationSource"],
        json!("raw_animation_index")
    );
    assert_eq!(
        items[0]["animatedAtlas"]["timeline"],
        json!([
            { "frameIndex": 0, "durationMs": 50 },
            { "frameIndex": 1, "durationMs": 50 }
        ])
    );
}

#[test]
fn missing_facade_resolution_fact_is_reported_without_nbt_inference() {
    let result = crate::atlas_repair::repaired_browser_atlas_items(
        &json!({ "items": [] }),
        &[],
        &[json!({
            "itemId": "i~BuildCraftTransport~pipeFacade~0~missing",
            "renderAssetRef": "nesqlpp:item/i~BuildCraftTransport~pipeFacade~0~missing",
            "modId": "BuildCraftTransport",
            "internalName": "pipeFacade",
            "semanticFamily": "facade.buildcraft",
            "nbtDescriptor": "{block:\"missing:block\",metadata:3}",
        })],
        &[],
    );

    assert_eq!(result.repaired_facades, 0);
    assert_eq!(result.unresolved_facades.len(), 1);
    assert_eq!(
        result.unresolved_facades[0]["code"],
        json!("FACADE_RESOLUTION_FACT_MISSING")
    );
    assert!(result.unresolved_facades[0]
        .get("targetLookupKey")
        .is_none());
}

#[test]
fn authoritative_facade_resolution_joins_exact_source_asset_and_item() {
    let source_item_id = "i~minecraft~stone~3";
    let source_asset_id = "nesqlpp:item/i~minecraft~stone~3";
    let facade_item_id = "i~BuildCraftTransport~pipeFacade~0~stone";
    let facade_asset_id = "nesqlpp:item/i~BuildCraftTransport~pipeFacade~0~stone";
    let result = crate::atlas_repair::repaired_browser_atlas_items(
        &json!({
            "items": [{
                "itemId": source_item_id,
                "assetId": source_asset_id,
                "hasStaticAtlas": true,
                "staticAtlas": {
                    "atlasFile": "textures/atlas/static.png",
                    "atlasWidth": 16,
                    "atlasHeight": 16,
                    "x": 0,
                    "y": 0,
                    "width": 16,
                    "height": 16
                }
            }]
        }),
        &[],
        &[json!({
            "itemId": facade_item_id,
            "renderAssetRef": facade_asset_id,
            "modId": "BuildCraftTransport",
            "internalName": "pipeFacade",
            "semanticFamily": "facade.buildcraft",
            "nbtDescriptor": "{block:\"wrong:target\",metadata:99}",
        })],
        &[json!({
            "facadeItemId": facade_item_id,
            "facadeAssetId": facade_asset_id,
            "status": "resolved",
            "reason": "runtime facade target resolved",
            "targetCount": 1,
            "resolvedTargetCount": 1,
            "targets": [{
                "sourceItemId": source_item_id,
                "sourceAssetId": source_asset_id,
                "targetIndex": 0,
                "blockRegistryName": "minecraft:stone",
                "meta": 3,
                "status": "resolved",
                "reason": "source-item-resolved-from-buildcraft-target"
            }]
        })],
    );

    assert_eq!(result.repaired_facades, 1);
    assert!(result.unresolved_facades.is_empty());
    let facade = result
        .items
        .iter()
        .find(|item| item["itemId"] == json!(facade_item_id))
        .unwrap();
    assert_eq!(facade["sourceItemId"], json!(source_item_id));
    assert_eq!(facade["sourceAssetId"], json!(source_asset_id));
    assert_eq!(
        facade["resolutionMode"],
        json!("authoritative_facade_resolution")
    );
}

#[test]
fn contradictory_facade_resolution_contracts_fail_closed_before_source_join() {
    let source_item_id = "i~minecraft~stone~3";
    let source_asset_id = "nesqlpp:item/i~minecraft~stone~3";
    let facade_item_id = "i~BuildCraftTransport~pipeFacade~0~stone";
    let facade_asset_id = "nesqlpp:item/i~BuildCraftTransport~pipeFacade~0~stone";
    let atlas = json!({
        "items": [{
            "itemId": source_item_id,
            "assetId": source_asset_id,
            "hasStaticAtlas": true,
            "staticAtlas": {
                "atlasFile": "textures/atlas/static.png",
                "atlasWidth": 16,
                "atlasHeight": 16,
                "x": 0,
                "y": 0,
                "width": 16,
                "height": 16
            }
        }]
    });
    let item = json!({
        "itemId": facade_item_id,
        "renderAssetRef": facade_asset_id,
        "modId": "BuildCraftTransport",
        "internalName": "pipeFacade",
        "semanticFamily": "facade.buildcraft"
    });
    let target = json!({
        "targetIndex": 0,
        "blockRegistryName": "minecraft:stone",
        "meta": 3,
        "sourceItemId": source_item_id,
        "sourceAssetId": source_asset_id,
        "status": "resolved",
        "reason": "source-item-resolved-from-buildcraft-target"
    });
    let contradictions = [
        json!({
            "facadeItemId": facade_item_id,
            "facadeAssetId": facade_asset_id,
            "status": "resolved",
            "reason": "target count mismatch",
            "targetCount": 2,
            "resolvedTargetCount": 1,
            "targets": [target.clone()]
        }),
        json!({
            "facadeItemId": facade_item_id,
            "facadeAssetId": facade_asset_id,
            "status": "resolved",
            "reason": "status should be partial",
            "targetCount": 2,
            "resolvedTargetCount": 1,
            "targets": [
                target.clone(),
                {
                    "targetIndex": 1,
                    "blockRegistryName": null,
                    "meta": 0,
                    "status": "unresolved",
                    "reason": "target unavailable"
                }
            ]
        }),
        json!({
            "facadeItemId": facade_item_id,
            "facadeAssetId": facade_asset_id,
            "status": "resolved",
            "reason": "target index mismatch",
            "targetCount": 1,
            "resolvedTargetCount": 1,
            "targets": [{
                "targetIndex": 1,
                "blockRegistryName": "minecraft:stone",
                "meta": 3,
                "sourceItemId": source_item_id,
                "sourceAssetId": source_asset_id,
                "status": "resolved",
                "reason": "source-item-resolved-from-buildcraft-target"
            }]
        }),
    ];

    for contradiction in contradictions {
        let result = crate::atlas_repair::repaired_browser_atlas_items(
            &atlas,
            &[],
            std::slice::from_ref(&item),
            std::slice::from_ref(&contradiction),
        );
        assert_eq!(result.repaired_facades, 0);
        assert_eq!(result.unresolved_facades.len(), 1);
        assert_eq!(
            result.unresolved_facades[0]["code"],
            json!("FACADE_RESOLUTION_FACT_INVALID")
        );
    }
}

#[test]
fn certified_static_animation_is_advisory_and_count_contradictions_block() {
    let certified_static = json!({
        "assetId": "asset:static",
        "runtimeFrameCount": 20,
        "declaredFrameCount": 20,
        "materializedFrameCount": 1,
        "distinctFrameCount": 1,
        "materializationStatus": "static",
        "materializationReason": "runtime exposed one distinct materialized rectangle",
        "atlasFile": "textures/atlas/static.png",
        "frames": [{ "index": 0, "x": 0, "y": 0, "width": 16, "height": 16 }]
    });
    let (diagnostic, blocking) =
        animation_materialization_diagnostic("item:static", "asset:static", &certified_static)
            .unwrap();
    assert!(!blocking);
    assert_eq!(diagnostic["code"], json!("ANIMATION_CERTIFIED_STATIC"));
    assert_eq!(diagnostic["materializedFrameCount"], json!(1));
    assert_eq!(diagnostic["distinctFrameCount"], json!(1));
    assert_eq!(
        diagnostic["materializationReason"],
        json!("runtime exposed one distinct materialized rectangle")
    );
    assert!(diagnostic.get("validationError").is_none());

    let invalid = json!({
        "assetId": "asset:invalid",
        "runtimeFrameCount": 20,
        "declaredFrameCount": 20,
        "materializedFrameCount": 1,
        "distinctFrameCount": 2,
        "materializationStatus": "static",
        "materializationReason": "contradictory counts"
    });
    let (diagnostic, blocking) =
        animation_materialization_diagnostic("item:invalid", "asset:invalid", &invalid).unwrap();
    assert!(blocking);
    assert_eq!(
        diagnostic["code"],
        json!("ANIMATION_MATERIALIZATION_INVALID")
    );
}

#[test]
fn wrapped_numeric_texture_frames_compile_without_invalid_bounds() {
    let animated_atlas = json!({
        "atlasFile": "assets/textures/atlas-assets/animated-atlases/item-native-animated.png",
        "atlasWidth": { "value": "2048" },
        "atlasHeight": { "value": "4096" },
        "frameDurationMs": { "value": "50" },
        "frameCount": { "value": "2" },
        "frames": [
            {
                "index": { "value": "0" },
                "x": { "value": "0" },
                "y": { "value": "0" },
                "width": { "value": "16" },
                "height": { "value": "16" }
            },
            {
                "index": { "value": 1 },
                "x": { "value": 16 },
                "y": { "value": 0 },
                "width": { "value": 16 },
                "height": { "value": 16 }
            }
        ],
        "timeline": [
            {
                "timelineIndex": { "value": "0" },
                "frameIndex": { "value": "0" },
                "durationMs": { "value": "50" }
            },
            {
                "timelineIndex": { "value": "1" },
                "frameIndex": { "value": "1" },
                "durationMs": { "value": "75" }
            }
        ]
    });

    let mut invalid_bounds = Vec::new();
    validate_frame_bounds(
        "i~AWWayofTime~lifeEssence~0",
        Some(&animated_atlas),
        &mut invalid_bounds,
    );
    assert!(invalid_bounds.is_empty(), "{invalid_bounds:?}");

    let normalized = normalize_timeline(
        Some(&animated_atlas),
        value_u64(&animated_atlas, "frameDurationMs"),
    );
    assert_eq!(
        normalized
            .as_array()
            .and_then(|values| values.get(1))
            .and_then(|value| value_u64(value, "durationMs")),
        Some(75)
    );

    let items = vec![json!({
        "itemId": "i~AWWayofTime~lifeEssence~0",
        "animatedAtlas": animated_atlas
    })];
    let payload = build_compact_texture_payload_from_atlas_items(&items).unwrap();
    assert_eq!(&payload[0..8], b"NEITEX1\0");
    assert_eq!(u32::from_le_bytes(payload[20..24].try_into().unwrap()), 2);
}

#[test]
fn timeline_frame_indices_wrap_to_available_exported_frames() {
    let animated_atlas = json!({
        "atlasFile": "textures/atlas/animated-main.webp",
        "frameCount": 16,
        "frameDurationMs": 50,
        "frames": [
            [0, 0, 0, 16, 16],
            [1, 16, 0, 16, 16],
            [2, 32, 0, 16, 16],
            [3, 48, 0, 16, 16]
        ],
        "timeline": [
            { "frameIndex": 0, "durationMs": 50 },
            { "frameIndex": 4, "durationMs": 50 },
            { "frameIndex": 5, "durationMs": 50 },
            { "frameIndex": 15, "durationMs": 50 }
        ]
    });

    let normalized = normalize_timeline(Some(&animated_atlas), Some(50));
    let values = normalized.as_array().expect("timeline");
    assert_eq!(values[0]["frameIndex"], json!(0));
    assert_eq!(values[1]["frameIndex"], json!(0));
    assert_eq!(values[2]["frameIndex"], json!(1));
    assert_eq!(values[3]["frameIndex"], json!(3));
}

#[test]
fn compact_string_pack_uses_native_binary_payload() {
    let items = vec![json!({
        "itemId": "minecraft:iron_ingot",
        "localizedName": "Iron Ingot",
        "modId": "minecraft",
        "internalName": "item.ingotIron",
        "groupKey": "",
        "groupLabel": "",
    })];
    let payload = build_compact_string_payload_from_items(&items).unwrap();
    assert_eq!(&payload[0..8], b"NEISTR1\0");
    assert_eq!(u32::from_le_bytes(payload[8..12].try_into().unwrap()), 1);
    assert_eq!(u32::from_le_bytes(payload[12..16].try_into().unwrap()), 1);
    assert_eq!(u32::from_le_bytes(payload[20..24].try_into().unwrap()), 6);
}

#[test]
fn compact_search_pack_uses_native_binary_payload() {
    let items = vec![json!({
        "itemId": "minecraft:iron_ingot",
        "publicItemId": "item:minecraft:iron_ingot",
        "localizedName": "Iron Ingot",
        "modId": "minecraft",
        "normalizedLocalizedName": "iron ingot",
        "normalizedInternalName": "item ingotiron",
        "normalizedItemId": "minecraft iron_ingot",
        "normalizedSearchTerms": "iron ingot minecraft item ingotiron",
        "pinyinFull": "tieding",
        "pinyinAcronym": "td",
        "popularityScore": 3,
        "searchRank": 7
    })];
    let payload = build_compact_search_payload_from_items(&items).unwrap();
    assert_eq!(&payload[0..8], b"NEISRC2\0");
    assert_eq!(u32::from_le_bytes(payload[8..12].try_into().unwrap()), 1);
    assert_eq!(u32::from_le_bytes(payload[12..16].try_into().unwrap()), 1);
    assert_eq!(u32::from_le_bytes(payload[20..24].try_into().unwrap()), 13);
}

#[test]
fn compact_recipe_pack_uses_native_binary_payload() {
    let pack = json!({
        "itemIndex": [{
            "itemId": "i~minecraft~iron_ingot~0",
            "producedBy": [{ "recipeId": "r1", "categoryId": "display~furnace", "displayName": "Furnace" }],
            "usedIn": [{ "recipeId": "r2", "categoryId": "display~crafting", "displayName": "Crafting" }]
        }],
        "uiPayloadIndex": [{
            "recipeId": "r1",
            "path": "recipes/ui-payload-shards/55.json",
            "payloadKey": "r1",
            "familyKey": "furnace",
            "recipeType": "furnace",
            "machineType": "Furnace",
            "handlerKey": "codechicken.nei.recipe.furnacerecipehandler"
        }],
        "categoryIndex": [{
            "categoryId": "display~furnace",
            "displayName": "Furnace",
            "recipeCount": 1,
            "sourceCategoryIds": ["codechicken.nei.recipe.furnacerecipehandler"],
            "machineIcon": {
                "itemId": "i~minecraft~furnace~0",
                "renderAssetRef": "nesqlpp:item/i~minecraft~furnace~0"
            }
        }]
    });
    let payload = build_compact_recipe_payload_from_pack(&pack).unwrap();
    assert_eq!(&payload[0..8], b"NEIRCP1\0");
    assert_eq!(u32::from_le_bytes(payload[8..12].try_into().unwrap()), 1);
    assert_eq!(u32::from_le_bytes(payload[16..20].try_into().unwrap()), 1);
    assert_eq!(u32::from_le_bytes(payload[20..24].try_into().unwrap()), 2);
    assert_eq!(u32::from_le_bytes(payload[24..28].try_into().unwrap()), 1);
    assert_eq!(u32::from_le_bytes(payload[28..32].try_into().unwrap()), 1);
    assert_eq!(u32::from_le_bytes(payload[36..40].try_into().unwrap()), 5);
    assert_eq!(u32::from_le_bytes(payload[40..44].try_into().unwrap()), 3);
    assert_eq!(u32::from_le_bytes(payload[44..48].try_into().unwrap()), 7);
    assert_eq!(u32::from_le_bytes(payload[48..52].try_into().unwrap()), 7);
}

#[test]
fn recipe_item_id_collector_supports_raw_item_input_variants() {
    let recipe = json!({
        "recipeId": "r~aMY5AipXMtWIL3tAQPCNgg==",
        "itemInputs": [{
            "slotIndex": 0,
            "groupId": "ig~XK1983d_NXCakt6S8nNPSg==",
            "variants": [{
                "itemId": "i~minecraft~cobblestone~0",
                "publicItemId": "item:i~minecraft~cobblestone~0",
                "variantId": null,
                "stackSize": 1
            }]
        }],
        "itemOutputs": [{
            "slotIndex": 0,
            "itemId": "i~minecraft~stone~0",
            "publicItemId": "item:i~minecraft~stone~0",
            "stackSize": 1,
            "probability": 1.0
        }]
    });

    assert_eq!(
        collect_recipe_item_ids(&recipe, &["itemInputs"]),
        vec!["i~minecraft~cobblestone~0"]
    );
    assert_eq!(
        collect_recipe_item_ids(&recipe, &["itemOutputs"]),
        vec!["i~minecraft~stone~0"]
    );
}

#[test]
fn rust_recipe_ui_payload_paths_match_raw_export_sha1_shards() {
    assert_eq!(
        rust_recipe_ui_payload_relative_path("r1"),
        "recipes/ui-payload-shards/55.json"
    );
    assert_eq!(
        rust_recipe_ui_payload_relative_path("r~H_tVg74GOf6PmoNkwtcMLQ=="),
        "recipes/ui-payload-shards/45.json"
    );
    assert_eq!(
        rust_recipe_ui_payload_relative_path("r~prZx3D_BO22sF1-hHpwvbA=="),
        "recipes/ui-payload-shards/96.json"
    );
}

#[test]
fn captured_ui_family_key_matches_nesqlpp_census_contract() {
    let handler = json!({
        "handlerKey": "gt.recipe.assemblyline",
        "canonicalMachineFamily": "GregTech-Machine",
        "imageResource": "textures/gui/legacy.png"
    });
    let layout = json!({
        "handlerKey": "gt.recipe.assemblyline",
        "layoutKind": "Machine",
        "width": 176,
        "height": 90,
        "yShift": -4,
        "maxRecipesPerPage": 2,
        "imageResource": " textures/gui/GT5UAssemblyLine.png "
    });

    assert_eq!(
        captured_ui_family_key(Some(&handler), Some(&layout)).as_deref(),
        Some("gregtech-machine|machine|176x90@-4#2|textures/gui/gt5uassemblyline.png")
    );
}

#[test]
fn runtime_recipe_type_ids_resolve_to_nei_handler_keys() {
    let handlers = vec![json!({
        "handlerKey": "gt.recipe.laserengraver",
        "handlerClass": "gt.recipe.laserengraver",
        "canonicalMachineFamily": "gregtech-machine"
    })];
    let layouts = vec![json!({
        "handlerKey": "gt.recipe.laserengraver",
        "canonicalMachineFamily": "gregtech-machine",
        "layoutKind": "machine",
        "width": 166,
        "height": 135,
        "maxRecipesPerPage": 2
    })];
    let recipe = json!({
        "family": "gregtech",
        "sourcePlugin": "gregtech",
        "machine": {
            "machineId": "rt~gregtech~gt.recipe.laserengraver~MV",
            "displayName": "gregtech - Laser Engraver (MV)"
        }
    });
    let context = RecipeHandlerContext::new(&handlers, &layouts);

    let (handler, layout) = context.resolve(&recipe);

    assert_eq!(
        handler
            .and_then(|value| value_string(value, "handlerKey"))
            .as_deref(),
        Some("gt.recipe.laserengraver")
    );
    assert_eq!(
        captured_ui_family_key(handler, layout).as_deref(),
        Some("gregtech-machine|machine|166x135@0#2|unknown")
    );
}

#[test]
fn public_recipe_layout_preserves_static_surface_fields_only() {
    let layout = json!({
        "handlerKey": "gt.recipe.assemblyline",
        "canonicalMachineFamily": "gregtech-machine",
        "layoutKind": "machine",
        "width": 176,
        "height": 90,
        "imageResource": "textures/gui/gt5u_assembly_line.png",
        "imageRegion": { "x": 4, "y": 8, "width": 176, "height": 90 },
        "nativeBackground": {
            "status": "captured",
            "kind": "gt-modular-ui",
            "assetRef": "assets/ui-backgrounds/gregtech/nei_single_recipe.png",
            "scaling": "nine-slice"
        },
        "hotspots": [{
            "id": "machine-info",
            "label": "Machine info",
            "x": 6,
            "y": 6,
            "width": 48,
            "height": 12
        }],
        "viewports": [{
            "id": "preview",
            "kind": "item-preview",
            "x": 120,
            "y": 8,
            "width": 32,
            "height": 32
        }]
    });

    let public_layout = public_recipe_layout(&layout);

    assert_eq!(
        public_layout["imageResource"],
        json!("textures/gui/gt5u_assembly_line.png")
    );
    assert_eq!(
        public_layout["canonicalMachineFamily"],
        json!("gregtech-machine")
    );
    assert_eq!(public_layout["imageRegion"]["x"], json!(4));
    assert!(public_layout["nativeBackground"].get("assetRef").is_none());
    assert!(public_layout["nativeBackground"].get("resource").is_none());
    assert_eq!(
        public_layout["nativeBackground"]["kind"],
        json!("gt-modular-ui")
    );
    assert!(public_layout.get("progressBars").is_none());
    assert!(public_layout.get("fluidBars").is_none());
    assert!(public_layout.get("energyBars").is_none());
    assert!(public_layout.get("dynamicPrimitives").is_none());
    assert_eq!(
        public_layout["hotspots"].as_array().unwrap()[0]["label"],
        json!("Machine info")
    );
    assert_eq!(
        public_layout["viewports"].as_array().unwrap()[0]["kind"],
        json!("item-preview")
    );
}

#[test]
fn ui_template_bindings_separate_capture_keys_from_stable_family_keys() {
    let captured_family_key =
        "gregtech-machine|machine|176x90@-4#2|textures/gui/gt5uassemblyline.png";
    let templates = enrich_ui_templates_with_presentation(vec![json!({
        "templateKey": "ui-template/assembly-line",
        "templateSignature": "assemblyline123",
        "captureKey": captured_family_key,
        "canonicalMachineFamily": "gregtech-machine",
        "layoutKind": "machine"
    })])
    .unwrap();
    let recipe_index = vec![json!({
        "recipeId": "r_gt_assembly_line",
        "captureKey": captured_family_key,
        "recipeType": "gt.recipe.assemblyline",
        "machineType": "Assembly Line"
    })];

    let bindings = build_ui_template_bindings(&recipe_index, &templates).unwrap();

    assert_eq!(
        bindings[0]["templateKey"],
        json!("ui-template/assembly-line")
    );
    assert_eq!(bindings[0]["captureKey"], json!(captured_family_key));
    assert_eq!(bindings[0]["familyKey"], json!("gregtech-machine"));
    assert_eq!(bindings[0]["presentationSurface"], json!("machine"));
    assert_eq!(bindings[0]["layoutId"], json!("gregtech-machine"));
    assert_eq!(bindings[0]["rendererId"], json!("gt_generic"));
}

#[test]
fn ui_template_bindings_reject_missing_v2_capture_key() {
    let capture_key = "gregtech-machine|machine|176x90@-4#2|textures/gui/gt5uassemblyline.png";
    let templates = enrich_ui_templates_with_presentation(vec![json!({
        "templateKey": "ui-template/assembly-line",
        "templateSignature": "assemblyline123",
        "captureKey": capture_key,
        "canonicalMachineFamily": "gregtech-machine",
        "layoutKind": "machine"
    })])
    .unwrap();
    let error = build_ui_template_bindings(
        &[json!({
            "recipeId": "r_missing_capture",
            "familyKey": capture_key,
            "recipeType": "gt.recipe.assemblyline"
        })],
        &templates,
    )
    .unwrap_err();
    assert!(error.to_string().contains("missing required v2 captureKey"));
}

#[test]
fn ui_template_bindings_reject_duplicate_capture_keys() {
    let capture_key = "gregtech-machine|machine|176x90@-4#2|textures/gui/gt5uassemblyline.png";
    let templates = vec![
        json!({
            "templateKey": "ui-template/assembly-line-a",
            "captureKey": capture_key,
        }),
        json!({
            "templateKey": "ui-template/assembly-line-b",
            "captureKey": capture_key,
        }),
    ];

    let error = build_ui_template_bindings(&[], &templates)
        .unwrap_err()
        .to_string();
    assert!(
        error.contains("duplicate captureKey")
            && error.contains(capture_key)
            && error.contains("ui-template/assembly-line-a")
            && error.contains("ui-template/assembly-line-b"),
        "{error}"
    );
}

#[test]
fn ui_presentation_catalog_rejects_unmapped_capture_families() {
    let error = enrich_ui_templates_with_presentation(vec![json!({
        "templateKey": "ui-template/unknown",
        "templateSignature": "unknown123",
        "captureKey": "unknown|machine|176x90@0#1|unknown",
        "canonicalMachineFamily": "unknown",
        "layoutKind": "machine"
    })])
    .unwrap_err();

    assert!(error
        .to_string()
        .contains("ui presentation catalog has no exact mapping"));
}

#[test]
fn ui_presentation_catalog_maps_generic_native_nei_to_web_renderer() {
    let templates = enrich_ui_templates_with_presentation(vec![
        json!({
            "templateKey": "ui-template/native-nei",
            "templateSignature": "native-nei-123",
            "captureKey": "native-nei|native-nei|166x85@0#4|unknown",
            "canonicalMachineFamily": "native-nei",
            "layoutKind": "native-nei"
        }),
        json!({
            "templateKey": "ui-template/fluid-machine",
            "templateSignature": "fluid-machine-123",
            "captureKey": "fluid-machine|fluid-machine|180x85@0#4|unknown",
            "canonicalMachineFamily": "fluid-machine",
            "layoutKind": "fluid-machine"
        }),
    ])
    .unwrap();

    let native_nei = templates
        .iter()
        .find(|template| template["templateKey"] == json!("ui-template/native-nei"))
        .unwrap();
    assert_eq!(native_nei["familyKey"], json!("native-nei"));
    assert_eq!(native_nei["presentationSurface"], json!("machine"));
    assert_eq!(native_nei["layoutId"], json!("native-nei-generic"));
    assert_eq!(native_nei["rendererId"], json!("native_nei"));
    let fluid_machine = templates
        .iter()
        .find(|template| template["templateKey"] == json!("ui-template/fluid-machine"))
        .unwrap();
    assert_eq!(fluid_machine["familyKey"], json!("fluid-machine"));
    assert_eq!(fluid_machine["rendererId"], json!("native_nei"));
}

#[test]
fn ui_presentation_catalog_synthesizes_exact_templates_for_direct_recipe_families() {
    let templates = enrich_ui_templates_with_presentation(Vec::new()).unwrap();
    let gregtech = templates
        .iter()
        .find(|template| template["captureKey"] == json!("gregtech"))
        .expect("synthetic GregTech web template");
    assert_eq!(gregtech["templateKey"], json!("web-authored/gregtech"));
    assert_eq!(gregtech["rendererId"], json!("gt_generic"));
    assert_eq!(gregtech["nativeBackground"]["status"], json!("semantic"));

    let bindings = build_ui_template_bindings(
        &[json!({
            "recipeId": "direct-gregtech",
            "captureKey": "gregtech",
            "recipeType": "gregtech",
            "machineType": "gregtech"
        })],
        &templates,
    )
    .unwrap();
    assert_eq!(bindings[0]["templateKey"], json!("web-authored/gregtech"));
    assert_eq!(bindings[0]["rendererId"], json!("gt_generic"));
}

#[test]
fn ui_assets_manifest_retires_native_background_png_assets() {
    let templates = vec![json!({
        "templateKey": "gt-machine@default",
        "imageResource": "",
        "nativeBackground": {
            "status": "captured",
            "kind": "gt-modular-ui",
            "assetRef": "assets/ui-backgrounds/gregtech/nei_single_recipe.png",
            "scaling": "nine-slice"
        }
    })];
    let manifest = build_ui_assets_manifest(&templates);
    assert_eq!(manifest["assetPolicy"], json!("web-authored-ui-only"));
    assert_eq!(
        manifest["retiredNativeArtifacts"],
        json!(["nei-background-png", "nei-frame-png"])
    );
    assert!(manifest["assets"].as_array().unwrap().is_empty());
}

#[test]
fn recipe_domain_skips_redundant_codechicken_workbench_nei_recipes() {
    let recipe = json!({
        "recipeId": "r_codechicken_shaped",
        "recipeType": "rt~minecraft~codechicken_nei_recipe_shaped recipehandler",
        "family": "crafting-table",
        "machine": {
            "displayName": "Shaped"
        }
    });
    let handler = json!({
        "handlerKey": "codechicken.nei.recipe.ShapedRecipeHandler",
        "handlerClass": "codechicken.nei.recipe.ShapedRecipeHandler",
        "displayName": "Shaped"
    });

    assert!(should_skip_redundant_nei_workbench_recipe(
        &recipe,
        Some(&handler)
    ));
}

#[test]
fn recipe_domain_keeps_non_vanilla_crafting_like_nei_handlers() {
    let recipe = json!({
        "recipeId": "r_ae_shaped",
        "recipeType": "rt~appliedenergistics2~nei_ae_shaped",
        "family": "crafting-table",
        "machine": {
            "displayName": "NEIAEShaped"
        }
    });
    let handler = json!({
        "handlerKey": "appeng.integration.modules.NEIHelpers.NEIAEShapedRecipeHandler",
        "handlerClass": "appeng.integration.modules.NEIHelpers.NEIAEShapedRecipeHandler",
        "displayName": "NEIAEShaped"
    });

    assert!(!should_skip_redundant_nei_workbench_recipe(
        &recipe,
        Some(&handler)
    ));
}

#[test]
fn native_ui_compile_ignores_legacy_nei_frame_png_assets() {
    let output = compile_fixture("raw-export-native-ui-gt", CompileScope::NativeUi, true);
    let ui_payload_index = read_fixture_json(output.path().join("recipes/ui-payload-index.json"));
    assert!(ui_payload_index["recipes"]
        .as_array()
        .unwrap()
        .iter()
        .all(|entry| entry.get("nativeFrame").is_none()));
    assert!(!output
        .path()
        .join("rust/recipe-native-frame-assets.json")
        .exists());
    assert!(!output.path().join("assets/nei-native-frames").exists());
    assert!(!output.path().join("assets/ui-backgrounds").exists());
}

#[test]
fn compact_ui_pack_uses_shared_native_string_table() {
    let templates = vec![json!({
        "templateKey": "furnace@default",
        "templateSignature": "abc123",
        "familyKey": "furnace",
        "canonicalMachineFamily": "furnace",
        "layoutKind": "furnace",
        "coordinateSpace": "nei_pixels",
        "scaleMode": "uniform-scale",
        "anchor": "top-left",
        "width": 166,
        "height": 65,
        "yShift": -4,
        "maxRecipesPerPage": 2,
        "imageResource": "textures/gui/furnace.png",
        "nativeBackground": {
            "status": "captured",
            "kind": "gt-modular-ui",
            "coordinateSpace": "nei_pixels",
            "scaleMode": "uniform-scale",
            "anchor": "top-left",
            "width": 166,
            "height": 65,
            "assetRef": "assets/ui-backgrounds/gregtech/nei_single_recipe.png",
            "resource": "gregtech:textures/gui/background/nei_single_recipe.png",
            "drawable": "GT_UI_TEXTURE_NEI_SINGLE_RECIPE",
            "scaling": "nine-slice",
            "texture": { "width": 18, "height": 18, "borderU": 4, "borderV": 4 },
            "recipeBackgroundOffset": { "x": 0, "y": 0 },
            "recipeBackgroundSize": { "width": 166, "height": 65 }
        },
        "handlerCount": 1,
        "slots": [
            { "role": "item-input", "startIndex": 0, "columns": 1, "rows": 1, "x": 45, "y": 24, "coordinateSpace": "nei_pixels", "anchor": "top-left", "slotWidth": 18, "slotHeight": 18, "pitchX": 18, "pitchY": 18 },
            { "role": "item-output", "startIndex": 1, "columns": 1, "rows": 1, "x": 115, "y": 24, "coordinateSpace": "nei_pixels", "anchor": "top-left", "slotWidth": 18, "slotHeight": 18, "pitchX": 18, "pitchY": 18 }
        ],
        "textOverlays": [{
            "text": "EU/t",
            "x": 80,
            "y": 10,
            "width": 24,
            "height": 8,
            "coordinateSpace": "nei_pixels",
            "anchor": "top-left"
        }],
        "dynamicPrimitives": [{
            "kind": "progress-bar",
            "role": "gt-progress",
            "x": 78,
            "y": 24,
            "width": 20,
            "height": 18,
            "coordinateSpace": "nei_pixels",
            "anchor": "top-left",
            "orientation": "horizontal",
            "source": "test-fixture"
        }]
    })];
    let recipe_index = vec![json!({
        "recipeId": "r1",
        "path": "recipes/ui-payload-shards/55.json",
        "payloadKey": "r1",
        "captureKey": "furnace|furnace|166x65@0#2|textures/gui/furnace.png",
        "recipeType": "furnace",
        "machineType": "Furnace"
    })];
    let mut templates = templates;
    templates[0]["captureKey"] = json!("furnace|furnace|166x65@0#2|textures/gui/furnace.png");
    templates[0]["presentationSurface"] = json!("machine");
    templates[0]["layoutId"] = json!("furnace");
    templates[0]["rendererId"] = json!("furnace");
    let bindings = build_ui_template_bindings(&recipe_index, &templates).unwrap();
    let mut strings = vec![String::new()];
    let mut string_refs = HashMap::new();
    string_refs.insert(String::new(), 0u32);
    let template_payload =
        build_compact_ui_template_payload(&templates, &mut strings, &mut string_refs).unwrap();
    let binding_payload =
        build_compact_ui_binding_payload(&bindings, &mut strings, &mut string_refs).unwrap();
    let string_payload = build_compact_ui_string_payload(&strings).unwrap();

    assert_eq!(&template_payload[0..8], UI_TEMPLATE_MAGIC);
    assert_eq!(
        u32::from_le_bytes(template_payload[8..12].try_into().unwrap()),
        UI_TEMPLATE_PAYLOAD_VERSION
    );
    assert_eq!(
        u32::from_le_bytes(template_payload[12..16].try_into().unwrap()),
        1
    );
    assert_eq!(
        u32::from_le_bytes(template_payload[16..20].try_into().unwrap()),
        2
    );
    assert_eq!(
        u32::from_le_bytes(template_payload[20..24].try_into().unwrap()),
        1
    );
    assert_eq!(
        u32::from_le_bytes(template_payload[24..28].try_into().unwrap()),
        1
    );
    assert_eq!(
        u32::from_le_bytes(template_payload[28..32].try_into().unwrap()),
        0
    );
    assert_eq!(
        u32::from_le_bytes(template_payload[32..36].try_into().unwrap()),
        0
    );
    assert_eq!(
        u32::from_le_bytes(template_payload[36..40].try_into().unwrap()),
        UI_TEMPLATE_ROW_STRIDE_U32
    );
    assert_eq!(
        u32::from_le_bytes(template_payload[40..44].try_into().unwrap()),
        UI_SLOT_ROW_STRIDE_U32
    );
    assert_eq!(
        u32::from_le_bytes(template_payload[44..48].try_into().unwrap()),
        UI_TEXT_ROW_STRIDE_U32
    );
    assert_eq!(
        u32::from_le_bytes(template_payload[48..52].try_into().unwrap()),
        UI_PRIMITIVE_ROW_STRIDE_U32
    );
    assert_eq!(
        u32::from_le_bytes(template_payload[52..56].try_into().unwrap()),
        UI_RECT_ROW_STRIDE_U32
    );
    assert_eq!(&binding_payload[0..8], UI_BINDING_MAGIC);
    assert_eq!(
        u32::from_le_bytes(binding_payload[8..12].try_into().unwrap()),
        UI_BINDING_PAYLOAD_VERSION
    );
    assert_eq!(
        u32::from_le_bytes(binding_payload[12..16].try_into().unwrap()),
        1
    );
    assert_eq!(
        u32::from_le_bytes(binding_payload[16..20].try_into().unwrap()),
        14
    );
    assert_eq!(&string_payload[0..8], UI_STRING_MAGIC);
    assert!(u32::from_le_bytes(string_payload[12..16].try_into().unwrap()) > 8);
    assert_eq!(bindings[0]["templateKey"], json!("furnace@default"));
    assert_eq!(bindings[0]["familyKey"], json!("furnace"));
    assert_eq!(
        bindings[0]["captureKey"],
        json!("furnace|furnace|166x65@0#2|textures/gui/furnace.png")
    );
    assert_eq!(bindings[0]["rendererId"], json!("furnace"));
}

#[test]
fn compact_ui_pack_rejects_legacy_rect_action_fields() {
    let templates = vec![json!({
        "templateKey": "furnace@default",
        "templateSignature": "abc123",
        "familyKey": "furnace",
        "canonicalMachineFamily": "furnace",
        "layoutKind": "furnace",
        "coordinateSpace": "nei_pixels",
        "scaleMode": "uniform-scale",
        "anchor": "top-left",
        "width": 166,
        "height": 65,
        "yShift": -4,
        "maxRecipesPerPage": 2,
        "imageResource": "textures/gui/furnace.png",
        "handlerCount": 1,
        "slots": [],
        "textOverlays": [],
        "dynamicPrimitives": [],
        "hotspots": [{
            "id": "legacy-hotspot",
            "kind": "hotspot",
            "role": "item-output",
            "label": "Output",
            "tooltip": "",
            "action": "show-usage",
            "x": 115,
            "y": 24,
            "width": 18,
            "height": 18,
            "coordinateSpace": "nei_pixels",
            "anchor": "top-left",
            "interactionKind": "none",
            "interactionTargetKind": "none",
            "interactionTargetId": "",
            "interactionPayloadSchema": "neonei/native-ui-interaction/v1"
        }],
        "viewports": []
    })];
    let mut strings = vec![String::new()];
    let mut string_refs = HashMap::new();
    string_refs.insert(String::new(), 0u32);

    let err = build_compact_ui_template_payload(&templates, &mut strings, &mut string_refs)
        .unwrap_err()
        .to_string();
    assert!(err.contains("legacy interaction field forbidden by v7 ABI"));
}

#[test]
fn zero_recipe_diagnostics_distinguish_legal_and_suspicious_handlers() {
    let value = json!({
        "summary": {
            "status": "warning",
            "totalHandlers": 8,
            "handlersWithLoadedRecipes": 6,
            "handlersWithExportedRecipes": 4,
            "expectedEmptyHandlers": 2,
            "nativeCoveredZeroExports": 3,
            "nonRecipeInfoZeroExports": 1,
            "suspiciousZeroExports": 1,
            "partialExports": 2
        }
    });
    let diagnostics = raw_export::zero_recipe_diagnostics_from_value(&value).unwrap();
    assert_eq!(diagnostics.status.as_deref(), Some("warning"));
    assert_eq!(diagnostics.legal_zero_recipe_handlers, 6);
    assert_eq!(diagnostics.suspicious_zero_exports, 1);
    assert_eq!(diagnostics.partial_exports, 2);
}

#[test]
fn native_ui_layout_report_tracks_gregtech_backgrounds_without_inline_primitives() {
    let temp = tempfile::tempdir().unwrap();
    let recipes_dir = temp.path().join("recipes");
    fs::create_dir_all(&recipes_dir).unwrap();
    write_json_value(
        &recipes_dir.join("handler-layout-index.json"),
        &json!({
            "schemaVersion": "neonei/recipe-handler-layout-index/v1",
            "layouts": [{
                "handlerKey": "gt.recipe.test",
                "handlerClass": "gregtech.nei.GTNEIDefaultHandler",
                "canonicalMachineFamily": "gregtech-machine",
                "layoutKind": "machine",
                "width": 176,
                "height": 90,
                "maxRecipesPerPage": 1,
                "imageRegion": { "x": 0, "y": 0, "width": 176, "height": 90 },
                "nativeBackground": {
                    "status": "captured",
                    "kind": "gt-modular-ui",
                    "assetRef": "assets/ui-backgrounds/gregtech/nei_single_recipe.png",
                    "resource": "gregtech:textures/gui/background/nei_single_recipe.png",
                    "scaling": "nine-slice",
                    "texture": { "width": 64, "height": 64, "borderU": 2, "borderV": 2 }
                }
            }]
        }),
    )
    .unwrap();
    write_json_value(
        &recipes_dir.join("ui-payload-index.json"),
        &json!({
            "schemaVersion": "neonei/recipe-ui-payload-index/v1",
            "recipes": [{
                "recipeId": "gt:test",
                "captureKey": "gregtech-machine|machine|176x90@0#1|unknown",
                "nativeLayout": {
                    "canonicalMachineFamily": "gregtech-machine",
                    "imageRegion": { "x": 0, "y": 0, "width": 176, "height": 90 }
                }
            }]
        }),
    )
    .unwrap();

    let report =
        native_ui_report::compile_native_ui_layout_report(temp.path(), captured_ui_family_key)
            .unwrap()
            .unwrap();

    assert_eq!(report["status"], json!("ready"));
    assert_eq!(report["counts"]["gregtechHandlerLayouts"], json!(1));
    assert!(report["counts"]
        .get("gregtechRecipeUiPayloadsWithProgressBars")
        .is_none());
    assert!(report["samples"]
        .get("gregtechRecipePayloadsMissingProgressBars")
        .is_none());
    assert_eq!(report["backgroundStatus"], json!("semantic"));
    assert_eq!(
        report["counts"]["gregtechRecipeUiPayloadsWithNativeBackgrounds"],
        json!(1)
    );
    assert!(temp
        .path()
        .join("rust")
        .join("native-ui-layout-report.json")
        .exists());
}

#[test]
fn production_manifest_entries_exclude_debug_json_packs() {
    let production_entries = runtime::rust_manifest_file_entries(CompileScope::All, false)
        .into_iter()
        .map(|(_, path)| path)
        .collect::<Vec<_>>();
    assert!(production_entries.contains(&"rust/browser.bin"));
    assert!(production_entries.contains(&"rust/search.bin"));
    assert!(production_entries.contains(&"rust/recipes.bin"));
    assert!(production_entries.contains(&"rust/textures.bin"));
    assert!(production_entries.contains(&"rust/atlas.meta.bin"));
    assert!(production_entries.contains(&"rust/ui-pack/ui_templates.bin"));
    assert!(production_entries.contains(&"rust/ui-pack/ui_bindings.bin"));
    assert!(production_entries.contains(&"rust/ui-pack/ui_strings.bin"));
    assert!(production_entries.contains(&"rust/ui-pack/ui_assets.manifest.json"));
    assert!(production_entries.contains(&"rust/ui-pack/ui_pack_report.json"));
    assert!(production_entries.contains(&"rust/native-ui-layout-report.json"));
    assert!(production_entries.contains(&"rust/raw-export-abi-validation-report.json"));
    assert!(production_entries.contains(&"rust/native-ui-export-abi-validation-report.json"));
    assert!(production_entries.contains(&"rust/ui-pack-abi-validation-report.json"));
    assert!(production_entries.contains(&"rust/pack-validation-report.json"));
    assert!(production_entries.contains(&"rust/semantic-validation-report.json"));
    assert!(production_entries.contains(&"rust/recipe-handler-metadata-report.json"));
    assert!(production_entries.contains(&"rust/recipe-fragmentation-report.json"));
    assert!(!production_entries.contains(&"rust/browser-pack.json"));
    assert!(!production_entries.contains(&"rust/search-pack.json"));
    assert!(!production_entries.contains(&"rust/recipe-pack.json"));
    assert!(!production_entries.contains(&"rust/texture-pack.json"));

    let debug_entries = runtime::rust_manifest_file_entries(CompileScope::All, true)
        .into_iter()
        .map(|(_, path)| path)
        .collect::<Vec<_>>();
    assert!(debug_entries.contains(&"rust/browser-pack.json"));
    assert!(debug_entries.contains(&"rust/search-pack.json"));
    assert!(debug_entries.contains(&"rust/recipe-pack.json"));
    assert!(debug_entries.contains(&"rust/texture-pack.json"));
}

#[test]
fn pack_abi_registry_is_scope_specific_and_fail_closed() {
    let search_paths = runtime_pack_artifact_specs(CompileScope::Search, false)
        .into_iter()
        .map(|spec| spec.relative_path)
        .collect::<Vec<_>>();
    assert!(search_paths.contains(&"rust/search.bin"));
    assert!(search_paths.contains(&"rust/strings.zh_cn.bin"));
    assert!(search_paths.contains(&"rust/raw-export-abi-validation-report.json"));
    assert!(search_paths.contains(&"rust/native-ui-export-abi-validation-report.json"));
    assert!(search_paths.contains(&"rust/ui-pack-abi-validation-report.json"));
    assert!(search_paths.contains(&"rust/semantic-validation-report.json"));
    assert!(!search_paths.contains(&"rust/browser.bin"));
    assert!(!search_paths.contains(&"rust/ui-pack/ui_templates.bin"));

    let empty_output = tempfile::tempdir().unwrap();
    let report =
        validate_runtime_pack_abi(empty_output.path(), CompileScope::Search, false).unwrap();
    assert_eq!(report.status, "blocked");
    assert!(report
        .missing_required_artifacts
        .contains(&"rust/search.bin".to_string()));
}

#[test]
fn schema_catalog_sections_are_descriptor_owned() {
    validate_schema_catalog_descriptors();

    let required_manifest_files = raw_export_required_manifest_files();
    let optional_manifest_files = raw_export_optional_manifest_files();
    assert_eq!(
        required_manifest_files,
        vec!["items", "fluids", "recipeIndex", "browserAtlasIndex"]
    );
    assert!(optional_manifest_files.contains(&"nativeUiValidation"));
    assert!(optional_manifest_files.contains(&"neiBrowserContract"));
    assert!(optional_manifest_files.contains(&"facadeResolutions"));
    assert!(optional_manifest_files.contains(&"animationFrameMaterializations"));
    assert_eq!(
        required_manifest_files.len() + optional_manifest_files.len(),
        RAW_EXPORT_MANIFEST_FILE_DESCRIPTORS.len()
    );

    let raw_export = raw_export_schema_section();
    assert_eq!(
        raw_export["manifest"]["requiredFiles"],
        json!(required_manifest_files)
    );
    assert_eq!(
        raw_export["manifest"]["optionalFiles"],
        json!(optional_manifest_files)
    );
    assert_eq!(
        raw_export["nativeUiValidationReport"]["policy"],
        json!("missing native UI validation, blocked geometry reports, and schema mismatches are compile blockers")
    );

    let dist_data = dist_data_schema_section();
    for descriptor in DIST_DATA_SCHEMA_DESCRIPTORS {
        assert_eq!(dist_data[descriptor.key], json!(descriptor.schema_version));
    }
    for descriptor in UI_PACK_SCHEMA_DESCRIPTORS {
        assert_eq!(
            dist_data["uiPack"][descriptor.key],
            json!(descriptor.schema_version)
        );
    }
    assert_eq!(
        dist_data["uiPack"]["bindingAbi"]["version"],
        json!(UI_BINDING_PAYLOAD_VERSION)
    );
    assert_eq!(
        dist_data["uiPack"]["bindingAbi"]["rowStrideU32"],
        json!(UI_BINDING_ROW_STRIDE_U32)
    );
    assert_eq!(
        dist_data["uiPack"]["bindingAbi"]["captureKeyPolicy"],
        json!("internal-template-match-only")
    );
    assert_eq!(
        dist_data["runtimeEntrypoints"],
        json!(runtime_entrypoint_paths())
    );

    let schemas = schemas::schema_catalog();
    assert_eq!(
        schemas["schemaVersion"],
        json!(SCHEMA_CATALOG_SCHEMA_VERSION)
    );
    assert_eq!(schemas["rawExport"], raw_export);
    assert_eq!(schemas["distData"], dist_data);
}

#[test]
fn manifest_collection_catalog_is_descriptor_owned() {
    validate_manifest_collection_descriptors();
    let catalog = manifest_collection_catalog();
    let collections = catalog["collections"].as_array().unwrap();

    assert_eq!(
        catalog["schemaVersion"],
        json!(MANIFEST_COLLECTION_CATALOG_SCHEMA_VERSION)
    );
    assert_eq!(
        catalog["schemaHashInput"],
        json!(SCHEMA_HASH_MANIFEST_COLLECTION_INPUT)
    );
    assert_eq!(collections.len(), MANIFEST_COLLECTION_DESCRIPTORS.len());
    assert!(collections.iter().any(|collection| {
        collection["id"] == json!(COLLECTION_BROWSER_ITEMS.id)
            && collection["logicalNames"] == json!(COLLECTION_BROWSER_ITEMS.logical_names)
            && collection["arrayFields"] == json!(COLLECTION_BROWSER_ITEMS.array_fields)
    }));
    assert!(collections.iter().any(|collection| {
        collection["id"] == json!(COLLECTION_TEXTURE_ROWS_WITH_MANIFEST.id)
            && collection["logicalNames"]
                == json!(COLLECTION_TEXTURE_ROWS_WITH_MANIFEST.logical_names)
            && collection["arrayFields"] == json!(["textures"])
    }));
    assert!(collections.iter().any(|collection| {
        collection["id"] == json!(COLLECTION_HANDLER_LAYOUTS.id)
            && collection["arrayFields"] == json!(["handler-layouts"])
    }));
    assert!(collections.iter().any(|collection| {
        collection["id"] == json!(COLLECTION_FACADE_RESOLUTIONS.id)
            && collection["logicalNames"] == json!(["facadeResolutions"])
    }));
    assert!(collections.iter().any(|collection| {
        collection["id"] == json!(COLLECTION_ANIMATION_FRAME_MATERIALIZATIONS.id)
            && collection["logicalNames"] == json!(["animationFrameMaterializations"])
    }));

    let raw_export = raw_export_schema_section();
    assert_eq!(raw_export["manifest"]["collections"], catalog);
}

#[test]
fn runtime_manifest_report_catalog_is_descriptor_owned() {
    validate_runtime_manifest_report_descriptors();

    let paths = runtime_manifest_paths_catalog();
    let schemas = runtime_report_schemas_catalog();
    let catalog = runtime_manifest_abi_catalog();

    assert_eq!(catalog["paths"], paths);
    assert_eq!(catalog["reportSchemas"], schemas);
    assert_eq!(
        RUNTIME_MANIFEST_REPORT_DESCRIPTORS.len(),
        schemas.as_object().unwrap().len()
    );

    for descriptor in RUNTIME_MANIFEST_REPORT_DESCRIPTORS {
        assert_eq!(paths[descriptor.key], json!(descriptor.path));
        assert_eq!(schemas[descriptor.key], json!(descriptor.schema_version));
        assert!(descriptor.path.starts_with("rust/"));
        assert!(descriptor.path.ends_with(".json"));
    }

    assert_eq!(
        paths["runtimeManifest"],
        json!("rust/runtime-manifest.json")
    );
    assert_eq!(
        schemas["migrationReadiness"],
        json!("neonei/rust-migration-readiness/current")
    );
}

#[test]
fn runtime_report_payload_policy_is_descriptor_owned() {
    validate_runtime_report_payload_policy();
    let policy = runtime_report_payload_policy_catalog();
    let catalog = runtime_manifest_abi_catalog();

    assert_eq!(catalog["reportPayloadPolicy"], policy);
    assert_eq!(
        policy["schemaHashInput"],
        json!(SCHEMA_HASH_RUNTIME_REPORT_PAYLOAD_POLICY_INPUT)
    );
    assert_eq!(policy["generatedAt"], json!(RUST_RUNTIME_GENERATED_AT));
    assert_eq!(
        policy["integrityAlgorithm"],
        json!(RUST_RUNTIME_INTEGRITY_ALGORITHM)
    );
    assert_eq!(
        policy["nativeRuntime"]["status"],
        json!(NATIVE_RUNTIME_STATUS_READY)
    );
    assert_eq!(
        policy["nativeRuntime"]["authority"],
        json!(NATIVE_RUNTIME_AUTHORITY_RUST)
    );
    assert_eq!(
        policy["runtimeManifestPathPolicy"],
        runtime_manifest_path_policy()
    );
}

#[test]
fn runtime_report_emission_uses_manifest_descriptors() {
    let output = tempfile::tempdir().unwrap();
    for spec in runtime_pack_artifact_specs(CompileScope::Search, false) {
        let path = output.path().join(spec.relative_path);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, b"runtime-report-descriptor-artifact").unwrap();
    }
    let pack_validation = output.path().join(PACK_ABI_VALIDATION_REPORT_PATH);
    fs::create_dir_all(pack_validation.parent().unwrap()).unwrap();
    fs::write(
        pack_validation,
        b"runtime-report-descriptor-pack-validation",
    )
    .unwrap();

    runtime::compile_runtime_reports(output.path(), CompileScope::Search, true, false).unwrap();

    let integrity = read_fixture_json(output.path().join("rust/integrity.json"));
    assert_eq!(
        integrity["algorithm"],
        json!(RUST_RUNTIME_INTEGRITY_ALGORITHM)
    );

    let runtime_manifest = read_fixture_json(output.path().join("rust/runtime-manifest.json"));
    assert_eq!(
        runtime_manifest["generatedAt"],
        json!(RUST_RUNTIME_GENERATED_AT)
    );
    assert_eq!(
        runtime_manifest["pathPolicy"],
        runtime_manifest_path_policy()
    );

    let deployment = read_fixture_json(output.path().join("rust/deployment-report.json"));
    assert_eq!(deployment["generatedAt"], json!(RUST_RUNTIME_GENERATED_AT));
    assert_eq!(
        deployment["cache"]["cacheKeyInputs"]["integrityAlgorithm"],
        json!(RUST_RUNTIME_INTEGRITY_ALGORITHM)
    );

    for descriptor in RUNTIME_MANIFEST_REPORT_DESCRIPTORS {
        let report = read_fixture_json(output.path().join(descriptor.path));
        assert_eq!(
            report["schemaVersion"],
            json!(descriptor.schema_version),
            "runtime report schema drift for {}",
            descriptor.key
        );
    }

    let manifest = read_fixture_json(output.path().join("manifest.json"));
    assert_eq!(
        manifest["nativeRuntime"]["status"],
        json!(NATIVE_RUNTIME_STATUS_READY)
    );
    assert_eq!(
        manifest["nativeRuntime"]["authority"],
        json!(NATIVE_RUNTIME_AUTHORITY_RUST)
    );
    for descriptor in RUNTIME_MANIFEST_REPORT_DESCRIPTORS {
        assert!(manifest["files"]
            .as_object()
            .unwrap()
            .values()
            .any(|value| value == &json!(descriptor.path)));
    }
}

#[test]
fn runtime_artifact_catalog_drives_schema_catalog_and_summary_surfaces() {
    validate_runtime_artifact_catalog_descriptors();
    let catalog = runtime_artifact_catalog();
    let artifacts = catalog["artifacts"].as_array().unwrap();
    assert_eq!(
        catalog["schemaVersion"],
        json!(RUNTIME_ARTIFACT_CATALOG_SCHEMA_VERSION)
    );
    assert_eq!(
        catalog["schemaHashInput"],
        json!(SCHEMA_HASH_RUNTIME_ARTIFACT_CATALOG_INPUT)
    );
    assert_eq!(catalog["producers"], json!(RUNTIME_PACK_PRODUCER_IDS));
    assert_eq!(artifacts.len(), runtime_artifact_catalog_specs().len());
    assert!(artifacts.iter().any(|artifact| {
        artifact["logicalName"] == json!("rustPackValidationReport")
            && artifact["path"] == json!(PACK_ABI_VALIDATION_REPORT_PATH)
            && artifact["kind"] == json!("report")
    }));
    assert!(artifacts.iter().any(|artifact| {
        artifact["logicalName"] == json!("rustUiTemplatesBin")
            && artifact["path"] == json!("rust/ui-pack/ui_templates.bin")
            && artifact["producers"] == json!(["ui"])
            && artifact["scopes"]
                .as_array()
                .unwrap()
                .contains(&json!("native-ui"))
    }));
    assert!(artifacts.iter().any(|artifact| {
        artifact["logicalName"] == json!("rustSearchBin")
            && artifact["producers"] == json!(["browser", "search"])
    }));

    let schemas = schemas::schema_catalog();
    assert_eq!(
        schemas["distData"]["runtimeArtifacts"]["schemaVersion"],
        json!(RUNTIME_ARTIFACT_CATALOG_SCHEMA_VERSION)
    );
    assert_eq!(
        schemas["distData"]["packValidationReport"],
        json!(PACK_ABI_VALIDATION_SCHEMA_VERSION)
    );
    assert!(schemas["distData"]["runtimeEntrypoints"]
        .as_array()
        .unwrap()
        .contains(&json!("rust/ui-pack/ui_templates.bin")));

    let output = tempfile::tempdir().unwrap();
    for relative_path in [
        "manifest.json",
        PACK_ABI_VALIDATION_REPORT_PATH,
        "rust/ui-pack/ui_templates.bin",
    ] {
        let path = output.path().join(relative_path);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, b"catalog-owned-artifact").unwrap();
    }
    let summary = reports::summarize_runtime_output(Some(output.path())).unwrap();
    assert!(summary.sizes.contains_key("manifest"));
    assert!(summary.sizes.contains_key("rustPackValidationReport"));
    assert!(summary.sizes.contains_key("rustUiTemplatesBin"));
    assert!(runtime_summary_artifact_specs()
        .iter()
        .any(|(_, path)| *path == PACK_ABI_VALIDATION_REPORT_PATH));
}

#[test]
fn ui_pack_abi_validation_blocks_corrupt_template_binary() {
    let output = compile_fixture("raw-export-minimal", CompileScope::NativeUi, true);
    let template_path = output.path().join("rust/ui-pack/ui_templates.bin");
    let mut bytes = fs::read(&template_path).unwrap();
    let payload_magic_offset = 24 + "neonei/ui-template-pack/current".len();
    bytes[payload_magic_offset] = b'X';
    fs::write(&template_path, bytes).unwrap();

    let report = ui_pack_abi::validate_ui_pack_abi(output.path(), CompileScope::NativeUi).unwrap();
    assert_eq!(report.status, "blocked");
    assert!(report
        .section_violations
        .iter()
        .any(|violation| violation.contains("ui_templates.bin")
            && violation.contains("payload magic mismatch")));
}

#[test]
fn native_ui_export_abi_validation_blocks_blocked_nesql_report() {
    let raw = tempfile::tempdir().unwrap();
    fs::create_dir_all(raw.path().join("validation")).unwrap();
    write_json_value(
        &raw.path().join("manifest.json"),
        &json!({
            "schemaVersion": "neonei/raw-export-fixture/v1",
            "files": {
                "nativeUiValidation": "validation/native-ui-abi.json"
            }
        }),
    )
    .unwrap();
    write_json_value(
        &raw.path().join("validation/native-ui-abi.json"),
        &json!({
            "schemaVersion": "nesqlpp/raw-export/alpha1/native-ui-validation",
            "status": "blocked",
            "layoutCount": 1,
            "slotCount": 1,
            "missingSurfaceCount": 1,
            "slotBoundsViolationCount": 0,
            "backgroundBoundsViolationCount": 0,
            "coordinateContractViolationCount": 0,
            "missingSurfaceSamples": ["missing-background:gregtech"]
        }),
    )
    .unwrap();

    let report = native_ui_export_abi::validate_native_ui_export_abi(raw.path()).unwrap();
    assert_eq!(report.status, "blocked");
    assert_eq!(report.raw_report_status.as_deref(), Some("blocked"));
    assert_eq!(report.missing_surface_count, 1);
    assert_eq!(report.policy.legacy_fallback, "forbidden");
    assert!(report
        .contract_violations
        .iter()
        .any(|violation| violation.contains("missingSurfaceCount")));
    assert_eq!(
        report.samples.missing_surface,
        vec!["missing-background:gregtech".to_string()]
    );
}

#[test]
fn raw_export_abi_validation_blocks_nonportable_manifest_paths() {
    let raw = tempfile::tempdir().unwrap();
    write_json_value(
        &raw.path().join("manifest.json"),
        &json!({
            "schemaVersion": "neonei/raw-export-fixture/v1",
            "files": {
                "items": "../items.jsonl",
                "fluids": "fluids.jsonl",
                "recipeIndex": "C:\\bad\\recipe-index.json",
                "browserAtlasIndex": "textures/browser-atlas-index.json"
            }
        }),
    )
    .unwrap();
    fs::write(raw.path().join("fluids.jsonl"), b"").unwrap();
    fs::create_dir_all(raw.path().join("textures")).unwrap();
    write_json_value(
        &raw.path().join("textures/browser-atlas-index.json"),
        &json!({"items": []}),
    )
    .unwrap();

    let report = raw_export_abi::validate_raw_export_abi(raw.path()).unwrap();
    assert_eq!(report.status, "blocked");
    assert!(report.missing_required_files.contains(&"items".to_string()));
    assert!(report
        .missing_required_files
        .contains(&"recipeIndex".to_string()));
    assert_eq!(report.path_violations.len(), 2);
    assert_eq!(report.policy.legacy_fallback, "forbidden");
}

#[test]
fn raw_export_abi_classifies_manifest_directory_without_generation_drift() {
    let raw = tempfile::tempdir().unwrap();
    fs::create_dir_all(raw.path().join("directory-entry")).unwrap();
    for path in [
        "items.jsonl",
        "fluids.jsonl",
        "recipes/recipe-index.json",
        "textures/browser-atlas-index.json",
    ] {
        let path = raw.path().join(path);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, b"{}\n").unwrap();
    }
    write_json_value(
        &raw.path().join("manifest.json"),
        &json!({
            "schemaVersion": "neonei/raw-export-fixture/v1",
            "files": {
                "items": "items.jsonl",
                "fluids": "fluids.jsonl",
                "recipeIndex": "recipes/recipe-index.json",
                "browserAtlasIndex": "textures/browser-atlas-index.json",
                "directoryFixture": "directory-entry"
            }
        }),
    )
    .unwrap();

    let report = raw_export_abi::validate_raw_export_abi(raw.path()).unwrap();
    assert_eq!(report.status, "blocked");
    assert!(report.missing_required_files.is_empty());
    assert_eq!(report.path_violations.len(), 1);
    let record = report
        .files
        .iter()
        .find(|record| record.logical_name == "directoryFixture")
        .unwrap();
    assert_eq!(record.status, "directory");
    assert_eq!(record.kind, Some("directory"));
    assert_eq!(record.bytes, None);
    assert_eq!(record.sha256, None);
    assert_eq!(record.row_count, None);
}

#[cfg(unix)]
#[test]
fn raw_export_abi_classifies_unsupported_manifest_entry() {
    use std::os::unix::net::UnixListener;

    let raw = tempfile::tempdir().unwrap();
    for path in [
        "items.jsonl",
        "fluids.jsonl",
        "recipes/recipe-index.json",
        "textures/browser-atlas-index.json",
    ] {
        let path = raw.path().join(path);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, b"{}\n").unwrap();
    }
    let socket_path = raw.path().join("unsupported.sock");
    let _listener = UnixListener::bind(&socket_path).unwrap();
    write_json_value(
        &raw.path().join("manifest.json"),
        &json!({
            "schemaVersion": "neonei/raw-export-fixture/v1",
            "files": {
                "items": "items.jsonl",
                "fluids": "fluids.jsonl",
                "recipeIndex": "recipes/recipe-index.json",
                "browserAtlasIndex": "textures/browser-atlas-index.json",
                "unsupportedFixture": "unsupported.sock"
            }
        }),
    )
    .unwrap();

    let report = raw_export_abi::validate_raw_export_abi(raw.path()).unwrap();
    assert_eq!(report.status, "blocked");
    assert!(report.missing_required_files.is_empty());
    assert_eq!(report.path_violations.len(), 1);
    let record = report
        .files
        .iter()
        .find(|record| record.logical_name == "unsupportedFixture")
        .unwrap();
    assert_eq!(record.status, "unsupported");
    assert_eq!(record.kind, Some("unsupported"));
    assert_eq!(record.bytes, None);
    assert_eq!(record.sha256, None);
    assert_eq!(record.row_count, None);
}

#[test]
fn strict_compile_validates_raw_export_abi_before_pack_emission() {
    let raw = tempfile::tempdir().unwrap();
    let output = tempfile::tempdir().unwrap();
    let receipt = tempfile::tempdir().unwrap();
    let report = receipt.path().join("compiler-report.json");
    write_json_value(
        &raw.path().join("manifest.json"),
        &json!({
            "schemaVersion": "neonei/raw-export-fixture/v1",
            "files": {
                "fluids": "fluids.jsonl",
                "recipeIndex": "recipes/recipe-index.json",
                "browserAtlasIndex": "browser-atlas-index.json"
            }
        }),
    )
    .unwrap();

    let error = run_command(Cli {
        command: Command::Compile {
            input: raw.path().to_path_buf(),
            output: output.path().to_path_buf(),
            report,
            scope: CompileScope::NativeUi,
            threads: Some(1),
            strict: true,
            debug_json: false,
        },
    })
    .expect_err("strict compile must fail before pack emission when raw export ABI is invalid");

    let message = format!("{error:#}");
    assert!(message.contains("raw export ABI validation blocked"));
    assert!(!output.path().join("current.json").exists());
    assert_eq!(
        fs::read_dir(output.path().join("generations"))
            .unwrap()
            .count(),
        0
    );
}

#[test]
fn failed_compile_preserves_current_pointer_and_generation_tree_exactly() {
    let output = tempfile::tempdir().unwrap();
    let receipt = tempfile::tempdir().unwrap();
    let report = receipt.path().join("compiler-report.json");
    run_command(Cli {
        command: Command::Compile {
            input: compiler_fixture_path("raw-export-minimal"),
            output: output.path().to_path_buf(),
            report: report.clone(),
            scope: CompileScope::NativeUi,
            threads: Some(1),
            strict: true,
            debug_json: false,
        },
    })
    .unwrap();

    let pointer_before = fs::read(output.path().join("current.json")).unwrap();
    let current_before =
        crate::output_generation::resolve_current_generation(output.path()).unwrap();
    let generation_path_before = current_before.path().to_path_buf();
    let seal_before = current_before.seal().clone();

    let invalid = tempfile::tempdir().unwrap();
    write_json_value(
        &invalid.path().join("manifest.json"),
        &json!({
            "schemaVersion": "neonei/raw-export-fixture/v1",
            "files": {}
        }),
    )
    .unwrap();
    let error = run_command(Cli {
        command: Command::Compile {
            input: invalid.path().to_path_buf(),
            output: output.path().to_path_buf(),
            report,
            scope: CompileScope::NativeUi,
            threads: Some(1),
            strict: true,
            debug_json: false,
        },
    })
    .unwrap_err();
    assert!(
        format!("{error:#}").contains("raw export ABI validation blocked"),
        "{error:#}"
    );

    assert_eq!(
        fs::read(output.path().join("current.json")).unwrap(),
        pointer_before
    );
    let current_after =
        crate::output_generation::resolve_current_generation(output.path()).unwrap();
    assert_eq!(current_after.path(), generation_path_before);
    assert_eq!(current_after.seal(), &seal_before);
    let generation_entries = fs::read_dir(output.path().join("generations"))
        .unwrap()
        .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
        .collect::<Vec<_>>();
    assert_eq!(generation_entries.len(), 1, "{generation_entries:?}");
    assert!(!generation_entries[0].ends_with(".staging"));
}

#[test]
fn strict_compile_validates_native_ui_export_abi_before_pack_emission() {
    let raw = tempfile::tempdir().unwrap();
    let output = tempfile::tempdir().unwrap();
    let receipt = tempfile::tempdir().unwrap();
    let report = receipt.path().join("compiler-report.json");
    fs::create_dir_all(raw.path().join("recipes")).unwrap();
    fs::create_dir_all(raw.path().join("textures")).unwrap();
    fs::create_dir_all(raw.path().join("validation")).unwrap();
    fs::write(raw.path().join("items.jsonl"), b"").unwrap();
    fs::write(raw.path().join("fluids.jsonl"), b"").unwrap();
    write_json_value(&raw.path().join("recipes/recipe-index.json"), &json!({})).unwrap();
    write_json_value(
        &raw.path().join("textures/browser-atlas-index.json"),
        &json!({}),
    )
    .unwrap();
    write_json_value(
        &raw.path().join("manifest.json"),
        &json!({
            "schemaVersion": "neonei/raw-export-fixture/v1",
            "files": {
                "items": "items.jsonl",
                "fluids": "fluids.jsonl",
                "recipeIndex": "recipes/recipe-index.json",
                "browserAtlasIndex": "textures/browser-atlas-index.json",
                "nativeUiValidation": "validation/native-ui-abi.json"
            }
        }),
    )
    .unwrap();
    write_json_value(
        &raw.path().join("validation/native-ui-abi.json"),
        &json!({
            "schemaVersion": "nesqlpp/raw-export/alpha1/native-ui-validation",
            "status": "blocked",
            "layoutCount": 1,
            "slotCount": 1,
            "missingSurfaceCount": 0,
            "slotBoundsViolationCount": 0,
            "backgroundBoundsViolationCount": 0,
            "coordinateContractViolationCount": 1,
            "coordinateContractSamples": ["surface-contract:gregtech"]
        }),
    )
    .unwrap();

    let error = run_command(Cli {
        command: Command::Compile {
            input: raw.path().to_path_buf(),
            output: output.path().to_path_buf(),
            report,
            scope: CompileScope::NativeUi,
            threads: Some(1),
            strict: true,
            debug_json: false,
        },
    })
    .expect_err(
        "strict compile must fail before pack emission when native UI ABI report is blocked",
    );

    let message = format!("{error:#}");
    assert!(message.contains("native UI export ABI validation blocked"));
    assert!(!output.path().join("current.json").exists());
    assert_eq!(
        fs::read_dir(output.path().join("generations"))
            .unwrap()
            .count(),
        0
    );
}

#[test]
fn stable_cli_inspect_validate_and_schemas_cover_fixture_contracts() {
    let temp = tempfile::tempdir().unwrap();
    let raw = compiler_fixture_path("raw-export-minimal");
    let inspect_report = temp.path().join("inspect-report.json");
    let validate_report = temp.path().join("validate-report.json");
    let schema_catalog = temp.path().join("schemas/catalog.json");

    run_command(Cli {
        command: Command::Inspect {
            input: raw.clone(),
            report: inspect_report.clone(),
        },
    })
    .unwrap();
    run_command(Cli {
        command: Command::Validate {
            input: raw,
            report: validate_report.clone(),
            output: None,
        },
    })
    .unwrap();
    run_command(Cli {
        command: Command::Schemas {
            output: Some(schema_catalog.clone()),
        },
    })
    .unwrap();

    let inspect: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(inspect_report).unwrap()).unwrap();
    let validate: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(validate_report).unwrap()).unwrap();
    let schemas: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(schema_catalog).unwrap()).unwrap();

    assert_eq!(inspect["mode"], json!("inspect"));
    assert_eq!(validate["mode"], json!("validate"));
    assert!(validate["blocked"].as_array().unwrap().is_empty());
    assert_eq!(
        schemas["schemaVersion"],
        json!("elysium-compiler/schema-catalog/v1")
    );
    assert!(schemas["compiler"]["cli"]["commands"]
        .as_array()
        .unwrap()
        .contains(&json!("schemas")));
    assert_eq!(
        schemas["compiler"]["cli"]["commands"],
        json!(COMPILER_COMMANDS)
    );
    assert_eq!(
        schemas["compiler"]["cli"]["commandCatalog"]["schemaVersion"],
        json!(COMPILER_COMMAND_CATALOG_SCHEMA_VERSION)
    );
    assert_eq!(
        schemas["compiler"]["cli"]["commandCatalog"]["dispatchPolicy"],
        json!("static-command-descriptor-ops-table")
    );
    assert_eq!(
        schemas["compiler"]["cli"]["commandCatalog"]["commands"],
        schemas["abi"]["compilerCapabilityAbi"]["commandCatalog"]["commands"]
    );
    assert!(schemas["compiler"]["cli"]["commandCatalog"]["commands"]
        .as_array()
        .unwrap()
        .iter()
        .any(|command| command["name"] == json!("compile")
            && command["capabilities"]
                .as_array()
                .unwrap()
                .contains(&json!("compiler.compile_kernel"))));
    assert_eq!(
        schemas["compiler"]["cli"]["compileScopes"],
        json!(COMPILE_SCOPES)
    );
    assert_eq!(
        schemas["compiler"]["cli"]["compileScopeCatalog"]["schemaVersion"],
        json!(COMPILE_SCOPE_CATALOG_SCHEMA_VERSION)
    );
    assert_eq!(
        schemas["compiler"]["cli"]["compileScopeCatalog"]["selectionPolicy"],
        json!("static-compile-scope-descriptor-table")
    );
    assert_eq!(
        schemas["compiler"]["cli"]["compileScopeCatalog"]["scopes"],
        schemas["abi"]["compilerCapabilityAbi"]["compileScopeCatalog"]["scopes"]
    );
    assert!(schemas["compiler"]["cli"]["compileScopeCatalog"]["scopes"]
        .as_array()
        .unwrap()
        .iter()
        .any(|scope| scope["name"] == json!("native-ui")
            && scope["runtimePackProducers"]
                .as_array()
                .unwrap()
                .contains(&json!("ui"))
            && scope["runtimeCapabilities"]
                .as_array()
                .unwrap()
                .contains(&json!("recipes.ui-pack"))));
    assert!(schemas["compiler"]["cli"]["compileScopeCatalog"]["scopes"]
        .as_array()
        .unwrap()
        .iter()
        .any(|scope| scope["name"] == json!("ui")
            && scope["policy"]
                == json!("emit recipe artifacts plus native UI binary and sidecar packs")));
    assert_eq!(
        schemas["compiler"]["compileKernel"]["schemaVersion"],
        json!("elysium-compiler/compile-kernel-catalog/v1")
    );
    assert_eq!(
        schemas["compiler"]["compileKernel"]["trace"]["schemaVersion"],
        json!(COMPILE_KERNEL_TRACE_SCHEMA_VERSION)
    );
    assert_eq!(
        schemas["compiler"]["compileKernel"]["trace"]["path"],
        json!(COMPILE_KERNEL_TRACE_REPORT_PATH)
    );
    assert_eq!(
        schemas["compiler"]["compileKernel"]["moduleCount"],
        json!(compile_kernel_modules().len())
    );
    let catalog_stage_count: usize = compile_kernel_modules()
        .iter()
        .map(|module| module.stages().len())
        .sum();
    assert_eq!(
        schemas["compiler"]["compileKernel"]["stageCount"],
        json!(catalog_stage_count)
    );
    assert_eq!(
        schemas["compiler"]["compileKernel"]["runtimePackCompilerPolicy"]["selection"],
        json!("descriptor-scope-table")
    );
    assert!(
        schemas["compiler"]["compileKernel"]["runtimePackCompilerPolicy"]["compilers"]
            .as_array()
            .unwrap()
            .iter()
            .any(|compiler| compiler["id"] == json!("ui")
                && compiler["capabilities"]
                    .as_array()
                    .unwrap()
                    .contains(&json!("compiler.native_ui_pack")))
    );
    assert!(schemas["compiler"]["compileKernel"]["modules"]
        .as_array()
        .unwrap()
        .iter()
        .any(|module| module["name"] == json!("compiler.runtime_packs")
            && module["stages"].as_array().unwrap().iter().any(|stage| {
                stage["name"] == json!("emit-runtime-packs")
                    && stage["contract"]["capabilities"]
                        .as_array()
                        .unwrap()
                        .contains(&json!("compiler.native_ui_pack"))
            })));
    assert_eq!(
        schemas["abi"]["compilerCapabilityAbi"]["requiredCommands"],
        json!(REQUIRED_COMPILER_COMMANDS)
    );
    assert_eq!(
        schemas["abi"]["compilerCapabilityAbi"]["nativeUi"]["requiredCapabilities"],
        json!(NATIVE_UI_REQUIRED_CAPABILITIES)
    );
    assert_eq!(
        schemas["abi"]["compilerCapabilityAbi"]["nativeUi"]["requiredFiles"],
        json!(NATIVE_UI_REQUIRED_FILES)
    );
    assert_eq!(
        schemas["abi"]["exportAbi"]["nativeUi"]["requiredFiles"],
        json!(NATIVE_UI_REQUIRED_FILES)
    );
    assert_eq!(
        schemas["abi"]["compilerCapabilityAbi"]["nativeUi"]["coordinateSpace"],
        json!(NATIVE_UI_COORDINATE_SPACE)
    );
    assert_eq!(
        schemas["abi"]["compilerCapabilityAbi"]["nativeUi"]["runtimeTransform"],
        json!(NATIVE_UI_RUNTIME_TRANSFORM)
    );
    assert_eq!(
        schemas["abi"]["compilerCapabilityAbi"]["nativeUi"]["fallbackPolicy"],
        json!(NATIVE_UI_FALLBACK_POLICY)
    );
    assert_eq!(
        schemas["abi"]["compilerCapabilityAbi"]["schemaHashInputs"]["compilerCapabilityAbi"],
        json!(SCHEMA_HASH_COMPILER_CAPABILITY_INPUT)
    );
    assert_eq!(
        schemas["abi"]["runtimeManifestAbi"]["schemaVersion"],
        json!(RUST_RUNTIME_MANIFEST_SCHEMA_VERSION)
    );
    assert_eq!(
        schemas["abi"]["runtimeManifestAbi"]["runtimeSchema"],
        json!(RUST_RUNTIME_SCHEMA)
    );
    assert_eq!(
        schemas["abi"]["runtimeManifestAbi"]["schemaRevision"],
        json!(RUST_RUNTIME_SCHEMA_REVISION)
    );
    assert_eq!(
        schemas["abi"]["runtimeManifestAbi"]["schemaHashInput"],
        json!(SCHEMA_HASH_RUNTIME_MANIFEST_INPUT)
    );
    assert!(RUST_RUNTIME_ENTRYPOINTS
        .iter()
        .any(|spec| spec.key == "uiTemplates" && spec.path == "rust/ui-pack/ui_templates.bin"));
    assert_eq!(
        schemas["rawExport"]["nativeBackground"]["strictPolicy"],
        json!(
            "nativeBackground is semantic layout metadata only; background PNG assets are retired and are not materialized"
        )
    );
    assert_eq!(
        schemas["rawExport"]["nativeFrame"]["status"],
        json!("retired")
    );
    assert_eq!(
        schemas["rawExport"]["nativeFrame"]["strictPolicy"],
        json!(
            "nativeFrame PNG capture is retired; recipe UI payloads must not depend on in-game NEI frame assets"
        )
    );
}

#[test]
fn runtime_pack_compiler_catalog_is_scope_authority() {
    fn compiler_ids(scope: CompileScope) -> Vec<&'static str> {
        runtime_pack_compilers(scope)
            .map(|compiler| compiler.id())
            .collect::<Vec<_>>()
    }

    assert_eq!(
        compiler_ids(CompileScope::All),
        vec!["browser", "recipes", "ui", "texture"]
    );
    assert_eq!(
        compiler_ids(CompileScope::NativeUi),
        vec!["browser", "recipes", "ui"]
    );
    assert_eq!(compiler_ids(CompileScope::Search), vec!["search"]);
    assert_eq!(compiler_ids(CompileScope::Browser), vec!["browser"]);
    assert_eq!(compiler_ids(CompileScope::Recipes), vec!["recipes"]);
    assert_eq!(compiler_ids(CompileScope::Ui), vec!["recipes", "ui"]);
    assert_eq!(compiler_ids(CompileScope::Textures), vec!["texture"]);

    let browser = runtime_pack_compilers(CompileScope::Browser)
        .next()
        .unwrap();
    assert!(browser.outputs().contains(&"rust/search.bin"));
    assert!(browser.capabilities().contains(&"compiler.search_pack"));

    let texture = runtime_pack_compilers(CompileScope::Textures)
        .next()
        .unwrap();
    assert!(texture.outputs().contains(&"rust/textures.bin"));
    assert!(texture.outputs().contains(&"rust/atlas.meta.bin"));
    assert!(texture.outputs().contains(&"rust/animations.bin"));
    assert!(!texture.outputs().contains(&"rust/texture.bin"));
    assert!(!texture.outputs().contains(&"rust/atlas_meta.bin"));
    assert!(!texture.outputs().contains(&"rust/animation.bin"));

    let ui = runtime_pack_compilers(CompileScope::Ui)
        .find(|compiler| compiler.id() == "ui")
        .unwrap();
    assert!(ui.inputs().contains(&"raw-export-manifest"));
    assert!(ui.outputs().contains(&"rust/ui-pack/ui_templates.bin"));
    assert!(ui
        .outputs()
        .contains(&"rust/ui-pack/ui_assets.manifest.json"));
    assert!(ui.outputs().contains(&"rust/ui-pack/ui_pack_report.json"));
    let ui_artifact_paths = runtime_pack_artifact_specs(CompileScope::Ui, false)
        .into_iter()
        .map(|spec| spec.relative_path)
        .collect::<BTreeSet<_>>();
    assert!(ui_artifact_paths.contains("rust/recipes.bin"));
    assert!(ui_artifact_paths.contains("rust/ui-pack/ui_templates.bin"));

    let texture = runtime_pack_compilers(CompileScope::Textures)
        .next()
        .unwrap();
    assert!(texture.inputs().contains(&"raw-export-manifest"));
    assert!(ui.debug_outputs().is_empty());
}

#[test]
fn runtime_pack_dag_honors_local_thread_pool_configuration() {
    let global_thread_count = rayon::current_num_threads();
    let single = CompilerThreadPool::new(Some(1)).unwrap();
    assert_eq!(single.thread_count(), 1);
    assert_eq!(single.install(rayon::current_num_threads), 1);

    let parallel = CompilerThreadPool::new(Some(4)).unwrap();
    assert_eq!(parallel.thread_count(), 4);
    assert_eq!(parallel.install(rayon::current_num_threads), 4);
    assert_eq!(rayon::current_num_threads(), global_thread_count);

    let error = CompilerThreadPool::new(Some(0)).unwrap_err().to_string();
    assert!(error.contains("at least 1"), "{error}");
}

#[test]
fn runtime_pack_dag_declares_dependency_order_and_exclusive_ownership() {
    assert_eq!(
        runtime_pack_execution_waves(CompileScope::All).unwrap(),
        vec![vec!["browser", "recipes"], vec!["ui"], vec!["texture"]]
    );
    assert_eq!(
        runtime_pack_execution_waves(CompileScope::NativeUi).unwrap(),
        vec![vec!["browser", "recipes"], vec!["ui"]]
    );
    assert_eq!(
        runtime_pack_execution_waves(CompileScope::Search).unwrap(),
        vec![vec!["search"]]
    );
    assert_eq!(
        runtime_pack_execution_waves(CompileScope::Ui).unwrap(),
        vec![vec!["recipes"], vec!["ui"]]
    );
    assert_eq!(
        runtime_pack_execution_batches(CompileScope::All).unwrap(),
        vec![
            vec!["browser"],
            vec!["recipes"],
            vec!["ui"],
            vec!["texture"]
        ]
    );
    assert_eq!(
        runtime_pack_execution_batches(CompileScope::NativeUi).unwrap(),
        vec![vec!["browser"], vec!["recipes"], vec!["ui"]]
    );

    for scope in [
        CompileScope::All,
        CompileScope::NativeUi,
        CompileScope::Search,
        CompileScope::Browser,
        CompileScope::Recipes,
        CompileScope::Ui,
        CompileScope::Textures,
    ] {
        assert!(
            runtime_pack_ownership_conflicts(scope).is_empty(),
            "runtime pack ownership conflict for {}",
            scope.as_str()
        );
    }

    let ui = runtime_pack_compilers(CompileScope::All)
        .find(|compiler| compiler.id() == "ui")
        .unwrap();
    assert_eq!(ui.dependencies(), &["recipes"]);
    assert!(ui.owned_outputs().contains(&"rust/ui-pack/"));
    let texture = runtime_pack_compilers(CompileScope::All)
        .find(|compiler| compiler.id() == "texture")
        .unwrap();
    assert_eq!(texture.dependencies(), &["ui"]);
}

#[test]
fn runtime_pack_compiler_outputs_are_projected_from_artifact_producers() {
    for scope in [
        CompileScope::All,
        CompileScope::NativeUi,
        CompileScope::Search,
        CompileScope::Browser,
        CompileScope::Recipes,
        CompileScope::Ui,
        CompileScope::Textures,
    ] {
        let active_compilers = runtime_pack_compilers(scope).collect::<Vec<_>>();
        let active_producers = active_compilers
            .iter()
            .map(|compiler| compiler.id())
            .collect::<BTreeSet<_>>();
        let actual_outputs = active_compilers
            .iter()
            .flat_map(|compiler| compiler.outputs())
            .collect::<BTreeSet<_>>();
        let expected_outputs = runtime_pack_artifact_specs(scope, false)
            .into_iter()
            .filter(|spec| {
                spec.producers()
                    .iter()
                    .any(|producer| active_producers.contains(producer))
            })
            .map(|spec| spec.relative_path)
            .collect::<BTreeSet<_>>();

        assert_eq!(
            actual_outputs,
            expected_outputs,
            "runtime pack compiler catalog drift for scope {}",
            scope.as_str()
        );
    }
}

#[test]
fn minimal_native_ui_fixture_compiles_through_stable_cli_boundary() {
    let output = compile_fixture("raw-export-minimal", CompileScope::NativeUi, false);
    let report = output.path().join("compiler-report.json");

    assert!(report.exists());
    assert!(output.path().join("rust/runtime-manifest.json").exists());
    assert!(output.path().join("rust/browser.bin").exists());
    assert!(output.path().join("rust/recipes.bin").exists());
    assert!(output.path().join("rust/ui-pack/ui_templates.bin").exists());
    assert!(output
        .path()
        .join("rust/compile-kernel-trace.json")
        .exists());
    assert!(output
        .path()
        .join("rust/pack-validation-report.json")
        .exists());
    assert!(output
        .path()
        .join("rust/raw-export-abi-validation-report.json")
        .exists());
    assert!(output
        .path()
        .join("rust/native-ui-export-abi-validation-report.json")
        .exists());
    let kernel_trace: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(output.path().join("rust/compile-kernel-trace.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(
        kernel_trace["schemaVersion"],
        json!("elysium-compiler/compile-kernel-trace/v1")
    );
    assert!(kernel_trace["stages"]
        .as_array()
        .unwrap()
        .iter()
        .any(|stage| stage["stage"] == json!("emit-runtime-packs")));
    assert!(kernel_trace["stages"]
        .as_array()
        .unwrap()
        .iter()
        .any(|stage| stage["module"] == json!("compiler.runtime_packs")
            && stage["stage"] == json!("emit-runtime-packs")));
    assert!(kernel_trace["stages"]
        .as_array()
        .unwrap()
        .iter()
        .any(|stage| stage["stage"] == json!("validate-raw-export-abi")
            && stage["contract"]["capabilities"]
                .as_array()
                .unwrap()
                .contains(&json!("compiler.raw_export_abi_validator"))));
    assert!(kernel_trace["stages"]
        .as_array()
        .unwrap()
        .iter()
        .any(
            |stage| stage["stage"] == json!("validate-native-ui-export-abi")
                && stage["module"] == json!("compiler.native_ui_export_abi")
                && stage["contract"]["capabilities"]
                    .as_array()
                    .unwrap()
                    .contains(&json!("compiler.native_ui_export_abi_validator"))
        ));
    assert!(kernel_trace["stages"]
        .as_array()
        .unwrap()
        .iter()
        .any(
            |stage| stage["stage"] == json!("validate-native-ui-pack-abi")
                && stage["contract"]["capabilities"]
                    .as_array()
                    .unwrap()
                    .contains(&json!("compiler.native_ui_pack_abi_validator"))
        ));
    assert!(kernel_trace["stages"]
        .as_array()
        .unwrap()
        .iter()
        .any(|stage| stage["stage"] == json!("validate-pack-abi")
            && stage["contract"]["capabilities"]
                .as_array()
                .unwrap()
                .contains(&json!("compiler.pack_abi_validator"))));
    assert!(!output
        .path()
        .join("assets/ui-backgrounds/gregtech/nei_single_recipe.png")
        .exists());

    let runtime_manifest: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(output.path().join("rust/runtime-manifest.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(runtime_manifest["compileScope"], json!("native-ui"));
    assert_eq!(
        runtime_manifest["schemaVersion"],
        json!(RUST_RUNTIME_MANIFEST_SCHEMA_VERSION)
    );
    assert_eq!(runtime_manifest["schema"], json!(RUST_RUNTIME_SCHEMA));
    assert_eq!(
        runtime_manifest["schemaRevision"],
        json!(RUST_RUNTIME_SCHEMA_REVISION)
    );
    assert!(runtime_manifest["files"]
        .as_array()
        .unwrap()
        .iter()
        .any(|entry| entry["path"] == json!("rust/ui-pack/ui_assets.manifest.json")));
    assert!(runtime_manifest["files"]
        .as_array()
        .unwrap()
        .iter()
        .any(|entry| entry["path"] == json!("rust/ui-pack/ui_template_catalog.json")));
    assert!(runtime_manifest["files"]
        .as_array()
        .unwrap()
        .iter()
        .any(|entry| entry["path"] == json!("rust/ui-pack/ui_template_binding_index.json")));
    assert!(runtime_manifest["files"]
        .as_array()
        .unwrap()
        .iter()
        .any(|entry| entry["path"] == json!("rust/ui-pack/ui_family_census.json")));
    assert!(runtime_manifest["files"]
        .as_array()
        .unwrap()
        .iter()
        .any(|entry| entry["path"] == json!("rust/raw-export-abi-validation-report.json")));
    assert!(runtime_manifest["files"]
        .as_array()
        .unwrap()
        .iter()
        .any(|entry| entry["path"] == json!("rust/native-ui-export-abi-validation-report.json")));
    assert!(runtime_manifest["files"]
        .as_array()
        .unwrap()
        .iter()
        .any(|entry| entry["path"] == json!("rust/ui-pack-abi-validation-report.json")));
    assert!(runtime_manifest["files"]
        .as_array()
        .unwrap()
        .iter()
        .any(|entry| entry["path"] == json!("rust/pack-validation-report.json")));

    let raw_export_abi: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(
            output
                .path()
                .join("rust/raw-export-abi-validation-report.json"),
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(raw_export_abi["status"], json!("ok"));
    assert_eq!(
        raw_export_abi["policy"]["legacyFallback"],
        json!("forbidden")
    );

    let native_ui_export_abi: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(
            output
                .path()
                .join("rust/native-ui-export-abi-validation-report.json"),
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(native_ui_export_abi["status"], json!("ok"));
    assert_eq!(
        native_ui_export_abi["policy"]["legacyFallback"],
        json!("forbidden")
    );

    let ui_pack_abi: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(
            output
                .path()
                .join("rust/ui-pack-abi-validation-report.json"),
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(ui_pack_abi["status"], json!("ok"));
    assert_eq!(ui_pack_abi["policy"]["legacyFallback"], json!("forbidden"));
    assert!(ui_pack_abi["artifacts"]
        .as_array()
        .unwrap()
        .iter()
        .any(
            |entry| entry["path"] == json!("rust/ui-pack/ui_templates.bin")
                && entry["sections"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .any(|section| section["name"] == json!("templates"))
        ));

    let pack_validation: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(output.path().join("rust/pack-validation-report.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(pack_validation["status"], json!("ok"));
    assert_eq!(
        pack_validation["policy"]["legacyFallback"],
        json!("forbidden")
    );

    let ui_pack_report: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(output.path().join("rust/ui-pack/ui_pack_report.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(ui_pack_report["status"], json!("ready"));
    assert_eq!(ui_pack_report["summary"]["templateCount"], json!(29));
    assert_eq!(
        ui_pack_report["assets"]["uiBackgrounds"]["missing"]
            .as_array()
            .unwrap()
            .len(),
        0
    );
    let ui_template_catalog =
        read_fixture_json(output.path().join("rust/ui-pack/ui_template_catalog.json"));
    let ui_binding_index = read_fixture_json(
        output
            .path()
            .join("rust/ui-pack/ui_template_binding_index.json"),
    );
    let ui_family_census =
        read_fixture_json(output.path().join("rust/ui-pack/ui_family_census.json"));
    assert_eq!(
        ui_template_catalog["schemaVersion"],
        json!("neonei/ui-template-catalog/current")
    );
    assert_eq!(
        ui_binding_index["schemaVersion"],
        json!("neonei/ui-template-binding-index/current")
    );
    assert_eq!(
        ui_family_census["schemaVersion"],
        json!("neonei/ui-family-census/current")
    );
}

#[test]
fn compile_kernel_catalog_is_authoritative_for_trace_contracts() {
    let catalog = compile_kernel_catalog();
    let modules = catalog["modules"].as_array().unwrap();
    let descriptor_count: usize = compile_kernel_modules()
        .iter()
        .map(|module| module.stages().len())
        .sum();

    assert_eq!(catalog["stageCount"], json!(descriptor_count));
    assert_eq!(
        catalog["trace"]["schemaVersion"],
        json!(COMPILE_KERNEL_TRACE_SCHEMA_VERSION)
    );
    assert_eq!(
        catalog["trace"]["path"],
        json!(COMPILE_KERNEL_TRACE_REPORT_PATH)
    );

    let stage_pairs = modules
        .iter()
        .flat_map(|module| {
            module["stages"]
                .as_array()
                .unwrap()
                .iter()
                .map(move |stage| (module["name"].clone(), stage.clone()))
        })
        .collect::<Vec<_>>();
    assert_eq!(stage_pairs.len(), descriptor_count);
    assert!(stage_pairs.iter().any(|(module, stage)| {
        module == &json!("compiler.native_ui_export_abi")
            && stage["name"] == json!("validate-native-ui-export-abi")
            && stage["contract"]["outputs"]
                .as_array()
                .unwrap()
                .contains(&json!("rust/native-ui-export-abi-validation-report.json"))
    }));
    assert!(stage_pairs.iter().any(|(module, stage)| {
        module == &json!("compiler.trace")
            && stage["name"] == json!("emit-kernel-trace")
            && stage["contract"]["outputs"]
                .as_array()
                .unwrap()
                .contains(&json!(COMPILE_KERNEL_TRACE_REPORT_PATH))
    }));
}

#[test]
fn scoped_compile_purges_out_of_scope_runtime_artifacts() {
    let output = tempfile::tempdir().unwrap();
    let rust_dir = output.path().join("rust");
    fs::create_dir_all(&rust_dir).unwrap();
    fs::write(rust_dir.join("browser.bin"), b"stale-browser-pack").unwrap();
    let receipt = tempfile::tempdir().unwrap();
    let report = receipt.path().join("compiler-report.json");

    run_command(Cli {
        command: Command::Compile {
            input: compiler_fixture_path("raw-export-minimal"),
            output: output.path().to_path_buf(),
            report,
            scope: CompileScope::Search,
            threads: Some(1),
            strict: true,
            debug_json: false,
        },
    })
    .unwrap();

    let generation = crate::output_generation::resolve_current_generation(output.path()).unwrap();
    assert!(generation.path().join("rust/search.bin").exists());
    assert!(!generation.path().join("rust/browser.bin").exists());
    assert_eq!(
        fs::read(output.path().join("rust/browser.bin")).unwrap(),
        b"stale-browser-pack"
    );
    let pack_validation =
        read_fixture_json(generation.path().join("rust/pack-validation-report.json"));
    assert_eq!(pack_validation["status"], json!("ok"));
    assert!(!pack_validation["artifacts"]
        .as_array()
        .unwrap()
        .iter()
        .any(|entry| entry["path"] == json!("rust/browser.bin")));
    let ui_pack_validation = read_fixture_json(
        generation
            .path()
            .join("rust/ui-pack-abi-validation-report.json"),
    );
    assert_eq!(ui_pack_validation["status"], json!("skipped"));
}

#[test]
fn native_ui_gt_fixture_matches_expected_reports_without_background_png_asset() {
    let output = compile_fixture("raw-export-native-ui-gt", CompileScope::NativeUi, true);
    for relative_path in [
        "recipes/ui-payload-index.json",
        "rust/runtime-manifest.json",
        "rust/native-ui-layout-report.json",
        "rust/ui-pack/ui_pack_report.json",
        "rust/ui-pack/ui_assets.manifest.json",
        "rust/ui-pack/ui_template_catalog.json",
        "rust/ui-pack/ui_template_binding_index.json",
        "rust/ui-pack/ui_family_census.json",
        "rust/integrity.json",
        "rust/raw-export-abi-validation-report.json",
        "rust/native-ui-export-abi-validation-report.json",
        "rust/ui-pack-abi-validation-report.json",
        "rust/pack-validation-report.json",
    ] {
        assert_expected_json_matches("raw-export-native-ui-gt", output.path(), relative_path);
    }
    assert!(!output
        .path()
        .join("assets/ui-backgrounds/gregtech/nei_single_recipe.png")
        .is_file());
}

#[test]
fn semantic_background_only_fixture_compiles_without_materialized_asset() {
    let output = compile_fixture(
        "raw-export-semantic-background-only",
        CompileScope::NativeUi,
        true,
    );
    let layout_report = read_fixture_json(output.path().join("rust/native-ui-layout-report.json"));
    let ui_assets = read_fixture_json(output.path().join("rust/ui-pack/ui_assets.manifest.json"));
    let ui_pack_report = read_fixture_json(output.path().join("rust/ui-pack/ui_pack_report.json"));

    assert_eq!(layout_report["backgroundStatus"], json!("semantic"));
    assert_eq!(layout_report["status"], json!("ready"));
    assert!(ui_assets["assets"].as_array().unwrap().is_empty());
    assert!(ui_pack_report["assets"]["uiBackgrounds"]["missing"]
        .as_array()
        .unwrap()
        .is_empty());
    for relative_path in [
        "recipes/ui-payload-index.json",
        "rust/runtime-manifest.json",
        "rust/native-ui-layout-report.json",
        "rust/ui-pack/ui_pack_report.json",
        "rust/ui-pack/ui_assets.manifest.json",
        "rust/ui-pack/ui_template_catalog.json",
        "rust/ui-pack/ui_template_binding_index.json",
        "rust/ui-pack/ui_family_census.json",
        "rust/integrity.json",
        "rust/raw-export-abi-validation-report.json",
        "rust/native-ui-export-abi-validation-report.json",
        "rust/ui-pack-abi-validation-report.json",
        "rust/pack-validation-report.json",
    ] {
        assert_expected_json_matches(
            "raw-export-semantic-background-only",
            output.path(),
            relative_path,
        );
    }
}

#[test]
fn sharded_recipes_fixture_compiles_all_declared_shards() {
    let output = compile_fixture("raw-export-sharded-recipes", CompileScope::NativeUi, true);
    let ui_payload_index = read_fixture_json(output.path().join("recipes/ui-payload-index.json"));
    let recipes = ui_payload_index["recipes"].as_array().unwrap();
    assert_eq!(recipes.len(), 2);
    assert!(recipes
        .iter()
        .any(|entry| entry["recipeId"] == json!("r_fixture_shard_a")));
    assert!(recipes
        .iter()
        .any(|entry| entry["recipeId"] == json!("r_fixture_shard_b")));
    assert!(recipes
        .iter()
        .all(|entry| entry["nativeLayout"].get("progressBars").is_none()
            && entry["nativeLayout"].get("dynamicPrimitives").is_none()
            && entry["nativeLayout"].get("fluidBars").is_none()
            && entry["nativeLayout"].get("energyBars").is_none()));
    assert_expected_json_matches(
        "raw-export-sharded-recipes",
        output.path(),
        "recipes/ui-payload-index.json",
    );
    for relative_path in [
        "rust/ui-pack/ui_template_catalog.json",
        "rust/ui-pack/ui_template_binding_index.json",
        "rust/ui-pack/ui_family_census.json",
        "rust/raw-export-abi-validation-report.json",
        "rust/native-ui-export-abi-validation-report.json",
        "rust/ui-pack-abi-validation-report.json",
        "rust/pack-validation-report.json",
    ] {
        assert_expected_json_matches("raw-export-sharded-recipes", output.path(), relative_path);
    }
}

#[test]
fn texture_atlas_fixture_materializes_runtime_atlas_without_missing_refs() {
    let output = compile_fixture("raw-export-texture-atlas", CompileScope::All, true);
    let missing_texture_report =
        read_fixture_json(output.path().join("rust/missing-texture-report.json"));
    assert_eq!(missing_texture_report["status"], json!("ok"));
    assert_eq!(
        missing_texture_report["counts"]["missingAtlasAssetFiles"],
        json!(0)
    );
    assert!(output
        .path()
        .join("textures/atlas/static-fixture.webp")
        .is_file());
    for relative_path in [
        "recipes/ui-payload-index.json",
        "rust/runtime-manifest.json",
        "rust/missing-texture-report.json",
        "rust/suspicious-texture-report.json",
        "rust/ui-pack/ui_template_catalog.json",
        "rust/ui-pack/ui_template_binding_index.json",
        "rust/ui-pack/ui_family_census.json",
        "rust/raw-export-abi-validation-report.json",
        "rust/native-ui-export-abi-validation-report.json",
        "rust/ui-pack-abi-validation-report.json",
    ] {
        assert_expected_json_matches("raw-export-texture-atlas", output.path(), relative_path);
    }
}

#[test]
fn missing_captured_ui_background_fixture_compiles_without_png_materialization() {
    let output = tempfile::tempdir().unwrap();
    let receipt = tempfile::tempdir().unwrap();
    let report = receipt.path().join("compiler-report.json");
    let raw = compiler_fixture_path("raw-export-missing-background-should-fail");

    run_command(Cli {
        command: Command::Compile {
            input: raw,
            output: output.path().to_path_buf(),
            report,
            scope: CompileScope::NativeUi,
            threads: Some(1),
            strict: true,
            debug_json: false,
        },
    })
    .unwrap();

    let generation = crate::output_generation::resolve_current_generation(output.path()).unwrap();
    assert!(generation
        .path()
        .join("rust/runtime-manifest.json")
        .exists());
    assert!(!generation.path().join("assets/ui-backgrounds").exists());
}
