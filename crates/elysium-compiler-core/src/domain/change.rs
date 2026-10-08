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
    /// Output of the process's ordered integration rule and a correlated input tuple.
    Integration,
    /// Fresh broken spawner: base registry, meta=0, count=1, only mobType string.
    /// The string is native getMobTypeFromStack(input0), not the full entity NBT.
    Soul { base: String },
    /// Fresh floating special flower, meta=0/count=1, with only a string `type`.
    /// Base supplies only the registry from a renderable example; all its tags are discarded.
    /// Missing `type` becomes an empty string; non-string values are outside
    /// the paired process's explicit input domain, never silently coerced.
    #[serde(rename = "floatingFlower")]
    FloatingFlower { base: String },
    /// Copy the offered map and tags, set count=1 and map_is_scaling byte=1.
    /// This is the result before ItemMap.onCreated allocates the final world ID.
    #[serde(rename = "mapScaling")]
    MapScaling {},
    /// Native Runic augmentation: copy the input and increment RS.HARDEN using
    /// Minecraft getByte coercion followed by signed-byte wrapping.
    Runic,
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
    /// Automagy PreserveFilterRecipe. Preserve the base item/count/tags; replace
    /// ContainedItems and FilterOptions using native inventory serialization.
    /// A later filter supplies metadata; otherwise the base metadata is retained.
    Filter {
        base: Stack,
        config: u32,
        metadata: Option<u32>,
        /// One native InventoryWithFilterOptions.writeCustomNBT result per config
        /// choice, in the same order. Numeric item lookup happens in the pinned game
        /// registry; these observations are not a promise to replay arbitrary NBT.
        configurations: Vec<BTreeMap<String, Nbt>>,
    },
    /// Native list append (e.g. Thaumic Machina wand augmentations):
    /// Copy the input item, metadata, count and tags. Append `value` to the list
    /// at `path`. Missing paths and Minecraft's empty end-list are created; an
    /// existing scalar is rejected because it cannot preserve native data.
    Append { path: String, value: Nbt },
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Change {
    pub input: u32,
    pub action: Edit,
    /// One native result per input choice, in matching order; excluded from recipe identity.
    pub samples: Vec<Stack>,
    /// One choice index (or an empty optional slot) per recipe input, per sample.
    /// Only integration uses tuples. Excluded from recipe identity with samples.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bindings: Option<Vec<Vec<Option<u32>>>>,
}

