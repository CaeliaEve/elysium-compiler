//! Native infusion predicates and state transitions. Display choices are specimens.
use super::{AspectAmount, Consumption, Edit, Item, Kind, MagicKind, Match, OutputRole, Recipe};
use crate::identity::Nbt;
use anyhow::{bail, ensure, Context, Result};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use super::infusion_ingredient::{Predicate as IngredientPredicate,matches as ingredient_match,components as enhanced_components};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub enum OreMode { Default, Strict, Relaxed, None }

#[derive(Clone,Copy,Debug,PartialEq,Eq,Serialize,Deserialize,JsonSchema)]
#[serde(rename_all="camelCase")]
pub enum StartupMatcher { Legacy, ItemAndTags, Enhanced }

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all="camelCase", deny_unknown_fields)]
pub struct Requirement {
    pub template: Template,
    /// Ore groups whose actual member list accepts the original template.
    pub default_ores: Vec<String>,
    pub ores: Vec<String>,
    pub tag_strings: BTreeMap<String,String>,
    /// craftingStart copies each component once before the later pedestal comparison.
    pub cycle_tag_strings:BTreeMap<String,String>,
    /// Central input is copied once, a pedestal input twice, before comparison.
    pub copies:u8,
}

/// A registered predicate, including wildcard metadata, has no display name/icon.
#[derive(Clone,Debug,PartialEq,Serialize,Deserialize,JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Template {pub registry:String,pub metadata:i32,pub nbt:Option<Nbt>}

#[derive(Clone,Debug,Serialize,Deserialize,JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ReplacementStack {pub item:Template,pub amount:i32}
#[derive(Clone,Debug,Serialize,Deserialize,JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct PedestalReplacement {pub input:Option<Template>,pub output:Option<ReplacementStack>,pub strict:bool}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all="camelCase", deny_unknown_fields)]
pub struct ItemState {
    pub id: String,
    /// Actual OreDictionary.getOreIDs order, before sorting for catalog Item.tags.
    pub ores: Vec<String>,
    pub wildcard_ores: Vec<String>,
    /// Native NBTBase.toString observations; compound order is not reconstructed from JSON.
    pub tag_strings: BTreeMap<String, String>,
    pub copied_tag_strings: BTreeMap<String,String>,
    pub pedestal_tag_strings: BTreeMap<String,String>,
    pub augment_keys:Vec<String>,
    pub copied_augment_keys:Vec<String>,
    pub lowercase_keys:BTreeMap<String,String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all="camelCase", deny_unknown_fields)]
pub struct LegacyPattern {
    pub mode: OreMode,
    pub startup:StartupMatcher,
    pub predicates:Option<Vec<IngredientPredicate>>,
    pub central: Option<Requirement>,
    pub components: Vec<Requirement>,
    pub states: Vec<ItemState>,
    pub allow_extra:bool,
}

#[derive(Clone,Debug,Serialize,Deserialize,JsonSchema)]
#[serde(tag="kind",rename_all="lowercase",deny_unknown_fields)]
pub enum FamiliarCondition {Skip,Any{required:Option<Vec<Option<u32>>>},All{required:Option<Vec<Option<u32>>>}}
#[derive(Clone,Debug,Serialize,Deserialize,JsonSchema)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct FamiliarAugment {pub name:String,pub requires_previous:bool,pub conflicts:Vec<Option<u32>>,pub conditions:Vec<FamiliarCondition>}
#[derive(Clone,Debug,Serialize,Deserialize,JsonSchema)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct Familiar {
    pub target:u32,pub names:BTreeMap<String,Option<u32>>,pub augments:Vec<FamiliarAugment>,pub items:Vec<String>,
    pub required_previous_level:i32,pub locale:String,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all="camelCase", deny_unknown_fields)]
pub struct RevolverUpgrade {
    pub id: i32,
    pub max_level: i32,
    pub instability: i32,
    /// Object identities encoded as indices, independently of the public numeric id.
    pub conflicts: Vec<u32>,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(tag="kind", rename_all="camelCase", deny_unknown_fields)]
pub enum Effect {
    /// Ordinary fixed/tag output is described by the existing output/change contract.
    Declared,
    Golem { namespace: String, key: String },
    OldFamiliar { key: String, previous: Option<String>, items: Vec<String> },
    Revolver { target: u32, registered: Vec<Option<u32>>, upgrades: Vec<RevolverUpgrade> },
    Familiar(Familiar),
    Wand {name:String,prerequisites:Vec<String>,#[serde(rename="definitionWand")] definition_wand:bool},
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all="camelCase", deny_unknown_fields)]
pub struct NativeInfusion {
    pub native_recipe: String,
    pub declared_input: Template,
    pub pattern: LegacyPattern,
    pub effect: Effect,
    pub base_aspects: Vec<AspectAmount>,
    #[serde(default, skip_serializing_if="Option::is_none")]
    pub random_aspects: Option<super::infusion_cost::RandomAspects>,
    pub base_instability: i32,
    pub pedestals:Vec<PedestalReplacement>,
    pub pedestal_limit:i32,
    /// Loaded native class contains the GTNH replacement Mixin state/helper.
    pub replacements_active:bool,
}

