//! LittleTiles' pinned CreativeCore matcher, independently of its NEI display samples.
use super::{Consumption, Item, Kind, Match, OutputRole, Recipe};
use crate::identity::Nbt;
use anyhow::{ensure, Context, Result};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CreativeCoreProcess {
    pub version: Version,
    pub shaped: bool,
    pub width: i32,
    pub height: Option<i32>,
    pub predicates: Vec<Option<Predicate>>,
    pub matching: Matching,
    pub consumption: Debit,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub enum Version {
    #[serde(rename = "1.5.14-GTNH")]
    Installed,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub enum Matching {
    #[serde(rename = "columnOffsetsUnmirroredThenMirror")]
    Shaped,
    #[serde(rename = "rowMajorFirstMatchRemoveFirstEqual")]
    Shapeless,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub enum Debit {
    #[serde(rename = "workbenchOnePerOccupiedCellThenNativeContainer")]
    Native,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Selector {
    pub registry: String,
    pub code: u16,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "lowercase", deny_unknown_fields)]
#[schemars(rename = "CreativeCorePredicate")]
pub enum Predicate {
    Item {
        minimum: i32,
        registry: String,
    },
    Itemstack {
        minimum: i32,
        registry: String,
        metadata: i32,
        #[serde(rename = "needNBT")]
        need_nbt: bool,
        nbt: Option<Nbt>,
    },
    Ore {
        minimum: i32,
        name: String,
        selectors: Vec<Selector>,
    },
    Block {
        minimum: i32,
        identity: Vec<String>,
        members: Vec<String>,
    },
    Material {
        minimum: i32,
        identity: Vec<String>,
        members: Vec<String>,
    },
}
impl Predicate {
    pub fn minimum(&self) -> i32 {
        match self {
            Self::Item { minimum, .. }
            | Self::Itemstack { minimum, .. }
            | Self::Ore { minimum, .. }
            | Self::Block { minimum, .. }
            | Self::Material { minimum, .. } => *minimum,
        }
    }
    pub fn accepts(&self, item: &Item, count: i32) -> bool {
        if count < self.minimum() {
            return false;
        }
        match self {
            Self::Item { registry, .. } => item.registry == *registry,
            Self::Itemstack {
                registry,
                metadata,
                need_nbt,
                nbt,
                ..
            } => {
                item.registry == *registry
                    && item.meta == *metadata
                    && (!need_nbt || nbt_equal(item.nbt.as_ref(), nbt.as_ref()))
            }
            Self::Ore { selectors, .. } => selectors.iter().any(|s| {
                s.registry == item.registry
                    && (s.code == 0 || s.code == item.meta.wrapping_add(1) as u16)
            }),
            Self::Block { members, .. } | Self::Material { members, .. } => {
                members.binary_search(&item.registry).is_ok()
            }
        }
    }
    /// ArrayList.remove(Object) calls the selected object's equality on each earlier entry.
    pub fn native_equals(&self, other: &Self) -> bool {
        if self.minimum() != other.minimum() {
            return false;
        }
        match (self, other) {
            (Self::Item { registry: a, .. }, Self::Item { registry: b, .. }) => a == b,
            (
                Self::Itemstack {
                    registry: a,
                    metadata: am,
                    nbt: an,
                    ..
                },
                Self::Itemstack {
                    registry: b,
                    metadata: bm,
                    nbt: bn,
                    need_nbt,
                    ..
                },
            ) => a == b && am == bm && (!need_nbt || nbt_equal(an.as_ref(), bn.as_ref())),
            (Self::Ore { name: a, .. }, Self::Ore { name: b, .. }) => a == b,
            (Self::Block { identity: a, .. }, Self::Block { identity: b, .. })
            | (Self::Material { identity: a, .. }, Self::Material { identity: b, .. }) => a == b,
            _ => false,
        }
    }
}
fn nbt_equal(a: Option<&Nbt>, b: Option<&Nbt>) -> bool {
    match (a, b) {
        (None, None) => true,
        (Some(a), Some(b)) => super::matching::native_tag(a, b, true),
        _ => false,
    }
}
fn registry(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 4096
        && value.contains(':')
        && !value.chars().any(char::is_control)
}
fn names(values: &[String]) -> bool {
    values.len() <= 65536
        && values.iter().all(|s| registry(s))
        && values.windows(2).all(|p| p[0] < p[1])
}
fn active(p: &CreativeCoreProcess) -> Result<usize> {
    ensure!(
        p.predicates.len() <= 65536,
        "CreativeCore predicate budget exceeded"
    );
    if p.shaped {
        let h = p.height.context("CreativeCore shaped height missing")?;
        ensure!(
            (0..=3).contains(&p.width) && (0..=3).contains(&h) && p.matching == Matching::Shaped,
            "CreativeCore native shaped dimensions/order changed"
        );
        let count = (p.width * h) as usize;
        ensure!(
            count <= p.predicates.len(),
            "CreativeCore native array bounds lost"
        );
        Ok(count)
    } else {
        ensure!(
            p.height.is_none()
                && p.predicates.len() <= 9
                && p.predicates.iter().all(Option::is_some)
                && p.matching == Matching::Shapeless,
            "CreativeCore native shapeless list/order changed"
        );
        Ok(p.predicates.len())
    }
}
pub(super) fn validate(
    p: &CreativeCoreProcess,
    r: &Recipe,
    items: &BTreeMap<&str, &Item>,
) -> Result<()> {
    let count = active(p)?;
    ensure!(
        r.source.owner == "creativecore"
            && r.source.handler == "com.creativemd.creativecore.api.nei.NEIRecipeInfoHandler"
            && r.duration.is_none()
            && r.energy.is_none()
            && r.magic.is_none()
            && r.properties.is_empty()
            && r.notes.len() == 1
            && r.notes[0].key == "creativecore:matching"
            && r.notes[0].source
                == if p.shaped {
                    "com.creativemd.creativecore.common.recipe.BetterShapedRecipe.matches"
                } else {
                    "com.creativemd.creativecore.common.recipe.entry.BetterShapelessRecipe.matches"
                },
        "CreativeCore source or native semantics missing"
    );
    for predicate in p.predicates.iter().flatten() {
        match predicate {
            Predicate::Item { registry: reg, .. } => {
                ensure!(registry(reg), "CreativeCore item identity invalid")
            }
            Predicate::Itemstack {
                registry: reg,
                metadata,
                nbt,
                ..
            } => ensure!(
                registry(reg)
                    && *metadata >= 0
                    && nbt
                        .as_ref()
                        .is_none_or(|n| super::matching::native_tag(n, n, true)),
                "CreativeCore literal metadata or identity-sensitive NBT invalid"
            ),
            Predicate::Ore {
                name, selectors, ..
            } => {
                ensure!(
                    name.len() <= 4096 && selectors.len() <= 65536,
                    "CreativeCore ore program budget exceeded"
                );
                let mut unique = BTreeSet::new();
                for selector in selectors {
                    ensure!(
                        registry(&selector.registry)
                            && unique.insert((&selector.registry, selector.code)),
                        "CreativeCore duplicate/invalid baked selector"
                    );
                }
            }
            Predicate::Block {
                identity, members, ..
            }
            | Predicate::Material {
                identity, members, ..
            } => ensure!(
                names(identity) && names(members),
                "CreativeCore block/material census invalid"
            ),
        }
    }
    let expected: Vec<_> = p.predicates[..count]
        .iter()
        .enumerate()
        .filter_map(|(i, v)| v.as_ref().map(|p| (i, p)))
        .collect();
    ensure!(
        r.inputs.len() == expected.len() && r.outputs.len() == 1,
        "CreativeCore input/output census changed"
    );
    let cells: Vec<_> = p.predicates[..count]
        .iter()
        .scan(0u32, |slot, v| {
            Some(v.as_ref().map(|_| {
                let index = *slot;
                *slot += 1;
                index
            }))
        })
        .collect();
    if p.shaped && p.width > 0 && p.height.unwrap() > 0 {
        let grid = r
            .grid
            .as_ref()
            .context("CreativeCore logical grid missing")?;
        ensure!(
            grid.width == p.width as u32
                && grid.height == p.height.unwrap() as u32
                && grid.mirror
                && grid.cells == cells,
            "CreativeCore grid lost holes, mirror or unused-tail boundary"
        );
    } else {
        ensure!(r.grid.is_none(), "CreativeCore acquired a GUI-derived grid");
    }
    for (slot, (raw, predicate)) in expected.into_iter().enumerate() {
        let input = &r.inputs[slot];
        ensure!(
            input.kind == Kind::Item && input.slot == slot as u32,
            "CreativeCore compact slots changed"
        );
        let mut seen = BTreeSet::new();
        for choice in &input.choices {
            let item = items
                .get(choice.id.as_str())
                .context("CreativeCore witness missing")?;
            ensure!(
                choice.amount == "1"
                    && matches!(choice.consume, Consumption::Consume)
                    && matches!(choice.rule,Match::CreativeCore{input} if input==raw as u32)
                    && predicate.accepts(item, predicate.minimum().max(1))
                    && seen.insert(&choice.id),
                "CreativeCore sample, stock/debit separation or typed predicate changed"
            );
            match predicate {
                Predicate::Item { .. } | Predicate::Block { .. } | Predicate::Material { .. } => {
                    ensure!(
                        item.meta == 0 && item.nbt.is_none(),
                        "CreativeCore broad witness is not canonical"
                    )
                }
                Predicate::Itemstack { nbt, need_nbt, .. } => ensure!(
                    item.nbt == if *need_nbt { nbt.clone() } else { None },
                    "CreativeCore stack witness altered its NBT policy"
                ),
                Predicate::Ore { selectors, .. } => ensure!(
                    item.nbt.is_none()
                        && selectors.iter().any(|s| s.registry == item.registry
                            && item.meta
                                == if s.code == 0 {
                                    0
                                } else {
                                    i32::from(s.code) - 1
                                }),
                    "CreativeCore ore witness is not a canonical baked example"
                ),
            }
        }
        let expected_count = match predicate {
            Predicate::Item { .. } | Predicate::Itemstack { .. } => 1,
            Predicate::Block { members, .. } | Predicate::Material { members, .. } => members.len(),
            Predicate::Ore { selectors, .. } => selectors
                .iter()
                .map(|s| {
                    (
                        &s.registry,
                        if s.code == 0 {
                            0
                        } else {
                            i32::from(s.code) - 1
                        },
                    )
                })
                .collect::<BTreeSet<_>>()
                .len(),
        };
        ensure!(
            input.choices.len() == expected_count && expected_count > 0,
            "CreativeCore candidate census is incomplete"
        );
    }
    let output = &r.outputs[0];
    ensure!(
        output.kind == Kind::Item
            && output.slot == 0
            && matches!(output.role, OutputRole::Result)
            && output.amount.is_some()
            && output.change.is_none()
            && output.quantity.is_none()
            && output.chance.numerator == "1"
            && output.chance.denominator == "1",
        "CreativeCore result is not its fixed native copy"
    );
    Ok(())
}

/// Returns removed native predicate indices by physical cell, or no matching recipe.
pub fn select(
    p: &CreativeCoreProcess,
    offered: [Option<(&Item, i32)>; 9],
) -> Result<Option<[Option<u32>; 9]>> {
    let count = active(p)?;
    if !p.shaped {
        let mut left: Vec<_> = (0..count).collect();
        let mut removed = [None; 9];
        for (cell, stack) in offered.iter().enumerate() {
            if let Some((item, amount)) = stack {
                let Some(selected) = left
                    .iter()
                    .copied()
                    .find(|i| p.predicates[*i].as_ref().unwrap().accepts(item, *amount))
                else {
                    return Ok(None);
                };
                let selected = p.predicates[selected].as_ref().unwrap();
                let at = left
                    .iter()
                    .position(|i| selected.native_equals(p.predicates[*i].as_ref().unwrap()))
                    .context("CreativeCore nonreflexive native equality")?;
                removed[cell] = Some(left.remove(at) as u32);
            }
        }
        return Ok(left.is_empty().then_some(removed));
    }
    for x in 0..=3 - p.width {
        for y in 0..=3 - p.height.unwrap() {
            for mirror in [false, true] {
                let mut selected = [None; 9];
                let mut valid = true;
                for column in 0..3 {
                    for row in 0..3 {
                        let sx = column - x;
                        let sy = row - y;
                        let cell = (column + 3 * row) as usize;
                        let index = (sx >= 0 && sy >= 0 && sx < p.width && sy < p.height.unwrap())
                            .then(|| {
                                ((if mirror { p.width - sx - 1 } else { sx }) + sy * p.width)
                                    as usize
                            });
                        let predicate = index.and_then(|i| p.predicates[i].as_ref());
                        match (predicate, offered[cell]) {
                            (None, None) => {}
                            (Some(p), Some((item, amount))) if p.accepts(item, amount) => {
                                selected[cell] = index.map(|i| i as u32)
                            }
                            _ => valid = false,
                        }
                    }
                }
                if valid {
                    return Ok(Some(selected));
                }
            }
        }
    }
    Ok(None)
}
