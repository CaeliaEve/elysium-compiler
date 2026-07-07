use crate::io::normalize_path;
use crate::json_ext::{first_non_empty, nested_value_string, value_string, value_u64};
use crate::manifest::{
    portable_relative_path, read_jsonl_file_values, read_jsonl_values, read_manifest_collection,
    read_manifest_json, RawManifest, COLLECTION_HANDLER_LAYOUTS,
};
use crate::recipe_domain::{
    captured_ui_family_key, classify_recipe_family_key, public_recipe_handler,
    public_recipe_layout, recipe_id, RecipeHandlerContext,
};
use anyhow::{anyhow, Context, Result};
use serde_json::json;
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use std::fs::{self, File};
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};

pub fn encode_recipe_file_name(value: &str) -> String {
    value
        .chars()
        .map(|character| match character {
            'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_' | '.' | '~' => character,
            _ => '_',
        })
        .collect()
}

pub fn rust_recipe_ui_payload_relative_path(recipe_id: &str) -> String {
    let shard = sha1_hex_prefix(recipe_id.as_bytes(), 2);
    format!("recipes/ui-payload-shards/{shard}.json")
}

pub const NATIVE_NEI_FRAME_ASSET_PREFIX: &str = "assets/nei-native-frames/";

pub fn public_recipe_native_frame(
    recipe_id: &str,
    recipe: &Value,
    public_layout: Option<&Value>,
    public_handler: Option<&Value>,
) -> Option<Value> {
    let source_value = recipe
        .get("nativeFrame")
        .or_else(|| recipe.get("nativeNeiFrame"))
        .or_else(|| recipe.get("metadata").and_then(|metadata| metadata.get("nativeFrame")))
        .or_else(|| {
            recipe
                .get("additionalData")
                .and_then(|additional_data| additional_data.get("nativeFrame"))
        })?;
    let source = source_value.as_object()?;
    let status = value_string(source_value, "status").unwrap_or_else(|| "captured".to_string());
    let asset_ref = source
        .get("assetRef")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())?
        .to_string();
    let width = source
        .get("width")
        .and_then(Value::as_u64)
        .or_else(|| public_layout.and_then(|layout| value_u64(layout, "width")))
        .unwrap_or(166);
    let height = source
        .get("height")
        .and_then(Value::as_u64)
        .or_else(|| public_layout.and_then(|layout| value_u64(layout, "height")))
        .unwrap_or(65);
    let coordinate_space = source
        .get("coordinateSpace")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("nei_pixels");

    let mut frame = serde_json::Map::new();
    for (key, value) in source {
        frame.insert(key.clone(), value.clone());
    }
    frame.insert("status".to_string(), Value::String(status));
    frame.insert("assetRef".to_string(), Value::String(asset_ref));
    frame.insert("width".to_string(), json!(width));
    frame.insert("height".to_string(), json!(height));
    frame.insert(
        "coordinateSpace".to_string(),
        Value::String(coordinate_space.to_string()),
    );
    frame
        .entry("source".to_string())
        .or_insert_with(|| Value::String("in-game-nei-render".to_string()));
    frame
        .entry("recipeId".to_string())
        .or_insert_with(|| Value::String(recipe_id.to_string()));
    if let Some(handler_key) =
        public_handler.and_then(|handler| value_string(handler, "handlerKey"))
    {
        frame
            .entry("handlerKey".to_string())
            .or_insert_with(|| Value::String(handler_key));
    }
    if let Some(handler_class) =
        public_handler.and_then(|handler| value_string(handler, "handlerClass"))
    {
        frame
            .entry("handlerClass".to_string())
            .or_insert_with(|| Value::String(handler_class));
    }
    Some(Value::Object(frame))
}

