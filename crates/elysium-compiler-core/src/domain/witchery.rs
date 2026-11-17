//! Pinned Witchery 0.24.1 / WitcheryExtras 1.3.8 and Witching Gadgets 1.7.25 native transitions.
use super::{Consumption, Item, Kind, Match, OutputRole, Quantity, Recipe};
use crate::identity::{integer,Nbt};
use anyhow::{ensure, Result};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum JarMode { Absent, Conditional }

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum OvenByproduct { None, Foul, Exhale, Hint, Breath, Whiff, Reek, Odour }

/// Native callback distinguishes a missing Genome key from a present malformed
/// Genome. In particular an empty root compound still yields foul fume.
pub fn oven_nbt_product(nbt: Option<&Nbt>) -> OvenByproduct {
    let Some(Nbt::Compound {value:root})=nbt else {return OvenByproduct::Foul;};
    let Some(genome)=root.get("Genome") else {return OvenByproduct::Foul;};
    let Nbt::Compound {value:genome}=genome else {return OvenByproduct::None;};
    let Some(Nbt::List {value:chromosomes,..})=genome.get("Chromosomes") else {return OvenByproduct::None;};
    let Some(Nbt::Compound {value:first})=chromosomes.first() else {return OvenByproduct::None;};
    match first.get("UID0") {
        Some(Nbt::String {value}) if value=="forestry.treeOak"=>OvenByproduct::Exhale,
        Some(Nbt::String {value}) if value=="forestry.treeSpruce"=>OvenByproduct::Hint,
        Some(Nbt::String {value}) if value=="forestry.treeBirch"=>OvenByproduct::Breath,
        _=>OvenByproduct::None,
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct OvenPrior {
    pub registry: Option<String>,
    pub meta: i32,
    pub subtypes: bool,
    pub jar_threshold: u32,
    pub ore_key: Option<String>,
}
pub fn oven_prior_matches(prior: &OvenPrior, item: &Item, jars: u32) -> bool {
    if prior.jar_threshold == 0 { return true; }
    let literal = prior.registry.as_deref() == Some(item.registry.as_str())
        && (!prior.subtypes || prior.meta == item.meta) && jars >= prior.jar_threshold;
    let ore = prior.ore_key.as_deref().is_some_and(|key| !key.is_empty()
        && item.tags.iter().any(|tag| tag == key));
    literal || ore
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct OvenProcess {
    pub selector: u32,
    pub jar_threshold: u32,
    pub jar_mode: JarMode,
    pub byproduct: OvenByproduct,
    pub byproduct_item: Option<String>,
    pub nbt_classified: bool,
    pub filtered_double: bool,
    pub base_ticks: u32,
    pub ticks_per_funnel: u32,
    pub maximum_funnels: u32,
    pub base_chance: f64,
    pub normal_funnel_chance: f64,
    pub filtered_funnel_chance: f64,
    pub earlier: Vec<OvenPrior>,
}

/// A successful byproduct roll removes one jar only when the side slot accepts it.
/// The patched oven uses the source table merely to admit smelting; the main result
/// comes from FurnaceRecipes.getSmeltingResult and the byproduct comes from the
/// original Witchery TileEntityWitchesOven.generateByProduct callback.
pub fn oven_jar_debit(process: &OvenProcess, roll: f64, normal: u32, filtered: u32,
                      side_accepts: bool) -> u32 {
    if process.jar_mode == JarMode::Absent || process.byproduct == OvenByproduct::None || !side_accepts { return 0; }
    let chance = (process.base_chance + f64::from(normal) * process.normal_funnel_chance
        + f64::from(filtered) * process.filtered_funnel_chance).min(1.0);
    if roll <= chance { 1 } else { 0 }
}
pub fn oven_ticks(process: &OvenProcess, funnels: u32) -> u32 {
    process.base_ticks - process.ticks_per_funnel * funnels.min(process.maximum_funnels)
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct KettleProcess {
    pub selector: u32,
    pub power: f32,
    pub dimension: i32,
    pub familiar_type: i32,
    pub base_output: u32,
    pub hat_bonus: u32,
    pub water_millibuckets: u32,
    pub bottles_per_item: u32,
}
pub fn kettle_output(process: &KettleProcess, witch_hat: bool) -> u32 {
    process.base_output + if witch_hat { process.hat_bonus } else { 0 }
}
pub fn kettle_bottle_debit(extracted: u32) -> u32 { extracted }

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RitualProcess {
    pub action_index: u32,
    pub expanded_index: u32,
    pub power: i32,
    pub water_millibuckets: u32,
    /// Earlier original recipes in this action, after expansion to item/meta keys.
    pub earlier: Vec<Vec<String>>,
    /// Current action-list item/meta keys. Order is immaterial, multiplicity matters.
    pub keys: Vec<String>,
}
pub fn ritual_first_match<'a>(patterns: impl IntoIterator<Item = &'a [String]>,
                              offered: &[String]) -> Option<usize> {
    let mut offered = offered.to_vec(); offered.sort();
    patterns.into_iter().enumerate().find_map(|(index, pattern)| {
        let mut pattern = pattern.to_vec(); pattern.sort();
        if pattern.iter().all(|key| offered.iter().filter(|item| *item == key).count()
            >= pattern.iter().filter(|item| *item == key).count()) { Some(index) } else { None }
    })
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum InfernalInput { Stack, Ore }
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct InfernalPrior {
    pub input: InfernalInput,
    pub ore_key: Option<String>,
    pub registry: Option<String>,
    pub meta: i32,
    pub meta_wildcard: bool,
    pub input_count: u32,
}
pub fn infernal_prior_matches(prior: &InfernalPrior, item: &Item, offered: u32) -> bool {
    if offered < prior.input_count { return false; }
    match prior.input {
        InfernalInput::Stack => prior.registry.as_deref() == Some(item.registry.as_str())
            && (prior.meta_wildcard || prior.meta == item.meta),
        InfernalInput::Ore => prior.ore_key.as_deref().is_some_and(|key|
            item.tags.iter().any(|tag| tag == key)),
    }
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct InfernalProcess {
    pub selector: u32,
    pub input: InfernalInput,
    pub ore_key: Option<String>,
    pub input_count: u32,
    pub base_ticks: u32,
    pub special_fuel: bool,
    pub bonus: bool,
    pub maximum_bellows: u32,
    pub no_bellows_chance: f64,
    pub per_bellows_chance: f64,
    pub ticks_per_bellows: u32,
    pub vis_divisor: u32,
    pub earlier: Vec<InfernalPrior>,
}
pub fn infernal_ticks(process: &InfernalProcess, bellows: u32, vis_boost: bool) -> u32 {
    let base = process.base_ticks / if vis_boost { process.vis_divisor } else { 1 };
    base.saturating_sub(process.ticks_per_bellows * bellows.min(process.maximum_bellows)).max(1)
}
pub fn infernal_bonus(process: &InfernalProcess, bellows: u32, rolls: &[f64]) -> u32 {
    if !process.bonus { return 0; }
    if bellows == 0 { return u32::from(rolls.first().is_some_and(|roll| *roll < process.no_bellows_chance)); }
    rolls.iter().take(bellows.min(process.maximum_bellows) as usize)
        .filter(|roll| **roll < process.per_bellows_chance).count() as u32
}

fn fixed_result(recipe: &Recipe, slot: u32) -> Result<()> {
    let output = recipe.outputs.iter().find(|output| output.slot == slot)
        .ok_or_else(|| anyhow::anyhow!("missing Witchery result slot {slot}"))?;
    ensure!(output.kind == Kind::Item && matches!(output.role, OutputRole::Result)
        && output.change.is_none() && output.quantity.is_none() && output.amount.is_some()
        && output.chance.numerator == "1" && output.chance.denominator == "1", "invalid fixed native result");
    integer(output.amount.as_deref().unwrap(), 1, i32::MAX.into())?;
    Ok(())
}
fn potential_result(recipe: &Recipe, slot: u32, stat: &str, nominal: u32, condition: &str) -> Result<()> {
    let output = recipe.outputs.iter().find(|output| output.slot == slot)
        .ok_or_else(|| anyhow::anyhow!("missing Witchery conditional result slot {slot}"))?;
    ensure!(output.kind == Kind::Item && matches!(output.role, OutputRole::Result)
        && output.amount.is_none() && output.change.is_none()
        && output.chance.numerator == "1" && output.chance.denominator == "1"
        && matches!(&output.quantity, Some(Quantity::Potential {stat:s, nominal:n, condition:c, ..})
            if s == stat && n == &nominal.to_string() && c.as_deref() == Some(condition)),
        "invalid conditional native result");
    Ok(())
}
pub fn validate_oven(p: &OvenProcess, recipe: &Recipe, items: &BTreeMap<&str, &Item>) -> Result<()> {
    ensure!(recipe.source.handler == "alkalus.main.nei.NEI_Handler_Oven" && recipe.duration.is_none()
        && recipe.energy.is_none() && recipe.grid.is_none() && recipe.magic.is_none(), "invalid oven handler/cost");
    ensure!(p.base_ticks == 180 && p.ticks_per_funnel == 20 && p.maximum_funnels == 3
        && p.base_chance == 0.3 && p.normal_funnel_chance == 0.25
        && p.filtered_funnel_chance == if p.filtered_double {0.8} else {0.3}, "oven callback constants changed");
    ensure!(p.jar_threshold <= 64 && recipe.inputs.len() == if p.jar_mode == JarMode::Conditional {2} else {1}
        && (p.jar_mode == JarMode::Conditional || p.jar_threshold == 0), "oven jar stock gate changed");
    ensure!(p.earlier.len() == p.selector as usize && p.earlier.iter().all(|prior|
        prior.jar_threshold <= 64 && prior.registry.as_deref().is_none_or(|name| !name.is_empty())
        && prior.ore_key.as_deref().is_none_or(|name| !name.is_empty())),
        "oven first-match registry mask changed");
    let input = &recipe.inputs[0];
    ensure!(input.kind == Kind::Item && input.slot == 0 && input.choices.len() == 1
        && input.choices[0].amount == "1" && matches!(input.choices[0].consume, Consumption::Consume)
        && match &input.choices[0].rule {
            Match::Wildcard {nbt:true,..}=>!p.nbt_classified,
            Match::WitcheryOvenNbt {product,..}=>p.nbt_classified && p.jar_mode==JarMode::Conditional && *product==p.byproduct,
            _=>false,
        }, "oven literal stack matcher or NBT partition changed");
    let offered = items.get(input.choices[0].id.as_str())
        .ok_or_else(|| anyhow::anyhow!("missing oven input identity"))?;
    ensure!(!p.earlier.iter().any(|prior| oven_prior_matches(prior, offered, p.jar_threshold)),
        "oven row is shadowed by an earlier native recipe");
    if p.jar_mode == JarMode::Conditional {
        let jar = &recipe.inputs[1];
        ensure!(jar.kind == Kind::Item && jar.slot == 1 && jar.choices.len() == 1
            && jar.choices[0].amount == p.jar_threshold.max(1).to_string()
            && matches!(jar.choices[0].consume, Consumption::Allocated), "oven conditional jar debit changed");
    }
    fixed_result(recipe, 0)?;
    let byproduct = p.byproduct != OvenByproduct::None && p.jar_mode == JarMode::Conditional;
    ensure!(recipe.outputs.len() == if byproduct {2} else {1}, "oven byproduct branch changed");
    if byproduct {
        potential_result(recipe, 1, "witchery:ovenByproduct", 1, "nativeJarRollAndOutputAccepts")?;
        ensure!(p.byproduct_item.as_deref()==Some(recipe.outputs[1].id.as_str()), "oven byproduct differs from captured native identity");
    } else {ensure!(p.byproduct_item.is_none() && p.byproduct==OvenByproduct::None,"oven has an unused byproduct definition");}
    Ok(())
}
pub fn validate_kettle(p: &KettleProcess, recipe: &Recipe, _items: &BTreeMap<&str, &Item>) -> Result<()> {
    ensure!(recipe.source.handler == "alkalus.main.nei.NEI_Handler_Kettle" && recipe.duration.is_none()
        && recipe.energy.is_none() && recipe.grid.is_none() && recipe.magic.is_none(), "invalid kettle handler/cost");
    ensure!(p.power.is_finite() && p.power >= 0.0 && p.water_millibuckets == 1000
        && p.bottles_per_item == 1 && p.base_output > 0, "invalid native kettle state");
    ensure!(recipe.inputs.len() >= 2 && recipe.inputs.last().unwrap().slot == 6
        && recipe.inputs.iter().any(|input| input.kind == Kind::Fluid && input.slot == 0
            && input.choices.len() == 1 && input.choices[0].amount == "1000")
        && recipe.inputs.last().unwrap().choices.len() == 1
        && matches!(recipe.inputs.last().unwrap().choices[0].consume, Consumption::Allocated), "kettle water/bottle extraction changed");
    for ingredient in recipe.inputs.iter().filter(|input| input.kind == Kind::Item && input.slot < 6) {
        ensure!(ingredient.choices.len() == 1 && ingredient.choices[0].amount == "1"
            && matches!(ingredient.choices[0].consume, Consumption::Consume)
            && matches!(ingredient.choices[0].rule, Match::Wildcard {meta:false, nbt:true}), "kettle action ingredient changed");
    }
    ensure!(recipe.outputs.len() == 1, "kettle has an invented result");
    if p.hat_bonus == 0 { fixed_result(recipe, 0)?; }
    else { potential_result(recipe, 0, "witchery:kettleHatYield", p.base_output + p.hat_bonus, "playerWitchHat")?; }
    Ok(())
}
pub fn validate_ritual(p: &RitualProcess, recipe: &Recipe, items: &BTreeMap<&str, &Item>) -> Result<()> {
    ensure!(recipe.source.handler == "alkalus.main.nei.NEI_Handler_Cauldron" && recipe.duration.is_none()
        && recipe.energy.is_none() && recipe.grid.is_none() && recipe.magic.is_none(), "invalid ritual handler/cost");
    ensure!(p.power >= 0 && p.water_millibuckets == 3000 && p.keys.len() <= 6
        && p.earlier.len() <= 65536, "invalid ritual native action list");
    ensure!(ritual_first_match(p.earlier.iter().map(Vec::as_slice).chain(std::iter::once(p.keys.as_slice())), &p.keys)
        == Some(p.earlier.len()), "ritual is shadowed by earlier native recipe");
    ensure!(recipe.inputs.iter().filter(|input| input.kind == Kind::Item).count() == p.keys.len()
        && recipe.inputs.iter().any(|input| input.kind == Kind::Fluid && input.slot == 0
            && input.choices.len() == 1 && input.choices[0].amount == "3000"), "ritual items/full water changed");
    let mut projected = Vec::new();
    for input in recipe.inputs.iter().filter(|input| input.kind == Kind::Item) {
        ensure!(input.choices.len() == 1 && input.choices[0].amount == "1"
            && matches!(input.choices[0].consume, Consumption::Consume)
            && matches!(input.choices[0].rule, Match::Wildcard { meta:false, nbt:true }),
            "ritual action key predicate changed");
        let item = items.get(input.choices[0].id.as_str())
            .ok_or_else(|| anyhow::anyhow!("missing ritual ingredient identity"))?;
        projected.push(format!("{}:{}", item.registry, item.meta));
    }
    projected.sort();
    let mut keys = p.keys.clone(); keys.sort();
    ensure!(projected == keys, "ritual action list differs from ingredient keys");
    fixed_result(recipe, 0)?;
    ensure!(recipe.outputs.len() == 1, "ritual has an invented output");
    Ok(())
}
pub fn validate_infernal(p: &InfernalProcess, recipe: &Recipe, items: &BTreeMap<&str, &Item>) -> Result<()> {
    ensure!(recipe.source.handler == "witchinggadgets.client.nei.NEIInfernalBlastfurnaceHandler"
        && recipe.duration.is_none()
        && recipe.energy.is_none() && recipe.grid.is_none() && recipe.magic.is_none(), "invalid infernal handler/cost");
    ensure!(p.input_count > 0 && p.base_ticks > 0 && p.maximum_bellows == 4
        && p.no_bellows_chance == 0.25 && p.per_bellows_chance == 0.44
        && p.ticks_per_bellows == 40 && p.vis_divisor == 2, "infernal callback constants changed");
    ensure!(p.earlier.len() == p.selector as usize
        && match p.input { InfernalInput::Ore => p.ore_key.as_deref().is_some_and(|key| !key.is_empty()),
            InfernalInput::Stack => p.ore_key.is_none() }
        && p.earlier.iter().all(|prior| prior.input_count > 0 && match prior.input {
            InfernalInput::Ore => prior.ore_key.as_deref().is_some_and(|key| !key.is_empty())
                && prior.registry.is_none(),
            InfernalInput::Stack => prior.registry.as_deref().is_some_and(|name| !name.is_empty())
                && prior.ore_key.is_none(),
        }), "infernal first-match registry mask changed");
    ensure!(recipe.inputs.len() == 1 && recipe.inputs[0].kind == Kind::Item && recipe.inputs[0].slot == 0
        && !recipe.inputs[0].choices.is_empty(), "infernal input shape changed");
    for choice in &recipe.inputs[0].choices {
        ensure!(choice.amount == p.input_count.to_string() && matches!(choice.consume, Consumption::Consume)
            && matches!(choice.rule, Match::Wildcard {nbt:true, ..}), "infernal counted ore/stack predicate changed");
        let offered = items.get(choice.id.as_str())
            .ok_or_else(|| anyhow::anyhow!("missing infernal input identity"))?;
        ensure!(!p.earlier.iter().any(|prior| infernal_prior_matches(prior, offered, p.input_count)),
            "infernal row is shadowed by an earlier native recipe");
    }
    fixed_result(recipe, 0)?;
    ensure!(recipe.outputs.len() == if p.bonus {2} else {1}, "infernal bonus branch changed");
    if p.bonus { potential_result(recipe, 1, "witchinggadgets:blastBonus", 4, "nativeBellowsRolls")?; }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn specimen() -> Item { Item { id:"item_fixture".into(),registry:"minecraft:log".into(),meta:2,
        nbt:None,name:"".into(),tooltip:vec![],stack_limit:64,durability:0,tools:BTreeMap::new(),
        armor:false,tags:vec!["logWood".into()],icon:None,order:None,aspects:None } }
    #[test]
    fn oven_nbt_classifier_replays_missing_malformed_and_species_branches() {
        use serde_json::json;
        assert_eq!(oven_nbt_product(None),OvenByproduct::Foul);
        for (root,want) in [
            (json!({}),OvenByproduct::Foul),
            (json!({"owner":{"type":"string","value":"abc"}}),OvenByproduct::Foul),
            (json!({"Genome":{"type":"string","value":"malformed"}}),OvenByproduct::None),
            (json!({"Genome":{"type":"compound","value":{}}}),OvenByproduct::None),
        ] {
            let nbt:Nbt=serde_json::from_value(json!({"type":"compound","value":root})).unwrap();
            assert_eq!(oven_nbt_product(Some(&nbt)),want);
        }
        for (uid,want) in [("forestry.treeOak",OvenByproduct::Exhale),("forestry.treeSpruce",OvenByproduct::Hint),
            ("forestry.treeBirch",OvenByproduct::Breath),("other",OvenByproduct::None)] {
            let nbt:Nbt=serde_json::from_value(json!({"type":"compound","value":{"Genome":{"type":"compound","value":{
                "Chromosomes":{"type":"list","element":"compound","value":[{"type":"compound","value":{"UID0":{"type":"string","value":uid}}}]}
            }}}})).unwrap();
            assert_eq!(oven_nbt_product(Some(&nbt)),want);
        }
    }
    #[test]
    fn oven_priority_replays_zero_jar_and_ore_bypass() {
        let item=specimen();
        let prior=OvenPrior{registry:Some("minecraft:log".into()),meta:1,subtypes:true,
            jar_threshold:3,ore_key:Some("logWood".into())};
        assert!(oven_prior_matches(&prior,&item,0),"ore tag bypasses jar threshold");
        let prior=OvenPrior{ore_key:None,..prior};
        assert!(!oven_prior_matches(&prior,&item,3),"subtype metadata must match");
        let prior=OvenPrior{jar_threshold:0,..prior};
        assert!(oven_prior_matches(&prior,&item,0),"native zero-jar recipe admits every input");
    }
    #[test]
    fn infernal_priority_replays_counted_stack_and_ore_predicates() {
        let item=specimen();
        let prior=InfernalPrior{input:InfernalInput::Stack,ore_key:None,registry:Some("minecraft:log".into()),
            meta:1,meta_wildcard:true,input_count:3};
        assert!(!infernal_prior_matches(&prior,&item,2));
        assert!(infernal_prior_matches(&prior,&item,3));
        let prior=InfernalPrior{input:InfernalInput::Ore,ore_key:Some("logWood".into()),
            registry:None,meta:0,meta_wildcard:false,input_count:4};
        assert!(infernal_prior_matches(&prior,&item,4));
    }
    #[test]
    fn oven_jar_roll_requires_accepting_side_slot_and_funnel_state() {
        let p=OvenProcess{selector:0,jar_threshold:1,jar_mode:JarMode::Conditional,
            byproduct:OvenByproduct::Foul,byproduct_item:None,nbt_classified:true,filtered_double:false,base_ticks:180,
            ticks_per_funnel:20,maximum_funnels:3,base_chance:0.3,
            normal_funnel_chance:0.25,filtered_funnel_chance:0.3,earlier:vec![]};
        assert_eq!(oven_jar_debit(&p,0.4,0,0,true),0);
        assert_eq!(oven_jar_debit(&p,0.4,1,0,true),1);
        assert_eq!(oven_jar_debit(&p,0.1,1,0,false),0);
        assert_eq!(oven_ticks(&p,3),120);
    }
    #[test]
    fn kettle_hat_and_extraction_bottles_are_separate() {
        let p=KettleProcess{selector:0,power:0.6,dimension:0,familiar_type:0,base_output:2,
            hat_bonus:1,water_millibuckets:1000,bottles_per_item:1};
        assert_eq!(kettle_output(&p,false),2);assert_eq!(kettle_output(&p,true),3);
        assert_eq!(kettle_bottle_debit(2),2);
    }
    #[test]
    fn ritual_replays_first_matching_multiset_with_multiplicity() {
        let a=vec!["a".into(),"a".into()];let b=vec!["a".into(),"b".into()];
        assert_eq!(ritual_first_match([a.as_slice(),b.as_slice()],&["a".into(),"b".into()]),Some(1));
        assert_eq!(ritual_first_match([a.as_slice(),b.as_slice()],&["a".into(),"a".into(),"b".into()]),Some(0));
    }
    #[test]
    fn infernal_bonus_is_one_quarter_or_four_bellows_trials() {
        let p=InfernalProcess{selector:0,input:InfernalInput::Stack,input_count:3,
            ore_key:None,base_ticks:1200,special_fuel:false,bonus:true,maximum_bellows:4,
            no_bellows_chance:0.25,per_bellows_chance:0.44,ticks_per_bellows:40,vis_divisor:2,earlier:vec![]};
        assert_eq!(infernal_bonus(&p,0,&[0.24]),1);
        assert_eq!(infernal_bonus(&p,0,&[0.25]),0);
        assert_eq!(infernal_bonus(&p,4,&[0.1,0.5,0.43,0.44]),2);
        assert_eq!(infernal_ticks(&p,4,true),440);
    }
}
