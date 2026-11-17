//! Pinned Forestry ghost-grid casting and reservoir transitions.
use super::{Category, Consumption, Fluid, Item, Kind, Match, OutputRole, Recipe};
use crate::identity::Nbt;
use anyhow::{ensure, Context, Result};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all="camelCase", deny_unknown_fields)]
pub struct NativeStack {
    pub registry: String, pub meta: i32, pub amount: i32, pub nbt: Option<Nbt>, pub ores: Vec<String>,
    pub subtypes: bool, pub max_damage: i32, pub container_owner: String, pub container: ContainerPolicy,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all="camelCase", deny_unknown_fields)]
pub struct ContainerResult {
    pub registry: String, pub meta: i32, pub amount: i32, pub nbt: Option<Nbt>, pub ores: Vec<String>,
    pub subtypes: bool, pub max_damage: i32,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(tag="kind", rename_all="lowercase", deny_unknown_fields)]
pub enum ContainerPolicy { None, Fixed { result: ContainerResult, leave: bool } }
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct NativeFluid { pub registry: String, pub amount: i32, pub nbt: Option<Nbt>, pub viscosity: i32 }
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(untagged)]
pub enum Ingredient { Stack(NativeStack), Choices(Vec<NativeStack>) }
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all="camelCase", deny_unknown_fields)]
pub struct WaxPlan { pub registry: String, pub max_damage: i32 }
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(tag="kind", rename_all="lowercase", rename_all_fields="camelCase", deny_unknown_fields)]
pub enum PlanUse {
    Keep { #[serde(rename="waxPlans")] wax_plans: Vec<WaxPlan> },
    Wax { #[serde(rename="maxDamage")] max_damage: i32, #[serde(rename="waxPlans")] wax_plans: Vec<WaxPlan> },
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all="camelCase", deny_unknown_fields)]
pub struct FabricatorRecipe {
    pub width: u32, pub height: u32, pub plan: Option<NativeStack>, pub ingredients: Vec<Option<Ingredient>>,
    pub fluid: Option<NativeFluid>, pub result: NativeStack, pub plan_use: PlanUse, pub stock_items: Vec<NativeStack>,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all="camelCase", deny_unknown_fields)]
pub struct FabricatorSmeltingRecipe { pub resource: NativeStack, pub product: NativeFluid, pub melting_point: i32 }
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct FabricatorProgram { pub crafting: Vec<FabricatorRecipe>, pub smelting: Vec<FabricatorSmeltingRecipe> }
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all="lowercase")]
pub enum FabricatorOperation { Craft, Smelt }
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct FabricatorProcess { pub program: String, pub operation: FabricatorOperation, pub selector: u32 }

