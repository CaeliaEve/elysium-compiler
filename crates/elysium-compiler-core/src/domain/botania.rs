//! Registry predicates and world settlement, independent of native NEI recipe caches.
use super::{Consumption,Item,Kind,Match,OutputRole,Recipe,Text};
use crate::identity::Nbt;
use anyhow::{ensure,Context,Result};
use schemars::JsonSchema;
use serde::{Deserialize,Serialize};
use std::collections::{BTreeMap,BTreeSet};

const POOL:&str="vazkii.botania.api.recipe.RecipeManaInfusion";
const MINI:&str="vazkii.botania.api.recipe.RecipeMiniFlower";
const RUNIC:&str="vazkii.botania.api.recipe.RecipeRuneAltar";
const HEAD:&str="vazkii.botania.common.crafting.recipe.HeadRecipe";
const DEFAULT_NOTE:&str="botania:headDefaultName";
const DEFAULT_SOURCE:&str="net.minecraft.util.StatCollector#translateToLocal(item.nameTag.name)";

#[derive(Clone,Debug,Serialize,Deserialize,JsonSchema)]
pub enum Version {#[serde(rename="1.12.28-GTNH")] Native}
#[derive(Clone,Debug,Serialize,Deserialize,JsonSchema)]
#[serde(rename_all="lowercase")]
pub enum Catalyst {None,Alchemy,Conjuration}
#[derive(Clone,Debug,Serialize,Deserialize,JsonSchema)]
pub enum PoolMatching {#[serde(rename="firstEligibleNativeRegistryEntry")] Native}
#[derive(Clone,Debug,Serialize,Deserialize,JsonSchema)]
pub enum PoolSettlement {#[serde(rename="serverDroppedItemUnitAndSignedManaUpdate")] Native}
#[derive(Clone,Debug,Serialize,Deserialize,JsonSchema)]
pub enum RunicMatching {#[serde(rename="physicalFirstRemainingPredicate")] Native}
#[derive(Clone,Debug,Serialize,Deserialize,JsonSchema)]
pub enum RunicSettlement {#[serde(rename="clearAltarRefundRunesAndConsumeDroppedLivingrock")] Native}
#[derive(Clone,Debug,Serialize,Deserialize,JsonSchema)]
pub enum RunicScope {#[serde(rename="stablePackedInputsWithoutCachedOverride")] Stable}

#[derive(Clone,Debug,Serialize,Deserialize,JsonSchema)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct Pool {
    pub version:Version,pub native_recipe:String,pub mana:i32,pub catalyst:Catalyst,
    pub matching:PoolMatching,pub settlement:PoolSettlement,
    #[serde(default,skip_serializing_if="Option::is_none")]
    pub selection:Option<PoolSelection>,
}
#[derive(Clone,Debug,Serialize,Deserialize,JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct PoolSelection {
    pub current:Vec<PoolPredicate>,pub earlier:Vec<PoolPredicate>,pub coercions:Vec<Coercion>,
}
#[derive(Clone,Debug,Serialize,Deserialize,JsonSchema)]
#[serde(tag="kind",deny_unknown_fields)]
pub enum PoolPredicate {
    #[serde(rename="item")] Item {registry:String,metadata:i32},
    #[serde(rename="subTileKey")] SubTileKey {key:String},
}
#[derive(Clone,Debug,Serialize,Deserialize,JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Coercion {pub input:String,pub value:String}
#[derive(Clone,Debug,Serialize,Deserialize,JsonSchema)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct Runic {
    pub version:Version,pub native_recipe:String,pub mana:i32,pub cooldown:u32,
    pub scope:RunicScope,pub matching:RunicMatching,pub settlement:RunicSettlement,
    pub predicates:Vec<Vec<Selector>>,pub earlier:Vec<Vec<Vec<Selector>>>,
    pub rune:String,pub livingrock:String,
    #[serde(default,skip_serializing_if="Vec::is_empty")]
    pub earlier_heads:Vec<u32>,
    #[serde(default,skip_serializing_if="Option::is_none")]
    pub head:Option<Head>,
    #[serde(default,skip_serializing_if="Option::is_none")]
    pub name_tag_default:Option<String>,
}
#[derive(Clone,Debug,Serialize,Deserialize,JsonSchema)]
pub enum HeadMatching {#[serde(rename="packedNameTagsEachRejectLocalizedDefault")] Native}
#[derive(Clone,Debug,Serialize,Deserialize,JsonSchema)]
pub enum HeadOutput {#[serde(rename="freshPlayerSkullFromLastNameTagOrInitialName")] Native}
#[derive(Clone,Debug,Serialize,Deserialize,JsonSchema)]
pub enum HeadVisibility {#[serde(rename="craftingNbtOutputQueryOnly")] Native}
#[derive(Clone,Debug,Serialize,Deserialize,JsonSchema)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct Head {
    pub initial_name:String,pub sample_name:String,pub name_tag:String,pub default_name_key:String,
    pub matching:HeadMatching,pub output:HeadOutput,pub visibility:HeadVisibility,
}
#[derive(Clone,Debug,Serialize,Deserialize,JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Selector {pub registry:String,pub metadata:i32}

fn base(r:&Recipe,handler:&str)->Result<()> {
    ensure!(r.source.owner=="Botania"&&r.source.handler==handler,"Botania process has a different native source");
    ensure!(r.duration.is_none()&&r.energy.is_none()&&r.grid.is_none()&&r.magic.is_none()&&r.properties.is_empty(),"Botania rule cannot contain GUI, fixed machine timing or untranslated rule properties");
    ensure!(r.outputs.len()==1,"Botania requires one conditional result");
    let out=&r.outputs[0];
    ensure!(out.kind==Kind::Item&&out.slot==0&&matches!(out.role,OutputRole::Result)&&out.quantity.is_none()&&out.change.is_none()&&out.chance.numerator=="1"&&out.chance.denominator=="1","Botania result is one native conditional specimen");
    crate::identity::integer(out.amount.as_deref().context("Missing Botania output amount")?,1,i32::MAX.into())?;
    Ok(())
}
fn pool_rule(rule:&Match,nested:bool)->bool {
    match rule {
        Match::Wildcard{nbt:true,..}=>true,
        Match::Except{base,exclude} if !nested=>!exclude.is_empty()&&exclude.len()<=4096&&pool_rule(base,true)&&exclude.iter().all(|f|pool_rule(&f.rule,true)),
        _=>false,
    }
}
fn root(item:&Item)->Result<Option<&BTreeMap<String,Nbt>>> {
    match &item.nbt {None=>Ok(None),Some(Nbt::Compound{value})=>Ok(Some(value)),_=>anyhow::bail!("Botania item root is not a compound")}
}
fn text_bound(text:&str)->bool {text.encode_utf16().count()<=16384}
fn item_predicate(predicate:&PoolPredicate,item:&Item)->bool {
    matches!(predicate,PoolPredicate::Item{registry,metadata} if *registry==item.registry&&(*metadata==32767||*metadata==item.meta))
}
fn covered(predicate:&PoolPredicate,earlier:&[PoolPredicate])->bool {
    earlier.iter().any(|prior|match(prior,predicate){
        (PoolPredicate::Item{registry:a,metadata:m},PoolPredicate::Item{registry:b,metadata:n})=>a==b&&(*m==32767||m==n),
        (PoolPredicate::SubTileKey{key:a},PoolPredicate::SubTileKey{key:b})=>a==b,
        _=>false,
    })
}
pub(super) fn validate_pool(p:&Pool,r:&Recipe,items:&BTreeMap<&str,&Item>)->Result<()> {
    base(r,"vazkii.botania.client.integration.nei.recipe.RecipeHandlerManaPool")?;
    ensure!(p.native_recipe==POOL||p.native_recipe==MINI,"Unadapted native pool definition");
    ensure!(r.inputs.len()==1,"Pool consumes one dropped item");
    let input=&r.inputs[0];
    ensure!(input.kind==Kind::Item&&input.slot==0&&!input.choices.is_empty(),"Invalid pool input role");
    for c in &input.choices {
        ensure!(c.amount=="1"&&matches!(c.consume,Consumption::Consume)&&c.returns.is_empty()&&pool_rule(&c.rule,false),"Pool consumes one item without containers and ignores matching NBT");
    }
    let Some(selection)=&p.selection else {ensure!(p.native_recipe==POOL,"MiniFlower lacks native subtile selection");return Ok(());};
    ensure!(!selection.current.is_empty()&&selection.current.len()+selection.earlier.len()<=65536,"Invalid pool native selector census");
    let mut keys=BTreeSet::new();
    for predicate in selection.current.iter().chain(&selection.earlier){match predicate {
        PoolPredicate::Item{registry:key,..}=>ensure!(registry(key),"Invalid pool selector registry"),
        PoolPredicate::SubTileKey{key}=>{ensure!(text_bound(key),"Pool subtile key exceeds text budget");keys.insert(key.as_str());},
    }}
    ensure!(!keys.is_empty(),"Pool special selection lacks a MiniFlower predicate");
    ensure!(if p.native_recipe==MINI {selection.current.len()==1&&matches!(selection.current[0],PoolPredicate::SubTileKey{..})}
        else {selection.current.iter().all(|predicate|matches!(predicate,PoolPredicate::Item{..}))},"Pool current selector disagrees with native recipe class");
    ensure!(!selection.coercions.is_empty()&&selection.coercions.len()<=65536,"Missing native pool key observations");
    let mut ids=BTreeSet::new();let mut observed=BTreeSet::new();let mut bytes=0usize;
    for coercion in &selection.coercions {
        bytes=bytes.checked_add(coercion.value.len()).context("Pool coercion byte overflow")?;
        ensure!(ids.insert(coercion.input.as_str())&&text_bound(&coercion.value)&&bytes<=1024*1024&&keys.contains(coercion.value.as_str()),"Invalid pool native key observation binding");
        let item=items.get(coercion.input.as_str()).context("Missing pool coercion item")?;
        match root(item)?.and_then(|tag|tag.get("type")) {
            None=>ensure!(coercion.value.is_empty(),"Missing flower type must read as empty"),
            Some(Nbt::String{value})=>ensure!(*value==coercion.value,"Native flower string key changed"),
            Some(_)=>{},
        }
        observed.insert(coercion.value.as_str());
    }
    ensure!(keys==observed,"A native subtile key lost all its observations");
    for c in &input.choices {
        let item=items.get(c.id.as_str()).context("Missing pool display item")?;
        let Match::Wildcard{meta,nbt:true}=&c.rule else {anyhow::bail!("Special pool rows require plain display witnesses");};
        if p.native_recipe==MINI {
            ensure!(*meta&&selection.coercions.iter().any(|observation|observation.input==c.id),"MiniFlower display witness lost its native key observation");
            let PoolPredicate::SubTileKey{key}=&selection.current[0] else {unreachable!()};
            ensure!(selection.coercions.iter().any(|observation|observation.input==c.id&&observation.value==*key),"MiniFlower witness has a different native key");
        }else{
            ensure!(selection.current.iter().any(|predicate|match predicate {
                PoolPredicate::Item{registry,metadata}=>*registry==item.registry&&if *meta{*metadata==32767}else{*metadata==item.meta},
                _=>false,
            }),"Pool display witness differs from the current native item selector");
        }
    }
    if p.native_recipe==POOL {
        ensure!(selection.current.iter().all(|predicate|covered(predicate,&selection.earlier)||input.choices.iter().any(|choice|items.get(choice.id.as_str()).is_some_and(|item|item_predicate(predicate,item)))),"Pool current native selector lost its display witness");
    }
    Ok(())
}
fn registry(s:&str)->bool {!s.is_empty()&&s.len()<=512}
fn groups(groups:&[Vec<Selector>],remaining:&mut usize)->Result<()> {
    ensure!(groups.len()<=16,"Runic predicates exceed physical altar slots");
    for group in groups {
        *remaining=remaining.checked_sub(group.len()).context("Runic selector budget exceeded")?;
        ensure!(group.iter().all(|s|registry(&s.registry)),"Invalid runic item registry");
    }
    Ok(())
}
pub(super) fn validate_runic(p:&Runic,r:&Recipe,items:&BTreeMap<&str,&Item>)->Result<()> {
    base(r,"vazkii.botania.client.integration.nei.recipe.RecipeHandlerRunicAltar")?;
    ensure!((p.native_recipe==RUNIC||p.native_recipe==HEAD)&&p.mana>0&&p.cooldown==60&&registry(&p.rune)&&registry(&p.livingrock)&&p.earlier.len()<=4096,"Invalid native runic completion or priority definition");
    ensure!(p.head.is_some()==(p.native_recipe==HEAD),"Head dynamic law disagrees with native runic class");
    ensure!(p.earlier_heads.len()<=p.earlier.len()&&p.earlier_heads.windows(2).all(|pair|pair[0]<pair[1])
        &&p.earlier_heads.iter().all(|index|(*index as usize)<p.earlier.len()),"Invalid earlier HeadRecipe selector indices");
    let names=p.head.is_some()||!p.earlier_heads.is_empty();
    ensure!(if names {p.name_tag_default.as_deref()==Some(DEFAULT_NOTE)}else{p.name_tag_default.is_none()},"Runic named-tag condition lost its native locale observation binding");
    if names {default_note(r)?;}
    let mut remaining=65536;groups(&p.predicates,&mut remaining)?;
    for prior in &p.earlier {groups(prior,&mut remaining)?;}
    ensure!(p.predicates.iter().all(|g|!g.is_empty())&&r.inputs.len()==p.predicates.len()+1,"Invalid runic input roles or empty predicate");
    for (index,input) in r.inputs.iter().enumerate() {
        let stone=index==p.predicates.len();
        ensure!(input.kind==Kind::Item&&input.slot==if stone{16}else{index as u32}&&!input.choices.is_empty()&&(!stone||input.choices.len()==1),"Runic altar and dropped livingrock must stay separate");
        for c in &input.choices {
            let item=items.get(c.id.as_str()).context("Missing runic item")?;
            let Match::Wildcard{meta,nbt:true}=&c.rule else {anyhow::bail!("Runic predicates ignore offered NBT");};
            ensure!(c.amount=="1"&&c.returns.is_empty(),"Runic template counts and container returns do not apply");
            if stone {
                ensure!(item.registry==p.livingrock&&*meta&&matches!(c.consume,Consumption::Consume),"Livingrock entity requires one native item of any metadata/NBT");
            }else{
                ensure!(if item.registry==p.rune {matches!(c.consume,Consumption::Keep)}else{matches!(c.consume,Consumption::Consume)},"Native rune refunds differ from input consumption");
                ensure!(p.predicates[index].iter().any(|s|s.registry==item.registry&&if *meta{s.metadata==32767}else{s.metadata==item.meta}),"Displayed runic candidate differs from native predicate");
            }
        }
        if !stone {
            ensure!(p.predicates[index].iter().all(|s|input.choices.iter().any(|c|items.get(c.id.as_str()).is_some_and(|item|s.registry==item.registry&&(s.metadata==32767||s.metadata==item.meta)))),"Native runic predicate lost all its display witnesses");
        }
    }
    if let Some(head)=&p.head {
        ensure!(!head.initial_name.is_empty()&&text_bound(&head.initial_name)&&text_bound(&head.sample_name)
            &&head.name_tag=="minecraft:name_tag"&&head.default_name_key=="item.nameTag.name","Invalid bound HeadRecipe name or native item role");
        let output=items.get(r.outputs[0].id.as_str()).context("Missing HeadRecipe result specimen")?;
        ensure!(output.registry=="minecraft:skull"&&output.meta==3&&r.outputs[0].amount.as_deref()==Some("1"),"HeadRecipe ignored fresh player-skull output");
        if head.sample_name.is_empty(){ensure!(output.nbt.is_none(),"Empty head name must leave fresh output untagged");}
        else {
            let tags=root(output)?.context("Named head lacks fresh output tags")?;
            ensure!(tags.len()==1&&matches!(tags.get("SkullOwner"),Some(Nbt::String{value}) if *value==head.sample_name),"HeadRecipe copied tags or lost the sample SkullOwner");
        }
        let mut named=false;
        for input in r.inputs.iter().take(p.predicates.len()) {for choice in &input.choices {
            let item=items.get(choice.id.as_str()).context("Missing head input witness")?;
            if item.registry!=head.name_tag {continue;}
            named=true;
            let display=root(item)?.and_then(|tag|tag.get("display"));
            ensure!(matches!(display,Some(Nbt::Compound{value}) if matches!(value.get("Name"),Some(Nbt::String{value}) if *value==head.sample_name)),"HeadRecipe name-tag specimen differs from its dynamic result name");
        }}
        ensure!(named||head.sample_name==head.initial_name,"HeadRecipe without a name-tag must retain its bound name");
    }
    Ok(())
}

fn default_note(recipe:&Recipe)->Result<&super::Note> {
    let mut notes=recipe.notes.iter().filter(|note|note.key==DEFAULT_NOTE);
    let note=notes.next().context("Missing source-bound native name-tag default observation")?;
    ensure!(notes.next().is_none()&&note.source==DEFAULT_SOURCE,"Invalid native name-tag default observation source");
    Ok(note)
}

/// Locale data lives in source-bound Text records, outside stable recipe identity. Never supply
/// an English fallback or infer a translated default name from the symbolic translation key.
pub(super) fn validate_observations(recipe:&Recipe,texts:&BTreeMap<&str,&Text>,locale:&str)->Result<()> {
    let Some(super::Process::BotaniaRunic(process))=&recipe.process else {return Ok(());};
    if process.head.is_none()&&process.earlier_heads.is_empty(){return Ok(());}
    let note=default_note(recipe)?;
    let text=texts.get(note.text.as_str()).context("Missing native name-tag default Text")?;
    ensure!(text.locale==locale&&!text.text.is_empty()&&text_bound(&text.text),"Native name-tag default observation is missing for the current source locale");
    if let Some(head)=&process.head {
        if process.predicates.iter().flatten().any(|selector|selector.registry==head.name_tag){
            ensure!(head.sample_name!=text.text,"HeadRecipe sample name is rejected by the observed native default-name comparison");
        }
    }
    Ok(())
}
