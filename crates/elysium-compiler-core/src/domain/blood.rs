#[cfg(test)]
mod tests {
    use crate::domain::Process;
    use serde_json::{json, Value};

    fn altar() -> Value {
        json!({"kind":"bloodmagic-altar","nativeIndex":0,"tier":3,"lp":500,"consumptionRate":20,"drainRate":4,
            "canBeFilled":false,"inputCount":"whole-stack","selection":"first-matching-registry-entry","earlier":[],
            "nbt":"native-presence-no-compound-at-required-keys","nbtKeys":["native"],"result":"stack-conversion",
            "serverWorld":true,"startRequiresPositiveTank":true,"numeric":"java-int-float-operations","recheckResultOnCompletion":true,
            "upgrades":{"speedPerRune":0.25,"decayPerSpeedRune":0.8,"baseCapacity":10000,"betterCapacityBase":1.14,"capacityPerRune":0.35,
                "bufferBaseCapacity":1000,"displacementBase":1.5,"orbCapacityPerRune":0.04,"quicknessBaseDelay":100,"quicknessDecayPerRune":0.8,
                "quicknessMinimumDelay":1,"rateRule":"tier-below-2-base-else-int-base-times-1-plus-0.25-speedRunes",
                "drainRule":"int-base-drain-times-float-0.8-power-speedRunes","quicknessRule":"max-1-floor-100-times-0.8-power-quicknessRunes",
                "acceleration":"native-buffer-transfer-tick-period","capacityRule":"int-10000-times-float-1.14-power-betterRunes-plus-0.35-capacityRunes"}})
    }
    fn binding() -> Value {
        json!({"kind":"bloodmagic-binding","ritual":"AW006Binding","activationCost":5000,"crystalLevel":2,"refreshCost":250,"cooldown":200,
            "components":[{"x":3,"y":0,"z":0,"stone":2}],"ownerNetwork":true,"activeRitual":true,"eventGate":true,"earlier":[],
            "structureRule":"native-valid-rotated-component-structure","activationGate":"native-crystal-level-and-owner-network-and-event",
            "nativeIndex":0,"selection":"first-matching-registry-entry","result":"world-entity-after-cooldown","itemBounds":[0,1,0,1,2,1],
            "state":"var1-is-selected-index-plus-one","cooldownRule":"decrement-per-affordable-refresh-then-spawn-at-nonpositive",
            "refreshDebit":"only-idle-scan-branch-including-no-match","networkGate":"refresh-cost-required-in-both-idle-and-pending-branches",
            "spawnOffset":[0.5,1.0,0.5]})
    }
    fn component(id: &str, weight: i32, default: bool) -> Value {
        json!({"id":id,"weight":weight,"required":[],"isBlock":true,"placementCallback":"net.minecraft.item.ItemBlock",
            "placementOutcome":"runtime-native-callback","default":default})
    }
    fn meteor() -> Value {
        let generation=json!({"shape":"integer-offsets-strict-sphere-radius-plus-half","eligibility":"air-after-optional-explosion",
            "poolChoice":"world-nextFloat-times-100-less-than-modified-fillerChance","componentChoice":"world-nextInt-totalWeight-cumulative-subtraction",
            "reagentPredicate":"any-required-reagent-identity-or-no-requirement","emptyPool":"native-default-fallback",
            "fillerOverride":"concatenate-active-reagent-fillers-in-active-reagent-order-unless-filtered-first-is-default",
            "radiusRule":"max-1-base-plus-largest-positive-and-most-negative-radiusChange",
            "fillerRule":"native-float-extrema-raw-adjustment-then-relative-increase-else-decrease-clamp-0-100",
            "explosion":"unless-any-active-disableExplosions-strength-modified-radius-times-4",
            "destroyRule":"invert-config-on-any-active-invertExplosionBlockDamage",
            "placement":"subtract-every-weight-skip-null-or-nonblock-then-call-native-placeBlockAt-stop-even-if-false",
            "placementSuccess":"runtime-world-and-native-block-callback-gate","jointCounts":"one-selected-block-per-eligible-site-not-independent-yields"});
        let mut result=json!({"kind":"bloodmagic-meteor","ritual":"AW019FallingTower","activationCost":15000,"crystalLevel":1,"refreshCost":0,"cooldown":0,
            "components":[{"x":2,"y":0,"z":0,"stone":2}],"ownerNetwork":true,"activeRitual":true,"eventGate":true,"earlier":[],
            "structureRule":"native-valid-rotated-component-structure","activationGate":"native-crystal-level-and-owner-network-and-event"});
        let impact=json!({
            "nativeIndex":1,"selection":"first-matching-registry-entry","cost":1000,"radius":2,"fillerChance":25.0,"result":"spatial-weighted-blocks",
            "itemBounds":[0,1,0,1,2,1],"reagentCost":1000,"ores":[component("item_ore",3,false),component("item_other",1,false)],
            "filler":[component("item_stone",1,true)],"fallback":component("item_stone",1,true),"reagents":[],"reagentIdentities":[],
            "destroyBlocks":false,"spawnY":257,"deactivateAfterSummon":true,
            "generation":generation})
        ;
        result.as_object_mut().unwrap().extend(impact.as_object().unwrap().clone());
        result
    }

