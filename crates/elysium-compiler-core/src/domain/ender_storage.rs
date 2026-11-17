//! An owned integrated-server snapshot; no refresh, inventory mutation or consuming recipe.
use super::{
    nei_diagrams::Component,
    nei_information::{required_nullable, RequiredNullable},
    references::{GeneticReferenceRow, ReferenceData},
    Fluid, Item, Origin,
};
use crate::identity::Nbt;
use anyhow::{ensure, Context, Result};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(rename = "EnderStorageType")]
pub enum StorageType {
    Item,
    Liquid,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(rename = "EnderStorageSnapshotScope")]
pub enum SnapshotScope {
    IntegratedServerGlobalAndCurrentPersonal,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(rename = "EnderStorageConsistency")]
pub enum Consistency {
    OwnedAtServerTick,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(rename = "EnderStorageAbsentFrequency")]
pub enum AbsentFrequency {
    UnallocatedNativeEmpty,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(rename = "EnderStorageLayer")]
pub enum Layer {
    LoadedServer,
    PersistedServer,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(rename = "EnderStorageOwnerScope")]
pub enum OwnerScope {
    Global,
    Personal,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[schemars(rename = "EnderStorageCounts")]
pub struct Counts {
    pub states: u32,
    pub loaded: u32,
    pub persisted: u32,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[schemars(rename = "EnderStorageQueryRule")]
pub struct QueryRule {
    pub role: String,
    pub registry: String,
    #[serde(deserialize_with = "required_nullable")]
    #[schemars(with = "RequiredNullable<u32>")]
    pub high_metadata: Option<u32>,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[schemars(rename = "EnderStorageAction")]
pub struct Action {
    pub key: String,
    pub owner: OwnerScope,
    pub requires_new_snapshot: bool,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[schemars(rename = "EnderStorageDefinition")]
pub struct Definition {
    pub source: Origin,
    pub group: String,
    pub name: String,
    pub description: String,
    pub storage_type: StorageType,
    pub personal_owner: String,
    pub captured_tick: String,
    pub scope: SnapshotScope,
    pub consistency: Consistency,
    pub absent_frequency: AbsentFrequency,
    pub ready: bool,
    pub configured_size: u32,
    pub sizes: Vec<u32>,
    pub tank_capacity: u32,
    pub query_rules: Vec<QueryRule>,
    pub personal_lock_item: Component,
    pub ignore_nbt: bool,
    pub actions: Vec<Action>,
    pub counts: Counts,
    pub consuming: bool,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[schemars(rename = "EnderStorageSlot")]
pub struct Slot {
    pub component: Component,
    pub amount: String,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "camelCase", deny_unknown_fields)]
#[schemars(rename = "EnderStorageContents")]
pub enum Contents {
    Item {
        #[serde(rename = "sizeIndex")]
        size_index: u32,
        slots: Vec<Option<Slot>>,
        #[serde(rename = "nonEmpty")]
        non_empty: bool,
    },
    Liquid {
        capacity: u32,
        #[serde(deserialize_with = "required_nullable")]
        #[schemars(with = "RequiredNullable<Component>")]
        fluid: Option<Component>,
        amount: String,
        #[serde(rename = "nonEmpty")]
        non_empty: bool,
    },
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[schemars(rename = "EnderStorageState")]
pub struct State {
    pub source: Origin,
    pub group: String,
    pub state: u32,
    pub owner: String,
    pub frequency: u32,
    pub colours: Vec<u32>,
    pub layer: Layer,
    #[serde(deserialize_with = "required_nullable")]
    #[schemars(with = "RequiredNullable<Nbt>")]
    pub raw: Option<Nbt>,
    pub contents: Contents,
    pub consuming: bool,
}

fn amount(value: &str) -> Result<u32> {
    let number = value.parse::<u32>()?;
    ensure!(
        number <= i32::MAX as u32 && number.to_string() == value,
        "Invalid native storage quantity"
    );
    Ok(number)
}
fn compound(tag: &Nbt) -> Result<&BTreeMap<String, Nbt>> {
    if let Nbt::Compound { value } = tag {
        Ok(value)
    } else {
        anyhow::bail!("Native stored tag is not a compound")
    }
}
fn int(tag: &Nbt) -> Result<i32> {
    let floor = |number: f64| {
        let truncated = number as i32;
        if number < (truncated as f64) {
            truncated.wrapping_sub(1)
        } else {
            truncated
        }
    };
    Ok(match tag {
        Nbt::Byte { value } => value.parse::<i8>()? as i32,
        Nbt::Short { value } => value.parse::<i16>()? as i32,
        Nbt::Int { value } => value.parse()?,
        Nbt::Long { value } => value.parse::<i64>()? as i32,
        Nbt::Float { value } => floor(f32::from_bits(u32::from_str_radix(value, 16)?) as f64),
        Nbt::Double { value } => floor(f64::from_bits(u64::from_str_radix(value, 16)?)),
        _ => anyhow::bail!("Native quantity is not numeric"),
    })
}
fn component_parts(component: &Component) -> Result<(&str, Option<i32>, &Option<Nbt>)> {
    Ok(match component {
        Component::Item {
            registry,
            meta,
            nbt,
            ..
        } => (registry, Some(*meta), nbt),
        Component::Fluid { registry, nbt, .. } => (registry, None, nbt),
        Component::ItemFact { .. } => anyhow::bail!("EnderStorage requires literal native contents"),
    })
}
fn persisted(state: &State) -> Result<()> {
    let Some(raw) = &state.raw else {
        ensure!(
            state.layer == Layer::LoadedServer,
            "Persisted state lost its raw tag"
        );
        return Ok(());
    };
    ensure!(
        state.layer == Layer::PersistedServer,
        "Loaded contents pretend to have a persisted source tag"
    );
    raw.validate()?;
    let tag = compound(raw)?;
    match &state.contents {
        Contents::Item {
            size_index, slots, ..
        } => {
            let Some(Nbt::Byte { value }) = tag.get("size") else {
                anyhow::bail!("Native persisted size type differs")
            };
            ensure!(
                value.parse::<u32>()? == *size_index,
                "Persisted size differs"
            );
            let Some(Nbt::List { element, value }) = tag.get("Items") else {
                anyhow::bail!("Native persisted item list absent")
            };
            ensure!(
                value.is_empty() || element == "compound",
                "Persisted item list type differs"
            );
            let mut seen = BTreeSet::new();
            for entry in value {
                let entry = compound(entry)?;
                let Some(Nbt::Short { value }) = entry.get("Slot") else {
                    anyhow::bail!("Persisted slot type differs")
                };
                let index = value.parse::<usize>()?;
                ensure!(seen.insert(index), "Persisted slot repeated");
                let slot = slots
                    .get(index)
                    .and_then(Option::as_ref)
                    .context("Persisted slot disappeared/outside size")?;
                let raw_amount = if let Some(quantity) = entry.get("Quantity") {
                    int(quantity)?
                } else {
                    let Some(Nbt::Byte { value }) = entry.get("Count") else {
                        anyhow::bail!("Persisted Count type differs")
                    };
                    value.parse::<i8>()? as i32
                };
                ensure!(
                    raw_amount >= 0 && amount(&slot.amount)? == raw_amount as u32,
                    "Persisted quantity differs"
                );
                let meta = if let Some(damage) = entry.get("Damage") {
                    let Nbt::Short { value } = damage else {
                        anyhow::bail!("Persisted Damage type differs")
                    };
                    (value.parse::<i16>()? as i32).max(0)
                } else {
                    0
                };
                let (_, actual_meta, nbt) = component_parts(&slot.component)?;
                let expected_nbt = entry
                    .get("tag")
                    .filter(|tag| matches!(tag, Nbt::Compound { .. }));
                ensure!(
                    actual_meta == Some(meta) && nbt.as_ref() == expected_nbt,
                    "Persisted metadata/NBT differs"
                );
            }
            ensure!(
                slots.iter().filter(|slot| slot.is_some()).count() == seen.len(),
                "Invented persisted inventory slot"
            );
        }
        Contents::Liquid {
            fluid,
            amount: quantity,
            ..
        } => {
            let tank = compound(tag.get("tank").context("Persisted tank absent")?)?;
            if tank.is_empty() {
                let component = fluid
                    .as_ref()
                    .context("Native empty tank lost water-zero sentinel")?;
                let (registry, meta, nbt) = component_parts(component)?;
                ensure!(
                    registry == "water"
                        && meta.is_none()
                        && nbt.is_none()
                        && amount(quantity)? == 0,
                    "Native empty tank sentinel differs"
                );
            } else {
                let Some(Nbt::String { value: name }) = tank.get("FluidName") else {
                    anyhow::bail!("Persisted FluidName absent")
                };
                let component = fluid.as_ref().context("Persisted fluid disappeared")?;
                let (registry, meta, nbt) = component_parts(component)?;
                let expected_nbt = tank
                    .get("Tag")
                    .filter(|tag| matches!(tag, Nbt::Compound { .. }));
                let raw_amount = int(tank.get("Amount").context("Persisted Amount absent")?)?;
                ensure!(
                    registry == name
                        && meta.is_none()
                        && nbt.as_ref() == expected_nbt
                        && raw_amount >= 0
                        && amount(quantity)? == raw_amount as u32,
                    "Persisted fluid identity/amount/NBT differs"
                );
            }
        }
    }
    Ok(())
}
pub fn validate(
    id: &str,
    rows: &[&GeneticReferenceRow],
    items: &BTreeMap<&str, &Item>,
) -> Result<()> {
    let ReferenceData::EnderStorageDefinition(def) = &rows[0].data else {
        anyhow::bail!("EnderStorage definition absent")
    };
    let suffix = if def.storage_type == StorageType::Item {
        "chestoverview"
    } else {
        "tankoverview"
    };
    ensure!(
        def.group == format!("neicustomdiagram.diagramgroup.enderstorage.{suffix}")
            && def.source.key == def.group
            && def.source.handler
                == "com.github.dcysteine.neicustomdiagram.api.diagram.CustomDiagramGroup"
            && def.source.owner == "neicustomdiagram"
            && def.ready
            && !def.consuming
            && def.ignore_nbt,
        "Native EnderStorage source/readiness differs"
    );
    ensure!(
        !def.personal_owner.is_empty()
            && [&def.personal_owner, &def.name, &def.description]
                .iter()
                .all(|text| text.len() <= 65536)
            && def.sizes == [9, 27, 54]
            && def.configured_size < 3
            && def.tank_capacity > 0
            && def.tank_capacity <= i32::MAX as u32,
        "Native storage context/configuration invalid"
    );
    let tick = def.captured_tick.parse::<i64>()?;
    ensure!(
        tick >= 0 && tick.to_string() == def.captured_tick,
        "Invalid captured tick"
    );
    ensure!(
        def.counts.states <= 8192
            && def.counts.loaded.checked_add(def.counts.persisted) == Some(def.counts.states)
            && rows.len() == 1 + def.counts.states as usize,
        "Native state count closure differs"
    );
    let values = rows.iter().map(|r| &r.data).collect::<Vec<_>>();
    ensure!(
        id == super::value_id(
            "reference",
            &serde_json::json!({"kind":"enderStorageSnapshot","records":values})
        )? && serde_json::to_vec(&values)?.len() <= 32 * 1024 * 1024,
        "Snapshot identity/byte budget differs"
    );
    let rules = &def.query_rules;
    ensure!(
        rules.len()
            == if def.storage_type == StorageType::Item {
                2
            } else {
                1
            },
        "Native storage query rules incomplete"
    );
    ensure!(
        rules[0].role == "block"
            && !rules[0].registry.is_empty()
            && rules[0].high_metadata
                == Some(if def.storage_type == StorageType::Item {
                    0
                } else {
                    1
                }),
        "Block metadata query rule differs"
    );
    if let Some(rule) = rules.get(1) {
        ensure!(
            rule.role == "pouch" && !rule.registry.is_empty() && rule.high_metadata.is_none(),
            "Pouch query rule differs"
        );
    }
    ensure!(def.actions.len() == 2, "Pinned native storage actions incomplete");
    for (index, action) in def.actions.iter().enumerate() {
        let owner = if index == 0 {
            OwnerScope::Global
        } else {
            OwnerScope::Personal
        };
        let suffix = if owner == OwnerScope::Global {
            "global"
        } else {
            "personal"
        };
        ensure!(
            action.owner == owner
                && !action.requires_new_snapshot
                && action.key == format!("{}-{}", def.group, suffix),
            "Native storage action differs"
        );
    }
    ensure!(
        matches!(def.personal_lock_item, Component::Item { meta: 0, .. }),
        "Personal lock item differs"
    );
    def.personal_lock_item.validate(items)?;
    let mut prior = None;
    let mut loaded = 0;
    let mut saved = 0;
    for (index, row) in rows[1..].iter().enumerate() {
        let ReferenceData::EnderStorageState(state) = &row.data else {
            anyhow::bail!("Mixed storage snapshot")
        };
        ensure!(
            state.state as usize == index
                && !state.consuming
                && state.group == def.group
                && serde_json::to_value(&state.source)? == serde_json::to_value(&def.source)?
                && state.frequency <= 4095
                && (state.owner == "global" || state.owner == def.personal_owner),
            "Storage state origin/frequency/owner differs"
        );
        let key = (if state.owner == "global" { 0 } else { 1 }, state.frequency);
        ensure!(
            prior.is_none_or(|old| old < key),
            "Native state order/uniqueness differs"
        );
        prior = Some(key);
        let f = state.frequency;
        let colours = if def.storage_type == StorageType::Item {
            vec![f >> 8 & 15, f >> 4 & 15, f & 15]
        } else {
            vec![f & 15, f >> 4 & 15, f >> 8 & 15]
        };
        ensure!(
            state.colours == colours,
            "Native frequency colour order differs"
        );
        match &state.contents {
            Contents::Item {
                size_index,
                slots,
                non_empty,
            } => {
                ensure!(
                    def.storage_type == StorageType::Item
                        && *size_index < 3
                        && slots.len() == def.sizes[*size_index as usize] as usize
                        && *non_empty == slots.iter().any(Option::is_some),
                    "Native inventory size/empty closure differs"
                );
                for slot in slots.iter().flatten() {
                    ensure!(
                        matches!(slot.component, Component::Item { .. }),
                        "Chest contains a fluid component"
                    );
                    amount(&slot.amount)?;
                    slot.component.validate(items)?;
                }
            }
            Contents::Liquid {
                capacity,
                fluid,
                amount: quantity,
                non_empty,
            } => {
                let quantity = amount(quantity)?;
                ensure!(
                    def.storage_type == StorageType::Liquid
                        && *capacity == def.tank_capacity
                        && quantity <= *capacity
                        && *non_empty == (quantity != 0)
                        && (fluid.is_some() || quantity == 0),
                    "Native tank capacity/empty closure differs"
                );
                if let Some(fluid) = fluid {
                    ensure!(
                        matches!(fluid, Component::Fluid { .. }),
                        "Tank contains an item component"
                    );
                }
            }
        }
        persisted(state)?;
        if state.layer == Layer::LoadedServer {
            loaded += 1;
        } else {
            saved += 1;
        }
    }
    ensure!(
        loaded == def.counts.loaded && saved == def.counts.persisted,
        "Storage layer counts differ"
    );
    Ok(())
}
pub fn validate_facts(rows: &[GeneticReferenceRow], fluids: &BTreeMap<&str, &Fluid>) -> Result<()> {
    for row in rows {
        if let ReferenceData::EnderStorageState(State {
            contents: Contents::Liquid {
                fluid: Some(fluid), ..
            },
            ..
        }) = &row.data
        {
            fluid.fluid(fluids)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::super::origin_id;
    use super::super::Domain;
    use super::*;
    use serde_json::{json, Value};
    #[test]
    fn native_enderstorage_preserves_raw_state_and_rejects_semantic_tampering() {
        let raw: Value = serde_json::from_str(include_str!(
            "../../../../contracts/fixtures/enderstorage-contract.json"
        ))
        .unwrap();
        let source = crate::source::Source::open(
            &std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../contracts/fixtures/source-semantic-40"),
        )
        .unwrap();
        let mut domain = Domain::load(&source).unwrap();
        let base = serde_json::to_value(&domain.items[0]).unwrap();
        for item in raw["items"].as_array().unwrap() {
            let mut value = domain
                .items
                .iter()
                .find(|old| old.id == item["id"].as_str().unwrap())
                .map(|item| serde_json::to_value(item).unwrap())
                .unwrap_or_else(|| base.clone());
            for key in ["id", "registry", "meta", "nbt"] {
                value[key] = item[key].clone();
            }
            let item: Item = serde_json::from_value(value).unwrap();
            domain.items.retain(|old| old.id != item.id);
            domain.items.push(item);
        }
        let base = serde_json::to_value(&domain.fluids[0]).unwrap();
        for fluid in raw["fluids"].as_array().unwrap() {
            let mut value = base.clone();
            for key in [
                "id",
                "registry",
                "nbt",
                "density",
                "temperature",
                "viscosity",
                "luminosity",
                "gaseous",
            ] {
                value[key] = fluid[key].clone();
            }
            let fluid: Fluid = serde_json::from_value(value).unwrap();
            domain.fluids.retain(|old| old.id != fluid.id);
            domain.fluids.push(fluid);
        }
        let groups: Vec<Vec<GeneticReferenceRow>> =
            serde_json::from_value(raw["domains"].clone()).unwrap();
        domain.references = groups.iter().flatten().cloned().collect();
        for group in &groups {
            let ReferenceData::EnderStorageDefinition(def) = &group[0].data else {
                panic!()
            };
            let mut category = domain.categories[0].clone();
            category.source = def.source.clone();
            category.id = origin_id("category", &category.source).unwrap();
            category.reference = Some(group[0].domain.clone());
            category.program = None;
            category.icon = None;
            category.machines.clear();
            domain.categories.push(category);
        }
        domain.validate(&source).unwrap();
        let empty:Value=serde_json::from_str(include_str!("../../../../contracts/fixtures/enderstorage-empty-contract.json")).unwrap();
        let empty_rows:Vec<GeneticReferenceRow>=serde_json::from_value(empty["records"].clone()).unwrap();
        let item_map=domain.items.iter().map(|item|(item.id.as_str(),item)).collect();
        super::super::references::validate(&empty_rows,&item_map).unwrap();
        let reject = |mut changed: Vec<GeneticReferenceRow>| {
            let values = changed.iter().map(|row| &row.data).collect::<Vec<_>>();
            let id = super::super::value_id(
                "reference",
                &json!({"kind":"enderStorageSnapshot","records":values}),
            )
            .unwrap();
            for row in &mut changed {
                row.domain = id.clone();
                row.id = row.content_id().unwrap();
            }
            let mut bad = domain.clone();
            bad.references
                .retain(|row| row.domain != groups[0][0].domain);
            bad.references.extend(changed);
            bad.categories
                .iter_mut()
                .find(|category| category.reference.as_deref() == Some(&groups[0][0].domain))
                .unwrap()
                .reference = Some(id);
            assert!(bad.validate(&source).is_err());
        };
        let mut bad = groups[0].clone();
        if let ReferenceData::EnderStorageState(state) = &mut bad[2].data {
            if let Contents::Item { slots, .. } = &mut state.contents {
                slots[8].as_mut().unwrap().amount = "6".into();
            }
        }
        reject(bad);
        let mut bad = groups[0].clone();
        if let ReferenceData::EnderStorageState(state) = &mut bad[1].data {
            state.colours.reverse();
        }
        reject(bad);
        let mut bad = groups[0].clone();
        if let ReferenceData::EnderStorageState(state) = &mut bad[2].data {
            state.owner = "Other".into();
        }
        reject(bad);
        let mut bad = groups[0].clone();
        if let ReferenceData::EnderStorageState(state) = &mut bad[2].data {
            state.raw = None;
        }
        reject(bad);
        let mut bad = groups[0].clone();
        if let ReferenceData::EnderStorageDefinition(def) = &mut bad[0].data {
            def.ready = false;
        }
        reject(bad);
        let mut missing = domain.clone();
        missing.fluids.clear();
        assert!(missing.validate(&source).is_err());
    }
}
