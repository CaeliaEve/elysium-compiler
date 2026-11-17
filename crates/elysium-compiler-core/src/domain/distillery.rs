//! Witchery 0.24.1 distillery: ordered matching, signed jar debit and per-slot placement.
use super::{Consumption, Item, Kind, Match, OutputRole, Quantity, Recipe};
use anyhow::{ensure, Context, Result};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

macro_rules! native_enum {($name:ident,$($variant:ident=>$value:literal),+)=>{
    #[derive(Clone,Copy,Debug,PartialEq,Eq,Serialize,Deserialize,JsonSchema)]
    pub enum $name {$(#[serde(rename=$value)]$variant),+}
};}
native_enum!(DistilleryMetadata,Exact=>"exact",Ignored=>"ignored");
native_enum!(DistilleryMatching,Native=>"subtypeMetadataIgnoreCountAndTagsEitherOrder");
native_enum!(DistilleryJarPolicy,Native=>"runtimeCountOnlyClayJarAdmission");
native_enum!(DistilleryDebit,Native=>"oneEachOccupiedSignedJarDeltaNoContainers");
native_enum!(DistilleryOutputPolicy,Native=>"emptyCopyMatchingMergeKeepsTagsMismatchedSkipThenDebit");
native_enum!(DistilleryWorkPolicy,Native=>"successTicksPauseOnPowerResetBlockedCarryRecipeChanges");
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DistilleryReagent {
    pub item: String,
    pub metadata: DistilleryMetadata,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DistilleryPrototype {
    pub id: String,
    pub amount: String,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DistilleryAdmission {
    pub reagents: [Option<DistilleryReagent>; 2],
    pub jars: i32,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DistilleryProcess {
    pub native_tile: String,
    pub native_index: u32,
    pub native_count: u32,
    pub reagents: [Option<DistilleryReagent>; 2],
    pub jars: i32,
    pub jar: String,
    pub outputs: [Option<DistilleryPrototype>; 4],
    pub earlier: Vec<DistilleryAdmission>,
    pub matching: DistilleryMatching,
    pub jar_policy: DistilleryJarPolicy,
    pub debit: DistilleryDebit,
    pub output_policy: DistilleryOutputPolicy,
    pub work_policy: DistilleryWorkPolicy,
    pub cook_ticks: u32,
    pub power_per_work_tick_bits: String,
}
pub(super) fn validate(
    p: &DistilleryProcess,
    r: &Recipe,
    items: &BTreeMap<&str, &Item>,
) -> Result<()> {
    ensure!(
        r.source.owner == "WitcheryExtras"
            && r.source.handler == "alkalus.main.nei.NEI_Handler_Distillery"
            && p.native_tile == "com.emoniph.witchery.blocks.BlockDistillery$TileEntityDistillery"
            && p.native_count > 0
            && p.native_count <= 65536
            && p.native_index < p.native_count
            && p.earlier.len() == p.native_index as usize
            && (p.native_count as u64).pow(2) <= 4194304
            && p.reagents.iter().any(Option::is_some)
            && p.cook_ticks == 800
            && p.power_per_work_tick_bits == "3f19999a",
        "Distillery native source, work or allocation changed"
    );
    ensure!(
        r.duration.is_none()
            && r.energy.is_none()
            && r.grid.is_none()
            && r.magic.is_none()
            && r.properties.is_empty()
            && r.notes.len() == 1
            && r.notes[0].key == "witchery:distillery"
            && r.notes[0].source == format!("{}.smeltItem", p.native_tile),
        "Distillery cannot become fixed-cost crafting"
    );
    let check_reagents = |reagents: &[Option<DistilleryReagent>; 2]| -> Result<()> {
        for reagent in reagents.iter().flatten() {
            let item = items
                .get(reagent.item.as_str())
                .context("Missing distillery reagent")?;
            ensure!(
                item.nbt.is_none()
                    && item.meta >= 0
                    && (reagent.metadata != DistilleryMetadata::Ignored || item.meta == 0),
                "Invalid distillery input matcher witness"
            );
        }
        Ok(())
    };
    check_reagents(&p.reagents)?;
    for earlier in &p.earlier {
        check_reagents(&earlier.reagents)?;
    }
    let jar = items
        .get(p.jar.as_str())
        .context("Missing native clay jar")?;
    ensure!(
        jar.nbt.is_none() && jar.meta >= 0,
        "Invalid distillery jar witness"
    );
    ensure!(
        r.inputs.len() == p.reagents.iter().flatten().count() + usize::from(p.jars != 0),
        "Distillery lost required empty slot or jar threshold"
    );
    for input in &r.inputs {
        ensure!(
            input.kind == Kind::Item && input.choices.len() == 1,
            "Distillery input changed"
        );
        let choice = &input.choices[0];
        ensure!(
            matches!(choice.consume, Consumption::Allocated) && choice.returns.is_empty(),
            "Distillery allocation acquired containers or fixed consumption"
        );
        if input.slot == 2 {
            ensure!(
                p.jars != 0
                    && choice.id == p.jar
                    && choice.amount == p.jars.max(1).to_string()
                    && matches!(choice.rule, Match::DistilleryJar),
                "Distillery jar admission became exact runtime matching"
            );
        } else {
            let reagent = p
                .reagents
                .get(input.slot as usize)
                .and_then(Option::as_ref)
                .context("Missing distillery reagent slot")?;
            ensure!(
                choice.id == reagent.item
                    && choice.amount == "1"
                    && matches!(choice.rule,Match::Wildcard{meta,nbt:true} if meta==(reagent.metadata==DistilleryMetadata::Ignored)),
                "Distillery reagent matcher/debit changed"
            );
        }
    }
    for prototype in p.outputs.iter().flatten() {
        let count: i32 = prototype.amount.parse()?;
        ensure!(
            count.to_string() == prototype.amount && items.contains_key(prototype.id.as_str()),
            "Invalid native inventory prototype"
        );
    }
    ensure!(
        r.outputs.len()
            == p.outputs
                .iter()
                .flatten()
                .filter(|prototype| prototype.amount.parse::<i32>().is_ok_and(|count| count > 0))
                .count(),
        "Distillery sparse outputs changed"
    );
    for output in &r.outputs {
        let prototype = p
            .outputs
            .get(output.slot as usize)
            .and_then(Option::as_ref)
            .context("Unknown distillery output slot")?;
        let count: i32 = prototype.amount.parse()?;
        ensure!(
            count > 0
                && count.to_string() == prototype.amount
                && items.contains_key(prototype.id.as_str())
                && output.kind == Kind::Item
                && output.id == prototype.id
                && matches!(output.role, OutputRole::Result)
                && output.amount.is_none()
                && output.change.is_none()
                && output.chance.numerator == "1"
                && output.chance.denominator == "1"
                && matches!(&output.quantity,Some(Quantity::Potential{stat,condition,nominal,sample,parameters})
              if stat=="witchery:distilleryOutputState"&&condition.as_deref()==Some("nativeDistillerySlotAccepts")
                &&nominal==&prototype.amount&&sample.as_ref()==Some(nominal)&&parameters.is_none()),
            "Distillery output became guaranteed placement or fixed tags"
        );
    }
    Ok(())
}

/// Matching ignores stack count/tags, while the native jar predicate checks only count.
pub fn matches(
    reagents: &[Option<DistilleryReagent>; 2],
    jars: i32,
    offered: [Option<&Item>; 2],
    jar_count: Option<i32>,
    items: &BTreeMap<&str, &Item>,
) -> bool {
    if jars != 0 && jar_count.is_none_or(|count| count < jars) {
        return false;
    }
    let one = |actual: Option<&Item>, expected: Option<&DistilleryReagent>| match (actual, expected)
    {
        (None, None) => true,
        (Some(actual), Some(expected)) => items.get(expected.item.as_str()).is_some_and(|target| {
            target.registry == actual.registry
                && (expected.metadata == DistilleryMetadata::Ignored || target.meta == actual.meta)
        }),
        _ => false,
    };
    (one(offered[0], reagents[0].as_ref()) && one(offered[1], reagents[1].as_ref()))
        || (one(offered[0], reagents[1].as_ref()) && one(offered[1], reagents[0].as_ref()))
}
pub fn selected(
    p: &DistilleryProcess,
    offered: [Option<&Item>; 2],
    jar_count: Option<i32>,
    items: &BTreeMap<&str, &Item>,
) -> bool {
    offered.iter().any(Option::is_some)
        && matches(&p.reagents, p.jars, offered, jar_count, items)
        && !p
            .earlier
            .iter()
            .any(|prior| matches(&prior.reagents, prior.jars, offered, jar_count, items))
}

/// Empty slots copy the prototype; equal item/meta slots merge and preserve existing tags.
/// Mismatched slots do not block the operation: their output is lost while inputs still debit.
pub fn placed_count(
    existing: Option<(&Item, i32, i32)>,
    prototype: &Item,
    count: i32,
) -> Option<i32> {
    match existing {
        None => Some(count),
        Some((item, amount, maximum))
            if item.registry == prototype.registry && item.meta == prototype.meta =>
        {
            let sum = amount.wrapping_add(count);
            if sum > 64 || sum > maximum {
                None
            } else {
                Some(sum)
            }
        }
        Some((_, amount, _)) => Some(amount),
    }
}
