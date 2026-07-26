use crate::diagnostics::{run_diagnostics_report_with_session, DiagnosticsMode};
use crate::manifest::{COLLECTION_ANIMATION_FRAME_MATERIALIZATIONS, COLLECTION_FACADE_RESOLUTIONS};
use crate::session::RawExportSession;
use crate::validation::compile_semantic_validation_report_with_session;
use flate2::write::GzEncoder;
use flate2::Compression;
use serde_json::json;
use std::fs;
use std::io::Write;
use std::path::Path;
use std::sync::Arc;

fn write_session_fixture() -> tempfile::TempDir {
    let root = tempfile::tempdir().expect("create session fixture");
    fs::create_dir_all(root.path().join("recipes")).expect("create recipes directory");
    fs::write(
        root.path().join("recipes/shard-0.jsonl"),
        b"{\"recipeId\":\"a\"}\n{\"recipeId\":\"b\"}\n",
    )
    .expect("write JSONL shard");
    fs::write(
        root.path().join("manifest.json"),
        serde_json::to_vec_pretty(&json!({
            "schemaVersion": "nesqlpp/raw-export/v1",
            "files": {
                "recipeShard": "recipes/shard-0.jsonl"
            }
        }))
        .expect("serialize manifest"),
    )
    .expect("write manifest");
    root
}

fn write_gzip_jsonl(path: &Path, rows: &[serde_json::Value]) {
    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    for row in rows {
        serde_json::to_writer(&mut encoder, row).expect("serialize gzip JSONL row");
        encoder.write_all(b"\n").expect("write gzip JSONL newline");
    }
    fs::write(path, encoder.finish().expect("finish gzip JSONL")).expect("write gzip JSONL");
}

#[test]
fn authoritative_resource_fact_collections_read_jsonl_gzip_from_manifest() {
    let root = tempfile::tempdir().expect("create resource fact fixture");
    write_gzip_jsonl(
        &root.path().join("facade-resolutions.jsonl.gz"),
        &[json!({
            "facadeItemId": "facade:item",
            "facadeAssetId": "facade:asset",
            "status": "resolved",
            "reason": "resolved",
            "targets": [{ "sourceItemId": "source:item", "sourceAssetId": "source:asset" }]
        })],
    );
    write_gzip_jsonl(
        &root
            .path()
            .join("animation-frame-materializations.jsonl.gz"),
        &[json!({
            "assetId": "animated:asset",
            "runtimeFrameCount": 2,
            "declaredFrameCount": 2,
            "materializedFrameCount": 2,
            "distinctFrameCount": 2,
            "materializationStatus": "materialized",
            "materializationReason": "captured"
        })],
    );
    fs::write(
        root.path().join("manifest.json"),
        serde_json::to_vec_pretty(&json!({
            "schemaVersion": "nesqlpp/raw-export/v1",
            "files": {
                "facadeResolutions": "facade-resolutions.jsonl.gz",
                "animationFrameMaterializations": "animation-frame-materializations.jsonl.gz"
            }
        }))
        .expect("serialize resource fact manifest"),
    )
    .expect("write resource fact manifest");

    let session = RawExportSession::open(root.path()).expect("open resource fact session");
    let facades = session
        .read_manifest_collection(COLLECTION_FACADE_RESOLUTIONS)
        .expect("read facade resolution collection");
    let animations = session
        .read_manifest_collection(COLLECTION_ANIMATION_FRAME_MATERIALIZATIONS)
        .expect("read animation materialization collection");

    assert_eq!(facades.len(), 1);
    assert_eq!(facades[0]["facadeItemId"], json!("facade:item"));
    assert_eq!(animations.len(), 1);
    assert_eq!(
        animations[0]["materializationStatus"],
        json!("materialized")
    );
    assert_eq!(session.manifest_io_metrics().source_decompression_count, 2);
}

