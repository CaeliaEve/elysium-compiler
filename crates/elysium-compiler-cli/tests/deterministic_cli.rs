use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn resolve_output(authority: &std::path::Path) -> PathBuf {
    elysium_compiler_core::output_generation::resolve_current_generation(authority)
        .unwrap()
        .path()
        .to_path_buf()
}

fn public_path(path: &std::path::Path) -> String {
    let path = path.to_string_lossy().replace('\\', "/");
    if let Some(path) = path.strip_prefix("//?/UNC/") {
        return format!("//{path}");
    }
    path.strip_prefix("//?/").unwrap_or(&path).to_string()
}

fn core_fixture_path(relative: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("elysium-compiler-core")
        .join("fixtures")
        .join(relative)
}

fn copy_tree(source: &std::path::Path, target: &std::path::Path) {
    fs::create_dir_all(target).unwrap();
    for entry in fs::read_dir(source).unwrap() {
        let entry = entry.unwrap();
        let source_path = entry.path();
        let target_path = target.join(entry.file_name());
        if source_path.is_dir() {
            copy_tree(&source_path, &target_path);
        } else {
            fs::copy(source_path, target_path).unwrap();
        }
    }
}

fn write_texture_blocker_fixture() -> tempfile::TempDir {
    let input = tempfile::tempdir().unwrap();
    copy_tree(&core_fixture_path("raw-export-texture-atlas"), input.path());
    let manifest_path = input.path().join("manifest.json");
    let mut manifest: Value = serde_json::from_slice(&fs::read(&manifest_path).unwrap()).unwrap();
    manifest["files"]["animations"] = json!("animations.jsonl");
    fs::write(manifest_path, serde_json::to_vec_pretty(&manifest).unwrap()).unwrap();
    fs::write(
        input.path().join("animations.jsonl"),
        b"{\"assetId\":\"nesqlpp:item/i~minecraft~iron_ingot~0\",\"frameCount\":20,\"frameDurationMs\":100}\n",
    )
    .unwrap();
    input
}

fn write_missing_atlas_asset_fixture() -> tempfile::TempDir {
    let input = tempfile::tempdir().unwrap();
    copy_tree(&core_fixture_path("raw-export-texture-atlas"), input.path());
    fs::remove_file(input.path().join("textures/atlas/static-fixture.webp")).unwrap();
    input
}

fn assert_pre_session_golden(thread_count: usize) {
    let output = tempfile::tempdir().unwrap();
    let receipt = tempfile::tempdir().unwrap();
    let report = receipt.path().join("compiler-report.json");
    let status = Command::new(env!("CARGO_BIN_EXE_elysium-compiler"))
        .arg("compile")
        .arg("--input")
        .arg(core_fixture_path("raw-export-texture-atlas"))
        .arg("--output")
        .arg(output.path())
        .arg("--report")
        .arg(&report)
        .arg("--scope")
        .arg("all")
        .arg("--threads")
        .arg(thread_count.to_string())
        .arg("--strict")
        .status()
        .unwrap();
    assert!(
        status.success(),
        "full compiler CLI process failed: {status}"
    );
    let generation = resolve_output(output.path());

    let catalog: Value = serde_json::from_slice(
        &fs::read(core_fixture_path(
            "golden/pre-session-raw-export-texture-atlas.sha256.json",
        ))
        .unwrap(),
    )
    .unwrap();
    let artifacts = catalog["artifacts"].as_object().unwrap();
    assert_eq!(
        artifacts.len(),
        41,
        "the frozen deterministic catalog changed"
    );
    for (relative_path, expected_hash) in artifacts {
        let bytes = fs::read(generation.join(relative_path))
            .unwrap_or_else(|error| panic!("read frozen artifact {relative_path}: {error}"));
        let actual_hash = format!("{:x}", Sha256::digest(bytes));
        assert_eq!(
            actual_hash,
            expected_hash.as_str().unwrap(),
            "pre-session artifact bytes changed: {relative_path}"
        );
    }
}

