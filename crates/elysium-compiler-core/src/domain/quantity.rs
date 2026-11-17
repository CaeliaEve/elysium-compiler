use super::{Consumption, Kind, Output, Recipe};
use crate::identity::integer;
use anyhow::{ensure, Context, Result};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

/// Correlated amounts, evaluated without arbitrary sampling.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "lowercase", deny_unknown_fields)]
pub enum Quantity {
    /// Native in-world yield depends on power, settings and output acceptance.
    #[serde(rename="spaceAcquisition")]
    SpaceAcquisition {nominal:String},
    /// No per-opening fixed amount: shared draws, retry state and split-stack budget.
    #[serde(rename="lootBag")]
    LootBag {},
    /// Prototype count scaled by the complete altar input stack at completion.
    #[serde(rename = "bloodAltar")]
    BloodAltar { nominal: String },
    /// Stack size for one selection in a shared HarvestCraft draw process.
    Harvest { nominal: String },
    /// Native signed output parameter; the shared Squeezer process determines actual completion.
    Squeezer {
        nominal: String,
    },
    /// Both fixed Soul Binder outputs share the last-vessel completion gate.
    Soul {
        nominal: String,
    },
    /// SAG shared task draw and correlated completion passes. See docs/grinding.md.
    Grinding {
        nominal: String,
        threshold: String,
    },
    /// One shared Java nextFloat draw, output when threshold >= draw. See docs/alloy.md.
    #[serde(rename = "sharedRoll")]
    SharedRoll {
        nominal: String,
        threshold: String,
    },
    /// All outputs share the recipe's native process outcome and yield.
    Harmony {
        outcome: super::HarmonyOutcome,
        nominal: String,
    },
    Draw {
        input: u32,
        after: Vec<u32>,
        limit: String,
    },
    Remainder {
        input: u32,
        after: Vec<u32>,
    },
    Branch {
        group: String,
        branch: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        condition: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        threshold: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        parameters: Option<std::collections::BTreeMap<String, String>>,
        nominal: String,
    },
    Potential {
        stat: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        condition: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        sample: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        parameters: Option<std::collections::BTreeMap<String, String>>,
        nominal: String,
    },
}