fn compound(tag:Option<&Nbt>)->Result<Option<&BTreeMap<String,Nbt>>> {
    match tag {None=>Ok(None),Some(Nbt::Compound{value})=>Ok(Some(value)),_=>bail!("infusion root tag is not a compound")}
}
fn child(tag:Option<&Nbt>)->Option<&BTreeMap<String,Nbt>> {
    match tag {Some(Nbt::Compound{value})=>Some(value),_=>None}
}
fn entries(tag:Option<&Nbt>)->Result<Vec<&BTreeMap<String,Nbt>>> {
    let list=compound(tag)?.and_then(|v|v.get("tb.upgrades"));
    if let Some(Nbt::List{element,value})=list {
        if element=="compound" {
            ensure!(value.len()<=4096,"revolver state exceeds budget");
            return value.iter().map(|v|child(Some(v)).context("invalid revolver compound entry")).collect();
        }
    }
    Ok(Vec::new())
}
fn registered(effect:&Effect,id:i32)->Result<Option<u32>> {
    let Effect::Revolver{registered,..}=effect else {bail!("not a revolver effect");};
    ensure!(id>=0,"native revolver array access fails on a negative id");
    Ok(registered.get(id as usize).copied().flatten())
}
pub fn level(effect:&Effect,tag:Option<&Nbt>)->Result<i32> {
    let Effect::Revolver{target,..}=effect else {return Ok(0);};
    for entry in entries(tag)? {
        if registered(effect,super::change::number(entry.get("id"))?)?==Some(*target) {
            return super::change::number(entry.get("level"));
        }
    }
    Ok(0)
}
pub fn admitted(effect:&Effect,tag:Option<&Nbt>,registry:&str)->Result<bool> {
    let root=compound(tag)?;
    Ok(match effect {
        Effect::Declared=>true,
        Effect::Familiar(_)=>bail!("familiar admission requires source-observed key order"),
        Effect::Wand{name,prerequisites,definition_wand}=>*definition_wand&&!wand_names(tag)?.contains(&name.as_str())
            &&prerequisites.iter().all(|required|wand_names(tag).is_ok_and(|names|names.contains(&required.as_str()))),
        Effect::Golem{namespace,key}=>child(root.and_then(|v|v.get(namespace)))
            .and_then(|v|child(v.get("upgrades"))).is_none_or(|v|!v.contains_key(key)),
        Effect::OldFamiliar{previous,items,..}=>items.iter().any(|v|v==registry)
            && match previous {None=>true,Some(key)=>(super::change::number(root.and_then(|v|v.get(key)))? as i8)!=0},
        Effect::Revolver{target,upgrades,..}=>{
            for entry in entries(tag)? {
                if let Some(index)=registered(effect,super::change::number(entry.get("id"))?)? {
                    let entry=upgrades.get(index as usize).context("missing registered upgrade object")?;
                    if entry.conflicts.contains(target) {return Ok(false);}
                }
            }
            level(effect,tag)?.wrapping_add(1)<=upgrades.get(*target as usize).context("missing target upgrade")?.max_level
        }
    })
}
pub fn transform(effect:&Effect,tag:Option<&Nbt>)->Result<Nbt> {
    let mut root=compound(tag)?.cloned().unwrap_or_default();
    match effect {
        Effect::Declared=>bail!("declared infusion output uses its separate result contract"),
        Effect::Familiar(_)=>bail!("familiar output requires source-observed copied key order"),
        Effect::Wand{name,..}=>{
            // getTagList returns a detached empty list for a scalar/wrong-type
            // value; this output explicitly attaches it after appending.
            let mut values=match root.get("Augmentations") {
                Some(Nbt::List{element,value}) if element=="string"=>value.clone(),
                _=>Vec::new(),
            };
            values.push(Nbt::String{value:name.clone()});
            root.insert("Augmentations".into(),Nbt::List{element:"string".into(),value:values});
        },
        Effect::Golem{namespace,key}=>{
            let mut data=child(root.get(namespace)).cloned().unwrap_or_default();
            match data.get_mut("upgrades") {
                Some(Nbt::Compound{value})=>{value.insert(key.clone(),Nbt::Byte{value:"1".into()});},
                None=>{data.insert("upgrades".into(),Nbt::Compound{value:BTreeMap::from([(key.clone(),Nbt::Byte{value:"1".into()})])});},
                // getCompoundTag creates a detached object for a present scalar. Do not replace it.
                Some(_)=>{},
            }
            root.insert(namespace.clone(),Nbt::Compound{value:data});
        },
        Effect::OldFamiliar{key,..}=>{root.insert(key.clone(),Nbt::Byte{value:"1".into()});},
        Effect::Revolver{target,upgrades,..}=>{
            let id=upgrades.get(*target as usize).context("missing revolver target")?.id;
            let next=level(effect,tag)?.wrapping_add(1);
            let list=root.entry("tb.upgrades".into()).or_insert_with(||Nbt::List{element:"end".into(),value:Vec::new()});
            if let Nbt::List{element,value}=list {
                if element=="compound"||element=="end"&&value.is_empty() {
                    let mut found=false;
                    for entry in value.iter_mut() {
                        let Nbt::Compound{value}=entry else {bail!("invalid revolver entry");};
                        if super::change::number(value.get("id"))?==id {
                            value.insert("level".into(),Nbt::Int{value:next.to_string()});found=true;break;
                        }
                    }
                    if !found {
                        *element="compound".into();
                        value.push(Nbt::Compound{value:BTreeMap::from([
                            ("id".into(),Nbt::Int{value:id.to_string()}),("level".into(),Nbt::Int{value:next.to_string()})])});
                    }
                }
            }
        },
    }
    Ok(Nbt::Compound{value:root})
}
/// Each inner iteration doubles one existing LinkedHashMap value; no keys are added/removed.
/// After 32 doublings every signed int is zero, including for huge positive levels.
pub fn doubled(value:i32,level:i32)->i32 {if level<=0{value}else if level>=32{0}else{value.wrapping_shl(level as u32)}}

