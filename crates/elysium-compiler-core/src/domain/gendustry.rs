//! Pinned Gendustry 1.9.4 machine rules; display examples never restrict the gene domain.
use super::{Consumption, Fluid, Item, Kind, Match, Recipe};
use crate::identity::Nbt;
use anyhow::{ensure, Context, Result};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap,BTreeSet};

#[derive(Clone,Copy,Debug,PartialEq,Eq,Serialize,Deserialize,JsonSchema)]
#[schemars(rename="GendustryOperation")]
#[serde(rename_all="camelCase")]
pub enum Operation { Imprint, Sample, TransposeSample, TransposeTemplate, Mutate, Replicate, Extract, Merge }
#[derive(Clone,Debug,Serialize,Deserialize,JsonSchema)]
#[schemars(rename="GendustryConfig")]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct Config {
    pub mj_per_item:f32,pub power_use_rate:f32,pub activation_energy:f32,pub max_stored_energy:f32,pub max_received_energy:f32,
    pub labware_consume_chance:Option<f32>,pub death_chance_natural:Option<f32>,pub death_chance_artificial:Option<f32>,
    pub mutagen_per_item:Option<i32>,pub tank_size:Option<i32>,pub secret_chance:Option<f32>,pub degrade_chance_natural:Option<f32>,
    pub dna_per_item:Option<i32>,pub protein_per_item:Option<i32>,pub dna_tank_size:Option<i32>,pub protein_tank_size:Option<i32>,pub make_pristine_bees:Option<bool>,
}
#[derive(Clone,Debug,Serialize,Deserialize,JsonSchema)]
#[schemars(rename="GendustryChromosome")]
#[serde(deny_unknown_fields)]
pub struct Chromosome { pub ordinal:u32,pub name:String }
#[derive(Clone,Debug,Serialize,Deserialize,JsonSchema)]
#[schemars(rename="GendustryTemplate")]
#[serde(deny_unknown_fields)]
pub struct Template { pub species:String,pub alleles:Vec<Option<String>> }
#[derive(Clone,Debug,Serialize,Deserialize,JsonSchema)]
#[schemars(rename="GendustryMutationProcess")]
#[serde(deny_unknown_fields)]
pub struct Mutation { #[serde(rename="class")]pub native_class:String,pub parent0:String,pub parent1:String,pub template:Vec<Option<String>>,pub secret:bool,pub conditions:Vec<String>,pub setting:MutationSetting,#[serde(rename="beeMutation")]pub bee_mutation:bool }
#[derive(Clone,Copy,Debug,PartialEq,Eq,Serialize,Deserialize,JsonSchema)]
#[schemars(rename="GendustryMutationSetting")]
#[serde(rename_all="lowercase")]
pub enum MutationSetting { Enabled, Disabled, Requirements }
#[derive(Clone,Copy,Debug,PartialEq,Eq,Serialize,Deserialize,JsonSchema)]
#[schemars(rename="GendustryRootKind")]
#[serde(rename_all="lowercase")]
pub enum RootKind { Bee, Tree, Butterfly, Other }
#[derive(Clone,Debug,Serialize,Deserialize,JsonSchema)]
#[schemars(rename="GendustryMemberForm")]
#[serde(deny_unknown_fields)]
pub struct MemberForm { pub ordinal:u32,pub registry:String,pub meta:i32,pub nbt:Option<Nbt> }
#[derive(Clone,Debug,Serialize,Deserialize,JsonSchema)]
#[schemars(rename="GendustryLifespan")]
#[serde(deny_unknown_fields)]
pub struct Lifespan { pub chromosome:u32,pub values:BTreeMap<String,i32> }
#[derive(Clone,Debug,Serialize,Deserialize,JsonSchema)]
#[schemars(rename="GendustryRoot")]
#[serde(deny_unknown_fields)]
pub struct Root { pub uid:String,#[serde(rename="class")]pub native_class:String,pub kind:RootKind,#[serde(rename="speciesChromosome")]pub species_chromosome:u32,pub lifespan:Option<Lifespan>,pub forms:Vec<MemberForm>,pub chromosomes:Vec<Chromosome>,pub required:Vec<u32>,#[serde(rename="defaultTemplate")]pub default_template:Vec<Option<String>>,pub templates:Vec<Template>,pub mutations:Vec<Mutation> }
#[derive(Clone,Debug,Serialize,Deserialize,JsonSchema)]
#[schemars(rename="GendustryDnaSource")]
#[serde(deny_unknown_fields)]
pub struct DnaSource { pub registry:String,pub meta:i32,pub amount:i32 }
/// A displayed NEI result is one observation of the machine callback. These
/// variants retain the other outcomes of the same input tuple and RNG draw.
#[derive(Clone,Debug,Serialize,Deserialize,JsonSchema)]
#[schemars(rename="GendustryOutputPolicy")]
#[serde(tag="kind",rename_all="camelCase",deny_unknown_fields)]
pub enum OutputPolicy {
    Direct,
    Sampler { #[serde(rename="sampleRegistry")] sample_registry:String, #[serde(rename="sampleMeta")] sample_meta:i32 },
    /// A non-drone bee may die before imprinting; natural/artificial rates
    /// come from Config and the subsequent labware draw still takes place.
    Imprinter { #[serde(rename="wasteRegistry")] waste_registry:String },
    /// World-restricted mutations can leave no candidate. A selected natural
    /// bee may degrade; an artificial bee may die and become waste.
    Mutatron { #[serde(rename="wasteRegistry")] waste_registry:String, #[serde(rename="emptyCandidatesReturnNull")] empty_candidates_return_null:bool },
}
#[derive(Clone,Debug,Serialize,Deserialize,JsonSchema)]
#[schemars(rename="GendustryGeneticsProcess")]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct GeneticsProcess { pub operation:Operation,pub config:Config,pub roots:Vec<Root>,pub slots:BTreeMap<String,u32>,pub rng_bound:u32,pub labware_threshold:Option<f32>,pub selector:u32,pub sources:Option<Vec<DnaSource>>,pub output_policy:OutputPolicy }
#[derive(Clone,Debug,Serialize,Deserialize,JsonSchema)]
#[schemars(rename="GendustryTemplateProcess")]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct TemplateProcess { pub operation:Operation,pub template:u32,pub samples:Vec<u32>,pub minimum_samples:u32,pub maximum_samples:u32,pub scan_order:String,pub roots:Vec<Root> }

fn roots_valid(roots:&[Root])->Result<()> {
    ensure!(roots.len()<=65536,"Gendustry root budget exceeded");let mut ids=BTreeSet::new();
    for root in roots {ensure!(!root.uid.is_empty()&&ids.insert(&root.uid)&&root.chromosomes.len()<=256,"Invalid Gendustry root");
        let ordinals:BTreeSet<_>=root.chromosomes.iter().map(|g|g.ordinal).collect();ensure!(ordinals.len()==root.chromosomes.len()&&ordinals.contains(&root.species_chromosome)&&root.required.iter().all(|g|ordinals.contains(g))&&root.default_template.len()==root.chromosomes.len(),"Invalid required/default chromosome");
        match (&root.kind,&root.lifespan) {
            (RootKind::Bee|RootKind::Butterfly,Some(life))=>ensure!(ordinals.contains(&life.chromosome)
                && !life.values.is_empty() && life.values.len()<=65536 && life.values.keys().all(|uid|!uid.is_empty()),
                "Invalid native living member lifespan registry"),
            (RootKind::Bee|RootKind::Butterfly,None)=>anyhow::bail!("Living native member requires its lifespan serialization law"),
            (_,None)=>{},(_,Some(_))=>anyhow::bail!("Lifespan law attached to a nonliving species root"),
        }
        for template in &root.templates {ensure!(template.alleles.len()==root.chromosomes.len(),"Native default genome width differs");}
        for mutation in &root.mutations {ensure!(mutation.template.len()==root.chromosomes.len()&&!mutation.parent0.is_empty()&&!mutation.parent1.is_empty(),"Native mutation genome differs");
            ensure!(mutation.conditions.is_empty(),"Localized mutation descriptions belong in source-bound notes, not semantic identity");}
    }Ok(())
}
pub(super) fn validate_genetics(p:&GeneticsProcess,r:&Recipe,items:&BTreeMap<&str,&Item>,fluids:&BTreeMap<&str,&Fluid>)->Result<()> {
    roots_valid(&p.roots)?;let machine=match p.operation{Operation::Imprint=>"Imprinter",Operation::Sample=>"Sampler",Operation::TransposeSample|Operation::TransposeTemplate=>"Transposer",Operation::Mutate=>"Mutatron",Operation::Replicate=>"Replicator",Operation::Extract=>"Extractor",Operation::Merge=>anyhow::bail!("Merge requires crafting")};
    ensure!(r.source.handler==format!("net.bdew.gendustry.nei.{machine}Handler")&&r.energy.is_none()&&r.duration.is_none()&&r.grid.is_none()&&r.magic.is_none(),"Gendustry native process acquired fixed timing/crafting");
    ensure!(p.rng_bound==100&&p.config.mj_per_item.is_finite()&&p.config.mj_per_item>0.0&&p.config.power_use_rate.is_finite()&&p.config.power_use_rate>=0.0&&p.config.activation_energy.is_finite()&&p.config.activation_energy>=0.0,"Invalid native processor energy/RNG");
    match (&p.operation,&p.output_policy) {
        (Operation::Sample,OutputPolicy::Sampler{sample_registry,sample_meta})=>ensure!(!sample_registry.is_empty()&&*sample_meta>=0,"Native sampler item binding missing"),
        (Operation::Imprint,OutputPolicy::Imprinter{waste_registry})=>{
            ensure!(!waste_registry.is_empty()&&p.config.death_chance_natural.is_some_and(f32::is_finite)&&p.config.death_chance_artificial.is_some_and(f32::is_finite),"Imprinter mortality policy/configuration missing");
        },
        (Operation::Mutate,OutputPolicy::Mutatron{waste_registry,empty_candidates_return_null})=>{
            ensure!(!waste_registry.is_empty()&&*empty_candidates_return_null&&p.config.degrade_chance_natural.is_some_and(f32::is_finite)&&p.config.death_chance_artificial.is_some_and(f32::is_finite)&&p.config.secret_chance.is_some_and(f32::is_finite),"Mutatron conditional outcome policy/configuration missing");
        },
        (Operation::Imprint|Operation::Mutate|Operation::Sample,_)=>anyhow::bail!("Gendustry native output policy missing"),
        (_,OutputPolicy::Direct)=>{},
        _=>anyhow::bail!("Gendustry conditional policy attached to another machine"),
    }
    let roles:&[&str]=match p.operation {Operation::Imprint=>&["individual","template","labware"],Operation::Sample=>&["individual","blank","labware"],Operation::TransposeSample|Operation::TransposeTemplate=>&["blank","template","labware"],Operation::Mutate=>&["parent0","parent1","labware"],Operation::Replicate=>&["template"],Operation::Extract=>&["individual","labware"],_=>unreachable!()};
    ensure!(p.slots.len()==roles.len()&&roles.iter().all(|role|p.slots.contains_key(*role)),"Native Gendustry slot roles differ");
    let mut slots=BTreeSet::new();for (role,slot) in &p.slots {ensure!(slots.insert(slot),"Gendustry input roles overlap");let input=r.inputs.iter().find(|i|i.kind==Kind::Item&&i.slot==*slot).context("Missing Gendustry input")?;
        for choice in &input.choices {ensure!(choice.amount=="1"&&choice.returns.is_empty()&&items.contains_key(choice.id.as_str())
            &&(p.operation==Operation::Extract&&role=="individual"||matches!(choice.rule,Match::Wildcard{nbt:true,..})),"Gendustry gene predicate restricted offered NBT");
            if role=="template"&&matches!(p.operation,Operation::Imprint|Operation::TransposeSample|Operation::TransposeTemplate|Operation::Replicate){ensure!(matches!(choice.consume,Consumption::Keep),"Native template was consumed");}
        }
    }
    if p.operation==Operation::Replicate {ensure!(p.labware_threshold.is_none()&&p.config.labware_consume_chance.is_none()&&p.config.dna_per_item.is_some()&&p.config.protein_per_item.is_some()&&p.config.make_pristine_bees.is_some(),"Replicator fluid/configuration missing");}
    else {ensure!(p.labware_threshold==p.config.labware_consume_chance&&p.labware_threshold.is_some_and(f32::is_finite),"Native labware compare threshold differs");}
    if p.operation==Operation::Extract {validate_extractor(p,r,items,fluids)?;}
    else {ensure!(p.sources.is_none(),"DNA sources attached to unrelated machine");}
    let fluid_inputs:Vec<_>=r.inputs.iter().filter(|input|input.kind==Kind::Fluid).collect();
    match p.operation {
        Operation::Mutate=>{let amount=p.config.mutagen_per_item.context("Mutagen requirement missing")?;ensure!(fluid_inputs.len()==1&&fluid_inputs[0].choices.iter().all(|choice|choice.amount==amount.to_string()),"Mutatron fluid stock requirement differs");},
        Operation::Replicate=>{ensure!(fluid_inputs.len()==2,"Replicator requires separate DNA and protein stock");let amounts=[p.config.dna_per_item.context("DNA requirement missing")?,p.config.protein_per_item.context("Protein requirement missing")?];for (input,amount) in fluid_inputs.iter().zip(amounts.iter()){ensure!(input.choices.iter().all(|choice|choice.amount==amount.to_string()),"Replicator fluid stock requirement differs");}},
        _=>ensure!(fluid_inputs.is_empty(),"Unexpected Gendustry fluid stock gate"),
    }
    if let Some(threshold)=p.labware_threshold {let accepted=(threshold.ceil() as i32).clamp(0,100) as u64;let slot=p.slots["labware"];let input=r.inputs.iter().find(|input|input.kind==Kind::Item&&input.slot==slot).context("Labware stock gate missing")?;
        for choice in &input.choices {let valid=match &choice.consume {Consumption::Keep=>accepted==0,Consumption::Consume=>accepted==100,Consumption::Chance{chance}=>{let numerator=chance.numerator.parse::<u128>()?;let denominator=chance.denominator.parse::<u128>()?;denominator>0&&numerator.checked_mul(100).is_some_and(|left|u128::from(accepted).checked_mul(denominator)==Some(left))},_=>false};ensure!(valid,"Labware consumption differs from native nextInt(100) threshold");}
    }
    ensure!(r.outputs.len()==1&&r.outputs[0].kind==if p.operation==Operation::Extract{Kind::Fluid}else{Kind::Item},"Gendustry process must keep one native result kind");
    ensure!(r.inputs.iter().filter(|i|i.kind==Kind::Item).count()==roles.len(),"Unexpected native Gendustry item input");
    ensure!(p.operation==Operation::Extract||r.outputs[0].change.is_some(),"Gendustry dynamic result action is missing");Ok(())
}

fn validate_extractor(p:&GeneticsProcess,r:&Recipe,items:&BTreeMap<&str,&Item>,fluids:&BTreeMap<&str,&Fluid>)->Result<()> {
    let sources=p.sources.as_ref().context("Extractor DNA source registry missing")?;
    let input=r.inputs.iter().find(|i|i.kind==Kind::Item&&i.slot==p.slots["individual"]).context("Extractor specimen missing")?;
    ensure!(input.choices.len()==1,"Extractor must identify one native source partition");
    let choice=&input.choices[0];
    ensure!(matches!(choice.consume,Consumption::Consume),"Native DNA source is not consumed");
    let amount=super::gendustry_fluid::source_amount(sources,choice,items)?;
    let output=r.outputs.first().context("DNA output missing")?;
    let fluid=fluids.get(output.id.as_str()).context("DNA fluid fact missing")?;
    ensure!(r.outputs.len()==1&&output.kind==Kind::Fluid&&output.amount.as_deref()==Some(amount.to_string().as_str())
        &&output.quantity.is_none()&&output.change.is_none()&&output.chance.numerator=="1"&&output.chance.denominator=="1"
        &&fluid.registry=="liquiddna"&&fluid.nbt.is_none()&&p.config.tank_size.is_some_and(|capacity|amount<=capacity),"Native DNA batch/fluid/tank differs");
    Ok(())
}
pub(super) fn validate_template(p:&TemplateProcess,r:&Recipe,items:&BTreeMap<&str,&Item>)->Result<()> {
    roots_valid(&p.roots)?;ensure!(p.operation==Operation::Merge&&r.source.handler=="net.bdew.gendustry.nei.TemplateCraftingHandler"&&p.minimum_samples==1&&p.maximum_samples==8&&p.scan_order=="columnMajor"&&r.grid.is_none()&&r.duration.is_none()&&r.energy.is_none()&&r.magic.is_none(),"GeneRecipe native domain/scan order differs");
    ensure!(!p.samples.is_empty()&&p.samples.len()<=8&&!p.samples.contains(&p.template)&&p.samples.iter().collect::<BTreeSet<_>>().len()==p.samples.len(),"GeneRecipe sample roles overlap");
    ensure!(r.inputs.len()==p.samples.len()+1,"Unexpected GeneRecipe input role");
    for slot in std::iter::once(&p.template).chain(p.samples.iter()) {
        let input=r.inputs.iter().find(|i|i.kind==Kind::Item&&i.slot==*slot).context("Template input missing")?;
        ensure!(input.choices.len()==1,"GeneRecipe example role must identify one native item");
        for choice in &input.choices {
            let item=items.get(choice.id.as_str()).context("Template role item missing")?;
            ensure!(item.registry==if *slot==p.template{"gendustry:GeneTemplate"}else{"gendustry:GeneSample"}
                &&item.meta==0&&item.nbt.is_none()&&choice.amount=="1"
                &&matches!(choice.consume,Consumption::Consume)&&matches!(choice.rule,Match::Wildcard{meta:true,nbt:true}),
                "GeneRecipe consumes one per occupied slot and ignores offered meta/NBT");
            if *slot==p.template {
                ensure!(choice.returns.is_empty(),"Native gene template has no crafting container");
            } else {
                ensure!(choice.returns.len()==1,"Native gene sample container return missing");
                let returned=&choice.returns[0];
                let blank=items.get(returned.id.as_str()).context("Blank gene sample item missing")?;
                ensure!(returned.kind==Kind::Item&&returned.amount=="1"&&blank.registry=="gendustry:GeneSampleBlank"
                    &&blank.meta==0&&blank.nbt.is_none(),"Native gene sample container return differs");
            }
        }
    }
    ensure!(r.outputs.len()==1&&r.outputs[0].change.is_some(),"GeneRecipe dynamic result missing");Ok(())
}

/// Native addSample removes the previous chromosome then appends its replacement.
/// Root tags, item metadata and offered stack count are retained by GeneRecipe.copy().
pub fn merge_template(mut tags:BTreeMap<String,Nbt>,samples:&[BTreeMap<String,Nbt>],valid_roots:&BTreeSet<String>)->Result<BTreeMap<String,Nbt>> {
    ensure!((1..=8).contains(&samples.len()),"GeneRecipe requires 1..8 samples");
    for sample in samples {let species=string(sample.get("species"));let allele=string(sample.get("allele"));let ordinal=integer(sample.get("chromosome"));
        ensure!(!allele.is_empty()&&valid_roots.contains(&species),"Unknown native gene root/allele");
        if tags.is_empty(){tags.insert("species".into(),Nbt::String{value:species.clone()});}
        ensure!(string(tags.get("species"))==species,"Native addSample rejects another species root");
        let mut genes=match tags.remove("samples"){Some(Nbt::List{element,value})if element=="compound"=>value,Some(Nbt::List{value,..})if value.is_empty()=>value,None=>Vec::new(),_=>anyhow::bail!("Malformed native gene sample list")};
        genes.retain(|gene|!matches!(gene,Nbt::Compound{value}if integer(value.get("chromosome"))==ordinal));
        genes.push(Nbt::Compound{value:sample.clone()});tags.insert("samples".into(),Nbt::List{element:"compound".into(),value:genes});
    }Ok(tags)
}
fn string(value:Option<&Nbt>)->String{match value{Some(Nbt::String{value})=>value.clone(),_=>String::new()}}
fn integer(value:Option<&Nbt>)->i32{match value{Some(Nbt::Int{value})|Some(Nbt::Byte{value})|Some(Nbt::Short{value})=>value.parse().unwrap_or(0),_=>0}}

fn compound(value:Option<&Nbt>)->Result<&BTreeMap<String,Nbt>> {match value{Some(Nbt::Compound{value})=>Ok(value),_=>anyhow::bail!("Missing native compound")}}
fn genes(tags:&BTreeMap<String,Nbt>)->Result<Vec<BTreeMap<String,Nbt>>> {
    let genome=compound(tags.get("Genome"))?;match genome.get("Chromosomes"){Some(Nbt::List{element,value})if element=="compound"=>value.iter().map(|row|compound(Some(row)).cloned()).collect(),_=>anyhow::bail!("Missing native chromosomes")}
}
fn samples(tags:&BTreeMap<String,Nbt>)->Result<Vec<BTreeMap<String,Nbt>>> {match tags.get("samples"){Some(Nbt::List{element,value})if element=="compound"=>value.iter().map(|row|compound(Some(row)).cloned()).collect(),None=>Ok(Vec::new()),Some(Nbt::List{value,..})if value.is_empty()=>Ok(Vec::new()),_=>anyhow::bail!("Malformed gene template samples")}}
fn member(root:&Root,item:&Item)->bool{root.forms.iter().any(|form|form.registry==item.registry&&form.meta==item.meta)}
fn waste(item:&Item,registry:&str)->bool{item.registry==registry&&item.meta==0&&item.nbt.is_none()}
/// Forestry's absent NA tag means natural; it only writes NA=false for artificial bees.
fn natural(tags:&BTreeMap<String,Nbt>)->bool { !tags.contains_key("NA") || integer(tags.get("NA"))!=0 }
fn threshold_successes(threshold:f32)->i32 { (threshold.ceil() as i32).clamp(0,100) }
fn parent_species(root:&Root,item:&Item,position:u32)->Result<Option<String>> {
    let required=match root.kind {RootKind::Bee=>Some(if position==0{1}else{0}),RootKind::Tree=>Some(if position==0{0}else{2}),_=>None};
    if !root.forms.iter().any(|form|form.registry==item.registry&&form.meta==item.meta&&required.is_none_or(|ordinal|form.ordinal==ordinal)){return Ok(None);}
    let Some(Nbt::Compound{value:tags})=&item.nbt else{return Ok(None)};
    Ok(genes(tags)?.iter().find(|gene|integer(gene.get("Slot"))==root.species_chromosome as i32)
        .map(|gene|string(gene.get("UID0"))).filter(|species|!species.is_empty()))
}
fn mutation_result(root:&Root,parent0:&Item,parent1:&Item,item:&Item,config:&Config,waste_registry:&str,
    restricted_bee_chance:impl FnMut(&Mutation)->Result<bool>)->Result<bool>{
    let Some(first)=parent_species(root,parent0,0)? else{return Ok(false)};
    let Some(second)=parent_species(root,parent1,1)? else{return Ok(false)};
    let mut candidates=mutation_candidates(root,&first,&second,restricted_bee_chance)?;
    let secret=threshold_successes(config.secret_chance.context("Secret selection threshold missing")?);
    if candidates.iter().any(|m|m.secret)&&candidates.iter().any(|m|!m.secret) {
        candidates.retain(|m|if m.secret{secret>0}else{secret<100});
    }
    if candidates.is_empty(){return Ok(false);}
    let parent_natural=natural(compound(parent0.nbt.as_ref())?);
    let death=threshold_successes(config.death_chance_artificial.context("Artificial mortality threshold missing")?);
    if root.kind==RootKind::Bee&&!parent_natural&&waste(item,waste_registry){return Ok(death>0);}
    if root.kind==RootKind::Bee&&!parent_natural&&death==100{return Ok(false);}
    let ordinal=if root.kind==RootKind::Bee{2}else{0};
    if !root.forms.iter().any(|form|form.ordinal==ordinal&&form.registry==item.registry&&form.meta==item.meta){return Ok(false);}
    let Some(Nbt::Compound{value:tags})=item.nbt.as_ref() else{return Ok(false)};
    if root.kind==RootKind::Bee {
        let degraded=threshold_successes(config.degrade_chance_natural.context("Natural degradation threshold missing")?);
        let result_natural=natural(tags);
        if (!parent_natural&&result_natural)||(parent_natural&&if result_natural{degraded==100}else{degraded==0})
            || integer(tags.get("GEN"))!=0 {return Ok(false);}
    }
    Ok(candidates.iter().any(|mutation| {
        let mut rows=Vec::new();
        for (slot,allele) in mutation.template.iter().enumerate() {
            let Some(uid)=allele else{return false};
            rows.push(BTreeMap::from([("Slot".into(),Nbt::Byte{value:slot.to_string()}),
                ("UID0".into(),Nbt::String{value:uid.clone()}),("UID1".into(),Nbt::String{value:uid.clone()})]));
        }
        let expected=encoded_genome(rows);
        if let Some(life)=&root.lifespan {
            let Some(uid)=mutation.template.get(life.chromosome as usize).and_then(Option::as_ref) else{return false};
            let Some(health)=life.values.get(uid) else{return false};
            if ["Health","MaxH"].iter().any(|key|tags.get(*key)!=Some(&Nbt::Int{value:health.to_string()})){return false;}
        }
        tags.get("Genome")==Some(&expected)&&(root.kind!=RootKind::Bee||tags.get("Mate")==Some(&expected))
    }))
}
fn encoded_genome(rows:Vec<BTreeMap<String,Nbt>>)->Nbt {Nbt::Compound{value:BTreeMap::from([("Chromosomes".into(),Nbt::List{element:"compound".into(),value:rows.into_iter().map(|value|Nbt::Compound{value}).collect()})])}}

/// Native imprint retains all offered root tags, but constructs a new serialized Genome.
/// Bee death/degradation and labware draws remain correlated process outcomes.
pub fn imprint(root:&Root,individual:&Item,template:&Item)->Result<Nbt>{
    ensure!(member(root,individual),"Imprint input is not a native member form");let template_tags=compound(template.nbt.as_ref())?;
    ensure!(string(template_tags.get("species"))==root.uid,"Imprint roots differ");let mut tags=compound(individual.nbt.as_ref())?.clone();let mut genome=genes(&tags)?;let offered=samples(template_tags)?;
    for sample in &offered {ensure!(string(sample.get("species"))==root.uid,"Template sample root differs");let index=integer(sample.get("chromosome"));let allele=string(sample.get("allele"));ensure!(!allele.is_empty()&&index>=0&&(index as usize)<root.chromosomes.len(),"Invalid imprint gene");
        let gene=genome.iter_mut().find(|g|integer(g.get("Slot"))==index).context("Native chromosome slot missing")?;
        gene.insert("UID0".into(),Nbt::String{value:allele.clone()});gene.insert("UID1".into(),Nbt::String{value:allele});
    }
    // Genome.writeToNBT writes just Slot, UID0 and UID1, discarding extra offered genome fields.
    for gene in &mut genome {gene.retain(|key,_|matches!(key.as_str(),"Slot"|"UID0"|"UID1"));}
    tags.insert("Genome".into(),encoded_genome(genome));let complete=root.required.iter().all(|required|offered.iter().any(|sample|integer(sample.get("chromosome"))==*required as i32));
    if integer(tags.get("IsAnalyzed"))!=0||complete{tags.insert("IsAnalyzed".into(),Nbt::Byte{value:"1".into()});}
    Ok(Nbt::Compound{value:tags})
}

/// The sampler's native draw domain contains both haplotypes of every non-null chromosome.
/// Identical alleles remain two distinct equally likely outcomes.
pub fn sample_outcomes(root:&Root,individual:&Item)->Result<Vec<Nbt>>{
    ensure!(member(root,individual),"Sampler input is not a native member form");let tags=compound(individual.nbt.as_ref())?;let mut result=Vec::new();
    for gene in genes(tags)? {let index=integer(gene.get("Slot"));for key in ["UID0","UID1"]{let allele=string(gene.get(key));ensure!(!allele.is_empty(),"Native chromosome allele is unresolved");result.push(Nbt::Compound{value:BTreeMap::from([
        ("species".into(),Nbt::String{value:root.uid.clone()}),("chromosome".into(),Nbt::Int{value:index.to_string()}),("allele".into(),Nbt::String{value:allele})])});}}
    ensure!(!result.is_empty(),"Native sampler has no draw candidates");Ok(result)
}

/// Replicator overwrites the array returned by getDefaultTemplate and emits
/// a fresh self-mated queen/sapling/butterfly (other roots use form zero). Return both result
/// tags and that array state. Official Forestry providers return a copy; custom providers
/// that share their default array need separate source/ownership evidence.
pub fn replicate(root:&Root,template:&Item,pristine:bool)->Result<(MemberForm,Vec<Option<String>>)> {
    let tags=compound(template.nbt.as_ref())?;ensure!(string(tags.get("species"))==root.uid,"Replication root differs");let offered=samples(tags)?;
    ensure!(root.required.iter().all(|required|offered.iter().any(|sample|integer(sample.get("chromosome"))==*required as i32)),"Replicator requires a complete template");
    let mut defaults=root.default_template.clone();for sample in offered {let index=integer(sample.get("chromosome"));ensure!(index>=0&&(index as usize)<defaults.len(),"Replication chromosome outside root");defaults[index as usize]=Some(string(sample.get("allele")));}
    let ordinal=if root.kind==RootKind::Bee{2}else{0};let mut form=root.forms.iter().find(|form|form.ordinal==ordinal).context("Native replicated member form missing")?.clone();let mut result=compound(form.nbt.as_ref())?.clone();
    let mut rows=Vec::new();for (index,allele) in defaults.iter().enumerate(){if let Some(allele)=allele{rows.push(BTreeMap::from([("Slot".into(),Nbt::Byte{value:index.to_string()}),("UID0".into(),Nbt::String{value:allele.clone()}),("UID1".into(),Nbt::String{value:allele.clone()})]));}}
    let genome=encoded_genome(rows);result.insert("Genome".into(),genome.clone());
    if let Some(life)=&root.lifespan {
        let uid=defaults.get(life.chromosome as usize).and_then(Option::as_ref).context("Replicated lifespan chromosome missing")?;
        let health=life.values.get(uid).context("Replicated lifespan allele is not registered")?;
        for key in ["Health","MaxH"] {result.insert(key.into(),Nbt::Int{value:health.to_string()});}
    }
    result.insert("IsAnalyzed".into(),Nbt::Byte{value:"1".into()});
    if root.kind==RootKind::Bee {
        result.insert("Mate".into(),genome);
        if pristine {result.remove("NA");} else {result.insert("NA".into(),Nbt::Byte{value:"0".into()});}
    }form.nbt=Some(Nbt::Compound{value:result});Ok((form,defaults))
}

/// Mutatron overrides are evaluated before the conditional bee callback. Genome/world
/// conditions are supplied by the explicit native environment; labels are never predicates.
pub fn mutation_candidates<'a>(root:&'a Root,parent0:&str,parent1:&str,mut restricted_bee_chance:impl FnMut(&Mutation)->Result<bool>)->Result<Vec<&'a Mutation>> {
    let mut candidates=Vec::new();for mutation in &root.mutations {
        if !((mutation.parent0==parent0&&mutation.parent1==parent1)||(mutation.parent0==parent1&&mutation.parent1==parent0)){continue;}
        match mutation.setting {MutationSetting::Disabled=>continue,MutationSetting::Requirements if mutation.bee_mutation=>if !restricted_bee_chance(mutation)?{continue;},_=>{}}
        candidates.push(mutation);
    }Ok(candidates)
}

/// Native FluidSourceRegistry.getValue checks exact meta first, then wildcard, and returns zero for an unknown item.
/// A zero or negative exact entry still blocks wildcard fallback.
pub fn dna_amount(sources:&[DnaSource],item:&Item)->i32{sources.iter().find(|s|s.registry==item.registry&&s.meta==item.meta).or_else(||sources.iter().find(|s|s.registry==item.registry&&s.meta==32767)).map(|s|s.amount).unwrap_or(0)}

pub(super) fn validate_change(recipe:&Recipe,output:&super::Output,items:&BTreeMap<&str,&Item>,operation:Operation,slots:&BTreeMap<String,u32>)->Result<()> {
    let change=output.change.as_ref().context("Gendustry output change missing")?;ensure!(output.kind==Kind::Item&&output.quantity.is_none()&&output.chance.numerator=="1"&&output.chance.denominator=="1","Native dynamic item outcome acquired independent probability/quantity");
    let roots=match &recipe.process {Some(super::Process::GendustryGenetics(p))=>{ensure!(p.operation==operation&&p.slots==*slots,"Gendustry action/process roles differ");&p.roots},Some(super::Process::GendustryTemplate(p))=>{ensure!(operation==Operation::Merge&&slots.get("template")==Some(&p.template),"Template action roles differ");&p.roots},_=>anyhow::bail!("Gendustry edit lacks its native process")};
    let bindings=change.bindings.as_ref().context("Gendustry observation tuples missing")?;ensure!(bindings.len()==change.samples.len(),"Gendustry observations omit tuples");
    for (sample,binding) in change.samples.iter().zip(bindings) {ensure!(binding.len()==recipe.inputs.len(),"Gendustry tuple width differs");let sample_item=items.get(sample.id.as_str()).context("Native result fact missing")?;
        let input_item=|slot:u32|->Result<&Item>{let (position,input)=recipe.inputs.iter().enumerate().find(|(_,i)|i.kind==Kind::Item&&i.slot==slot).context("Native action input missing")?;let index=binding[position].context("Required native input absent")? as usize;let choice=input.choices.get(index).context("Native tuple choice outside input")?;Ok(*items.get(choice.id.as_str()).context("Native input fact missing")?)};
        match operation {
            Operation::Merge=>{let Some(super::Process::GendustryTemplate(p))=&recipe.process else{unreachable!()};let offered=input_item(p.template)?;
                let samples=p.samples.iter().map(|slot|compound(input_item(*slot)?.nbt.as_ref()).cloned()).collect::<Result<Vec<_>>>()?;
                let tags=offered.nbt.as_ref().map(|n|compound(Some(n)).cloned()).transpose()?.unwrap_or_default();let expected=merge_template(tags,&samples,&roots.iter().map(|r|r.uid.clone()).collect())?;
                ensure!(sample_item.registry==offered.registry&&sample_item.meta==offered.meta&&sample_item.nbt==Some(Nbt::Compound{value:expected}),"Native GeneRecipe observation differs from offered genes/tags");
            },
            Operation::TransposeSample=>{let offered=input_item(*slots.get("template").context("Source sample missing")?)?;ensure!(sample_item.registry==offered.registry&&sample_item.meta==offered.meta&&sample_item.nbt==offered.nbt,"Native sample copy lost offered state");},
            Operation::TransposeTemplate=>{let blank=input_item(*slots.get("blank").context("Blank template missing")?)?;let template=input_item(*slots.get("template").context("Source template missing")?)?;let offered=samples(compound(template.nbt.as_ref())?)?;
                let tags=blank.nbt.as_ref().map(|n|compound(Some(n)).cloned()).transpose()?.unwrap_or_default();let expected=if offered.is_empty(){tags}else{merge_template(tags,&offered,&roots.iter().map(|r|r.uid.clone()).collect())?};ensure!(sample_item.registry==blank.registry&&sample_item.meta==blank.meta&&sample_item.nbt==Some(Nbt::Compound{value:expected}),"Native template copy lost offered blank/source state");},
            Operation::Imprint=>{let input=input_item(*slots.get("individual").context("Imprint individual missing")?)?;let template=input_item(*slots.get("template").context("Imprint template missing")?)?;let root=roots.iter().find(|r|member(r,input)).context("Native individual root missing")?;
                let Some(super::Process::GendustryGenetics(p))=&recipe.process else{unreachable!()};let OutputPolicy::Imprinter{waste_registry}=&p.output_policy else{unreachable!()};
                let can_die=root.kind==RootKind::Bee&&root.forms.iter().any(|form|form.ordinal!=0&&form.registry==input.registry&&form.meta==input.meta);
                let death=if can_die {
                    let tags=compound(input.nbt.as_ref())?;
                    threshold_successes(if natural(tags){p.config.death_chance_natural}else{p.config.death_chance_artificial}
                        .context("Imprinter mortality threshold missing")?)
                }else{0};
                let successful=sample_item.registry==input.registry&&sample_item.meta==input.meta&&sample_item.nbt==Some(imprint(root,input,template)?);
                ensure!(sample.amount=="1"&&((death<100&&successful)||(death>0&&waste(sample_item,waste_registry))),"Native imprint observation is outside success/mortality branches");},
            Operation::Sample=>{let input=input_item(*slots.get("individual").context("Sampler individual missing")?)?;let root=roots.iter().find(|r|member(r,input)).context("Native individual root missing")?;
                let Some(super::Process::GendustryGenetics(p))=&recipe.process else{unreachable!()};
                let OutputPolicy::Sampler{sample_registry,sample_meta}=&p.output_policy else{anyhow::bail!("Native sampler item binding missing")};
                ensure!(sample_item.registry==*sample_registry&&sample_item.meta==*sample_meta
                    &&sample_item.nbt.as_ref().is_some_and(|nbt|sample_outcomes(root,input).is_ok_and(|outcomes|outcomes.contains(nbt)))&&sample.amount=="1","Native sampler observation is outside item/genome draw domain");},
            Operation::Replicate=>{let input=input_item(*slots.get("template").context("Replication template missing")?)?;let root=roots.iter().find(|r|input.nbt.as_ref().and_then(|n|compound(Some(n)).ok()).is_some_and(|tags|string(tags.get("species"))==r.uid)).context("Native template root missing")?;
                let Some(super::Process::GendustryGenetics(p))=&recipe.process else{unreachable!()};let (expected,_)=replicate(root,input,p.config.make_pristine_bees.context("Pristine setting missing")?)?;ensure!(sample_item.registry==expected.registry&&sample_item.meta==expected.meta&&sample_item.nbt==expected.nbt&&sample.amount=="1","Native replication observation differs");},
            Operation::Mutate=>{let input=input_item(*slots.get("parent0").context("First mutation parent missing")?)?;let second=input_item(*slots.get("parent1").context("Second mutation parent missing")?)?;let root=roots.iter().find(|r|member(r,input)).context("Native mutation root missing")?;
                let Some(super::Process::GendustryGenetics(p))=&recipe.process else{unreachable!()};let OutputPolicy::Mutatron{waste_registry,..}=&p.output_policy else{unreachable!()};
                // Exported NEI prototypes do not witness bee-housing conditions. An unknown
                // restricted-world condition cannot admit an observed successful mutation.
                ensure!(sample.amount=="1"&&mutation_result(root,input,second,sample_item,&p.config,waste_registry,|_|Ok(false))?,"Native mutation observation is outside selected genome/mortality branches");},Operation::Extract=>anyhow::bail!("Extractor emits fluid, not an item edit"),
        }
    }Ok(())
}

#[cfg(test)]
mod mutation_tests {
    use super::*;
    use serde_json::json;

