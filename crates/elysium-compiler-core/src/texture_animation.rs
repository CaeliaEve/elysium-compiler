use crate::json_ext::{numeric_value_u64_lossy, optional_value_string, value_string, value_u64};
use crate::packs::texture::normalize_timeline_frame_index;
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet};

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

pub fn promote_animation_facts_to_animated_atlas_items(
    items: &mut [Value],
    materialization_by_asset: &BTreeMap<String, Value>,
    animation_by_asset: &BTreeMap<String, Value>,
    native_sprite_by_asset: &BTreeMap<String, Value>,
) {
    for item in items {
        let asset_id = value_string(item, "assetId").unwrap_or_default();
        let has_animated_atlas = item
            .get("hasAnimatedAtlas")
            .and_then(Value::as_bool)
            .unwrap_or(false);
        if has_animated_atlas {
            continue;
        }
        let Some(fact) = materialization_by_asset.get(&asset_id) else {
            continue;
        };
        let (frame_duration_ms, frame_duration_source) = resolved_frame_duration(
            fact,
            animation_by_asset.get(&asset_id),
            native_sprite_by_asset.get(&asset_id),
        );
        let EvaluatedAnimationMaterialization::Materialized(mut animated_atlas) =
            evaluate_animation_materialization_with_duration(fact, Some(frame_duration_ms))
        else {
            continue;
        };
        if let Some(object) = animated_atlas.as_object_mut() {
            object.insert(
                "frameDurationSource".to_string(),
                json!(frame_duration_source),
            );
        }
        if let Some(object) = item.as_object_mut() {
            object.insert("hasAnimatedAtlas".to_string(), json!(true));
            object.insert("animatedAtlas".to_string(), animated_atlas);
            object.insert(
                "animationPromotion".to_string(),
                json!("authoritative_animation_frame_materialization"),
            );
        }
    }
}

fn resolved_frame_duration(
    fact: &Value,
    animation: Option<&Value>,
    native_sprite: Option<&Value>,
) -> (u64, &'static str) {
    if let Some(value) = positive_frame_duration(Some(fact)) {
        return (value, "animation_frame_materialization");
    }
    if let Some(value) = positive_frame_duration(animation) {
        return (value, "raw_animation_index");
    }
    if let Some(value) = positive_frame_duration(native_sprite) {
        return (value, "native_sprite_metadata");
    }
    (100, "default_100ms")
}

fn positive_frame_duration(value: Option<&Value>) -> Option<u64> {
    value
        .and_then(|value| value.get("frameDurationMs"))
        .and_then(numeric_value_u64_lossy)
        .filter(|value| *value > 0)
}

enum EvaluatedAnimationMaterialization {
    Materialized(Value),
    CertifiedStatic,
    CertifiedUnavailable,
    Invalid(String),
}

fn required_count(fact: &Value, key: &str) -> Result<u64, String> {
    fact.get(key)
        .and_then(numeric_value_u64_lossy)
        .ok_or_else(|| format!("{key} must be an unsigned integer"))
}

