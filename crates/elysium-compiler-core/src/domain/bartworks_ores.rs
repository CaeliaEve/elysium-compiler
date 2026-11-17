//! Pinned Bartworks world-generation rules, native ore variants and metadata lookup.
use super::{
    references::{GeneticReferenceRow, ReferenceData},
    Item, Origin,
};
use anyhow::{ensure, Context, Result};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

macro_rules! record {
    ($name:ident,$schema:literal,{$($field:ident:$ty:ty),*$(,)?}) => {
        #[derive(Clone,Debug,Serialize,Deserialize,JsonSchema)]
        #[serde(rename_all="camelCase",deny_unknown_fields)]
        #[schemars(rename=$schema)]
        pub struct $name {$(pub $field:$ty),*}
    };
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(rename = "BartworksOreOverlayPolicy")]
pub enum OverlayPolicy {
    OreFeatureForBothVariantsThenLayerOnlyDedup,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(rename = "BartworksOreLayerDiscovery")]
pub enum LayerDiscovery {
    ActiveListThenMetadataLookup,
}
record!(Counts,"BartworksOreCounts",{layers:u32,materials:u32,lookups:u32});
record!(Definition,"BartworksOresDefinition",{source:Origin,ready:bool,consuming:bool,native_overlay_policy:OverlayPolicy,layer_discovery:LayerDiscovery,enabled_weight:i32,registered_materials:u32,counts:Counts});
record!(Dimension,"BartworksOreDimension",{key:String,name:String,id:i32,provider:String});
record!(Replacement,"BartworksOreReplacement",{registry:Option<String>,metas:Vec<i32>});
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(rename = "BartworksOreRoleKind")]
pub enum RoleKind {
    Primary,
    Secondary,
    Between,
    Sporadic,
}
record!(Role,"BartworksOreRole",{role:RoleKind,meta:i32,bartworks:bool,item:String});
record!(Layer,"BartworksOreLayer",{key:String,ordinal:u32,active_ordinal:Option<u32>,generator_class:String,enabled:bool,min_y:i32,max_y:i32,weight:i32,density:i32,size:i32,bw_mask:u8,dimension:Dimension,replacement:Replacement,roles:Vec<Role>});
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(rename = "BartworksOreVariant")]
pub enum Variant {
    Ore,
    SmallOre,
}
record!(Sample,"BartworksOreSample",{variant:Variant,native_overlay_eligible:bool,item:Option<String>,amount:Option<i32>});
record!(Material,"BartworksOreMaterial",{key:String,meta:i16,ordinal:u32,default_name:String,name:String,ore_feature:bool,small_ore_feature:bool,samples:Vec<Sample>});
record!(Lookup,"BartworksOreLookup",{meta:i16,ordinal:u32,layers:Vec<String>});

