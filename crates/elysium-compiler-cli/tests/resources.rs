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

fn resolve(root: &Path, report: &Value, expected: &str, output: &str) -> Output {
    let bytes = serde_json::to_vec(report).unwrap();
    fs::write(root.join("check.json"), &bytes).unwrap();
    Command::new(env!("CARGO_BIN_EXE_elysium-compiler"))
        .args(["resolve", "--resources"])
        .arg(root.join("inventory"))
        .arg("--report")
        .arg(root.join("check.json"))
        .args([
            "--sha256",
            &hash(&bytes),
            "--environment",
            expected,
            "--output",
        ])
        .arg(root.join(output))
        .output()
        .unwrap()
}

// Formal Source fixture, not a diagnostic converted into domain records.
fn picture_source(root: &Path, env: &Value, pixels: &[u8]) -> Value {
    use flate2::{write::GzEncoder, Compression};
    let source = root.join("source");
    fs::create_dir_all(&source).unwrap();
    let environment = serde_json::to_vec(env).unwrap();
    fs::write(source.join("environment.json"), &environment).unwrap();
    let png_path = format!("assets/{}.png", hash(pixels));
    fs::create_dir_all(source.join("assets")).unwrap();
    fs::write(source.join(&png_path), pixels).unwrap();
    let mut files = vec![
        json!({"path":"environment.json","kind":"environment","encoding":"json",
        "bytes":environment.len(),"decodedBytes":environment.len(),"rows":1,"sha256":hash(&environment)}),
        json!({"path":png_path,"kind":"asset","encoding":"png","bytes":pixels.len(),"decodedBytes":pixels.len(),"rows":0,"sha256":hash(pixels)}),
    ];
    let mut assets = vec![];
    for kind in ["resource", "capture"] {
        let mut asset = json!({"path":png_path,"width":2,"height":1,"frames":[],"interpolate":false,
            "source":{"kind":kind,"location":"demo:textures/a.png"}});
        asset["id"] = json!(format!(
            "asset_{}",
            hash(&serde_json::to_vec(&asset).unwrap())
        ));
        assets.push(asset);
    }
    assets.sort_by(|a, b| a["id"].as_str().cmp(&b["id"].as_str()));
    for kind in elysium_compiler_core::source::CORE_COLLECTIONS {
        let mut plain = Vec::new();
        if *kind == "assets" {
            for asset in &assets {
                plain.extend(serde_json::to_vec(asset).unwrap());
                plain.push(b'\n');
            }
        }
        let mut gzip = GzEncoder::new(Vec::new(), Compression::default());
        gzip.write_all(&plain).unwrap();
        let bytes = gzip.finish().unwrap();
        let path = format!("records/{kind}/part-000000.jsonl.gz");
        fs::create_dir_all(source.join(&path).parent().unwrap()).unwrap();
        fs::write(source.join(&path), &bytes).unwrap();
        files.push(json!({"path":path,"kind":kind,"encoding":"jsonl.gzip","bytes":bytes.len(),
            "decodedBytes":plain.len(),"rows":if *kind == "assets" {2} else {0},"sha256":hash(&bytes)}));
    }
    files.sort_by(|a, b| a["path"].as_str().cmp(&b["path"].as_str()));
    let mut manifest = json!({"format":"elysium.source","revision":elysium_compiler_core::source::SOURCE_REVISION,"producer":{"name":"nesql","version":"fixture"},
        "environment":hash(&environment),"scope":{"mode":"selection","collections":elysium_compiler_core::source::CORE_COLLECTIONS},"files":files});
    manifest["id"] = json!(hash(&serde_json::to_vec(&manifest).unwrap()));
    fs::write(
        source.join("manifest.json"),
        serde_json::to_vec(&manifest).unwrap(),
    )
    .unwrap();
    manifest
}

fn png(colors: &[u8], fast: bool) -> Vec<u8> {
    use image::{
        codecs::png::{CompressionType, FilterType, PngEncoder},
        ImageEncoder,
    };
    let mut out = Vec::new();
    PngEncoder::new_with_quality(
        &mut out,
        if fast {
            CompressionType::Fast
        } else {
            CompressionType::Best
        },
        FilterType::NoFilter,
    )
    .write_image(colors, 2, 1, image::ExtendedColorType::Rgba8)
    .unwrap();
    out
}

