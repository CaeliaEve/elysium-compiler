//! ForgeMultipart 1.7.2 MicroRecipe. Rows are witnesses of shared native rules.
use super::{
    fmp_program::Program, Consumption, Edit, Item, Kind, Match, OutputRole, Recipe, Stack,
};
use crate::identity::{item_id, Nbt};
use anyhow::{ensure, Context, Result};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum Branch {
    Hollow,
    Gluing,
    Thinning,
    Splitting,
    HollowFill,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum SawNbt {
    FreshWhenDamageable,
    Preserve,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Saw {
    pub registry: String,
    pub owner: String,
    pub strength: i32,
    pub max_strength: i32,
    pub durability: i32,
    pub damageable: bool,
    pub nbt: SawNbt,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Procedure {
    pub program: String,
    pub branch: Branch,
    pub material_id: u32,
    pub layout: Vec<Option<u32>>,
}

#[derive(Clone, Copy)]
struct Part {
    class: i32,
    size: i32,
}
struct Evaluator<'a> {
    process: &'a Procedure,
    rules: &'a Program,
    projections: BTreeMap<&'a str, Option<&'a str>>,
}
impl Evaluator<'_> {
    fn part(&self, item: &Item, items: &BTreeMap<&str, &Item>) -> Option<Part> {
        if item.registry == self.rules.settings().micro_registry {
            return self.micro(item);
        }
        (self.rules.find_material(item, items).ok() == Some(self.process.material_id as i32))
            .then_some(Part { class: 0, size: 8 })
    }
    fn micro(&self, item: &Item) -> Option<Part> {
        if item.registry != self.rules.settings().micro_registry {
            return None;
        }
        let projection = self.projections.get(item.id.as_str()).copied().flatten();
        (self.rules.micro_id(item, projection).ok() == Some(self.process.material_id as i32))
            .then_some(Part {
                class: item.meta >> 8,
                size: item.meta & 255,
            })
    }
    fn create(&self, count: i32, class: i32, size: i32) -> Result<Stack> {
        ensure!(count > 0, "Native micro result has nonpositive count");
        let material = self.rules.material(self.process.material_id)?;
        let id = if size == 8 {
            material.block.clone()
        } else {
            let tag = Nbt::Compound {
                value: BTreeMap::from([(
                    "mat".into(),
                    Nbt::String {
                        value: material.name.clone(),
                    },
                )]),
            };
            item_id(
                &self.rules.settings().micro_registry,
                ((class << 8) | size).max(0),
                Some(&tag),
            )?
        };
        Ok(Stack {
            id,
            amount: count.to_string(),
        })
    }
    /// Evaluate actual physical cells in native branch priority. Extra tags on
    /// micro inputs are discarded; size=8 returns the material block's own NBT.
    fn result(
        &self,
        grid: &[Option<&Item>; 9],
        items: &BTreeMap<&str, &Item>,
    ) -> Result<Option<(Branch, Stack)>> {
        if grid[4].is_none() {
            if let Some(first) = grid[0].and_then(|i| self.micro(i)) {
                if first.class == 0
                    && (1..9).filter(|i| *i != 4).all(|i| {
                        grid[i]
                            .and_then(|v| self.micro(v))
                            .is_some_and(|p| p.class == 0 && p.size == first.size)
                    })
                {
                    return Ok(Some((Branch::Hollow, self.create(8, 1, first.size)?)));
                }
            }
        }
        let parts: Option<Vec<Part>> = grid.iter().flatten().map(|i| self.micro(i)).collect();
        if let Some(parts) = parts {
            if parts.len() > 1 {
                let first = parts[0];
                if parts
                    .iter()
                    .all(|p| p.class == first.class && (first.class < 2 || p.size == first.size))
                {
                    let smallest = parts.iter().map(|p| p.size).min().unwrap();
                    let count = parts.len() as i32;
                    let sum: i32 = parts.iter().map(|p| p.size).sum();
                    let out = match (first.class, count) {
                        (3, 2) => Some(self.create(1, 0, smallest)?),
                        (2, 2) => Some(self.create(1, 3, smallest)?),
                        (2, 4) => Some(self.create(1, 0, smallest)?),
                        (0 | 1, _) => match [1, 2, 4].into_iter().find(|bit| bit & sum != 0) {
                            None => Some(self.create(sum / 8, 0, 8)?),
                            Some(bit) if bit > smallest => {
                                Some(self.create(sum / bit, first.class, bit)?)
                            }
                            _ => None,
                        },
                        _ => None,
                    };
                    if let Some(out) = out {
                        return Ok(Some((Branch::Gluing, out)));
                    }
                }
            }
        }
        let saw = grid.iter().enumerate().find_map(|(i, item)| {
            item.and_then(|item| {
                self.rules
                    .saws
                    .iter()
                    .find(|s| s.registry == item.registry)
                    .map(|s| (i, s))
            })
        });
        if let Some((i, saw)) = saw {
            let can = saw.strength
                >= self
                    .rules
                    .material(self.process.material_id)?
                    .cutter_strength
                || saw.strength == self.rules.settings().max_saw_strength;
            if i / 3 < 2 {
                if let Some(part) = grid[i + 3].and_then(|v| self.part(v, items)) {
                    if part.size != 1
                        && can
                        && grid
                            .iter()
                            .enumerate()
                            .all(|(j, v)| v.is_none() || j == i || j == i + 3)
                    {
                        return Ok(Some((
                            Branch::Thinning,
                            self.create(2, part.class, part.size / 2)?,
                        )));
                    }
                }
            }
            if i % 3 < 2 {
                if let Some(part) = grid[i + 1].and_then(|v| self.micro(v)) {
                    let split = match part.class {
                        0 | 1 => Some(3),
                        3 => Some(2),
                        _ => None,
                    };
                    if let Some(class) = split {
                        if can
                            && grid
                                .iter()
                                .enumerate()
                                .all(|(j, v)| v.is_none() || j == i || j == i + 1)
                        {
                            return Ok(Some((
                                Branch::Splitting,
                                self.create(2, class, part.size)?,
                            )));
                        }
                    }
                }
            }
        }
        let occupied: Vec<_> = grid.iter().flatten().collect();
        if occupied.len() == 1 {
            if let Some(part) = self.micro(occupied[0]) {
                if part.class == 1 {
                    return Ok(Some((Branch::HollowFill, self.create(1, 0, part.size)?)));
                }
            }
        }
        Ok(None)
    }
}

/// Context-independent gates; complete lookup and native witness validation is
/// performed with the category's shared program by Domain::validate.
pub(super) fn validate(p: &Procedure, r: &Recipe, items: &BTreeMap<&str, &Item>) -> Result<()> {
    ensure!(
        r.source.handler == "com.rwtema.extrautils.nei.FMPMicroBlocksHandler"
            && p.program
                .strip_prefix("program_")
                .is_some_and(crate::source::is_digest)
            && p.material_id < 65536
            && p.layout.len() == 9
            && r.grid.is_none()
            && r.magic.is_none()
            && r.duration.is_none()
            && r.energy.is_none()
            && r.outputs.len() == 1,
        "Invalid FMP native procedure"
    );
    let slots: BTreeSet<_> = p.layout.iter().flatten().copied().collect();
    ensure!(
        slots.len() == p.layout.iter().flatten().count() && slots.len() == r.inputs.len(),
        "FMP example layout aliases slots"
    );
    for input in &r.inputs {
        ensure!(
            input.kind == Kind::Item && slots.contains(&input.slot) && !input.choices.is_empty(),
            "FMP input lacks physical cell"
        );
        for c in &input.choices {
            ensure!(
                items.contains_key(c.id.as_str()) && c.amount == "1" && c.returns.is_empty(),
                "FMP consumes one per occupied cell with process-owned saw remainder"
            );
        }
    }
    let out = &r.outputs[0];
    let change = out
        .change
        .as_ref()
        .context("FMP output lost its procedural result")?;
    ensure!(
        out.kind == Kind::Item
            && out.slot == 0
            && matches!(out.role, OutputRole::Result)
            && out.quantity.is_none()
            && out.chance.numerator == "1"
            && out.chance.denominator == "1"
            && matches!(change.action, Edit::FmpResult {})
            && r.inputs.iter().any(|i| i.slot == change.input),
        "Invalid FMP correlated result"
    );
    let bindings = change
        .bindings
        .as_ref()
        .context("FMP native result samples lack tuples")?;
    ensure!(
        !bindings.is_empty() && bindings.len() == change.samples.len() && bindings.len() <= 65536,
        "FMP omitted native tuple samples"
    );
    ensure!(
        change
            .samples
            .iter()
            .any(|s| s.id == out.id && Some(s.amount.as_str()) == out.amount.as_deref()),
        "FMP default result is not a native sample"
    );
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
        "FMP process and category programs differ"
    );
    rules.material(p.material_id)?;
    let mut projections = BTreeMap::new();
    for input in &r.inputs {
        for c in &input.choices {
            let item = items[c.id.as_str()];
            if let Some(saw) = rules.saws.iter().find(|s| s.registry == item.registry) {
                ensure!(
                    matches!(c.consume, Consumption::Wear)
                        && item.durability == saw.durability
                        && matches!(
                            c.rule,
                            Match::Wildcard {
                                meta: true,
                                nbt: true
                            }
                        ),
                    "FMP saw wear/matching changed"
                );
            } else if item.registry == rules.settings().micro_registry {
                let Match::FmpMaterial {
                    program,
                    material_id,
                    native_text,
                } = &c.rule
                else {
                    anyhow::bail!("FMP part lost its shared material predicate")
                };
                ensure!(
                    matches!(c.consume, Consumption::Consume)
                        && program == &p.program
                    && *material_id == p.material_id as i32
                        && rules.micro_id(item, native_text.as_deref())? == p.material_id as i32,
                    "FMP native material predicate rejects its example"
                );
                if let Some(prior) = projections.insert(item.id.as_str(), native_text.as_deref()) {
                    ensure!(
                        prior == native_text.as_deref(),
                        "FMP same item has inconsistent native mat text"
                    );
                }
            } else {
                ensure!(
                    matches!(c.consume, Consumption::Consume)
                        && matches!(c.rule, Match::Exact)
                        && rules.find_material(item, items)? == p.material_id as i32,
                    "FMP full block changed first native material precedence"
                );
            }
        }
    }
    let evaluator = Evaluator {
        process: p,
        rules,
        projections,
    };
    let change = r.outputs[0].change.as_ref().unwrap();
    for (tuple, sample) in change
        .bindings
        .as_ref()
        .unwrap()
        .iter()
        .zip(&change.samples)
    {
        ensure!(tuple.len() == r.inputs.len(), "FMP tuple arity changed");
        let mut grid = [None; 9];
        for (cell, slot) in p.layout.iter().enumerate() {
            if let Some(slot) = slot {
                let index = r
                    .inputs
                    .iter()
                    .position(|i| i.slot == *slot)
                    .context("FMP physical slot absent")?;
                let c = tuple[index]
                    .and_then(|c| r.inputs[index].choices.get(c as usize))
                    .context("FMP tuple lost occupied input")?;
                grid[cell] = Some(*items.get(c.id.as_str()).context("FMP tuple item absent")?);
            }
        }
        let (branch, result) = evaluator
            .result(&grid, items)?
            .context("Native FMP procedure rejects sample")?;
        ensure!(
            branch == p.branch
                && result.id == sample.id
                && result.amount == sample.amount
                && items.contains_key(sample.id.as_str()),
            "FMP sample differs from native procedural result"
        );
    }
    Ok(())
}
