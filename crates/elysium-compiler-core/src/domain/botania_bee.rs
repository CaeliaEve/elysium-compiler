//! Snapshot law precursor; no native callback admission or cache transitions.
use super::botania_cache::{Allele as CachedAllele,Chromosome as CachedChromosome,Read};
use super::botania_cache_machine::Load;
use crate::identity::Nbt;
use anyhow::{ensure,Result};
use schemars::JsonSchema;
use serde::{Deserialize,Serialize};
use std::collections::BTreeMap;

#[derive(Clone,Debug,Deserialize,Serialize,JsonSchema)]
#[schemars(rename="BeePoolAllele")]
#[serde(deny_unknown_fields)]
pub struct Allele {pub uid:String,pub dominant:bool,pub slots:Vec<u32>}
#[derive(Clone,Debug,Deserialize,Serialize,JsonSchema)]
#[schemars(rename="BeePoolRegistration")]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct Registration {pub key:String,pub uid:String,pub dominant:bool,pub slots:Vec<u32>,#[serde(default,skip_serializing_if="std::ops::Not::not")]pub alias_only:bool}
#[derive(Clone,Debug,Deserialize,Serialize,JsonSchema)]
#[schemars(rename="BeePoolPair")]
#[serde(deny_unknown_fields)]
pub struct Pair {
    #[serde(deserialize_with="super::nei_information::required_nullable")]
    #[schemars(with="super::nei_information::RequiredNullable<String>")]pub primary:Option<String>,
    #[serde(deserialize_with="super::nei_information::required_nullable")]
    #[schemars(with="super::nei_information::RequiredNullable<String>")]pub secondary:Option<String>,
}
#[derive(Clone,Debug,Deserialize,Serialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct Source {
    pub forestry:String,pub magicbees:String,pub root:String,pub chromosomes:Vec<String>,
    pub definitions:BTreeMap<String,Allele>,pub alleles:Vec<Registration>,pub aliases:BTreeMap<String,String>,
    pub templates:BTreeMap<String,Vec<Option<String>>>,pub defaults:Vec<Option<String>>,pub forest:Vec<Option<Pair>>,
    pub forms:BTreeMap<String,String>,pub clear_invalid_chromosomes:bool,
}
#[derive(Clone,Debug,Deserialize,Serialize,JsonSchema)]
#[schemars(rename="BeePoolStack")]
#[serde(deny_unknown_fields)]
pub struct Stack {pub registry:String,pub meta:i32,pub amount:String,
    #[serde(deserialize_with="super::nei_information::required_nullable")]
    #[schemars(with="super::nei_information::RequiredNullable<Nbt>")]pub nbt:Option<Nbt>}
#[derive(Clone,Debug,Serialize)]
#[serde(tag="state",rename_all="camelCase")]
pub enum Outcome {NotMatched,NativeError,Matched{output:Stack}}
#[derive(Clone,Debug,Serialize)]
#[serde(rename_all="camelCase")]
pub struct Transition {pub result:Outcome,pub offered_after:Option<Stack>}

#[derive(Debug)]
struct NativeFailure;
impl std::fmt::Display for NativeFailure {fn fmt(&self,f:&mut std::fmt::Formatter<'_>)->std::fmt::Result {write!(f,"Native pool callback failed")}}
impl std::error::Error for NativeFailure {}
fn native<T>()->Result<T>{Err(NativeFailure.into())}
fn text(tag:Option<&Nbt>)->&str {if let Some(Nbt::String{value})=tag {value}else{""}}
fn tag_byte(tag:Option<&Nbt>)->Result<i8> {
    fn floor(value:f64,single:bool)->i32 {
        let narrowed=value as i32;
        let converted=if single {(narrowed as f32) as f64}else{narrowed as f64};
        if value<converted {narrowed.wrapping_sub(1)}else{narrowed}
    }
    Ok(match tag {
        Some(Nbt::Byte{value})|Some(Nbt::Short{value})|Some(Nbt::Int{value})|Some(Nbt::Long{value})=>value.parse::<i64>()? as i8,
        Some(Nbt::Float{value})=>floor(f32::from_bits(u32::from_str_radix(value,16)?) as f64,true) as i8,
        Some(Nbt::Double{value})=>floor(f64::from_bits(u64::from_str_radix(value,16)?),false) as i8,
        _=>0,
    })
}
impl Source {
    fn descriptor(&self,uid:Option<&str>)->Option<&Allele>{uid.and_then(|uid|self.definitions.get(uid))}
    fn valid(allele:Option<&Allele>,slot:usize)->bool {allele.is_some_and(|a|a.slots.contains(&(slot as u32)))}
    fn active(&self,pair:Option<&Pair>)->Option<&Allele> {
        let pair=pair?;let a=self.descriptor(pair.primary.as_deref())?;let b=self.descriptor(pair.secondary.as_deref())?;
        Some(if a.dominant {a}else if b.dominant {b}else{a})
    }
    fn cached(&self,allele:Option<&Allele>)->Option<CachedAllele>{allele.map(|a|CachedAllele{
        uid:a.uid.clone(),dominant:a.dominant,source_identity:true,bee_species:Self::valid(Some(a),0)})}
    fn parse(&self,genome:&mut Nbt)->Result<Option<CachedChromosome>> {
        let count=self.chromosomes.len();let mut cells:Vec<Option<Pair>>=(0..count).map(|_|None).collect();
        let mut lookup=BTreeMap::<&str,&Allele>::new();
        for row in &self.alleles {if !row.alias_only {lookup.insert(&row.key,self.definitions.get(&row.uid).ok_or_else(||anyhow::anyhow!("Invalid native allele registration"))?);}}
        for (key,uid) in &self.aliases {if !lookup.contains_key(key.as_str()){lookup.insert(key,self.definitions.get(uid).ok_or_else(||anyhow::anyhow!("Invalid native alias descriptor"))?);}}
        if let Nbt::Compound{value}=genome {
            if let Some(Nbt::List{element,value})=value.get_mut("Chromosomes") {
                if element=="compound"||value.is_empty() {
                    ensure!(value.len()<=65536,"Native pool genome exceeds entry budget");
                    for raw in value {
                        let Nbt::Compound{value:entry}=raw else {continue;};
                        let slot=tag_byte(entry.get("Slot"))? as i32;
                        if slot<0||slot as usize>=count {continue;}let slot=slot as usize;
                        let mut a=lookup.get(text(entry.get("UID0"))).copied();
                        let mut b=lookup.get(text(entry.get("UID1"))).copied();
                        if self.clear_invalid_chromosomes {
                            let fallback=if Self::valid(a,slot){a}else if Self::valid(b,slot){b}else{self.descriptor(self.defaults[slot].as_deref())};
                            let mut changed=false;
                            if !Self::valid(a,slot){a=fallback;changed=true;}
                            if !Self::valid(b,slot){b=fallback;changed=true;}
                            if changed {
                                let (Some(a),Some(b))=(a,b) else {return native();};
                                entry.insert("UID0".into(),Nbt::String{value:a.uid.clone()});
                                entry.insert("UID1".into(),Nbt::String{value:b.uid.clone()});
                            }
                        }
                        if !Self::valid(a,slot)||!Self::valid(b,slot){return native();}
                        cells[slot]=Some(Pair{primary:a.map(|v|v.uid.clone()),secondary:b.map(|v|v.uid.clone())});
                    }
                }
            }
        }
        let species=self.active(cells[0].as_ref());
        if cells[0].is_some() {
            let Some(template)=species.and_then(|a|self.templates.get(&a.uid)) else {return native();};
            if template.len()!=count{return native();}
        }
        Ok(cells[0].as_ref().map(|cell| CachedChromosome{
            primary:self.cached(self.descriptor(cell.primary.as_deref())),
            secondary:self.cached(self.descriptor(cell.secondary.as_deref())),
        }))
    }
    pub(crate) fn load_genome(&self,genome:&mut Nbt)->Result<Load>{
        ensure!(self.forestry=="4.10.17"&&self.magicbees=="2.9.4-GTNH"&&self.chromosomes.len()==14
            &&self.defaults.len()==14&&self.forest.len()==14,"Invalid native pool source");
        match self.parse(genome){
            Ok(chromosome)=>Ok(Load::Loaded(chromosome)),
            Err(error) if error.is::<NativeFailure>()=>Ok(Load::NativeError),
            Err(error)=>Err(error),
        }
    }
    fn read(&self,genome:&mut Nbt,used:bool,reads:&[Read],at:&mut usize)->Result<Option<CachedAllele>> {
        let read=reads.get(*at).ok_or_else(||anyhow::anyhow!("Incomplete explicit pool cache-read trace"))?;*at+=1;
        match read {
            Read::Miss=>{
                let Some(cell)=self.parse(genome)? else{return Ok(None);};
                let (Some(a),Some(b))=(cell.primary,cell.secondary) else{return Ok(None);};
                Ok(Some(if a.dominant{a}else if b.dominant{b}else{a}))
            },
            Read::Hit{chromosome}=>{
                if !used{return Ok(None);}
                let Some(cell)=chromosome else{return Ok(None);};
                let (Some(a),Some(b))=(&cell.primary,&cell.secondary) else{return Ok(None);};
                for allele in [a,b] {ensure!(!allele.uid.is_empty()&&allele.uid.encode_utf16().count()<=16384,"Invalid cached pool allele");}
                Ok(Some(if a.dominant{a}else if b.dominant{b}else{a}.clone()))
            }
        }
    }
    pub fn evaluate(&self,form:&str,input:&str,output:&str,offered:Option<Stack>,reads:Vec<Read>)->Result<Outcome> {
        Ok(self.transition(form,input,output,offered,reads)?.result)
    }
    /// Match-time repairs remain observable after mismatch or native failure.
    /// Output conversion owns a second copy, as the native callback does.
    pub fn transition(&self,form:&str,input:&str,output:&str,mut offered:Option<Stack>,reads:Vec<Read>)->Result<Transition> {
        let result=if let Some(owned)=offered.as_mut(){self.evaluate_owned(form,input,output,owned,reads)?}else{Outcome::NotMatched};
        Ok(Transition{result,offered_after:offered})
    }
    fn evaluate_owned(&self,form:&str,input:&str,output:&str,owned:&mut Stack,reads:Vec<Read>)->Result<Outcome> {
        let registry=self.forms.get(form).ok_or_else(||anyhow::anyhow!("Unknown native pool form"))?;
        if *registry!=owned.registry{return Ok(Outcome::NotMatched);}
        ensure!(self.forestry=="4.10.17"&&self.magicbees=="2.9.4-GTNH"&&self.chromosomes.len()==14
            &&self.defaults.len()==14&&self.forest.len()==14&&reads.len()<=2,"Invalid native pool source or cache-read trace");
        if let Some(tag)=&owned.nbt {tag.validate()?;}
        let mut evaluate=||->Result<Outcome>{
            let mut at=0;let mut empty=BTreeMap::new();
            let tag=match &mut owned.nbt {Some(Nbt::Compound{value})=>Some(value),None=>None,_=>Some(&mut empty)};
            let species=if tag.as_ref().is_some_and(|tag|tag.contains_key("Genome")) {None}
                else{self.cached(self.active(self.forest[0].as_ref()))};
            let mut tag=tag;
            let species=if let Some(genome)=tag.as_mut().and_then(|tag|tag.get_mut("Genome")) {self.read(genome,true,&reads,&mut at)?}else{species};
            if let Some(mate)=tag.as_mut().and_then(|tag|tag.get_mut("Mate")){self.read(mate,false,&reads,&mut at)?;}
            ensure!(at==reads.len(),"Unused explicit pool cache-read trace");
            let Some(species)=species.filter(|a|a.bee_species) else{return native();};
            if !species.source_identity||species.uid!=input{return Ok(Outcome::NotMatched);}
            if tag.is_none(){return native();}
            let mut result=owned.clone();
            let Some(Nbt::Compound{value:result_tag})=&mut result.nbt else{return native();};
            let Some(Nbt::Compound{value:genome})=result_tag.get_mut("Genome") else{return native();};
            let Some(Nbt::List{value,..})=genome.get_mut("Chromosomes") else{return native();};
            if let Some(Nbt::Compound{value:first})=value.first_mut() {
                first.insert("UID0".into(),Nbt::String{value:output.into()});
                first.insert("UID1".into(),Nbt::String{value:output.into()});
            }
            result.amount="1".into();Ok(Outcome::Matched{output:result})
        };
        match evaluate(){Err(error) if error.is::<NativeFailure>()=>Ok(Outcome::NativeError),other=>other}
    }
}
