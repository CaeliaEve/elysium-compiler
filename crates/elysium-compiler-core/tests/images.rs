use elysium_compiler_core::{
    catalog::{compile, Catalog},
    domain::{content_id, Domain},
    source::{Source, SourceFile},
};
use flate2::{write::GzEncoder, Compression};
use sha2::{Digest, Sha256};
use std::{fs, io::Write, path::Path};

fn fixture() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../contracts/fixtures/source-semantic-40")
}

fn invalid_asset_source(target: &Path, corrupt: bool) {
    let source = Source::open(&fixture()).unwrap();
    let mut domain = Domain::load(&source).unwrap();
    let mut manifest = source.manifest.clone();
    let mut asset = domain.assets[0].clone();
    asset.source.location = "fixture:invalid-pixels".into();
    if corrupt {
        let bytes = b"not a PNG image";
        let sha256 = format!("{:x}", Sha256::digest(bytes));
        asset.path = format!("assets/{sha256}.png");
        fs::create_dir_all(target.join("assets")).unwrap();
        fs::write(target.join(&asset.path), bytes).unwrap();
        manifest.files.push(SourceFile {
            path: asset.path.clone(),
            kind: "asset".into(),
            encoding: "png".into(),
            bytes: bytes.len() as u64,
            decoded_bytes: bytes.len() as u64,
            rows: 0,
            sha256,
        });
    } else {
        asset.width += 1;
    }
    asset.id = content_id("asset", &asset).unwrap();
    domain.assets.push(asset);
    let records = serde_json::to_value(domain).unwrap();
    let original = manifest.files.clone();
    for file in &mut manifest.files {
        if corrupt
            && file.kind == "asset"
            && file.path.ends_with(".png")
            && target.join(&file.path).exists()
        {
            continue;
        }
        let bytes = if file.kind == "assets" {
            let mut rows = records[&file.kind]
                .as_array()
                .unwrap()
                .iter()
                .collect::<Vec<_>>();
            rows.sort_by_key(|row| row["id"].as_str().unwrap());
            let mut decoded = Vec::new();
            for row in &rows {
                serde_json::to_writer(&mut decoded, row).unwrap();
                decoded.push(b'\n');
            }
            let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
            encoder.write_all(&decoded).unwrap();
            let bytes = encoder.finish().unwrap();
            file.rows = rows.len() as u64;
            file.decoded_bytes = decoded.len() as u64;
            file.bytes = bytes.len() as u64;
            file.sha256 = format!("{:x}", Sha256::digest(&bytes));
            bytes
        } else {
            source
                .read_file(
                    original
                        .iter()
                        .find(|prior| prior.path == file.path)
                        .unwrap(),
                )
                .unwrap()
        };
        let path = target.join(&file.path);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, bytes).unwrap();
    }
    manifest.files.sort_by(|a, b| a.path.cmp(&b.path));
    manifest.id = manifest.digest().unwrap();
    fs::write(
        target.join("manifest.json"),
        serde_json::to_vec(&manifest).unwrap(),
    )
    .unwrap();
}

#[test]
fn inspect_and_compile_reject_bad_pixels_without_changing_publication() {
    let output = tempfile::tempdir().unwrap();
    compile(&fixture(), output.path()).unwrap();
    let pointer = fs::read(output.path().join("current.json")).unwrap();
    for corrupt in [true, false] {
        let input = tempfile::tempdir().unwrap();
        invalid_asset_source(input.path(), corrupt);
        let source = Source::open(input.path()).unwrap();
        let expected = if corrupt {
            "decode asset"
        } else {
            "asset dimensions differ from pixels"
        };
        let inspect_error = Domain::load(&source).unwrap_err();
        assert!(
            format!("{inspect_error:#}").contains(expected),
            "{inspect_error:#}"
        );
        let compile_error = compile(input.path(), output.path()).unwrap_err();
        assert!(
            format!("{compile_error:#}").contains(expected),
            "{compile_error:#}"
        );
        assert_eq!(
            fs::read(output.path().join("current.json")).unwrap(),
            pointer
        );
        Catalog::current(output.path()).unwrap().verify().unwrap();
    }
}