fn familiar_pairs(f:&Familiar,item:&Item,state:&ItemState,copied:bool)->Result<Vec<(Option<u32>,i32)>> {
    let tags=child(compound(item.nbt.as_ref())?.and_then(|root|root.get("augments")));
    let keys=if copied{&state.copied_augment_keys}else{&state.augment_keys};
    keys.iter().map(|key|{
        let lowered=state.lowercase_keys.get(key).context("missing native familiar lowercase observation")?;
        Ok((f.names.get(lowered).copied().flatten(),super::change::number(tags.and_then(|v|v.get(key)))?))
    }).collect()
}
fn familiar_first(pairs:&[(Option<u32>,i32)],target:u32)->Result<Option<i32>> {
    for (augment,level) in pairs {let id=augment.context("native familiar lookup dereferences an unknown augment")?;if id==target{return Ok(Some(*level));}}
    Ok(None)
}
fn familiar_admitted(f:&Familiar,item:&Item,state:&ItemState)->Result<bool> {
    if !f.items.contains(&item.registry){return Ok(false);}
    let pairs=familiar_pairs(f,item,state,false)?;let level=familiar_first(&pairs,f.target)?.unwrap_or(0);let next=level.wrapping_add(1);
    if f.required_previous_level>level||next<=0{return Ok(false);}
    let target=f.augments.get(f.target as usize).context("missing familiar target")?;
    if pairs.iter().any(|(id,_)|target.conflicts.contains(id)){return Ok(false);}
    if target.requires_previous {
        let mut last=None;
        for (id,level) in &pairs {let id=id.context("native familiar previous-level lookup dereferences an unknown augment")?;if id==f.target{last=Some(*level);}}
        if match last{None=>next!=1,Some(level)=>level!=next.wrapping_sub(1)}{return Ok(false);}
    }
    for condition in &target.conditions {
        let (any,required)=match condition{FamiliarCondition::Skip=>continue,FamiliarCondition::Any{required}=>(true,required),FamiliarCondition::All{required}=>(false,required)};
        let Some(required)=required else {continue;};let mut result=!any;
        for id in required.iter().flatten(){let found=familiar_first(&pairs,*id)?.is_some();if any&&found{result=true;break;}if !any&&!found{result=false;break;}}
        if !result{return Ok(false);}
    }
    Ok(true)
}
fn observed_admitted(p:&NativeInfusion,item:&Item)->Result<bool> {
    if let Effect::Familiar(f)=&p.effect {familiar_admitted(f,item,state(&p.pattern,&item.id)?)}else{admitted(&p.effect,item.nbt.as_ref(),&item.registry)}
}
fn observed_transform(p:&NativeInfusion,item:&Item)->Result<Nbt> {
    let Effect::Familiar(f)=&p.effect else {return transform(&p.effect,item.nbt.as_ref());};
    let pairs=familiar_pairs(f,item,state(&p.pattern,&item.id)?,true)?;
    let next=familiar_first(&pairs,f.target)?.map_or(1,|n|n.wrapping_add(1));
    let mut tags=compound(item.nbt.as_ref())?.cloned().unwrap_or_default();
    if next>0 {
        let mut augments=child(tags.get("augments")).cloned().unwrap_or_default();
        augments.insert(f.augments[f.target as usize].name.clone(),Nbt::Int{value:next.to_string()});
        tags.insert("augments".into(),Nbt::Compound{value:augments});
    }
    Ok(Nbt::Compound{value:tags})
}

