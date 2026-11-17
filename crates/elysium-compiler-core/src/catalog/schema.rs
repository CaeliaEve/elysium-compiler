//! Compose nested internally tagged enums into closed object alternatives.
//! A parent `additionalProperties:false` only sees parent properties in draft 7;
//! it must be applied after both tags and the selected payload have been merged.
use anyhow::{ensure, Result};
use serde_json::Value;

pub(super) fn contract() -> Result<Value> {
    let mut value = serde_json::to_value(schemars::schema_for!(super::Contract))?;
    compose(&mut value)?;
    Ok(value)
}

fn compose(value: &mut Value) -> Result<()> {
    match value {
        Value::Array(rows) => for row in rows { compose(row)?; },
        Value::Object(row) => {
            let parent = row.get("properties").and_then(Value::as_object).cloned();
            if row.get("additionalProperties") == Some(&Value::Bool(false)) {
                if let (Some(properties), Some(Value::Array(branches))) = (parent, row.get_mut("oneOf")) {
                    ensure!(!branches.is_empty(), "tagged object has no schema alternatives");
                    for branch in branches {
                        let branch = branch.as_object_mut().ok_or_else(|| anyhow::anyhow!("tagged object alternative must be an object"))?;
                        ensure!(branch.get("type").and_then(Value::as_str) == Some("object"), "tagged payload must be an object");
                        let fields = branch.entry("properties").or_insert_with(|| serde_json::json!({})).as_object_mut().unwrap();
                        for (name, property) in &properties {
                            ensure!(!fields.contains_key(name), "nested schema redefines outer tag {name}");
                            fields.insert(name.clone(), property.clone());
                        }
                        branch.insert("additionalProperties".into(), Value::Bool(false));
                    }
                    row.remove("additionalProperties");
                }
            }
            for child in row.values_mut() { compose(child)?; }
        }
        _ => {}
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn nested_native_tags_are_composed_without_allowing_unknown_fields() {
        let schema = super::contract().unwrap();
        let branch = schema["definitions"]["Process"]["oneOf"].as_array().unwrap().iter()
            .find(|v| v["properties"]["kind"]["enum"][0] == "binnie_genetics").unwrap();
        assert!(branch.get("additionalProperties").is_none());
        let alternatives=branch["oneOf"].as_array().unwrap();assert_eq!(alternatives.len(),9);
        for variant in alternatives {
            assert_eq!(variant["additionalProperties"],false);
            assert_eq!(variant["properties"]["kind"],branch["properties"]["kind"]);
            assert!(variant["properties"].get("machine").is_some());
        }
    }
}
