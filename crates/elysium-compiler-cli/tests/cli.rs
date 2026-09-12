use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn run(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_elysium-compiler"))
        .args(args)
        .output()
        .unwrap()
}

fn success(args: &[&str]) -> Value {
    let output = run(args);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).unwrap()
}

fn path(path: &Path) -> &str {
    path.to_str().unwrap()
}

#[test]
fn cli_compiles_checks_and_reports_the_same_snapshot() {
    let source = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../contracts/fixtures/source");
    let work = tempfile::tempdir().unwrap();
    let catalog = work.path().join("catalog");
    let receipt = work.path().join("receipt.json");
    let inspect = success(&["inspect", "--input", path(&source)]);
    assert_eq!(inspect["counts"]["recipes"], 14);
    assert_eq!(inspect["counts"]["aspects"], 3);
    assert_eq!(
        inspect["counts"].as_object().unwrap().len(),
        inspect["scope"]["collections"].as_array().unwrap().len()
    );
    let args = [
        "compile",
        "--input",
        path(&source),
        "--output",
        path(&catalog),
        "--report",
        path(&receipt),
    ];
    let first = success(&args);
    assert_eq!(first, success(&args));
    assert_eq!(first["source"], inspect["id"]);
    let report: Value = serde_json::from_slice(&fs::read(&receipt).unwrap()).unwrap();
    assert_eq!(report, first);
    assert_eq!(
        success(&["check", "--input", path(&catalog)])["id"],
        first["id"]
    );

    let pointer = fs::read(catalog.join("current.json")).unwrap();
    let failure = run(&[
        "compile",
        "--input",
        path(&work.path().join("absent")),
        "--output",
        path(&catalog),
    ]);
    assert!(!failure.status.success());
    assert!(failure.stdout.is_empty());
    assert_eq!(fs::read(catalog.join("current.json")).unwrap(), pointer);

    let hidden = work.path().join("uncreated");
    assert!(!run(&[
        "compile",
        "--input",
        path(&source),
        "--output",
        path(&hidden.join("..").join("catalog")),
    ])
    .status
    .success());
    assert!(!hidden.exists());
    assert!(!run(&[
        "compile",
        "--input",
        path(&source),
        "--output",
        path(&catalog),
        "--report",
        path(&catalog.join("current.json")),
    ])
    .status
    .success());
    assert_eq!(fs::read(catalog.join("current.json")).unwrap(), pointer);

    let copied = work.path().join("source");
    let manifest = elysium_compiler_core::source::Source::open(&source)
        .unwrap()
        .manifest;
    for file in
        std::iter::once("manifest.json").chain(manifest.files.iter().map(|file| file.path.as_str()))
    {
        let target = copied.join(file);
        fs::create_dir_all(target.parent().unwrap()).unwrap();
        fs::copy(source.join(file), target).unwrap();
    }
    assert!(!run(&[
        "compile",
        "--input",
        path(&copied),
        "--output",
        path(&catalog),
        "--report",
        path(&copied.join("manifest.json")),
    ])
    .status
    .success());
    assert_eq!(
        fs::read(copied.join("manifest.json")).unwrap(),
        fs::read(source.join("manifest.json")).unwrap()
    );
    assert!(!run(&[
        "compile",
        "--input",
        path(&copied),
        "--output",
        path(&copied.join("output")),
    ])
    .status
    .success());
    assert!(!copied.join("output").exists());

    #[cfg(unix)]
    {
        let linked = work.path().join("linked");
        std::os::unix::fs::symlink(&copied, &linked).unwrap();
        assert!(!run(&[
            "compile",
            "--input",
            path(&source),
            "--output",
            path(&linked.join("output")),
        ])
        .status
        .success());
        assert!(!copied.join("output").exists());
    }

    let schema = work.path().join("schema.json");
    success(&["schema", "--output", path(&schema)]);
    fs::write(&schema, b"previous schema").unwrap();
    success(&["schema", "--output", path(&schema)]);
    let schema: Value = serde_json::from_slice(&fs::read(schema).unwrap()).unwrap();
    assert!(schema["definitions"]["Choice"]["properties"]["returns"].is_object());
    assert!(!run(&["schemas"]).status.success());
    assert!(!run(&[
        "compile",
        "--input",
        path(&source),
        "--output",
        path(&catalog),
        "--scope",
        "all"
    ])
    .status
    .success());
}
