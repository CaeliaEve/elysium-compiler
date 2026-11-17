//! Installed Matter Manipulator 0.0.51 law. Codec observations are native facts,
//! not a promise to reproduce arbitrary Gson/plugin state from canonical NBT.
use super::{Consumption,Edit,Item,Kind,Match,OutputRole,Recipe};
use crate::identity::Nbt;
use anyhow::{bail,ensure,Context,Result};
use base64::{engine::general_purpose::STANDARD,Engine};
use schemars::JsonSchema;
use serde::{Deserialize,Serialize};
use std::collections::{BTreeMap,BTreeSet};

#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize,JsonSchema)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct Upgrade {pub name:String,pub ordinal:u8,pub id:i32,pub bit:u16,pub provides_caps:i32}
#[derive(Clone,Debug,Serialize,Deserialize,JsonSchema)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct Mapping {pub metadata:i32,pub upgrade:Option<Upgrade>}
#[derive(Clone,Copy,Debug,PartialEq,Eq,Serialize,Deserialize,JsonSchema)]
#[serde(rename_all="camelCase")]
pub enum Origin {NativeItemList,RegisteredItemClass,DeclaredBitsetWitness}
#[derive(Clone,Copy,Debug,PartialEq,Eq,Serialize,Deserialize,JsonSchema)]
pub enum Codec {#[serde(rename="mm0051GsonV1")]Mm0051GsonV1}
#[derive(Clone,Debug,Serialize,Deserialize,JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct State {pub id:String,pub normalized:Nbt,pub origin:Origin}
#[derive(Clone,Debug,Serialize,Deserialize,JsonSchema)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct MatterManipulator {
    pub native_recipe:String,pub version:String,pub codec:Codec,pub tier:u8,pub manipulator:String,
    pub upgrade:Upgrade,pub upgrades:Vec<Upgrade>,pub allowed:Vec<Upgrade>,pub meta_items:Vec<String>,
    pub mappings:Vec<Mapping>,pub default_upgrade:Option<Upgrade>,pub states:Vec<State>,
}
const HANDLER:&str="com.recursive_pineapple.matter_manipulator.client.nei.UpdateRecipeNEIHandler";
fn compound(nbt:Option<&Nbt>)->Result<Option<&BTreeMap<String,Nbt>>>{match nbt{None=>Ok(None),Some(Nbt::Compound{value})=>Ok(Some(value)),_=>bail!("Matter root is not a compound")}}