pub fn materialize_native_frame_assets(
    input: &Path,
    output: &Path,
    recipe_ui_index: &[Value],
) -> Result<Value> {
    let mut assets_by_ref = BTreeMap::<String, BTreeSet<String>>::new();
    let mut malformed = Vec::new();
    for entry in recipe_ui_index {
        let recipe_id = value_string(entry, "recipeId").unwrap_or_default();
        let Some(frame) = entry.get("nativeFrame").filter(|value| value.is_object()) else {
            malformed.push(json!({
                "recipeId": recipe_id,
                "reason": "native-frame-payload-missing",
                "required": "nativeFrame",
            }));
            continue;
        };
        let status = value_string(frame, "status").unwrap_or_default();
        let asset_ref = value_string(frame, "assetRef").unwrap_or_default();
        if status != "captured" {
            malformed.push(json!({
                "recipeId": recipe_id,
                "assetRef": asset_ref,
                "reason": "native-frame-status-not-captured",
                "status": status,
            }));
            continue;
        }
        if asset_ref.trim().is_empty() {
            malformed.push(json!({
                "recipeId": recipe_id,
                "reason": "native-frame-asset-ref-missing",
            }));
            continue;
        }
        assets_by_ref
            .entry(asset_ref)
            .or_default()
            .insert(recipe_id);
    }

    let mut copied = Vec::new();
    let mut missing = malformed;
    for (asset_ref, recipe_ids) in assets_by_ref {
        if !asset_ref.starts_with(NATIVE_NEI_FRAME_ASSET_PREFIX) {
            missing.push(json!({
                "assetRef": asset_ref,
                "recipeIds": recipe_ids.into_iter().collect::<Vec<_>>(),
                "reason": "native-frame-asset-outside-prefix",
                "expectedPrefix": NATIVE_NEI_FRAME_ASSET_PREFIX,
            }));
            continue;
        }
        let Some(relative) = portable_relative_path(&asset_ref) else {
            missing.push(json!({
                "assetRef": asset_ref,
                "recipeIds": recipe_ids.into_iter().collect::<Vec<_>>(),
                "reason": "non-portable-path",
            }));
            continue;
        };
        let source = input.join(&relative);
        if !source.is_file() {
            missing.push(json!({
                "assetRef": asset_ref,
                "recipeIds": recipe_ids.into_iter().collect::<Vec<_>>(),
                "path": normalize_path(relative.as_path()),
                "reason": "raw-export-native-frame-asset-missing",
            }));
            continue;
        }
        let target = output.join(&relative);
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::copy(&source, &target).with_context(|| {
            format!(
                "copy native NEI frame asset {} -> {}",
                source.display(),
                target.display()
            )
        })?;
        copied.push(json!({
            "assetRef": asset_ref,
            "recipeIds": recipe_ids.into_iter().collect::<Vec<_>>(),
            "path": normalize_path(relative.as_path()),
        }));
    }
    Ok(json!({
        "schemaVersion": "neonei/native-nei-frame-assets/current",
        "assetPrefix": NATIVE_NEI_FRAME_ASSET_PREFIX,
        "copied": copied,
        "missing": missing,
    }))
}

pub fn read_compiled_recipe_ui_payload_index(output: &Path) -> Result<Option<Vec<Value>>> {
    let path = output.join("recipes").join("ui-payload-index.json");
    if !path.exists() {
        return Ok(None);
    }
    let text = fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    let value: Value =
        serde_json::from_str(&text).with_context(|| format!("parse {}", path.display()))?;
    Ok(Some(
        value
            .get("recipes")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default(),
    ))
}

