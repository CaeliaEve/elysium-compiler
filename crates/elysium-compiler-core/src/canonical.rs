//! Native Source canonical JSON: ordered UTF-8 keys and bounded plain decimal numbers.
use anyhow::{ensure, Result};
use serde_json::Value;

// serde_json::to_value widens f32 to f64 before choosing its decimal text.
// Its stream serializer preserves the native f32 shortest decimal instead,
// matching Java Float.toString and keeping native record identities intact.
pub(crate) fn value<T: serde::Serialize + ?Sized>(value: &T) -> Result<Value> {
    Ok(serde_json::from_slice(&serde_json::to_vec(value)?)?)
}

pub(crate) fn bytes(value: &Value) -> Result<Vec<u8>> {
    let mut output = Vec::new();
    append(value, &mut output, 0)?;
    Ok(output)
}

fn append(value: &Value, out: &mut Vec<u8>, depth: usize) -> Result<()> {
    ensure!(depth <= 64, "canonical JSON exceeds nesting limit");
    match value {
        Value::Number(number) => out.extend_from_slice(decimal(&number.to_string())?.as_bytes()),
        Value::Array(rows) => {
            out.push(b'[');
            for (index, row) in rows.iter().enumerate() {
                if index > 0 { out.push(b','); }
                append(row, out, depth + 1)?;
            }
            out.push(b']');
        }
        Value::Object(rows) => {
            out.push(b'{');
            let mut keys: Vec<_> = rows.keys().collect();
            keys.sort_unstable_by(|a, b| a.as_bytes().cmp(b.as_bytes()));
            for (index, key) in keys.into_iter().enumerate() {
                if index > 0 { out.push(b','); }
                out.extend_from_slice(&serde_json::to_vec(key)?);
                out.push(b':');
                append(&rows[key], out, depth + 1)?;
            }
            out.push(b'}');
        }
        _ => out.extend_from_slice(&serde_json::to_vec(value)?),
    }
    Ok(())
}

fn decimal(text: &str) -> Result<String> {
    let negative = text.starts_with('-');
    let unsigned = text.strip_prefix('-').unwrap_or(text);
    let (mantissa, exponent) = unsigned.split_once(['e', 'E'])
        .map_or(Ok((unsigned, 0)), |(a, b)| b.parse::<i32>().map(|b| (a, b)))?;
    let (integer, fraction) = mantissa.split_once('.').unwrap_or((mantissa, ""));
    let raw = format!("{integer}{fraction}");
    let leading = raw.trim_start_matches('0');
    if leading.is_empty() { return Ok("0".into()); }
    let digits = leading.trim_end_matches('0');
    let scale = fraction.len() as i32 - exponent - (leading.len() - digits.len()) as i32;
    ensure!(digits.len() <= 100 && scale.abs() <= 100, "JSON number exceeds native canonical range");
    let point = digits.len() as i32 - scale;
    let number = if scale <= 0 {
        format!("{digits}{}", "0".repeat((-scale) as usize))
    } else if point <= 0 {
        format!("0.{}{digits}", "0".repeat((-point) as usize))
    } else {
        format!("{}.{}", &digits[..point as usize], &digits[point as usize..])
    };
    Ok(if negative { format!("-{number}") } else { number })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn native_decimals_preserve_fractional_values_and_normalize_integer_floats() {
        for (raw, expected) in [("1.0", "1"), ("-0.0", "0"), ("1e-7", "0.0000001"),
            ("1.1400", "1.14"), ("1e30", "1000000000000000000000000000000"),
            ("-1250.0500", "-1250.05")] {
            assert_eq!(decimal(raw).unwrap(), expected);
        }
        for raw in ["1e-101", "1e101"] { assert!(decimal(raw).is_err()); }
        let value = serde_json::json!({"z": -0.0, "coords": [0.5, 1.0, 0.5], "rate": 1e-7});
        assert_eq!(bytes(&value).unwrap(), br#"{"coords":[0.5,1,0.5],"rate":0.0000001,"z":0}"#);
        assert_eq!(super::value(&0.37f32).unwrap(), serde_json::json!(0.37));
    }
}
