//! Pinned small-ore/vein configuration, live generators and native NEI projections.
use super::{
    references::{GeneticReferenceRow, ReferenceData},
    Item, Origin,
};
use anyhow::{ensure, Context, Result};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(rename = "GtMineralFamily")]
pub enum Family {
    SmallOre,
    Vein,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(rename = "GtMineralHostPolicy")]
pub enum HostPolicy {
    NativeDimensionSevenHosts,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(rename = "GtMineralShareSemantics")]
pub enum ShareSemantics {
    RelativeAmount,
    RelativeWeight,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[schemars(rename = "GtMineralCounts")]
pub struct Counts {
    pub entries: u32,
    pub dimension_shares: u32,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(rename = "GtMineralMaterial")]
pub struct Material {
    pub key: String,
    pub meta: i32,
    pub name: String,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(rename = "GtMineralDropLookup")]
pub struct DropLookup {
    pub name: String,
    pub meta: i16,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[schemars(rename = "GtMineralDefinition")]
pub struct Definition {
    pub source: Origin,
    pub family: Family,
    pub ready: bool,
    pub consuming: bool,
    pub host_policy: HostPolicy,
    pub share_semantics: ShareSemantics,
    pub dust_materials: Vec<Material>,
    pub drop_lookup: Vec<DropLookup>,
    pub restrict_biome_support: Option<bool>,
    pub counts: Counts,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[schemars(rename = "GtMineralSettings")]
pub struct Settings {
    pub min_y: i32,
    pub max_y: i32,
    pub amount: Option<i32>,
    pub weight: Option<i32>,
    pub density: Option<i32>,
    pub size: Option<i32>,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[schemars(rename = "GtMineralFlags")]
pub struct Flags {
    pub overworld: bool,
    pub nether: bool,
    pub end: bool,
    pub twilight_forest: bool,
    pub end_asteroid: Option<bool>,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[schemars(rename = "GtMineralGenerator")]
pub struct Generator {
    pub min_y: i16,
    pub max_y: i16,
    pub amount: Option<i16>,
    pub weight: Option<i16>,
    pub density: Option<i16>,
    pub size: Option<i16>,
    pub metas: Vec<i16>,
    pub biome: String,
    pub providers: Vec<String>,
    pub blacklisted_providers: Vec<String>,
    pub flags: Flags,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(rename = "GtMineralDimension")]
pub struct Dimension {
    pub name: String,
    pub enabled: bool,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(rename = "GtMineralHost")]
pub struct Host {
    pub host: u8,
    pub item: String,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(rename = "GtMineralRoleKind")]
pub enum RoleKind {
    Small,
    Primary,
    Secondary,
    Between,
    Sporadic,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(rename = "GtMineralRole")]
pub struct Role {
    pub role: RoleKind,
    pub material: Material,
    pub meta: i16,
    pub hosts: Vec<Host>,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(rename = "GtMineralDrop")]
pub struct Drop {
    pub item: String,
    pub amount: i32,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[schemars(rename = "GtMineralEntry")]
pub struct Entry {
    pub key: String,
    pub ordinal: u32,
    pub display_name: Option<String>,
    pub configured: Settings,
    pub enabled_by_default: bool,
    pub generator: Option<Generator>,
    pub dimensions: Vec<Dimension>,
    pub display_dimensions: Vec<String>,
    pub display_height: String,
    pub native_weight: i16,
    pub roles: Vec<Role>,
    pub drops: Vec<Drop>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(rename = "GtMineralNonFinite")]
pub enum NonFinite {
    Nan,
    PositiveInfinity,
    NegativeInfinity,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[schemars(rename = "GtMineralDimensionShare")]
pub struct DimensionShare {
    pub dimension: String,
    pub ordinal: u32,
    pub key: String,
    pub numerator: i16,
    pub denominator: i32,
    pub native_share: Option<String>,
    pub non_finite: Option<NonFinite>,
}
fn text(value: &str) -> bool {
    !value.is_empty() && value.len() <= 4096
}
fn material(value: &Material) -> Result<()> {
    ensure!(
        text(&value.key) && text(&value.name),
        "Invalid GT material identity or display name"
    );
    Ok(())
}
fn settings(value: &Settings, small: bool) -> Result<()> {
    ensure!(
        if small {
            value.amount.is_some()
                && value.weight.is_none()
                && value.density.is_none()
                && value.size.is_none()
        } else {
            value.amount.is_none()
                && value.weight.is_some()
                && value.density.is_some()
                && value.size.is_some()
        },
        "GT mineral family settings differ"
    );
    Ok(())
}

pub fn validate(
    id: &str,
    rows: &[&GeneticReferenceRow],
    items: &BTreeMap<&str, &Item>,
) -> Result<()> {
    let ReferenceData::GtMineralsDefinition(definition) = &rows[0].data else {
        anyhow::bail!("Missing GT mineral definition")
    };
    let small = definition.family == Family::SmallOre;
    ensure!(
        definition.ready
            && !definition.consuming
            && !definition.source.owner.is_empty()
            && definition.source.handler
                == if small {
                    "gtneioreplugin.plugin.gregtech5.PluginGT5SmallOreStat"
                } else {
                    "gtneioreplugin.plugin.gregtech5.PluginGT5VeinStat"
                }
            && definition.source.key
                == if small {
                    "GTOrePluginOreSmall"
                } else {
                    "GTOrePluginVein"
                },
        "GT mineral source/readiness/consumption differs"
    );
    ensure!(
        matches!(
            (&definition.share_semantics, small),
            (ShareSemantics::RelativeAmount, true) | (ShareSemantics::RelativeWeight, false)
        ) && definition.restrict_biome_support.is_some() == small,
        "GT mineral share interpretation differs"
    );
    ensure!(
        definition.counts.entries <= 65536
            && definition.counts.dimension_shares <= 262144
            && rows.len()
                == 1 + definition.counts.entries as usize
                    + definition.counts.dimension_shares as usize
            && definition.dust_materials.len() <= 64
            && definition.drop_lookup.len() <= 262144,
        "Incomplete or oversized GT mineral domain"
    );
    let data: Vec<_> = rows.iter().map(|row| &row.data).collect();
    ensure!(
        id == super::value_id(
            "reference",
            &serde_json::json!({"kind":"gtMinerals","records":data})
        )?,
        "GT mineral content identity differs"
    );
    for dust in &definition.dust_materials {
        material(dust)?;
    }
    let mut previous = "";
    for lookup in &definition.drop_lookup {
        ensure!(
            text(&lookup.name) && lookup.name.as_str() > previous,
            "Invalid duplicate GT drop lookup"
        );
        previous = &lookup.name;
    }
    ensure!(
        small || definition.dust_materials.is_empty() && definition.drop_lookup.is_empty(),
        "Vein definition contains small-ore drops"
    );
    let mut entries = BTreeMap::new();
    let mut expected = BTreeMap::<(String, String), u32>::new();
    previous = "";
    for ordinal in 0..definition.counts.entries {
        let ReferenceData::GtMineralEntry(entry) = &rows[1 + ordinal as usize].data else {
            anyhow::bail!("Missing GT mineral entry")
        };
        ensure!(
            text(&entry.key)
                && entry.key.as_str() > previous
                && entry.ordinal == ordinal
                && entry.dimensions.len() <= 4096
                && entry.display_dimensions.len() <= 4096
                && entry.drops.len() <= 65536,
            "Invalid GT mineral entry order or budget"
        );
        previous = &entry.key;
        entries.insert(entry.key.as_str(), entry);
        settings(&entry.configured, small)?;
        ensure!(
            entry.native_weight
                == (if small {
                    entry.configured.amount
                } else {
                    entry.configured.weight
                })
                .unwrap() as i16
                && entry.display_height
                    == format!("{}-{}", entry.configured.min_y, entry.configured.max_y),
            "GT mineral helper values differ from definition"
        );
        ensure!(
            entry.display_name.is_some() != small
                && entry.display_name.as_ref().is_none_or(|name| text(name)),
            "GT mineral display name differs"
        );
        let mut last = "";
        for dimension in &entry.dimensions {
            ensure!(
                text(&dimension.name) && dimension.name.as_str() > last,
                "Invalid GT mineral original dimensions"
            );
            last = &dimension.name;
        }
        let mut last = "";
        for dimension in &entry.display_dimensions {
            ensure!(
                text(dimension) && dimension.as_str() > last,
                "Invalid GT mineral projected dimensions"
            );
            last = dimension;
            *expected
                .entry((dimension.clone(), entry.key.clone()))
                .or_default() += 1;
        }
        let roles: &[RoleKind] = if small {
            &[RoleKind::Small]
        } else {
            &[
                RoleKind::Primary,
                RoleKind::Secondary,
                RoleKind::Between,
                RoleKind::Sporadic,
            ]
        };
        ensure!(
            entry.roles.len() == roles.len(),
            "GT mineral semantic layers missing"
        );
        let mut registry = None;
        for (role, kind) in entry.roles.iter().zip(roles) {
            material(&role.material)?;
            ensure!(
                serde_json::to_value(&role.role)? == serde_json::to_value(kind)?
                    && role.meta == role.material.meta as i16
                    && role.hosts.len() == 7,
                "GT mineral role, metadata or native hosts differ"
            );
            for (host_index, host) in role.hosts.iter().enumerate() {
                let fact = items
                    .get(host.item.as_str())
                    .context("Native mineral host item absent")?;
                ensure!(
                    host.host as usize == host_index
                        && fact.meta
                            == i32::from(role.meta)
                                + (if small { 16000 } else { 0 })
                                + host_index as i32 * 1000
                        && fact.nbt.is_none(),
                    "GT mineral host metadata or NBT differs"
                );
                if let Some(expected) = registry {
                    ensure!(
                        expected == fact.registry.as_str(),
                        "GT mineral host registry changed"
                    );
                } else {
                    registry = Some(fact.registry.as_str());
                }
            }
        }
        ensure!(
            small || entry.drops.is_empty(),
            "Vein samples reclassified as guaranteed drops"
        );
        for drop in &entry.drops {
            ensure!(
                drop.amount > 0 && items.contains_key(drop.item.as_str()),
                "Invalid or missing native small-ore drop"
            );
        }
        if let Some(generator) = &entry.generator {
            ensure!(
                entry.enabled_by_default
                    && generator.min_y == entry.configured.min_y as i16
                    && generator.metas
                        == entry.roles.iter().map(|role| role.meta).collect::<Vec<_>>()
                    && text(&generator.biome)
                    && generator.providers.len() <= 256
                    && generator.blacklisted_providers.len() <= 256
                    && generator
                        .providers
                        .iter()
                        .chain(&generator.blacklisted_providers)
                        .all(|name| text(name)),
                "GT mineral live generator is invalid"
            );
            let expected_max = if small {
                (i32::from(generator.min_y) + 1).max(entry.configured.max_y) as i16
            } else {
                let max = entry.configured.max_y as i16;
                if i32::from(max) < i32::from(generator.min_y) + 9 {
                    (i32::from(generator.min_y) + 9) as i16
                } else {
                    max
                }
            };
            ensure!(
                generator.max_y == expected_max && generator.flags.end_asteroid.is_some() != small,
                "Native GT generator height or flags differ"
            );
            if small {
                ensure!(
                    generator.amount == Some(entry.configured.amount.unwrap().max(1) as i16)
                        && generator.weight.is_none()
                        && generator.density.is_none()
                        && generator.size.is_none(),
                    "Native adjusted small-ore amount differs"
                );
            } else {
                ensure!(
                    generator.amount.is_none()
                        && generator.weight == Some(entry.configured.weight.unwrap() as i16)
                        && generator.density == Some(entry.configured.density.unwrap() as i16)
                        && generator.size == Some(entry.configured.size.unwrap().max(1) as i16),
                    "Native vein settings differ"
                );
            }
        } else {
            ensure!(
                !entry.enabled_by_default && !small,
                "Enabled GT generator is absent"
            );
        }
    }
    let mut actual = BTreeMap::<(String, String), u32>::new();
    let mut groups = BTreeMap::<&str, Vec<&DimensionShare>>::new();
    let mut last_dimension = "";
    let mut next_ordinal = 0;
    for row in &rows[1 + definition.counts.entries as usize..] {
        let ReferenceData::GtMineralDimensionShare(share) = &row.data else {
            anyhow::bail!("Unexpected GT mineral record")
        };
        if share.dimension != last_dimension {
            ensure!(
                text(&share.dimension) && share.dimension.as_str() > last_dimension,
                "Invalid GT dimension share order"
            );
            last_dimension = &share.dimension;
            next_ordinal = 0;
        }
        let entry = entries
            .get(share.key.as_str())
            .context("GT share references missing mineral")?;
        ensure!(
            share.ordinal == next_ordinal && share.numerator == entry.native_weight,
            "Invalid GT native share ordinal/numerator"
        );
        next_ordinal += 1;
        groups.entry(&share.dimension).or_default().push(share);
        *actual
            .entry((share.dimension.clone(), share.key.clone()))
            .or_default() += 1;
    }
    ensure!(
        actual == expected && groups.len() <= 4096,
        "GT mineral dimension projection incomplete"
    );
    for group in groups.values() {
        let total = group
            .iter()
            .try_fold(0i32, |sum, share| {
                sum.checked_add(i32::from(share.numerator))
            })
            .context("GT share sum overflow")?;
        for share in group {
            ensure!(
                share.denominator == total,
                "GT dimension share denominator differs"
            );
            let ratio = f64::from(share.numerator) / f64::from(total);
            if ratio.is_finite() {
                let value = share
                    .native_share
                    .as_ref()
                    .context("Finite GT share missing")?;
                ensure!(
                    value.len() <= 32
                        && share.non_finite.is_none()
                        && value.parse::<f64>()?.to_bits() == ratio.to_bits(),
                    "GT native display share differs"
                );
            } else {
                let expected = if ratio.is_nan() {
                    NonFinite::Nan
                } else if ratio.is_sign_positive() {
                    NonFinite::PositiveInfinity
                } else {
                    NonFinite::NegativeInfinity
                };
                ensure!(
                    share.native_share.is_none() && share.non_finite.as_ref() == Some(&expected),
                    "Native non-finite GT share was hidden or forged"
                );
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn fixture() -> serde_json::Value {
        serde_json::from_str(include_str!(
            "../../../../contracts/fixtures/gt-minerals-contract.json"
        ))
        .unwrap()
    }
    fn items(raw: &serde_json::Value) -> Vec<Item> {
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
            .map(|item| {
                let mut fact = serde_json::to_value(&domain.items[0]).unwrap();
                for field in ["id", "registry", "meta", "nbt"] {
                    fact[field] = item[field].clone();
                }
                fact["icon"] = serde_json::Value::Null;
                serde_json::from_value(fact).unwrap()
            })
            .collect()
    }
    fn reseal(rows: &mut [GeneticReferenceRow]) {
        let data: Vec<_> = rows.iter().map(|row| &row.data).collect();
        let domain = super::super::value_id(
            "reference",
            &serde_json::json!({"kind":"gtMinerals","records":data}),
        )
        .unwrap();
        for row in rows {
            row.domain = domain.clone();
            row.id = row.content_id().unwrap();
        }
    }
    #[test]
    fn java_gt_mineral_records_reject_lost_layers_and_forged_probabilities() {
        let raw = fixture();
        let facts = items(&raw);
        let index = facts.iter().map(|item| (item.id.as_str(), item)).collect();
        for field in ["small", "veins", "disabled"] {
            let mut rows: Vec<GeneticReferenceRow> =
                serde_json::from_value(raw[field].clone()).unwrap();
            super::super::references::validate(&rows, &index).unwrap();
            if let ReferenceData::GtMineralEntry(entry) = &mut rows[1].data {
                entry.roles[0].hosts.pop();
            }
            reseal(&mut rows);
            assert!(super::super::references::validate(&rows, &index).is_err());
        }
        let mut rows: Vec<GeneticReferenceRow> =
            serde_json::from_value(raw["small"].clone()).unwrap();
        if let ReferenceData::GtMineralDimensionShare(share) = &mut rows[2].data {
            share.native_share = Some("1.0".into());
            share.non_finite = None;
        }
        reseal(&mut rows);
        assert!(super::super::references::validate(&rows, &index).is_err());
        let mut rows: Vec<GeneticReferenceRow> =
            serde_json::from_value(raw["veins"].clone()).unwrap();
        if let ReferenceData::GtMineralEntry(entry) = &mut rows[1].data {
            entry.generator = None;
        }
        reseal(&mut rows);
        assert!(super::super::references::validate(&rows, &index).is_err());
    }
    #[test]
    fn java_gt_minerals_complete_domain_requires_source_and_item_facts() {
        let raw = fixture();
        let source = crate::source::Source::open(
            &std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../contracts/fixtures/source-semantic-40"),
        )
        .unwrap();
        let mut domain = super::super::Domain::load(&source).unwrap();
        domain.items.extend(items(&raw));
        for field in ["small", "veins"] {
            let rows: Vec<GeneticReferenceRow> =
                serde_json::from_value(raw[field].clone()).unwrap();
            let ReferenceData::GtMineralsDefinition(definition) = &rows[0].data else {
                panic!()
            };
            let mut category = domain.categories[0].clone();
            category.source = definition.source.clone();
            category.id = super::super::origin_id("category", &category.source).unwrap();
            category.reference = Some(rows[0].domain.clone());
            category.program = None;
            category.icon = None;
            category.machines.clear();
            domain.categories.push(category);
            domain.references.extend(rows);
        }
        domain.validate(&source).unwrap();
        domain.items.pop();
        assert!(domain.validate(&source).is_err());
    }
}
