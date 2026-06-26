use anyhow::{Context, Result};
use serde_json::Value;
use std::fs;
use std::path::Path;

pub fn push_u32(bytes: &mut Vec<u8>, value: u32) {
    bytes.extend_from_slice(&value.to_le_bytes());
}

pub fn push_i32(bytes: &mut Vec<u8>, value: i32) {
    bytes.extend_from_slice(&value.to_le_bytes());
}

pub fn write_binary_pack(path: &Path, schema: &str, value: &Value) -> Result<()> {
    let payload = serde_json::to_vec(value)?;
    write_binary_pack_payload(path, schema, &payload)
}

pub fn write_binary_pack_payload(path: &Path, schema: &str, payload: &[u8]) -> Result<()> {
    let schema_bytes = schema.as_bytes();
    let mut bytes = Vec::with_capacity(24 + schema_bytes.len() + payload.len());
    bytes.extend_from_slice(b"NNEIBIN\0");
    bytes.extend_from_slice(&1u32.to_le_bytes());
    bytes.extend_from_slice(&(schema_bytes.len() as u32).to_le_bytes());
    bytes.extend_from_slice(&(payload.len() as u64).to_le_bytes());
    bytes.extend_from_slice(schema_bytes);
    bytes.extend_from_slice(payload);
    fs::write(path, bytes).with_context(|| format!("write {}", path.display()))
}

pub fn intern_compact_string(
    strings: &mut Vec<String>,
    refs: &mut std::collections::HashMap<String, u32>,
    value: Option<String>,
) -> u32 {
    let normalized = value.unwrap_or_default();
    if let Some(existing) = refs.get(&normalized) {
        return *existing;
    }
    let next = strings.len() as u32;
    strings.push(normalized.clone());
    refs.insert(normalized, next);
    next
}