#[test]
fn ui_scope_compiles_recipe_and_ui_artifacts_from_frozen_pipeline() {
    let output = tempfile::tempdir().unwrap();
    let receipt = tempfile::tempdir().unwrap();
    let report = receipt.path().join("compiler-report.json");
    let status = Command::new(env!("CARGO_BIN_EXE_elysium-compiler"))
        .arg("compile")
        .arg("--input")
        .arg(core_fixture_path("raw-export-texture-atlas"))
        .arg("--output")
        .arg(output.path())
        .arg("--report")
        .arg(&report)
        .arg("--scope")
        .arg("ui")
        .arg("--threads")
        .arg("4")
        .arg("--strict")
        .status()
        .unwrap();
    assert!(status.success(), "UI compiler CLI process failed: {status}");
    let generation = resolve_output(output.path());
    let receipt_json: Value = serde_json::from_slice(&fs::read(&report).unwrap()).unwrap();
    assert_eq!(receipt_json["output"], public_path(&generation));
    assert_eq!(
        receipt_json["outputAuthority"],
        public_path(&fs::canonicalize(output.path()).unwrap())
    );
    assert_eq!(
        receipt_json["outputPointer"],
        public_path(
            &fs::canonicalize(output.path())
                .unwrap()
                .join("current.json")
        )
    );

    let catalog: Value = serde_json::from_slice(
        &fs::read(core_fixture_path(
            "golden/pre-session-raw-export-texture-atlas.sha256.json",
        ))
        .unwrap(),
    )
    .unwrap();
    for relative_path in [
        "rust/recipes.bin",
        "recipes/ui-payload-index.json",
        "rust/ui-pack/ui_templates.bin",
        "rust/ui-pack/ui_bindings.bin",
        "rust/ui-pack/ui_strings.bin",
    ] {
        let expected_hash = catalog["artifacts"][relative_path].as_str().unwrap();
        let actual_hash = format!(
            "{:x}",
            Sha256::digest(fs::read(generation.join(relative_path)).unwrap())
        );
        assert_eq!(
            actual_hash, expected_hash,
            "UI artifact changed: {relative_path}"
        );
    }
}

#[test]
fn pre_session_cli_deterministic_artifacts_remain_byte_identical() {
    assert_pre_session_golden(1);
}

#[test]
fn parallel_pack_dag_remains_byte_identical() {
    assert_pre_session_golden(4);
}

#[test]
fn cli_failure_never_promotes_or_mutates_flat_root_legacy_artifacts() {
    let input = tempfile::tempdir().unwrap();
    let output = tempfile::tempdir().unwrap();
    let receipt = tempfile::tempdir().unwrap();
    let report = receipt.path().join("compiler-report.json");
    fs::create_dir_all(output.path().join("rust")).unwrap();
    fs::write(output.path().join("manifest.json"), "stale").unwrap();
    fs::write(output.path().join("rust/runtime-manifest.json"), "stale").unwrap();

    let status = Command::new(env!("CARGO_BIN_EXE_elysium-compiler"))
        .arg("compile")
        .arg("--input")
        .arg(input.path())
        .arg("--output")
        .arg(output.path())
        .arg("--report")
        .arg(report)
        .arg("--strict")
        .status()
        .unwrap();

    assert!(!status.success());
    assert_eq!(
        fs::read(output.path().join("manifest.json")).unwrap(),
        b"stale"
    );
    assert_eq!(
        fs::read(output.path().join("rust/runtime-manifest.json")).unwrap(),
        b"stale"
    );
    assert!(!output.path().join("current.json").exists());
    assert_eq!(
        fs::read_dir(output.path().join("generations"))
            .unwrap()
            .count(),
        0
    );
}

