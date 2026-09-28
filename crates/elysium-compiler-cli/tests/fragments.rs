use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
};

fn hash(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
fn seal(value: &mut Value) {
    value.as_object_mut().unwrap().remove("id");
    value["id"] = json!(hash(&serde_json::to_vec(value).unwrap()));
}
fn capture(root: &Path) -> Value {
    let fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../contracts/fixtures/source");
    let mut source: Value =
        serde_json::from_slice(&fs::read(fixture.join("manifest.json")).unwrap()).unwrap();
    let mut env: Value =
        serde_json::from_slice(&fs::read(fixture.join("environment.json")).unwrap()).unwrap();
    env["settings"]["profile"] = json!("full");
    env["settings"]["handlers"] = json!("");
    let request = json!({"key":"fixture","name":"fixture","profile":"full","handlers":[],"probes":env["probes"],"world":"test-copy"});
    let mut selection = request.clone();
    selection.as_object_mut().unwrap().remove("key");
    selection.as_object_mut().unwrap().remove("name");
    let mut runtime = env.clone();
    runtime.as_object_mut().unwrap().remove("probes");
    runtime["settings"]
        .as_object_mut()
        .unwrap()
        .remove("profile");
    runtime["settings"]
        .as_object_mut()
        .unwrap()
        .remove("handlers");
    let env_bytes = serde_json::to_vec(&env).unwrap();
    let environment = hash(&env_bytes);
    source["environment"] = json!(environment);
    fs::create_dir_all(root.join("blobs")).unwrap();
    for file in source["files"].as_array_mut().unwrap() {
        let bytes = if file["kind"] == "environment" {
            file["sha256"] = json!(environment);
            file["bytes"] = json!(env_bytes.len());
            file["decodedBytes"] = json!(env_bytes.len());
            env_bytes.clone()
        } else {
            fs::read(fixture.join(file["path"].as_str().unwrap())).unwrap()
        };
        fs::write(
            root.join("blobs").join(file["sha256"].as_str().unwrap()),
            bytes,
        )
        .unwrap();
    }
    seal(&mut source);
    let mut manifest = json!({"format":"elysium.capture","revision":1,"state":"complete","request":request,
        "provenance":{"revision":1,"environment":environment,"runtime":hash(&serde_json::to_vec(&runtime).unwrap()),"session":"a".repeat(64),"selection":hash(&serde_json::to_vec(&selection).unwrap())},
        "source":source,"files":source["files"]});
    seal(&mut manifest);
    manifest
}

fn run(root: &Path, manifest: &Value, output: &Path) -> Output {
    let bytes = serde_json::to_vec(manifest).unwrap();
    fs::write(root.join("manifest.json"), &bytes).unwrap();
    Command::new(env!("CARGO_BIN_EXE_elysium-compiler"))
        .args(["assemble", "--input"])
        .arg(root)
        .args(["--sha256", &hash(&bytes), "--output"])
        .arg(output)
        .output()
        .unwrap()
}

#[test]
fn assembles_complete_native_fragments_and_reuses_only_verified_source() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().join("capture");
    let manifest = capture(&root);
    let output = dir.path().join("source");
    let result = run(&root, &manifest, &output);
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let source: Value =
        serde_json::from_slice(&fs::read(output.join("manifest.json")).unwrap()).unwrap();
    assert_eq!(
        source, manifest["source"],
        "Assembly changed the native Source identity"
    );
    let result = run(&root, &manifest, &output);
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    assert_eq!(
        serde_json::from_slice::<Value>(&result.stdout).unwrap()["reused"],
        true
    );
    let pinned = Command::new(env!("CARGO_BIN_EXE_elysium-compiler"))
        .args(["assemble", "--input"])
        .arg(&root)
        .args(["--sha256", &"0".repeat(64), "--output"])
        .arg(dir.path().join("bad-pin"))
        .output()
        .unwrap();
    assert!(!pinned.status.success());
    assert!(String::from_utf8_lossy(&pinned.stderr).contains("capture manifest digest mismatch"));
    assert!(!dir.path().join("bad-pin").exists());
    // The complete manifest pins content, even when a previous assembly exists.
    let blob = root
        .join("blobs")
        .join(manifest["files"][0]["sha256"].as_str().unwrap());
    let original = fs::read(&blob).unwrap();
    fs::write(&blob, b"corrupt").unwrap();
    assert!(!run(&root, &manifest, &output).status.success());
    fs::write(&blob, original).unwrap();
    let inspect = Command::new(env!("CARGO_BIN_EXE_elysium-compiler"))
        .args(["inspect", "--input"])
        .arg(&output)
        .output()
        .unwrap();
    assert!(inspect.status.success());
    assert!(!run(&root, &manifest, &root.join("nested")).status.success());
    let file = manifest["files"][0]["path"].as_str().unwrap();
    fs::write(output.join(file), b"corrupt").unwrap();
    assert!(!run(&root, &manifest, &output).status.success());
    assert_eq!(
        fs::read(output.join(file)).unwrap(),
        b"corrupt",
        "Overwrote an existing damaged result"
    );
}

