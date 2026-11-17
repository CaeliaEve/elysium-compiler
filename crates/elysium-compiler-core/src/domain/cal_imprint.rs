//! GT 2.9 CALImprintRecipe. Imprint metadata is a CircuitImprint enum ID.
use super::{Consumption,Edit,Item,Kind,Match,OutputRole,Recipe,Stack};
use crate::identity::{item_id,Nbt};
use anyhow::{ensure,Context,Result};
use schemars::JsonSchema;
use serde::{Deserialize,Serialize};
use std::collections::{BTreeMap,BTreeSet};

#[derive(Clone,Copy,Debug,PartialEq,Eq,Serialize,Deserialize,JsonSchema)]
#[serde(rename_all="camelCase")]
pub enum Role {Machine,Imprint}
#[derive(Clone,Debug,Serialize,Deserialize,JsonSchema)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct Imprint {
    pub native_recipe:String,pub machine:u32,pub imprint:u32,pub occupied_limit:u8,pub tag:String,
    #[schemars(length(min=1,max=1000))]
    pub imprint_ids:Vec<i32>,
}
pub(super) fn validate_ids(ids:&[i32])->Result<()> {
    ensure!(!ids.is_empty()&&ids.len()<=1000&&ids.iter().all(|id|(0..1000).contains(id))
        &&ids.windows(2).all(|p|p[0]<p[1]),"CAL imprint enum IDs must be bounded, sorted and unique");Ok(())
}
fn compound(nbt:&Nbt)->Result<&BTreeMap<String,Nbt>>{if let Nbt::Compound{value}=nbt{Ok(value)}else{anyhow::bail!("Native CAL compound read faults on a scalar tag")}}
pub(super) fn accepts(role:Role,item:&Item,ids:&[i32])->Result<bool>{
    match role{
        Role::Machine=>match &item.nbt {None=>Ok(true),Some(nbt)=>{
            let root=compound(nbt)?;
            // Native legacy-name lookup wins even when id_imprint is present.
            ensure!(!root.contains_key("Type"),"CAL legacy Type requires native item-name lookup context");
            Ok(!root.contains_key("id_imprint")||ids.binary_search(&super::change::number(root.get("id_imprint"))?).is_err())
        }},
        Role::Imprint=>Ok(ids.binary_search(&item.meta).is_ok()),
    }
}
pub(super) fn apply(machine:&Item,imprint:&Item)->Result<Stack>{
    let mut root=machine.nbt.as_ref().map(compound).transpose()?.cloned().unwrap_or_default();
    ensure!((0..1000).contains(&imprint.meta),"CAL imprint metadata is outside its enum domain");
    root.insert("id_imprint".into(),Nbt::Int{value:imprint.meta.to_string()});
    Ok(Stack{id:item_id(&machine.registry,machine.meta,Some(&Nbt::Compound{value:root}))?,amount:"1".into()})
}
pub(super) fn validate(p:&Imprint,r:&Recipe,items:&BTreeMap<&str,&Item>)->Result<()> {
    validate_ids(&p.imprint_ids)?;
    ensure!(r.source.handler=="gregtech.nei.GTNEIImprintHandler"&&p.native_recipe=="gregtech.common.recipes.CALImprintRecipe"
        &&p.occupied_limit==2&&p.tag=="id_imprint"&&p.machine!=p.imprint&&r.inputs.len()==2&&r.outputs.len()==1
        &&r.grid.is_none()&&r.magic.is_none()&&r.duration.is_none()&&r.energy.is_none(),"Invalid pinned CAL imprint process");
    for (slot,role) in [(p.machine,Role::Machine),(p.imprint,Role::Imprint)] {
        let input=r.inputs.iter().find(|i|i.slot==slot).context("CAL input slot missing")?;
        ensure!(input.kind==Kind::Item&&!input.choices.is_empty(),"CAL input must have native item examples");
        for c in &input.choices{let item=items.get(c.id.as_str()).context("CAL item fact missing")?;
            ensure!(c.amount=="1"&&matches!(c.consume,Consumption::Consume)&&c.returns.is_empty()
                &&matches!(&c.rule,Match::CalInput{role:r,imprint_ids} if *r==role&&imprint_ids==&p.imprint_ids)
                &&accepts(role,item,&p.imprint_ids)?,"CAL native input role, admission or debit changed");}
    }
    let output=&r.outputs[0];
    ensure!(output.kind==Kind::Item&&matches!(output.role,OutputRole::Result)&&output.amount.as_deref()==Some("1")
        &&output.quantity.is_none()&&output.chance.numerator=="1"&&output.chance.denominator=="1","CAL output must be one copied machine");
    let change=output.change.as_ref().context("CAL output lost dynamic machine NBT")?;
    ensure!(change.input==p.machine&&matches!(change.action,Edit::CalImprint{imprint} if imprint==p.imprint),"CAL output edit differs from native roles");
    let bindings=change.bindings.as_ref().context("CAL output lost correlated samples")?;
    let a=r.inputs.iter().position(|i|i.slot==p.machine).unwrap();let b=r.inputs.iter().position(|i|i.slot==p.imprint).unwrap();
    ensure!(bindings.len()==change.samples.len()&&bindings.len()==r.inputs[a].choices.len()*r.inputs[b].choices.len(),"CAL examples omit native input tuples");
    let mut seen=BTreeSet::new();
    for (tuple,sample) in bindings.iter().zip(&change.samples){
        ensure!(tuple.len()==2&&seen.insert(tuple.clone()),"CAL duplicate or malformed tuple");
        let machine=r.inputs[a].choices.get(tuple[a].context("CAL machine cannot be empty")? as usize).context("CAL machine choice outside examples")?;
        let imprint=r.inputs[b].choices.get(tuple[b].context("CAL imprint cannot be empty")? as usize).context("CAL imprint choice outside examples")?;
        let expected=apply(items[machine.id.as_str()],items[imprint.id.as_str()])?;
        ensure!(sample.id==expected.id&&sample.amount==expected.amount&&items.contains_key(sample.id.as_str()),"CAL example lost native tags or installed enum ID");
    }
    let first=change.samples.first().context("CAL output example missing")?;
    ensure!(output.id==first.id&&output.amount.as_ref()==Some(&first.amount),"CAL display differs from dynamic sample");Ok(())
}
