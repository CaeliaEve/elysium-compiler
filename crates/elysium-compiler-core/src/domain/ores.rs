use super::{check, origin_id, Domain, Origin};
use crate::identity::{integer, item_id, Nbt};
use anyhow::{ensure, Context, Result};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct OreGroup {
    pub id: String,
    pub source: Origin,
    /// Exact native ore dictionary name, not a translated item label.
    pub name: String,
    /// Native Java string ordering of all registered non-null names.
    pub order: u32,
    /// Number of original registration-list positions, including duplicates.
    pub members: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct OreTemplate {
    pub registry: String,
    /// Raw stored metadata; 32767 remains a wildcard template, never a display stack.
    pub meta: i32,
    pub nbt: Option<Nbt>,
    /// Signed native stackSize. Registry metadata, not a consumption quantity.
    pub amount: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct OreMember {
    pub id: String,
    pub group: String,
    pub index: u32,
    pub template: OreTemplate,
    /// A verified concrete NEI example; null when none is known safe to display.
    pub display: Option<String>,
}

pub(super) fn validate(domain: &Domain) -> Result<()> {
    ensure!(
        domain.ore_groups.len() <= 65536,
        "ore group count exceeds its budget"
    );
    let groups: BTreeMap<_, _> = domain
        .ore_groups
        .iter()
        .map(|row| (row.id.as_str(), row))
        .collect();
    ensure!(
        groups.len() == domain.ore_groups.len(),
        "duplicate ore group identity"
    );
    let items: BTreeMap<_, _> = domain
        .items
        .iter()
        .map(|row| (row.id.as_str(), row))
        .collect();
    let mut names = BTreeSet::new();
    let mut ordered: Vec<_> = domain.ore_groups.iter().collect();
    ordered.sort_by(|a, b| a.name.encode_utf16().cmp(b.name.encode_utf16()));
    for (index, group) in ordered.iter().enumerate() {
        check::origin(&group.source)?;
        ensure!(
            group.source.owner == "Forge"
                && group.source.handler == "net.minecraftforge.oredict.OreDictionary"
                && group.source.key == group.name
                && !group.name.is_empty()
                && group.name.len() <= 256,
            "invalid native ore group origin/name"
        );
        ensure!(
            group.id == origin_id("oregroup", &group.source)?,
            "ore group identity mismatch"
        );
        ensure!(
            names.insert(&group.name) && group.order as usize == index,
            "ore group names/order are not unique and sorted"
        );
        ensure!(
            group.members <= 1_000_000,
            "ore group registration count exceeds its budget"
        );
    }
    let mut positions: BTreeMap<&str, BTreeSet<u32>> = BTreeMap::new();
    for row in &domain.ore_members {
        let group = groups
            .get(row.group.as_str())
            .context("ore member has no group")?;
        ensure!(
            row.index < group.members
                && row.id == format!("{}.member_{:08x}", row.group, row.index),
            "ore member position/identity mismatch"
        );
        ensure!(
            positions
                .entry(row.group.as_str())
                .or_default()
                .insert(row.index),
            "duplicate ore member position"
        );
        item_id(
            &row.template.registry,
            row.template.meta,
            row.template.nbt.as_ref(),
        )?;
        integer(&row.template.amount, i32::MIN as i64, i32::MAX as i64)?;
        if let Some(display) = &row.display {
            let item = items
                .get(display.as_str())
                .context("ore display item is missing")?;
            ensure!(
                item.registry == row.template.registry && item.meta != 32767,
                "ore display is not a concrete example of its registry item"
            );
            ensure!(
                row.template.meta == 32767
                    || (item.meta == row.template.meta
                        && serde_json::to_value(&item.nbt)?
                            == serde_json::to_value(&row.template.nbt)?),
                "ore display does not represent the exact template"
            );
        }
    }
    for group in &domain.ore_groups {
        ensure!(
            positions.get(group.id.as_str()).map_or(0, BTreeSet::len) == group.members as usize,
            "ore group registration count is incomplete"
        );
    }
    Ok(())
}
