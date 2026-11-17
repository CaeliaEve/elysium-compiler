//! Native structure component/tier lookup is reference information, not material consumption.
use super::{
    references::{GeneticReferenceRow, ReferenceData},
    Domain, Item, Origin,
};
use anyhow::{ensure, Context, Result};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

const GT: &str = "blockrenderer6343.integration.gregtech.GTNEIMultiblockHandler";
const COMPAT: &str = "blockrenderer6343.integration.structurelib.StructureCompatNEIHandler";
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum Family {
    Gt,
    Compat,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum Matching {
    NativeItemHashDamageSum,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum Projection {
    CompleteNativeMapAndItemPanel,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[schemars(rename = "StructureCatalogCounts")]
pub struct Counts {
    pub controllers: u32,
    pub lookups: u32,
    pub unresolved_lookups: u32,
    pub tier_lookups: u32,
    pub panel_items: u32,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[schemars(rename = "StructureCatalogDefinition")]
pub struct Definition {
    pub source: Origin,
    pub family: Family,
    pub ready: bool,
    pub consuming: bool,
    pub matching: Matching,
    pub projection: Projection,
    pub counts: Counts,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct StructureTierItem {
    pub native_hash: String,
    pub items: Vec<String>,
    pub tier: i32,
    pub channels: Vec<String>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum StructureChannelDefinitionKind {
    Bounded,
    Unbounded,
    Custom,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum StructureChannelValidation {
    InclusiveBounds,
    OpaqueCustom,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum StructureChannelNormalization {
    ClampToBounds,
    OpaqueCustom,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct StructureChannelDefinition {
    pub kind: StructureChannelDefinitionKind,
    pub minimum: Option<i32>,
    pub maximum: Option<i32>,
    pub validation: StructureChannelValidation,
    pub normalization: StructureChannelNormalization,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[schemars(rename = "StructureCatalogController")]
pub struct Controller {
    pub ordinal: u32,
    pub structure: String,
    pub controller: String,
    pub origin: Origin,
    pub has_data: bool,
    pub has_tier_data: bool,
    pub max_tier: u32,
    pub channels: BTreeMap<String, i32>,
    pub channel_definitions: BTreeMap<String, StructureChannelDefinition>,
    pub tier_items: Vec<StructureTierItem>,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct StructureComponentLookup {
    pub native_hash: String,
    pub items: Vec<String>,
    pub structures: Vec<String>,
}

fn hash(raw: &str) -> Result<i64> {
    let value = raw
        .parse::<i64>()
        .context("Invalid native structure session hash")?;
    ensure!(
        value.to_string() == raw
            && value % 31 == 0
            && (-133_143_986_176..=133_143_986_114).contains(&value),
        "Invalid native item/damage hash representation"
    );
    Ok(value)
}
fn specimens<'a>(members: &'a [String], items: &BTreeMap<&str, &Item>) -> Result<()> {
    ensure!(
        members.len() <= 65536 && members.windows(2).all(|pair| pair[0] < pair[1]),
        "Invalid native component specimen ordering"
    );
    for id in members {
        ensure!(
            items.contains_key(id.as_str()),
            "Native structure specimen fact absent"
        );
    }
    Ok(())
}
pub(super) fn validate(
    id: &str,
    rows: &[&GeneticReferenceRow],
    items: &BTreeMap<&str, &Item>,
) -> Result<()> {
    let ReferenceData::StructureCatalogDefinition(definition) = &rows[0].data else {
        anyhow::bail!("Missing native structure definition")
    };
    ensure!(
        definition.ready
            && !definition.consuming
            && definition.source.handler
                == match definition.family {
                    Family::Gt => GT,
                    Family::Compat => COMPAT,
                },
        "Native structure family, readiness or consumption differs"
    );
    let counts = &definition.counts;
    ensure!(
        counts.controllers <= 8192
            && counts.lookups <= 65536
            && counts.panel_items <= 1048576
            && rows.len() == 1 + counts.controllers as usize + counts.lookups as usize,
        "Incomplete native structure catalogue"
    );
    let data: Vec<_> = rows.iter().map(|row| &row.data).collect();
    ensure!(
        id == super::value_id(
            "reference",
            &serde_json::json!({"kind":"nativeStructures","records":data})
        )?,
        "Native structure catalogue content identity differs"
    );
    let mut controllers = BTreeSet::new();
    let mut projections = BTreeMap::<&str, &[String]>::new();
    let mut tiers = 0u32;
    for (ordinal, row) in rows[1..1 + counts.controllers as usize].iter().enumerate() {
        let ReferenceData::StructureCatalogController(controller) = &row.data else {
            anyhow::bail!("Expected native controller record")
        };
        ensure!(
            controller.ordinal as usize == ordinal
                && controllers.insert(controller.structure.as_str())
                && controller.structure == super::origin_id("structure", &controller.origin)?
                && controller.max_tier >= 1
                && controller.max_tier <= i32::MAX as u32
                && controller.channels.len() <= 256
                && controller.channel_definitions.len() <= 256
                && controller.tier_items.len() <= 4096,
            "Invalid native controller identity/order/tier"
        );
        specimens(std::slice::from_ref(&controller.controller), items)?;
        match definition.family {
            Family::Compat => ensure!(
                controller
                    .origin
                    .key
                    .ends_with(&format!(":{}", controller.controller)),
                "Native compatibility provider/controller binding differs"
            ),
            Family::Gt => {
                let native = controller
                    .origin
                    .key
                    .parse::<i32>()
                    .context("Invalid GT structure registry index")?;
                let item = items[controller.controller.as_str()];
                ensure!(
                    native >= 0
                        && native.to_string() == controller.origin.key
                        && item.registry == "gregtech:gt.blockmachines"
                        && item.meta == native,
                    "Native GT controller/index binding differs"
                );
            }
        }
        ensure!(
            controller
                .channels
                .keys()
                .all(|channel| !channel.is_empty() && channel.len() <= 128),
            "Invalid native construction channels"
        );
        ensure!(
            !controller.has_tier_data || controller.has_data,
            "Native tier information has no data flag"
        );
        if !controller.has_data {
            ensure!(
                controller.max_tier == 1
                    && controller.channels.is_empty()
                    && controller.channel_definitions.is_empty()
                    && controller.tier_items.is_empty(),
                "Native empty tier information gained invented rules"
            );
        }
        if !controller.has_tier_data {
            ensure!(
                controller.max_tier == 1 && controller.tier_items.is_empty(),
                "Native tier-free information gained item tiers"
            );
        }
        for (channel, definition) in &controller.channel_definitions {
            let maximum = controller
                .channels
                .get(channel)
                .context("Native channel definition has no stored maximum")?;
            ensure!(
                definition.maximum.is_none_or(|value| value == *maximum),
                "Native channel definition/stored maximum differs"
            );
            match definition.kind {
                StructureChannelDefinitionKind::Bounded => ensure!(
                    definition.minimum.is_some_and(|minimum| minimum >= 1
                        && definition.maximum.is_some_and(|maximum| maximum >= minimum))
                        && definition.validation == StructureChannelValidation::InclusiveBounds
                        && definition.normalization == StructureChannelNormalization::ClampToBounds,
                    "Invalid native bounded channel definition"
                ),
                StructureChannelDefinitionKind::Unbounded => ensure!(
                    definition.minimum == Some(1)
                        && definition.maximum == Some(i32::MAX)
                        && definition.validation == StructureChannelValidation::InclusiveBounds
                        && definition.normalization == StructureChannelNormalization::ClampToBounds,
                    "Invalid native unbounded channel definition"
                ),
                StructureChannelDefinitionKind::Custom => {}
            }
        }
        let mut previous = None;
        for tier in &controller.tier_items {
            let key = hash(&tier.native_hash)?;
            ensure!(
                previous.is_none_or(|value| value < key)
                    && tier.channels.len() <= 256
                    && tier
                        .channels
                        .iter()
                        .all(|channel| !channel.is_empty() && channel.len() <= 128)
                    && tier.channels.iter().collect::<BTreeSet<_>>().len() == tier.channels.len(),
                "Invalid native tier item/channel lookup"
            );
            previous = Some(key);
            specimens(&tier.items, items)?;
            if let Some(prior) = projections.insert(&tier.native_hash, &tier.items) {
                ensure!(
                    prior == tier.items,
                    "Inconsistent native hash collision projection"
                );
            }
            tiers = tiers.checked_add(1).context("Native tier count overflow")?;
        }
    }
    let mut previous = None;
    let mut unresolved = 0;
    let mut memberships = 0usize;
    for row in &rows[1 + counts.controllers as usize..] {
        let ReferenceData::StructureComponentLookup(lookup) = &row.data else {
            anyhow::bail!("Expected native component association")
        };
        let key = hash(&lookup.native_hash)?;
        ensure!(
            previous.is_none_or(|value| value < key)
                && !lookup.structures.is_empty()
                && lookup.structures.windows(2).all(|pair| pair[0] < pair[1])
                && lookup
                    .structures
                    .iter()
                    .all(|value| controllers.contains(value.as_str())),
            "Invalid native component/controller association"
        );
        previous = Some(key);
        specimens(&lookup.items, items)?;
        if let Some(prior) = projections.insert(&lookup.native_hash, &lookup.items) {
            ensure!(
                prior == lookup.items,
                "Native tier/component projections disagree"
            );
        }
        if lookup.items.is_empty() {
            unresolved += 1;
        }
        memberships += lookup.structures.len();
        ensure!(
            memberships <= 2097152,
            "Native component membership budget exceeded"
        );
    }
    ensure!(
        tiers == counts.tier_lookups && unresolved == counts.unresolved_lookups,
        "Native structure evidence counts differ"
    );
    Ok(())
}

/// Every declared controller owns actual captured geometry; a GUI display cannot substitute for it.
pub(super) fn validate_facts(domain: &Domain) -> Result<()> {
    let structures: BTreeMap<_, _> = domain
        .structures
        .iter()
        .map(|structure| (structure.id.as_str(), structure))
        .collect();
    let mut owned = BTreeSet::new();
    for row in &domain.references {
        if let ReferenceData::StructureCatalogController(controller) = &row.data {
            let structure = structures
                .get(controller.structure.as_str())
                .context("Native structure controller has no captured geometry")?;
            ensure!(
                owned.insert(controller.structure.as_str())
                    && structure.controller == controller.controller
                    && serde_json::to_value(&structure.source)?
                        == serde_json::to_value(&controller.origin)?,
                "Native controller geometry ownership differs"
            );
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn fixture() -> (Vec<GeneticReferenceRow>, Vec<Item>) {
        let raw: serde_json::Value = serde_json::from_str(
            &std::fs::read_to_string(
                std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                    .join("../../contracts/fixtures/structure-reference-contract.json"),
            )
            .unwrap(),
        )
        .unwrap();
        let rows = serde_json::from_value(raw["records"].clone()).unwrap();
        let items = raw["items"]
            .as_array()
            .unwrap()
            .iter()
            .map(|item| {
                let mut data = item.clone();
                for (key, value) in [
                    ("name", serde_json::json!("local_fixture")),
                    ("tooltip", serde_json::json!([])),
                    ("stackLimit", serde_json::json!(64)),
                    ("durability", serde_json::json!(0)),
                    ("tools", serde_json::json!({})),
                    ("armor", serde_json::json!(false)),
                    ("tags", serde_json::json!([])),
                    ("icon", serde_json::Value::Null),
                    ("order", serde_json::Value::Null),
                    ("aspects", serde_json::Value::Null),
                ] {
                    data[key] = value;
                }
                serde_json::from_value(data).unwrap()
            })
            .collect();
        (rows, items)
    }
    fn rebind(rows: &mut [GeneticReferenceRow]) {
        let data: Vec<_> = rows.iter().map(|row| &row.data).collect();
        let domain = super::super::value_id(
            "reference",
            &serde_json::json!({"kind":"nativeStructures","records":data}),
        )
        .unwrap();
        for row in rows {
            row.domain = domain.clone();
            row.id = row.content_id().unwrap();
        }
    }
    #[test]
    fn native_structure_target_wire_preserves_order_and_independent_policies() {
        let raw = serde_json::json!({
            "ordinal":0,"structure":"structure_fixture","controller":"item_fixture",
            "origin":{"owner":"fixture","handler":"fixture.Controller","key":"owned"},
            "hasData":true,"hasTierData":true,"maxTier":3,
            "channels":{"coil":3,"glass":3,"bounded":9,"custom":-7},
            "channelDefinitions":{
                "bounded":{"kind":"bounded","minimum":2,"maximum":9,
                    "validation":"inclusiveBounds","normalization":"clampToBounds"},
                "custom":{"kind":"custom","minimum":null,"maximum":null,
                    "validation":"opaqueCustom","normalization":"clampToBounds"}
            },
            "tierItems":[{"nativeHash":"31","items":[],"tier":-2,
                "channels":["glass","coil","not-scanned"]}]
        });
        let controller: Controller = serde_json::from_value(raw.clone()).unwrap();
        assert_eq!(serde_json::to_value(controller).unwrap(), raw);
        let mut legacy = raw;
        legacy["tierItems"][0]
            .as_object_mut()
            .unwrap()
            .remove("channels");
        legacy["tierItems"][0]["channel"] = serde_json::json!("coil");
        assert!(serde_json::from_value::<Controller>(legacy).is_err());
    }
    #[test]
    fn native_structure_java_rows_preserve_tiers_and_unresolved_lookup() {
        let (rows, items) = fixture();
        let facts = items.iter().map(|item| (item.id.as_str(), item)).collect();
        super::super::references::validate(&rows, &facts).unwrap();
        let ReferenceData::StructureCatalogDefinition(definition) = &rows[0].data else {
            panic!()
        };
        assert_eq!(definition.counts.unresolved_lookups, 1);
    }
    #[test]
    fn native_structure_definition_only_data_keeps_independent_channel_maxima() {
        let (mut rows, items) = fixture();
        let facts = items.iter().map(|item| (item.id.as_str(), item)).collect();
        let ReferenceData::StructureCatalogController(controller) = &mut rows[1].data else {
            panic!()
        };
        assert!(controller.has_data && controller.has_tier_data);
        assert_eq!(controller.channels["bounded"], 9);
        assert_eq!(
            controller.tier_items[0].channels,
            ["glass", "coil", "not-scanned"]
        );
        assert!(controller.tier_items[1].channels.is_empty());
        assert_eq!(controller.tier_items[1].tier, -2);
        controller.has_tier_data = false;
        controller.max_tier = 1;
        controller.tier_items.clear();
        let ReferenceData::StructureCatalogDefinition(definition) = &mut rows[0].data else {
            panic!()
        };
        definition.counts.tier_lookups = 0;
        rebind(&mut rows);
        super::super::references::validate(&rows, &facts).unwrap();
    }
    #[test]
    fn native_structure_channel_guards_reject_corruption_after_content_rebinding() {
        let (rows, items) = fixture();
        let facts = items.iter().map(|item| (item.id.as_str(), item)).collect();
        for case in 0..7 {
            let mut bad = rows.clone();
            let ReferenceData::StructureCatalogController(controller) = &mut bad[1].data else {
                panic!()
            };
            match case {
                0 => controller.tier_items[0].channels.push("glass".into()),
                1 => {
                    controller
                        .channel_definitions
                        .get_mut("bounded")
                        .unwrap()
                        .maximum = Some(8)
                }
                2 => controller.has_data = false,
                3 => {
                    controller
                        .channel_definitions
                        .get_mut("bounded")
                        .unwrap()
                        .minimum = Some(0)
                }
                4 => {
                    controller
                        .channel_definitions
                        .get_mut("unbounded")
                        .unwrap()
                        .minimum = Some(2)
                }
                5 => {
                    controller.tier_items[0].channels =
                        (0..257).map(|index| format!("channel{index}")).collect()
                }
                6 => controller.tier_items[0].channels = vec!["x".repeat(129)],
                _ => unreachable!(),
            }
            rebind(&mut bad);
            assert!(
                super::super::references::validate(&bad, &facts).is_err(),
                "case {case}"
            );
        }
    }
    #[test]
    fn native_structure_membership_counts_and_geometry_are_strict() {
        let (rows, items) = fixture();
        let facts = items.iter().map(|item| (item.id.as_str(), item)).collect();
        let mut bad = rows.clone();
        if let ReferenceData::StructureComponentLookup(lookup) = &mut bad[2].data {
            lookup.structures[0] = "structure_wrong".into();
        }
        rebind(&mut bad);
        assert!(super::super::references::validate(&bad, &facts).is_err());
        let mut bad = rows.clone();
        if let ReferenceData::StructureCatalogDefinition(definition) = &mut bad[0].data {
            definition.counts.unresolved_lookups = 0;
        }
        rebind(&mut bad);
        assert!(super::super::references::validate(&bad, &facts).is_err());
        let source = crate::source::Source::open(
            &std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../contracts/fixtures/source-semantic-40"),
        )
        .unwrap();
        let mut domain = Domain::load(&source).unwrap();
        domain.structures.clear();
        domain.references = rows;
        assert!(validate_facts(&domain).is_err());
    }
}