#[test]
fn relative_jsonl_and_runtime_descriptor_do_not_share_retained_source_payloads() {
    let fixture = write_session_fixture();
    let session = RawExportSession::open(fixture.path()).expect("open raw-export session");

    let rows = session
        .read_relative_jsonl("recipes/shard-0.jsonl")
        .expect("read JSONL shard");
    assert_eq!(rows.len(), 2);
    let descriptors = session
        .runtime_file_descriptors(&[("recipeShard", "recipeShard")])
        .expect("build runtime descriptor");
    assert_eq!(descriptors.len(), 1);

    let metrics = session.manifest_io_metrics();
    assert_eq!(metrics.source_open_count, 2, "{metrics:?}");
    assert_eq!(metrics.source_read_count, 2, "{metrics:?}");
    assert_eq!(metrics.jsonl_parse_count, 2, "{metrics:?}");
    assert_eq!(metrics.jsonl_row_parse_count, 4, "{metrics:?}");
}

#[test]
fn overlapping_runtime_descriptors_reuse_one_cached_descriptor() {
    let fixture = write_session_fixture();
    let session = RawExportSession::open(fixture.path()).expect("open raw-export session");

    let first = session
        .runtime_file_descriptors(&[("first", "recipeShard"), ("second", "recipeShard")])
        .expect("build overlapping runtime descriptors");
    let second = session
        .runtime_file_descriptors(&[("third", "recipeShard")])
        .expect("build repeated runtime descriptor");
    assert_eq!(first.len(), 2);
    assert_eq!(second.len(), 1);
    assert_eq!(first[0]["sha256"], first[1]["sha256"]);
    assert_eq!(first[0]["sha256"], second[0]["sha256"]);

    let descriptor_a = session
        .describe_relative_file("recipes/shard-0.jsonl")
        .expect("describe fixture file")
        .expect("fixture descriptor exists");
    let descriptor_b = session
        .describe_relative_file("recipes/shard-0.jsonl")
        .expect("describe fixture file again")
        .expect("fixture descriptor still exists");
    assert!(Arc::ptr_eq(&descriptor_a, &descriptor_b));

    let metrics = session.manifest_io_metrics();
    assert_eq!(metrics.source_open_count, 1, "{metrics:?}");
    assert_eq!(metrics.source_read_count, 1, "{metrics:?}");
    assert_eq!(metrics.jsonl_parse_count, 1, "{metrics:?}");
}

#[test]
fn relative_jsonl_rejects_parent_traversal() {
    let fixture = write_session_fixture();
    let session = RawExportSession::open(fixture.path()).expect("open raw-export session");
    let error = session
        .read_relative_jsonl("../outside.jsonl")
        .expect_err("parent traversal must fail closed");
    assert!(
        format!("{error:#}").contains("portable and relative"),
        "{error:#}"
    );
}

#[test]
fn semantic_validation_reuses_session_json_cache() {
    let root = tempfile::tempdir().expect("create semantic validation fixture");
    fs::write(
        root.path().join("export-report.json"),
        serde_json::to_vec(&json!({
            "counts": {
                "items": 2,
                "recipes": 1
            }
        }))
        .expect("serialize export report"),
    )
    .expect("write export report");
    fs::write(
        root.path().join("manifest.json"),
        serde_json::to_vec_pretty(&json!({
            "schemaVersion": "nesqlpp/raw-export/v1",
            "files": {
                "exportReport": "export-report.json"
            }
        }))
        .expect("serialize manifest"),
    )
    .expect("write manifest");
    let output = tempfile::tempdir().expect("create semantic validation output");
    let session = RawExportSession::open(root.path()).expect("open raw-export session");

    session
        .read_manifest_json("exportReport")
        .expect("prime export report cache")
        .expect("export report exists");
    compile_semantic_validation_report_with_session(&session, output.path())
        .expect("compile semantic validation report");

    let metrics = session.manifest_io_metrics();
    assert_eq!(metrics.source_open_count, 1, "{metrics:?}");
    assert_eq!(metrics.source_read_count, 1, "{metrics:?}");
    assert_eq!(metrics.json_parse_count, 1, "{metrics:?}");
    assert!(metrics.cache_hit_count >= 1, "{metrics:?}");
}