/// Integer.parseInt iterates UTF-16 chars; JDK8 BMP decimal digits are admitted,
/// while supplementary digit code points are rejected as surrogate pairs.
fn java_int(value:&str)->Result<i32>{
    let mut chars=value.chars();let mut negative=false;let first=chars.next().context("empty native integer")?;
    let digits=if first=='-'||first=='+'{negative=first=='-';chars.collect::<String>()}else{value.to_owned()};
    ensure!(!digits.is_empty()&&digits.len()<=262144,"invalid native integer");
    let starts=[0x30,0x660,0x6f0,0x7c0,0x966,0x9e6,0xa66,0xae6,0xb66,0xbe6,0xc66,0xce6,0xd66,
        0xe50,0xed0,0xf20,0x1040,0x1090,0x17e0,0x1810,0x1946,0x19d0,0x1a80,0x1a90,0x1b50,0x1bb0,
        0x1c40,0x1c50,0xa620,0xa8d0,0xa900,0xa9d0,0xaa50,0xabf0,0xff10];
    let mut result=0i64;for c in digits.chars(){let code=c as u32;
        let start=starts.iter().find(|start|code>=**start&&code<**start+10).context("invalid native decimal digit")?;
        result=result*10+i64::from(code-*start);ensure!(result<=i64::from(i32::MAX)+i64::from(negative),"native integer overflow");
    }Ok(if negative{(-result) as i32}else{result as i32})
}
fn integer(tag:&Nbt)->Result<i32>{
    Ok(match tag {
        Nbt::Byte{value}|Nbt::Short{value}|Nbt::Int{value}|Nbt::Long{value}=>value.parse::<i64>()? as i32,
        Nbt::Float{value}=>f32::from_bits(u32::from_str_radix(value,16)?) as i32,
        Nbt::Double{value}=>f64::from_bits(u64::from_str_radix(value,16)?) as i32,
        Nbt::String{value}=>java_int(value)?,_=>bail!("Matter BitSet expects numeric/string primitive flags"),
    })
}
pub fn bits(root:Option<&Nbt>)->Result<Vec<bool>>{
    let Some(tag)=compound(root)?.and_then(|r|r.get("installedUpgrades")) else{return Ok(vec![]);};
    let mut bits=match tag {
        Nbt::ByteArray{value}=>STANDARD.decode(value)?.into_iter().map(|v|v!=0).collect::<Vec<_>>(),
        Nbt::IntArray{value}=>value.iter().map(|v|Ok(v.parse::<i32>()?!=0)).collect::<Result<Vec<_>>>()?,
        Nbt::List{value,..}=>value.iter().map(|v|Ok(integer(v)?!=0)).collect::<Result<Vec<_>>>()?,
        _=>bail!("Matter BitSet expects an array, not base64 text"),
    };
    ensure!(bits.len()<=65536,"Matter BitSet exceeds state budget");while bits.last()==Some(&false){bits.pop();}Ok(bits)
}
pub fn result(p:&MatterManipulator,item:&Item)->Result<Option<Nbt>>{
    let Some(state)=p.states.iter().find(|s|s.id==item.id)else{return Ok(None);};
    let mut tags=compound(Some(&state.normalized))?.context("Matter normalization is not a compound")?.clone();
    let mut flags=bits(Some(&state.normalized))?;let bit=p.upgrade.bit as usize;
    ensure!(!flags.get(bit).copied().unwrap_or(false),"Matter upgrade already installed");
    flags.resize(flags.len().max(bit+1),false);flags[bit]=true;
    tags.insert("installedUpgrades".into(),Nbt::ByteArray{value:STANDARD.encode(flags.into_iter().map(u8::from).collect::<Vec<_>>())});
    Ok(Some(Nbt::Compound{value:tags}))
}
fn lookup(p:&MatterManipulator,meta:i32)->Option<&Upgrade>{p.mappings.iter().find(|m|m.metadata==meta).map_or(p.default_upgrade.as_ref(),|m|m.upgrade.as_ref())}
fn compact(value:i32)->Nbt{
    if i8::try_from(value).is_ok(){Nbt::Byte{value:value.to_string()}}
    else if i16::try_from(value).is_ok(){Nbt::Short{value:value.to_string()}}else{Nbt::Int{value:value.to_string()}}
}
fn raw_int(tags:Option<&BTreeMap<String,Nbt>>,key:&str,default:i32)->Result<i32>{tags.and_then(|t|t.get(key)).map_or(Ok(default),integer)}
fn json_int(value:&Nbt,depth:u8)->Result<i32>{
    ensure!(depth<=48,"Matter version nesting exceeds budget");
    match value{
        Nbt::List{value,..}if value.len()==1=>json_int(&value[0],depth+1),
        Nbt::IntArray{value}if value.len()==1=>Ok(value[0].parse()?),
        Nbt::ByteArray{value}=>{let bytes=STANDARD.decode(value)?;ensure!(bytes.len()==1,"Matter version array is not singleton");Ok(i32::from(bytes[0] as i8))},
        _=>integer(value),
    }
}
pub(super) fn validate(p:&MatterManipulator,r:&Recipe,items:&BTreeMap<&str,&Item>)->Result<()> {
    ensure!(r.source.handler==HANDLER&&p.native_recipe=="com.recursive_pineapple.matter_manipulator.common.items.RecipeInstallUpgrade"
        &&p.version=="0.0.51-GTNH"&&p.tier<=3&&r.grid.is_none()&&r.duration.is_none()&&r.energy.is_none()
        &&r.magic.is_none()&&r.properties.is_empty()&&r.inputs.len()==2&&r.outputs.len()==1,"invalid Matter native process");
    crate::identity::registry_name(&p.manipulator)?;
    ensure!(!p.upgrades.is_empty()&&p.upgrades.len()<=16&&p.upgrades.iter().enumerate().all(|(i,u)|u.ordinal as usize==i&&!u.name.is_empty()&&u.name.len()<=256)
        &&p.upgrades.iter().map(|u|u.bit).collect::<BTreeSet<_>>().len()==p.upgrades.len(),"invalid Matter declared upgrade table");
    let registered=|u:&Upgrade|p.upgrades.get(u.ordinal as usize)==Some(u);
    ensure!(registered(&p.upgrade)&&!p.allowed.is_empty()&&p.allowed.len()<=16&&p.allowed.iter().all(registered)
        &&p.allowed.windows(2).all(|v|v[0].ordinal<v[1].ordinal)&&p.allowed.contains(&p.upgrade),"Matter upgrade is outside its native tier");
    ensure!(!p.meta_items.is_empty()&&p.meta_items.len()<=65536&&p.meta_items.windows(2).all(|v|v[0]<v[1])
        &&!p.meta_items.contains(&p.manipulator)&&p.mappings.len()<=65536&&p.mappings.windows(2).all(|v|v[0].metadata<v[1].metadata)
        &&p.mappings.iter().all(|m|m.upgrade.as_ref().is_none_or(registered))&&p.default_upgrade.as_ref().is_none_or(registered),"invalid Matter native metadata/class mapping");
    for registry in &p.meta_items{crate::identity::registry_name(registry)?;}
    for (slot,input) in r.inputs.iter().enumerate(){ensure!(input.kind==Kind::Item&&input.slot==slot as u32&&!input.choices.is_empty(),"Matter native input missing");
        for choice in &input.choices{ensure!(choice.amount=="1"&&matches!(choice.consume,Consumption::Consume|Consumption::Keep|Consumption::Damage{..}|Consumption::GtTool{..} | Consumption::NativeContainer { .. }),"Matter takes one per occupied cell");
            let item=items.get(choice.id.as_str()).context("Matter specimen item missing")?;
            if slot==0 {ensure!(matches!(choice.rule,Match::MatterManipulatorInput)&&item.registry==p.manipulator
                &&!bits(item.nbt.as_ref())?.get(p.upgrade.bit as usize).copied().unwrap_or(false),"Matter center is not an eligible native state");}
            else {ensure!(matches!(choice.rule,Match::MatterManipulatorUpgrade)&&p.meta_items.contains(&item.registry)&&lookup(p,item.meta)==Some(&p.upgrade),"Matter component lookup selects another native upgrade");}
        }
    }
    ensure!(p.states.len()==r.inputs[0].choices.len()&&p.states.len()<=65536,"Matter normalization count differs from center specimens");
    let mut bytes=0;
    for (state,choice) in p.states.iter().zip(&r.inputs[0].choices){
        ensure!(state.id==choice.id,"Matter normalization order differs");state.normalized.validate()?;bytes+=serde_json::to_vec(&state.normalized)?.len();
        ensure!(bytes<=32*1024*1024,"Matter normalization bytes exceed budget");
        let original=items[state.id.as_str()];let saved=compound(Some(&state.normalized))?.context("Matter normalized root missing")?;
        let flags=bits(original.nbt.as_ref())?;
        let canonical_flags=if flags.is_empty(){Nbt::List{element:"end".into(),value:vec![]}}else{Nbt::ByteArray{value:STANDARD.encode(flags.iter().copied().map(u8::from).collect::<Vec<_>>())}};
        ensure!(saved.get("installedUpgrades")==Some(&canonical_flags)&&matches!(saved.get("config"),Some(Nbt::Compound{..}))
            &&["jv","dv","config","charge","installedUpgrades","upgradeProvidedCapabilities"].iter().all(|key|saved.contains_key(*key))
            &&saved.keys().all(|k|["jv","dv","config","encKey","uplinkAddress","charge","installedUpgrades","upgradeProvidedCapabilities"].contains(&k.as_str())),"Matter normalized codec facts differ");
        let input=compound(original.nbt.as_ref())?;
        let version=input.and_then(|r|r.get("jv")).map_or(Ok(0),|v|json_int(v,0))?;let version=if version==0{1}else{version};
        let mut caps=raw_int(input,"upgradeProvidedCapabilities",0)?;
        for u in &p.upgrades{if flags.get(u.bit as usize).copied().unwrap_or(false){caps|=u.provides_caps;}}
        ensure!(saved.get("jv")==Some(&compact(version))&&saved.get("dv")==Some(&compact(raw_int(input,"dv",0)?))
            &&saved.get("upgradeProvidedCapabilities")==Some(&compact(caps)),"Matter version or PRE-install capability normalization differs");
    }
    let out=&r.outputs[0];let change=out.change.as_ref().context("Matter state edit missing")?;
    ensure!(out.kind==Kind::Item&&out.slot==0&&out.role==OutputRole::Result&&out.quantity.is_none()&&out.amount.as_deref()==Some("1")
        &&out.chance.numerator=="1"&&out.chance.denominator=="1"&&change.input==0&&matches!(change.action,Edit::MatterManipulator{})
        &&change.bindings.is_none()&&change.samples.len()==p.states.len()&&change.samples.first().is_some_and(|s|s.id==out.id),"invalid Matter dynamic result");
    for (state,sample) in p.states.iter().zip(&change.samples){let original=items[state.id.as_str()];let output=items.get(sample.id.as_str()).context("Matter result specimen missing")?;
        ensure!(sample.amount=="1"&&output.registry==original.registry&&output.meta==original.meta&&output.nbt==result(p,original)?,"Matter result differs from root replacement and bit installation");
    }Ok(())
}

#[cfg(test)]
mod tests{
    use super::*;
    #[test]fn gson_flags_use_nonzero_numeric_arrays_not_base64(){
        let root=Nbt::Compound{value:BTreeMap::from([("installedUpgrades".into(),Nbt::IntArray{value:vec!["0".into(),"-4".into(),"0".into()]})])};
        assert_eq!(bits(Some(&root)).unwrap(),vec![false,true]);
        let invalid=Nbt::Compound{value:BTreeMap::from([("installedUpgrades".into(),Nbt::String{value:"AAE=".into()})])};assert!(bits(Some(&invalid)).is_err());
        assert_eq!(java_int("＋1").is_ok(),false);assert_eq!(java_int("-٢").unwrap(),-2);
    }
}