fn materialized_animation_frames(fact: &Value) -> Result<(Vec<Value>, usize), String> {
    let raw_frames = fact
        .get("frames")
        .and_then(Value::as_array)
        .ok_or_else(|| "frames must be an array".to_string())?;
    let mut frames = Vec::with_capacity(raw_frames.len());
    let mut unique_content_hashes = BTreeSet::new();
    for (index, frame) in raw_frames.iter().enumerate() {
        let materialization_index = value_u64(frame, "materializationIndex")
            .ok_or_else(|| "frame materializationIndex must be an unsigned integer".to_string())?;
        if materialization_index != index as u64 {
            return Err("frame materializationIndex must be contiguous and ordered".to_string());
        };
        let frame_index = value_u64(frame, "frameIndex")
            .ok_or_else(|| "frame frameIndex must be an unsigned integer".to_string())?;
        let content_hash = value_string(frame, "contentHash")
            .filter(|value| !value.trim().is_empty())
            .ok_or_else(|| "frame contentHash must be a non-empty string".to_string())?;
        let rect = frame
            .get("rect")
            .filter(|value| value.is_object())
            .ok_or_else(|| "frame rect must be an object".to_string())?;
        let x = value_u64(rect, "x")
            .ok_or_else(|| "frame rect.x must be an unsigned integer".to_string())?;
        let y = value_u64(rect, "y")
            .ok_or_else(|| "frame rect.y must be an unsigned integer".to_string())?;
        let width = value_u64(rect, "width")
            .ok_or_else(|| "frame rect.width must be an unsigned integer".to_string())?;
        let height = value_u64(rect, "height")
            .ok_or_else(|| "frame rect.height must be an unsigned integer".to_string())?;
        if width == 0 || height == 0 {
            return Err("frame width and height must be positive".to_string());
        }
        unique_content_hashes.insert(content_hash.clone());
        frames.push(json!({
            "index": materialization_index,
            "materializationIndex": materialization_index,
            "frameIndex": frame_index,
            "contentHash": content_hash,
            "x": x,
            "y": y,
            "width": width,
            "height": height,
        }));
    }
    Ok((frames, unique_content_hashes.len()))
}

fn evaluate_animation_materialization(fact: &Value) -> EvaluatedAnimationMaterialization {
    evaluate_animation_materialization_with_duration(fact, None)
}

fn evaluate_animation_materialization_with_duration(
    fact: &Value,
    fallback_frame_duration_ms: Option<u64>,
) -> EvaluatedAnimationMaterialization {
    let Some(asset_id) = value_string(fact, "assetId").filter(|value| !value.trim().is_empty())
    else {
        return EvaluatedAnimationMaterialization::Invalid(
            "assetId must be a non-empty string".to_string(),
        );
    };
    let Some(status) = value_string(fact, "materializationStatus") else {
        return EvaluatedAnimationMaterialization::Invalid(
            "materializationStatus is required".to_string(),
        );
    };
    let Some(reason) =
        value_string(fact, "materializationReason").filter(|value| !value.trim().is_empty())
    else {
        return EvaluatedAnimationMaterialization::Invalid(
            "materializationReason must be a non-empty string".to_string(),
        );
    };
    let counts = [
        "runtimeFrameCount",
        "declaredFrameCount",
        "materializedFrameCount",
        "distinctFrameCount",
    ]
    .map(|key| required_count(fact, key));
    let [Ok(runtime_frame_count), Ok(declared_frame_count), Ok(materialized_frame_count), Ok(distinct_frame_count)] =
        counts
    else {
        let error = counts
            .into_iter()
            .find_map(Result::err)
            .unwrap_or_else(|| "animation frame counts are invalid".to_string());
        return EvaluatedAnimationMaterialization::Invalid(error);
    };
    if distinct_frame_count > materialized_frame_count {
        return EvaluatedAnimationMaterialization::Invalid(
            "distinctFrameCount cannot exceed materializedFrameCount".to_string(),
        );
    }

    match status.as_str() {
        "materialized" => {
            if materialized_frame_count < 2 || distinct_frame_count < 2 {
                return EvaluatedAnimationMaterialization::Invalid(
                    "materialized status requires materializedFrameCount>=2 and distinctFrameCount>=2"
                        .to_string(),
                );
            }
            let (frames, observed_distinct_count) = match materialized_animation_frames(fact) {
                Ok(frames) => frames,
                Err(error) => return EvaluatedAnimationMaterialization::Invalid(error),
            };
            if frames.len() as u64 != materialized_frame_count
                || observed_distinct_count as u64 != distinct_frame_count
            {
                return EvaluatedAnimationMaterialization::Invalid(
                    "frame descriptors must exactly match materializedFrameCount and distinctFrameCount"
                        .to_string(),
                );
            }
            let Some(atlas_file) = optional_value_string(Some(fact), "atlasFile") else {
                return EvaluatedAnimationMaterialization::Invalid(
                    "materialized status requires atlasFile".to_string(),
                );
            };
            let frame_duration_ms = positive_frame_duration(Some(fact))
                .or(fallback_frame_duration_ms.filter(|value| *value > 0))
                .unwrap_or(100);
            let timeline = normalize_animation_fact_timeline(
                Some(fact),
                materialized_frame_count,
                frame_duration_ms,
            );
            let (computed_width, computed_height) = animation_frame_extent(&frames);
            EvaluatedAnimationMaterialization::Materialized(json!({
                "atlasFile": atlas_file,
                "atlasWidth": value_u64(fact, "atlasWidth").unwrap_or(computed_width),
                "atlasHeight": value_u64(fact, "atlasHeight").unwrap_or(computed_height),
                "frameCount": materialized_frame_count,
                "runtimeFrameCount": runtime_frame_count,
                "declaredFrameCount": declared_frame_count,
                "materializedFrameCount": materialized_frame_count,
                "distinctFrameCount": distinct_frame_count,
                "frameDurationMs": frame_duration_ms,
                "frames": frames,
                "timeline": timeline,
                "materializationStatus": status,
                "materializationReason": reason,
                "animationSource": "authoritative_animation_frame_materialization",
                "sourceAssetId": asset_id,
            }))
        }
        "static" if materialized_frame_count >= 1 && distinct_frame_count == 1 => {
            EvaluatedAnimationMaterialization::CertifiedStatic
        }
        "static" => EvaluatedAnimationMaterialization::Invalid(
            "static status requires materializedFrameCount>=1 and distinctFrameCount=1".to_string(),
        ),
        "unavailable" if materialized_frame_count == 0 => {
            EvaluatedAnimationMaterialization::CertifiedUnavailable
        }
        "unavailable" => EvaluatedAnimationMaterialization::Invalid(
            "unavailable status requires materializedFrameCount=0".to_string(),
        ),
        _ => EvaluatedAnimationMaterialization::Invalid(format!(
            "unknown materializationStatus {status}; expected materialized, static, or unavailable"
        )),
    }
}

