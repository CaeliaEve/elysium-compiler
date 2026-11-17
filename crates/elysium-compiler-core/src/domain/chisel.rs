//! Chisel 2.16.15-GTNH native lookup partitions and pickup/tool transitions.
use super::{Item,Output,Recipe};
use crate::identity::Nbt;
use anyhow::Result;
use schemars::JsonSchema;
use serde::{Deserialize,Serialize};
use std::collections::BTreeMap;

macro_rules! native_enum {
    ($name:ident,$($variant:ident=>$value:literal),+$(,)?)=>{
        #[derive(Clone,Copy,Debug,PartialEq,Eq,Serialize,Deserialize,JsonSchema)]
        pub enum $name{$(#[serde(rename=$value)]$variant),+}
    };
}
native_enum!(ChiselMode,Single=>"single",Stack=>"stack");
native_enum!(Selection,Native=>"nativeLookupExactPartitionsThenWildcard");
native_enum!(SourceNbt,Ignored=>"ignored");
native_enum!(CanChisel,Native=>"nativeItemChiselAlways");
native_enum!(OnChisel,Native=>"configuredAllowChiselDamage");
native_enum!(DamagePolicy,Native=>"vanillaCreativeUnbreakingUnbreakable");
native_enum!(TargetTag,Native=>"chiselTarget");
native_enum!(ToolOwner,Native=>"team.chisel.item.chisel.ItemChisel");
#[derive(Clone,Debug,Serialize,Deserialize,JsonSchema)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct TargetSelection{pub item:String,pub count:i32}
#[derive(Clone,Debug,Serialize,Deserialize,JsonSchema)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct SourceSelection{pub item:String,pub native_id:i32,pub native_wildcard:bool,pub targets:Vec<TargetSelection>}
#[derive(Clone,Debug,Serialize,Deserialize,JsonSchema)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct Tool{pub item:String,pub owner:ToolOwner,pub damage_enabled:bool,pub max_damage:i32}
#[derive(Clone,Debug,Serialize,Deserialize,JsonSchema)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct ChiselProcess{
    pub mode:ChiselMode,pub source_slot:u32,pub tool_slot:u32,pub target:String,pub target_count:i32,
    pub selection:Selection,pub source_nbt:SourceNbt,pub can_chisel:CanChisel,pub on_chisel:OnChisel,
    pub damage_policy:DamagePolicy,pub damage_points:u32,pub target_tag:TargetTag,
    pub sources:Vec<SourceSelection>,pub tools:Vec<Tool>,
}
#[derive(Clone,Debug,PartialEq)]
pub struct ReplayedTool{pub id:String,pub registry:String,pub meta:i32,pub nbt:Option<Nbt>,pub amount:i32}
#[derive(Clone,Debug,PartialEq)]
pub struct CarvingStep{pub output:ReplayedTool,pub remaining_source:Option<ReplayedTool>,pub tool:Option<ReplayedTool>}
pub fn validate(process:&ChiselProcess,recipe:&Recipe,items:&BTreeMap<&str,&Item>)->Result<()>{
    use super::{Consumption,Kind,Match,OutputRole,Quantity};
    anyhow::ensure!(recipe.source.owner=="chisel"&&recipe.source.handler=="team.chisel.compat.nei.RecipeHandlerChisel","Chisel process is attached to an unadapted handler");
    anyhow::ensure!(process.source_slot==0&&process.tool_slot==1&&process.damage_points==1&&process.target_count>0,"Invalid native Chisel allocation/count/damage");
    anyhow::ensure!(!process.sources.is_empty()&&process.sources.len()<=65536&&!process.tools.is_empty()&&process.tools.len()<=65536,"Chisel source/tool partitions exceed budget");
    let target=items.get(process.target.as_str()).ok_or_else(||anyhow::anyhow!("Missing Chisel selected target"))?;
    let mut source_keys=std::collections::BTreeSet::new();let mut numeric_ids=BTreeMap::new();
    for source in &process.sources{
        let anchor=items.get(source.item.as_str()).ok_or_else(||anyhow::anyhow!("Missing Chisel source partition"))?;
        anyhow::ensure!(anchor.nbt.is_none()&&(!source.native_wildcard||anchor.meta==32767)&&source_keys.insert((&anchor.registry,anchor.meta,source.native_wildcard)),"Invalid or duplicate native Chisel source partition");
        anyhow::ensure!((0..=32767).contains(&source.native_id)&&source.targets.len()<=1,"Invalid native Chisel item ID/target projection");
        if let Some(previous)=numeric_ids.insert(&anchor.registry,source.native_id){anyhow::ensure!(previous==source.native_id,"Chisel registry has inconsistent native numeric IDs");}
        for observed in &source.targets{anyhow::ensure!(items.contains_key(observed.item.as_str())&&observed.item==process.target&&observed.count==process.target_count,"Invalid observed Chisel target projection/count");}
    }
    let mut tool_keys=std::collections::BTreeSet::new();
    for tool in &process.tools{
        let anchor=items.get(tool.item.as_str()).ok_or_else(||anyhow::anyhow!("Missing Chisel tool"))?;
        anyhow::ensure!(anchor.meta==0&&anchor.nbt.is_none()&&tool.max_damage>=0&&anchor.durability==tool.max_damage
            &&(tool.damage_enabled||tool.max_damage==0)&&tool_keys.insert(&anchor.registry),"Invalid canonical Chisel tool/durability policy");
        anyhow::ensure!(tool.damage_enabled==process.tools[0].damage_enabled,"Chisel tools disagree on the native global damage configuration");
    }
    anyhow::ensure!(recipe.inputs.len()==2&&recipe.outputs.len()==2&&recipe.duration.is_none()&&recipe.energy.is_none()&&recipe.grid.is_none()&&recipe.magic.is_none(),"Chisel row changed allocation or acquired machine costs");
    let source=&recipe.inputs[0];let tool=&recipe.inputs[1];
    anyhow::ensure!(source.kind==Kind::Item&&source.slot==0&&tool.kind==Kind::Item&&tool.slot==1,"Chisel slots changed");
    let expected:Vec<_>=process.sources.iter().filter(|s|s.targets.iter().any(|t|t.item==process.target&&t.count==process.target_count)).collect();
    anyhow::ensure!(source.choices.len()==expected.len()&&!expected.is_empty(),"Chisel source choices omit native selection partitions");
    for(choice,decision)in source.choices.iter().zip(expected){
        anyhow::ensure!(choice.id==decision.item&&choice.amount=="1"&&choice.returns.is_empty()
            &&matches!(choice.rule,Match::ChiselSource{meta,nbt} if meta==decision.native_wildcard&&nbt)
            &&match process.mode{ChiselMode::Single=>matches!(choice.consume,Consumption::Consume),ChiselMode::Stack=>matches!(choice.consume,Consumption::Stack)},"Chisel source choice changed native matcher or consumption");
    }
    anyhow::ensure!(tool.choices.len()==process.tools.len(),"Chisel tool choices omit compatible registered tools");
    for(choice,policy)in tool.choices.iter().zip(&process.tools){
        anyhow::ensure!(choice.id==policy.item&&choice.amount=="1"&&choice.returns.is_empty()&&matches!(choice.consume,Consumption::Keep)
            &&matches!(choice.rule,Match::ChiselTool{source} if source==0),"Chisel tool lost correlated admission/state retention");
    }
    let output=&recipe.outputs[0];let returned=&recipe.outputs[1];
    let expected_id=match process.mode{ChiselMode::Single=>target.id.clone(),ChiselMode::Stack=>crate::identity::item_id(&target.registry,target.meta,None)?};
    let expected_count=match process.mode{ChiselMode::Single=>process.target_count,ChiselMode::Stack=>1}.to_string();
    anyhow::ensure!(output.kind==Kind::Item&&output.slot==0&&matches!(output.role,OutputRole::Result)&&output.id==expected_id&&output.amount.is_none()&&output.change.is_none()
        &&output.chance.numerator=="1"&&output.chance.denominator=="1"
        &&matches!(&output.quantity,Some(Quantity::Potential{stat,condition,sample,parameters,nominal})
            if stat=="chisel:selection"&&condition.as_deref()==Some("compatibleToolAndNativeTarget")&&sample.as_ref()==Some(&expected_count)&&nominal==&expected_count&&parameters.is_none()),"Chisel selected target became a guaranteed or unrelated yield");
    anyhow::ensure!(returned.kind==Kind::Item&&returned.slot==1&&matches!(returned.role,OutputRole::Return)&&returned.quantity.is_none()&&returned.change.is_some()
        &&returned.chance.numerator=="1"&&returned.chance.denominator=="1","Chisel tool state is not a native conditional return");
    Ok(())
}
fn selected<'a>(process:&'a ChiselProcess,offered:&Item,items:&BTreeMap<&str,&Item>)->Option<&'a SourceSelection>{
    process.sources.iter().find(|s|!s.native_wildcard&&items.get(s.item.as_str()).is_some_and(|i|i.registry==offered.registry&&i.meta==offered.meta))
        .or_else(||process.sources.iter().find(|s|s.native_wildcard&&items.get(s.item.as_str()).is_some_and(|i|i.registry==offered.registry)))
}
pub fn accepts_source(process:&ChiselProcess,offered:&Item,items:&BTreeMap<&str,&Item>)->bool{
    matches!(&offered.nbt,None|Some(Nbt::Compound{..}))&&selected(process,offered,items).is_some_and(|s|s.targets.iter().any(|t|t.item==process.target&&t.count==process.target_count))
}
pub fn accepts_tool(process:&ChiselProcess,offered:&Item,items:&BTreeMap<&str,&Item>)->bool{
    matches!(&offered.nbt,None|Some(Nbt::Compound{..}))&&process.tools.iter().any(|t|items.get(t.item.as_str()).is_some_and(|i|i.registry==offered.registry))
}
pub fn validate_change(recipe:&Recipe,output:&Output,items:&BTreeMap<&str,&Item>)->Result<()>{
    use super::{Edit,Kind,OutputRole,Process};
    let Some(Process::Chisel(process))=&recipe.process else{anyhow::bail!("Chisel tool change lacks native process")};
    validate(process,recipe,items)?;
    let change=output.change.as_ref().ok_or_else(||anyhow::anyhow!("Missing Chisel tool change"))?;
    anyhow::ensure!(output.slot==1&&output.kind==Kind::Item&&matches!(output.role,OutputRole::Return)&&change.input==1&&change.bindings.is_none()
        &&matches!(&change.action,Edit::ChiselTool{source,mode} if *source==0&&*mode==process.mode),"Chisel tool change is not tied to the native source/mode");
    anyhow::ensure!(change.samples.len()==process.tools.len(),"Chisel tool samples omit compatible tools");
    let source=items.get(recipe.inputs[0].choices[0].id.as_str()).ok_or_else(||anyhow::anyhow!("Missing Chisel source example"))?;
    for(policy,sample)in process.tools.iter().zip(&change.samples){
        let tool=items.get(policy.item.as_str()).ok_or_else(||anyhow::anyhow!("Missing Chisel canonical tool"))?;
        let expected=replay_tool(process,tool,Some(source),1,false,true,items)?.ok_or_else(||anyhow::anyhow!("Canonical Chisel example unexpectedly broke"))?;
        anyhow::ensure!(sample.id==expected.id&&sample.amount=="1"&&items.contains_key(sample.id.as_str()),"Chisel tool sample differs from native state replay");
    }
    let first=&change.samples[0];
    anyhow::ensure!(output.id==first.id&&output.amount.as_deref()==Some(first.amount.as_str()),"Chisel return differs from its first native tool sample");
    Ok(())
}
// damage_applied is the observed native Unbreaking roll after onChisel; false skips wear.
// The world/player context is explicit, so a preview never promises tool survival.
#[allow(clippy::too_many_arguments)]
pub fn replay_tool(process:&ChiselProcess,offered:&Item,source:Option<&Item>,source_amount:i32,creative:bool,damage_applied:bool,items:&BTreeMap<&str,&Item>)->Result<Option<ReplayedTool>>{
    replay_tool_stack(process,offered,1,source,source_amount,creative,damage_applied,items)
}
#[allow(clippy::too_many_arguments)]
pub fn replay_tool_stack(process:&ChiselProcess,offered:&Item,tool_amount:i32,source:Option<&Item>,source_amount:i32,creative:bool,damage_applied:bool,items:&BTreeMap<&str,&Item>)->Result<Option<ReplayedTool>>{
    anyhow::ensure!(tool_amount>0,"Chisel tool amount must be positive");
    anyhow::ensure!(source_amount>0,"Chisel source amount must be positive");
    let source=source.ok_or_else(||anyhow::anyhow!("Chisel tool transition lacks source"))?;
    anyhow::ensure!(accepts_source(process,source,items)&&accepts_tool(process,offered,items),"Chisel source/tool is not admitted by native selection");
    let policy=process.tools.iter().find(|t|items.get(t.item.as_str()).is_some_and(|i|i.registry==offered.registry)).unwrap();
    let mut tags=match &offered.nbt{None=>BTreeMap::new(),Some(Nbt::Compound{value})=>value.clone(),_=>anyhow::bail!("Chisel tool NBT must be a compound")};
    let remaining=match process.mode{ChiselMode::Single=>source_amount-1,ChiselMode::Stack=>0};
    let mut target=BTreeMap::new();
    if remaining>0{
        let selected=selected(process,source,items).unwrap();
        target.insert("id".into(),Nbt::Short{value:(selected.native_id as i16).to_string()});
        target.insert("Count".into(),Nbt::Byte{value:(remaining as i8).to_string()});
        target.insert("Damage".into(),Nbt::Short{value:(source.meta as i16).to_string()});
        if let Some(nbt)=&source.nbt{anyhow::ensure!(matches!(nbt,Nbt::Compound{..}),"Chisel source NBT must be a compound");target.insert("tag".into(),nbt.clone());}
    }
    tags.insert("chiselTarget".into(),Nbt::Compound{value:target});
    let unbreakable=super::change::number(tags.get("Unbreakable"))? as i8!=0;
    let mut meta=offered.meta;let mut amount=tool_amount;
    if policy.damage_enabled&&policy.max_damage>0&&!creative&&!unbreakable&&damage_applied{
        meta=meta.wrapping_add(1);
        if meta>policy.max_damage{amount-=1;meta=0;if amount==0{return Ok(None)}}
    }
    let nbt=Some(Nbt::Compound{value:tags});let id=crate::identity::item_id(&offered.registry,meta,nbt.as_ref())?;
    Ok(Some(ReplayedTool{id,registry:offered.registry.clone(),meta,nbt,amount}))
}
/// One native pickup; whole-stack conversion still invokes the tool callback exactly once.
#[allow(clippy::too_many_arguments)]
pub fn replay_selection(process:&ChiselProcess,source:&Item,source_amount:i32,tool:&Item,tool_amount:i32,creative:bool,damage_applied:bool,items:&BTreeMap<&str,&Item>)->Result<CarvingStep>{
    let changed=replay_tool_stack(process,tool,tool_amount,Some(source),source_amount,creative,damage_applied,items)?;
    let target=items.get(process.target.as_str()).ok_or_else(||anyhow::anyhow!("Missing Chisel native selected target"))?;
    let (nbt,amount)=match process.mode{ChiselMode::Single=>(target.nbt.clone(),process.target_count),ChiselMode::Stack=>(None,source_amount)};
    let id=crate::identity::item_id(&target.registry,target.meta,nbt.as_ref())?;
    let output=ReplayedTool{id,registry:target.registry.clone(),meta:target.meta,nbt,amount};
    let remaining_source=if process.mode==ChiselMode::Single&&source_amount>1{Some(ReplayedTool{id:source.id.clone(),registry:source.registry.clone(),meta:source.meta,nbt:source.nbt.clone(),amount:source_amount-1})}else{None};
    Ok(CarvingStep{output,remaining_source,tool:changed})
}
#[cfg(test)]
mod tests{
    use super::*;
    use serde_json::json;
    fn item(registry:&str,meta:i32,nbt:Option<Nbt>)->Item{
        Item{id:crate::identity::item_id(registry,meta,nbt.as_ref()).unwrap(),registry:registry.into(),meta,nbt,name:"text_fixture".into(),tooltip:vec![],stack_limit:64,durability:5,tools:BTreeMap::new(),armor:false,tags:vec![],icon:None,order:None,aspects:None}
    }
    fn fixture()->(ChiselProcess,Vec<Item>){
        let items=vec![item("fixture:source",32767,None),item("fixture:source",4,None),item("fixture:target",0,None),item("fixture:other",0,None),item("chisel:iron",0,None)];
        let p:ChiselProcess=serde_json::from_value(json!({
            "mode":"single","sourceSlot":0,"toolSlot":1,"target":items[2].id,"targetCount":1,
            "selection":"nativeLookupExactPartitionsThenWildcard","sourceNbt":"ignored",
            "canChisel":"nativeItemChiselAlways","onChisel":"configuredAllowChiselDamage",
            "damagePolicy":"vanillaCreativeUnbreakingUnbreakable","damagePoints":1,"targetTag":"chiselTarget",
            "sources":[{"item":items[0].id,"nativeId":17,"nativeWildcard":true,"targets":[{"item":items[2].id,"count":1}]},
                       {"item":items[1].id,"nativeId":17,"nativeWildcard":false,"targets":[{"item":items[3].id,"count":1}]}],
            "tools":[{"item":items[4].id,"owner":"team.chisel.item.chisel.ItemChisel","damageEnabled":true,"maxDamage":5}]
        })).unwrap();
        (p,items)
    }
    #[test]fn exact_native_metadata_partition_overrides_wildcard_and_ignores_nbt(){
        let(p,items)=fixture();let map=items.iter().map(|i|(i.id.as_str(),i)).collect();
        let mut offered=item("fixture:source",4,Some(Nbt::Compound{value:BTreeMap::from([("extra".into(),Nbt::Int{value:"42".into()})])}));
        assert!(!accepts_source(&p,&offered,&map),"earlier exact ore selection was bypassed");
        offered.meta=5;
        assert!(accepts_source(&p,&offered,&map),"native wildcard partition rejected unrelated input tags");
        let mut changed=p.clone();changed.target_count=2;
        assert!(!accepts_source(&changed,&offered,&map),"native target count was discarded");
    }
    #[test]fn tool_state_retains_remaining_source_and_wraps_native_nbt_count(){
        let(p,items)=fixture();let map=items.iter().map(|i|(i.id.as_str(),i)).collect();
        let mut offered=items[4].clone();offered.meta=3;offered.nbt=Some(Nbt::Compound{value:BTreeMap::from([("owner".into(),Nbt::String{value:"keep".into()})])});
        let source=item("fixture:source",5,Some(Nbt::Compound{value:BTreeMap::from([("fixture".into(),Nbt::Int{value:"42".into()})])}));
        let out=replay_tool(&p,&offered,Some(&source),130,false,true,&map).unwrap().unwrap();
        assert_eq!(out.meta,4);
        let Some(Nbt::Compound{value})=out.nbt else{panic!("tool compound missing")};
        assert_eq!(value["owner"],Nbt::String{value:"keep".into()});
        let Nbt::Compound{value:target}=&value["chiselTarget"]else{panic!("target missing")};
        assert_eq!(target["id"],Nbt::Short{value:"17".into()});
        assert_eq!(target["Count"],Nbt::Byte{value:"-127".into()});
        assert_eq!(target["Damage"],Nbt::Short{value:"5".into()});
        assert_eq!(target["tag"],source.nbt.unwrap());
    }
    #[test]fn tool_wear_respects_creative_unbreakable_config_roll_and_break(){
        let(mut p,items)=fixture();let map=items.iter().map(|i|(i.id.as_str(),i)).collect();
        let source=item("fixture:source",5,None);let mut tool=items[4].clone();tool.meta=5;
        assert!(replay_tool(&p,&tool,Some(&source),1,false,true,&map).unwrap().is_none(),"native >maxDamage break was lost");
        let retained=replay_tool_stack(&p,&tool,3,Some(&source),1,false,true,&map).unwrap().unwrap();
        assert_eq!((retained.meta,retained.amount),(0,2),"native stacked tool break count/reset changed");
        for(creative,roll)in[(true,true),(false,false)]{
            let out=replay_tool(&p,&tool,Some(&source),1,creative,roll,&map).unwrap().unwrap();assert_eq!(out.meta,5);
        }
        tool.nbt=Some(Nbt::Compound{value:BTreeMap::from([("Unbreakable".into(),Nbt::Byte{value:"1".into()})])});
        assert_eq!(replay_tool(&p,&tool,Some(&source),1,false,true,&map).unwrap().unwrap().meta,5);
        p.tools[0].damage_enabled=false;tool.nbt=None;
        assert_eq!(replay_tool(&p,&tool,Some(&source),1,false,true,&map).unwrap().unwrap().meta,5);
        p.mode=ChiselMode::Stack;
        let out=replay_tool(&p,&tool,Some(&source),8,false,true,&map).unwrap().unwrap();
        let Some(Nbt::Compound{value})=out.nbt else{panic!("tool compound missing")};
        assert_eq!(value["chiselTarget"],Nbt::Compound{value:BTreeMap::new()});
    }
    #[test]fn native_pickup_modes_keep_single_ghost_count_but_drop_stack_output_tags(){
        let(mut p,mut items)=fixture();
        items[2]=item("fixture:target",0,Some(Nbt::Compound{value:BTreeMap::from([("nativeTarget".into(),Nbt::Int{value:"9".into()})])}));
        p.target=items[2].id.clone();p.target_count=4;p.sources[0].targets[0]=TargetSelection{item:p.target.clone(),count:4};
        let map=items.iter().map(|i|(i.id.as_str(),i)).collect();let source=item("fixture:source",5,None);
        let single=replay_selection(&p,&source,8,&items[4],1,false,true,&map).unwrap();
        assert_eq!(single.output.amount,4);assert_eq!(single.output.nbt,items[2].nbt);
        assert_eq!(single.remaining_source.unwrap().amount,7);assert_eq!(single.tool.unwrap().meta,1);
        p.mode=ChiselMode::Stack;
        let stack=replay_selection(&p,&source,8,&items[4],1,false,true,&map).unwrap();
        assert_eq!(stack.output.amount,8);assert!(stack.output.nbt.is_none()&&stack.remaining_source.is_none());
        assert_eq!(stack.tool.unwrap().meta,1,"Whole-stack pickup applied wear per output item");
    }
}
