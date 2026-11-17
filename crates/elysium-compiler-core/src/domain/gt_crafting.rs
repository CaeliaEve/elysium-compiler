//! Installed GT result observations over a bounded exported-candidate domain.
//! This checks complete state coverage and native matching; it does not execute
//! Java callbacks or pretend samples implement arbitrary player NBT/charge.
use super::{change::Stack, Edit, Item, Match, Process, Recipe};
use crate::identity::{integer, Nbt};
use anyhow::{ensure, Context, Result};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

const STATES: usize = 65536;
const TRANSITIONS: usize = 8388608;

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct GtCrafting {
    pub version: GtCraftingVersion,
    pub scope: GtCraftingScope,
    pub rules: GtCraftingRules,
    pub prototype: Stack,
    pub base: Stack,
    /// Native simulated discharge for each concrete offered count-one specimen.
    pub transfers: Vec<Vec<i32>>,
    /// One actual native result witness per distinct rule-relevant result state.
    /// Index-aligned with output.change.samples, never a Cartesian tuple table.
    pub states: Vec<GtCraftingWitness>,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub enum GtCraftingVersion { #[serde(rename = "5.09.51.482")] Installed }
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub enum GtCraftingScope { #[serde(rename = "exportedCandidates")] ExportedCandidates }

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct GtCraftingRules {
    pub keeping_nbt: bool,
    pub overwrite_nbt: bool,
    pub removable: bool,
    pub enchantments: Vec<GtCraftingEnchantment>,
    pub declared_levels: Vec<i32>,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct GtCraftingEnchantment {
    pub id: i32,
    pub class: String,
    pub level: i32,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct GtCraftingWitness {
    pub choices: Vec<u32>,
    /// Physical 3x3 inventory cell for each logical input, not GUI coordinates.
    pub positions: Vec<u8>,
}

#[derive(Clone)]
struct Candidate<'a> { item: &'a Item, tag: Option<String>, mask: u16, transfer: i32 }
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
struct Key { used: u16, remaining: u16, transfer: i32, inherited: Option<String>, overwritten: Option<String> }
#[derive(Clone)]
struct State { key: Key, inherited: Option<(usize, usize)> }
type ResultKey = (i32, Option<String>, Option<String>);

impl Key {
    fn result(&self) -> ResultKey { (self.transfer, self.inherited.clone(), self.overwritten.clone()) }
}

fn same_tags(left: Option<&Nbt>, right: Option<&Nbt>) -> Result<bool> {
    match (left, right) { (None, None) => Ok(true), (Some(a), Some(b)) => a.native_equal(b), _ => Ok(false) }
}
fn stack_match(template: &Item, offered: &Item, wildcard: bool) -> bool {
    template.registry == offered.registry && (wildcard
        || super::native_item_metadata::value(template) == super::native_item_metadata::value(offered))
}
fn initial(count: usize, shapeless: bool) -> State {
    State { key: Key { used: 0, remaining: if shapeless { (1 << count) - 1 } else { 0 },
        transfer: 0, inherited: None, overwritten: None }, inherited: None }
}
fn advance(state: &State, slot: usize, choice: usize, groups: &[Vec<Candidate<'_>>], rules: &GtCraftingRules,
    base: &Item, shapeless: bool) -> Result<Option<State>> {
    let candidate = &groups[slot][choice];
    if rules.keeping_nbt && state.key.used != 0 {
        let first = state.inherited.and_then(|(s,c)| groups[s][c].item.nbt.as_ref());
        if !same_tags(first, candidate.item.nbt.as_ref())? { return Ok(None); }
    }
    let mut next = state.clone();
    if shapeless {
        let available = next.key.remaining & candidate.mask;
        if available == 0 { return Ok(None); }
        next.key.remaining &= !(available & available.wrapping_neg());
    }
    if rules.keeping_nbt && state.key.used == 0 {
        next.inherited = candidate.tag.as_ref().map(|_| (slot,choice));
        next.key.inherited = candidate.tag.clone();
    }
    if !rules.keeping_nbt && rules.overwrite_nbt && next.key.overwritten.is_none() && candidate.tag.is_some()
        && candidate.item.registry == base.registry
        && (super::native_item_metadata::value(candidate.item) == super::native_item_metadata::value(base)
            || super::native_item_metadata::value(candidate.item) == 32767
            || super::native_item_metadata::value(base) == 32767) {
        next.key.overwritten = candidate.tag.clone();
    }
    next.key.used |= 1 << slot;
    next.key.transfer = next.key.transfer.wrapping_add(candidate.transfer);
    Ok(Some(next))
}

fn offered_orders(recipe: &Recipe) -> Result<Vec<Vec<usize>>> {
    let Some(grid) = &recipe.grid else { return Ok(Vec::new()); };
    let mut result = Vec::new();
    for mirrored in [false,true] {
        if mirrored && !grid.mirror { continue; }
        let mut order = Vec::new();
        for row in 0..grid.height as usize { for column in 0..grid.width as usize {
            let x = if mirrored { grid.width as usize - 1 - column } else { column };
            if let Some(slot) = grid.cells[row * grid.width as usize + x] { order.push(slot as usize); }
        }}
        ensure!(order.len() == recipe.inputs.len(), "GT grid omits logical inputs");
        result.push(order);
    }
    Ok(result)
}

fn possible_states(recipe: &Recipe, p: &GtCrafting, groups: &[Vec<Candidate<'_>>], base: &Item) -> Result<BTreeSet<ResultKey>> {
    let shapeless = recipe.grid.is_none();
    let orders = offered_orders(recipe)?;
    let count = groups.len();
    let mut outputs = BTreeSet::new();
    // An empty fixed order means all offered permutations, represented by used subsets.
    for order in if shapeless { vec![Vec::new()] } else { orders } {
        let mut transitions = 0;
        let start = initial(count,shapeless);
        let mut states = BTreeMap::from([(start.key.clone(),start)]);
        for depth in 0..count {
            let mut next = BTreeMap::new();
            for state in states.values() { for slot in 0..count {
                if state.key.used & (1 << slot) != 0 || !shapeless && order[depth] != slot { continue; }
                for choice in 0..groups[slot].len() {
                    transitions += 1; ensure!(transitions <= TRANSITIONS, "GT state transitions exceed budget");
                    if let Some(value) = advance(state,slot,choice,groups,&p.rules,base,shapeless)? {
                        next.entry(value.key.clone()).or_insert(value);
                        ensure!(next.len() <= STATES, "GT state count exceeds budget");
                    }
                }
            }}
            states = next;
        }
        outputs.extend(states.keys().map(Key::result));
        ensure!(outputs.len() <= STATES, "GT result count exceeds budget");
    }
    Ok(outputs)
}

fn witness_key(recipe: &Recipe, p: &GtCrafting, witness: &GtCraftingWitness,
    groups: &[Vec<Candidate<'_>>], base: &Item) -> Result<ResultKey> {
    let count = groups.len();
    ensure!(witness.choices.len() == count && witness.positions.len() == count, "GT witness does not bind all input slots");
    let cells: BTreeSet<_> = witness.positions.iter().copied().collect();
    ensure!(cells.len() == count && cells.iter().all(|cell| *cell < 9), "invalid GT offered inventory positions");
    if let Some(grid) = &recipe.grid {
        let mut valid = false;
        for mirrored in [false,true] { for y in 0..=(3-grid.height) { for x in 0..=(3-grid.width) {
            if mirrored && !grid.mirror { continue; }
            let mut matches = true;
            for (cell,slot) in grid.cells.iter().enumerate() { if let Some(slot) = slot {
                let column = cell as u32 % grid.width;
                let column = if mirrored { grid.width-1-column } else { column };
                matches &= witness.positions[*slot as usize] as u32 == (y+cell as u32/grid.width)*3+x+column;
            }}
            valid |= matches;
        }}}
        ensure!(valid, "GT witness changed the shaped logical grid");
    }
    let mut order:Vec<_>=(0..count).collect();order.sort_by_key(|slot|witness.positions[*slot]);
    let mut state = initial(count,recipe.grid.is_none());
    for slot in order {
        let choice = witness.choices[slot] as usize;
        ensure!(choice < groups[slot].len(), "GT witness candidate is missing");
        state = advance(&state,slot,choice,groups,&p.rules,base,recipe.grid.is_none())?.context("GT witness does not match native keeping/greedy rules")?;
    }
    Ok(state.key.result())
}

pub(super) fn validate(recipe: &Recipe, items: &BTreeMap<&str,&Item>) -> Result<()> {
    let Some(Process::GtCrafting(p)) = &recipe.process else { anyhow::bail!("GT result requires its native process"); };
    super::native_crafting::validate(recipe)?;
    ensure!(recipe.source.owner == "NEI" && recipe.source.key == recipe.source.handler
        && matches!(recipe.source.handler.as_str(), "codechicken.nei.recipe.ShapedRecipeHandler" | "codechicken.nei.recipe.ShapelessRecipeHandler"),
        "GT result belongs to native crafting handlers");
    ensure!(!p.rules.overwrite_nbt || recipe.grid.is_none(), "GT overwrite applies only to shapeless recipes");
    ensure!(p.rules.enchantments.len() <= 4096 && p.rules.declared_levels.len() <= 4096
        && p.rules.enchantments.len() <= p.rules.declared_levels.len(), "invalid GT enchantment declarations");
    for (index,enchantment) in p.rules.enchantments.iter().enumerate() {
        ensure!(!enchantment.class.is_empty() && enchantment.level == p.rules.declared_levels[index], "GT enchantment order/level changed");
    }
    let prototype = items.get(p.prototype.id.as_str()).context("GT prototype item missing")?;
    let base = items.get(p.base.id.as_str()).context("GT normalized base missing")?;
    integer(&p.prototype.amount,1,i32::MAX.into())?;
    ensure!(p.prototype.amount == p.base.amount && prototype.registry == base.registry,
        "GT normalization changed its native item/count");
    ensure!(p.transfers.len() == recipe.inputs.len(), "GT transfer observations omit inputs");
    let mut total = 0;
    let mut groups = Vec::new();
    for (slot,input) in recipe.inputs.iter().enumerate() {
        total += input.choices.len(); ensure!(total <= STATES, "GT candidate count exceeds budget");
        ensure!(p.transfers[slot].len() == input.choices.len(), "GT transfer observations omit candidates");
        let mut candidates = Vec::new();
        for (choice,specimen) in input.choices.iter().enumerate() {
            let item = *items.get(specimen.id.as_str()).context("GT input item missing")?;
            ensure!(item.meta != 32767 && item.nbt.as_ref().is_none_or(|tag| matches!(tag,Nbt::Compound{..})), "GT candidates must be concrete owned item compounds");
            let mut mask = 0;
            for (required,group) in recipe.inputs.iter().enumerate() { for template in &group.choices {
                let accepted = match template.rule {
                    Match::Wildcard { meta,nbt:true } => stack_match(items.get(template.id.as_str()).context("GT requirement item missing")?,item,meta),
                    Match::NativeItemMetadata { nbt:true, .. } => super::matching::simple(&template.rule,item,items.get(template.id.as_str()).context("GT requirement item missing")?),
                    _ => anyhow::bail!("GT ingredient lost native metadata/NBT matching"),
                };
                if accepted { mask |= 1 << required; break; }
            }}
            candidates.push(Candidate { item, mask, transfer:p.transfers[slot][choice],
                tag:item.nbt.as_ref().map(|tag|serde_json::to_value(tag).and_then(|value|serde_json::to_string(&value))).transpose()? });
        }
        groups.push(candidates);
    }
    let output = &recipe.outputs[0];let change = output.change.as_ref().context("GT result observations missing")?;
    ensure!(matches!(change.action,Edit::GtCrafting{}) && change.input == 0 && change.bindings.is_none(), "GT observations are multi-input states, not single-input projections or tuple products");
    ensure!(!p.states.is_empty() && p.states.len() <= STATES && change.samples.len() == p.states.len(), "GT observations omit native states");
    ensure!(output.id == change.samples[0].id && output.amount.as_deref() == Some(change.samples[0].amount.as_str()), "GT default output differs from its first observation");
    let expected = possible_states(recipe,p,&groups,base)?;let mut captured = BTreeSet::new();
    for (index,state) in p.states.iter().enumerate() {
        let key = witness_key(recipe,p,state,&groups,base)?;
        ensure!(captured.insert(key), "GT observations repeat a folded native state");
        let result = &change.samples[index];let item = items.get(result.id.as_str()).context("GT observed result item missing")?;
        ensure!(result.amount == p.prototype.amount && item.nbt.as_ref().is_none_or(|tag| matches!(tag,Nbt::Compound{..})), "GT observed result changed native count or compound tags");
    }
    ensure!(captured == expected, "GT observations omit or invent reachable result states");
    Ok(())
}
