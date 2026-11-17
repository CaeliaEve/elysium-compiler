//! Manual EnderIO enchanter: count-selected level, signed XP and whole book-slot clearing.
use super::{Consumption, Item, Kind, Match, OutputRole, Quantity, Recipe};
use crate::identity::{integer, Nbt};
use anyhow::{ensure, Context, Result};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct EnchanterProcess {
    pub version: Version,
    pub level: u16,
    pub max_level: u16,
    pub items_per_level: u32,
    pub cost: i32,
    pub base_cost: i32,
    pub cost_per_level: i32,
    pub enchantment: i32,
    pub offered_minimum: String,
    pub offered_maximum: String,
    pub book_policy: BookPolicy,
    pub material_policy: MaterialPolicy,
    pub pickup_policy: PickupPolicy,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub enum Version {
    #[serde(rename = "2.9.28")]
    Installed,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub enum BookPolicy {
    #[serde(rename = "runtimePresenceClearWholeSlot")]
    Native,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub enum MaterialPolicy {
    #[serde(rename = "nativeCountDeterminesLevelDebitDivisorTimesLevelNoContainers")]
    Native,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub enum PickupPolicy {
    #[serde(rename = "creativeBypassXpElseSignedLevelDebit")]
    Native,
}
pub(super) fn validate(
    p: &EnchanterProcess,
    r: &Recipe,
    items: &BTreeMap<&str, &Item>,
) -> Result<()> {
    ensure!(
        r.source.owner == "EnderIO"
            && r.source.handler == "crazypants.enderio.nei.EnchanterRecipeHandler"
            && p.level > 0
            && p.level <= p.max_level
            && p.max_level <= 32767
            && p.items_per_level > 0
            && p.items_per_level <= i32::MAX as u32
            && p.cost_per_level >= 0
            && p.enchantment >= 0,
        "Invalid native enchanter registration or level law"
    );
    let minimum = u64::from(p.items_per_level) * u64::from(p.level);
    let maximum = if p.level == p.max_level {
        i32::MAX as u64
    } else {
        (u64::from(p.items_per_level) * (u64::from(p.level) + 1) - 1).min(i32::MAX as u64)
    };
    ensure!(
        minimum <= i32::MAX as u64
            && integer(&p.offered_minimum, 1, i32::MAX as i64)? == minimum as i64
            && integer(&p.offered_maximum, 1, i32::MAX as i64)? == maximum as i64
            && p.cost
                == p.base_cost.wrapping_add(
                    p.cost_per_level
                        .wrapping_mul(i32::from(p.level))
                        .wrapping_mul(i32::from(p.level))
                ),
        "Enchanter count interval or signed XP changed"
    );
    ensure!(
        r.duration.is_none()
            && r.energy.is_none()
            && r.grid.is_none()
            && r.magic.is_none()
            && r.properties.is_empty()
            && r.inputs.len() == 2
            && r.outputs.len() == 1
            && r.notes.len() == 1
            && r.notes[0].key == "enderio:enchanter"
            && r.notes[0].source
                == "crazypants.enderio.machine.enchanter.ContainerEnchanter$3.onPickupFromSlot",
        "Enchanter pickup/source semantics missing"
    );
    for (slot, input) in r.inputs.iter().enumerate() {
        ensure!(
            input.kind == Kind::Item && input.slot == slot as u32 && !input.choices.is_empty(),
            "Enchanter physical slots changed"
        );
        for choice in &input.choices {
            ensure!(
                matches!(choice.consume, Consumption::Allocated)
                    && choice.returns.is_empty()
                    && choice.amount == if slot == 0 { "1" } else { &p.offered_minimum },
                "Enchanter conditional slot debit changed"
            );
            if slot == 0 {
                let item = items
                    .get(choice.id.as_str())
                    .context("Missing book-slot sample")?;
                ensure!(
                    input.choices.len() == 1
                        && item.registry == "minecraft:writable_book"
                        && item.meta == 0
                        && item.nbt.is_none()
                        && matches!(choice.rule, Match::EnchanterBook),
                    "Enchanter normal insertion sample became its runtime predicate"
                );
            } else {
                let base = if let Match::Except { base, .. } = &choice.rule {
                    base.as_ref()
                } else {
                    &choice.rule
                };
                let primitive = |rule: &Match| {
                    matches!(
                        rule,
                        Match::Wildcard { nbt: true, .. } | Match::NativeMetadataKey { .. }
                    )
                };
                ensure!(
                    primitive(base),
                    "Enchanter material is not native metadata/ignored NBT"
                );
                if let Match::Except { exclude, .. } = &choice.rule {
                    ensure!(
                        exclude.iter().all(|prior| primitive(&prior.rule)),
                        "Enchanter acquired an unknown priority filter"
                    );
                }
                ensure!(
                    choice
                        .rule
                        .indexes_example(items[choice.id.as_str()], items),
                    "Enchanter example is shadowed"
                );
            }
        }
    }
    let output = &r.outputs[0];
    ensure!(
        output.kind == Kind::Item
            && output.slot == 0
            && matches!(output.role, OutputRole::Result)
            && output.amount.is_none()
            && output.change.is_none()
            && output.chance.numerator == "1"
            && output.chance.denominator == "1"
            && matches!(&output.quantity,Some(Quantity::Potential{stat,condition,nominal,sample,parameters})if stat=="enderio:enchanterPickup"
            &&condition.as_deref()==Some("nativeEnchanterLevelAndPickup")&&nominal=="1"&&sample.is_none()&&parameters.is_none()),
        "Enchanter output lost its pickup condition"
    );
    let book = items
        .get(output.id.as_str())
        .context("Missing enchanted book")?;
    ensure!(
        book.registry == "minecraft:enchanted_book" && book.meta == 0,
        "Invalid enchanter output item"
    );
    let Some(Nbt::Compound { value: root }) = &book.nbt else {
        anyhow::bail!("Enchanted book needs native NBT")
    };
    let Some(Nbt::List { element, value }) = root.get("StoredEnchantments") else {
        anyhow::bail!("Native enchantment list missing")
    };
    ensure!(
        root.len() == 1 && element == "compound" && value.len() == 1,
        "Invalid native enchantment list"
    );
    let Nbt::Compound { value } = &value[0] else {
        anyhow::bail!("Invalid native enchantment compound")
    };
    ensure!(
        value.len() == 2
            && matches!(value.get("id"),Some(Nbt::Short{value})if value==&(p.enchantment as i16).to_string())
            && matches!(value.get("lvl"),Some(Nbt::Short{value})if value==&p.level.to_string()),
        "Enchanter result id/level differs from source"
    );
    Ok(())
}