fn state<'a>(p:&'a LegacyPattern,id:&str)->Result<&'a ItemState> {
    p.states.binary_search_by(|s|s.id.as_str().cmp(id)).ok().map(|i|&p.states[i]).context("native infusion comparison observation is missing")
}
fn replacement_match(target:&Template,offered:&Template,strict:bool)->bool {
    target.registry==offered.registry&&(!strict&&target.metadata==32767||target.metadata==offered.metadata)
}
fn replacement<'a>(rules:&'a [PedestalReplacement],item:&Item)->Option<&'a PedestalReplacement> {
    let offered=Template{registry:item.registry.clone(),metadata:item.meta,nbt:None};
    rules.iter().find(|r|r.input.as_ref().is_some_and(|i|replacement_match(i,&offered,r.strict)))
}
fn validate_pedestal(p:&NativeInfusion,c:&super::Choice,item:&Item,items:&BTreeMap<&str,&Item>)->Result<()> {
    let rule=if p.replacements_active{replacement(&p.pedestals,item)}else{None};
    let Some(rule)=rule else {
        let consumption=if p.pattern.startup==StartupMatcher::Enhanced{matches!(c.consume,Consumption::Allocated)}else{matches!(c.consume,Consumption::Stack)};
        ensure!(consumption&&c.returns.len()<=1,"native pedestal consumption differs from its startup/cycle allocation");
        for returned in &c.returns{ensure!(returned.kind==Kind::Item&&crate::identity::integer(&returned.amount,1,i64::from(p.pedestal_limit))?>0,"native pedestal return exceeds its inventory limit");}
        return Ok(());
    };
    let offered=Template{registry:item.registry.clone(),metadata:item.meta,nbt:None};
    let keep=rule.output.as_ref().is_some_and(|out|replacement_match(&offered,&out.item,rule.strict));
    if keep {ensure!(matches!(c.consume,Consumption::Keep)&&c.returns.is_empty(),"GTNH matching output preserves the offered pedestal including NBT/count");return Ok(());}
    ensure!(matches!(c.consume,Consumption::Stack),"GTNH replacement must replace the offered pedestal");
    if let Some(out)=&rule.output {
        ensure!(out.amount>0&&c.returns.len()==1,"GTNH replacement return missing");
        let returned=&c.returns[0];let actual=items.get(returned.id.as_str()).context("GTNH pedestal replacement fact missing")?;
        ensure!(returned.kind==Kind::Item&&returned.amount==out.amount.min(p.pedestal_limit).to_string()&&actual.registry==out.item.registry&&actual.meta==out.item.metadata&&actual.nbt==out.item.nbt,"GTNH replacement return differs from its definition and inventory limit");
    } else {ensure!(c.returns.is_empty(),"GTNH null replacement clears the pedestal without a native container");}
    Ok(())
}
pub fn legacy_match(p:&LegacyPattern,requirement:&Requirement,offered:&Item,amount:i32,_items:&BTreeMap<&str,&Item>)->Result<bool> {
    legacy_phase(p,requirement,offered,amount,false)
}
pub(super) fn legacy_phase(p:&LegacyPattern,requirement:&Requirement,offered:&Item,amount:i32,cycle:bool)->Result<bool> {
    let template=&requirement.template;
    let offered_state=state(p,&offered.id)?;
    let tag_strings=if cycle{&offered_state.tag_strings}else if requirement.copies==1{&offered_state.copied_tag_strings}else{&offered_state.pedestal_tag_strings};
    let required_tags=if cycle{&requirement.cycle_tag_strings}else{&requirement.tag_strings};
    if !cycle&&p.startup==StartupMatcher::ItemAndTags {
        return Ok(machina_component(template,required_tags,&offered.registry,offered.nbt.is_some(),tag_strings));
    }
    if template.nbt.is_some()&&(offered.nbt.is_none()||required_tags.iter().any(|(key,value)|tag_strings.get(key)!=Some(value))) {return Ok(false);}
    let wildcard=template.metadata==32767;let rewrite=wildcard&&!cycle;let metadata=if rewrite{32767}else{offered.meta};
    let ores=if rewrite{&offered_state.wildcard_ores}else{&offered_state.ores};
    let fuzzy=match p.mode {
        OreMode::Default=>ores.first().is_some_and(|name|requirement.default_ores.contains(name)),
        OreMode::Strict=>ores.iter().collect::<BTreeSet<_>>()==requirement.ores.iter().collect::<BTreeSet<_>>(),
        OreMode::Relaxed=>ores.iter().any(|name|requirement.ores.contains(name)),
        OreMode::None=>offered.registry==template.registry&&metadata==template.metadata&&match (&offered.nbt,&template.nbt){
            (None,None)=>true,(Some(a),Some(b))=>super::matching::native_tag(a,b,true),_=>false},
    };
    Ok(fuzzy||offered.registry==template.registry&&(metadata==template.metadata||wildcard)&&i64::from(amount)<=i64::from(offered.stack_limit))
}
fn wand_names(tag:Option<&Nbt>)->Result<Vec<&str>> {
    if let Some(Nbt::List{element,value})=compound(tag)?.and_then(|root|root.get("Augmentations")) {
        if element=="string" {
            ensure!(value.len()<=4096,"wand augmentation NBT exceeds budget");
            return value.iter().map(|v|if let Nbt::String{value}=v{Ok(value.as_str())}else{bail!("invalid augmentation string entry")}).collect();
        }
    }
    Ok(Vec::new())
}
fn machina_component(template:&Template,required:&BTreeMap<String,String>,registry:&str,has_tag:bool,offered:&BTreeMap<String,String>)->bool {
    template.registry==registry&&(template.nbt.is_none()||has_tag&&required.iter().all(|(key,value)|offered.get(key)==Some(value)))
}
/// Greedy declaration-order matching; research, registry priority and altar world state are external.
pub fn matches(p:&NativeInfusion,central:&Item,outer:&[&Item],items:&BTreeMap<&str,&Item>)->Result<bool> {
    if !observed_admitted(p,central)? {return Ok(false);}
    if let Some(required)=&p.pattern.central {if !legacy_match(&p.pattern,required,central,1,items)? {return Ok(false);}}
    let enhanced=p.pattern.startup==StartupMatcher::Enhanced;
    if enhanced {
        let predicates=p.pattern.predicates.as_ref().context("enhanced startup predicates missing")?;
        let (center,components)=predicates.split_first().context("enhanced central predicate missing")?;
        if !ingredient_match(center,central)?||!enhanced_components(components,outer)?{return Ok(false);}
    }
    // Preserve each phase's own first match. Combining both predicates while
    // selecting would silently skip the first startup match and invent priority.
    for cycle in [false,true] {
        if enhanced&&!cycle{continue;}
        let mut remaining=outer.to_vec();
        for required in &p.pattern.components {
            let mut matched=None;
            for (index,item) in remaining.iter().enumerate() {if legacy_phase(&p.pattern,required,item,1,cycle)? {matched=Some(index);break;}}
            let Some(index)=matched else {return Ok(false);};remaining.remove(index);
        }
        if !cycle&&!p.pattern.allow_extra&&!remaining.is_empty(){return Ok(false);}
    }
    Ok(true)
}