fn text(value: &str) -> bool {
    !value.is_empty() && value.len() <= 4096
}
fn sample<'a>(id: &str, meta: i32, items: &BTreeMap<&str, &'a Item>) -> Result<&'a Item> {
    let item = *items
        .get(id)
        .context("Bartworks native sample is missing")?;
    ensure!(
        item.meta == meta && item.nbt.is_none(),
        "Bartworks native sample metadata or NBT differs"
    );
    Ok(item)
}
pub fn validate(
    id: &str,
    rows: &[&GeneticReferenceRow],
    items: &BTreeMap<&str, &Item>,
) -> Result<()> {
    let ReferenceData::BartworksOresDefinition(def) = &rows[0].data else {
        anyhow::bail!("Missing Bartworks ore definition")
    };
    ensure!(
        def.ready
            && !def.consuming
            && text(&def.source.owner)
            && def.source.handler == "bartworks.neiHandler.OreNEIHandler"
            && def.source.key == def.source.handler,
        "Bartworks ore source/readiness differs"
    );
    let data: Vec<_> = rows.iter().map(|row| &row.data).collect();
    ensure!(
        id == super::value_id(
            "reference",
            &serde_json::json!({"kind":"bartworksOres","records":data})
        )?,
        "Bartworks native domain identity differs"
    );
    let counts = &def.counts;
    ensure!(
        counts.layers <= 65536
            && counts.materials <= 65536
            && counts.lookups <= 65536
            && def.registered_materials <= 65536
            && counts.materials <= def.registered_materials
            && rows.len() == 1 + (counts.layers + counts.materials + counts.lookups) as usize,
        "Incomplete Bartworks ore domain"
    );
    let mut keys = BTreeSet::new();
    let mut active = 0;
    let mut weight = 0i32;
    let mut expected = BTreeMap::<i16, BTreeMap<&str, u32>>::new();
    let mut bw_registry: Option<&str> = None;
    let mut gt_registry: Option<&str> = None;
    for ordinal in 0..counts.layers {
        let ReferenceData::BartworksOreLayer(layer) = &rows[1 + ordinal as usize].data else {
            anyhow::bail!("Bartworks layer order differs")
        };
        ensure!(
            layer.ordinal == ordinal
                && text(&layer.key)
                && keys.insert(layer.key.as_str())
                && layer.roles.len() == 4
                && layer.bw_mask <= 15
                && layer.enabled == layer.active_ordinal.is_some(),
            "Invalid Bartworks generator identity/roles"
        );
        if layer.enabled {
            ensure!(
                layer.active_ordinal == Some(active) && ordinal == active,
                "Bartworks active generator order differs"
            );
            active += 1;
            weight = weight.wrapping_add(layer.weight);
        }
        let moon = layer.generator_class == "bartworks.system.oregen.BWWorldGenRoss128ba";
        ensure!(
            moon || layer.generator_class == "bartworks.system.oregen.BWWorldGenRoss128b",
            "Unknown Bartworks generator subclass"
        );
        ensure!(
            text(&layer.dimension.name)
                && layer.dimension.key
                    == if moon {
                        "moon.Ross128ba"
                    } else {
                        "planet.Ross128b"
                    }
                && layer.dimension.provider
                    == if moon {
                        "bwcrossmod.galacticraft.planets.ross128ba.WorldProviderRoss128ba"
                    } else {
                        "bwcrossmod.galacticraft.planets.ross128b.WorldProviderRoss128b"
                    },
            "Bartworks dimension policy differs"
        );
        ensure!(
            layer
                .replacement
                .registry
                .as_ref()
                .is_none_or(|name| text(name) && name.contains(':'))
                && layer.replacement.metas.len() <= 256,
            "Invalid Bartworks replacement rule"
        );
        let roles = [
            RoleKind::Primary,
            RoleKind::Secondary,
            RoleKind::Between,
            RoleKind::Sporadic,
        ];
        for (index, role) in layer.roles.iter().enumerate() {
            ensure!(
                role.role == roles[index] && role.bartworks == (layer.bw_mask & (8 >> index) != 0),
                "Bartworks role ownership differs"
            );
            let item = sample(&role.item, role.meta, items)?;
            let registry = if role.bartworks {
                &mut bw_registry
            } else {
                &mut gt_registry
            };
            if let Some(previous) = registry {
                ensure!(
                    *previous == item.registry,
                    "Bartworks native sample owner differs"
                );
            } else {
                *registry = Some(item.registry.as_str());
            }
            *expected
                .entry(role.meta as i16)
                .or_default()
                .entry(layer.key.as_str())
                .or_default() += 1;
        }
    }
    ensure!(
        weight == def.enabled_weight,
        "Bartworks enabled native weight differs"
    );
    let mut metas = BTreeSet::new();
    let mut previous = None;
    for index in 0..counts.materials {
        let ReferenceData::BartworksOreMaterial(material) =
            &rows[(1 + counts.layers + index) as usize].data
        else {
            anyhow::bail!("Bartworks material order differs")
        };
        ensure!(
            material.ordinal < def.registered_materials
                && previous.is_none_or(|value| value < material.ordinal)
                && metas.insert(material.meta)
                && material.key == format!("werkstoff:{}", material.meta)
                && text(&material.default_name)
                && text(&material.name)
                && (material.ore_feature || material.small_ore_feature)
                && material.samples.len() == 2,
            "Invalid Werkstoff material identity/features"
        );
        previous = Some(material.ordinal);
        for (index, value) in material.samples.iter().enumerate() {
            ensure!(
                value.variant
                    == if index == 0 {
                        Variant::Ore
                    } else {
                        Variant::SmallOre
                    }
                    && value.native_overlay_eligible == material.ore_feature
                    && value.item.is_some() == value.amount.is_some(),
                "Werkstoff native variant policy differs"
            );
            if let Some(id) = &value.item {
                ensure!(
                    value.amount.is_some_and(|amount| amount > 0),
                    "Invalid Werkstoff sample amount"
                );
                let item = sample(id, material.meta as i32, items)?;
                if index == 0 {
                    if let Some(registry) = bw_registry {
                        ensure!(
                            item.registry == registry,
                            "Werkstoff ordinary ore owner differs"
                        );
                    }
                }
            }
        }
    }
    let mut actual = BTreeMap::new();
    let mut previous = None;
    for ordinal in 0..counts.lookups {
        let ReferenceData::BartworksOreLookup(lookup) =
            &rows[(1 + counts.layers + counts.materials + ordinal) as usize].data
        else {
            anyhow::bail!("Bartworks lookup order differs")
        };
        ensure!(
            lookup.ordinal == ordinal
                && previous.is_none_or(|meta| meta < lookup.meta)
                && !lookup.layers.is_empty()
                && lookup.layers.len() <= 262144,
            "Invalid Bartworks metadata lookup"
        );
        previous = Some(lookup.meta);
        let mut members = BTreeMap::new();
        for key in &lookup.layers {
            ensure!(
                keys.contains(key.as_str()),
                "Bartworks metadata refers to unknown layer"
            );
            *members.entry(key.as_str()).or_insert(0u32) += 1;
        }
        actual.insert(lookup.meta, members);
    }
    ensure!(
        expected == actual,
        "Bartworks metadata membership multiplicity differs"
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn fixture() -> serde_json::Value {
        serde_json::from_str(
            &std::fs::read_to_string(
                std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                    .join("../../contracts/fixtures/bartworks-ores-contract.json"),
            )
            .unwrap(),
        )
        .unwrap()
    }
    fn facts(raw: &serde_json::Value) -> Vec<Item> {
        let source = crate::source::Source::open(
            &std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../contracts/fixtures/source-semantic-40"),
        )
        .unwrap();
        let domain = super::super::Domain::load(&source).unwrap();
        raw["items"]
            .as_array()
            .unwrap()
            .iter()
            .map(|raw| {
                let mut item = serde_json::to_value(&domain.items[0]).unwrap();
                for key in ["id", "registry", "meta", "nbt", "tags"] {
                    item[key] = raw[key].clone();
                }
                item["icon"] = serde_json::Value::Null;
                serde_json::from_value(item).unwrap()
            })
            .collect()
    }
    fn reseal(rows: &mut [GeneticReferenceRow]) {
        let data: Vec<_> = rows.iter().map(|row| &row.data).collect();
        let id = super::super::value_id(
            "reference",
            &serde_json::json!({"kind":"bartworksOres","records":data}),
        )
        .unwrap();
        for row in rows {
            row.domain = id.clone();
            row.id = row.content_id().unwrap();
        }
    }
    #[test]
    fn java_bartworks_ore_rules_and_duplicate_lookups_are_checked() {
        let raw = fixture();
        let items = facts(&raw);
        let index = items.iter().map(|item| (item.id.as_str(), item)).collect();
        let rows: Vec<GeneticReferenceRow> =
            serde_json::from_value(raw["records"].clone()).unwrap();
        super::super::references::validate(&rows, &index).unwrap();
        let mut lost = rows.clone();
        if let ReferenceData::BartworksOreLookup(lookup) = &mut lost[4].data {
            lookup.layers.pop();
        }
        reseal(&mut lost);
        assert!(super::super::references::validate(&lost, &index)
            .unwrap_err()
            .to_string()
            .contains("multiplicity"));
        let mut wrong = rows.clone();
        if let ReferenceData::BartworksOreLayer(layer) = &mut wrong[1].data {
            layer.bw_mask = 0;
        }
        reseal(&mut wrong);
        assert!(super::super::references::validate(&wrong, &index).is_err());
        assert!(super::super::references::validate(&rows, &BTreeMap::new()).is_err());
        let mut empty = vec![rows[0].clone()];
        if let ReferenceData::BartworksOresDefinition(def) = &mut empty[0].data {
            def.counts = Counts {
                layers: 0,
                materials: 0,
                lookups: 0,
            };
            def.registered_materials = 0;
            def.enabled_weight = 0;
        }
        reseal(&mut empty);
        super::super::references::validate(&empty, &BTreeMap::new()).unwrap();
        if let ReferenceData::BartworksOresDefinition(def) = &mut empty[0].data {
            def.ready = false;
        }
        reseal(&mut empty);
        assert!(super::super::references::validate(&empty, &BTreeMap::new()).is_err());
    }
    #[test]
    fn java_bartworks_ore_complete_domain_requires_source_and_samples() {
        let raw = fixture();
        let source = crate::source::Source::open(
            &std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../contracts/fixtures/source-semantic-40"),
        )
        .unwrap();
        let mut domain = super::super::Domain::load(&source).unwrap();
        let rows: Vec<GeneticReferenceRow> =
            serde_json::from_value(raw["records"].clone()).unwrap();
        let ReferenceData::BartworksOresDefinition(def) = &rows[0].data else {
            panic!()
        };
        let mut category = domain.categories[0].clone();
        category.source = def.source.clone();
        category.id = super::super::origin_id("category", &category.source).unwrap();
        category.reference = Some(rows[0].domain.clone());
        category.program = None;
        category.icon = None;
        category.machines.clear();
        domain.categories.push(category);
        domain.references = rows;
        for item in facts(&raw) {
            domain.items.retain(|existing| existing.id != item.id);
            domain.items.push(item);
        }
        domain.validate(&source).unwrap();
        domain.categories.pop();
        assert!(domain.validate(&source).is_err());
    }
}