#[test]
fn strict_texture_blocker_preserves_external_diagnostics_without_publishing() {
    let input = write_texture_blocker_fixture();
    let output = tempfile::tempdir().unwrap();
    let receipt = tempfile::tempdir().unwrap();
    let non_strict_report = receipt.path().join("non-strict-report.json");
    let non_strict_status = Command::new(env!("CARGO_BIN_EXE_elysium-compiler"))
        .arg("compile")
        .arg("--input")
        .arg(input.path())
        .arg("--output")
        .arg(output.path())
        .arg("--report")
        .arg(&non_strict_report)
        .arg("--scope")
        .arg("textures")
        .arg("--threads")
        .arg("1")
        .status()
        .unwrap();
    assert!(non_strict_status.success());
    let published = resolve_output(output.path());
    let published_missing_report: Value = serde_json::from_slice(
        &fs::read(published.join("rust/missing-texture-report.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(
        published_missing_report["counts"]["blockingActionableIssues"],
        json!(1)
    );
    let pointer_before = fs::read(output.path().join("current.json")).unwrap();
    let generation_count_before = fs::read_dir(output.path().join("generations"))
        .unwrap()
        .count();

    let strict_report = receipt.path().join("strict-failure-report.json");
    let strict_status = Command::new(env!("CARGO_BIN_EXE_elysium-compiler"))
        .arg("compile")
        .arg("--input")
        .arg(input.path())
        .arg("--output")
        .arg(output.path())
        .arg("--report")
        .arg(&strict_report)
        .arg("--scope")
        .arg("textures")
        .arg("--threads")
        .arg("1")
        .arg("--strict")
        .status()
        .unwrap();

    assert!(!strict_status.success());
    let failure_receipt: Value =
        serde_json::from_slice(&fs::read(&strict_report).unwrap()).unwrap();
    assert_eq!(failure_receipt["status"], json!("blocked"));
    assert_eq!(failure_receipt["published"], json!(false));
    assert_eq!(
        failure_receipt["diagnostics"]["missingTextureReport"]["counts"]
            ["blockingActionableIssues"],
        json!(1)
    );
    assert_eq!(
        failure_receipt["diagnostics"]["missingTextureReport"]["issues"][0]["code"],
        json!("EXPECTED_ANIMATED_BUT_STATIC")
    );
    assert_eq!(
        fs::read(output.path().join("current.json")).unwrap(),
        pointer_before
    );
    assert_eq!(
        fs::read_dir(output.path().join("generations"))
            .unwrap()
            .count(),
        generation_count_before
    );
    assert!(fs::read_dir(output.path().join("generations"))
        .unwrap()
        .all(|entry| !entry
            .unwrap()
            .file_name()
            .to_string_lossy()
            .ends_with(".staging")));
}

#[test]
fn strict_missing_atlas_asset_preserves_failure_receipt_and_output_authority() {
    let input = write_missing_atlas_asset_fixture();
    let output = tempfile::tempdir().unwrap();
    let receipt = tempfile::tempdir().unwrap();
    let first_report = receipt.path().join("first-strict-failure-report.json");

    let first_status = Command::new(env!("CARGO_BIN_EXE_elysium-compiler"))
        .arg("compile")
        .arg("--input")
        .arg(input.path())
        .arg("--output")
        .arg(output.path())
        .arg("--report")
        .arg(&first_report)
        .arg("--scope")
        .arg("textures")
        .arg("--threads")
        .arg("1")
        .arg("--strict")
        .status()
        .unwrap();

    assert!(!first_status.success());
    assert!(!output.path().join("current.json").exists());
    assert_eq!(
        fs::read_dir(output.path().join("generations"))
            .unwrap()
            .count(),
        0
    );
    assert_missing_atlas_failure_receipt(&first_report);

    let successful_report = receipt.path().join("successful-report.json");
    let successful_status = Command::new(env!("CARGO_BIN_EXE_elysium-compiler"))
        .arg("compile")
        .arg("--input")
        .arg(core_fixture_path("raw-export-texture-atlas"))
        .arg("--output")
        .arg(output.path())
        .arg("--report")
        .arg(&successful_report)
        .arg("--scope")
        .arg("textures")
        .arg("--threads")
        .arg("1")
        .arg("--strict")
        .status()
        .unwrap();
    assert!(successful_status.success());
    let pointer_before = fs::read(output.path().join("current.json")).unwrap();
    let generation_count_before = fs::read_dir(output.path().join("generations"))
        .unwrap()
        .count();

    let second_report = receipt.path().join("second-strict-failure-report.json");
    let second_status = Command::new(env!("CARGO_BIN_EXE_elysium-compiler"))
        .arg("compile")
        .arg("--input")
        .arg(input.path())
        .arg("--output")
        .arg(output.path())
        .arg("--report")
        .arg(&second_report)
        .arg("--scope")
        .arg("textures")
        .arg("--threads")
        .arg("1")
        .arg("--strict")
        .status()
        .unwrap();

    assert!(!second_status.success());
    assert_missing_atlas_failure_receipt(&second_report);
    assert_eq!(
        fs::read(output.path().join("current.json")).unwrap(),
        pointer_before
    );
    assert_eq!(
        fs::read_dir(output.path().join("generations"))
            .unwrap()
            .count(),
        generation_count_before
    );
    assert!(fs::read_dir(output.path().join("generations"))
        .unwrap()
        .all(|entry| !entry
            .unwrap()
            .file_name()
            .to_string_lossy()
            .ends_with(".staging")));
}

fn assert_missing_atlas_failure_receipt(report: &std::path::Path) {
    let failure_receipt: Value = serde_json::from_slice(&fs::read(report).unwrap()).unwrap();
    assert_eq!(
        failure_receipt["schemaVersion"],
        json!("elysium-compiler/compile-failure-receipt/v1")
    );
    assert_eq!(failure_receipt["status"], json!("blocked"));
    assert_eq!(failure_receipt["published"], json!(false));
    assert_eq!(
        failure_receipt["diagnostics"]["missingTextureReport"]["counts"]["missingAtlasAssetFiles"],
        json!(1)
    );
    let missing_path = failure_receipt["diagnostics"]["missingTextureReport"]
        ["missingAtlasAssetFiles"][0]
        .as_str()
        .unwrap();
    assert!(
        missing_path.contains("textures/atlas/static-fixture.webp"),
        "missing atlas path was not preserved: {missing_path}"
    );
}
