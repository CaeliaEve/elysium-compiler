//! Pinned IC2 Advanced matching and complete finite count-one result observations.
//! Native electric callbacks are Java observations, not an arbitrary-player-NBT interpreter.
use super::{change::Stack, Consumption, Edit, Item, Kind, Match, OutputRole, Process, Recipe};
use crate::identity::{integer, Nbt};
use anyhow::{ensure, Context, Result};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

const LIMIT: usize = 65536;
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[schemars(rename = "Ic2Advanced")]
pub struct Advanced {
    pub version: Version,
    pub scope: Scope,
    pub native_index: u32,
    pub layout: Layout,
    pub visibility: Visibility,
    pub prototype: Stack,
    pub charges: Vec<Vec<String>>,
    pub states: Vec<super::gt_crafting::GtCraftingWitness>,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[schemars(rename = "Ic2AdvancedVersion")]
pub enum Version {
    #[serde(rename = "2.2.828-experimental")]
    Installed,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[schemars(rename = "Ic2AdvancedScope")]
pub enum Scope {
    #[serde(rename = "exportedCountOneCandidates")]
    ExportedCountOneCandidates,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[schemars(rename = "Ic2AdvancedLayout")]
pub struct Layout {
    pub class: String,
    pub shapeless: bool,
    pub width: u8,
    pub height: u8,
    pub input_count: u8,
    pub masks: Option<Vec<u16>>,
    pub mirrored_masks: Option<Vec<u16>>,
    pub mirrored_order: Option<Vec<u8>>,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[schemars(rename = "Ic2AdvancedVisibility")]
pub struct Visibility {
    pub hidden: bool,
    pub hide_secret_recipes: Option<bool>,
    pub config_value: Option<String>,
}
#[derive(Clone, Copy)]
struct Candidate {
    charge: f64,
    mask: u16,
}
fn bits(value: &str) -> Result<u64> {
    ensure!(
        value.len() == 16
            && value
                .bytes()
                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b)),
        "IC2 double observation is not canonical raw bits"
    );
    Ok(u64::from_str_radix(value, 16)?)
}
fn placements(layout: &Layout) -> Result<Vec<Vec<u8>>> {
    let count = layout.input_count as usize;
    ensure!((1..=9).contains(&count), "Invalid IC2 compact input count");
    if layout.shapeless {
        ensure!(
            layout.class == "ic2.core.AdvShapelessRecipe"
                && layout.width == 0
                && layout.height == 0
                && layout.masks.is_none()
                && layout.mirrored_masks.is_none()
                && layout.mirrored_order.is_none(),
            "IC2 shapeless rule acquired shaped masks"
        );
        return Ok(vec![]);
    }
    ensure!(
        layout.class == "ic2.core.AdvRecipe"
            && (1..=3).contains(&layout.width)
            && (1..=3).contains(&layout.height),
        "Invalid IC2 shaped native dimensions/class"
    );
    ensure!(
        layout.mirrored_masks.is_some() == layout.mirrored_order.is_some(),
        "Incomplete IC2 native mirrored layout"
    );
    let base: Vec<u8> = (0..count as u8).collect();
    let mut result = Vec::new();
    for (masks, order) in [
        (layout.masks.as_ref().context("IC2 masks missing")?, &base),
        (
            layout.mirrored_masks.as_ref().unwrap_or(&Vec::new()),
            layout.mirrored_order.as_ref().unwrap_or(&base),
        ),
    ] {
        ensure!(
            masks.len() <= 9
                && order.len() == count
                && order.iter().copied().collect::<BTreeSet<_>>() == base.iter().copied().collect(),
            "IC2 mirrored compact order is not a permutation"
        );
        for mask in masks {
            ensure!(
                *mask > 0 && *mask <= 511 && mask.count_ones() == count as u32,
                "IC2 mask lost native compact cells"
            );
            let mut positions = vec![0; count];
            let mut input = 0;
            for cell in 0..9 {
                if mask & (1 << (8 - cell)) != 0 {
                    positions[order[input] as usize] = cell;
                    input += 1;
                }
            }
            result.push(positions);
        }
    }
    ensure!(
        !result.is_empty() && !layout.masks.as_ref().unwrap().is_empty(),
        "IC2 shaped mask list is empty"
    );
    Ok(result)
}
fn expected(
    groups: &[Vec<Candidate>],
    paths: &[Vec<u8>],
    shapeless: bool,
) -> Result<BTreeSet<u64>> {
    let count = groups.len();
    let mut outputs = BTreeSet::new();
    let mut transitions = 0usize;
    for path in if shapeless {
        vec![vec![]]
    } else {
        paths.to_vec()
    } {
        let mut order: Vec<_> = (0..count).collect();
        if !shapeless {
            order.sort_by_key(|slot| path[*slot]);
        }
        let mut states =
            BTreeSet::from([(0u16, if shapeless { (1 << count) - 1 } else { 0 }, 0u64)]);
        for depth in 0..count {
            let mut next = BTreeSet::new();
            for (used, remaining, total) in states {
                for (slot, group) in groups.iter().enumerate() {
                    if used & (1 << slot) != 0 || !shapeless && order[depth] != slot {
                        continue;
                    }
                    for candidate in group {
                        transitions += 1;
                        ensure!(
                            transitions <= 8388608,
                            "IC2 state transitions exceed budget"
                        );
                        let mut left = remaining;
                        if shapeless {
                            let available = left & candidate.mask;
                            if available == 0 {
                                continue;
                            }
                            left &= !(available & available.wrapping_neg());
                        }
                        next.insert((
                            used | (1 << slot),
                            left,
                            (f64::from_bits(total) + candidate.charge).to_bits(),
                        ));
                        ensure!(next.len() <= LIMIT, "IC2 state count exceeds budget");
                    }
                }
            }
            states = next;
        }
        for (_, remaining, total) in states {
            ensure!(remaining == 0, "Incomplete IC2 greedy allocation");
            outputs.insert(total);
        }
        ensure!(outputs.len() <= LIMIT, "IC2 output count exceeds budget");
    }
    Ok(outputs)
}
pub(super) fn validate(recipe: &Recipe, items: &BTreeMap<&str, &Item>) -> Result<()> {
    let Some(Process::Ic2Advanced(p)) = &recipe.process else {
        anyhow::bail!("IC2 Advanced output requires its native process");
    };
    let count = recipe.inputs.len();
    let paths = placements(&p.layout)?;
    ensure!(
        count == p.layout.input_count as usize
            && p.native_index < 262144
            && p.charges.len() == count,
        "IC2 native index/input observations differ"
    );
    ensure!(
        recipe.source.owner == "IC2"
            && recipe.source.handler
                == format!(
                    "ic2.neiIntegration.core.recipehandler.{}",
                    if p.layout.shapeless {
                        "AdvShapelessRecipeHandler"
                    } else {
                        "AdvRecipeHandler"
                    }
                ),
        "IC2 Advanced belongs to its native handler"
    );
    ensure!(
        recipe.duration.is_none() && recipe.energy.is_none() && recipe.magic.is_none(),
        "IC2 manual crafting acquired machine costs"
    );
    let parsed = p
        .visibility
        .config_value
        .as_deref()
        .is_some_and(|value| value.eq_ignore_ascii_case("true"));
    ensure!(
        !p.visibility.hidden
            && p.visibility.hide_secret_recipes.is_none()
            && p.visibility.config_value.is_none()
            || p.visibility.hidden && p.visibility.hide_secret_recipes == Some(parsed) && !parsed,
        "IC2 recipe visibility differs from canShow"
    );
    if p.layout.shapeless {
        ensure!(
            recipe.grid.is_none(),
            "IC2 shapeless recipe acquired a grid"
        );
    } else {
        let grid = recipe.grid.as_ref().context("IC2 shaped grid missing")?;
        let mut cells = vec![None; 9];
        for (slot, cell) in paths[0].iter().enumerate() {
            cells[*cell as usize] = Some(slot as u32);
        }
        ensure!(
            grid.width == 3
                && grid.height == 3
                && grid.cells == cells
                && grid.mirror == p.layout.mirrored_order.is_some(),
            "IC2 display grid lost native compact cells"
        );
    }
    let mut groups = Vec::new();
    let mut total = 0;
    for (slot, input) in recipe.inputs.iter().enumerate() {
        ensure!(
            input.slot == slot as u32
                && input.kind == Kind::Item
                && !input.choices.is_empty()
                && p.charges[slot].len() == input.choices.len(),
            "IC2 input/charge shape differs"
        );
        total += input.choices.len();
        ensure!(total <= LIMIT, "IC2 candidates exceed budget");
        let mut group = Vec::new();
        for (index, choice) in input.choices.iter().enumerate() {
            ensure!(
                choice.amount == "1"
                    && matches!(choice.rule, Match::Wildcard { nbt: true, .. })
                    && matches!(
                        choice.consume,
                        Consumption::Consume
                            | Consumption::Keep
                            | Consumption::Damage { .. }
                            | Consumption::GtTool { .. } | Consumption::NativeContainer { .. }
                    ),
                "IC2 input lost one-per-cell or native metadata/NBT/container semantics"
            );
            let offered = items
                .get(choice.id.as_str())
                .context("IC2 offered item missing")?;
            ensure!(
                offered.meta != 32767,
                "IC2 offered item remained a wildcard"
            );
            let mut mask = 0;
            for (required, input) in recipe.inputs.iter().enumerate() {
                for template in &input.choices {
                    let Match::Wildcard { meta, nbt: true } = template.rule else {
                        anyhow::bail!("IC2 requirement lost native predicate");
                    };
                    let template = items
                        .get(template.id.as_str())
                        .context("IC2 native requirement missing")?;
                    if offered.registry == template.registry
                        && (meta || offered.meta == template.meta)
                    {
                        mask |= 1 << required;
                        break;
                    }
                }
            }
            group.push(Candidate {
                charge: f64::from_bits(bits(&p.charges[slot][index])?),
                mask,
            });
        }
        groups.push(group);
    }
    let prototype = items
        .get(p.prototype.id.as_str())
        .context("IC2 native output prototype missing")?;
    integer(&p.prototype.amount, 1, i32::MAX.into())?;
    ensure!(
        prototype
            .nbt
            .as_ref()
            .is_none_or(|tag| matches!(tag, Nbt::Compound { .. })),
        "Invalid IC2 prototype tags"
    );
    ensure!(
        recipe.outputs.len() == 1,
        "IC2 crafting requires one output"
    );
    let output = &recipe.outputs[0];
    let change = output
        .change
        .as_ref()
        .context("IC2 native output observations missing")?;
    ensure!(
        output.kind == Kind::Item
            && output.slot == 0
            && matches!(output.role, OutputRole::Result)
            && output.quantity.is_none()
            && output.chance.numerator == "1"
            && output.chance.denominator == "1"
            && matches!(change.action, Edit::Ic2Advanced {})
            && change.input == 0
            && change.bindings.is_none(),
        "Invalid IC2 native result projection"
    );
    ensure!(
        !p.states.is_empty() && p.states.len() <= LIMIT && change.samples.len() == p.states.len(),
        "IC2 output observations omit native states"
    );
    ensure!(
        output.id == change.samples[0].id
            && output.amount.as_deref() == Some(change.samples[0].amount.as_str()),
        "IC2 default output differs from its first observation"
    );
    let mut captured = BTreeSet::new();
    for (witness, sample) in p.states.iter().zip(&change.samples) {
        ensure!(
            witness.choices.len() == count
                && witness.positions.len() == count
                && witness
                    .positions
                    .iter()
                    .copied()
                    .collect::<BTreeSet<_>>()
                    .len()
                    == count
                && witness.positions.iter().all(|cell| *cell < 9),
            "Invalid IC2 native witness positions"
        );
        if !p.layout.shapeless {
            ensure!(
                paths.contains(&witness.positions),
                "IC2 witness is outside native occupancy masks"
            );
        }
        let mut order: Vec<_> = (0..count).collect();
        order.sort_by_key(|slot| witness.positions[*slot]);
        let mut remaining = (1u16 << count) - 1;
        let mut total = 0.0;
        for slot in order {
            let candidate = groups[slot]
                .get(witness.choices[slot] as usize)
                .context("IC2 witness choice is missing")?;
            if p.layout.shapeless {
                let available = remaining & candidate.mask;
                ensure!(
                    available != 0,
                    "IC2 witness violates first-remaining greedy matching"
                );
                remaining &= !(available & available.wrapping_neg());
            }
            total += candidate.charge;
        }
        ensure!(
            captured.insert(total.to_bits()),
            "IC2 output observations duplicate a result state"
        );
        let item = items
            .get(sample.id.as_str())
            .context("IC2 observed output item missing")?;
        ensure!(
            sample.amount == p.prototype.amount
                && item
                    .nbt
                    .as_ref()
                    .is_none_or(|tag| matches!(tag, Nbt::Compound { .. })),
            "IC2 observed output changed native count or compound tags"
        );
    }
    ensure!(
        captured == expected(&groups, &paths, p.layout.shapeless)?,
        "IC2 observations omit or invent reachable result states"
    );
    Ok(())
}