pub fn animation_materialization_diagnostic(
    item_id: &str,
    asset_id: &str,
    fact: &Value,
) -> Option<(Value, bool)> {
    let (code, blocking, validation_error) = match evaluate_animation_materialization(fact) {
        EvaluatedAnimationMaterialization::Materialized(_) => return None,
        EvaluatedAnimationMaterialization::CertifiedStatic => {
            ("ANIMATION_CERTIFIED_STATIC", false, None)
        }
        EvaluatedAnimationMaterialization::CertifiedUnavailable => {
            ("ANIMATION_CERTIFIED_UNAVAILABLE", false, None)
        }
        EvaluatedAnimationMaterialization::Invalid(error) => {
            ("ANIMATION_MATERIALIZATION_INVALID", true, Some(error))
        }
    };
    let mut diagnostic = json!({
        "code": code,
        "itemId": item_id,
        "assetId": asset_id,
        "materializationStatus": fact.get("materializationStatus").cloned().unwrap_or(Value::Null),
        "materializationReason": fact.get("materializationReason").cloned().unwrap_or(Value::Null),
        "runtimeFrameCount": fact.get("runtimeFrameCount").cloned().unwrap_or(Value::Null),
        "declaredFrameCount": fact.get("declaredFrameCount").cloned().unwrap_or(Value::Null),
        "materializedFrameCount": fact.get("materializedFrameCount").cloned().unwrap_or(Value::Null),
        "distinctFrameCount": fact.get("distinctFrameCount").cloned().unwrap_or(Value::Null),
    });
    if let (Some(object), Some(error)) = (diagnostic.as_object_mut(), validation_error) {
        object.insert("validationError".to_string(), json!(error));
    }
    Some((diagnostic, blocking))
}

fn animation_frame_extent(frames: &[Value]) -> (u64, u64) {
    frames.iter().fold((0, 0), |(max_x, max_y), frame| {
        (
            max_x.max(value_u64(frame, "x").unwrap_or(0) + value_u64(frame, "width").unwrap_or(0)),
            max_y.max(value_u64(frame, "y").unwrap_or(0) + value_u64(frame, "height").unwrap_or(0)),
        )
    })
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