pub fn build_raw_recipe_ui_payload_index(
    input: &Path,
    manifest: &RawManifest,
) -> Result<Vec<Value>> {
    let recipe_index = read_manifest_json(input, manifest, "recipeIndex")?
        .ok_or_else(|| anyhow!("ui-pack compiler blocked: recipeIndex is missing"))?;
    let handlers = read_jsonl_values(input, manifest, "neiHandlers")?;
    let layouts = read_manifest_collection(input, manifest, COLLECTION_HANDLER_LAYOUTS)?;
    let handler_context = RecipeHandlerContext::new(&handlers, &layouts);
    let mut recipes = Vec::new();
    for shard in recipe_index
        .get("shards")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default()
    {
        let Some(path) = shard.get("path").and_then(Value::as_str) else {
            continue;
        };
        let shard_path = input.join(path.replace('\\', "/").trim_start_matches('/'));
        recipes.extend(read_jsonl_file_values(&shard_path)?);
    }

    let mut entries = Vec::with_capacity(recipes.len());
    for recipe in &recipes {
        let recipe_id = recipe_id(recipe);
        let (handler, layout) = handler_context.resolve(recipe);
        let public_handler = handler.map(public_recipe_handler);
        let public_layout = layout
            .map(public_recipe_layout)
            .or_else(|| recipe.get("nativeLayout").cloned());
        let native_frame = public_recipe_native_frame(
            &recipe_id,
            recipe,
            public_layout.as_ref(),
            public_handler.as_ref(),
        );
        let raw_family_key = first_non_empty(&[
            value_string(recipe, "family"),
            value_string(recipe, "sourcePlugin"),
            value_string(recipe, "recipeType"),
            nested_value_string(recipe, &["machine", "machineId"]),
        ])
        .unwrap_or_else(|| "unknown".to_string());
        let family_key = captured_ui_family_key(handler, layout)
            .unwrap_or_else(|| classify_recipe_family_key(recipe, &raw_family_key, handler));
        let recipe_type = first_non_empty(&[
            value_string(recipe, "recipeType"),
            nested_value_string(recipe, &["machine", "machineId"]),
            Some(family_key.clone()),
        ])
        .unwrap_or_else(|| family_key.clone());
        let machine_type = first_non_empty(&[
            public_handler
                .as_ref()
                .and_then(|handler| value_string(handler, "localizedName")),
            public_handler
                .as_ref()
                .and_then(|handler| value_string(handler, "displayName")),
            nested_value_string(recipe, &["machine", "displayName"]),
            value_string(recipe, "displayName"),
            nested_value_string(recipe, &["machine", "machineId"]),
            Some(recipe_type.clone()),
        ])
        .unwrap_or_else(|| recipe_type.clone());
        let handler_key = public_handler
            .as_ref()
            .and_then(|handler| value_string(handler, "handlerKey"));
        let mut entry = json!({
            "recipeId": recipe_id,
            "path": rust_recipe_ui_payload_relative_path(&recipe_id),
            "payloadKey": recipe_id,
            "familyKey": family_key,
            "recipeType": recipe_type,
            "machineType": machine_type,
            "handlerKey": handler_key,
            "nativeLayout": public_layout,
        });
        if let Some(native_frame) = native_frame {
            if let Some(object) = entry.as_object_mut() {
                object.insert("nativeFrame".to_string(), native_frame);
            }
        }
        entries.push(entry);
    }
    Ok(entries)
}

struct RecipeUiPayloadShardWriter {
    writer: BufWriter<File>,
    first_payload: bool,
}

pub struct RecipeUiPayloadShardWriters {
    output: PathBuf,
    writers: BTreeMap<String, RecipeUiPayloadShardWriter>,
}

impl RecipeUiPayloadShardWriters {
    pub fn new(output: &Path) -> Result<Self> {
        fs::create_dir_all(output.join("recipes").join("ui-payload-shards"))?;
        Ok(Self {
            output: output.to_path_buf(),
            writers: BTreeMap::new(),
        })
    }