/// Conservative display bounds; the rules retain correlation and attainable values.
pub fn quantity_bounds(recipe: &Recipe, output: &Output) -> Result<(i64, i64)> {
    let Some(rule) = &output.quantity else {
        if matches!(recipe.process,Some(super::Process::NativeBotaniaPool(_))){
            return Ok((0,i64::from(i32::MAX)));
        }
        if matches!(recipe.process,Some(super::Process::WirelessTerminalUpgrade(_))) {
            ensure!(output.kind==Kind::Item&&output.role==super::OutputRole::Result&&output.change.is_some(),
                "wireless output must retain its offered-stack copy");
            return Ok((1,i64::from(i32::MAX)));
        }
        if matches!((&recipe.process, &output.change),
            (Some(super::Process::Chisel(_)),Some(change)) if matches!(change.action,super::Edit::ChiselTool{..})) {
            ensure!(output.kind==Kind::Item && matches!(output.role,super::OutputRole::Return),
                "Chisel change must retain its conditional tool return");
            // A broken tool disappears. An offered stack can also contain more
            // tools than the preview; do not advertise a guaranteed fixed return.
            return Ok((0,i64::from(i32::MAX)));
        }
        let value = integer(
            output
                .amount
                .as_deref()
                .context("fixed output lacks amount")?,
            1,
            i64::MAX,
        )?;
        return Ok((value, value));
    };
    ensure!(
        output.amount.is_none() && output.change.is_none(),
        "computed output has a fixed amount or item change"
    );
    ensure!(
        output.chance.numerator == "1" && output.chance.denominator == "1",
        "computed output cannot carry an independent probability"
    );
    match rule {
        Quantity::SpaceAcquisition{nominal}=>{
            let Some(super::Process::SpaceAcquisition(p))=&recipe.process else{anyhow::bail!("Space yield lacks acquisition laws");};
            let n=integer(nominal,1,i64::from(i32::MAX))?;
            ensure!(output.kind==Kind::Fluid&&nominal==&p.base_amount,"Space nominal differs from its native registry");
            Ok((0,if p.operation==super::space_acquisition::Operation::Pump {n*64*128*4}else{i64::from(i32::MAX)}))
        }
        Quantity::LootBag {} => {
            ensure!(output.kind==Kind::Item&&matches!(recipe.process,Some(super::Process::LootBagOpening(_))),
                "Grouped loot quantity requires native opening laws");
            // Native q is a signed int stack budget, not units. The last draw
            // may exceed q with grouped/split outputs; retry pending state persists.
            Ok((0,i64::MAX))
        }
        Quantity::BloodAltar { nominal } => {
            ensure!(matches!(&recipe.process, Some(super::Process::BloodAltar(p)) if !p.can_be_filled)
                && output.kind == Kind::Item, "Altar quantity requires stack conversion");
            integer(nominal, 1, i32::MAX as i64)?;
            // Signed-int multiplication can wrap or eliminate a result.
            // Nominal is a per-item preview, not a guaranteed whole-stack yield.
            Ok((0, i32::MAX as i64))
        }
        Quantity::Harvest { nominal } => {
            let Some(super::Process::HarvestDrops(drops)) = &recipe.process else {
                anyhow::bail!("Harvest quantity lacks its shared draw process");
            };
            ensure!(output.kind == Kind::Item, "Harvest quantity requires an item");
            drops.bounds(output.slot, nominal)
        }
        Quantity::Squeezer { nominal } => {
            ensure!(
                matches!(
                    recipe.process,
                    Some(super::Process::ForestrySqueezer { .. })
                ),
                "Squeezer quantity lacks its shared process"
            );
            let amount = integer(nominal, i64::from(i32::MIN), i64::from(i32::MAX))?;
            if output.kind == Kind::Item
                && matches!(&recipe.process, Some(super::Process::ForestrySqueezer { chance, .. }) if chance.numerator == "0")
            {
                return Ok((0, 0));
            }
            Ok((
                if output.kind == Kind::Item {
                    amount.min(0)
                } else {
                    0
                },
                amount.max(0),
            ))
        }
        Quantity::Soul { nominal } => {
            ensure!(
                matches!(
                    recipe.process,
                    Some(super::Process::Soul { spawner: false, .. })
                ) && output.kind == Kind::Item,
                "soul quantity requires fixed Soul Binder outputs"
            );
            Ok((0, integer(nominal, 1, i64::from(i32::MAX))?))
        }
        Quantity::Grinding { nominal, threshold } => {
            let Some(super::Process::Sag { bonus, .. }) = recipe.process else {
                anyhow::bail!("grinding quantity requires SAG process");
            };
            ensure!(output.kind == Kind::Item, "SAG requires item results");
            let cutoff: f32 = threshold.parse().context("invalid grinding threshold")?;
            ensure!(
                cutoff.is_finite() && (0.0..=1.0).contains(&cutoff),
                "invalid grinding threshold"
            );
            let nominal = integer(nominal, 1, i64::from(i32::MAX))?;
            // A terminating binary32 decrement-by-one loop has fewer than 2^25 passes.
            // This includes saved ball states unrelated to the current registration table.
            // UI presents the nominal per pass, not this deliberately loose safety ceiling.
            Ok((0, nominal * if bonus { 33_554_432 } else { 1 }))
        }
        Quantity::SharedRoll { nominal, threshold } => {
            ensure!(
                matches!(
                    recipe.process,
                    Some(super::Process::Alloy { .. } | super::Process::Splice { .. })
                ) && output.kind == Kind::Item,
                "shared roll requires an assembly item result"
            );
            let cutoff: f32 = threshold.parse().context("invalid shared roll threshold")?;
            ensure!(
                cutoff.is_finite() && (0.0..=1.0).contains(&cutoff),
                "invalid native float cutoff"
            );
            // Capacity and destination state may further reduce completed output.
            Ok((0, integer(nominal, 1, i64::from(i32::MAX))?))
        }
        Quantity::Harmony { nominal, .. } => {
            let process = recipe
                .process
                .as_ref()
                .context("harmony quantity lacks its shared process")?;
            process.validate_parameters()?;
            ensure!(
                matches!(process, super::Process::Harmony { .. }),
                "harmony quantity requires harmony process"
            );
            let base = integer(nominal, 1, i64::MAX)?;
            // Rust's saturating float cast matches Java d2l for nonnegative finite values.
            Ok((0, (base as f64 * process.max_parallel() as f64) as i64))
        }
        Quantity::Branch {
            group,
            parameters,
            nominal,
            ..
        } => {
            if let Some(parameters) = parameters {
                for key in parameters.keys() {
                    ensure!(
                        key.len() <= 64
                            && key.chars().next().is_some_and(|c| c.is_ascii_alphabetic())
                            && key
                                .chars()
                                .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-'),
                        "invalid quantity parameter key"
                    );
                }
            }
            let max_val = integer(nominal, 1, i64::MAX)?;
            let mut seen_branches = BTreeSet::new();
            for candidate in &recipe.outputs {
                if let Some(Quantity::Branch {
                    group: g,
                    branch: b,
                    ..
                }) = &candidate.quantity
                {
                    if g == group {
                        ensure!(
                            seen_branches.insert(b),
                            "duplicate branch in quantity group"
                        );
                    }
                }
            }
            Ok((0, max_val))
        }
        Quantity::Potential {
            stat,
            nominal,
            sample,
            parameters,
            ..
        } => {
            if let Some(parameters) = parameters {
                for key in parameters.keys() {
                    ensure!(
                        key.len() <= 64
                            && key.chars().next().is_some_and(|c| c.is_ascii_alphabetic())
                            && key
                                .chars()
                                .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-'),
                        "invalid quantity parameter key"
                    );
                }
            }
            // Forestry can legally produce no fruit for a valid tree genome.
            // Zero is a semantic bound here, unlike fixed recipe quantities.
            let max_val = integer(nominal, 0, i64::MAX)?;
            if let Some(s) = sample {
                let sample_val = integer(s, 0, i64::MAX)?;
                ensure!(
                    sample_val <= max_val,
                    "potential sample {} exceeds nominal upper bound {}",
                    sample_val,
                    max_val
                );
            }
            if stat == "binnie:geneticTransform" {
                let Some(super::Process::BinnieGenetics(process)) = &recipe.process else {
                    anyhow::bail!("genetic quantity requires its native Binnie process");
                };
                // The captured specimen is a preview, not the maximum yield of
                // all admitted genomes. A bee drone yields 10; a mated bee 50.
                let upper = if output.kind == Kind::Fluid {
                    let super::binnie_genetics::GeneticsProcess::Genepool(pool) = process else {
                        anyhow::bail!("dynamic genetic fluid requires the genepool");
                    };
                    if pool.domain.root_class == "forestry.apiculture.genetics.BeeHelper" { 50 } else { 10 }
                } else { 1 };
                ensure!(max_val <= upper, "genetic specimen exceeds native yield");
                return Ok((0, upper));
            }
            if stat == "chisel:selection" {
                let Some(super::Process::Chisel(process))=&recipe.process else {
                    anyhow::bail!("Chisel quantity requires its native process");
                };
                let preview=if process.mode==super::chisel::ChiselMode::Stack {1}else{process.target_count};
                ensure!(output.kind==Kind::Item && max_val==i64::from(preview),
                    "Chisel preview differs from the native selection");
                return Ok((0,if process.mode==super::chisel::ChiselMode::Stack {i64::from(i32::MAX)}else{max_val}));
            }
            Ok((0, max_val))
        }
        Quantity::Draw { .. } | Quantity::Remainder { .. } => {
            ensure!(
                output.kind == Kind::Fluid,
                "computed quantities require fluid outputs"
            );
            let (input, after) = match rule {
                Quantity::Draw { input, after, .. } | Quantity::Remainder { input, after } => {
                    (*input, after)
                }
                _ => unreachable!(),
            };
            ensure!(
                after.len() <= 128,
                "quantity draw chain exceeds 128 entries"
            );
            let mut draws = BTreeSet::new();
            let mut remainder = None;
            for candidate in &recipe.outputs {
                match &candidate.quantity {
                    Some(Quantity::Draw { input: group, .. }) if *group == input => {
                        ensure!(
                            candidate.kind == Kind::Fluid && draws.insert(candidate.slot),
                            "quantity group repeats a draw slot"
                        );
                    }
                    Some(Quantity::Remainder {
                        input: group,
                        after: closing,
                    }) if *group == input => {
                        ensure!(
                            candidate.kind == Kind::Fluid && remainder.is_none(),
                            "quantity group has multiple remainders"
                        );
                        remainder = Some(closing);
                    }
                    _ => {}
                }
            }
            let closing = remainder.context("quantity group lacks its remainder")?;
            ensure!(
                closing.len() == draws.len()
                    && closing.iter().copied().collect::<BTreeSet<_>>() == draws,
                "quantity remainder does not include every draw"
            );
            let source = recipe
                .inputs
                .iter()
                .find(|row| row.kind == Kind::Fluid && row.slot == input)
                .context("quantity refers to a missing fluid input")?;
            ensure!(
                source.choices.len() == 1
                    && matches!(source.choices[0].consume, Consumption::Consume)
                    && source.choices[0].returns.is_empty(),
                "quantity requires one consumed fluid input alternative"
            );
            let initial = integer(&source.choices[0].amount, 1, i64::MAX)?;
            let (mut low, mut high) = (initial, initial);
            let mut used = BTreeSet::new();
            for (index, slot) in after.iter().enumerate() {
                ensure!(
                    *slot != output.slot && used.insert(*slot),
                    "quantity has a cyclic or repeated draw"
                );
                let prior = recipe
                    .outputs
                    .iter()
                    .find(|row| row.kind == Kind::Fluid && row.slot == *slot)
                    .context("quantity refers to a missing preceding output")?;
                let Some(Quantity::Draw {
                    input: prior_input,
                    after: prefix,
                    limit,
                }) = &prior.quantity
                else {
                    anyhow::bail!("quantity depends on an output that is not a draw");
                };
                ensure!(
                    *prior_input == input && prefix == &after[..index],
                    "quantity draw order or input is inconsistent"
                );
                ensure!(
                    prior.amount.is_none()
                        && prior.change.is_none()
                        && prior.chance.numerator == "1"
                        && prior.chance.denominator == "1",
                    "preceding draw has incompatible fixed fields"
                );
                let limit = integer(limit, 1, i64::MAX)?;
                ensure!(low > 1, "quantity draw can exhaust its input budget");
                low = (low - limit).max(1);
                high -= 1;
            }
            match rule {
                Quantity::Draw { limit, .. } => {
                    let limit = integer(limit, 1, i64::MAX)?;
                    ensure!(low > 1, "quantity draw can exhaust its input budget");
                    Ok((1, limit.min(high - 1)))
                }
                Quantity::Remainder { .. } => Ok((low, high)),
                _ => unreachable!(),
            }
        }
    }
}