#[test]
fn links_native_resources_to_source_only_with_matching_session_and_exact_pixels() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    // Include partially transparent and transparent RGB, so alpha and hidden
    // channels cannot accidentally disappear from the comparison.
    let colors = [10, 20, 30, 128, 40, 50, 60, 0];
    let local = png(&colors, true);
    let native = png(&colors, false);
    assert_ne!(local, native);
    let base = archive(root, "base.jar", &[("assets/demo/textures/a.png", &local)]);
    assert!(run(root, json!([base]), "inventory").status.success());
    let runtime = json!({"game":"Minecraft 1.7.10","loader":"Forge","locale":"en_US",
        "mods":[{"id":"fixture","name":"Fixture","version":"1","sha256":base["sha256"]}],
        "inputs":[],"resources":[],"knowledge":{},"settings":{"iconPixels":"64"}});
    let mut env = runtime.clone();
    env["probes"] = json!([{"count":1,"channels":{}}]);
    env["settings"]["profile"] = json!("full");
    env["settings"]["handlers"] = json!("");
    let source = picture_source(root, &env, &native);
    let mut check_env = env.clone();
    check_env["settings"]["profile"] = json!("data");
    let request = json!({"key":"capture","name":"capture","profile":"full","world":"test-copy","handlers":[],"probes":env["probes"]});
    let mut check_request = request.clone();
    check_request["profile"] = json!("data");
    check_request["check"] = json!({"domain":"resources","resources":["demo:textures/a.png"]});
    let proof = |environment: &Value, request: &Value| {
        let mut selection = request.clone();
        selection.as_object_mut().unwrap().remove("key");
        selection.as_object_mut().unwrap().remove("name");
        json!({"revision":1,"environment":hash(&serde_json::to_vec(environment).unwrap()),
            "runtime":hash(&serde_json::to_vec(&runtime).unwrap()),"session":"a".repeat(64),"selection":hash(&serde_json::to_vec(&selection).unwrap())})
    };
    let job = json!({"id":"native-source-job","state":"succeeded","request":request,"result":{"id":source["id"],"path":"ignored/relocatable"},
        "provenance":proof(&env,&request)});
    let report = json!({"format":"nesql.check","job":"native-resource-job","status":"complete","environment":check_env,
        "request":check_request,"provenance":proof(&check_env,&check_request),
        "rows":[{"resource":"demo:textures/a.png","path":"assets/demo/textures/a.png","status":"passed","bytes":local.len().to_string(),"sha256":hash(&local)}]});
    let invoke = |job: &Value, report: &Value, output: &str| {
        let job_bytes = serde_json::to_vec(job).unwrap();
        let report_bytes = serde_json::to_vec(report).unwrap();
        fs::write(root.join("capture.json"), &job_bytes).unwrap();
        fs::write(root.join("check.json"), &report_bytes).unwrap();
        Command::new(env!("CARGO_BIN_EXE_elysium-compiler"))
            .args(["resolve", "--resources"])
            .arg(root.join("inventory"))
            .arg("--report")
            .arg(root.join("check.json"))
            .args([
                "--sha256",
                &hash(&report_bytes),
                "--environment",
                report["provenance"]["environment"].as_str().unwrap(),
            ])
            .arg("--source")
            .arg(root.join("source"))
            .arg("--capture")
            .arg(root.join("capture.json"))
            .args(["--capture-sha256", &hash(&job_bytes), "--output"])
            .arg(root.join(output))
            .output()
            .unwrap()
    };
    let output = invoke(&job, &report, "linked");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let linked: Value =
        serde_json::from_slice(&fs::read(root.join("linked/manifest.json")).unwrap()).unwrap();
    assert_eq!(linked["source"]["id"], source["id"]);
    assert_eq!(linked["source"]["scope"]["mode"], "selection");
    let matches = linked["source"]["assets"].as_array().unwrap();
    assert_eq!(
        matches.len(),
        1,
        "GL capture was falsely certified as a direct resource"
    );
    assert_eq!(matches[0]["pixels"], hash(&colors));
    for mode in [
        "session",
        "runtime",
        "selection",
        "source",
        "failed",
        "missing",
        "world",
        "locale",
        "pixels",
        "alpha",
    ] {
        picture_source(root, &env, &native);
        let mut bad = job.clone();
        match mode {
            "session" | "runtime" | "selection" => bad["provenance"][mode] = json!("b".repeat(64)),
            "source" => bad["result"]["id"] = json!("b".repeat(64)),
            "failed" => bad["state"] = json!("failed"),
            "missing" => {
                bad.as_object_mut().unwrap().remove("provenance");
            }
            "world" => {
                bad["request"]["world"] = json!("other-world");
                bad["provenance"] = proof(&env, &bad["request"]);
            }
            "locale" => {
                let mut changed = env.clone();
                changed["locale"] = json!("zh_CN");
                let mut changed_runtime = runtime.clone();
                changed_runtime["locale"] = json!("zh_CN");
                let manifest = picture_source(root, &changed, &native);
                bad["result"]["id"] = manifest["id"].clone();
                bad["provenance"] = proof(&changed, &request);
                bad["provenance"]["runtime"] =
                    json!(hash(&serde_json::to_vec(&changed_runtime).unwrap()));
            }
            "pixels" | "alpha" => {
                let mut changed = colors;
                changed[if mode == "alpha" { 3 } else { 4 }] += 1;
                let manifest = picture_source(root, &env, &png(&changed, false));
                bad["result"]["id"] = manifest["id"].clone();
            }
            _ => unreachable!(),
        }
        let out = invoke(&bad, &report, "rejected");
        assert!(!out.status.success(), "accepted {mode}");
        assert!(!root.join("rejected").exists(), "published {mode}");
        if matches!(mode, "pixels" | "alpha") {
            assert!(
                String::from_utf8_lossy(&out.stderr).contains("pixels differ"),
                "Failed before pixel comparison"
            );
        }
        if matches!(mode, "locale" | "world" | "session") {
            assert!(
                String::from_utf8_lossy(&out.stderr).contains("contexts differ"),
                "Failed before context comparison"
            );
        }
    }
}