pub(super) fn compound(value: &Option<Nbt>) -> Result<()> {
    if let Some(nbt)=value { ensure!(matches!(nbt,Nbt::Compound{..}),"Native Forestry NBT root must be a compound");nbt.validate()?; }
    Ok(())
}
fn registry(value: &str)->Result<()> {ensure!(!value.is_empty()&&value.len()<=65535,"Invalid native Forestry registry name");Ok(())}
impl NativeStack {
    pub fn validate(&self)->Result<()> {
        registry(&self.registry)?;compound(&self.nbt)?;
        ensure!((0..=32767).contains(&self.meta)&&self.amount>0&&self.max_damage>=0,"Invalid native Forestry stack value");
        ensure!(self.ores.len()<=4096&&self.ores.iter().all(|n|!n.is_empty()&&n.len()<=65535),"Invalid native Forestry ores");
        ensure!(self.ores.iter().collect::<BTreeSet<_>>().len()==self.ores.len(),"Duplicate native Forestry ore membership");
        ensure!(!self.container_owner.is_empty()&&self.container_owner.len()<=65535,"Invalid native Forestry callback owner");
        if let ContainerPolicy::Fixed{result,..}=&self.container {
            registry(&result.registry)?;compound(&result.nbt)?;
            ensure!((0..=32767).contains(&result.meta)&&result.amount>0&&result.max_damage>=0&&result.ores.len()<=4096,"Invalid native container return");
            ensure!(["net.minecraft.item.Item","gregtech.api.items.GTGenericItem","gregtech.api.items.MetaGeneratedItemX01","gregtech.api.items.MetaGeneratedItemX32","gregtech.common.items.MetaGeneratedItem01","ic2.core.item.resources.ItemCell"].contains(&self.container_owner.as_str()),"Unadapted native container result callback");
        }
        Ok(())
    }
    pub fn matches(&self,offered:&Self)->bool {
        self.registry==offered.registry&&(self.meta==32767||self.meta==offered.meta)
            && (empty_nbt(&self.nbt)||same_nbt(&self.nbt,&offered.nbt))
    }
    pub fn matches_sample(&self,item:&Item)->bool {self.registry==item.registry&&(self.meta==32767||self.meta==item.meta)&&self.nbt==item.nbt}
}
impl NativeFluid {
    pub fn validate(&self)->Result<()>{registry(&self.registry)?;compound(&self.nbt)?;ensure!(self.amount>0,"Invalid native Forestry fluid volume");Ok(())}
    pub fn same_fluid(&self,other:&Self)->bool {self.registry==other.registry&&same_nbt(&self.nbt,&other.nbt)}
}
fn empty_nbt(value:&Option<Nbt>)->bool {value.as_ref().is_none_or(|n|matches!(n,Nbt::Compound{value} if value.is_empty()))}
fn same_nbt(a:&Option<Nbt>,b:&Option<Nbt>)->bool {match(a,b){(None,None)=>true,(Some(a),Some(b))=>super::matching::native_tag(a,b,true),_=>false}}
impl Ingredient {
    pub fn members(&self)->&[NativeStack] {match self{Self::Stack(stack)=>std::slice::from_ref(stack),Self::Choices(values)=>values}}
    fn matches(&self,offered:Option<&NativeStack>)->bool {offered.is_some_and(|o|self.members().iter().any(|s|s.matches(o)))}
}
impl PlanUse {
    fn wax_plans(&self)->&[WaxPlan]{match self{Self::Keep{wax_plans}|Self::Wax{wax_plans,..}=>wax_plans}}
    /// TileFabricator applies the callback to the actual offered plan even if no plan was required.
    pub fn after_use(&self,offered:&NativeStack,result_count:i32)->Result<Option<NativeStack>>{
        ensure!(result_count>0,"Invalid native casting result count");
        let Some(wax)=self.wax_plans().iter().find(|w|w.registry==offered.registry) else{return Ok(Some(offered.clone()))};
        let mut next=offered.clone();next.meta=next.meta.wrapping_add(result_count);
        Ok((next.meta<wax.max_damage).then_some(next))
    }
    fn validate(&self)->Result<()>{
        ensure!(self.wax_plans().len()<=4096,"Native wax plan registry exceeds budget");let mut names=BTreeSet::new();
        for wax in self.wax_plans(){registry(&wax.registry)?;ensure!(wax.max_damage>0&&names.insert(&wax.registry),"Invalid or duplicate native wax plan");}
        if let Self::Wax{max_damage,..}=self{ensure!(*max_damage>0,"Invalid native wax plan durability");}Ok(())
    }
}
impl FabricatorProgram {
    pub fn validate(&self)->Result<()>{
        ensure!(self.crafting.len()<=262144&&self.smelting.len()<=262144,"Fabricator registry exceeds budget");
        for recipe in &self.crafting{
            ensure!((1..=3).contains(&recipe.width)&&(1..=3).contains(&recipe.height)&&recipe.ingredients.len()==(recipe.width*recipe.height)as usize,"Invalid Fabricator matrix");
            recipe.result.validate()?;recipe.plan_use.validate()?;
            if let Some(plan)=&recipe.plan{plan.validate()?;}
            if let Some(fluid)=&recipe.fluid{fluid.validate()?;}
            for ingredient in recipe.ingredients.iter().flatten(){ensure!(ingredient.members().len()<=65536,"Fabricator alternatives exceed budget");for stack in ingredient.members(){stack.validate()?;}}
            ensure!(recipe.stock_items.len()<=65536,"Fabricator stock policy exceeds budget");for stack in &recipe.stock_items{stack.validate()?;}
            if let PlanUse::Wax{max_damage,wax_plans}=&recipe.plan_use{
                let plan=recipe.plan.as_ref().context("Wax policy has no required plan")?;
                ensure!(wax_plans.iter().any(|w|w.registry==plan.registry&&w.max_damage==*max_damage),"Required wax plan differs from offered-plan policy");
            }
        }
        for recipe in &self.smelting{recipe.resource.validate()?;recipe.product.validate()?;}
        Ok(())
    }
    /// First shifted or mirrored native matrix match. Recipe display counts do not constrain selection.
    pub fn select(&self,plan:Option<&NativeStack>,grid:&[Option<NativeStack>])->Result<Option<usize>>{
        ensure!(grid.len()==9,"Fabricator ghost matrix must have nine cells");
        for(index,recipe)in self.crafting.iter().enumerate(){
            if recipe.plan.as_ref().is_some_and(|required|!plan.is_some_and(|p|required.matches(p))){continue;}
            for x in 0..=3-recipe.width{for y in 0..=3-recipe.height{for mirror in [true,false]{
                let matched=(0..3).all(|gx|(0..3).all(|gy|{
                    let cell=if gx>=x&&gx<x+recipe.width&&gy>=y&&gy<y+recipe.height{
                        let local=if mirror{recipe.width-1-(gx-x)}else{gx-x};recipe.ingredients[(local+(gy-y)*recipe.width)as usize].as_ref()
                    }else{None};
                    match cell{Some(i)=>i.matches(grid[(gx+gy*3)as usize].as_ref()),None=>grid[(gx+gy*3)as usize].is_none()}
                }));if matched{return Ok(Some(index));}
            }}}
        }Ok(None)
    }
}
impl FabricatorProcess {
    pub fn validate(&self,program:&FabricatorProgram)->Result<()>{
        ensure!(self.program.strip_prefix("program_").is_some_and(crate::source::is_digest),"Invalid Fabricator program identity");
        match self.operation{FabricatorOperation::Craft=>ensure!((self.selector as usize)<program.crafting.len(),"Fabricator casting selector out of range"),FabricatorOperation::Smelt=>ensure!((self.selector as usize)<program.smelting.len(),"Fabricator smelting selector out of range")};Ok(())
    }
}
pub fn validate_recipe(process:&FabricatorProcess,recipe:&Recipe,program:&FabricatorProgram,category:&Category,items:&BTreeMap<&str,&Item>,fluids:&BTreeMap<&str,&Fluid>)->Result<()>{
    process.validate(program)?;
    ensure!(recipe.source.handler=="forestry.factory.recipes.nei.NEIHandlerFabricator"&&category.program.as_ref()==Some(&process.program),"Fabricator handler or category program differs");
    ensure!(recipe.duration.is_none()&&recipe.energy.is_none(),"Fabricator heat work has no fixed recipe time or EU/t");
    match process.operation{
        FabricatorOperation::Craft=>{
            let native=&program.crafting[process.selector as usize];let mut slot=0;let mut cells=Vec::new();
            if let Some(plan)=&native.plan{
                item_input(recipe,slot,std::slice::from_ref(plan),items,if matches!(native.plan_use,PlanUse::Wax{..}){InputUse::Damage(native.result.amount as u32)}else{InputUse::Keep})?;slot+=1;
            }
            for ingredient in &native.ingredients{
                if let Some(ingredient)=ingredient{ensure!(!ingredient.members().is_empty(),"Selected Fabricator ingredient is impossible");item_input(recipe,slot,ingredient.members(),items,InputUse::Allocated)?;cells.push(Some(slot));slot+=1;}else{cells.push(None);}
            }
            ensure!(cells.iter().any(Option::is_some),"An empty ghost matrix cannot supply a native stock set");
            let grid=recipe.grid.as_ref().context("Fabricator ghost grid omitted")?;
            ensure!(grid.width==native.width&&grid.height==native.height&&grid.mirror&&grid.cells==cells,"Fabricator grid differs from native matrix");
            ensure!(recipe.inputs.len()==slot as usize+usize::from(native.fluid.is_some()),"Fabricator inputs omitted or added");
            if let Some(fluid)=&native.fluid{fluid_input(recipe,fluid,fluids,false)?;}
            let output=recipe.outputs.first().context("Fabricator result omitted")?;let item=items.get(output.id.as_str()).context("Fabricator result fact missing")?;
            ensure!(recipe.outputs.len()==1&&output.kind==Kind::Item&&output.slot==0&&native.result.matches_sample(item)&&output.amount.as_deref()==Some(native.result.amount.to_string().as_str())&&output.quantity.is_none(),"Fabricator result differs from native fixed result");
            guaranteed(output)?;
        }
        FabricatorOperation::Smelt=>{
            let native=&program.smelting[process.selector as usize];
            ensure!(native.melting_point<=5000&&native.product.amount<=2000,"Selected native smelting cannot fit heat or reservoir capacity");
            item_input(recipe,0,std::slice::from_ref(&native.resource),items,InputUse::Consume)?;
            ensure!(recipe.inputs.len()==1&&recipe.outputs.len()==1&&recipe.grid.is_none(),"Fabricator smelting added casting inputs or grid");
            let output=&recipe.outputs[0];let fluid=fluids.get(output.id.as_str()).context("Smelting product fact missing")?;
            ensure!(output.kind==Kind::Fluid&&output.slot==0&&fluid.registry==native.product.registry&&fluid.nbt==native.product.nbt&&output.amount.as_deref()==Some(native.product.amount.to_string().as_str())&&output.quantity.is_none(),"Fabricator smelting product differs");guaranteed(output)?;
        }
    }Ok(())
}
#[derive(Clone,Copy)]enum InputUse{Keep,Consume,Allocated,Damage(u32)}
fn item_input(recipe:&Recipe,slot:u32,templates:&[NativeStack],items:&BTreeMap<&str,&Item>,use_kind:InputUse)->Result<()>{
    let input=recipe.inputs.iter().find(|i|i.kind==Kind::Item&&i.slot==slot).context("Native Forestry item input omitted")?;ensure!(!input.choices.is_empty(),"Native Forestry input has no examples");
    for choice in &input.choices{
        let item=items.get(choice.id.as_str()).context("Native Forestry input fact missing")?;
        ensure!(choice.amount=="1"&&choice.returns.is_empty(),"Native Forestry nominal item demand is one");
        ensure!(templates.iter().any(|s|s.matches_sample(item)&&matches!(choice.rule,Match::Wildcard{meta,nbt} if meta==(s.meta==32767)&&nbt==empty_nbt(&s.nbt))),"Native Forestry predicate or input example differs");
        ensure!(match use_kind{InputUse::Keep=>matches!(choice.consume,Consumption::Keep),InputUse::Consume=>matches!(choice.consume,Consumption::Consume),InputUse::Allocated=>matches!(choice.consume,Consumption::Allocated),InputUse::Damage(p)=>matches!(choice.consume,Consumption::Damage{points} if points==p)},"Native Forestry item consumption differs");
    }Ok(())
}
pub(super) fn fluid_input(recipe:&Recipe,native:&NativeFluid,fluids:&BTreeMap<&str,&Fluid>,dynamic:bool)->Result<()>{
    let input=recipe.inputs.iter().find(|i|i.kind==Kind::Fluid&&i.slot==0).context("Native Forestry fluid input omitted")?;
    ensure!(input.choices.len()==1,"Native Forestry fluid input has alternatives");let choice=&input.choices[0];let fluid=fluids.get(choice.id.as_str()).context("Native Forestry fluid fact missing")?;
    ensure!(fluid.registry==native.registry&&fluid.nbt==native.nbt&&choice.amount==native.amount.to_string()&&choice.returns.is_empty(),"Native Forestry fluid predicate/amount differs");
    ensure!(if dynamic{matches!(choice.rule,Match::Wildcard{meta:false,nbt:true})&&matches!(choice.consume,Consumption::Allocated)}else{matches!(choice.rule,Match::Exact)&&matches!(choice.consume,Consumption::Consume)},"Native Forestry fluid allocation differs");Ok(())
}
pub(super) fn guaranteed(output:&super::Output)->Result<()>{ensure!(matches!(output.role,OutputRole::Result)&&output.change.is_none()&&output.chance.numerator=="1"&&output.chance.denominator=="1","Native Forestry result chance or role differs");Ok(())}
#[cfg(test)]
mod tests {
    #[test]
    fn native_wax_plan_schema_and_serialization_use_the_same_fields() {
        let schema=serde_json::to_value(schemars::schema_for!(super::PlanUse)).unwrap();
        for value in [super::PlanUse::Keep{wax_plans:vec![]},super::PlanUse::Wax{max_damage:8,wax_plans:vec![]}] {
            let value=serde_json::to_value(value).unwrap();
            let variant=schema["oneOf"].as_array().unwrap().iter().find(|v|v["properties"]["kind"]["enum"][0]==value["kind"]).unwrap();
            let fields=variant["properties"].as_object().unwrap();
            for field in value.as_object().unwrap().keys(){assert!(fields.contains_key(field),"Native wax field {field} is absent from schema");}
        }
    }

    #[test]
    fn native_fabricator_process_is_typed() {
        let value = serde_json::json!({"kind":"forestryFabricator","program":format!("program_{}","0".repeat(64)),"operation":"craft","selector":0});
        assert!(serde_json::from_value::<crate::domain::Process>(value).is_ok(), "Native Fabricator process contract is missing");
    }
}