pub(super) fn validate_pattern(pattern:&LegacyPattern,items:&BTreeMap<&str,&Item>)->Result<()> {
    ensure!(pattern.states.len()<=65536&&pattern.states.windows(2).all(|s|s[0].id<s[1].id),"invalid native infusion state table");
    for s in &pattern.states {
        let item=items.get(s.id.as_str()).context("native infusion observed item missing")?;
        ensure!(s.ores.len()<=65536&&s.wildcard_ores.len()<=65536&&s.tag_strings.len()<=4096
            &&s.ores.iter().collect::<BTreeSet<_>>().len()==s.ores.len()&&s.wildcard_ores.iter().collect::<BTreeSet<_>>().len()==s.wildcard_ores.len()
            &&s.ores.iter().collect::<BTreeSet<_>>()==item.tags.iter().collect::<BTreeSet<_>>(),"native infusion ore observations disagree with item facts");
        let keys=compound(item.nbt.as_ref())?.into_iter().flat_map(|v|v.keys().map(String::as_str)).collect::<BTreeSet<_>>();
        for strings in [&s.tag_strings,&s.copied_tag_strings,&s.pedestal_tag_strings]{
            ensure!(strings.keys().map(String::as_str).collect::<BTreeSet<_>>()==keys&&strings.values().all(|s|s.len()<=1048576),"invalid native NBT string observations");
        }
        let augments=child(compound(item.nbt.as_ref())?.and_then(|v|v.get("augments")));
        let augment_keys:BTreeSet<_>=augments.into_iter().flat_map(|v|v.keys()).collect();
        ensure!(s.augment_keys.len()<=4096&&s.augment_keys.iter().collect::<BTreeSet<_>>()==augment_keys&&s.augment_keys.len()==augment_keys.len()
            &&s.copied_augment_keys.iter().collect::<BTreeSet<_>>()==augment_keys&&s.copied_augment_keys.len()==augment_keys.len()
            &&s.lowercase_keys.keys().collect::<BTreeSet<_>>()==augment_keys&&s.lowercase_keys.values().all(|v|v.len()<=65535),"invalid native familiar key-order observations");
    }
    if let Some(req)=&pattern.central{ensure!(req.copies==1,"central infusion comparison copies once");}
    for req in &pattern.components{ensure!(req.copies==2,"pedestal infusion comparison copies twice");}
    for req in pattern.central.iter().chain(&pattern.components) {
        crate::identity::registry_name(&req.template.registry)?;
        if let Some(nbt)=&req.template.nbt{nbt.validate()?;}
        ensure!(req.default_ores.len()<=65536&&req.default_ores.windows(2).all(|p|p[0]<p[1]),"invalid native infusion requirement");
        let template=&req.template;
        ensure!(req.ores.len()<=65536&&req.ores.iter().collect::<BTreeSet<_>>().len()==req.ores.len(),"registered infusion ore observation differs");
        let keys=compound(template.nbt.as_ref())?.into_iter().flat_map(|v|v.keys().map(String::as_str)).collect::<BTreeSet<_>>();
        for strings in [&req.tag_strings,&req.cycle_tag_strings]{ensure!(strings.keys().map(String::as_str).collect::<BTreeSet<_>>()==keys
            &&strings.values().all(|s|s.len()<=1048576),"registered infusion template copy observation differs");}
    }
    Ok(())
}

