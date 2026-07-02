use crate::cli::CompileScope;
use crate::io::{sha256_file, write_json_value};
use crate::native_ui_pack_abi::{
    UI_BINDING_MAGIC, UI_BINDING_MAGIC_REPORT, UI_BINDING_PAYLOAD_VERSION,
    UI_BINDING_ROW_STRIDE_U32, UI_BINDING_SCHEMA, UI_BINDING_STRING_REF_COLUMNS,
    UI_PRIMITIVE_ROW_STRIDE_U32, UI_PRIMITIVE_STRING_REF_COLUMNS, UI_RECT_ROW_STRIDE_U32,
    UI_RECT_STRING_REF_COLUMNS, UI_SLOT_ROW_STRIDE_U32, UI_SLOT_STRING_REF_COLUMNS,
    UI_STRING_MAGIC, UI_STRING_MAGIC_REPORT, UI_STRING_PAYLOAD_VERSION, UI_STRING_SCHEMA,
    UI_TEMPLATE_MAGIC, UI_TEMPLATE_MAGIC_REPORT, UI_TEMPLATE_PAYLOAD_VERSION,
    UI_TEMPLATE_ROW_STRIDE_U32, UI_TEMPLATE_SCHEMA, UI_TEMPLATE_STRING_REF_COLUMNS,
    UI_TEXT_ROW_STRIDE_U32, UI_TEXT_STRING_REF_COLUMNS,
};
use crate::version::PACK_ABI_VERSION;
use anyhow::{anyhow, Result};
use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};

pub const UI_PACK_ABI_VALIDATION_SCHEMA_VERSION: &str =
    "elysium-compiler/ui-pack-abi-validation/v1";
pub const UI_PACK_ABI_VALIDATION_REPORT_PATH: &str = "rust/ui-pack-abi-validation-report.json";

const NATIVE_BINARY_PACK_MAGIC: &[u8; 8] = b"NNEIBIN\0";
const NATIVE_BINARY_PACK_HEADER_BYTES: usize = 24;

#[derive(Clone, Copy, Debug)]
enum UiPackArtifactKind {
    BinaryPack,
    Manifest,
    Report,
}

