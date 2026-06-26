use anyhow::{Context, Result};
use serde_json::Value;
use std::fs;
use std::path::Path;

pub fn first_non_empty(values: &[Option<String>]) -> Option<String> {
    values
        .iter()
        .flatten()
        .map(|value| value.trim().to_string())
        .find(|value| !value.is_empty())
}

pub fn nested_value_string(value: &Value, path: &[&str]) -> Option<String> {
    let mut current = value;
    for key in path {
        current = current.get(*key)?;
    }
    current.as_str().map(str::to_string)
}

pub fn read_json_file(path: &Path) -> Result<Value> {
    let text = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    serde_json::from_str(&text).with_context(|| format!("parse {}", path.display()))
}

pub fn json_array(value: &Value, key: &str) -> Vec<Value> {
    value
        .get(key)
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default()
}

pub fn value_string(value: &Value, key: &str) -> Option<String> {
    value.get(key)?.as_str().map(str::to_string)
}

pub fn value_i64(value: &Value, key: &str) -> Option<i64> {
    value.get(key)?.as_i64()
}

pub fn value_u64(value: &Value, key: &str) -> Option<u64> {
    numeric_value_u64(value.get(key)?)
}

pub fn optional_value_string(value: Option<&Value>, key: &str) -> Option<String> {
    value?.get(key)?.as_str().map(str::to_string)
}

pub fn optional_value_u64(value: Option<&Value>, key: &str) -> Option<u64> {
    value_u64(value?, key)
}

pub fn numeric_value_u64(value: &Value) -> Option<u64> {
    match value {
        Value::Number(number) => number.as_u64(),
        Value::String(text) => text.trim().parse::<u64>().ok(),
        Value::Object(object) => object.get("value").and_then(numeric_value_u64),
        _ => None,
    }
}

pub fn numeric_value_u64_lossy(value: &Value) -> Option<u64> {
    match value {
        Value::Number(number) => number
            .as_u64()
            .or_else(|| number.as_f64().map(|value| value as u64)),
        Value::String(text) => text.trim().parse::<u64>().ok(),
        Value::Object(object) => object.get("value").and_then(numeric_value_u64_lossy),
        _ => None,
    }
}