    // These fail if the compiler drops the native gate or treats process as opaque JSON.
    #[test]
    fn altar_contract_retains_stack_nbt_and_upgrade_state() {
        let value = altar();
        let process: Process = serde_json::from_value(value.clone()).expect("typed native altar process must decode");
        assert_eq!(serde_json::to_value(process).unwrap(), value);
        for (path, bad) in [
            ("/tier",json!(-1)), ("/lp",json!("500")), ("/nbtKeys",json!("native")),
            ("/nbt",json!("exact-nbt")), ("/inputCount",json!("single-item")),
            ("/upgrades/rateRule",json!("unbounded-rate")),
        ] {
            let mut invalid=value.clone(); *invalid.pointer_mut(path).unwrap()=bad;
            assert!(serde_json::from_value::<Process>(invalid).is_err(), "{path}");
        }
        let mut extra=value; extra["guaranteedDuration"]=json!(25);
        assert!(serde_json::from_value::<Process>(extra).is_err());
    }
    #[test]
    fn binding_contract_retains_conditional_entity_and_refresh_branch() {
        let value=binding();
        let process: Process=serde_json::from_value(value.clone()).expect("typed native binding process must decode");
        assert_eq!(serde_json::to_value(process).unwrap(), value);
        for (path,bad) in [
            ("/activationCost",json!(-1)), ("/components/0/stone",json!("fire")),
            ("/result",json!("guaranteed-inventory-item")), ("/refreshDebit",json!("each-tick")),
            ("/itemBounds",json!([0,1,0,1,2])),
        ] {
            let mut invalid=value.clone(); *invalid.pointer_mut(path).unwrap()=bad;
            assert!(serde_json::from_value::<Process>(invalid).is_err(), "{path}");
        }
    }
    #[test]
    fn meteor_contract_retains_joint_spatial_pools_and_callback_gates() {
        let value=meteor();
        let process: Process=serde_json::from_value(value.clone()).expect("typed native meteor process must decode");
        assert_eq!(serde_json::to_value(process).unwrap(), value);
        for (path,bad) in [
            ("/cost",json!(-1)), ("/ores/0/weight",json!(1.5)), ("/reagentIdentities",json!({})),
            ("/generation/jointCounts",json!("independent-item-yields")), ("/generation/placementSuccess",json!("always-success")),
            ("/result",json!("fixed-ore-amounts")),
        ] {
            let mut invalid=value.clone(); *invalid.pointer_mut(path).unwrap()=bad;
            assert!(serde_json::from_value::<Process>(invalid).is_err(), "{path}");
        }
        let mut extra=value; extra["fixedYield"]=json!(18);
        assert!(serde_json::from_value::<Process>(extra).is_err());
    }
}
use super::{Consumption, Item, Kind, Match, OutputRole, Quantity, Recipe};
use crate::identity::integer;
use anyhow::{ensure, Result};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