impl UiPackArtifactKind {
    const fn as_str(self) -> &'static str {
        match self {
            UiPackArtifactKind::BinaryPack => "binary-pack",
            UiPackArtifactKind::Manifest => "manifest",
            UiPackArtifactKind::Report => "report",
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct UiPackArtifactSpec {
    logical_name: &'static str,
    relative_path: &'static str,
    kind: UiPackArtifactKind,
    binary_schema: Option<&'static str>,
}

impl UiPackArtifactSpec {
    const fn binary(
        logical_name: &'static str,
        relative_path: &'static str,
        binary_schema: &'static str,
    ) -> Self {
        Self {
            logical_name,
            relative_path,
            kind: UiPackArtifactKind::BinaryPack,
            binary_schema: Some(binary_schema),
        }
    }

    const fn manifest(logical_name: &'static str, relative_path: &'static str) -> Self {
        Self {
            logical_name,
            relative_path,
            kind: UiPackArtifactKind::Manifest,
            binary_schema: None,
        }
    }

    const fn report(logical_name: &'static str, relative_path: &'static str) -> Self {
        Self {
            logical_name,
            relative_path,
            kind: UiPackArtifactKind::Report,
            binary_schema: None,
        }
    }
}

const UI_PACK_ARTIFACT_SPECS: &[UiPackArtifactSpec] = &[
    UiPackArtifactSpec::binary(
        "rustUiStringsBin",
        "rust/ui-pack/ui_strings.bin",
        UI_STRING_SCHEMA,
    ),
    UiPackArtifactSpec::binary(
        "rustUiTemplatesBin",
        "rust/ui-pack/ui_templates.bin",
        UI_TEMPLATE_SCHEMA,
    ),
    UiPackArtifactSpec::binary(
        "rustUiBindingsBin",
        "rust/ui-pack/ui_bindings.bin",
        UI_BINDING_SCHEMA,
    ),
    UiPackArtifactSpec::manifest(
        "rustUiAssetsManifest",
        "rust/ui-pack/ui_assets.manifest.json",
    ),
    UiPackArtifactSpec::manifest(
        "rustUiTemplateCatalog",
        "rust/ui-pack/ui_template_catalog.json",
    ),
    UiPackArtifactSpec::manifest(
        "rustUiTemplateBindingIndex",
        "rust/ui-pack/ui_template_binding_index.json",
    ),
    UiPackArtifactSpec::report("rustUiFamilyCensus", "rust/ui-pack/ui_family_census.json"),
    UiPackArtifactSpec::report("rustUiPackReport", "rust/ui-pack/ui_pack_report.json"),
];

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UiPackAbiValidationReport {
    pub schema_version: &'static str,
    pub pack_abi_version: &'static str,
    pub generated_at: &'static str,
    pub status: &'static str,
    pub compile_scope: String,
    pub expected_artifact_count: usize,
    pub present_artifact_count: usize,
    pub missing_required_artifacts: Vec<String>,
    pub section_violations: Vec<String>,
    pub artifacts: Vec<UiPackArtifactRecord>,
    pub policy: UiPackAbiPolicy,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UiPackArtifactRecord {
    pub logical_name: String,
    pub path: String,
    pub kind: &'static str,
    pub status: &'static str,
    pub bytes: Option<u64>,
    pub sha256: Option<String>,
    pub envelope_schema: Option<String>,
    pub payload_magic: Option<String>,
    pub version: Option<u32>,
    pub sections: Vec<UiPackSectionRecord>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UiPackSectionRecord {
    pub name: &'static str,
    pub rows: u32,
    pub stride_u32: u32,
    pub bytes: u64,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UiPackAbiPolicy {
    pub missing_required_artifact: &'static str,
    pub envelope_schema: &'static str,
    pub binary_section_layout: &'static str,
    pub string_ref_integrity: &'static str,
    pub legacy_fallback: &'static str,
}

struct NativeBinaryEnvelope {
    schema: String,
    payload_start: usize,
    payload_end: usize,
}

#[derive(Default)]
struct UiStringTableValidation {
    string_count: u32,
}

impl UiPackAbiValidationReport {
    pub fn is_blocked(&self) -> bool {
        self.status == "blocked"
    }
}

pub fn ui_pack_abi_validation_report_path(output: &Path) -> PathBuf {
    output.join(UI_PACK_ABI_VALIDATION_REPORT_PATH)
}

pub fn write_ui_pack_abi_validation_report(
    output: &Path,
    scope: CompileScope,
    strict: bool,
) -> Result<UiPackAbiValidationReport> {
    let report = validate_ui_pack_abi(output, scope)?;
    let path = ui_pack_abi_validation_report_path(output);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    write_json_value(&path, &serde_json::to_value(&report)?)?;
    if strict && report.is_blocked() {
        return Err(anyhow!(
            "native UI pack ABI validation blocked: missing={}, section violations={}",
            report.missing_required_artifacts.len(),
            report.section_violations.len()
        ));
    }
    Ok(report)
}

pub fn validate_ui_pack_abi(
    output: &Path,
    scope: CompileScope,
) -> Result<UiPackAbiValidationReport> {
    if !ui_pack_scope_applies(scope) {
        return Ok(UiPackAbiValidationReport {
            schema_version: UI_PACK_ABI_VALIDATION_SCHEMA_VERSION,
            pack_abi_version: PACK_ABI_VERSION,
            generated_at: "deterministic-rust-compiler",
            status: "skipped",
            compile_scope: scope.as_str().to_string(),
            expected_artifact_count: 0,
            present_artifact_count: 0,
            missing_required_artifacts: Vec::new(),
            section_violations: Vec::new(),
            artifacts: Vec::new(),
            policy: ui_pack_abi_policy(),
        });
    }

    let mut artifacts = Vec::with_capacity(UI_PACK_ARTIFACT_SPECS.len());
    let mut missing_required_artifacts = Vec::new();
    let mut section_violations = Vec::new();
    let mut string_table = UiStringTableValidation::default();

    for spec in UI_PACK_ARTIFACT_SPECS {
        let path = output.join(spec.relative_path);
        if !path.exists() {
            missing_required_artifacts.push(spec.relative_path.to_string());
            artifacts.push(UiPackArtifactRecord {
                logical_name: spec.logical_name.to_string(),
                path: spec.relative_path.to_string(),
                kind: spec.kind.as_str(),
                status: "missing",
                bytes: None,
                sha256: None,
                envelope_schema: spec.binary_schema.map(str::to_string),
                payload_magic: None,
                version: None,
                sections: Vec::new(),
            });
            continue;
        }
        let bytes = path.metadata()?.len();
        let sha256 = sha256_file(&path)?;
        if let Some(expected_schema) = spec.binary_schema {
            let bytes_buffer = fs::read(&path)?;
            let record = validate_binary_artifact(
                spec,
                &bytes_buffer,
                bytes,
                sha256,
                expected_schema,
                &mut string_table,
                &mut section_violations,
            );
            artifacts.push(record);
        } else {
            artifacts.push(validate_json_sidecar(
                spec,
                &path,
                bytes,
                sha256,
                &mut section_violations,
            )?);
        }
    }

    let present_artifact_count = artifacts
        .iter()
        .filter(|artifact| artifact.status == "present")
        .count();
    let status = if missing_required_artifacts.is_empty() && section_violations.is_empty() {
        "ok"
    } else {
        "blocked"
    };

    Ok(UiPackAbiValidationReport {
        schema_version: UI_PACK_ABI_VALIDATION_SCHEMA_VERSION,
        pack_abi_version: PACK_ABI_VERSION,
        generated_at: "deterministic-rust-compiler",
        status,
        compile_scope: scope.as_str().to_string(),
        expected_artifact_count: UI_PACK_ARTIFACT_SPECS.len(),
        present_artifact_count,
        missing_required_artifacts,
        section_violations,
        artifacts,
        policy: ui_pack_abi_policy(),
    })
}

pub fn ui_pack_scope_applies(scope: CompileScope) -> bool {
    matches!(
        scope,
        CompileScope::All | CompileScope::NativeUi | CompileScope::Ui
    )
}

fn ui_pack_abi_policy() -> UiPackAbiPolicy {
    UiPackAbiPolicy {
        missing_required_artifact: "fail-closed",
        envelope_schema: "NNEIBIN version 1 envelope schema must match the declared UI pack schema",
        binary_section_layout:
            "UI binary sections must have exact magic, version, stride, and byte length",
        string_ref_integrity:
            "all UI template and binding string references must resolve into ui_strings.bin",
        legacy_fallback: "forbidden",
    }
}

fn validate_binary_artifact(
    spec: &UiPackArtifactSpec,
    bytes_buffer: &[u8],
    bytes: u64,
    sha256: String,
    expected_schema: &str,
    string_table: &mut UiStringTableValidation,
    section_violations: &mut Vec<String>,
) -> UiPackArtifactRecord {
    let envelope = match parse_native_binary_envelope(bytes_buffer, expected_schema) {
        Ok(envelope) => envelope,
        Err(error) => {
            section_violations.push(format!("{}: {}", spec.relative_path, error));
            return UiPackArtifactRecord {
                logical_name: spec.logical_name.to_string(),
                path: spec.relative_path.to_string(),
                kind: spec.kind.as_str(),
                status: "invalid",
                bytes: Some(bytes),
                sha256: Some(sha256),
                envelope_schema: Some(expected_schema.to_string()),
                payload_magic: None,
                version: None,
                sections: Vec::new(),
            };
        }
    };
    let payload = &bytes_buffer[envelope.payload_start..envelope.payload_end];
    let validation = match expected_schema {
        UI_TEMPLATE_SCHEMA => validate_template_payload(payload, string_table.string_count),
        UI_BINDING_SCHEMA => validate_binding_payload(payload, string_table.string_count),
        UI_STRING_SCHEMA => validate_string_payload(payload),
        _ => Err(anyhow!("unsupported UI binary schema: {expected_schema}")),
    };
    match validation {
        Ok(payload_report) => {
            if expected_schema == UI_STRING_SCHEMA {
                string_table.string_count = payload_report.string_count.unwrap_or(0);
            }
            UiPackArtifactRecord {
                logical_name: spec.logical_name.to_string(),
                path: spec.relative_path.to_string(),
                kind: spec.kind.as_str(),
                status: "present",
                bytes: Some(bytes),
                sha256: Some(sha256),
                envelope_schema: Some(envelope.schema),
                payload_magic: Some(payload_report.magic),
                version: Some(payload_report.version),
                sections: payload_report.sections,
            }
        }
        Err(error) => {
            section_violations.push(format!("{}: {}", spec.relative_path, error));
            UiPackArtifactRecord {
                logical_name: spec.logical_name.to_string(),
                path: spec.relative_path.to_string(),
                kind: spec.kind.as_str(),
                status: "invalid",
                bytes: Some(bytes),
                sha256: Some(sha256),
                envelope_schema: Some(envelope.schema),
                payload_magic: payload_magic(payload).ok(),
                version: payload_version(payload).ok(),
                sections: Vec::new(),
            }
        }
    }
}

fn validate_json_sidecar(
    spec: &UiPackArtifactSpec,
    path: &Path,
    bytes: u64,
    sha256: String,
    section_violations: &mut Vec<String>,
) -> Result<UiPackArtifactRecord> {
    let text = fs::read_to_string(path)?;
    let value = serde_json::from_str::<serde_json::Value>(&text);
    let status = match value {
        Ok(value) => {
            if value
                .get("schemaVersion")
                .and_then(serde_json::Value::as_str)
                .is_none()
            {
                section_violations.push(format!(
                    "{}: JSON sidecar must declare schemaVersion",
                    spec.relative_path
                ));
                "invalid"
            } else {
                "present"
            }
        }
        Err(error) => {
            section_violations.push(format!(
                "{}: invalid JSON sidecar: {}",
                spec.relative_path, error
            ));
            "invalid"
        }
    };
    Ok(UiPackArtifactRecord {
        logical_name: spec.logical_name.to_string(),
        path: spec.relative_path.to_string(),
        kind: spec.kind.as_str(),
        status,
        bytes: Some(bytes),
        sha256: Some(sha256),
        envelope_schema: None,
        payload_magic: None,
        version: None,
        sections: Vec::new(),
    })
}

struct UiPayloadValidation {
    magic: String,
    version: u32,
    string_count: Option<u32>,
    sections: Vec<UiPackSectionRecord>,
}

fn validate_template_payload(payload: &[u8], string_count: u32) -> Result<UiPayloadValidation> {
    ensure_magic(payload, UI_TEMPLATE_MAGIC)?;
    let version = read_u32(payload, 8)?;
    let template_count = read_u32(payload, 12)?;
    let slot_count = read_u32(payload, 16)?;
    let text_count = read_u32(payload, 20)?;
    let primitive_count = read_u32(payload, 24)?;
    let hotspot_count = read_u32(payload, 28)?;
    let viewport_count = read_u32(payload, 32)?;
    let template_stride = read_u32(payload, 36)?;
    let slot_stride = read_u32(payload, 40)?;
    let text_stride = read_u32(payload, 44)?;
    let primitive_stride = read_u32(payload, 48)?;
    let rect_stride = read_u32(payload, 52)?;
    if version != UI_TEMPLATE_PAYLOAD_VERSION
        || template_stride != UI_TEMPLATE_ROW_STRIDE_U32
        || slot_stride != UI_SLOT_ROW_STRIDE_U32
        || text_stride != UI_TEXT_ROW_STRIDE_U32
        || primitive_stride != UI_PRIMITIVE_ROW_STRIDE_U32
        || rect_stride != UI_RECT_ROW_STRIDE_U32
    {
        return Err(anyhow!(
            "template section contract mismatch: version={}, templateStride={}, slotStride={}, textStride={}, primitiveStride={}, rectStride={}",
            version,
            template_stride,
            slot_stride,
            text_stride,
            primitive_stride,
            rect_stride
        ));
    }
    let header_bytes = 8usize + 12 * 4;
    let template_bytes = table_bytes(template_count, template_stride)?;
    let slot_bytes = table_bytes(slot_count, slot_stride)?;
    let text_bytes = table_bytes(text_count, text_stride)?;
    let primitive_bytes = table_bytes(primitive_count, primitive_stride)?;
    let hotspot_bytes = table_bytes(hotspot_count, rect_stride)?;
    let viewport_bytes = table_bytes(viewport_count, rect_stride)?;
    let expected_len = header_bytes
        .checked_add(template_bytes)
        .and_then(|value| value.checked_add(slot_bytes))
        .and_then(|value| value.checked_add(text_bytes))
        .and_then(|value| value.checked_add(primitive_bytes))
        .and_then(|value| value.checked_add(hotspot_bytes))
        .and_then(|value| value.checked_add(viewport_bytes))
        .ok_or_else(|| anyhow!("template payload length overflow"))?;
    if payload.len() != expected_len {
        return Err(anyhow!(
            "template payload length mismatch: expected={}, actual={}",
            expected_len,
            payload.len()
        ));
    }
    if string_count > 0 {
        validate_string_refs(
            payload,
            header_bytes,
            template_count,
            template_stride,
            UI_TEMPLATE_STRING_REF_COLUMNS,
            string_count,
        )?;
        validate_string_refs(
            payload,
            header_bytes + template_bytes,
            slot_count,
            slot_stride,
            UI_SLOT_STRING_REF_COLUMNS,
            string_count,
        )?;
        validate_string_refs(
            payload,
            header_bytes + template_bytes + slot_bytes,
            text_count,
            text_stride,
            UI_TEXT_STRING_REF_COLUMNS,
            string_count,
        )?;
        validate_string_refs(
            payload,
            header_bytes + template_bytes + slot_bytes + text_bytes,
            primitive_count,
            primitive_stride,
            UI_PRIMITIVE_STRING_REF_COLUMNS,
            string_count,
        )?;
        validate_string_refs(
            payload,
            header_bytes + template_bytes + slot_bytes + text_bytes + primitive_bytes,
            hotspot_count,
            rect_stride,
            UI_RECT_STRING_REF_COLUMNS,
            string_count,
        )?;
        validate_string_refs(
            payload,
            header_bytes
                + template_bytes
                + slot_bytes
                + text_bytes
                + primitive_bytes
                + hotspot_bytes,
            viewport_count,
            rect_stride,
            UI_RECT_STRING_REF_COLUMNS,
            string_count,
        )?;
    }
    Ok(UiPayloadValidation {
        magic: UI_TEMPLATE_MAGIC_REPORT.to_string(),
        version,
        string_count: None,
        sections: vec![
            section_record("templates", template_count, template_stride),
            section_record("slots", slot_count, slot_stride),
            section_record("textOverlays", text_count, text_stride),
            section_record("dynamicPrimitives", primitive_count, primitive_stride),
            section_record("hotspots", hotspot_count, rect_stride),
            section_record("viewports", viewport_count, rect_stride),
        ],
    })
}

fn validate_binding_payload(payload: &[u8], string_count: u32) -> Result<UiPayloadValidation> {
    ensure_magic(payload, UI_BINDING_MAGIC)?;
    let version = read_u32(payload, 8)?;
    let binding_count = read_u32(payload, 12)?;
    let row_stride = read_u32(payload, 16)?;
    if version != UI_BINDING_PAYLOAD_VERSION || row_stride != UI_BINDING_ROW_STRIDE_U32 {
        return Err(anyhow!(
            "binding section contract mismatch: version={}, rowStride={}",
            version,
            row_stride
        ));
    }
    let header_bytes = 8usize + 3 * 4;
    let rows_bytes = table_bytes(binding_count, row_stride)?;
    let expected_len = header_bytes
        .checked_add(rows_bytes)
        .ok_or_else(|| anyhow!("binding payload length overflow"))?;
    if payload.len() != expected_len {
        return Err(anyhow!(
            "binding payload length mismatch: expected={}, actual={}",
            expected_len,
            payload.len()
        ));
    }
    if string_count > 0 {
        validate_string_refs(
            payload,
            header_bytes,
            binding_count,
            row_stride,
            UI_BINDING_STRING_REF_COLUMNS,
            string_count,
        )?;
    }
    Ok(UiPayloadValidation {
        magic: UI_BINDING_MAGIC_REPORT.to_string(),
        version,
        string_count: None,
        sections: vec![section_record("bindings", binding_count, row_stride)],
    })
}

fn validate_string_payload(payload: &[u8]) -> Result<UiPayloadValidation> {
    ensure_magic(payload, UI_STRING_MAGIC)?;
    let version = read_u32(payload, 8)?;
    let string_count = read_u32(payload, 12)?;
    let string_bytes_len = read_u32(payload, 16)?;
    if version != UI_STRING_PAYLOAD_VERSION {
        return Err(anyhow!(
            "string section contract mismatch: version={}",
            version
        ));
    }
    let header_bytes = 8usize + 3 * 4;
    let offsets_bytes = table_bytes(string_count, 1)?;
    let strings_start = header_bytes
        .checked_add(offsets_bytes)
        .ok_or_else(|| anyhow!("string payload offset overflow"))?;
    let expected_len = strings_start
        .checked_add(string_bytes_len as usize)
        .ok_or_else(|| anyhow!("string payload length overflow"))?;
    if payload.len() != expected_len {
        return Err(anyhow!(
            "string payload length mismatch: expected={}, actual={}",
            expected_len,
            payload.len()
        ));
    }
    for index in 0..string_count {
        let offset = read_u32(payload, header_bytes + index as usize * 4)? as usize;
        if offset >= string_bytes_len as usize {
            return Err(anyhow!(
                "string offset {} out of bounds: offset={}, stringBytes={}",
                index,
                offset,
                string_bytes_len
            ));
        }
        let absolute = strings_start + offset;
        let terminates = payload[absolute..expected_len].contains(&0);
        if !terminates {
            return Err(anyhow!("string {} is not NUL-terminated", index));
        }
    }
    Ok(UiPayloadValidation {
        magic: UI_STRING_MAGIC_REPORT.to_string(),
        version,
        string_count: Some(string_count),
        sections: vec![
            section_record("stringOffsets", string_count, 1),
            UiPackSectionRecord {
                name: "stringBytes",
                rows: string_bytes_len,
                stride_u32: 0,
                bytes: string_bytes_len as u64,
            },
        ],
    })
}

fn parse_native_binary_envelope(
    bytes: &[u8],
    expected_schema: &str,
) -> Result<NativeBinaryEnvelope> {
    if bytes.len() < NATIVE_BINARY_PACK_HEADER_BYTES {
        return Err(anyhow!("native binary envelope too small: {}", bytes.len()));
    }
    if &bytes[0..8] != NATIVE_BINARY_PACK_MAGIC {
        return Err(anyhow!(
            "native binary envelope magic mismatch: {}",
            String::from_utf8_lossy(&bytes[0..8])
        ));
    }
    let version = read_u32(bytes, 8)?;
    if version != 1 {
        return Err(anyhow!(
            "native binary envelope version mismatch: {}",
            version
        ));
    }
    let schema_len = read_u32(bytes, 12)? as usize;
    let payload_len = read_u64(bytes, 16)? as usize;
    let schema_start = NATIVE_BINARY_PACK_HEADER_BYTES;
    let schema_end = schema_start
        .checked_add(schema_len)
        .ok_or_else(|| anyhow!("native binary envelope schema length overflow"))?;
    let payload_end = schema_end
        .checked_add(payload_len)
        .ok_or_else(|| anyhow!("native binary envelope payload length overflow"))?;
    if schema_end > bytes.len() || payload_end != bytes.len() {
        return Err(anyhow!(
            "native binary envelope length mismatch: schema={}, payload={}, bytes={}",
            schema_len,
            payload_len,
            bytes.len()
        ));
    }
    let schema = std::str::from_utf8(&bytes[schema_start..schema_end])?.to_string();
    if schema != expected_schema {
        return Err(anyhow!(
            "native binary envelope schema mismatch: expected {}, got {}",
            expected_schema,
            schema
        ));
    }
    Ok(NativeBinaryEnvelope {
        schema,
        payload_start: schema_end,
        payload_end,
    })
}

fn ensure_magic(payload: &[u8], expected: &[u8; 8]) -> Result<()> {
    if payload.len() < 8 {
        return Err(anyhow!("payload too small: {}", payload.len()));
    }
    if &payload[0..8] != expected {
        return Err(anyhow!(
            "payload magic mismatch: expected {}, got {}",
            String::from_utf8_lossy(expected),
            String::from_utf8_lossy(&payload[0..8])
        ));
    }
    Ok(())
}

fn payload_magic(payload: &[u8]) -> Result<String> {
    if payload.len() < 8 {
        return Err(anyhow!("payload too small: {}", payload.len()));
    }
    Ok(String::from_utf8_lossy(&payload[0..8]).replace('\0', "_NUL"))
}

fn payload_version(payload: &[u8]) -> Result<u32> {
    read_u32(payload, 8)
}

fn read_u32(bytes: &[u8], offset: usize) -> Result<u32> {
    let end = offset
        .checked_add(4)
        .ok_or_else(|| anyhow!("u32 read offset overflow"))?;
    if end > bytes.len() {
        return Err(anyhow!("u32 read out of bounds: {}/{}", end, bytes.len()));
    }
    Ok(u32::from_le_bytes(bytes[offset..end].try_into()?))
}

fn read_u64(bytes: &[u8], offset: usize) -> Result<u64> {
    let end = offset
        .checked_add(8)
        .ok_or_else(|| anyhow!("u64 read offset overflow"))?;
    if end > bytes.len() {
        return Err(anyhow!("u64 read out of bounds: {}/{}", end, bytes.len()));
    }
    Ok(u64::from_le_bytes(bytes[offset..end].try_into()?))
}

fn table_bytes(rows: u32, stride_u32: u32) -> Result<usize> {
    (rows as usize)
        .checked_mul(stride_u32 as usize)
        .and_then(|value| value.checked_mul(4))
        .ok_or_else(|| anyhow!("section byte size overflow"))
}

fn section_record(name: &'static str, rows: u32, stride_u32: u32) -> UiPackSectionRecord {
    UiPackSectionRecord {
        name,
        rows,
        stride_u32,
        bytes: rows as u64 * stride_u32 as u64 * 4,
    }
}

fn validate_string_refs(
    payload: &[u8],
    table_start: usize,
    row_count: u32,
    row_stride: u32,
    ref_columns: &[u32],
    string_count: u32,
) -> Result<()> {
    for row in 0..row_count {
        for column in ref_columns {
            let offset = table_start + ((row * row_stride + *column) as usize * 4);
            let string_ref = read_u32(payload, offset)?;
            if string_ref >= string_count {
                return Err(anyhow!(
                    "string ref out of bounds: row={}, column={}, ref={}, stringCount={}",
                    row,
                    column,
                    string_ref,
                    string_count
                ));
            }
        }
    }
    Ok(())
}
