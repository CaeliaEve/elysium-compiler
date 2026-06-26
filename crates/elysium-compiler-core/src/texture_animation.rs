use crate::json_ext::{numeric_value_u64_lossy, optional_value_string, value_string, value_u64};
use crate::packs::texture::normalize_timeline_frame_index;
use serde_json::{json, Value};
use std::collections::BTreeMap;

pub fn expected_animated_item(animation: Option<&Value>, native_sprite: Option<&Value>) -> bool {
    animation.is_some_and(|value| {
        value_u64(value, "frameCount").unwrap_or(0) > 1
            || value_u64(value, "frameDurationMs").unwrap_or(0) > 0
            || value
                .get("timeline")
                .and_then(Value::as_array)
                .is_some_and(|values| !values.is_empty())
    }) || native_sprite.is_some_and(|value| {
        value_u64(value, "frameCount").unwrap_or(0) > 1
            || value_u64(value, "frameDurationMs").unwrap_or(0) > 0
            || value
                .get("frames")
                .and_then(Value::as_array)
                .is_some_and(|values| values.len() > 1)
            || optional_value_string(Some(value), "animationMode").is_some()
            || optional_value_string(Some(value), "spriteMetadataFile").is_some()
    })
}

pub fn promote_animation_facts_to_animated_atlas(
    atlas: &Value,
    animation_by_asset: &BTreeMap<String, Value>,
    native_sprite_by_asset: &BTreeMap<String, Value>,
) -> Value {
    let mut promoted = atlas.clone();
    let Some(items) = promoted.get_mut("items").and_then(Value::as_array_mut) else {
        return promoted;
    };

    for item in items {
        let asset_id = value_string(item, "assetId").unwrap_or_default();
        let has_animated_atlas = item
            .get("hasAnimatedAtlas")
            .and_then(Value::as_bool)
            .unwrap_or(false);
        if has_animated_atlas {
            continue;
        }
        let animation = animation_by_asset.get(&asset_id);
        let native_sprite = native_sprite_by_asset.get(&asset_id);
        if !expected_animated_item(animation, native_sprite) {
            continue;
        }
        let Some(static_atlas) = item.get("staticAtlas").filter(|value| value.is_object()) else {
            continue;
        };
        let fact = animation.or(native_sprite);
        let frame_count = fact
            .and_then(|value| value_u64(value, "frameCount"))
            .unwrap_or(0);
        if frame_count <= 1 {
            continue;
        }
        let frame_duration_ms = fact
            .and_then(|value| value_u64(value, "frameDurationMs"))
            .unwrap_or(100);
        let x = value_u64(static_atlas, "x").unwrap_or(0);
        let y = value_u64(static_atlas, "y").unwrap_or(0);
        let width = value_u64(static_atlas, "width").unwrap_or(16);
        let height = value_u64(static_atlas, "height").unwrap_or(16);
        let frames = (0..frame_count)
            .map(|index| {
                json!({
                    "index": index,
                    "x": x,
                    "y": y,
                    "width": width,
                    "height": height,
                })
            })
            .collect::<Vec<_>>();
        let timeline = normalize_animation_fact_timeline(fact, frame_count, frame_duration_ms);
        let mut animated_atlas = static_atlas.clone();
        if let Some(object) = animated_atlas.as_object_mut() {
            object.insert("frameCount".to_string(), json!(frame_count));
            object.insert("frameDurationMs".to_string(), json!(frame_duration_ms));
            object.insert("frames".to_string(), Value::Array(frames));
            object.insert("timeline".to_string(), Value::Array(timeline));
            object.insert(
                "animationSource".to_string(),
                json!("native_sprite_fact_promotion"),
            );
        }
        if let Some(object) = item.as_object_mut() {
            object.insert("hasAnimatedAtlas".to_string(), json!(true));
            object.insert("animatedAtlas".to_string(), animated_atlas);
            object.insert(
                "animationPromotion".to_string(),
                json!("native_sprite_fact_promotion"),
            );
        }
    }
    promoted
}

fn normalize_animation_fact_timeline(
    fact: Option<&Value>,
    frame_count: u64,
    fallback_duration_ms: u64,
) -> Vec<Value> {
    if let Some(timeline) = fact
        .and_then(|value| value.get("timeline"))
        .and_then(Value::as_array)
        .filter(|values| !values.is_empty())
    {
        return timeline
            .iter()
            .enumerate()
            .map(|(index, value)| {
                if let Some(pair) = value.as_array() {
                    let frame_index = pair
                        .first()
                        .and_then(numeric_value_u64_lossy)
                        .unwrap_or(index as u64);
                    json!({
                        "frameIndex": normalize_timeline_frame_index(frame_index, frame_count),
                        "durationMs": pair
                            .get(1)
                            .and_then(numeric_value_u64_lossy)
                            .unwrap_or(fallback_duration_ms),
                    })
                } else if let Some(object) = value.as_object() {
                    let frame_index = object
                        .get("frameIndex")
                        .and_then(numeric_value_u64_lossy)
                        .unwrap_or(index as u64);
                    json!({
                        "frameIndex": normalize_timeline_frame_index(frame_index, frame_count),
                        "durationMs": object
                            .get("durationMs")
                            .and_then(numeric_value_u64_lossy)
                            .unwrap_or(fallback_duration_ms),
                    })
                } else {
                    json!({
                        "frameIndex": index as u64,
                        "durationMs": fallback_duration_ms,
                    })
                }
            })
            .collect();
    }
    (0..frame_count)
        .map(|frame_index| {
            json!({
                "frameIndex": frame_index,
                "durationMs": fallback_duration_ms,
            })
        })
        .collect()
}

pub fn expected_animation_reason(
    animation: Option<&Value>,
    native_sprite: Option<&Value>,
) -> &'static str {
    if native_sprite
        .and_then(|value| optional_value_string(Some(value), "spriteMetadataFile"))
        .is_some()
    {
        return "native sprite metadata exists";
    }
    if native_sprite
        .and_then(|value| optional_value_string(Some(value), "animationMode"))
        .is_some()
    {
        return "native sprite animation mode exists";
    }
    if animation
        .and_then(|value| value.get("timeline").and_then(Value::as_array))
        .is_some_and(|values| !values.is_empty())
    {
        return "raw animation timeline exists";
    }
    if animation
        .and_then(|value| value_u64(value, "frameCount"))
        .unwrap_or(0)
        > 1
        || native_sprite
            .and_then(|value| value_u64(value, "frameCount"))
            .unwrap_or(0)
            > 1
    {
        return "frameCount indicates multiple frames";
    }
    "animation timing facts exist"
}