#[test]
fn diagnostics_summary_and_raw_abi_share_one_source_open_and_parse() {
    let fixture = write_session_fixture();
    let session = RawExportSession::open(fixture.path()).expect("open raw-export session");
    let report_dir = tempfile::tempdir().expect("create diagnostics report directory");
    let report = report_dir.path().join("diagnostics.json");

    run_diagnostics_report_with_session(&session, None, &report, false, DiagnosticsMode::Inspect)
        .expect("run diagnostics through session");

    let metrics = session.manifest_io_metrics();
    assert_eq!(metrics.source_open_count, 1, "{metrics:?}");
    assert_eq!(metrics.source_read_count, 1, "{metrics:?}");
    assert_eq!(metrics.jsonl_parse_count, 1, "{metrics:?}");
    assert_eq!(metrics.jsonl_row_parse_count, 2, "{metrics:?}");
    assert_eq!(metrics.raw_export_abi_validation_count, 1, "{metrics:?}");
}

fn write_generation_pointer(authority: &Path, generation_id: &str) {
    fs::write(
        authority.join("current.json"),
        serde_json::to_vec_pretty(&json!({
            "schemaVersion": "nesqlpp/raw-export-generation-pointer/v1",
            "generationId": generation_id,
            "relativePath": format!("generations/{generation_id}"),
        }))
        .expect("serialize generation pointer"),
    )
    .expect("write generation pointer");
}

#[cfg(unix)]
fn create_directory_link(target: &Path, link: &Path) {
    std::os::unix::fs::symlink(target, link).expect("create directory symlink");
}

#[cfg(unix)]
fn remove_directory_link(link: &Path) {
    fs::remove_file(link).expect("remove directory symlink");
}

#[cfg(windows)]
fn create_directory_link(target: &Path, link: &Path) {
    let status = std::process::Command::new("cmd")
        .args(["/d", "/s", "/c", "mklink", "/J"])
        .arg(link)
        .arg(target)
        .status()
        .expect("run mklink /J");
    assert!(status.success(), "mklink /J failed: {status}");
}

#[cfg(windows)]
fn remove_directory_link(link: &Path) {
    fs::remove_dir(link).expect("remove directory junction");
}

#[test]
fn session_rejects_authority_link() {
    let parent = tempfile::tempdir().expect("create link parent");
    let target = write_session_fixture();
    let authority = parent.path().join("raw-export-link");
    create_directory_link(target.path(), &authority);

    let error = RawExportSession::open(&authority).expect_err("authority link must fail closed");
    assert!(
        format!("{error:#}")
            .contains("raw-export authority must not be a symlink or reparse point"),
        "{error:#}"
    );
    remove_directory_link(&authority);
}

#[test]
fn session_rejects_generations_directory_link() {
    let authority = tempfile::tempdir().expect("create raw-export authority");
    let outside = tempfile::tempdir().expect("create outside generations root");
    let generation_id = "generation-0001";
    let generation = outside.path().join(generation_id);
    fs::create_dir_all(&generation).expect("create outside generation");
    fs::write(
        generation.join("manifest.json"),
        br#"{"schemaVersion":"fixture/v1","files":{}}"#,
    )
    .expect("write outside generation manifest");
    let generations = authority.path().join("generations");
    create_directory_link(outside.path(), &generations);
    write_generation_pointer(authority.path(), generation_id);

    let error = RawExportSession::open(authority.path())
        .expect_err("generations directory link must fail closed");
    assert!(
        format!("{error:#}")
            .contains("raw-export generations directory must not be a symlink or reparse point"),
        "{error:#}"
    );
    remove_directory_link(&generations);
}

#[test]
fn session_rejects_current_generation_link() {
    let authority = tempfile::tempdir().expect("create raw-export authority");
    let outside = write_session_fixture();
    let generation_id = "generation-0001";
    let generations = authority.path().join("generations");
    fs::create_dir_all(&generations).expect("create generations directory");
    let generation = generations.join(generation_id);
    create_directory_link(outside.path(), &generation);
    write_generation_pointer(authority.path(), generation_id);

    let error = RawExportSession::open(authority.path())
        .expect_err("current generation link must fail closed");
    assert!(
        format!("{error:#}")
            .contains("raw-export current generation must not be a symlink or reparse point"),
        "{error:#}"
    );
    remove_directory_link(&generation);
}
