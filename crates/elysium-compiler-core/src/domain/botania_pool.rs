//! Mixed native pool rows are conditional constructor displays. Admission and
//! output depend on the full ordered registry and its bound cache snapshot.
use super::{botania,botania_bee::Stack,botania_program::Program,botania_session::Entry,
    Category,Consumption,Edit,Item,Kind,Match,OutputRole,Recipe};
use anyhow::{ensure,Context,Result};
use schemars::JsonSchema;
use serde::{Deserialize,Serialize};
use std::collections::BTreeMap;

#[derive(Clone,Debug,Serialize,Deserialize,JsonSchema)]
pub enum Scope {#[serde(rename="serialOwnedSnapshot")] Snapshot}
#[derive(Clone,Debug,Serialize,Deserialize,JsonSchema)]
pub enum Display {#[serde(rename="constructorPrototype")] Prototype}
#[derive(Clone,Debug,Serialize,Deserialize,JsonSchema)]
pub enum Matching {#[serde(rename="callbacksBeforeCatalystFirstEligible")] Native}
#[derive(Clone,Debug,Serialize,Deserialize,JsonSchema)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct Pool {
    pub version:botania::Version,pub native_recipe:String,pub program:String,pub index:u32,
    pub mana:i32,pub catalyst:botania::Catalyst,pub scope:Scope,pub display:Display,
    pub matching:Matching,pub settlement:botania::PoolSettlement,
}
pub(super) fn validate(p:&Pool,r:&Recipe,items:&BTreeMap<&str,&Item>)->Result<()>{
    ensure!(r.source.owner=="Botania"&&r.source.handler=="vazkii.botania.client.integration.nei.recipe.RecipeHandlerManaPool"
        &&p.program.strip_prefix("program_").is_some_and(crate::source::is_digest)&&p.index<4096,
        "Invalid native pool source/program");
    ensure!(r.grid.is_none()&&r.duration.is_none()&&r.energy.is_none()
        &&r.magic.is_none()&&r.properties.is_empty()&&r.inputs.len()==1&&r.outputs.len()==1,
        "Native pool cannot contain GUI, fixed timing or extra roles");
    let input=&r.inputs[0];let output=&r.outputs[0];
    ensure!(input.kind==Kind::Item&&input.slot==0&&!input.choices.is_empty()
        &&input.choices.iter().all(|c|c.amount=="1"&&matches!(c.consume,Consumption::Consume)
            &&c.returns.is_empty()&&matches!(c.rule,Match::NativePoolInput)),
        "Native pool consumes one dropped unit through ordered matching");
    ensure!(output.kind==Kind::Item&&output.slot==0&&matches!(output.role,OutputRole::Result)
        &&output.quantity.is_none()&&output.chance.numerator=="1"&&output.chance.denominator=="1"
        &&items.contains_key(output.id.as_str()),"Invalid native pool constructor prototype");
    crate::identity::integer(output.amount.as_deref().context("Missing pool prototype count")?,1,i32::MAX.into())?;
    let change=output.change.as_ref().context("Pool constructor prototype lacks conditional output law")?;
    ensure!(change.input==0&&matches!(change.action,Edit::NativePoolResult{})
        &&change.samples.is_empty()&&change.bindings.is_none(),"Pool constructor cannot claim evaluated output samples");
    Ok(())
}
fn identity(stack:&Stack,item:&Item)->bool {
    stack.registry==item.registry&&stack.meta==item.meta&&stack.nbt==item.nbt
}
pub(super) fn validate_program(p:&Pool,r:&Recipe,rules:&Program,category:&Category,items:&BTreeMap<&str,&Item>)->Result<()>{
    validate(p,r,items)?;
    ensure!(category.program.as_deref()==Some(p.program.as_str()),"Native pool category/program mismatch");
    let entry=rules.entries.get(p.index as usize).context("Native pool registry index is absent")?;
    let (native,alchemy,conjuration,mana,prototype)=match entry{
        Entry::Bee{alchemy,conjuration,mana,prototype,..}=>("magicbees.main.utils.compat.botania.SpeciesRecipeManaInfusion",alchemy,conjuration,mana,prototype),
        Entry::Item{alchemy,conjuration,mana,output,..}=>("vazkii.botania.api.recipe.RecipeManaInfusion",alchemy,conjuration,mana,output),
        Entry::Mini{alchemy,conjuration,mana,output,..}=>("vazkii.botania.api.recipe.RecipeMiniFlower",alchemy,conjuration,mana,output),
    };
    ensure!(p.native_recipe==native&&p.mana==*mana&&(!*alchemy||matches!(p.catalyst,botania::Catalyst::Alchemy))
        &&(!*conjuration||matches!(p.catalyst,botania::Catalyst::Conjuration)),"Native pool row differs from its registry definition");
    let output=&r.outputs[0];let item=items[output.id.as_str()];
    ensure!(identity(prototype,item)&&output.amount.as_deref()==Some(prototype.amount.as_str()),"Native pool prototype differs from bound output");
    let mut witnesses=Vec::new();
    for choice in &r.inputs[0].choices{
        let item=items.get(choice.id.as_str()).context("Missing pool display item")?;
        witnesses.push(*item);
        let bound=match entry{
            Entry::Bee{display_input,..}=>identity(display_input,item),
            Entry::Item{predicates,..}=>predicates.iter().any(|v|v.registry==item.registry&&(v.metadata==32767||v.metadata==item.meta))&&item.nbt.is_none(),
            Entry::Mini{key,coercions,..}=>match &item.nbt{
                None=>key.is_empty(),
                Some(crate::identity::Nbt::Compound{value})=>match value.get("type"){
                    None=>key.is_empty(),Some(crate::identity::Nbt::String{value})=>value==key,
                    _=>coercions.iter().any(|v|identity(&v.input,item)&&v.value==*key),
                },_=>false,
            },
        };
        ensure!(bound,"Native pool display differs from bound input role");
    }
    if let Entry::Item{predicates,..}=entry {
        ensure!(!predicates.is_empty()&&predicates.iter().all(|v|witnesses.iter().any(|i|v.registry==i.registry&&(v.metadata==32767||v.metadata==i.meta))),
            "Native pool predicate lost its display witness");
    }else{ensure!(witnesses.len()==1,"Native dynamic pool definition has one constructor input");}
    Ok(())
}
