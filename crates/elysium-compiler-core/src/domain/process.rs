use super::{Consumption, Fluid, Kind, Match, OutputRole, Quantity, Recipe};
use crate::identity::integer;
use anyhow::{ensure, Context, Result};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[cfg(test)]
mod tests {
    #[test]
    fn enchanter_preserves_level_gate_and_signed_xp() {
        let value = serde_json::json!({"kind":"enchanter","level":2,"maxLevel":5,"itemsPerLevel":3,"cost":-7});
        let process: super::Process = serde_json::from_value(value.clone()).unwrap();
        assert_eq!(serde_json::to_value(process).unwrap(), value);
    }

    #[test]
    fn map_scaling_requires_an_explicit_world_process() {
        let value = serde_json::json!({"kind":"mapScaling"});
        assert!(serde_json::from_value::<super::Process>(value.clone()).is_ok());
        assert!(serde_json::from_value::<super::super::Edit>(value).is_ok());
        assert!(serde_json::from_value::<super::Process>(
            serde_json::json!({"kind":"mapScaling","finalMapId":37})
        )
        .is_err());
    }

    #[test]
    fn runic_costs_preserve_native_rounding_and_large_repetition() {
        for (charge, expected) in [
            (-8, (1, 0, 1)),
            (-5, (1, 1, 3)),
            (0, (1, 32, 5)),
            (3, (4, 256, 6)),
            (30, (31, i32::MAX, 20)),
            (i32::MAX, (2147483648, i32::MAX, 1073741828)),
        ] {
            assert_eq!(super::runic_costs(charge), expected);
        }
        assert!(serde_json::from_value::<super::Process>(
            serde_json::json!({"kind":"runic","charge":3})
        )
        .is_ok());
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum HarmonyMode {
    Single,
    Parallel,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum HarmonyOutcome {
    Success,
    Failure,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum InscriberMode {
    Inscribe,
    Press,
}

/// Native shared process, not independent probabilities on individual outputs.
/// See docs/harmony.md for state, energy, rounding and correlated yield rules.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(
    tag = "kind",
    rename_all = "lowercase",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
pub enum Process {
    /// EnderIO manual enchanter. The offered material count determines the level;
    /// cost is signed native XP levels, not energy. See docs/enchanter.md.
    Enchanter {
        level: u16,
        #[serde(rename = "maxLevel")]
        max_level: u16,
        #[serde(rename = "itemsPerLevel")]
        items_per_level: u32,
        cost: i32,
    },
    /// AE2 registered recipe, after the name-press guard and earlier registry
    /// rows. top absent has a native empty-slot predicate; see docs/inscriber.md.
    Inscriber {
        mode: InscriberMode,
        top: Option<String>,
        bottom: Option<String>,
        #[serde(rename = "namePress")]
        name_press: Option<String>,
    },
    /// Vanilla map extension. Input eligibility depends on world MapData (scale < 4).
    /// The output is a pre-onCreated sample; the world allocates the final map ID.
    /// Missing data may be initialized server-side. See docs/maps.md.
    #[serde(rename = "mapScaling")]
    MapScaling {},
    /// Dynamic TC runic augmentation. Charge, component multiplicity and magic
    /// costs are an observed sample. Actual costs use the offered IRunicArmor's
    /// native final charge, including nested upgrades; see docs/runic.md.
    Runic { charge: i32 },
    Harmony {
        mode: HarmonyMode,
        hydrogen: String,
        helium: String,
        ticks: String,
        #[serde(rename = "startEu")]
        start_eu: String,
        #[serde(rename = "outputEu")]
        output_eu: String,
        /// Round-trip decimal representation of the native binary64 base chance.
        chance: String,
        #[serde(rename = "rocketTier")]
        rocket_tier: u8,
        #[serde(rename = "compressionTier")]
        compression_tier: u8,
    },
}

impl Process {
    pub fn max_parallel(&self) -> i64 {
        match self {
            Self::Harmony {
                mode: HarmonyMode::Parallel,
                ..
            } => 1_048_576,
            _ => 1,
        }
    }

    pub(super) fn validate_parameters(&self) -> Result<()> {
        if let Self::Enchanter {
            level,
            max_level,
            items_per_level,
            ..
        } = self
        {
            ensure!(
                *level > 0
                    && *level <= *max_level
                    && *max_level <= 32767
                    && *items_per_level > 0
                    && u64::from(*level) * u64::from(*items_per_level) <= 64,
                "invalid enchanter level or input count"
            );
            return Ok(());
        }
        let Self::Harmony {
            hydrogen,
            helium,
            ticks,
            start_eu,
            output_eu,
            chance,
            rocket_tier,
            compression_tier,
            ..
        } = self
        else {
            return Ok(());
        };
        for value in [hydrogen, helium, ticks, start_eu, output_eu] {
            integer(value, 1, i64::MAX)?;
        }
        let rate: f64 = chance.parse().context("invalid harmony chance")?;
        ensure!(
            rate.is_finite() && (0.0..=1.0).contains(&rate),
            "invalid harmony chance"
        );
        ensure!(
            *rocket_tier <= 9 && *compression_tier == (*rocket_tier).min(8),
            "invalid harmony tiers"
        );
        Ok(())
    }
}

pub(super) fn validate(
    recipe: &Recipe,
    fluids: &BTreeMap<&str, &Fluid>,
    items: &BTreeMap<&str, &super::Item>,
    aspects: &[super::Aspect],
) -> Result<()> {
    ensure!(
        !recipe
            .inputs
            .iter()
            .flat_map(|i| &i.choices)
            .any(|c| matches!(c.rule, Match::Ae))
            || matches!(recipe.process, Some(Process::Inscriber { .. })),
        "AE precise inputs require an inscriber process"
    );
    let Some(process) = &recipe.process else {
        ensure!(
            !recipe.inputs.iter().any(|i| i
                .choices
                .iter()
                .any(|c| matches!(c.consume, Consumption::Buffer | Consumption::Pedestals))),
            "buffer consumption requires a native process"
        );
        return Ok(());
    };
    process.validate_parameters()?;
    if let Process::Enchanter {
        level,
        items_per_level,
        ..
    } = process
    {
        return validate_enchanter(recipe, *level, *items_per_level, items);
    }
    if let Process::Inscriber {
        mode,
        top,
        bottom,
        name_press,
    } = process
    {
        return validate_inscriber(
            recipe,
            *mode,
            top.as_deref(),
            bottom.as_deref(),
            name_press.as_deref(),
            items,
        );
    }
    if matches!(process, Process::MapScaling {}) {
        return validate_map(recipe, items);
    }
    if let Process::Runic { charge } = process {
        return validate_runic(recipe, *charge, items, aspects);
    }
    let Process::Harmony {
        mode,
        hydrogen,
        helium,
        rocket_tier,
        ..
    } = process
    else {
        unreachable!()
    };
    ensure!(
        recipe.duration.is_none()
            && recipe.energy.is_none()
            && recipe.grid.is_none()
            && recipe.magic.is_none(),
        "harmony process cannot have fixed timing, energy or crafting semantics"
    );
    ensure!(
        recipe.inputs.len() == if *mode == HarmonyMode::Single { 3 } else { 2 },
        "invalid harmony inputs"
    );
    let trigger = recipe
        .inputs
        .iter()
        .find(|i| i.kind == Kind::Item && i.slot == 0)
        .context("harmony trigger missing")?;
    ensure!(
        !trigger.choices.is_empty(),
        "harmony requires a native block trigger"
    );
    for c in &trigger.choices {
        ensure!(
            c.amount == "1"
                && matches!(c.consume, Consumption::Keep)
                && c.returns.is_empty()
                && matches!(
                    c.rule,
                    Match::Wildcard {
                        meta: true,
                        nbt: true
                    }
                ),
            "invalid harmony trigger semantics"
        );
    }
    let h = integer(hydrogen, 1, i64::MAX)?;
    let he = integer(helium, 1, i64::MAX)?;
    let required = if *mode == HarmonyMode::Single {
        vec![(0, "hydrogen", h), (1, "helium", he)]
    } else {
        vec![(
            2,
            "rawstarmatter",
            ((he as f64 * 1.24e-5 * 8.0).ceil() as i64).max(1),
        )]
    };
    for (slot, registry, amount) in required {
        let input = recipe
            .inputs
            .iter()
            .find(|i| i.kind == Kind::Fluid && i.slot == slot)
            .context("harmony buffer missing")?;
        ensure!(input.choices.len() == 1, "harmony buffer has alternatives");
        let c = &input.choices[0];
        let fluid = fluids
            .get(c.id.as_str())
            .context("harmony buffer fluid missing")?;
        ensure!(
            fluid.registry == registry
                && fluid.nbt.is_none()
                && c.amount == amount.to_string()
                && matches!(c.consume, Consumption::Buffer)
                && c.returns.is_empty()
                && matches!(
                    c.rule,
                    Match::Wildcard {
                        meta: false,
                        nbt: true
                    }
                ),
            "invalid harmony buffer semantics"
        );
    }
    let mut failures = 0;
    let mut successes = 0;
    for output in &recipe.outputs {
        ensure!(
            matches!(output.role, OutputRole::Result),
            "harmony output is not a result"
        );
        let Some(Quantity::Harmony { outcome, nominal }) = &output.quantity else {
            anyhow::bail!("harmony output lacks shared quantity");
        };
        super::quantity_bounds(recipe, output)?;
        match outcome {
            HarmonyOutcome::Success => successes += 1,
            HarmonyOutcome::Failure => {
                failures += 1;
                ensure!(
                    output.kind == Kind::Fluid
                        && nominal == &(14400_i64 * (1_i64 << (rocket_tier + 1))).to_string(),
                    "invalid harmony failure quantity"
                );
                let fluid = fluids
                    .get(output.id.as_str())
                    .context("harmony failure fluid missing")?;
                ensure!(
                    fluid.registry == "molten.spacetime" && fluid.nbt.is_none(),
                    "invalid harmony failure fluid"
                );
            }
        }
    }
    ensure!(
        failures == 1 && successes > 0,
        "harmony requires normal outputs and exactly one failure output"
    );
    Ok(())
}

fn validate_enchanter(
    recipe: &Recipe,
    level: u16,
    items_per_level: u32,
    items: &BTreeMap<&str, &super::Item>,
) -> Result<()> {
    use crate::identity::Nbt;
    ensure!(
        recipe.duration.is_none()
            && recipe.energy.is_none()
            && recipe.grid.is_none()
            && recipe.magic.is_none()
            && recipe.inputs.len() == 2
            && recipe.outputs.len() == 1,
        "invalid enchanter process shape"
    );
    for slot in 0..2 {
        let input = recipe
            .inputs
            .iter()
            .find(|i| i.slot == slot && i.kind == Kind::Item)
            .context("missing enchanter input")?;
        let amount = if slot == 0 {
            1
        } else {
            u32::from(level) * items_per_level
        };
        ensure!(!input.choices.is_empty(), "empty enchanter input");
        for choice in &input.choices {
            ensure!(
                choice.amount == amount.to_string()
                    && matches!(choice.consume, Consumption::Consume)
                    && choice.returns.is_empty(),
                "invalid enchanter material consumption"
            );
            if slot == 0 {
                let book = items
                    .get(choice.id.as_str())
                    .context("missing writable book")?;
                ensure!(
                    book.registry == "minecraft:writable_book"
                        && matches!(
                            choice.rule,
                            Match::Wildcard {
                                meta: true,
                                nbt: true
                            }
                        ),
                    "invalid enchanter book predicate"
                );
            } else {
                let base = match &choice.rule {
                    Match::Except { base, .. } => base.as_ref(),
                    other => other,
                };
                ensure!(
                    matches!(base, Match::Wildcard { nbt: true, .. }),
                    "invalid enchanter material predicate"
                );
                if let Match::Except { exclude, .. } = &choice.rule {
                    ensure!(
                        exclude
                            .iter()
                            .all(|c| matches!(c.rule, Match::Wildcard { nbt: true, .. })),
                        "invalid enchanter priority predicate"
                    );
                }
            }
        }
    }
    let output = &recipe.outputs[0];
    ensure!(
        output.kind == Kind::Item
            && output.slot == 0
            && output.amount.as_deref() == Some("1")
            && output.quantity.is_none()
            && output.change.is_none()
            && matches!(output.role, OutputRole::Result)
            && output.chance.numerator == "1"
            && output.chance.denominator == "1",
        "invalid enchanter result"
    );
    let book = items
        .get(output.id.as_str())
        .context("missing enchanted book")?;
    ensure!(
        book.registry == "minecraft:enchanted_book" && book.meta == 0,
        "invalid enchanted book identity"
    );
    let Some(Nbt::Compound { value: root }) = &book.nbt else {
        anyhow::bail!("missing enchantment NBT")
    };
    let Some(Nbt::List {
        element,
        value: list,
    }) = root.get("StoredEnchantments")
    else {
        anyhow::bail!("missing stored enchantment")
    };
    ensure!(
        root.len() == 1 && element == "compound" && list.len() == 1,
        "invalid stored enchantment list"
    );
    let Nbt::Compound { value: enchantment } = &list[0] else {
        anyhow::bail!("invalid enchantment tag")
    };
    ensure!(
        enchantment.len() == 2
            && matches!(enchantment.get("id"), Some(Nbt::Short { .. }))
            && matches!(enchantment.get("lvl"), Some(Nbt::Short {value}) if value == &level.to_string()),
        "enchanted book level differs from process"
    );
    Ok(())
}

fn validate_inscriber(
    recipe: &Recipe,
    mode: InscriberMode,
    top: Option<&str>,
    bottom: Option<&str>,
    name_press: Option<&str>,
    items: &BTreeMap<&str, &super::Item>,
) -> Result<()> {
    ensure!(
        recipe.duration.is_none()
            && recipe.energy.is_none()
            && recipe.grid.is_none()
            && recipe.magic.is_none(),
        "inscriber cannot have fixed timing/energy or crafting semantics"
    );
    for id in [top, bottom, name_press].into_iter().flatten() {
        let item = items.get(id).context("missing inscriber process item")?;
        Match::Ae.validate_item(item, items, false, true)?;
    }
    if let (Some(top), Some(name)) = (top, name_press) {
        let same = |id: &str| {
            items[id].registry == items[name].registry && items[id].meta == items[name].meta
        };
        ensure!(
            !same(top) || bottom.is_some_and(|id| !same(id)),
            "inscriber row is preempted by name presses"
        );
    }
    ensure!(
        recipe.inputs.len()
            == 1 + usize::from(top.is_some()) + usize::from(top.is_some() && bottom.is_some()),
        "invalid inscriber input count"
    );
    for (slot, anchor) in [(0, top), (1, bottom.filter(|_| top.is_some())), (2, None)] {
        if slot != 2 && anchor.is_none() {
            continue;
        }
        let input = recipe
            .inputs
            .iter()
            .find(|i| i.kind == Kind::Item && i.slot == slot)
            .context("inscriber input missing")?;
        ensure!(
            !input.choices.is_empty() && (slot == 2 || input.choices.len() == 1),
            "invalid inscriber alternatives"
        );
        for c in &input.choices {
            ensure!(
                c.amount == "1"
                    && c.returns.is_empty()
                    && matches!(c.rule, Match::Ae)
                    && if slot == 2 || mode == InscriberMode::Press {
                        matches!(c.consume, Consumption::Consume)
                    } else {
                        matches!(c.consume, Consumption::Keep)
                    },
                "invalid inscriber consumption or matching"
            );
            ensure!(
                anchor.is_none_or(|id| id == c.id),
                "inscriber plate does not match process"
            );
        }
    }
    ensure!(recipe.outputs.len() == 1, "inscriber requires one output");
    let output = &recipe.outputs[0];
    ensure!(
        output.slot == 0
            && output.kind == Kind::Item
            && output.quantity.is_none()
            && output.change.is_none()
            && output.amount.is_some()
            && matches!(output.role, OutputRole::Result)
            && output.chance.numerator == "1"
            && output.chance.denominator == "1",
        "invalid inscriber result"
    );
    Ok(())
}

fn validate_map(recipe: &Recipe, items: &BTreeMap<&str, &super::Item>) -> Result<()> {
    ensure!(
        recipe.duration.is_none()
            && recipe.energy.is_none()
            && recipe.magic.is_none()
            && recipe.inputs.len() == 9
            && recipe.outputs.len() == 1,
        "invalid map scaling process"
    );
    let grid = recipe
        .grid
        .as_ref()
        .context("map scaling requires its native grid")?;
    ensure!(
        grid.width == 3
            && grid.height == 3
            && grid.mirror
            && grid.cells == (0..9).map(Some).collect::<Vec<_>>(),
        "invalid map scaling grid"
    );
    for slot in 0..9 {
        let input = recipe
            .inputs
            .iter()
            .find(|i| i.kind == Kind::Item && i.slot == slot)
            .context("map scaling input missing")?;
        ensure!(
            !input.choices.is_empty(),
            "map scaling input has no samples"
        );
        for c in &input.choices {
            let item = items
                .get(c.id.as_str())
                .context("map scaling sample missing")?;
            ensure!(
                item.registry
                    == if slot == 4 {
                        "minecraft:filled_map"
                    } else {
                        "minecraft:paper"
                    }
                    && (slot == 4 || item.meta == 0)
                    && item.meta != 32767
                    && c.amount == "1"
                    && matches!(c.consume, Consumption::Consume)
                    && c.returns.is_empty()
                    && matches!(c.rule, Match::Wildcard { meta, nbt: true } if meta == (slot == 4)),
                "invalid map scaling ingredient"
            );
        }
    }
    let output = &recipe.outputs[0];
    ensure!(
        output.kind == Kind::Item
            && output.slot == 0
            && output.quantity.is_none()
            && output.amount.as_deref() == Some("1")
            && matches!(output.role, OutputRole::Result)
            && output.chance.numerator == "1"
            && output.chance.denominator == "1"
            && output
                .change
                .as_ref()
                .is_some_and(|c| c.input == 4 && matches!(c.action, super::Edit::MapScaling {})),
        "map scaling requires a pending native result"
    );
    Ok(())
}

fn runic_costs(charge: i32) -> (i64, i32, i32) {
    (
        1 + i64::from(charge.max(0)),
        (32.0 * 2.0_f64.powi(charge)) as i32,
        5 + charge / 2,
    )
}

fn validate_runic(
    recipe: &Recipe,
    charge: i32,
    items: &BTreeMap<&str, &super::Item>,
    aspects: &[super::Aspect],
) -> Result<()> {
    let (pedestals, vis, instability) = runic_costs(charge);
    let magic = recipe
        .magic
        .as_ref()
        .context("runic magic sample missing")?;
    ensure!(
        magic.kind == super::MagicKind::Infusion
            && magic.central == Some(0)
            && magic.instability == Some(instability)
            && magic.payment.is_none()
            && !magic.creative
            && recipe.duration.is_none()
            && recipe.energy.is_none()
            && recipe.grid.is_none(),
        "invalid runic process sample"
    );
    ensure!(
        magic.research.len() == 1 && magic.research[0].key == "RUNICAUGMENTATION",
        "invalid runic research"
    );
    let actual: BTreeMap<_, _> = magic
        .aspects
        .iter()
        .map(|cost| {
            let aspect = aspects
                .iter()
                .find(|aspect| aspect.id == cost.aspect)
                .context("runic aspect missing")?;
            ensure!(
                aspect.source.owner == "Thaumcraft"
                    && aspect.source.handler == "thaumcraft.api.aspects.Aspect",
                "invalid runic aspect origin"
            );
            Ok((aspect.source.key.as_str(), cost.amount.clone()))
        })
        .collect::<Result<_>>()?;
    let expected = if vis > 0 {
        BTreeMap::from([
            ("tutamen", (vis / 2).to_string()),
            ("praecantatio", (vis / 2).to_string()),
            ("potentia", vis.to_string()),
        ])
    } else {
        BTreeMap::new()
    };
    ensure!(
        actual == expected,
        "runic aspect sample differs from native charge formula"
    );
    ensure!(
        recipe.inputs.len() == 3 && recipe.outputs.len() == 1,
        "runic requires central and two component groups"
    );
    for slot in 0..3 {
        let input = recipe
            .inputs
            .iter()
            .find(|input| input.kind == Kind::Item && input.slot == slot)
            .context("runic input missing")?;
        ensure!(
            !input.choices.is_empty() && (slot != 0 || input.choices.len() == 1),
            "runic samples require separate central branches"
        );
        let mut component_rule = None;
        for c in &input.choices {
            ensure!(
                (slot != 0 || c.returns.is_empty())
                    && c.amount
                        == if slot == 2 {
                            pedestals.to_string()
                        } else {
                            "1".into()
                        },
                "invalid runic component count"
            );
            if slot == 0 {
                ensure!(
                    matches!(c.consume, Consumption::Consume)
                        && matches!(
                            c.rule,
                            Match::Wildcard {
                                meta: true,
                                nbt: true
                            }
                        ),
                    "invalid runic central predicate"
                );
            } else {
                let Match::Infusion { template, .. } = &c.rule else {
                    anyhow::bail!("invalid runic component predicate");
                };
                let rule = serde_json::to_string(&c.rule)?;
                ensure!(
                    component_rule
                        .as_ref()
                        .is_none_or(|expected| expected == &rule),
                    "mixed runic component predicates"
                );
                component_rule = Some(rule);
                let item = items
                    .get(template.as_str())
                    .context("runic component missing")?;
                ensure!(
                    item.registry
                        == if slot == 1 {
                            "minecraft:diamond"
                        } else {
                            "Thaumcraft:ItemResource"
                        }
                        && item.meta == if slot == 1 { 0 } else { 14 }
                        && item.nbt.is_none(),
                    "invalid runic component identity"
                );
                ensure!(
                    if slot == 2 {
                        matches!(c.consume, Consumption::Pedestals)
                    } else {
                        matches!(c.consume, Consumption::Consume)
                    },
                    "invalid runic pedestal consumption"
                );
            }
        }
    }
    let output = &recipe.outputs[0];
    ensure!(
        output.kind == Kind::Item
            && output.slot == 0
            && output.quantity.is_none()
            && matches!(output.role, OutputRole::Result)
            && output.chance.numerator == "1"
            && output.chance.denominator == "1"
            && output.change.as_ref().is_some_and(
                |change| change.input == 0 && matches!(change.action, super::Edit::Runic)
            ),
        "invalid runic transformation"
    );
    Ok(())
}
