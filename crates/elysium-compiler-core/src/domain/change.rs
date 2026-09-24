use super::{Consumption, Item, Kind, Match, Output, Recipe};
use crate::identity::{integer, item_id, Nbt};
use anyhow::{bail, ensure, Context, Result};
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
    /// Forestry 4.10.17 native member analysis. A previously analyzed stack is copied
    /// unchanged; otherwise serialize the individual into a fresh compound after analyze().
    /// Item, metadata and the entire offered count are retained. Samples are canonical
    /// native serialization fixed points, so their only tag difference is IsAnalyzed.
    Analyze,
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
    /// Native list append (e.g. Thaumic Machina wand augmentations):
    /// Copy the input item, metadata, count and tags. Append `value` to the list
    /// at `path`. If `path` does not exist or is not a list, creates a new list.
    Append { path: String, value: Nbt },
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
        Edit::Analyze => {
            ensure!(
                recipe.inputs.len() == 2 && recipe.outputs.len() == 1,
                "analysis requires one subject, one honey input and one result"
            );
            let mut branch = None;
            for choice in &input.choices {
                let Match::Member { root, analyzed } = &choice.rule else {
                    anyhow::bail!("analysis requires native member matching")
                };
                ensure!(
                    matches!(choice.consume, Consumption::Stack)
                        && choice.amount == "1"
                        && choice.returns.is_empty(),
                    "analysis consumes a whole stack with one-item examples"
                );
                let state = (root.as_str(), *analyzed);
                ensure!(
                    branch.is_none_or(|previous| previous == state),
                    "analysis mixes roots or states"
                );
                branch = Some(state);
            }
            let (_, done) = branch.context("analysis has no input members")?;
            let honey = recipe
                .inputs
                .iter()
                .find(|input| input.kind == Kind::Fluid)
                .context("analysis requires honey")?;
            ensure!(
                honey.choices.len() == 1
                    && honey.choices[0].amount == "100"
                    && honey.choices[0].returns.is_empty()
                    && if done {
                        matches!(honey.choices[0].consume, Consumption::Keep)
                    } else {
                        matches!(honey.choices[0].consume, Consumption::Consume)
                    },
                "analysis must require 100 mB, consumed only for an unanalyzed subject"
            );
            ensure!(
                recipe.duration.as_deref() == Some(if done { "1" } else { "500" })
                    && recipe.energy.as_deref() == Some(if done { "1" } else { "2" })
                    && output.chance.numerator == "1"
                    && output.chance.denominator == "1",
                "invalid native analysis cost or chance"
            );
        }
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
        Edit::Append { path, value } => {
            ensure!(
                !path.is_empty() && path.len() <= 65535,
                "invalid append path"
            );
            value.validate()?;
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
        output.id == first.id && output.amount.as_deref() == Some(first.amount.as_str()),
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
        Edit::Analyze => {
            let mut tags = compound(&input.nbt)?
                .context("analysis sample has no tags")?
                .clone();
            tags.insert("IsAnalyzed".into(), Nbt::Byte { value: "1".into() });
            (
                &input.registry,
                input.meta,
                amount,
                Some(Nbt::Compound { value: tags }),
            )
        }
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
        Edit::Append { path, value } => {
            let mut tags = compound(&input.nbt)?.cloned().unwrap_or_default();
            let element = value.name().to_string();
            match tags.get_mut(path) {
                None => {
                    tags.insert(
                        path.clone(),
                        Nbt::List {
                            element,
                            value: vec![value.clone()],
                        },
                    );
                }
                Some(Nbt::List {
                    element: existing_elem,
                    value: list,
                }) => {
                    if list.is_empty() || existing_elem == "end" {
                        *existing_elem = element;
                        list.push(value.clone());
                    } else {
                        ensure!(
                            existing_elem == &element,
                            "append element type mismatch with existing list: expected {existing_elem}, got {element}"
                        );
                        list.push(value.clone());
                    }
                }
                Some(_) => {
                    bail!("append target path is not a list: {path}");
                }
            }
            (
                &input.registry,
                input.meta,
                amount,
                Some(Nbt::Compound { value: tags }),
            )
        }
    };
    Ok(Stack {
        id: item_id(registry, meta, nbt.as_ref())?,
        amount: count.to_owned(),
    })
}

/// Validate canonical examples, not a substitute genome for an incomplete native stack.
pub(super) fn member(item: &Item, root: &str, analyzed: bool) -> Result<()> {
    let tags = compound(&item.nbt)?.context("member sample has no tags")?;
    let living = match root {
        "rootTrees" => false,
        "rootBees" | "rootButterflies" => true,
        _ => anyhow::bail!("unknown Forestry species root"),
    };
    ensure!(
        matches!(tags.get("IsAnalyzed"), Some(Nbt::Byte { value }) if value == if analyzed { "1" } else { "0" }),
        "member sample has the wrong analysis state"
    );
    for key in tags.keys() {
        ensure!(
            matches!(key.as_str(), "IsAnalyzed" | "Genome" | "Mate")
                || living && matches!(key.as_str(), "Health" | "MaxH")
                || root == "rootBees" && matches!(key.as_str(), "NA" | "GEN"),
            "member sample contains nonserialized data"
        );
    }
    if living {
        for key in ["Health", "MaxH"] {
            ensure!(
                matches!(tags.get(key), Some(Nbt::Int { .. })),
                "living member lacks native health fields"
            );
        }
    }
    if let Some(value) = tags.get("NA") {
        ensure!(
            matches!(value, Nbt::Byte { value } if value == "0"),
            "noncanonical natural flag"
        );
    }
    if let Some(value) = tags.get("GEN") {
        ensure!(
            matches!(value, Nbt::Int { value } if value.parse::<i32>().is_ok_and(|value| value > 0)),
            "noncanonical generation"
        );
    }
    genome(tags.get("Genome").context("member sample has no genome")?)?;
    if let Some(mate) = tags.get("Mate") {
        genome(mate)?;
    }
    Ok(())
}

fn genome(value: &Nbt) -> Result<()> {
    let Nbt::Compound { value } = value else {
        anyhow::bail!("invalid genome compound")
    };
    ensure!(value.len() == 1, "noncanonical genome fields");
    let Some(Nbt::List { element, value }) = value.get("Chromosomes") else {
        anyhow::bail!("genome has no chromosomes")
    };
    ensure!(
        element == "compound" && !value.is_empty() && value.len() <= 128,
        "invalid chromosome list"
    );
    let mut prior = -1;
    for chromosome in value {
        let Nbt::Compound { value } = chromosome else {
            anyhow::bail!("invalid chromosome")
        };
        ensure!(value.len() == 3, "noncanonical chromosome fields");
        let Some(Nbt::Byte { value: slot }) = value.get("Slot") else {
            anyhow::bail!("missing chromosome slot")
        };
        let slot = integer(slot, 0, 127)?;
        ensure!(slot > prior, "chromosome slots must be ordered and unique");
        prior = slot;
        for key in ["UID0", "UID1"] {
            ensure!(
                matches!(value.get(key), Some(Nbt::String { value }) if !value.is_empty()),
                "missing chromosome allele"
            );
        }
    }
    Ok(())
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