#[test]
fn native_resolution_selects_exact_bytes_and_rejects_incomplete_or_mismatched_evidence() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    let base = archive(root, "base.jar", &[("assets/demo/textures/a.png", b"base")]);
    let pack = archive(
        root,
        "pack.zip",
        &[("assets/demo/textures/a.png", b"override")],
    );
    assert!(run(root, json!([base, pack]), "inventory").status.success());
    let env = json!({"mods":[{"sha256":base["sha256"]}],"resources":["pack.zip"],"inputs":[{"path":"resourcepacks/pack.zip","sha256":pack["sha256"]}]});
    let expected = hash(&serde_json::to_vec(&env).unwrap());
    let report = json!({"format":"nesql.check","job":"fixture","status":"complete","environment":env,
      "request":{"world":"test-copy","check":{"domain":"resources","resources":["demo:textures/a.png"]}},
      "rows":[{"resource":"demo:textures/a.png","status":"passed","path":"assets/demo/textures/a.png","bytes":"8","sha256":hash(b"override")}]});
    let output = resolve(root, &report, &expected, "bound");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let binding: Value =
        serde_json::from_slice(&fs::read(root.join("bound/manifest.json")).unwrap()).unwrap();
    assert_eq!(binding["format"], "elysium.bindings");
    assert_eq!(binding["environment"], expected);
    assert_eq!(binding["entries"][0]["origins"], json!(["pack.zip"]));
    assert_eq!(
        fs::read(
            root.join("bound")
                .join(binding["entries"][0]["blob"].as_str().unwrap())
        )
        .unwrap(),
        b"override"
    );
    assert!(!resolve(root, &report, &expected, "bound").status.success());
    assert!(!resolve(root, &report, &expected, "inventory/nested")
        .status
        .success());
    for mode in [
        "failed",
        "stopped",
        "domain",
        "missing",
        "extra",
        "path",
        "bytes",
        "digest",
        "environment",
        "unloaded",
        "disabled",
        "inventory",
        "blob",
    ] {
        let mut bad = report.clone();
        let mut fingerprint = expected.clone();
        match mode {
            "failed" => bad["rows"][0]["status"] = json!("failed"),
            "stopped" => bad["status"] = json!("stopped"),
            "domain" => bad["request"]["check"]["domain"] = json!("recipes"),
            "missing" => bad["rows"] = json!([]),
            "extra" => bad["rows"]
                .as_array_mut()
                .unwrap()
                .push(report["rows"][0].clone()),
            "path" => bad["rows"][0]["path"] = json!("assets/demo/textures/other.png"),
            "bytes" => bad["rows"][0]["bytes"] = json!("08"),
            "digest" => bad["rows"][0]["sha256"] = json!(hash(b"absent")),
            "environment" => fingerprint = "0".repeat(64),
            "unloaded" => {
                bad["environment"]["inputs"] = json!([]);
                fingerprint = hash(&serde_json::to_vec(&bad["environment"]).unwrap());
            }
            "disabled" => {
                bad["environment"]["resources"] = json!([]);
                fingerprint = hash(&serde_json::to_vec(&bad["environment"]).unwrap());
            }
            "inventory" => {
                fs::write(root.join("inventory/manifest.json"), b"{}").unwrap();
            }
            "blob" => {
                fs::write(
                    root.join("inventory/blobs").join(hash(b"override")),
                    b"changed",
                )
                .unwrap();
            }
            _ => unreachable!(),
        }
        let original = if mode == "inventory" {
            None
        } else {
            Some(fs::read(root.join("inventory/manifest.json")).unwrap())
        };
        let out = resolve(root, &bad, &fingerprint, "bad-result");
        assert!(!out.status.success(), "accepted {mode}");
        assert!(!root.join("bad-result").exists());
        // Rebuild the intentionally corrupted manifest for the subsequent blob case.
        if mode == "inventory" {
            assert!(run(root, json!([base, pack]), "fresh").status.success());
            fs::copy(
                root.join("fresh/manifest.json"),
                root.join("inventory/manifest.json"),
            )
            .unwrap();
        } else {
            assert_eq!(
                original.unwrap(),
                fs::read(root.join("inventory/manifest.json")).unwrap()
            );
        }
    }
}
