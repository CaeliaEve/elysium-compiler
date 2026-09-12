use super::{Item, Kind, Output, Recipe};
use crate::identity::{integer, item_id, Nbt};
use anyhow::{ensure, Context, Result};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Stack {
    pub id: String,
    pub amount: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "lowercase", deny_unknown_fields)]
pub enum Edit {
    /// Copy the input item, metadata, count and tags. Replace root tags, then cap
    /// numeric tags using Minecraft getInteger semantics (missing/non-numeric = 0).
    Patch {
        set: BTreeMap<String, Nbt>,
        limits: BTreeMap<String, i32>,
    },
    /// TC4Tweaks inheritance: output scalars win, compounds merge and equal-type lists append.
    /// An untagged base instead inherits input metadata/count and the entire input compound.
    Merge {
        base: Stack,
        /// None copies all root keys. A nonempty list filters only the root merge.
        keys: Option<Vec<String>>,
        /// Both input and base must be armor or have at least one native tool class.
        tools: bool,
    },
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Change {
    pub input: u32,
    pub action: Edit,
    /// One native result per input choice, in matching order; excluded from recipe identity.
    pub samples: Vec<Stack>,
}

pub(super) fn validate(
    recipe: &Recipe,
    output: &Output,
    items: &BTreeMap<&str, &Item>,
) -> Result<()> {
    let Some(change) = &output.change else {
        return Ok(());
    };
    ensure!(
        output.kind == Kind::Item,
        "only item outputs can change NBT"
    );
    let input = recipe
        .inputs
        .iter()
        .find(|input| input.kind == Kind::Item && input.slot == change.input)
        .context("output change refers to a missing item input")?;
    ensure!(
        change.samples.len() == input.choices.len(),
        "output change samples omit input choices"
    );
    match &change.action {
        Edit::Patch { set, limits } => {
            ensure!(!set.is_empty() || !limits.is_empty(), "empty output patch");
            ensure!(
                set.len() + limits.len() <= 4096,
                "output patch exceeds its budget"
            );
            Nbt::Compound { value: set.clone() }.validate()?;
            ensure!(
                limits
                    .keys()
                    .all(|key| key.len() <= 65535 && !set.contains_key(key)),
                "output patch has an invalid or overlapping limit"
            );
        }
        Edit::Merge { base, keys, .. } => {
            let base_item = items
                .get(base.id.as_str())
                .context("missing change base item")?;
            integer(&base.amount, 1, i64::MAX)?;
            if let Some(keys) = keys {
                ensure!(
                    !keys.is_empty()
                        && keys.len() <= 4096
                        && keys.windows(2).all(|pair| pair[0] < pair[1])
                        && keys.iter().all(|key| key.len() <= 65535),
                    "invalid inheritance key filter"
                );
                // The pinned API dereferences a null output tag in this branch. Do not invent a successful result.
                ensure!(
                    base_item.nbt.is_some(),
                    "native filtered inheritance into an untagged base is unsupported"
                );
            }
        }
    }
    for (choice, sample) in input.choices.iter().zip(&change.samples) {
        let offered = items
            .get(choice.id.as_str())
            .context("missing changed input")?;
        let expected = apply(&change.action, offered, &choice.amount, items)?;
        ensure!(
            sample.id == expected.id && sample.amount == expected.amount,
            "changed output example differs from its NBT rule"
        );
        ensure!(
            items.contains_key(sample.id.as_str()),
            "missing changed output item"
        );
    }
    let first = change
        .samples
        .first()
        .context("output change has no samples")?;
    ensure!(
        output.id == first.id && output.amount == first.amount,
        "default output differs from its first input choice"
    );
    Ok(())
}

fn apply(
    action: &Edit,
    input: &Item,
    amount: &str,
    items: &BTreeMap<&str, &Item>,
) -> Result<Stack> {
    let (registry, meta, count, nbt) = match action {
        Edit::Patch { set, limits } => {
            let mut tags = compound(&input.nbt)?.cloned().unwrap_or_default();
            tags.extend(set.clone());
            for (key, limit) in limits {
                let value = number(tags.get(key))?.min(*limit);
                tags.insert(
                    key.clone(),
                    Nbt::Int {
                        value: value.to_string(),
                    },
                );
            }
            (
                &input.registry,
                input.meta,
                amount,
                Some(Nbt::Compound { value: tags }),
            )
        }
        Edit::Merge { base, keys, tools } => {
            let template = items
                .get(base.id.as_str())
                .context("missing merge template")?;
            let source = compound(&input.nbt)?;
            let eligible = !*tools
                || ((input.armor || !input.tools.is_empty())
                    && (template.armor || !template.tools.is_empty()));
            if !eligible || source.is_none_or(BTreeMap::is_empty) {
                return Ok(base.clone());
            }
            let source = source.expect("nonempty source");
            if let Some(dest) = compound(&template.nbt)? {
                let mut dest = dest.clone();
                let filter = keys
                    .as_ref()
                    .map(|keys| keys.iter().map(String::as_str).collect::<BTreeSet<_>>());
                merge(source, &mut dest, filter.as_ref());
                (
                    &template.registry,
                    template.meta,
                    base.amount.as_str(),
                    Some(Nbt::Compound { value: dest }),
                )
            } else {
                ensure!(
                    keys.is_none(),
                    "native filtered inheritance into an untagged base is unsupported"
                );
                (&template.registry, input.meta, amount, input.nbt.clone())
            }
        }
    };
    Ok(Stack {
        id: item_id(registry, meta, nbt.as_ref())?,
        amount: count.to_owned(),
    })
}

/// NBTPrimitive conversions in Minecraft 1.7.10, including Java narrowing and
/// MathHelper.floor_* at the signed-int boundary. Float/double values are IEEE bits.
fn number(tag: Option<&Nbt>) -> Result<i32> {
    Ok(match tag {
        Some(
            Nbt::Byte { value } | Nbt::Short { value } | Nbt::Int { value } | Nbt::Long { value },
        ) => value.parse::<i64>()? as i32,
        Some(Nbt::Float { value }) => {
            let value = f32::from_bits(u32::from_str_radix(value, 16)?);
            let narrowed = value as i32;
            if value < narrowed as f32 {
                narrowed.wrapping_sub(1)
            } else {
                narrowed
            }
        }
        Some(Nbt::Double { value }) => {
            let value = f64::from_bits(u64::from_str_radix(value, 16)?);
            let narrowed = value as i32;
            if value < narrowed as f64 {
                narrowed.wrapping_sub(1)
            } else {
                narrowed
            }
        }
        _ => 0,
    })
}

fn compound(nbt: &Option<Nbt>) -> Result<Option<&BTreeMap<String, Nbt>>> {
    match nbt {
        None => Ok(None),
        Some(Nbt::Compound { value }) => Ok(Some(value)),
        _ => anyhow::bail!("item NBT root must be a compound"),
    }
}

fn merge(
    source: &BTreeMap<String, Nbt>,
    dest: &mut BTreeMap<String, Nbt>,
    keys: Option<&BTreeSet<&str>>,
) {
    for (key, source) in source {
        if keys.is_some_and(|keys| !keys.contains(key.as_str())) {
            continue;
        }
        match dest.get_mut(key) {
            None => {
                dest.insert(key.clone(), source.clone());
            }
            Some(Nbt::Compound { value: target }) => {
                if let Nbt::Compound { value: source } = source {
                    merge(source, target, None);
                }
            }
            Some(Nbt::List {
                element: kind,
                value: target,
            }) => {
                if let Nbt::List { element, value } = source {
                    if kind == element {
                        target.extend(value.iter().cloned());
                    }
                }
            }
            _ => {}
        }
    }
}