pub(super) fn validate(
    recipe: &Recipe,
    output: &Output,
    items: &BTreeMap<&str, &Item>,
) -> Result<()> {
    let Some(change) = &output.change else {
        return Ok(());
    };
    if matches!(change.action, Edit::Integration) {
        return super::integration::validate_change(recipe, output, items);
    }
    ensure!(
        change.bindings.is_none(),
        "correlated bindings require an integration output"
    );
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
        Edit::Integration => unreachable!("integration is validated as a tuple"),
        Edit::FloatingFlower { base } => {
            let Some(super::Process::FloatingFlowers { special }) = &recipe.process else {
                bail!("floating flower change requires its ordered process");
            };
            super::floating::validate(recipe, special, items)?;
            let template = items
                .get(base.as_str())
                .context("missing floating flower base")?;
            ensure!(
                template.meta == 0,
                "floating flower base must have metadata zero"
            );
        }
        Edit::Soul { base } => {
            ensure!(
                matches!(
                    recipe.process,
                    Some(super::Process::Soul { spawner: true, .. })
                ) && change.input == 0
                    && output.slot == 1,
                "soul reconstruction requires spawner output"
            );
            let item = items.get(base.as_str()).context("missing spawner base")?;
            ensure!(
                item.meta == 0 && item.nbt.is_none(),
                "spawner base must be fresh"
            );
        }
        Edit::MapScaling {} => ensure!(
            matches!(recipe.process, Some(super::Process::MapScaling {})),
            "pending map output requires its world process"
        ),
        Edit::Runic => ensure!(
            matches!(recipe.process, Some(super::Process::Runic { .. })),
            "runic output requires its dynamic process"
        ),
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
        Edit::Filter {
            base,
            config,
            metadata,
            configurations,
        } => {
            let base_item = items
                .get(base.id.as_str())
                .context("missing filter base item")?;
            integer(&base.amount, 1, i64::MAX)?;
            ensure!(
                metadata.is_none_or(|metadata| metadata > *config),
                "filter metadata input must follow configuration"
            );
            ensure!(
                change.input == metadata.unwrap_or(*config),
                "filter samples must bind to metadata, or configuration when metadata is absent"
            );
            let config_input = recipe
                .inputs
                .iter()
                .find(|input| input.kind == Kind::Item && input.slot == *config)
                .context("filter configuration input is missing")?;
            if let Some(metadata) = metadata {
                let metadata_input = recipe
                    .inputs
                    .iter()
                    .find(|input| input.kind == Kind::Item && input.slot == *metadata)
                    .context("filter metadata input is missing")?;
                ensure!(
                    !metadata_input.choices.is_empty(),
                    "filter metadata input has no choices"
                );
            }
            ensure!(
                !config_input.choices.is_empty()
                    && configurations.len() == config_input.choices.len(),
                "filter configuration observations omit input choices"
            );
            ensure!(base_item.id == base.id, "invalid filter base item");
            let mut expected = None;
            for (choice, observation) in config_input.choices.iter().zip(configurations) {
                let config_item = items
                    .get(choice.id.as_str())
                    .context("missing filter config item")?;
                filter_configuration(config_item, observation)?;
                let tags = filter_tags(base_item, observation)?;
                ensure!(
                    metadata.is_none() || expected.as_ref().is_none_or(|value| value == &tags),
                    "filter configuration alternatives require separate output branches"
                );
                expected = Some(tags);
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
        let expected = apply(recipe, &change.action, offered, &choice.amount, items)?;
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
    recipe: &Recipe,
    action: &Edit,
    input: &Item,
    amount: &str,
    items: &BTreeMap<&str, &Item>,
) -> Result<Stack> {
    let (registry, meta, count, nbt) = match action {
        Edit::Integration => bail!("integration cannot be evaluated from a single input"),
        Edit::FloatingFlower { base } => {
            let template = items
                .get(base.as_str())
                .context("missing floating flower base")?;
            let value = match compound(&input.nbt)?.and_then(|tags| tags.get("type")) {
                None => String::new(),
                Some(Nbt::String { value }) => value.clone(),
                Some(_) => bail!("non-string flower type is outside the declared input domain"),
            };
            (
                &template.registry,
                0,
                "1",
                Some(Nbt::Compound {
                    value: BTreeMap::from([("type".into(), Nbt::String { value })]),
                }),
            )
        }
        Edit::Soul { base } => {
            let template = items.get(base.as_str()).context("missing spawner base")?;
            let Some(Match::Soul { filter }) = recipe
                .inputs
                .first()
                .and_then(|i| i.choices.first())
                .map(|c| &c.rule)
            else {
                anyhow::bail!("missing soul predicate")
            };
            let name = filter
                .sample_name(input, items)?
                .context("missing spawner soul")?;
            (
                &template.registry,
                0,
                "1",
                Some(Nbt::Compound {
                    value: BTreeMap::from([(
                        "mobType".into(),
                        Nbt::String {
                            value: name.to_owned(),
                        },
                    )]),
                }),
            )
        }
        Edit::MapScaling {} => {
            ensure!(
                input.registry == "minecraft:filled_map",
                "map scaling requires a filled map"
            );
            let mut tags = compound(&input.nbt)?.cloned().unwrap_or_default();
            tags.insert("map_is_scaling".into(), Nbt::Byte { value: "1".into() });
            (
                &input.registry,
                input.meta,
                "1",
                Some(Nbt::Compound { value: tags }),
            )
        }
        Edit::Runic => {
            let mut tags = compound(&input.nbt)?.cloned().unwrap_or_default();
            let hardened = (number(tags.get("RS.HARDEN"))? as i8).wrapping_add(1);
            tags.insert(
                "RS.HARDEN".into(),
                Nbt::Byte {
                    value: hardened.to_string(),
                },
            );
            (
                &input.registry,
                input.meta,
                amount,
                Some(Nbt::Compound { value: tags }),
            )
        }
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
        Edit::Filter {
            base,
            config,
            metadata,
            configurations,
        } => {
            let template = items
                .get(base.id.as_str())
                .context("missing filter template")?;
            let config_input = recipe
                .inputs
                .iter()
                .find(|input| input.kind == Kind::Item && input.slot == *config)
                .context("missing filter config input")?;
            let index = if metadata.is_some() {
                0
            } else {
                config_input
                    .choices
                    .iter()
                    .position(|choice| choice.id == input.id)
                    .context("filter sample is not a configuration choice")?
            };
            let configuration = configurations
                .get(index)
                .context("missing native filter configuration")?;
            let tags = filter_tags(template, configuration)?;
            (
                &template.registry,
                if metadata.is_some() {
                    input.meta
                } else {
                    template.meta
                },
                base.amount.as_str(),
                Some(Nbt::Compound { value: tags }),
            )
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
pub(super) fn number(tag: Option<&Nbt>) -> Result<i32> {
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

fn filter_tags(
    template: &Item,
    configuration: &BTreeMap<String, Nbt>,
) -> Result<BTreeMap<String, Nbt>> {
    let mut tags = compound(&template.nbt)?.cloned().unwrap_or_default();
    tags.remove("FilterOptions");
    tags.extend(configuration.clone());
    Ok(tags)
}

/// Inventory item ids are runtime observations. Check their canonical structure,
/// and independently recompute the options that do not require the game registry.
fn filter_configuration(config: &Item, observed: &BTreeMap<String, Nbt>) -> Result<()> {
    Nbt::Compound {
        value: observed.clone(),
    }
    .validate()?;
    ensure!(
        observed
            .keys()
            .all(|key| matches!(key.as_str(), "ContainedItems" | "FilterOptions")),
        "filter serialization includes unrelated root tags"
    );
    let Some(Nbt::List { element, value }) = observed.get("ContainedItems") else {
        bail!("filter serialization has no inventory list")
    };
    ensure!(
        value.len() <= 9
            && if value.is_empty() {
                element == "end"
            } else {
                element == "compound"
            },
        "invalid native filter inventory list"
    );
    let mut previous = -1;
    for entry in value {
        let Nbt::Compound { value: entry } = entry else {
            bail!("invalid filter inventory entry")
        };
        ensure!(
            entry
                .keys()
                .all(|key| matches!(key.as_str(), "Slot" | "id" | "Count" | "Damage" | "tag")),
            "nonserialized filter inventory field"
        );
        let Some(Nbt::Byte { value: slot }) = entry.get("Slot") else {
            bail!("invalid filter inventory slot")
        };
        let slot = integer(slot, 0, 8)?;
        ensure!(
            slot > previous,
            "filter inventory slots must be sorted and unique"
        );
        previous = slot;
        ensure!(
            matches!(entry.get("id"), Some(Nbt::Short { .. }))
                && matches!(entry.get("Damage"), Some(Nbt::Short { .. }))
                && matches!(entry.get("Count"), Some(Nbt::Int { .. }))
                && entry
                    .get("tag")
                    .is_none_or(|tag| matches!(tag, Nbt::Compound { .. })),
            "invalid native filter stack serialization"
        );
    }
    let mut options = BTreeMap::new();
    if let Some(Nbt::Compound { value: source }) =
        compound(&config.nbt)?.and_then(|tags| tags.get("FilterOptions"))
    {
        for key in ["useItemCount", "ignoreNBT", "ignoreMetadata"] {
            if number(source.get(key))? as i8 != 0 {
                options.insert(key.into(), Nbt::Byte { value: "1".into() });
            }
        }
        if let Some(Nbt::String { value }) = source.get("nameFilter") {
            let value = value.trim_matches(|c| c <= '\u{20}');
            if !value.is_empty() {
                options.insert(
                    "nameFilter".into(),
                    Nbt::String {
                        value: value.into(),
                    },
                );
            }
        }
    }
    let expected = (!options.is_empty()).then_some(Nbt::Compound { value: options });
    ensure!(
        observed.get("FilterOptions") == expected.as_ref(),
        "filter options differ from native normalization"
    );
    Ok(())
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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn runic_byte_coercion_preserves_native_wrap_and_floor() {
        for (nbt, expected) in [
            (json!({"type":"int","value":"383"}), -128_i8),
            (json!({"type":"double","value":"bff8000000000000"}), -1),
            (json!({"type":"long","value":"9223372036854775807"}), 0),
            (json!({"type":"string","value":"127"}), 1),
        ] {
            let tag: Nbt = serde_json::from_value(nbt).unwrap();
            assert_eq!(
                (number(Some(&tag)).unwrap() as i8).wrapping_add(1),
                expected
            );
        }
    }

    fn empty_filter() -> BTreeMap<String, Nbt> {
        BTreeMap::from([(
            "ContainedItems".into(),
            Nbt::List {
                element: "end".into(),
                value: vec![],
            },
        )])
    }

    fn item(registry: &str, meta: i32) -> Item {
        serde_json::from_value(json!({
            "id": item_id(registry, meta, None).unwrap(), "registry": registry,
            "meta": meta, "name": "test", "tooltip": [], "stackLimit": 64,
            "durability": 0, "tools": {}, "armor": false, "tags": []
        }))
        .unwrap()
    }

    #[test]
    fn pending_map_retains_data_but_forces_one_item() {
        let recipe: Recipe = serde_json::from_value(json!({
            "id":"test", "source":{"owner":"minecraft","handler":"test","key":"test"},
            "category":"test", "inputs":[], "outputs":[], "properties":{}, "order":0,
            "process":{"kind":"mapScaling"}
        }))
        .unwrap();
        for source in [
            None,
            Some(Nbt::Compound {
                value: BTreeMap::from([
                    (
                        "owner".into(),
                        Nbt::String {
                            value: "retained".into(),
                        },
                    ),
                    ("map_is_scaling".into(), Nbt::Int { value: "0".into() }),
                ]),
            }),
        ] {
            let mut input = item("minecraft:filled_map", 37);
            input.nbt = source.clone();
            let result = apply(
                &recipe,
                &Edit::MapScaling {},
                &input,
                "17",
                &BTreeMap::new(),
            )
            .unwrap();
            let mut tags = compound(&source).unwrap().cloned().unwrap_or_default();
            tags.insert("map_is_scaling".into(), Nbt::Byte { value: "1".into() });
            assert_eq!(
                result.id,
                item_id(
                    "minecraft:filled_map",
                    37,
                    Some(&Nbt::Compound { value: tags })
                )
                .unwrap()
            );
            assert_eq!(result.amount, "1");
            assert_eq!(input.nbt, source);
        }
    }

    #[test]
    fn filter_matches_native_automagy_fixtures() {
        let fixtures: serde_json::Value =
            serde_json::from_str(include_str!("filter-native-fixtures.json")).unwrap();
        for fixture in fixtures.as_array().unwrap() {
            let mut base = item("minecraft:paper", 7);
            base.nbt = serde_json::from_value(fixture["base"].clone()).unwrap();
            base.id = item_id(&base.registry, base.meta, base.nbt.as_ref()).unwrap();
            let mut config = item("Automagy:enchantedPaper", 1);
            config.nbt = serde_json::from_value(fixture["config"].clone()).unwrap();
            config.id = item_id(&config.registry, config.meta, config.nbt.as_ref()).unwrap();
            let metadata = item("Automagy:enchantedPaper", 2);
            let has_metadata = fixture["hasMetadata"].as_bool().unwrap();
            let mut product = item("minecraft:paper", fixture["meta"].as_i64().unwrap() as i32);
            product.nbt = serde_json::from_value(fixture["expected"].clone()).unwrap();
            product.id = item_id(&product.registry, product.meta, product.nbt.as_ref()).unwrap();
            let recipe: Recipe = serde_json::from_value(json!({
                "id":"recipe_test", "source":{"owner":"Automagy","handler":"native-test","key":"native-test"},
                "category":"category_test", "inputs":[
                    {"slot":0,"kind":"item","choices":[{"id":config.id,"amount":"1","consume":{"kind":"consume"},"returns":[],"rule":{"kind":"exact"}}]},
                    {"slot":1,"kind":"item","choices":[{"id":metadata.id,"amount":"1","consume":{"kind":"consume"},"returns":[],"rule":{"kind":"exact"}}]}
                ], "outputs":[{"slot":0,"kind":"item","id":product.id,"amount":"3","role":"result",
                    "chance":{"numerator":"1","denominator":"1"}, "change":{
                        "input": if has_metadata {1} else {0}, "action":{"kind":"filter",
                            "base":{"id":base.id,"amount":"3"}, "config":0, "metadata":if has_metadata {Some(1)} else {None},
                            "configurations":[fixture["normalized"]["value"].clone()]},
                        "samples":[{"id":product.id,"amount":"3"}]}}], "properties":{},"order":0
            })).expect("native filter contract accepts optional metadata and normalized configuration facts");
            let items = BTreeMap::from([
                (base.id.as_str(), &base),
                (config.id.as_str(), &config),
                (metadata.id.as_str(), &metadata),
                (product.id.as_str(), &product),
            ]);
            validate(&recipe, &recipe.outputs[0], &items).unwrap();
            let mut corrupt = recipe.clone();
            if let Some(change) = corrupt.outputs[0].change.as_mut() {
                let mut action = serde_json::to_value(&change.action).unwrap();
                action["configurations"][0]["owner"] = json!({"type":"string","value":"injected"});
                change.action = serde_json::from_value(action).unwrap();
            }
            assert!(
                validate(&corrupt, &corrupt.outputs[0], &items).is_err(),
                "arbitrary config root tags must not be copied"
            );
        }
    }

    #[test]
    fn filter_keeps_recipe_product_and_count() {
        let base = item("automagy:product", 0);
        let config = item("automagy:paper", 1);
        let metadata = item("automagy:paper", 2);
        let recipe: Recipe = serde_json::from_value(json!({
            "id": "recipe_test", "source": {"owner": "automagy", "handler": "test", "key": "test"},
            "category": "category_test", "inputs": [{
                "slot": 0, "kind": "item", "choices": [{"id": config.id, "amount": "1",
                "consume": {"kind": "consume"}, "returns": [], "rule": {"kind": "exact"}}]
            }], "outputs": [], "properties": {}, "order": 0
        }))
        .unwrap();
        let items = BTreeMap::from([(base.id.as_str(), &base), (config.id.as_str(), &config)]);
        let action = Edit::Filter {
            base: Stack {
                id: base.id.clone(),
                amount: "3".into(),
            },
            config: 0,
            metadata: Some(1),
            configurations: vec![empty_filter()],
        };
        let result = apply(&recipe, &action, &metadata, "1", &items).unwrap();
        assert_eq!(
            result.amount, "3",
            "filter must retain the recipe output count"
        );
        assert_eq!(
            result.id,
            item_id(
                "automagy:product",
                2,
                Some(&Nbt::Compound {
                    value: empty_filter()
                })
            )
            .unwrap(),
            "only metadata comes from the later filter; its item type is not the product"
        );
    }

    #[test]
    fn filter_rejects_samples_bound_to_configuration_instead_of_metadata() {
        let base = item("automagy:product", 0);
        let config = item("automagy:paper", 1);
        let metadata = item("automagy:paper", 2);
        let mut product = item("automagy:product", 1);
        product.nbt = Some(Nbt::Compound {
            value: empty_filter(),
        });
        product.id = item_id(&product.registry, product.meta, product.nbt.as_ref()).unwrap();
        let recipe: Recipe = serde_json::from_value(json!({
            "id": "recipe_test", "source": {"owner": "automagy", "handler": "test", "key": "test"},
            "category": "category_test", "inputs": [
                {"slot": 0, "kind": "item", "choices": [{"id": config.id, "amount": "1",
                    "consume": {"kind": "consume"}, "returns": [], "rule": {"kind": "exact"}}]},
                {"slot": 1, "kind": "item", "choices": [{"id": metadata.id, "amount": "1",
                    "consume": {"kind": "consume"}, "returns": [], "rule": {"kind": "exact"}}]}
            ], "outputs": [{"slot": 0, "kind": "item", "id": product.id, "amount": "3", "role": "result",
                "chance": {"numerator": "1", "denominator": "1"},
                "change": {"input": 0, "action": {"kind": "filter",
                    "base": {"id": base.id, "amount": "3"}, "config": 0, "metadata": 1, "configurations": [empty_filter()]},
                    "samples": [{"id": product.id, "amount": "3"}]}
            }], "properties": {}, "order": 0
        })).unwrap();
        let items = BTreeMap::from([
            (base.id.as_str(), &base),
            (config.id.as_str(), &config),
            (metadata.id.as_str(), &metadata),
            (product.id.as_str(), &product),
        ]);
        assert!(
            validate(&recipe, &recipe.outputs[0], &items).is_err(),
            "matching a fabricated sample must not bypass the declared metadata input"
        );
        let mut correct_product = product.clone();
        correct_product.meta = 2;
        correct_product.id =
            item_id(&correct_product.registry, 2, correct_product.nbt.as_ref()).unwrap();
        let mut items = items;
        items.insert(correct_product.id.as_str(), &correct_product);
        let mut valid = recipe.clone();
        valid.outputs[0].id = correct_product.id.clone();
        let change = valid.outputs[0].change.as_mut().unwrap();
        change.input = 1;
        change.samples[0].id = correct_product.id.clone();
        validate(&valid, &valid.outputs[0], &items).unwrap();

        let mut other_config = config.clone();
        other_config.nbt = Some(Nbt::Compound {
            value: BTreeMap::from([(
                "FilterOptions".into(),
                Nbt::Compound {
                    value: BTreeMap::from([("ignoreNBT".into(), Nbt::Byte { value: "1".into() })]),
                },
            )]),
        });
        other_config.id = item_id(
            &other_config.registry,
            other_config.meta,
            other_config.nbt.as_ref(),
        )
        .unwrap();
        items.insert(other_config.id.as_str(), &other_config);
        let mut alternative = valid.inputs[0].choices[0].clone();
        alternative.id = other_config.id.clone();
        valid.inputs[0].choices.push(alternative);
        if let Edit::Filter { configurations, .. } =
            &mut valid.outputs[0].change.as_mut().unwrap().action
        {
            let mut normalized = empty_filter();
            normalized.insert(
                "FilterOptions".into(),
                Nbt::Compound {
                    value: BTreeMap::from([("ignoreNBT".into(), Nbt::Byte { value: "1".into() })]),
                },
            );
            configurations.push(normalized);
        }
        assert!(validate(&valid, &valid.outputs[0], &items).is_err(),
            "a later configuration candidate with a different native result requires its own branch");
    }
}
