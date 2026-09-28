use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::fs;
use std::io::Write;
use std::path::Path;
use std::process::{Command, Output};
use zip::write::SimpleFileOptions;

fn hash(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn archive(root: &Path, name: &str, entries: &[(&str, &[u8])]) -> Value {
    let file = fs::File::create(root.join(name)).unwrap();
    let mut writer = zip::ZipWriter::new(file);
    for (path, bytes) in entries {
        writer
            .start_file(
                *path,
                SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated),
            )
            .unwrap();
        writer.write_all(bytes).unwrap();
    }
    writer.finish().unwrap();
    let bytes = fs::read(root.join(name)).unwrap();
    json!({"key":name,"path":name,"bytes":bytes.len(),"sha256":hash(&bytes)})
}

fn run(root: &Path, archives: Value, output: &str) -> Output {
    fs::write(
        root.join("input.json"),
        serde_json::to_vec(&json!({"format":"elysium.resources","revision":1,"archives":archives}))
            .unwrap(),
    )
    .unwrap();
    Command::new(env!("CARGO_BIN_EXE_elysium-compiler"))
        .args(["resources", "--input"])
        .arg(root.join("input.json"))
        .arg("--output")
        .arg(root.join(output))
        .output()
        .unwrap()
}

#[test]
fn imports_exact_bytes_and_preserves_overrides_without_selecting_a_winner() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    let base = archive(
        root,
        "base.jar",
        &[
            ("assets/demo/textures/stone.png", b"base pixels"),
            ("assets/demo/lang/en_US.lang", b"tile.stone=Stone\n"),
            ("secret.txt", b"do not copy"),
            ("Mod.class", b"do not execute"),
        ],
    );
    let pack = archive(
        root,
        "pack.zip",
        &[
            ("assets/demo/textures/stone.png", b"override pixels"),
            ("assets/demo/textures/copy.png", b"base pixels"),
            (
                "assets/demo/textures/stone.png.mcmeta",
                b"{\"animation\":{}}",
            ),
        ],
    );
    let output = run(root, json!([base, pack]), "result");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let manifest: Value =
        serde_json::from_slice(&fs::read(root.join("result/manifest.json")).unwrap()).unwrap();
    assert_eq!(manifest["format"], "elysium.resources");
    assert_eq!(manifest["resolution"], "unverified");
    let rows = manifest["entries"].as_array().unwrap();
    assert_eq!(rows.len(), 5);
    let overrides: Vec<_> = rows
        .iter()
        .filter(|r| r["path"] == "assets/demo/textures/stone.png")
        .collect();
    assert_eq!(overrides.len(), 2);
    assert_ne!(overrides[0]["sha256"], overrides[1]["sha256"]);
    for row in rows {
        let bytes = fs::read(root.join("result").join(row["blob"].as_str().unwrap())).unwrap();
        assert_eq!(hash(&bytes), row["sha256"]);
        assert_eq!(bytes.len() as u64, row["bytes"].as_u64().unwrap());
        assert!(row.get("selected").is_none());
    }
    assert_eq!(fs::read_dir(root.join("result/blobs")).unwrap().count(), 4);
    assert!(!root.join("result/secret.txt").exists());
    let original = fs::read(root.join("result/manifest.json")).unwrap();
    assert!(!run(root, json!([base, pack]), "result").status.success());
    assert_eq!(
        original,
        fs::read(root.join("result/manifest.json")).unwrap()
    );
    // Relocation and caller ordering must not change portable identity.
    let elsewhere = tempfile::tempdir().unwrap();
    for name in ["base.jar", "pack.zip"] {
        fs::copy(root.join(name), elsewhere.path().join(name)).unwrap();
    }
    assert!(run(elsewhere.path(), json!([pack, base]), "result")
        .status
        .success());
    assert_eq!(
        original,
        fs::read(elsewhere.path().join("result/manifest.json")).unwrap()
    );
}

#[test]
fn rejects_changed_inputs_and_unsafe_resource_paths_without_publishing() {
    for mode in ["hash", "size", "path", "duplicate-key", "broken"] {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        let entry = if mode == "path" {
            "assets/demo/../../outside.png"
        } else {
            "assets/demo/textures/ok.png"
        };
        let mut source = archive(root, "base.jar", &[(entry, b"bytes")]);
        if mode == "hash" {
            source["sha256"] = json!("0".repeat(64));
        }
        if mode == "size" {
            source["bytes"] = json!(1);
        }
        if mode == "broken" {
            fs::write(root.join("base.jar"), b"broken").unwrap();
            source["bytes"] = json!(6);
            source["sha256"] = json!(hash(b"broken"));
        }
        let inputs = if mode == "duplicate-key" {
            json!([source, source])
        } else {
            json!([source])
        };
        let output = run(root, inputs, "result");
        assert!(!output.status.success(), "unexpected success for {mode}");
        assert!(
            !root.join("result").exists(),
            "published failed import {mode}"
        );
        assert!(!root.join("outside.png").exists());
    }
}

#[test]
fn rejects_ambiguous_corrupt_and_oversized_archives() {
    for mode in ["duplicate", "crc", "oversized", "link", "count"] {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        let mut source = archive(
            root,
            "base.jar",
            &[
                ("assets/demo/textures/a.png", b"pixels"),
                ("assets/demo/textures/b.png", b"other"),
            ],
        );
        let mut bytes = fs::read(root.join("base.jar")).unwrap();
        let central = bytes.windows(4).position(|s| s == b"PK\x01\x02").unwrap();
        match mode {
            "duplicate" => {
                let name = b"assets/demo/textures/b.png";
                let indices: Vec<_> = bytes
                    .windows(name.len())
                    .enumerate()
                    .filter_map(|(i, b)| (b == name).then_some(i))
                    .collect();
                for i in indices {
                    bytes[i + name.len() - 5] = b'a';
                }
            }
            "crc" => {
                bytes[central + 16..central + 20].copy_from_slice(&0_u32.to_le_bytes());
            }
            "oversized" => {
                bytes[central + 24..central + 28]
                    .copy_from_slice(&(64_u32 * 1024 * 1024 + 1).to_le_bytes());
            }
            "link" => {
                bytes[central + 5] = 3;
                bytes[central + 38..central + 42]
                    .copy_from_slice(&(0o120777_u32 << 16).to_le_bytes());
            }
            "count" => {
                let end = bytes.windows(4).rposition(|s| s == b"PK\x05\x06").unwrap();
                bytes[end + 10..end + 12].copy_from_slice(&1_u16.to_le_bytes());
            }
            _ => unreachable!(),
        }
        source["bytes"] = json!(bytes.len());
        source["sha256"] = json!(hash(&bytes));
        fs::write(root.join("base.jar"), bytes).unwrap();
        let output = run(root, json!([source]), "result");
        assert!(!output.status.success(), "unexpected success for {mode}");
        assert!(
            !root.join("result").exists(),
            "published failed import {mode}"
        );
    }
}