#[test]
fn rejects_incomplete_mixed_and_damaged_fragments_before_source_publication() {
    for mode in [
        "writing",
        "missing",
        "extra",
        "environment",
        "selection",
        "runtime",
        "session",
        "diagnostic",
        "blob",
        "missing-blob",
        "source-id",
        "capture-id",
        "reference",
    ] {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("capture");
        let mut manifest = capture(&root);
        let output = dir.path().join("source");
        match mode {
            "writing" => {
                manifest["state"] = json!("writing");
                manifest.as_object_mut().unwrap().remove("source");
            }
            "missing" => {
                manifest["files"].as_array_mut().unwrap().pop();
            }
            "extra" => {
                let extra = manifest["files"][0].clone();
                manifest["files"].as_array_mut().unwrap().push(extra);
            }
            "environment" | "selection" | "runtime" => {
                manifest["provenance"][mode] = json!("b".repeat(64))
            }
            "session" => manifest["provenance"][mode] = json!("invalid"),
            "diagnostic" => manifest["request"]["check"] = json!({"domain":"recipes"}),
            "blob" => fs::write(
                root.join("blobs")
                    .join(manifest["files"][0]["sha256"].as_str().unwrap()),
                b"corrupt",
            )
            .unwrap(),
            "missing-blob" => fs::remove_file(
                root.join("blobs")
                    .join(manifest["files"][0]["sha256"].as_str().unwrap()),
            )
            .unwrap(),
            "source-id" => manifest["source"]["id"] = json!("b".repeat(64)),
            "capture-id" => {}
            "reference" => {
                // Remove the asset records using a valid empty shard. Envelope
                // integrity still passes; formal domain validation must reject.
                let bytes =
                    flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default())
                        .finish()
                        .unwrap();
                fs::write(root.join("blobs").join(hash(&bytes)), &bytes).unwrap();
                let empty =
                    json!({"bytes":bytes.len(),"decodedBytes":0,"rows":0,"sha256":hash(&bytes)});
                for file in manifest["source"]["files"].as_array_mut().unwrap() {
                    if file["kind"] == "assets" {
                        for key in ["bytes", "decodedBytes", "rows", "sha256"] {
                            file[key] = empty[key].clone();
                        }
                    }
                }
                manifest["files"] = manifest["source"]["files"].clone();
                seal(&mut manifest["source"]);
            }
            _ => unreachable!(),
        }
        seal(&mut manifest);
        if mode == "capture-id" {
            manifest["id"] = json!("b".repeat(64));
        }
        let result = run(&root, &manifest, &output);
        assert!(!result.status.success(), "accepted {mode}");
        assert!(!output.exists(), "published {mode}");
        if mode == "reference" {
            assert!(
                String::from_utf8_lossy(&result.stderr).contains("missing asset:"),
                "{}",
                String::from_utf8_lossy(&result.stderr)
            );
        }
    }
}
