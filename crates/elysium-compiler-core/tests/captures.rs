use elysium_compiler_core::{
    fragments::assemble,
    source::{SourceManifest, SOURCE_REVISION},
};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::fs;

fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

#[test]
fn large_capture_reaches_blob_verification_without_count_or_double_manifest_limit() {
    // Exercise real parsing, identity and descriptor checks without writing 180,000 files.
    // An intentionally absent first blob must still block publication.
    for count in [100_001, 180_001] {
        let root = tempfile::tempdir().unwrap();
        let input = root.path().join("capture");
        fs::create_dir_all(input.join("blobs")).unwrap();
        let environment = digest(b"environment");
        let mut files = vec![
            json!({"path":"environment.json","kind":"environment","encoding":"json",
            "bytes":11,"decodedBytes":11,"rows":1,"sha256":environment}),
        ];
        for index in 1..count {
            files.push(json!({"path":format!("records/items/part-{index:06}.jsonl.gz"),"kind":"items",
                "encoding":"jsonl.gzip","bytes":20,"decodedBytes":0,"rows":0,"sha256":digest(b"empty")}));
        }
        let mut source: SourceManifest =
            serde_json::from_value(json!({"format":"elysium.source","revision":SOURCE_REVISION,
            "id":"", "environment":environment,"producer":{"name":"nesql","version":"fixture"},
            "scope":{"mode":"selection","collections":["items"]},"files":files}))
            .unwrap();
        source.id = source.digest().unwrap();
        source.validate().unwrap();
        let mut capture = json!({"format":"elysium.capture","revision":1,"state":"complete",
            "provenance":{},"request":{},"files":source.files,"source":source});
        capture["id"] = Value::String(digest(&serde_json::to_vec(&capture).unwrap()));
        let bytes = serde_json::to_vec(&capture).unwrap();
        if count == 180_001 {
            assert!(bytes.len() > 64 * 1024 * 1024);
        }
        fs::write(input.join("manifest.json"), &bytes).unwrap();
        let output = root.path().join("output");
        let error = assemble(&input, &digest(&bytes), &output).unwrap_err();
        assert!(
            error.chain().any(|cause| cause
                .downcast_ref::<std::io::Error>()
                .is_some_and(|e| e.kind() == std::io::ErrorKind::NotFound)),
            "{error:#}"
        );
        assert!(!output.exists());
    }
}