// Named, closed native algorithms are schema data, never opaque JSON or user code.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct PriorPredicate {
    pub registry: String, pub meta: i32, pub metadata_wildcard: bool, pub tier: i32, pub tagged: bool, pub keys: Vec<String>,
}
fn prior(values:&[PriorPredicate],native_index:u32,altar:bool,meteor:bool)->Result<()> {
    ensure!(values.len()<=native_index as usize && native_index<65536,"Invalid native Blood registry priority");
    for value in values {
        ensure!(!value.registry.is_empty()&&value.registry.len()<=65535 && value.meta>=0
            && value.metadata_wildcard==(meteor&&value.meta==32767) && (altar||value.tier==0)
            && (altar||!value.tagged) && (value.tagged||value.keys.is_empty())
            && value.keys.len()<=65536 && value.keys.iter().all(|k|k.len()<=65535)
            && value.keys.windows(2).all(|k|k[0]<k[1]),"Invalid earlier native Blood predicate");
    }
    Ok(())
}
macro_rules! native_rule {
    ($name:ident, $token:literal) => {
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
        pub enum $name { #[serde(rename = $token)] Native }
    };
}
native_rule!(RegistrySelection, "first-matching-registry-entry");
native_rule!(WholeStack, "whole-stack");
native_rule!(JavaNumbers, "java-int-float-operations");
native_rule!(AltarRate, "tier-below-2-base-else-int-base-times-1-plus-0.25-speedRunes");
native_rule!(AltarDrain, "int-base-drain-times-float-0.8-power-speedRunes");
native_rule!(AltarQuickness, "max-1-floor-100-times-0.8-power-quicknessRunes");
native_rule!(AltarAcceleration, "native-buffer-transfer-tick-period");
native_rule!(AltarCapacity, "int-10000-times-float-1.14-power-betterRunes-plus-0.35-capacityRunes");
native_rule!(OrbOwnerTag, "ownerName");
native_rule!(KeptOrb, "keep");
native_rule!(OrbCapacity, "int-maxEssence-times-float-1-plus-0.04-orbCapacityRunes");
native_rule!(OrbDebit, "only-LP-accepted-by-network");
native_rule!(RitualStructure, "native-valid-rotated-component-structure");
native_rule!(RitualActivation, "native-crystal-level-and-owner-network-and-event");
native_rule!(BindingState, "var1-is-selected-index-plus-one");
native_rule!(BindingCooldown, "decrement-per-affordable-refresh-then-spawn-at-nonpositive");
native_rule!(BindingDebit, "only-idle-scan-branch-including-no-match");
native_rule!(BindingNetworkGate, "refresh-cost-required-in-both-idle-and-pending-branches");
native_rule!(BindingResult, "world-entity-after-cooldown");
native_rule!(MeteorResult, "spatial-weighted-blocks");
native_rule!(MeteorShape, "integer-offsets-strict-sphere-radius-plus-half");
native_rule!(MeteorEligibility, "air-after-optional-explosion");
native_rule!(MeteorPoolChoice, "world-nextFloat-times-100-less-than-modified-fillerChance");
native_rule!(MeteorComponentChoice, "world-nextInt-totalWeight-cumulative-subtraction");
native_rule!(MeteorReagentPredicate, "any-required-reagent-identity-or-no-requirement");
native_rule!(MeteorEmptyPool, "native-default-fallback");
native_rule!(MeteorFillerOverride, "concatenate-active-reagent-fillers-in-active-reagent-order-unless-filtered-first-is-default");
native_rule!(MeteorRadius, "max-1-base-plus-largest-positive-and-most-negative-radiusChange");
native_rule!(MeteorFillerChance, "native-float-extrema-raw-adjustment-then-relative-increase-else-decrease-clamp-0-100");
native_rule!(MeteorExplosion, "unless-any-active-disableExplosions-strength-modified-radius-times-4");
native_rule!(MeteorDestroy, "invert-config-on-any-active-invertExplosionBlockDamage");
native_rule!(MeteorPlacement, "subtract-every-weight-skip-null-or-nonblock-then-call-native-placeBlockAt-stop-even-if-false");
native_rule!(MeteorPlacementSuccess, "runtime-world-and-native-block-callback-gate");
native_rule!(MeteorJointCounts, "one-selected-block-per-eligible-site-not-independent-yields");
native_rule!(NativePlacementOutcome, "runtime-native-callback");

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub enum AltarNbt {
    #[serde(rename = "ignore")] Ignore,
    #[serde(rename = "native-presence-no-compound-at-required-keys")] Tagged,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub enum AltarResult {
    #[serde(rename = "stack-conversion")] Conversion,
    #[serde(rename = "soul-network-charge")] Charge,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub enum RitualKey {
    #[serde(rename = "AW006Binding")] Binding,
    #[serde(rename = "AW019FallingTower")] FallingTower,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub enum PlacementCallback {
    #[serde(rename = "net.minecraft.item.ItemBlock")] ItemBlock,
    /// Pinned 5.09.51.482 stores ore metadata in the TileEntity, then native impact marks it natural.
    #[serde(rename = "gregtech.common.blocks.ItemOres")] GregtechOres,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AltarUpgrades {
    pub speed_per_rune: f64, pub decay_per_speed_rune: f64, pub base_capacity: u32,
    pub better_capacity_base: f64, pub capacity_per_rune: f64, pub buffer_base_capacity: u32,
    pub displacement_base: f64, pub orb_capacity_per_rune: f64, pub quickness_base_delay: u32,
    pub quickness_decay_per_rune: f64, pub quickness_minimum_delay: u32,
    pub rate_rule: AltarRate, pub drain_rule: AltarDrain, pub quickness_rule: AltarQuickness,
    pub acceleration: AltarAcceleration, pub capacity_rule: AltarCapacity,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AltarOrb {
    pub max_essence: u32, pub free: bool, pub owner_tag: OrbOwnerTag, pub require_owner: bool,
    pub input: KeptOrb, pub capacity_rule: OrbCapacity, pub debit: OrbDebit,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AltarProcess {
    pub earlier: Vec<PriorPredicate>,
    pub native_index: u32, pub tier: u32, pub lp: u32, pub consumption_rate: u32, pub drain_rate: u32,
    pub can_be_filled: bool, pub input_count: WholeStack, pub selection: RegistrySelection,
    pub nbt: AltarNbt, pub nbt_keys: Vec<String>, pub result: AltarResult, pub server_world: bool,
    pub start_requires_positive_tank: bool, pub upgrades: AltarUpgrades, pub numeric: JavaNumbers,
    pub recheck_result_on_completion: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub orb: Option<AltarOrb>,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RitualComponent { pub x: i32, pub y: i32, pub z: i32, pub stone: u8 }
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BindingProcess {
    pub earlier: Vec<PriorPredicate>,
    pub ritual: RitualKey, pub activation_cost: u32, pub crystal_level: u32, pub refresh_cost: u32,
    pub cooldown: u32, pub components: Vec<RitualComponent>, pub owner_network: bool, pub active_ritual: bool,
    pub event_gate: bool, pub structure_rule: RitualStructure, pub activation_gate: RitualActivation,
    pub native_index: u32, pub selection: RegistrySelection, pub result: BindingResult,
    pub item_bounds: [i32; 6], pub state: BindingState, pub cooldown_rule: BindingCooldown,
    pub refresh_debit: BindingDebit, pub network_gate: BindingNetworkGate, pub spawn_offset: [f64; 3],
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MeteorComponent {
    pub id: Option<String>, pub weight: i32, pub required: Vec<String>, pub is_block: bool,
    pub placement_callback: Option<PlacementCallback>, pub placement_outcome: NativePlacementOutcome,
    #[serde(rename = "default")] pub is_default: bool,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ReagentIdentity { pub id: String, pub name: String }
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MeteorModifier {
    pub reagent: String, pub radius_change: i32, pub filler_chance_change: i32, pub raw_filler_chance_change: f64,
    pub disable_explosions: bool, pub invert_explosion_block_damage: bool, pub filler: Vec<MeteorComponent>,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MeteorGeneration {
    pub shape: MeteorShape, pub eligibility: MeteorEligibility, pub pool_choice: MeteorPoolChoice,
    pub component_choice: MeteorComponentChoice, pub reagent_predicate: MeteorReagentPredicate,
    pub empty_pool: MeteorEmptyPool, pub filler_override: MeteorFillerOverride, pub radius_rule: MeteorRadius,
    pub filler_rule: MeteorFillerChance, pub explosion: MeteorExplosion, pub destroy_rule: MeteorDestroy,
    pub placement: MeteorPlacement, pub placement_success: MeteorPlacementSuccess, pub joint_counts: MeteorJointCounts,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MeteorProcess {
    pub earlier: Vec<PriorPredicate>,
    pub ritual: RitualKey, pub activation_cost: u32, pub crystal_level: u32, pub refresh_cost: u32, pub cooldown: u32,
    pub components: Vec<RitualComponent>, pub owner_network: bool, pub active_ritual: bool, pub event_gate: bool,
    pub structure_rule: RitualStructure, pub activation_gate: RitualActivation, pub native_index: u32,
    pub selection: RegistrySelection, pub cost: u32, pub radius: u32, pub filler_chance: f64, pub result: MeteorResult,
    pub item_bounds: [i32; 6], pub reagent_cost: u32, pub ores: Vec<MeteorComponent>, pub filler: Vec<MeteorComponent>,
    pub fallback: MeteorComponent, pub reagents: Vec<MeteorModifier>, pub reagent_identities: Vec<ReagentIdentity>,
    pub destroy_blocks: bool, pub spawn_y: u32, pub deactivate_after_summon: bool, pub generation: MeteorGeneration,
}

pub(super) fn validate_altar(process: &AltarProcess, recipe: &Recipe, items: &BTreeMap<&str, &Item>) -> Result<()> {
    prior(&process.earlier,process.native_index,true,false)?;
    common(recipe, "NEIAltarRecipeHandler", items)?;
    ensure!(process.native_index < 65536 && process.tier <= i32::MAX as u32 && process.lp <= i32::MAX as u32
        && process.consumption_rate <= i32::MAX as u32 && process.drain_rate <= i32::MAX as u32,
        "altar parameters exceed native signed int domain");
    ensure!(process.server_world && process.start_requires_positive_tank && process.recheck_result_on_completion,
        "altar runtime world/tank/completion gates are required");
    ensure!(process.nbt_keys.len() <= 65536 && process.nbt_keys.iter().all(|key| key.len() <= 65535)
        && process.nbt_keys.windows(2).all(|keys| keys[0] < keys[1]), "invalid altar NBT key predicate");
    ensure!(process.nbt == AltarNbt::Tagged || process.nbt_keys.is_empty(), "untagged altar cannot require NBT keys");
    let choice=&recipe.inputs[0].choices[0];
    ensure!(matches!(&choice.rule, Match::BloodAltar { tagged, keys }
        if *tagged == (process.nbt == AltarNbt::Tagged) && keys == &process.nbt_keys), "altar ingredient must retain native NBT matcher");
    let upgrades=&process.upgrades;
    ensure!(upgrades.speed_per_rune == 0.25 && upgrades.decay_per_speed_rune == 0.8 && upgrades.base_capacity == 10000
        && upgrades.better_capacity_base == 1.14 && upgrades.capacity_per_rune == 0.35 && upgrades.buffer_base_capacity == 1000
        && upgrades.displacement_base == 1.5 && upgrades.orb_capacity_per_rune == 0.04 && upgrades.quickness_base_delay == 100
        && upgrades.quickness_decay_per_rune == 0.8 && upgrades.quickness_minimum_delay == 1, "altar native rune formulas changed");
    if process.can_be_filled {
        let orb=process.orb.as_ref().ok_or_else(|| anyhow::anyhow!("orb charging lacks native orb parameters"))?;
        ensure!(process.result == AltarResult::Charge && matches!(choice.consume, Consumption::Keep)
            && recipe.outputs.is_empty() && !orb.free && orb.require_owner && orb.max_essence <= i32::MAX as u32,
            "orb charging cannot become a crafted item or bypass owner/capacity gate");
    } else {
        ensure!(process.result == AltarResult::Conversion && process.orb.is_none() && matches!(choice.consume, Consumption::Stack),
            "altar conversion consumes the complete offered stack");
        ensure!(recipe.outputs.len() == 1, "altar conversion requires one scaled result");
        let output=&recipe.outputs[0];
        ensure!(output.kind == Kind::Item && items.contains_key(output.id.as_str()) && output.amount.is_none()
            && output.change.is_none() && matches!(output.role, OutputRole::Result)
            && output.chance.numerator == "1" && output.chance.denominator == "1", "invalid whole-stack altar output");
        if let Some(Quantity::BloodAltar { nominal })=&output.quantity { integer(nominal,1,i32::MAX as i64)?; }
        else { anyhow::bail!("altar result needs native whole-stack quantity"); }
    }
    Ok(())
}

pub(super) fn validate_binding(process: &BindingProcess, recipe: &Recipe, items: &BTreeMap<&str, &Item>) -> Result<()> {
    prior(&process.earlier,process.native_index,false,false)?;
    common(recipe,"NEIBindingRitualHandler",items)?;
    ritual(&process.components,process.activation_cost,process.crystal_level,process.owner_network,process.active_ritual,process.event_gate)?;
    ensure!(process.ritual == RitualKey::Binding && process.cooldown == 200 && process.refresh_cost <= i32::MAX as u32
        && process.native_index < 65536 && process.item_bounds == [0,1,0,1,2,1] && process.spawn_offset == [0.5,1.0,0.5],
        "invalid native binding cooldown, bounds or cost");
    let choice=&recipe.inputs[0].choices[0];
    ensure!(matches!(choice.consume,Consumption::Consume) && matches!(choice.rule,Match::Wildcard {meta:false,nbt:true}),
        "binding consumes one exact-metadata item and ignores NBT");
    fixed_result(recipe,items)?;
    Ok(())
}

pub(super) fn validate_meteor(process: &MeteorProcess, recipe: &Recipe, items: &BTreeMap<&str, &Item>) -> Result<()> {
    prior(&process.earlier,process.native_index,false,true)?;
    common(recipe,"NEIMeteorRecipeHandler",items)?;
    ritual(&process.components,process.activation_cost,process.crystal_level,process.owner_network,process.active_ritual,process.event_gate)?;
    ensure!(process.ritual == RitualKey::FallingTower && process.refresh_cost == 0 && process.cooldown == 0 && process.native_index < 65536
        && process.cost <= i32::MAX as u32 && process.radius >= 1 && process.radius <= i32::MAX as u32
        && process.filler_chance.is_finite() && (0.0..=100.0).contains(&process.filler_chance) && process.item_bounds == [0,1,0,1,2,1]
        && process.reagent_cost == 1000 && process.spawn_y == 257 && process.deactivate_after_summon,
        "invalid native meteor world/summon parameters");
    let choice=&recipe.inputs[0].choices[0];
    let focus=items.get(choice.id.as_str()).unwrap();
    ensure!(matches!(choice.consume,Consumption::Consume) && matches!(choice.rule,Match::Wildcard {meta,nbt:true} if meta == (focus.meta == 32767))
        && recipe.outputs.is_empty(), "meteor requires native focus matcher and spatial pools, not fixed outputs");
    ensure!(process.reagent_identities.len() <= 65536 && process.reagents.len() <= 65536, "meteor reagent budget exceeded");
    let mut identities=BTreeSet::new();
    for reagent in &process.reagent_identities {
        ensure!(!reagent.id.is_empty() && reagent.id.len() <= 128 && reagent.name.len() <= 65535 && identities.insert(reagent.id.as_str()),
            "invalid or duplicated meteor reagent identity");
    }
    component(&process.fallback,items,&identities)?;
    ensure!(process.fallback.is_default && process.fallback.is_block && process.fallback.weight > 0,
        "meteor requires valid native default fallback block");
    pool(&process.ores,items,&identities)?; pool(&process.filler,items,&identities)?;
    let mut modifiers=BTreeSet::new();
    for modifier in &process.reagents {
        ensure!(identities.contains(modifier.reagent.as_str()) && modifiers.insert(modifier.reagent.as_str())
            && modifier.raw_filler_chance_change.is_finite(), "invalid meteor reagent modifier reference or numeric field");
        pool(&modifier.filler,items,&identities)?;
    }
    Ok(())
}

fn common(recipe: &Recipe, handler: &str, items: &BTreeMap<&str, &Item>) -> Result<()> {
    ensure!(recipe.source.handler == format!("WayofTime.alchemicalWizardry.client.nei.{handler}")
        && recipe.duration.is_none() && recipe.energy.is_none() && recipe.grid.is_none() && recipe.magic.is_none(),
        "Blood process must retain native handler and conditional world duration");
    ensure!(recipe.inputs.len() == 1 && recipe.inputs[0].kind == Kind::Item && recipe.inputs[0].slot == 0
        && recipe.inputs[0].choices.len() == 1, "Blood ritual requires one native input slot and predicate");
    let choice=&recipe.inputs[0].choices[0];
    ensure!(choice.amount == "1" && choice.returns.is_empty() && items.contains_key(choice.id.as_str()), "invalid native Blood unit input");
    Ok(())
}
fn ritual(components: &[RitualComponent], cost:u32, level:u32, owner:bool, active:bool, event:bool) -> Result<()> {
    ensure!(cost <= i32::MAX as u32 && level <= i32::MAX as u32 && owner && active && event
        && !components.is_empty() && components.len() <= 4096, "native ritual activation/world gates missing");
    let mut positions=BTreeSet::new();
    for component in components { ensure!(component.stone <= 6 && positions.insert((component.x,component.y,component.z)), "invalid ritual component geometry"); }
    Ok(())
}
fn fixed_result(recipe:&Recipe,items:&BTreeMap<&str,&Item>) -> Result<()> {
    ensure!(recipe.outputs.len() == 1, "binding requires a single conditional result entity");
    let output=&recipe.outputs[0];
    ensure!(output.kind == Kind::Item && output.slot == 0 && items.contains_key(output.id.as_str()) && output.quantity.is_none()
        && output.change.is_none() && matches!(output.role,OutputRole::Result) && output.chance.numerator == "1" && output.chance.denominator == "1",
        "invalid native binding result");
    integer(output.amount.as_deref().unwrap_or(""),1,i32::MAX as i64)?;
    Ok(())
}
fn pool(pool:&[MeteorComponent],items:&BTreeMap<&str,&Item>,identities:&BTreeSet<&str>) -> Result<()> {
    ensure!(pool.len() <= 65536,"meteor pool budget exceeded");
    // Preserve signed component weights and their order. Native Java sum/nextInt failures are not silently normalized.
    for value in pool { component(value,items,identities)?; }
    Ok(())
}
fn component(component:&MeteorComponent,items:&BTreeMap<&str,&Item>,identities:&BTreeSet<&str>) -> Result<()> {
    ensure!(component.id.as_deref().is_none_or(|id| items.contains_key(id)) && component.required.len() <= 65536
        && component.required.iter().all(|id| identities.contains(id.as_str())), "meteor component has missing item or reagent identity");
    ensure!(component.is_block == component.placement_callback.is_some() && (!component.is_block || component.id.is_some()),
        "meteor block placement callback is missing or attached to a nonblock");
    Ok(())
}
