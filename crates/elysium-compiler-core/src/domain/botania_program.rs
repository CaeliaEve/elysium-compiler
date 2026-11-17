//! Versioned shared pool transport. Preserves registry order and native cache
//! identity without inlining the allele registry into each recipe.
use super::botania_bee::{Allele,Pair,Registration,Source,Stack};
use super::botania_cache_machine::{Entry as CacheEntry,Segment,State};
use super::botania_session::{Entry,Session};
use crate::identity::{integer,Nbt};
use anyhow::{ensure,Result};
use schemars::JsonSchema;
use serde::{Deserialize,Serialize};
use std::collections::{BTreeMap,BTreeSet};

#[derive(Clone,Debug,Serialize,Deserialize,JsonSchema)]
#[schemars(rename="BeePoolRuntime")]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct Runtime {
    pub runtime:String,pub artifact_sha256:String,pub clock:String,
    pub before_nanos:String,pub after_nanos:String,pub scope:String,
}
#[derive(Clone,Debug,Serialize,Deserialize,JsonSchema)]
#[schemars(rename="BeePoolSettings")]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct Settings {
    pub revision:u32,pub forestry:String,pub magicbees:String,pub root:String,
    pub clear_invalid_chromosomes:bool,pub shift:u32,pub mask:u32,pub runtime:Runtime,
}
#[derive(Clone,Debug,Serialize,Deserialize,JsonSchema)]
#[schemars(rename="BeePoolDefinition")]
#[serde(deny_unknown_fields)]
pub struct Definition {pub key:String,pub allele:Allele}
#[derive(Clone,Debug,Serialize,Deserialize,JsonSchema)]
#[schemars(rename="BeePoolAlias")]
#[serde(deny_unknown_fields)]
pub struct Alias {pub key:String,pub uid:String}
#[derive(Clone,Debug,Serialize,Deserialize,JsonSchema)]
#[schemars(rename="BeePoolTemplate")]
#[serde(deny_unknown_fields)]
pub struct Template {pub key:String,pub alleles:Vec<Option<String>>}
#[derive(Clone,Debug,Serialize,Deserialize,JsonSchema)]
#[schemars(rename="BeePoolForm")]
#[serde(deny_unknown_fields)]
pub struct Form {pub key:String,pub registry:String}
#[derive(Clone,Debug,Default,Serialize,Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct Program {
    pub settings:Vec<Settings>,pub chromosomes:Vec<String>,pub definitions:Vec<Definition>,
    pub alleles:Vec<Registration>,pub aliases:Vec<Alias>,pub templates:Vec<Template>,
    pub defaults:Vec<Option<String>>,pub forest:Vec<Option<Pair>>,pub forms:Vec<Form>,
    pub entries:Vec<Entry>,pub keys:Vec<Nbt>,pub cache_entries:Vec<CacheEntry>,pub segments:Vec<Segment>,
}
fn text(value:&str)->bool{!value.is_empty()&&value.encode_utf16().count()<=16384}
fn ordered<T>(rows:&[T],key:impl Fn(&T)->&str)->bool {
    rows.iter().all(|row|text(key(row)))&&rows.windows(2).all(|pair|
        key(&pair[0]).encode_utf16().cmp(key(&pair[1]).encode_utf16()).is_lt())
}
fn stack(value:&Stack)->Result<()> {
    ensure!(text(&value.registry)&&value.registry.len()<=512,"Invalid native pool stack registry");
    integer(&value.amount,1,i64::from(i32::MAX))?;
    if let Some(tag)=&value.nbt{ensure!(matches!(tag,Nbt::Compound{..}),"Native pool root tags are not a compound");tag.validate()?;}
    Ok(())
}
impl Program {
    pub fn source(&self)->Result<Source> {
        let settings=self.settings.first().ok_or_else(||anyhow::anyhow!("Missing native pool settings"))?;
        Ok(Source{forestry:settings.forestry.clone(),magicbees:settings.magicbees.clone(),root:settings.root.clone(),
            chromosomes:self.chromosomes.clone(),definitions:self.definitions.iter().map(|v|(v.key.clone(),v.allele.clone())).collect(),
            alleles:self.alleles.clone(),aliases:self.aliases.iter().map(|v|(v.key.clone(),v.uid.clone())).collect(),
            templates:self.templates.iter().map(|v|(v.key.clone(),v.alleles.clone())).collect(),
            defaults:self.defaults.clone(),forest:self.forest.clone(),
            forms:self.forms.iter().map(|v|(v.key.clone(),v.registry.clone())).collect(),
            clear_invalid_chromosomes:settings.clear_invalid_chromosomes})
    }
    pub fn cache(&self)->Result<State> {
        let settings=self.settings.first().ok_or_else(||anyhow::anyhow!("Missing native pool settings"))?;
        Ok(State{shift:settings.shift,mask:settings.mask,keys:self.keys.clone(),entries:self.cache_entries.clone(),segments:self.segments.clone()})
    }
    pub fn session(&self)->Result<Session> {
        self.validate()?;Session::new(self.source()?,self.entries.clone(),self.cache()?)
    }
    pub fn validate(&self)->Result<()> {
        ensure!(self.settings.len()==1&&self.entries.len()<=4096&&self.definitions.len()<=262144
            &&self.alleles.len()<=524288&&self.aliases.len()<=262144&&self.templates.len()<=262144
            &&serde_json::to_vec(self)?.len()<=16*1024*1024,"Native pool program exceeds its budget");
        let s=&self.settings[0];let runtime=&s.runtime;
        let before=runtime.before_nanos.parse::<i64>()?;let after=runtime.after_nanos.parse::<i64>()?;
        ensure!(s.revision==1&&s.forestry=="4.10.17"&&s.magicbees=="2.9.4-GTNH"&&s.root=="rootBees"
            &&runtime.runtime=="guava17StrongAccess"&&runtime.clock=="javaSystemNanoTime"
            &&runtime.artifact_sha256=="8c36a80ea613d0b6b8040a17cf837c5bbe3677bc1b06a058a6c174fdb787ebbc"
            &&runtime.scope=="serialOwnedTransitionsFromBoundSnapshotWithExplicitTickerReads"
            &&before.to_string()==runtime.before_nanos&&after.to_string()==runtime.after_nanos
            &&after.wrapping_sub(before)>=0,"Invalid native pool runtime binding");
        let chromosome_types=[
            "forestry.api.apiculture.IAlleleBeeSpecies","forestry.api.genetics.IAlleleFloat",
            "forestry.api.genetics.IAlleleInteger","forestry.api.genetics.IAlleleInteger",
            "forestry.api.genetics.IAlleleTolerance","forestry.api.genetics.IAlleleBoolean",
            "forestry.api.genetics.IAllele","forestry.api.genetics.IAlleleTolerance",
            "forestry.api.genetics.IAlleleBoolean","forestry.api.genetics.IAlleleBoolean",
            "forestry.api.genetics.IAlleleFlowers","forestry.api.genetics.IAlleleInteger",
            "forestry.api.genetics.IAlleleArea","forestry.api.apiculture.IAlleleBeeEffect"];
        ensure!(self.chromosomes==chromosome_types&&self.defaults.len()==14&&self.forest.len()==14
            &&ordered(&self.definitions,|v|&v.key)&&ordered(&self.aliases,|v|&v.key)
            &&ordered(&self.templates,|v|&v.key)&&ordered(&self.forms,|v|&v.key),"Invalid native pool source tables");
        let definitions:BTreeMap<_,_>=self.definitions.iter().map(|v|(v.key.as_str(),&v.allele)).collect();
        for row in &self.definitions {
            ensure!(row.key==row.allele.uid&&row.allele.slots.len()<=14
                &&row.allele.slots.iter().all(|&slot|slot<14)
                &&row.allele.slots.windows(2).all(|p|p[0]<p[1]),"Invalid native pool allele descriptor");
        }
        let mut registrations=BTreeSet::new();
        for row in &self.alleles {
            let allele=definitions.get(row.uid.as_str()).ok_or_else(||anyhow::anyhow!("Native pool registration lacks its object descriptor"))?;
            ensure!(text(&row.key)&&registrations.insert((row.alias_only,&row.key))
                &&row.dominant==allele.dominant&&row.slots==allele.slots
                &&(!row.alias_only||self.aliases.iter().any(|v|v.key==row.key&&v.uid==row.uid)),
                "Invalid native pool registration");
        }
        for row in &self.aliases {ensure!(definitions.contains_key(row.uid.as_str()),"Unknown native pool alias target");}
        for row in &self.templates {ensure!(row.alleles.len()==14,"Invalid native template length");}
        for uid in self.defaults.iter().chain(self.templates.iter().flat_map(|v|&v.alleles))
            .chain(self.forest.iter().flatten().flat_map(|v|[&v.primary,&v.secondary])).flatten(){
            ensure!(definitions.contains_key(uid.as_str()),"Unknown native pool template allele");
        }
        let forms=self.forms.iter().map(|v|v.key.as_str()).collect::<Vec<_>>();
        ensure!(forms==["beeDroneGE","beeLarvaeGE","beePrincessGE","beeQueenGE"]
            &&self.forms.iter().all(|v|text(&v.registry)&&v.registry.len()<=512)
            &&self.forms.iter().map(|v|&v.registry).collect::<BTreeSet<_>>().len()==4,"Invalid native bee forms");
        let mut predicates=0usize;
        for entry in &self.entries {
            match entry {
                Entry::Bee{form,input_species,output_species,display_input,prototype,cached_output,..}=>{
                    ensure!(forms.contains(&form.as_str())&&[input_species,output_species].iter()
                        .all(|uid|definitions.get(uid.as_str()).is_some_and(|a|a.slots.contains(&0))),"Unknown native pool recipe species or form");
                    let registry=&self.forms.iter().find(|v|v.key==*form).unwrap().registry;
                    for value in [display_input,prototype,cached_output] {
                        stack(value)?;ensure!(value.registry==*registry,"Native bee pool stack differs from its bound form");
                    }
                }
                Entry::Item{predicates:rows,output,..}=>{
                    predicates+=rows.len();ensure!(predicates<=65536&&rows.iter()
                        .all(|v|text(&v.registry)&&v.registry.len()<=512),"Invalid native pool predicates");stack(output)?;
                }
                Entry::Mini{key,output,coercions,..}=>{
                    ensure!(key.encode_utf16().count()<=16384&&coercions.len()<=4096,"Native MiniFlower key exceeds budget");stack(output)?;
                    let mut seen=BTreeMap::new();
                    for row in coercions {
                        stack(&row.input)?;
                        ensure!(row.value.encode_utf16().count()<=16384,"Native subtile observation exceeds text budget");
                        let identity=crate::identity::item_id(&row.input.registry,row.input.meta,row.input.nbt.as_ref())?;
                        if let Some(previous)=seen.insert(identity,&row.value){ensure!(*previous==row.value,"Conflicting native subtile observations");}
                        match &row.input.nbt{
                            Some(Nbt::Compound{value})=>match value.get("type"){
                                None=>ensure!(row.value.is_empty(),"Absent native subtile is not empty"),
                                Some(Nbt::String{value})=>ensure!(*value==row.value,"Native string subtile changed"),
                                _=>{},
                            },
                            None=>ensure!(row.value.is_empty(),"Absent native subtile is not empty"),
                            _=>unreachable!(),
                        }
                    }
                }
            }
        }
        let cache=self.cache()?;cache.validate()?;
        for allele in cache.entries.iter().filter_map(|v|v.chromosome.as_ref())
            .flat_map(|v|[v.primary.as_ref(),v.secondary.as_ref()]).flatten() {
            if allele.source_identity {
                let source=definitions.get(allele.uid.as_str()).ok_or_else(||anyhow::anyhow!("Native cache object lacks its bound source"))?;
                ensure!(source.dominant==allele.dominant&&source.slots.contains(&0)==allele.bee_species,
                    "Native cache allele differs from its bound object");
            }
        }
        Ok(())
    }
}
