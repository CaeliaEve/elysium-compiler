//! ExtraUtilities 1.2.12 native RecipeMicroBlocks and its shared FMP lookup.
use super::{Consumption, Edit, Item, Kind, Match, OutputRole, Recipe, Stack};
use crate::identity::{item_id, Nbt};
use anyhow::{ensure, Context, Result};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(rename = "XuNativeMaterial")]
pub struct Material {
    pub id: u32,
    pub name: String,
    pub block: String,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[schemars(rename = "XuNativeSettings")]
pub struct Settings {
    pub native_recipe: String,
    pub version: String,
    pub fmp_version: String,
    pub micro_registry: String,
    pub lookup: Lookup,
    pub alias_policy: super::fmp_program::AliasPolicy,
    pub missing_policy: super::fmp_program::MissingPolicy,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(rename = "XuLookupState")]
pub enum Lookup {
    Unused,
    Ready,
}
#[derive(Clone, Debug, Default, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(rename = "XuNativeProgram")]
pub struct Program {
    pub materials: Vec<Material>,
    pub names: Vec<super::fmp_program::Name>,
    pub aliases: Vec<super::fmp_program::Alias>,
    pub settings: Vec<Settings>,
}
impl Program {
    pub fn validate(&self) -> Result<()> {
        ensure!(
            self.settings.len() == 1
                && self.materials.len() <= 65536
                && self.names.len() <= 262144
                && self.aliases.len() <= 262144,
            "XU lookup exceeds source budget"
        );
        let settings = &self.settings[0];
        ensure!(
            settings.native_recipe
                == "com.rwtema.extrautils.multipart.microblock.RecipeMicroBlocks"
                && settings.version == "1.2.12"
                && settings.fmp_version == "1.7.2"
                && !settings.micro_registry.is_empty(),
            "Unadapted XU native definition"
        );
        ensure!(
            settings.lookup != Lookup::Unused
                || self.materials.is_empty() && self.names.is_empty() && self.aliases.is_empty(),
            "Unused XU lookup cannot invent material facts"
        );
        let mut seen = BTreeSet::new();
        for (index, material) in self.materials.iter().enumerate() {
            ensure!(
                material.id == index as u32
                    && seen.insert(&material.name)
                    && material.name.len() <= 65535
                    && material
                        .block
                        .strip_prefix("item_")
                        .is_some_and(crate::source::is_digest),
                "XU ordered material facts changed"
            );
        }
        ensure!(
            self.names.windows(2).all(|w| w[0]
                .name
                .encode_utf16()
                .cmp(w[1].name.encode_utf16())
                .is_lt())
                && self.aliases.windows(2).all(|w| w[0]
                    .name
                    .encode_utf16()
                    .cmp(w[1].name.encode_utf16())
                    .is_lt()),
            "XU lookup keys duplicate or unordered"
        );
        for entry in &self.names {
            ensure!(entry.name.len() <= 65535, "XU native name exceeds budget");
        }
        for entry in &self.aliases {
            ensure!(
                entry.name.len() <= 65535 && entry.target.len() <= 65535,
                "XU native alias exceeds budget"
            );
        }
        Ok(())
    }
    fn micro_registry(&self) -> &str {
        &self.settings[0].micro_registry
    }
    fn resolve(&self, name: &str) -> i32 {
        let name = self
            .aliases
            .iter()
            .find(|a| a.name == name)
            .map_or(name, |a| a.target.as_str());
        self.names
            .iter()
            .find(|n| n.name == name)
            .map_or(0, |n| n.id)
    }
    fn micro_id(&self, item: &Item, projection: Option<&str>) -> Result<i32> {
        let tag = match &item.nbt {
            Some(Nbt::Compound { value }) => value.get("mat"),
            None => None,
            _ => anyhow::bail!("XU micro NBT must be a compound"),
        };
        let Some(tag) = tag else {
            ensure!(
                projection.is_none(),
                "XU invented material text for absent mat"
            );
            return Ok(0);
        };
        let name = if let Nbt::String { value } = tag {
            ensure!(
                projection == Some(value.as_str()),
                "XU observed mat differs from string tag"
            );
            value.as_str()
        } else {
            projection.context("XU non-string mat requires observed native NBT text")?
        };
        ensure!(
            name.len() <= 524288,
            "XU native NBT projection exceeds budget"
        );
        Ok(self.resolve(name))
    }
    fn full_id(&self, item: &Item, items: &BTreeMap<&str, &Item>) -> Result<i32> {
        for material in &self.materials {
            let block = items
                .get(material.block.as_str())
                .context("XU native material block missing")?;
            if block.registry == item.registry && block.meta == item.meta && block.nbt == item.nbt {
                return Ok(self.resolve(&material.name));
            }
        }
        Ok(-1)
    }
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(untagged, deny_unknown_fields)]
#[schemars(rename = "XuNativeCell")]
pub enum Cell {
    Integer(i32),
    Literal { stack: Literal },
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(rename = "XuNativeLiteralStack")]
pub struct Literal {
    pub registry: String,
    pub meta: i32,
    pub amount: String,
    pub nbt: Option<Nbt>,
}
#[derive(Clone, Copy, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(rename = "XuNativeScope")]
pub enum Scope {
    NativeRegistryWitnesses,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[schemars(rename = "XuNativeState")]
pub struct State {
    pub offered_mirrored: bool,
    pub matched_mirrored: bool,
    pub first_part: Option<u32>,
    pub output: Stack,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[schemars(rename = "XuNativeProcedure")]
pub struct Procedure {
    pub native_recipe: String,
    pub native_index: u32,
    pub program: String,
    pub scope: Scope,
    pub material_id: Option<i32>,
    pub cells: Vec<Option<Cell>>,
    pub layout: Vec<Option<u32>>,
    pub material_texts: Vec<Option<String>>,
    pub prototype: Stack,
    pub states: Vec<State>,
}

struct Outcome {
    mirrored: bool,
    material: Option<i32>,
    first: Option<u32>,
    output: Stack,
}
fn evaluate(
    p: &Procedure,
    rules: &Program,
    grid: &[Option<(&Item, u32)>; 9],
    items: &BTreeMap<&str, &Item>,
    projections: &BTreeMap<u32, Option<&str>>,
) -> Result<Option<Outcome>> {
    for mirrored in [true, false] {
        let mut accepted = true;
        let mut material = 0;
        'cells: for x in 0..3 {
            for y in 0..3 {
                let index = (if mirrored { 2 - x } else { x }) + y * 3;
                let offered = grid[x + y * 3];
                match (&p.cells[index], offered) {
                    (None, None) => {}
                    (Some(Cell::Literal { stack }), Some((offered, _))) => {
                        if offered.registry != stack.registry
                            || stack.meta != 32767 && offered.meta != stack.meta
                        {
                            accepted = false;
                            break 'cells;
                        }
                    }
                    (Some(Cell::Integer(shape)), Some((offered, slot))) => {
                        let id = if *shape == 0 {
                            if offered.registry == rules.micro_registry() {
                                accepted = false;
                                break 'cells;
                            }
                            let id = rules.full_id(offered, items)?;
                            if id <= 0 {
                                accepted = false;
                                break 'cells;
                            }
                            id
                        } else {
                            if offered.registry != rules.micro_registry()
                                || !matches!(offered.nbt, Some(Nbt::Compound { .. }))
                            {
                                accepted = false;
                                break 'cells;
                            }
                            let id = rules
                                .micro_id(offered, projections.get(&slot).copied().flatten())?;
                            if id == 0 {
                                accepted = false;
                                break 'cells;
                            }
                            id
                        };
                        if material == 0 {
                            material = id;
                        } else if material != id {
                            accepted = false;
                            break 'cells;
                        }
                    }
                    _ => {
                        accepted = false;
                        break 'cells;
                    }
                }
            }
        }
        if accepted {
            let prototype = items
                .get(p.prototype.id.as_str())
                .context("XU output prototype absent")?;
            let first = grid.iter().flatten().find(|(item, _)| {
                item.registry == rules.micro_registry()
                    && matches!(item.nbt, Some(Nbt::Compound { .. }))
            });
            let nbt = first.map_or(prototype.nbt.as_ref(), |(item, _)| item.nbt.as_ref());
            return Ok(Some(Outcome {
                mirrored,
                material: (material != 0).then_some(material),
                first: first.map(|(_, slot)| *slot),
                output: Stack {
                    id: item_id(&prototype.registry, prototype.meta, nbt)?,
                    amount: p.prototype.amount.clone(),
                },
            }));
        }
    }
    Ok(None)
}

pub(super) fn validate(p: &Procedure, r: &Recipe, items: &BTreeMap<&str, &Item>) -> Result<()> {
    ensure!(
        r.source.handler == "com.rwtema.extrautils.nei.MicroBlocksHandler"
            && p.native_recipe == "com.rwtema.extrautils.multipart.microblock.RecipeMicroBlocks"
            && p.program
                .strip_prefix("program_")
                .is_some_and(crate::source::is_digest)
            && p.native_index < 262144
            && p.cells.len() == 9
            && p.layout.len() == 9
            && p.states.len() == 2
            && !p.states[0].offered_mirrored
            && p.states[1].offered_mirrored
            && r.grid.is_none()
            && r.magic.is_none()
            && r.duration.is_none()
            && r.energy.is_none()
            && r.outputs.len() == 1,
        "Invalid XU native witness process"
    );
    crate::identity::integer(&p.prototype.amount, 1, i32::MAX as i64)?;
    ensure!(
        items.contains_key(p.prototype.id.as_str()),
        "XU prototype missing"
    );
    let mut slots = BTreeSet::new();
    for (cell, slot) in p.cells.iter().zip(&p.layout) {
        ensure!(
            cell.is_some() == slot.is_some(),
            "XU physical empty cells changed"
        );
        if let Some(slot) = slot {
            ensure!(slots.insert(*slot), "XU repeats an input cell");
        }
    }
    for cell in p.cells.iter().flatten() {
        if let Cell::Literal { stack } = cell {
            crate::identity::integer(&stack.amount, i32::MIN as i64, i32::MAX as i64)?;
            item_id(&stack.registry, stack.meta, stack.nbt.as_ref())?;
        }
    }
    ensure!(
        slots.len() == r.inputs.len()
            && r.inputs.iter().all(|input| input.kind == Kind::Item
                && slots.contains(&input.slot)
                && input.choices.len() == 1),
        "XU inputs lack physical witness cells"
    );
    ensure!(
        p.material_texts.len() == r.inputs.len()
            && r.inputs
                .iter()
                .enumerate()
                .all(|(index, input)| input.slot == index as u32),
        "XU material text observations lack their ordered input slots"
    );
    let output = &r.outputs[0];
    ensure!(
        output.kind == Kind::Item
            && output.slot == 0
            && matches!(output.role, OutputRole::Result)
            && output.quantity.is_none()
            && output.chance.numerator == "1"
            && output.chance.denominator == "1"
            && output.id == p.states[0].output.id
            && output.amount.as_deref() == Some(p.states[0].output.amount.as_str()),
        "XU default output differs from its native state"
    );
    for state in &p.states {
        crate::identity::integer(&state.output.amount, 1, i32::MAX as i64)?;
    }
    if let Some(change) = &output.change {
        ensure!(
            !r.inputs.is_empty()
                && r.inputs.iter().any(|input| input.slot == change.input)
                && matches!(change.action, Edit::XuResult {})
                && change.samples.len() == 2,
            "XU lost its conditional native result"
        );
        let bindings = change
            .bindings
            .as_ref()
            .context("XU native result samples lack bindings")?;
        ensure!(
            bindings.len() == 2
                && bindings
                    .iter()
                    .all(|b| b.len() == r.inputs.len() && b.iter().all(|index| *index == Some(0))),
            "XU witness binding differs from its offered cells"
        );
        for (sample, state) in change.samples.iter().zip(&p.states) {
            ensure!(
                sample.id == state.output.id && sample.amount == state.output.amount,
                "XU native state/sample mismatch"
            );
        }
    } else {
        ensure!(
            p.states.iter().all(|state| state.first_part.is_none()
                && state.output.id == p.prototype.id
                && state.output.amount == p.prototype.amount),
            "XU dynamic NBT result cannot become a fixed output"
        );
    }
    Ok(())
}

pub(super) fn validate_program(
    p: &Procedure,
    r: &Recipe,
    rules: &Program,
    category: &super::Category,
    items: &BTreeMap<&str, &Item>,
) -> Result<()> {
    validate(p, r, items)?;
    ensure!(
        category.program.as_ref() == Some(&p.program),
        "XU category/process program differs"
    );
    ensure!(
        rules.settings[0].lookup == Lookup::Ready
            || !p
                .cells
                .iter()
                .flatten()
                .any(|cell| matches!(cell, Cell::Integer(_))),
        "Native integer template requires a ready XU material lookup"
    );
    let mut projections = BTreeMap::new();
    for (index, input) in r.inputs.iter().enumerate() {
        let item = items
            .get(input.choices[0].id.as_str())
            .context("XU observed input missing")?;
        let text = p.material_texts[index].as_deref();
        if item.registry == rules.micro_registry() {
            rules.micro_id(item, text)?;
        } else {
            ensure!(
                text.is_none(),
                "XU material text attached to an unrelated item"
            );
        }
        projections.insert(input.slot, text);
    }
    for (cell, slot) in p.cells.iter().zip(&p.layout) {
        let (Some(cell), Some(slot)) = (cell, slot) else {
            continue;
        };
        let input = r
            .inputs
            .iter()
            .find(|i| i.slot == *slot)
            .context("XU native cell input missing")?;
        ensure!(
            input.choices.len() == 1,
            "XU scope contains one native registry witness per cell"
        );
        let choice = &input.choices[0];
        let offered = items
            .get(choice.id.as_str())
            .context("XU input fact missing")?;
        ensure!(
            choice.amount == "1"
                && matches!(
                    choice.consume,
                    Consumption::Consume | Consumption::Keep | Consumption::Damage {..} | Consumption::GtTool {..} | Consumption::NativeContainer { .. }
                ),
            "XU native workbench consumes one per occupied cell"
        );
        match cell {
            Cell::Integer(0) => ensure!(
                matches!(choice.rule, Match::Exact)
                    && p.material_id.is_some_and(|id| id > 0)
                    && rules.full_id(offered, items)? == p.material_id.unwrap(),
                "XU full block changed first native material precedence"
            ),
            Cell::Integer(_) => {
                let Match::FmpMaterial {
                    program,
                    material_id,
                    native_text,
                } = &choice.rule
                else {
                    anyhow::bail!("XU micro material lost its shared lookup predicate")
                };
                ensure!(
                    offered.registry == rules.micro_registry()
                        && matches!(offered.nbt, Some(Nbt::Compound { .. }))
                        && program == &p.program
                        && p.material_id == Some(*material_id)
                        && *material_id != 0
                        && rules.micro_id(offered, native_text.as_deref())? == *material_id,
                    "XU nonzero material example rejected"
                );
                ensure!(
                    native_text.as_deref() == p.material_texts[input.slot as usize].as_deref(),
                    "XU material predicate/text observation differs"
                );
            }
            Cell::Literal { stack } => {
                let wildcard = stack.meta == 32767;
                ensure!(
                    offered.registry == stack.registry
                        && (wildcard || offered.meta == stack.meta)
                        && matches!(choice.rule,Match::Wildcard{meta,nbt:true}if meta==wildcard),
                    "XU literal changed native item/meta/ignored-NBT matching"
                );
            }
        }
    }
    for state in &p.states {
        let mut grid = [None; 9];
        for (cell, slot) in p.layout.iter().enumerate() {
            if let Some(slot) = slot {
                let input = r.inputs.iter().find(|input| input.slot == *slot).unwrap();
                let item = items[input.choices[0].id.as_str()];
                let target = if state.offered_mirrored {
                    cell / 3 * 3 + 2 - cell % 3
                } else {
                    cell
                };
                grid[target] = Some((item, *slot));
            }
        }
        let actual = evaluate(p, rules, &grid, items, &projections)?
            .context("Native XU rejects its offered witness")?;
        ensure!(
            actual.mirrored == state.matched_mirrored
                && actual.material == p.material_id
                && actual.first == state.first_part
                && actual.output.id == state.output.id
                && actual.output.amount == state.output.amount
                && items.contains_key(state.output.id.as_str()),
            "XU witness differs from native matcher or first-tagged result"
        );
    }
    Ok(())
}
