//! Owned, pinned BaseItemTickable/decay-chest transition. No wall-clock estimates.
use crate::identity::Nbt;
use anyhow::{ensure, Result};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Batch {
    pub input: Option<Nbt>,
    pub state: Option<Nbt>,
    pub outcome: String,
    pub steps: u8,
}

pub fn batch(threshold: i32, input: &Option<Nbt>) -> Result<Batch> {
    ensure!(threshold > 0, "Decay threshold must be positive");
    if let Some(input) = input {
        input.validate()?;
        ensure!(
            matches!(input, Nbt::Compound { .. }),
            "Decay root must be a compound"
        );
    }
    let mut state = input.clone();
    let mut steps = 0;
    let mut ticking = match ticking(&mut state, threshold)? {
        Some(value) => value,
        None => {
            return Ok(Batch {
                input: input.clone(),
                state,
                outcome: "fault".into(),
                steps,
            })
        }
    };
    let mut result = false;
    while steps < 20 && ticking {
        ticking = self::ticking(&mut state, threshold)?.expect("owned state type cannot change");
        result = tick(&mut state, threshold)?;
        steps += 1;
    }
    Ok(Batch {
        input: input.clone(),
        state,
        outcome: if !ticking && !result {
            "complete"
        } else {
            "ongoing"
        }
        .into(),
        steps,
    })
}

fn create(threshold: i32) -> Nbt {
    Nbt::Compound {
        value: BTreeMap::from([(
            "TickableItem".into(),
            Nbt::Compound {
                value: BTreeMap::from([
                    ("Tick".into(), Nbt::Long { value: "0".into() }),
                    (
                        "maxTick".into(),
                        Nbt::Long {
                            value: threshold.to_string(),
                        },
                    ),
                    ("isActive".into(), Nbt::Byte { value: "1".into() }),
                ]),
            },
        )]),
    }
}
fn ticking(state: &mut Option<Nbt>, threshold: i32) -> Result<Option<bool>> {
    if state.is_none() {
        *state = Some(create(threshold));
        return Ok(Some(true));
    }
    let Some(Nbt::Compound { value: root }) = state else {
        unreachable!()
    };
    Ok(match root.get("TickableItem") {
        None => Some(false),
        Some(Nbt::Compound { value }) => Some(boolean(value.get("isActive"))?),
        Some(_) => None,
    })
}
fn tick(state: &mut Option<Nbt>, threshold: i32) -> Result<bool> {
    let Some(Nbt::Compound { value: root }) = state else {
        *state = Some(create(threshold));
        return Ok(true);
    };
    if !root.contains_key("TickableItem") {
        *state = Some(create(threshold));
        return Ok(true);
    }
    let Some(Nbt::Compound { value }) = root.get_mut("TickableItem") else {
        unreachable!()
    };
    let tick = long(value.get("Tick"))?;
    if i64::from(threshold).wrapping_sub(tick) <= 0 {
        value.insert("isActive".into(), Nbt::Byte { value: "0".into() });
        return Ok(false);
    }
    if !boolean(value.get("isActive"))? {
        return Ok(false);
    }
    value.insert(
        "Tick".into(),
        Nbt::Long {
            value: tick.wrapping_add(1).to_string(),
        },
    );
    Ok(true)
}
fn floating(tag: &Nbt) -> Result<Option<f64>> {
    Ok(match tag {
        Nbt::Float { value } => Some(f64::from(f32::from_bits(u32::from_str_radix(value, 16)?))),
        Nbt::Double { value } => Some(f64::from_bits(u64::from_str_radix(value, 16)?)),
        _ => None,
    })
}
fn long(tag: Option<&Nbt>) -> Result<i64> {
    Ok(match tag {
        Some(
            Nbt::Byte { value } | Nbt::Short { value } | Nbt::Int { value } | Nbt::Long { value },
        ) => value.parse()?,
        Some(tag @ Nbt::Float { .. }) => floating(tag)?.unwrap() as i64,
        Some(tag @ Nbt::Double { .. }) => floating(tag)?.unwrap().floor() as i64,
        _ => 0,
    })
}
fn boolean(tag: Option<&Nbt>) -> Result<bool> {
    Ok(match tag {
        Some(
            Nbt::Byte { value } | Nbt::Short { value } | Nbt::Int { value } | Nbt::Long { value },
        ) => (value.parse::<i64>()? as i8) != 0,
        Some(tag) => {
            if let Some(number) = floating(tag)? {
                let integer = number as i32;
                let floor = if number < f64::from(integer) {
                    integer.wrapping_sub(1)
                } else {
                    integer
                };
                (floor as i8) != 0
            } else {
                false
            }
        }
        None => false,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn actual_native_saved_states_numeric_coercions_and_overflow_match() {
        #[derive(Deserialize)]
        struct Observed {
            threshold: i32,
            #[serde(flatten)]
            batch: Batch,
        }
        let observations: Vec<Observed> = serde_json::from_str(include_str!(
            "../../../../contracts/fixtures/native-decay-state-parity.json"
        ))
        .unwrap();
        assert_eq!(observations.len(), 336);
        for observed in observations {
            assert_eq!(
                batch(observed.threshold, &observed.batch.input).unwrap(),
                observed.batch,
                "threshold={}",
                observed.threshold
            );
        }
    }
}
