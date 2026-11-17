//! Avaritia 1.77: native 9x9 selectors, then ordinary registered results only
//! when the earlier two-unit-stack repair branch does not apply.
use super::{Consumption, CraftingSelector, Grid, Item, Kind, Match, MatchCase, OutputRole, Recipe};
use anyhow::{ensure, Context, Result};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Process {
    pub native_index: u32,
    pub earlier: Vec<CraftingSelector>,
    /// Complete intersection of item types admitted by both occupied cells.
    /// The Item.isRepairable branch is audited by the native producer. Maximum
    /// is the signed native no-argument field, never a guessed tool quantity.
    pub repair: Vec<Repair>,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Repair {
    pub item: String,
    pub enabled: bool,
    pub maximum: i32,
}

fn grid(value: Option<&Grid>, inputs: usize) -> Result<()> {
    if let Some(g) = value {
        ensure!((1..=9).contains(&g.width) && (1..=9).contains(&g.height)
            && g.cells.len() == (g.width * g.height) as usize, "invalid extreme grid dimensions");
        let occupied: Vec<_> = g.cells.iter().flatten().copied().collect();
        ensure!(occupied.len() == inputs && occupied.iter().copied().collect::<BTreeSet<_>>() == (0..inputs as u32).collect(),
            "extreme grid must use each occupied input once");
    }
    Ok(())
}

fn predicate(id: &str, rule: &Match, items: &BTreeMap<&str, &Item>) -> Result<()> {
    let item = items.get(id).context("missing extreme selector item")?;
    ensure!(item.meta != 32767, "extreme selector needs concrete visual metadata");
    ensure!(match rule {
        Match::Exact => item.nbt.is_some(),
        Match::Wildcard { meta, nbt } => (*meta || *nbt) && (*nbt || item.nbt.is_some()),
        Match::NativeItemMetadata { nbt, .. } => *nbt || item.nbt.is_some(),
        _ => false,
    }, "extreme selector lost its native metadata/conditional-NBT predicate");
    Ok(())
}

pub(super) fn validate(p: &Process, recipe: &Recipe, items: &BTreeMap<&str, &Item>) -> Result<()> {
    let shaped = match recipe.source.handler.as_str() {
        "fox.spiteful.avaritia.compat.nei.ExtremeShapedRecipeHandler" => true,
        "fox.spiteful.avaritia.compat.nei.ExtremeShapelessRecipeHandler" => false,
        _ => anyhow::bail!("extreme process is bound to native Avaritia handlers"),
    };
    ensure!(recipe.grid.is_some() == shaped && (1..=81).contains(&recipe.inputs.len())
        && recipe.magic.is_none() && recipe.duration.is_none() && recipe.energy.is_none(), "invalid extreme recipe shape");
    ensure!(p.native_index < 262144 && p.earlier.len() <= p.native_index as usize, "invalid extreme native order");
    grid(recipe.grid.as_ref(), recipe.inputs.len())?;
    for (index, input) in recipe.inputs.iter().enumerate() {
        ensure!(input.kind == Kind::Item && input.slot == index as u32 && !input.choices.is_empty(), "invalid extreme input cell");
        for choice in &input.choices {
            ensure!(choice.amount == "1" && matches!(choice.consume, Consumption::Consume | Consumption::Keep
                | Consumption::Damage { .. } | Consumption::GtTool { .. } | Consumption::NativeContainer { .. }), "extreme crafting consumes one item or an audited container/tool rule");
            predicate(&choice.id, &choice.rule, items)?;
        }
    }
    ensure!(recipe.outputs.len() == 1, "extreme crafting has one registered result; container returns belong to inputs");
    let output = &recipe.outputs[0];
    ensure!(output.kind == Kind::Item && output.slot == 0 && matches!(output.role, OutputRole::Result)
        && output.amount.is_some() && output.quantity.is_none() && output.change.is_none()
        && output.chance.numerator == "1" && output.chance.denominator == "1", "invalid extreme registered result");
    let mut total = 0usize;
    for prior in &p.earlier {
        ensure!((1..=81).contains(&prior.inputs.len()), "invalid preceding extreme selector count");
        grid(prior.grid.as_ref(), prior.inputs.len())?;
        for choices in &prior.inputs {
            ensure!(!choices.is_empty() && choices.len() <= 65536, "empty/oversized preceding extreme alternatives");
            total += choices.len();
            ensure!(total <= 1048576, "extreme selector candidates exceed budget");
            for MatchCase { id, rule } in choices { predicate(id, rule, items)?; }
        }
    }
    validate_repair(&p.repair, recipe, items)
}

/// Both pinned managers run the same two-unit-stack Item repair predicate
/// before their registry. Require the complete item-type intersection.
pub(super) fn validate_repair(repairs: &[Repair], recipe: &Recipe, items: &BTreeMap<&str, &Item>) -> Result<()> {
    let registries = |slot: usize| -> BTreeSet<&str> {
        recipe.inputs[slot].choices.iter().map(|c|items[c.id.as_str()].registry.as_str()).collect()
    };
    let expected = if recipe.inputs.len() == 2 { registries(0).intersection(&registries(1)).copied().collect() } else { BTreeSet::new() };
    let mut actual = BTreeSet::new();
    for repair in repairs {
        let item = items.get(repair.item.as_str()).context("missing native repair item")?;
        ensure!(actual.insert(item.registry.as_str()) && expected.contains(item.registry.as_str())
            && (!repair.enabled || repair.maximum > 0), "invalid or duplicate native repair condition");
    }
    ensure!(actual == expected, "process omitted a potential native repair item");
    Ok(())
}
