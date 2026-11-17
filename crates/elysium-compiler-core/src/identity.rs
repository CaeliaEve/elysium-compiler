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
    /// Minecraft 1.7.10 hashCode for cache routing, never content identity.
    /// Native equality and hashing even disagree on signed floating zero.
    pub fn native_hash(&self) -> Result<i32> {
        self.validate()?;
        self.native_hash_node()
    }

    fn native_hash_node(&self) -> Result<i32> {
        let text_hash = |text: &str| text.encode_utf16().fold(0i32, |hash, ch| hash.wrapping_mul(31).wrapping_add(ch as i32));
        let long_hash = |bits: u64| (bits ^ (bits >> 32)) as i32;
        Ok(match self {
            Self::Byte {value} => 1 ^ value.parse::<i8>()? as i32,
            Self::Short {value} => 2 ^ value.parse::<i16>()? as i32,
            Self::Int {value} => 3 ^ value.parse::<i32>()?,
            Self::Long {value} => 4 ^ long_hash(value.parse::<i64>()? as u64),
            Self::Float {value} => {
                let mut bits=u32::from_str_radix(value,16)?;
                if bits & 0x7f800000 == 0x7f800000 && bits & 0x007fffff != 0 {bits=0x7fc00000;}
                5 ^ bits as i32
            }
            Self::Double {value} => {
                let mut bits=u64::from_str_radix(value,16)?;
                if bits & 0x7ff0000000000000 == 0x7ff0000000000000 && bits & 0x000fffffffffffff != 0 {bits=0x7ff8000000000000;}
                6 ^ long_hash(bits)
            }
            Self::ByteArray {value} => 7 ^ STANDARD.decode(value)?.iter().fold(1i32,|hash,byte|hash.wrapping_mul(31).wrapping_add(*byte as i8 as i32)),
            Self::String {value} => 8 ^ text_hash(value),
            Self::List {value,..} => {
                let mut hash=1i32;
                for cell in value {hash=hash.wrapping_mul(31).wrapping_add(cell.native_hash_node()?);}
                9 ^ hash // Native list hashing omits the declared element type.
            }
            Self::Compound {value} => {
                let mut hash=0i32;
                for (key,cell) in value {hash=hash.wrapping_add(text_hash(key)^cell.native_hash_node()?);}
                10 ^ hash
            }
            Self::IntArray {value} => {
                let mut hash=1i32;
                for integer in value {hash=hash.wrapping_mul(31).wrapping_add(integer.parse::<i32>()?);}
                11 ^ hash
            }
        })
    }

    /// Minecraft's NBTBase.equals for DISTINCT owned trees, not content identity.
    /// IEEE signed zeros compare equal, NaNs do not, and empty list element types matter.
    /// Keep derived PartialEq and item_id lossless: native matching must not change hashes.
    pub fn native_equal(&self, other: &Self) -> Result<bool> {
        self.validate()?;
        other.validate()?;
        self.native_equal_node(other)
    }

    fn native_equal_node(&self, other: &Self) -> Result<bool> {
        Ok(match (self, other) {
            (Self::Float { value: a }, Self::Float { value: b }) =>
                f32::from_bits(u32::from_str_radix(a, 16)?) == f32::from_bits(u32::from_str_radix(b, 16)?),
            (Self::Double { value: a }, Self::Double { value: b }) =>
                f64::from_bits(u64::from_str_radix(a, 16)?) == f64::from_bits(u64::from_str_radix(b, 16)?),
            (Self::List { element: a_type, value: a }, Self::List { element: b_type, value: b }) => {
                if a_type != b_type || a.len() != b.len() { return Ok(false); }
                for (left, right) in a.iter().zip(b) { if !left.native_equal_node(right)? { return Ok(false); } }
                true
            }
            (Self::Compound { value: a }, Self::Compound { value: b }) => {
                if a.len() != b.len() { return Ok(false); }
                for (key, left) in a {
                    let Some(right) = b.get(key) else { return Ok(false); };
                    if !left.native_equal_node(right)? { return Ok(false); }
                }
                true
            }
            _ => self == other,
        })
    }

    pub fn validate(&self) -> Result<()> {
        self.validate_node(0, &mut 0)?;
        ensure!(
            serde_json::to_vec(self)?.len() <= 512 * 1024,
            "NBT exceeds 512 KiB"
        );
        Ok(())
    }

    pub fn name(&self) -> &str {
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
    registry_name(registry)?;
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
        !registry.is_empty(),
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

// Registry names are native identity data, never paths. The first colon
// separates an item/block namespace; the rest is preserved verbatim.
pub fn registry_name(value: &str) -> Result<()> {
    let (namespace, name) = value
        .split_once(':')
        .context("registry name requires a namespace")?;
    ensure!(
        !namespace.is_empty() && !name.is_empty(),
        "invalid namespaced Forge registry key: {value}"
    );
    Ok(())
}

pub fn property_key(value: &str) -> Result<()> {
    let (namespace, name) = value
        .split_once(':')
        .context("property key requires a namespace")?;
    let identifier = |byte: u8| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'.' | b'-');
    ensure!(
        !namespace.is_empty()
            && namespace.bytes().all(identifier)
            && !name.is_empty()
            && name
                .bytes()
                .all(|byte| identifier(byte) || matches!(byte, b'/' | b'$')),
        "invalid property key: {value}"
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