    fn genome(species:&str,secondary:&str)->Nbt {
        let chromosome=Nbt::Compound{value:BTreeMap::from([
            ("Slot".into(),Nbt::Byte{value:"0".into()}),
            ("UID0".into(),Nbt::String{value:species.into()}),
            ("UID1".into(),Nbt::String{value:secondary.into()}),
        ])};
        Nbt::Compound{value:BTreeMap::from([("Chromosomes".into(),Nbt::List{element:"compound".into(),value:vec![chromosome]})])}
    }
    fn item(registry:&str,nbt:Nbt)->Item {
        serde_json::from_value(json!({"id":"test_item","registry":registry,"meta":0,"nbt":nbt,"name":"test","tooltip":[],"stackLimit":1,"durability":0,"tools":{},"armor":false,"tags":[],"icon":null,"order":null,"aspects":null})).unwrap()
    }
    fn fixture()->(Root,Item,Item,Item) {
        let root=Root{uid:"rootBees".into(),native_class:"fixture.BeeRoot".into(),kind:RootKind::Bee,species_chromosome:0,lifespan:None,
            forms:vec![MemberForm{ordinal:0,registry:"fixture:drone".into(),meta:0,nbt:None},MemberForm{ordinal:1,registry:"fixture:princess".into(),meta:0,nbt:None},MemberForm{ordinal:2,registry:"fixture:queen".into(),meta:0,nbt:None}],
            chromosomes:vec![Chromosome{ordinal:0,name:"Species".into()}],required:vec![],default_template:vec![None],templates:vec![],
            mutations:vec![Mutation{native_class:"fixture.Mutation".into(),parent0:"parent.a".into(),parent1:"parent.b".into(),template:vec![Some("result.valid".into())],secret:false,conditions:vec![],setting:MutationSetting::Enabled,bee_mutation:true},
                Mutation{native_class:"fixture.OtherMutation".into(),parent0:"parent.c".into(),parent1:"parent.d".into(),template:vec![Some("result.other".into())],secret:false,conditions:vec![],setting:MutationSetting::Enabled,bee_mutation:true}]};
        let parent0=item("fixture:princess",Nbt::Compound{value:BTreeMap::from([("Genome".into(),genome("parent.a","parent.a"))])});
        let parent1=item("fixture:drone",Nbt::Compound{value:BTreeMap::from([("Genome".into(),genome("parent.b","parent.b"))])});
        let result=item("fixture:queen",Nbt::Compound{value:BTreeMap::from([("Genome".into(),genome("result.valid","result.valid")),("Mate".into(),genome("result.valid","result.valid"))])});
        (root,parent0,parent1,result)
    }
    fn config()->Config {
        serde_json::from_value(json!({"mjPerItem":100,"powerUseRate":10,"activationEnergy":0,"maxStoredEnergy":100,"maxReceivedEnergy":10,
            "secretChance":0,"degradeChanceNatural":0,"deathChanceArtificial":0})).unwrap()
    }
    #[test]
    fn tree_mutation_parent_is_native_pollen_not_blossom() {
        let (mut root,_,_,_)=fixture();root.kind=RootKind::Tree;
        root.forms=vec![MemberForm{ordinal:1,registry:"fixture:blossom".into(),meta:0,nbt:None},
            MemberForm{ordinal:2,registry:"fixture:pollen".into(),meta:0,nbt:None}];
        let pollen=item("fixture:pollen",Nbt::Compound{value:BTreeMap::from([("Genome".into(),genome("parent.b","parent.b"))])});
        assert_eq!(parent_species(&root,&pollen,1).unwrap(),Some("parent.b".into()));
        let mut wrong=pollen;wrong.registry="fixture:blossom".into();
        assert_eq!(parent_species(&root,&wrong,1).unwrap(),None);
    }
    fn observed(root:&Root,parent0:&Item,parent1:&Item,result:&Item,config:&Config)->bool {
        mutation_result(root,parent0,parent1,result,config,"fixture:waste",|_|Ok(false)).unwrap()
    }
    #[test]
    fn mutatron_observation_must_use_new_genome_lifespan() {
        let (mut root,parent0,parent1,mut result)=fixture();let cfg=config();
        root.lifespan=Some(Lifespan{chromosome:0,values:BTreeMap::from([("result.valid".into(),47)])});
        if let Some(Nbt::Compound{value:tags})=&mut result.nbt {
            tags.insert("Health".into(),Nbt::Int{value:"47".into()});tags.insert("MaxH".into(),Nbt::Int{value:"47".into()});
        }
        assert!(observed(&root,&parent0,&parent1,&result,&cfg));
        for key in ["Health","MaxH"] {
            let mut changed=result.clone();if let Some(Nbt::Compound{value:tags})=&mut changed.nbt{tags.insert(key.into(),Nbt::Int{value:"10".into()});}
            assert!(!observed(&root,&parent0,&parent1,&changed,&cfg),"Default form health leaked into mutated genome");
        }
    }
    #[test]
    fn sampler_observation_cannot_move_gene_tags_to_another_item() {
        let (root,mut individual,_,_)=fixture();individual.id="individual".into();
        let nbt=sample_outcomes(&root,&individual).unwrap()[0].clone();
        let mut sample=item("gendustry:GeneSample",nbt);sample.id="sample".into();
        let mut raw:serde_json::Value=serde_json::from_str(include_str!("../../../../contracts/fixtures/gendustry-replicator-0-contract.json")).unwrap();
        let p=&mut raw["recipe"]["process"];p["operation"]=json!("sample");p["roots"]=serde_json::to_value(vec![root]).unwrap();p["slots"]=json!({"individual":0});p["outputPolicy"]=json!({"kind":"sampler","sampleRegistry":"gendustry:GeneSample","sampleMeta":0});
        raw["recipe"]["inputs"]=json!([{"slot":0,"kind":"item","choices":[{"id":"individual","amount":"1","consume":{"kind":"consume"},"returns":[],"rule":{"kind":"wildcard","meta":false,"nbt":true}}]}]);
        raw["recipe"]["outputs"]=json!([{"slot":0,"kind":"item","id":"sample","amount":"1","quantity":null,"chance":{"numerator":"1","denominator":"1"},"role":"result",
            "change":{"input":0,"action":{"kind":"gendustryGeneSample","operation":"sample","slots":{"individual":0}},"samples":[{"id":"sample","amount":"1"}],"bindings":[[0]]}}]);
        let recipe:Recipe=serde_json::from_value(raw["recipe"].clone()).unwrap();let slots=BTreeMap::from([("individual".into(),0)]);
        let check=|s:&Item|validate_change(&recipe,&recipe.outputs[0],&BTreeMap::from([("individual",&individual),("sample",s)]),Operation::Sample,&slots);
        check(&sample).unwrap();sample.registry="minecraft:paper".into();assert!(check(&sample).is_err(),"Sampler accepted arbitrary item with copied gene NBT");
    }
    #[test]
    fn mutatron_rejects_unrelated_pair_secondary_and_natural_state() {
        let (root,parent0,parent1,mut result)=fixture();let cfg=config();
        assert!(observed(&root,&parent0,&parent1,&result,&cfg));
        if let Some(Nbt::Compound{value:tags})=&mut result.nbt {tags.insert("Genome".into(),genome("result.other","result.other"));tags.insert("Mate".into(),genome("result.other","result.other"));}
        assert!(!observed(&root,&parent0,&parent1,&result,&cfg),"unrelated mutation accepted");
        if let Some(Nbt::Compound{value:tags})=&mut result.nbt {tags.insert("Genome".into(),genome("result.valid","wrong.secondary"));tags.insert("Mate".into(),genome("result.valid","result.valid"));}
        assert!(!observed(&root,&parent0,&parent1,&result,&cfg),"altered secondary genome accepted");
        if let Some(Nbt::Compound{value:tags})=&mut result.nbt {tags.insert("Genome".into(),genome("result.valid","result.valid"));tags.insert("NA".into(),Nbt::Byte{value:"0".into()});}
        assert!(!observed(&root,&parent0,&parent1,&result,&cfg),"wrong inherited natural state accepted");
        if let Some(Nbt::Compound{value:tags})=&mut result.nbt {tags.remove("NA");tags.insert("Mate".into(),genome("result.other","result.other"));}
        assert!(!observed(&root,&parent0,&parent1,&result,&cfg),"unrelated mating genome accepted");
        let (_,_,_,mut result)=fixture();result.registry="fixture:princess".into();
        assert!(!observed(&root,&parent0,&parent1,&result,&cfg),"unmated member form accepted");
        assert!(!observed(&root,&parent1,&parent0,&result,&cfg),"invalid parent forms accepted");
    }
    #[test]
    fn mutatron_overrides_require_world_evidence_and_preserve_null() {
        let (mut root,parent0,parent1,result)=fixture();let cfg=config();
        root.mutations[0].setting=MutationSetting::Disabled;
        assert!(!observed(&root,&parent0,&parent1,&result,&cfg));
        root.mutations[0].setting=MutationSetting::Requirements;
        assert!(!observed(&root,&parent0,&parent1,&result,&cfg),"unknown bee housing was treated as eligible");
        assert!(mutation_result(&root,&parent0,&parent1,&result,&cfg,"fixture:waste",|_|Ok(true)).unwrap());
        root.mutations[0].bee_mutation=false;
        assert!(observed(&root,&parent0,&parent1,&result,&cfg),"native non-bee requirements incorrectly invoked bee housing");
    }
    #[test]
    fn mutatron_rng_endpoints_control_secret_degradation_and_waste() {
        let (mut root,mut parent0,parent1,mut result)=fixture();let mut cfg=config();
        let mut hidden=root.mutations[0].clone();hidden.secret=true;hidden.template=vec![Some("result.hidden".into())];root.mutations.push(hidden);
        cfg.secret_chance=Some(100.0);
        assert!(!observed(&root,&parent0,&parent1,&result,&cfg),"ordinary result survived guaranteed secret branch");
        cfg.secret_chance=Some(0.0);cfg.degrade_chance_natural=Some(100.0);
        assert!(!observed(&root,&parent0,&parent1,&result,&cfg),"natural result survived guaranteed degradation");
        if let Some(Nbt::Compound{value:tags})=&mut result.nbt {tags.insert("NA".into(),Nbt::Byte{value:"0".into()});}
        assert!(observed(&root,&parent0,&parent1,&result,&cfg));
        if let Some(Nbt::Compound{value:tags})=&mut parent0.nbt {tags.insert("NA".into(),Nbt::Byte{value:"0".into()});}
        let mut waste=result.clone();waste.registry="fixture:waste".into();waste.nbt=None;
        assert!(!observed(&root,&parent0,&parent1,&waste,&cfg),"waste accepted with zero artificial death chance");
        cfg.death_chance_artificial=Some(100.0);
        assert!(observed(&root,&parent0,&parent1,&waste,&cfg));
        assert!(!observed(&root,&parent0,&parent1,&result,&cfg),"queen survived guaranteed artificial mortality");
        root.mutations.clear();
        assert!(!observed(&root,&parent0,&parent1,&waste,&cfg),"waste accepted when no mutation candidate exists");
    }
    #[test]
    fn imprinter_observations_respect_native_mortality_endpoints_and_drone_exemption() {
        let (root,mut individual,_,_)=fixture();individual.id="individual".into();
        let mut template=item("fixture:template",Nbt::Compound{value:BTreeMap::from([
            ("species".into(),Nbt::String{value:root.uid.clone()}),
            ("samples".into(),Nbt::List{element:"compound".into(),value:vec![]})])});template.id="template".into();
        let slots=BTreeMap::from([("individual".into(),0),("template".into(),1)]);
        for (drone,natural_bee,death,success,wasted) in [
            (false,true,0.0,true,false),(false,true,100.0,false,true),
            (false,false,0.0,true,false),(false,false,100.0,false,true),
            (false,true,0.1,true,true),(false,false,99.1,false,true),
            (true,true,100.0,true,false),(true,false,100.0,true,false),
        ] {
            individual.registry=if drone{"fixture:drone"}else{"fixture:princess"}.into();
            if let Some(Nbt::Compound{value:tags})=&mut individual.nbt {
                tags.remove("NA");if !natural_bee{tags.insert("NA".into(),Nbt::Byte{value:"0".into()});}
            }
            let mut result=individual.clone();result.id="result".into();result.nbt=Some(imprint(&root,&individual,&template).unwrap());
            let mut waste=result.clone();waste.registry="fixture:waste".into();waste.nbt=None;
            let mut cfg=config();cfg.death_chance_natural=Some(if natural_bee{death}else{100.0-death});
            cfg.death_chance_artificial=Some(if natural_bee{100.0-death}else{death});
            let process=GeneticsProcess{operation:Operation::Imprint,config:cfg,roots:vec![root.clone()],slots:slots.clone(),
                rng_bound:100,labware_threshold:None,selector:0,sources:None,
                output_policy:OutputPolicy::Imprinter{waste_registry:"fixture:waste".into()}};
            let recipe:Recipe=serde_json::from_value(json!({"id":"recipe","source":{"owner":"gendustry","handler":"net.bdew.gendustry.nei.ImprinterHandler","key":"test"},
                "category":"category","duration":null,"energy":null,"grid":null,"magic":null,"order":0,"properties":{},"process":{"kind":"gendustryGenetics", "operation":process.operation,
                    "config":process.config,"roots":process.roots,"slots":process.slots,"rngBound":100,"labwareThreshold":null,"selector":0,"sources":null,"outputPolicy":process.output_policy},
                "inputs":[{"slot":0,"kind":"item","choices":[{"id":"individual","amount":"1","consume":{"kind":"consume"},"returns":[],"rule":{"kind":"wildcard","meta":false,"nbt":true}}]},
                    {"slot":1,"kind":"item","choices":[{"id":"template","amount":"1","consume":{"kind":"keep"},"returns":[],"rule":{"kind":"wildcard","meta":false,"nbt":true}}]}],
                "outputs":[{"slot":0,"kind":"item","id":"result","amount":"1","quantity":null,"role":"result","chance":{"numerator":"1","denominator":"1"},
                    "change":{"input":0,"action":{"kind":"gendustryMutation","operation":"imprint","slots":slots},"bindings":[[0,0]],"samples":[{"id":"result","amount":"1"}]}}]})).unwrap();
            for (observed,expected) in [(&result,success),(&waste,wasted)] {
                let items=BTreeMap::from([("individual",&individual),("template",&template),("result",observed)]);
                assert_eq!(validate_change(&recipe,&recipe.outputs[0],&items,Operation::Imprint,&slots).is_ok(),expected,
                    "drone={drone}, natural={natural_bee}, death={death}, waste={}",observed.registry=="fixture:waste");
            }
        }
    }
    #[test]
    fn replication_lifespan_uses_selected_gene_instead_of_default_form_health() {
        let (mut root,_,_,_)=fixture();
        root.default_template=vec![Some("result.valid".into()),None,Some("fixture.short".into())];
        root.lifespan=Some(Lifespan{chromosome:2,values:BTreeMap::from([("fixture.short".into(),10),("fixture.long".into(),47)])});
        for form in &mut root.forms {form.nbt=Some(Nbt::Compound{value:BTreeMap::from([
            ("Health".into(),Nbt::Int{value:"10".into()}),("MaxH".into(),Nbt::Int{value:"10".into()})])});}
        let template=item("fixture:template",Nbt::Compound{value:BTreeMap::from([
            ("species".into(),Nbt::String{value:root.uid.clone()}),
            ("samples".into(),Nbt::List{element:"compound".into(),value:vec![Nbt::Compound{value:BTreeMap::from([
                ("chromosome".into(),Nbt::Int{value:"2".into()}),("allele".into(),Nbt::String{value:"fixture.long".into()})])}]})])});
        for kind in [RootKind::Bee,RootKind::Butterfly] {
            root.kind=kind;let (result,_)=replicate(&root,&template,true).unwrap();
            let tags=compound(result.nbt.as_ref()).unwrap();
            assert_eq!(tags.get("Health"),Some(&Nbt::Int{value:"47".into()}),"default form health leaked into a different lifespan genome");
            assert_eq!(tags.get("MaxH"),Some(&Nbt::Int{value:"47".into()}));
        }
    }
    #[test]
    fn replicated_bee_is_analyzed_mated_queen_with_native_na_tag() {
        let (mut root,_,_,_)=fixture();root.required.clear();
        for form in &mut root.forms {form.nbt=Some(Nbt::Compound{value:BTreeMap::from([("Genome".into(),genome("result.valid","result.valid"))])});}
        root.default_template=vec![Some("result.valid".into())];
        let template=item("fixture:template",Nbt::Compound{value:BTreeMap::from([("species".into(),Nbt::String{value:root.uid.clone()})])});
        let (result,_)=replicate(&root,&template,false).unwrap();
        assert!(!natural(compound(result.nbt.as_ref()).unwrap()),"replicated artificial bee reads as natural in Forestry");
        assert_eq!(result.registry,"fixture:queen","replicator emitted an unmated princess");
        let tags=compound(result.nbt.as_ref()).unwrap();
        assert_eq!(integer(tags.get("IsAnalyzed")),1,"replicator lost native analyze()");
        assert_eq!(tags.get("Mate"),Some(&genome("result.valid","result.valid")),"replicated queen lost self-mating genome");
    }
}