    pub fn write_payload(&mut self, recipe_id: &str, payload: &Value) -> Result<()> {
        let shard_path = rust_recipe_ui_payload_relative_path(recipe_id);
        if !self.writers.contains_key(&shard_path) {
            let absolute_path = self.output.join(&shard_path);
            if let Some(parent) = absolute_path.parent() {
                fs::create_dir_all(parent)?;
            }
            let file = File::create(&absolute_path).with_context(|| {
                format!("create recipe UI payload shard {}", absolute_path.display())
            })?;
            let mut writer = BufWriter::new(file);
            writer.write_all(
                b"{\n  \"schemaVersion\": \"neonei/recipe-ui-payload-shard/v1\",\n  \"payloads\": {",
            )?;
            self.writers.insert(
                shard_path.clone(),
                RecipeUiPayloadShardWriter {
                    writer,
                    first_payload: true,
                },
            );
        }
        let shard = self
            .writers
            .get_mut(&shard_path)
            .ok_or_else(|| anyhow!("recipe UI payload shard writer disappeared: {shard_path}"))?;
        if shard.first_payload {
            shard.writer.write_all(b"\n")?;
            shard.first_payload = false;
        } else {
            shard.writer.write_all(b",\n")?;
        }
        write!(shard.writer, "    {}: ", serde_json::to_string(recipe_id)?)?;
        serde_json::to_writer(&mut shard.writer, payload)?;
        Ok(())
    }

    pub fn finish(self) -> Result<()> {
        for (shard_path, mut shard) in self.writers {
            if shard.first_payload {
                shard.writer.write_all(b"\n")?;
            }
            shard.writer.write_all(b"  }\n}\n")?;
            shard
                .writer
                .flush()
                .with_context(|| format!("flush recipe UI payload shard {shard_path}"))?;
        }
        Ok(())
    }
}

pub fn sha1_hex_prefix(bytes: &[u8], hex_len: usize) -> String {
    let mut h0: u32 = 0x6745_2301;
    let mut h1: u32 = 0xefcd_ab89;
    let mut h2: u32 = 0x98ba_dcfe;
    let mut h3: u32 = 0x1032_5476;
    let mut h4: u32 = 0xc3d2_e1f0;

    let bit_len = (bytes.len() as u64).wrapping_mul(8);
    let mut message = bytes.to_vec();
    message.push(0x80);
    while message.len() % 64 != 56 {
        message.push(0);
    }
    message.extend_from_slice(&bit_len.to_be_bytes());

    for chunk in message.chunks_exact(64) {
        let mut words = [0u32; 80];
        for (index, word) in words.iter_mut().take(16).enumerate() {
            let offset = index * 4;
            *word = u32::from_be_bytes([
                chunk[offset],
                chunk[offset + 1],
                chunk[offset + 2],
                chunk[offset + 3],
            ]);
        }
        for index in 16..80 {
            words[index] =
                (words[index - 3] ^ words[index - 8] ^ words[index - 14] ^ words[index - 16])
                    .rotate_left(1);
        }

        let mut a = h0;
        let mut b = h1;
        let mut c = h2;
        let mut d = h3;
        let mut e = h4;
        for (index, word) in words.iter().enumerate() {
            let (f, k) = match index {
                0..=19 => ((b & c) | ((!b) & d), 0x5a82_7999),
                20..=39 => (b ^ c ^ d, 0x6ed9_eba1),
                40..=59 => ((b & c) | (b & d) | (c & d), 0x8f1b_bcdc),
                _ => (b ^ c ^ d, 0xca62_c1d6),
            };
            let temp = a
                .rotate_left(5)
                .wrapping_add(f)
                .wrapping_add(e)
                .wrapping_add(k)
                .wrapping_add(*word);
            e = d;
            d = c;
            c = b.rotate_left(30);
            b = a;
            a = temp;
        }
        h0 = h0.wrapping_add(a);
        h1 = h1.wrapping_add(b);
        h2 = h2.wrapping_add(c);
        h3 = h3.wrapping_add(d);
        h4 = h4.wrapping_add(e);
    }

    let hex = format!("{h0:08x}{h1:08x}{h2:08x}{h3:08x}{h4:08x}");
    hex.chars().take(hex_len).collect()
}
