use anyhow::{ensure, Context, Result};
use base64::{engine::general_purpose::STANDARD, Engine};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum Nbt {
    Byte { value: String },
    Short { value: String },
    Int { value: String },
    Long { value: String },
    Float { value: String },
    Double { value: String },
    ByteArray { value: String },
    String { value: String },
    List { element: String, value: Vec<Nbt> },
    Compound { value: BTreeMap<String, Nbt> },
    IntArray { value: Vec<String> },
}

impl Nbt {
    pub fn validate(&self) -> Result<()> {
        self.validate_node(0, &mut 0)?;
        ensure!(
            serde_json::to_vec(self)?.len() <= 512 * 1024,
            "NBT exceeds 512 KiB"
        );
        Ok(())
    }

    fn name(&self) -> &str {
        match self {
            Self::Byte { .. } => "byte",
            Self::Short { .. } => "short",
            Self::Int { .. } => "int",
            Self::Long { .. } => "long",
            Self::Float { .. } => "float",
            Self::Double { .. } => "double",
            Self::ByteArray { .. } => "byte_array",
            Self::String { .. } => "string",
            Self::List { .. } => "list",
            Self::Compound { .. } => "compound",
            Self::IntArray { .. } => "int_array",
        }
    }

    fn validate_node(&self, depth: u32, nodes: &mut u32) -> Result<()> {
        *nodes += 1;
        ensure!(
            depth <= 48 && *nodes <= 65536,
            "NBT exceeds nesting or node limit"
        );
        match self {
            Self::Byte { value } => {
                integer(value, i8::MIN as i64, i8::MAX as i64)?;
            }
            Self::Short { value } => {
                integer(value, i16::MIN as i64, i16::MAX as i64)?;
            }
            Self::Int { value } => {
                integer(value, i32::MIN as i64, i32::MAX as i64)?;
            }
            Self::Long { value } => {
                integer(value, i64::MIN, i64::MAX)?;
            }
            Self::Float { value } => bits(value, 8)?,
            Self::Double { value } => bits(value, 16)?,
            Self::ByteArray { value } => {
                ensure!(value.len() <= 350_000, "NBT byte array exceeds size limit");
                let bytes = STANDARD.decode(value).context("decode NBT byte array")?;
                ensure!(
                    bytes.len() <= 256 * 1024 && STANDARD.encode(bytes) == *value,
                    "invalid NBT byte array"
                );
            }
            Self::String { value } => ensure!(
                value.encode_utf16().count() <= 131072,
                "NBT string exceeds size limit"
            ),
            Self::IntArray { value } => {
                ensure!(value.len() <= 65536, "NBT int array exceeds size limit");
                for value in value {
                    integer(value, i32::MIN as i64, i32::MAX as i64)?;
                }
            }
            Self::List { element, value } => {
                ensure!(
                    matches!(
                        element.as_str(),
                        "end"
                            | "byte"
                            | "short"
                            | "int"
                            | "long"
                            | "float"
                            | "double"
                            | "byte_array"
                            | "string"
                            | "list"
                            | "compound"
                            | "int_array"
                    ),
                    "unknown NBT list element type"
                );
                for child in value {
                    ensure!(child.name() == element, "heterogeneous NBT list");
                    child.validate_node(depth + 1, nodes)?;
                }
            }
            Self::Compound { value } => {
                for (key, child) in value {
                    ensure!(key.len() <= 65535, "NBT compound key exceeds size limit");
                    child.validate_node(depth + 1, nodes)?;
                }
            }
        }
        Ok(())
    }
}

pub fn item_id(registry: &str, meta: i32, nbt: Option<&Nbt>) -> Result<String> {
    resource_name(registry)?;
    if let Some(nbt) = nbt {
        nbt.validate()?;
    }
    let key = json!({ "kind": "item", "registry": registry, "meta": meta, "nbt": nbt });
    Ok(format!(
        "item_{:x}",
        Sha256::digest(serde_json::to_vec(&key)?)
    ))
}

pub fn fluid_id(registry: &str, nbt: Option<&Nbt>) -> Result<String> {
    ensure!(
        !registry.is_empty()
            && registry.len() <= 256
            && registry.bytes().all(|byte| byte.is_ascii_alphanumeric()
                || matches!(byte, b'_' | b'.' | b'-' | b'/' | b':')),
        "invalid Forge fluid registry name: {registry}"
    );
    if let Some(nbt) = nbt {
        nbt.validate()?;
    }
    let key = json!({ "kind": "fluid", "registry": registry, "nbt": nbt });
    Ok(format!(
        "fluid_{:x}",
        Sha256::digest(serde_json::to_vec(&key)?)
    ))
}

pub fn resource_name(value: &str) -> Result<()> {
    let (namespace, name) = value
        .split_once(':')
        .context("resource name requires a namespace")?;
    let identifier = |byte: u8| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'.' | b'-');
    ensure!(
        !namespace.is_empty()
            && namespace.bytes().all(identifier)
            && !name.is_empty()
            && name.bytes().all(|byte| identifier(byte) || byte == b'/'),
        "invalid registered resource name: {value}"
    );
    Ok(())
}

pub fn integer(value: &str, minimum: i64, maximum: i64) -> Result<i64> {
    let number: i64 = value.parse().context("expected an exact decimal integer")?;
    ensure!(
        number.to_string() == value && number >= minimum && number <= maximum,
        "integer is not canonical or outside range: {value}"
    );
    Ok(number)
}

fn bits(value: &str, width: usize) -> Result<()> {
    ensure!(
        value.len() == width
            && value
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte)),
        "invalid IEEE bit pattern"
    );
    Ok(())
}