pub(super) fn validate(p:&NativeInfusion,r:&Recipe,items:&BTreeMap<&str,&Item>)->Result<()> {
    let enhanced=p.pattern.startup==StartupMatcher::Enhanced;
    ensure!(super::magic::ari_source(&r.source,super::magic::ARI_INFUSION)
        &&r.grid.is_none()&&r.duration.is_none()&&r.energy.is_none()
        &&(enhanced||r.inputs.len()==p.pattern.components.len()+1)
        &&!r.inputs.is_empty()&&r.inputs.len()<=4096&&p.pattern.components.len()<=4095&&r.outputs.len()==1,"invalid native infusion branch");
    let declared=&p.declared_input;
    crate::identity::registry_name(&declared.registry)?;
    if let Some(nbt)=&declared.nbt{nbt.validate()?;compound(Some(nbt))?;}
    let expected=match &p.effect {
        Effect::Declared=>if p.native_recipe=="com.gtnewhorizon.gtnhlib.api.thaumcraft.EnhancedInfusionRecipe"{
            "com.gtnewhorizon.gtnhlib.api.thaumcraft.EnhancedInfusionRecipe"
        }else if p.native_recipe=="net.glease.tc4tweak.api.infusionrecipe.EnhancedInfusionRecipe"{
            "net.glease.tc4tweak.api.infusionrecipe.EnhancedInfusionRecipe"
        }else{"thaumcraft.api.crafting.InfusionRecipe"},
        Effect::Golem{namespace,key}=>{
            ensure!(namespace=="gadomancy"&&key=="runicShield","unadapted golem upgrade effect");
            "makeo.gadomancy.common.crafting.InfusionUpgradeRecipe"
        },
        Effect::OldFamiliar{key,previous,items:accepted}=>{
            let prior=match key.as_str(){"AttackUpgrade1"|"CdUpgrade1"|"RangeUpgrade1"=>None,"AttackUpgrade2"=>Some("AttackUpgrade1"),"AttackUpgrade3"=>Some("AttackUpgrade2"),_=>bail!("unknown old familiar upgrade")};
            ensure!(previous.as_deref()==prior&&!accepted.is_empty()&&accepted.len()<=65536&&accepted.windows(2).all(|p|p[0]<p[1])
                &&accepted.contains(&declared.registry),"invalid old familiar class/upgrade binding");
            for registry in accepted {crate::identity::registry_name(registry)?;}
            "makeo.gadomancy.common.crafting.Familiar_Old_AugmentInfusion"
        },
        Effect::Revolver{target,registered,upgrades}=>{
            ensure!(!upgrades.is_empty()&&upgrades.len()<=4096&&registered.len()<=4096&&(*target as usize)<upgrades.len(),"invalid revolver upgrade graph");
            ensure!(registered.iter().flatten().all(|n|(*n as usize)<upgrades.len())
                &&upgrades.iter().all(|u|u.conflicts.len()<=4096&&u.conflicts.iter().all(|n|(*n as usize)<upgrades.len())),"invalid revolver object link");
            "tb.utils.RevolverInfusionRecipe"
        },
        Effect::Familiar(f)=>{
            ensure!(!f.augments.is_empty()&&f.augments.len()<=4096&&(f.target as usize)<f.augments.len()
                &&f.names.len()<=4096&&!f.items.is_empty()&&f.items.len()<=65536&&f.items.windows(2).all(|v|v[0]<v[1])
                &&f.items.contains(&declared.registry)&&!f.locale.is_empty()&&f.locale.len()<=256,"invalid familiar source binding");
            for registry in &f.items{crate::identity::registry_name(registry)?;}
            let link=|id:&Option<u32>|id.is_none_or(|id|(id as usize)<f.augments.len());
            ensure!(f.names.iter().all(|(key,id)|key.len()<=65535&&link(id)),"invalid familiar name registry");
            for augment in &f.augments{
                ensure!(augment.name.len()<=65535&&augment.conflicts.len()<=4096&&augment.conflicts.iter().all(link)&&augment.conditions.len()<=4096,"invalid familiar augment definition");
                for condition in &augment.conditions{if let FamiliarCondition::Any{required}|FamiliarCondition::All{required}=condition{
                    ensure!(required.as_ref().is_none_or(|v|v.len()<=4096&&v.iter().all(link)),"invalid familiar precondition");
                }}
            }
            "makeo.gadomancy.common.crafting.EtherealFamiliarUpgradeRecipe"
        },
        Effect::Wand{name,prerequisites,definition_wand}=>{
            let expected=match name.as_str(){"Taint Capping"=>vec!["Tainted Core"],"Tainted Core"|"Charge Buffer"|"Contact Discharge"|"Vis Channel"=>vec![],_=>bail!("unknown native wand augmentation")};
            ensure!(*definition_wand&&prerequisites.iter().map(String::as_str).collect::<Vec<_>>()==expected,"wand prerequisite source differs");
            "jcm2606.thaumicmachina.core.crafting.infusion.RecipeInfusionWandAugmentation"
        },
    };
    ensure!(p.native_recipe==expected,"native infusion law/source mismatch");
    let enhanced_owner=p.native_recipe=="net.glease.tc4tweak.api.infusionrecipe.EnhancedInfusionRecipe";
    ensure!(p.pattern.startup==if enhanced_owner{StartupMatcher::Enhanced}else if matches!(p.effect,Effect::Wand{..}){StartupMatcher::ItemAndTags}else{StartupMatcher::Legacy},"infusion startup matcher differs from its native owner");
    if enhanced{super::infusion_ingredient::validate(p.pattern.predicates.as_ref().context("enhanced predicates missing")?)?;}
    else{ensure!(p.pattern.predicates.is_none(),"ordinary infusion has enhanced predicates");}
    ensure!(p.pedestal_limit>0&&p.pedestals.len()<=4096&&(p.native_recipe=="com.gtnewhorizon.gtnhlib.api.thaumcraft.EnhancedInfusionRecipe"||p.pedestals.is_empty()&&!p.replacements_active),"unexpected native pedestal replacement table, activation or limit");
    for rule in &p.pedestals {
        for template in rule.input.iter().chain(rule.output.iter().map(|o|&o.item)) {
            crate::identity::registry_name(&template.registry)?;
            if let Some(nbt)=&template.nbt{nbt.validate()?;compound(Some(nbt))?;}
        }
    }
    ensure!(p.pattern.allow_extra==(enhanced||matches!(p.effect,Effect::Familiar(_))),"native infusion extra-pedestal policy differs");
    let predicate=!enhanced&&matches!(p.effect,Effect::Declared|Effect::Golem{..});
    ensure!(if predicate{p.pattern.central.as_ref().is_some_and(|c|c.template==p.declared_input)}else{p.pattern.central.is_none()},"native central predicate differs");
    validate_pattern(&p.pattern,items)?;
    let magic=r.magic.as_ref().context("native infusion magic missing")?;
    ensure!(magic.kind==MagicKind::Infusion&&magic.central==Some(0)&&magic.payment.is_none()&&!magic.creative
        &&magic.research.len()<=1&&(!matches!(p.effect,Effect::Revolver{..})||magic.research.len()==1),"native infusion research or kind differs");
    let base:BTreeMap<_,_>=p.base_aspects.iter().map(|a|Ok((a.aspect.as_str(),crate::identity::integer(&a.amount,i32::MIN as i64,i32::MAX as i64)? as i32))).collect::<Result<_>>()?;
    ensure!(base.len()==p.base_aspects.len()&&base.len()<=4096,"duplicate native base aspect");
    if let Some(random)=&p.random_aspects {
        ensure!(p.native_recipe=="thaumcraft.api.crafting.InfusionRecipe"&&matches!(p.effect,Effect::Declared)
            &&p.base_aspects.is_empty()&&magic.aspects.is_empty(),"random infusion costs cannot acquire a fixed sample price");
        random.ranges()?;
    }
    for (slot,input) in r.inputs.iter().enumerate() {
        ensure!(input.kind==Kind::Item&&input.slot==slot as u32&&!input.choices.is_empty(),"native infusion specimen input missing");
        for c in &input.choices {
            ensure!(c.amount=="1"&&matches!(c.rule,Match::Exact)&&(slot!=0||matches!(c.consume,Consumption::Stack)&&c.returns.is_empty()),"native infusion choices are exact one-item specimens");
            let item=items.get(c.id.as_str()).context("native infusion specimen missing")?;state(&p.pattern,&c.id)?;
            ensure!(if enhanced {enhanced_specimen(p,slot,item)?}
                else if slot==0 {observed_admitted(p,item)?&&match &p.pattern.central{None=>true,Some(req)=>legacy_match(&p.pattern,req,item,1,items)?}}
                else {legacy_match(&p.pattern,&p.pattern.components[slot-1],item,1,items)?&&legacy_phase(&p.pattern,&p.pattern.components[slot-1],item,1,true)?},"native infusion specimen does not satisfy its startup/cycle rules");
            if slot>0{validate_pedestal(p,c,item,items)?;}
            if slot==0 {
                let level=level(&p.effect,item.nbt.as_ref())?;
                let actual:BTreeMap<_,_>=magic.aspects.iter().map(|a|Ok((a.aspect.as_str(),crate::identity::integer(&a.amount,i32::MIN as i64,i32::MAX as i64)? as i32))).collect::<Result<_>>()?;
                let costs:BTreeMap<_,_>=base.iter().map(|(k,v)|(*k,if matches!(p.effect,Effect::Revolver{..}){doubled(*v,level)}else{*v})).collect();
                let instability=if let Effect::Revolver{target,upgrades,..}=&p.effect{p.base_instability.wrapping_add(upgrades[*target as usize].instability.wrapping_mul(level))}else{p.base_instability};
                ensure!(actual.len()==magic.aspects.len()&&actual==costs&&magic.instability==Some(instability),"native infusion sample costs differ from its center state");
            }
        }
    }
    let out=&r.outputs[0];
    ensure!(out.kind==Kind::Item&&out.slot==0&&out.role==OutputRole::Result&&out.quantity.is_none()&&out.chance.numerator=="1"&&out.chance.denominator=="1","invalid native infusion result");
    let outer:Vec<_>=r.inputs.iter().skip(1).map(|i|items.get(i.choices[0].id.as_str()).copied().context("missing first pedestal specimen")).collect::<Result<_>>()?;
    ensure!(matches(p,items[r.inputs[0].choices[0].id.as_str()],&outer,items)?,"first infusion specimen tuple does not match");
    if matches!(p.effect,Effect::Declared) {ensure!(!out.change.as_ref().is_some_and(|c|matches!(c.action,Edit::NativeInfusion{})),"declared infusion cannot use an upgrade edit");return Ok(());}
    let change=out.change.as_ref().context("native infusion upgrade edit missing")?;
    ensure!(matches!(change.action,Edit::NativeInfusion{})&&change.input==0&&change.bindings.is_none()
        &&change.samples.len()==r.inputs[0].choices.len()&&out.amount.as_deref()==Some("1")&&out.id==change.samples[0].id,"invalid infusion result bindings");
    for (c,s) in r.inputs[0].choices.iter().zip(&change.samples) {
        let original=items[c.id.as_str()];let result=items.get(s.id.as_str()).context("native infusion result fact missing")?;
        ensure!(s.amount=="1"&&result.registry==original.registry&&result.meta==original.meta&&result.nbt==Some(observed_transform(p,original)?),"native infusion NBT transition differs");
    }
    Ok(())
}
fn enhanced_specimen(p:&NativeInfusion,slot:usize,item:&Item)->Result<bool>{
    let predicates=p.pattern.predicates.as_ref().context("enhanced predicates missing")?;
    if slot==0{return ingredient_match(&predicates[0],item);}
    for predicate in predicates.iter().skip(1){if ingredient_match(predicate,item)?{return Ok(true);}}
    for required in &p.pattern.components{if legacy_phase(&p.pattern,required,item,1,true)?{return Ok(true);}}
    Ok(false)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn ingredient_item(registry:&str,meta:i32,nbt:Option<Nbt>,tags:&[&str])->Item {
        Item{id:format!("item_{registry}_{meta}"),registry:registry.into(),meta,nbt,name:"text_name".into(),tooltip:vec![],
            stack_limit:64,durability:0,tools:BTreeMap::new(),armor:false,tags:tags.iter().map(|v|(*v).into()).collect(),icon:None,order:None,aspects:None}
    }
    #[test]
    fn enhanced_wildcard_items_keep_exact_nbt_checks_inside_or() {
        let tag=compound([("key",Nbt::Int{value:"1".into()})]);
        let predicate=IngredientPredicate::Any{options:vec![IngredientPredicate::Never,IngredientPredicate::Items{
            templates:vec![Template{registry:"native:wood".into(),metadata:32767,nbt:Some(tag.clone())}],nbt:true}]};
        assert!(ingredient_match(&predicate,&ingredient_item("native:wood",7,Some(tag),&[])).unwrap());
        assert!(!ingredient_match(&predicate,&ingredient_item("native:wood",7,None,&[])).unwrap());
        assert!(!ingredient_match(&predicate,&ingredient_item("native:wood",7,Some(compound([("key",Nbt::Int{value:"2".into()})])),&[])).unwrap());
    }
    #[test]
    fn enhanced_greedy_matching_accepts_extras_but_does_not_backtrack() {
        let predicates=vec![IngredientPredicate::Ore{name:"a".into(),exclusive:false},IngredientPredicate::Ore{name:"b".into(),exclusive:false}];
        let both=ingredient_item("native:both",0,None,&["a","b"]);
        let only_a=ingredient_item("native:a",0,None,&["a"]);
        let extra=ingredient_item("native:extra",0,None,&[]);
        assert!(!enhanced_components(&predicates,&[&both,&only_a]).unwrap());
        assert!(enhanced_components(&predicates,&[&only_a,&both,&extra]).unwrap());
        assert!(!ingredient_match(&IngredientPredicate::Ore{name:"a".into(),exclusive:true},&both).unwrap());
    }
    fn compound(rows: impl IntoIterator<Item=(&'static str,Nbt)>)->Nbt {
        Nbt::Compound { value: rows.into_iter().map(|(k,v)|(k.into(),v)).collect() }
    }
    #[test]
    fn golem_scalar_upgrade_container_is_detached_in_native_code() {
        let effect=Effect::Golem {namespace:"gadomancy".into(),key:"runicShield".into()};
        let original=compound([("gadomancy",compound([("upgrades",Nbt::Int {value:"7".into()})]))]);
        assert!(admitted(&effect,Some(&original),"native:golem").unwrap());
        assert_eq!(transform(&effect,Some(&original)).unwrap(),original);
        let tagged=compound([("gadomancy",compound([("upgrades",compound([("runicShield",Nbt::Byte {value:"0".into()})]))]))]);
        assert!(!admitted(&effect,Some(&tagged),"native:golem").unwrap(),"presence, not boolean truth");
    }
    #[test]
    fn revolver_registry_identity_negative_ids_and_int_wrapping_are_preserved() {
        let effect=Effect::Revolver {target:1,registered:vec![Some(0)],upgrades:vec![
            RevolverUpgrade {id:0,max_level:2,instability:1,conflicts:vec![]},
            RevolverUpgrade {id:0,max_level:2,instability:3,conflicts:vec![]}]};
        let tags=compound([("tb.upgrades",Nbt::List {element:"compound".into(),value:vec![compound([
            ("id",Nbt::Int {value:"0".into()}),("level",Nbt::Int {value:"9".into()})]) ]})]);
        assert_eq!(level(&effect,Some(&tags)).unwrap(),0,"equal numeric ids are not equal objects");
        let bad=compound([("tb.upgrades",Nbt::List {element:"compound".into(),value:vec![compound([
            ("id",Nbt::Int {value:"-1".into()})]) ]})]);
        assert!(level(&effect,Some(&bad)).is_err(),"native array access fails on negative id");
        assert_eq!(doubled(i32::MAX,1),-2);
        assert_eq!(doubled(3,32),0);
        assert_eq!(doubled(3,-1),3);
    }
    #[test]
    fn familiar_first_alias_last_prerequisite_and_copied_order_are_distinct() {
        let mut familiar=Familiar{target:0,names:BTreeMap::from([("shock".into(),Some(0))]),items:vec!["native:familiar".into()],
            required_previous_level:1,locale:"en-US".into(),augments:vec![FamiliarAugment{name:"shock".into(),requires_previous:false,conflicts:vec![],conditions:vec![]}]};
        let item=Item{id:"item_familiar".into(),registry:"native:familiar".into(),meta:0,
            nbt:Some(compound([("augments",compound([("shock",Nbt::Int{value:"1".into()}),("SHOCK",Nbt::Int{value:"2".into()})]))])),
            name:"text_name".into(),tooltip:vec![],stack_limit:1,durability:0,tools:BTreeMap::new(),armor:false,tags:vec![],icon:None,order:None,aspects:None};
        let state=ItemState{id:item.id.clone(),ores:vec![],wildcard_ores:vec![],tag_strings:BTreeMap::new(),copied_tag_strings:BTreeMap::new(),pedestal_tag_strings:BTreeMap::new(),
            augment_keys:vec!["shock".into(),"SHOCK".into()],copied_augment_keys:vec!["SHOCK".into(),"shock".into()],
            lowercase_keys:BTreeMap::from([("shock".into(),"shock".into()),("SHOCK".into(),"shock".into())])};
        assert!(familiar_admitted(&familiar,&item,&state).unwrap());
        familiar.augments[0].requires_previous=true;
        assert!(!familiar_admitted(&familiar,&item,&state).unwrap(),"first level is 1, but the last alias is level 2");
        familiar.augments[0].requires_previous=false;
        let p=NativeInfusion{native_recipe:"native".into(),declared_input:Template{registry:item.registry.clone(),metadata:0,nbt:None},
            pattern:LegacyPattern{mode:OreMode::None,startup:StartupMatcher::Legacy,predicates:None,central:None,components:vec![],states:vec![state],allow_extra:true},
            effect:Effect::Familiar(familiar),base_aspects:vec![],random_aspects:None,base_instability:0,pedestals:vec![],pedestal_limit:1,replacements_active:false};
        let result=observed_transform(&p,&item).unwrap();
        assert_eq!(child(child(Some(&result)).unwrap().get("augments")).unwrap().get("shock"),Some(&Nbt::Int{value:"3".into()}),"output reads the copied order");
    }
    #[test]
    fn familiar_unknown_alias_error_depends_on_lookup_order() {
        assert!(familiar_first(&[(None,0),(Some(0),1)],0).is_err());
        assert_eq!(familiar_first(&[(Some(0),1),(None,0)],0).unwrap(),Some(1));
    }
    #[test]
    fn pedestal_retention_reverses_the_wildcard_target_and_ignores_nbt() {
        let offered=Template{registry:"native:pearl".into(),metadata:4,nbt:None};
        let wildcard=Template{metadata:32767,..offered.clone()};
        assert!(replacement_match(&wildcard,&offered,false));
        assert!(!replacement_match(&offered,&wildcard,false),"retention uses offered as target");
        let changed_tags=Template{nbt:Some(compound([("changed",Nbt::Byte{value:"1".into()})])),..offered.clone()};
        assert!(replacement_match(&offered,&changed_tags,true),"same item/meta retains original tags instead of writing the new ones");
    }
    #[test]
    fn machina_startup_ignores_metadata_but_preserves_asymmetric_required_tags() {
        let required=Template{registry:"native:core".into(),metadata:14,nbt:Some(compound([("key",Nbt::Int{value:"2".into()})]))};
        let tags=BTreeMap::from([("key".into(),"2".into())]);
        assert!(machina_component(&required,&tags,"native:core",true,&tags));
        assert!(!machina_component(&required,&tags,"native:other",true,&tags));
        assert!(!machina_component(&required,&tags,"native:core",false,&BTreeMap::new()));
    }
    #[test]
    fn machina_augmentation_replaces_malformed_lists_and_checks_names() {
        let effect=Effect::Wand{name:"Taint Capping".into(),prerequisites:vec!["Tainted Core".into()],definition_wand:true};
        let bad=compound([("Augmentations",Nbt::Int{value:"4".into()})]);
        assert!(!admitted(&effect,Some(&bad),"native:any").unwrap());
        let result=transform(&effect,Some(&bad)).unwrap();
        assert_eq!(child(Some(&result)).unwrap().get("Augmentations"),Some(&Nbt::List{element:"string".into(),value:vec![Nbt::String{value:"Taint Capping".into()}]}));
        let ready=compound([("Augmentations",Nbt::List{element:"string".into(),value:vec![Nbt::String{value:"Tainted Core".into()}]})]);
        assert!(admitted(&effect,Some(&ready),"native:any").unwrap(),"native matcher does not restrict the offered center class");
        assert!(!admitted(&effect,Some(&transform(&effect,Some(&ready)).unwrap()),"native:any").unwrap());
    }
}
