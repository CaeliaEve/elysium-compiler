//! Pinned Binnie 2.5.24 genetic processors and the separate, non-consuming Database domain.
use super::{Consumption, Fluid, Item, Kind, Match, Origin, OutputRole, Quantity, Recipe};
use crate::identity::Nbt;
use anyhow::{ensure, Context, Result};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

macro_rules! native_enum {
    ($name:ident, $($variant:ident => $value:literal),+ $(,)?) => {
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
        pub enum $name { $(#[serde(rename=$value)] $variant),+ }
    };
}
native_enum!(ToleranceKind, Temperature => "Temperature", Humidity => "Humidity", Ph => "PH");
native_enum!(AcclimatiserSelection, Native => "firstSystemThenUniformValidOccupiedSlot");
native_enum!(AcclimatiserAlteration, Native => "activeBoundsExpandClampFiveThenWidthProbabilityRandomHomolog");
native_enum!(AcclimatiserConsumption, Native => "firstItemEqualReagentEvenUnchanged");
native_enum!(ToleranceGate, Native => "nativeCanAlterActiveTolerance");
native_enum!(AnalysisGate, Native => "analysableAndUnanalysed");
native_enum!(AnalysisTransform, Individual => "nativeAnalyseRewrite", Sequence => "nativeSequenceAnalysedFlag");
native_enum!(Conversion, Native => "nativeBreedingSystemFirstConversion", None => "none");
native_enum!(SequenceStrength, Native => "oneMinusSquaredQualityTimesThreeQuarters");
native_enum!(SequenceResume, Native => "storedSequencedInteger");
native_enum!(SequenceProgress, Native => "truncatedProgressInteger");
native_enum!(SequenceEncoding, Native => "signedByteAfterIntegerTruncation");
native_enum!(Discovery, Native => "worldOwnerGeneTracker");
native_enum!(SequenceOutputGate, Native => "emptyVialCountBelow64");
native_enum!(TimeScale, Mismatch => "mismatchedGenesHalfIncrement", GeneCount => "geneCount");
native_enum!(SerumKind, Native => "nativeSingleOrArray");
native_enum!(ManipulationGate, Native => "nativeDnaManipulable");
native_enum!(SerumRootGate, Native => "firstSerumGeneRootMember");
native_enum!(DefunctGate, Native => "allGenesAlreadyBothHomologsUid");
native_enum!(ChargesGate, Nonzero => "nonzero", DamagePositive => "damagePositive");
native_enum!(GeneTransform, Native => "analyseThenSetBothHomologs");
native_enum!(BeeOffset, Native => "humidityAndLaterMinusOne");
native_enum!(MismatchComparison, Native => "alleleObjectIdentity");
native_enum!(Drain, Native => "floatAccumulatorOneMillibucketPerTick");
native_enum!(PersistentState, Native => "accumulatorsPersistBetweenCycles");
native_enum!(ChromosomeSelection, Native => "uniformNonNullNativeChromosome");
native_enum!(HomologSelection, Native => "uniformActiveInactive");
native_enum!(IsolationOutputGate, Native => "emptySequenceSlot");
native_enum!(InvalidGenome, Native => "noGeneNoConsumption");
native_enum!(DnaYield, Native => "beeDrone10OtherBee30MatedBee50Other10");
native_enum!(IncubatorCallback, Fixed => "fixed", Larvae => "larvaeToDrone");
native_enum!(IncubatorSelection, Native => "firstRegistryMatchCached");
native_enum!(IncubatorFluidGate, Native => "amountOnlyAfterSelection");
native_enum!(IncubatorOutputGate, Native => "nativeRoomCheck");
native_enum!(PolymeriserTimeScale, Native => "geneCountTimesCatalyst");
native_enum!(ReferenceKind, Native => "binnie_genetics_reference");
native_enum!(ContainerKind, Serum => "serum", Genome => "genome");
native_enum!(Acceptor, Native => "zeroCharges");
native_enum!(AcceptorRootGate, Single => "singleSerumAcceptsAnyRoot", Array => "sameRootForExistingArray");
native_enum!(GenePolicy, Single => "replaceSingleGene", Array => "replaceSameChromosomeAppendNew");
native_enum!(Tracker, Native => "GeneTracker.worldOwner");
native_enum!(ReferenceDiscovery, Native => "sequencerCompletion");

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all="camelCase", deny_unknown_fields)]
pub struct Allele { pub uid: String, pub class: String, pub compatible: bool, pub tolerance_bounds: Option<[i8;2]> }
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all="camelCase", deny_unknown_fields)]
pub struct Chromosome { pub ordinal: u8, pub name: String, pub allele_class: String, pub invalid: bool, pub alleles: Vec<Allele> }
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all="camelCase", deny_unknown_fields)]
pub struct Domain { pub root: String, pub root_class: String, pub system_class: String, pub chromosomes: Vec<Chromosome>, pub member_types: Vec<u8>, pub members: Vec<NativeMember>, pub conversions: Vec<NativeConversion>, pub resources: Resources }
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct NativeMember { pub r#type: u8, pub manipulable: bool, pub stack: NativeStack, pub analysed: NativeStack, pub dna_yield: u32 }
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct NativeConversion { pub input: NativeStack, pub registered: NativeStack, pub output: NativeStack, pub analysed: NativeStack }
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct Resources { pub serum: String, pub genome: String, pub sequence: String, pub dna_dye: NativeStack, pub fluorescent_dye: NativeStack, pub enzyme: NativeStack, pub empty_sequence: NativeStack, pub raw_dna: NativeFluid, pub polymer_bacteria: NativeFluid, pub vector_bacteria: NativeFluid, pub ethanol: NativeFluid }
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct NativeStack { pub item: String, pub meta: i32, pub amount: i32, pub nbt: Option<Nbt> }
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct NativeFluid { pub fluid: String, pub amount: i32, pub nbt: Option<Nbt> }
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Effect { pub item: NativeStack, pub effect: f32 }
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ToleranceSystem { pub root: String, pub chromosome: u8, pub r#type: ToleranceKind }
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all="camelCase", deny_unknown_fields)]
pub struct IncubatorRecipe { pub callback: IncubatorCallback, pub input: NativeStack, pub fluid_input: NativeFluid, pub fluid_output: Option<NativeFluid>, pub output: Option<NativeStack>, pub chance: f32, pub loss_chance: f32 }

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all="camelCase", deny_unknown_fields)]
pub struct Acclimatiser {
    pub domain: Domain, pub system_ordinal: u32, pub chromosome: u8, pub tolerance_type: ToleranceKind, pub effect: f32,
    pub systems: Vec<ToleranceSystem>, pub temperature: Vec<Effect>, pub humidity: Vec<Effect>,
    pub rf_per_tick: u32, pub attempt_interval: u32, pub selection: AcclimatiserSelection,
    pub alteration: AcclimatiserAlteration, pub consumption: AcclimatiserConsumption, pub input_gate: ToleranceGate,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all="camelCase", deny_unknown_fields)]
pub struct Analyser { pub domain: Domain, pub base_ticks: u32, pub base_rf: u32, pub dye_per_tick: f32, pub input_gate: AnalysisGate, pub transformation: AnalysisTransform, pub conversion: Conversion }
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all="camelCase", deny_unknown_fields)]
pub struct Sequencer { pub domain: Domain, pub base_ticks: u32, pub rf_per_tick: u32, pub grade: i32, pub owner_required: bool, pub strength: SequenceStrength,
    pub resume: SequenceResume, pub progress_write: SequenceProgress, pub progress_encoding: SequenceEncoding, pub discovery: Discovery, pub dye_charge_per_cycle: f32, pub output_gate: SequenceOutputGate }
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all="camelCase", deny_unknown_fields)]
pub struct Splicing {
    pub domain: Domain, pub base_ticks: u32, pub base_rf: u32, pub time_scale: TimeScale, pub serum: SerumKind, pub input_gate: ManipulationGate,
    pub root_gate: SerumRootGate, pub defunct_gate: DefunctGate, pub charges_gate: ChargesGate, pub serum_damage: u32, pub transformation: GeneTransform,
    pub bee_chromosome_offset: BeeOffset, pub allele_class_gate: bool, pub mismatch_comparison: MismatchComparison,
    #[serde(default, skip_serializing_if="Option::is_none")] pub bacteria: Option<u32>,
    #[serde(default, skip_serializing_if="Option::is_none")] pub tank_gate: Option<u32>,
    #[serde(default, skip_serializing_if="Option::is_none")] pub drain: Option<Drain>,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all="camelCase", deny_unknown_fields)]
pub struct Isolator { pub domain: Domain, pub base_ticks: u32, pub base_rf: u32, pub enzyme_charge: f32, pub ethanol: u32, pub consumption_threshold: f32,
    pub chromosome_selection: ChromosomeSelection, pub homolog_selection: HomologSelection, pub output_gate: IsolationOutputGate, pub invalid_genome: InvalidGenome }
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all="camelCase", deny_unknown_fields)]
pub struct Genepool { pub domain: Domain, pub base_ticks: u32, pub base_rf: u32, pub r#yield: DnaYield, pub ethanol_per_dna: f32, pub enzyme_charge: f32,
    pub delete_whole_input_stack: bool, pub tank_gate: u32, pub drain: Drain, pub state: PersistentState }
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all="camelCase", deny_unknown_fields)]
pub struct Incubator { pub native_index: u32, pub recipe: IncubatorRecipe, pub earlier: Vec<IncubatorRecipe>, pub rf_per_tick: u32, pub attempt_interval: u32,
    pub selection: IncubatorSelection, pub fluid_gate: IncubatorFluidGate, pub output_gate: IncubatorOutputGate }
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all="camelCase", deny_unknown_fields)]
pub struct Polymeriser { pub domain: Domain, pub base_ticks: u32, pub base_rf: u32, pub time_scale: PolymeriserTimeScale, pub charge_gate: ChargesGate,
    pub gold_optional: bool, pub gold_time_scale: f32, pub gold_charge: f32, pub dna_per_gene: u32, pub bacteria_per_gene: u32,
    pub tank_gate: u32, pub drain: Drain, pub charge_change: i32, pub state: PersistentState }

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(tag="machine", rename_all="lowercase")]
pub enum GeneticsProcess {
    Acclimatiser(Acclimatiser), Sequencer(Sequencer), Analyser(Analyser), Splicer(Splicing), Isolator(Isolator),
    Inoculator(Splicing), Genepool(Genepool), Incubator(Incubator), Polymeriser(Polymeriser),
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct GeneReference { pub root: String, pub chromosome: u8, pub allele: String, pub serum: String, pub genome: String, pub sequence: String }
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all="camelCase", deny_unknown_fields)]
pub struct Catalogue { pub root: String, pub root_class: String, pub system_class: String, pub chromosomes: Vec<Chromosome>, pub member_types: Vec<u8>, pub members: Vec<NativeMember>, pub conversions: Vec<NativeConversion>, pub resources: Resources, pub genes: Vec<GeneReference> }
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all="camelCase", deny_unknown_fields)]
pub struct ContainerReference { pub kind: ContainerKind, pub empty: String, pub item: String, pub acceptor: Acceptor, pub root_gate: AcceptorRootGate, pub gene_policy: GenePolicy }
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct GeneticsReference {
    #[serde(default, skip_serializing_if="Option::is_none")] pub id: Option<String>,
    pub kind: ReferenceKind, pub source: Origin, pub catalog: Vec<Catalogue>, pub containers: Vec<ContainerReference>,
    pub tracker: Tracker, pub discovery: ReferenceDiscovery, pub consuming: bool,
}

impl Domain {
    pub fn validate(&self) -> Result<()> {
        let pair=match self.root_class.as_str() {
            "forestry.apiculture.genetics.BeeHelper" => "binnie.core.genetics.BeeBreedingSystem",
            "forestry.arboriculture.genetics.TreeHelper" => "binnie.core.genetics.TreeBreedingSystem",
            "forestry.lepidopterology.genetics.ButterflyHelper" => "binnie.core.genetics.MothBreedingSystem",
            "binnie.botany.genetics.FlowerHelper" => "binnie.core.genetics.FlowerBreedingSystem",
            _ => anyhow::bail!("Unknown native Binnie member/genome callback"),
        };
        ensure!(!self.root.is_empty() && self.root.len()<=65535 && self.system_class==pair && !self.chromosomes.is_empty() && self.chromosomes.len()<=128,
            "Invalid Binnie native root/karyotype");
        ensure!(!self.member_types.is_empty() && self.member_types.len()<=128,"Missing native Binnie member types");
        let mut types=BTreeSet::new();for ordinal in &self.member_types {ensure!(types.insert(ordinal),"Duplicate native member type");}
        for (index,chromosome) in self.chromosomes.iter().enumerate() {
            ensure!(usize::from(chromosome.ordinal)==index && !chromosome.name.is_empty() && !chromosome.allele_class.is_empty()
                && chromosome.alleles.len()<=65536,"Invalid native chromosome ordinal, callback class or allele budget");
            let mut seen=BTreeSet::new();for allele in &chromosome.alleles {ensure!(!allele.uid.is_empty() && allele.uid.len()<=65535 && !allele.class.is_empty()&&seen.insert(allele.uid.as_str()),"Invalid or duplicate native allele identity");
                if let Some([low,high])=allele.tolerance_bounds{ensure!((-5..=0).contains(&low)&&(0..=5).contains(&high),"Native tolerance bounds outside five");}}
        }
        ensure!(!self.members.is_empty()&&self.members.len()<=65536&&self.conversions.len()<=65536,"Native member/conversion catalogue exceeds budget");
        for member in &self.members {ensure!(self.member_types.contains(&member.r#type)&&member.stack.amount==1&&[10,30,50].contains(&member.dna_yield),"Native member type/count/yield differs");
            ensure!(member.stack.item==member.analysed.item&&member.stack.meta==member.analysed.meta,"Native member analysis changed its item type");
            let original=member.stack.nbt.as_ref().context("Native member genotype missing")?;let analysed=member.analysed.nbt.as_ref().context("Native analysed genotype missing")?;
            ensure!(genome_nbt(original)?==genome_nbt(analysed)?&&byte(compound(analysed)?.get("IsAnalyzed"))?==1,"Native member analysis changed its genotype");
            for chromosome in &self.chromosomes{if let Ok((a,b))=homologs(original,chromosome.ordinal){for allele in [a,b]{ensure!(chromosome.alleles.iter().any(|g|g.compatible&&g.uid==allele)||chromosome.invalid,"Native member genotype uses an incompatible unregistered allele");}}}
        }
        ensure!(self.conversions.is_empty()||self.root_class=="binnie.botany.genetics.FlowerHelper","Unknown native root conversion callback");
        for conversion in &self.conversions {ensure!(conversion.input.amount>0&&conversion.output.amount==1&&conversion.analysed.amount==1,"Native conversion specimen count differs");
            let selected=self.conversions.iter().find(|c|c.input.item==conversion.input.item&&c.input.meta==conversion.input.meta).context("Native conversion selector absent")?;
            ensure!(conversion.output.item==selected.registered.item&&conversion.output.meta==selected.registered.meta&&conversion.output.nbt==selected.registered.nbt,"Native first-itemEqual conversion priority changed");
            ensure!(conversion.output.item==conversion.analysed.item&&conversion.output.meta==conversion.analysed.meta&&genome_nbt(conversion.output.nbt.as_ref().context("Conversion genome absent")?)?==genome_nbt(conversion.analysed.nbt.as_ref().context("Analysed conversion genome absent")?)?,"Native conversion analysis changed its genome");}
        ensure!(!self.resources.serum.is_empty()&&!self.resources.genome.is_empty()&&!self.resources.sequence.is_empty()
            &&self.resources.serum!=self.resources.genome&&self.resources.serum!=self.resources.sequence&&self.resources.genome!=self.resources.sequence,"Native genetic item resource identities invalid");
        for resource in [&self.resources.dna_dye,&self.resources.fluorescent_dye,&self.resources.enzyme,&self.resources.empty_sequence]{ensure!(!resource.item.is_empty()&&resource.amount==1,"Native genetic labware resource invalid");}
        for (resource,expected) in [(&self.resources.raw_dna,"binnie.dna.raw"),(&self.resources.polymer_bacteria,"binnie.bacteriapoly"),(&self.resources.vector_bacteria,"binnie.bacteriavector"),(&self.resources.ethanol,"bioethanol")]{ensure!(resource.fluid==expected&&resource.amount==1,"Native genetic fluid resource identity differs");}
        Ok(())
    }
}
impl GeneticsReference {
    pub fn validate(&self,items:&BTreeMap<&str,&Item>) -> Result<()> {
        ensure!(!self.consuming && self.source.owner=="Genetics" && self.source.handler=="binnie.genetics.nei.DatabaseRecipeHandler"
            && self.catalog.len()<=65536 && self.containers.len()==2,"Database must be the native read-only reference domain");
        let mut roots=BTreeSet::new();
        for catalog in &self.catalog {
            let domain=Domain{root:catalog.root.clone(),root_class:catalog.root_class.clone(),system_class:catalog.system_class.clone(),chromosomes:catalog.chromosomes.clone(),member_types:catalog.member_types.clone(),members:catalog.members.clone(),conversions:catalog.conversions.clone(),resources:catalog.resources.clone()};
            domain.validate()?;ensure!(roots.insert(catalog.root.as_str()) && catalog.genes.len()<=65536,"Duplicate or oversized native gene catalogue");
            for gene in &catalog.genes {
                ensure!(gene.root==catalog.root && usize::from(gene.chromosome)<catalog.chromosomes.len(),"Reference gene root/chromosome is invalid");
                let chromosome=&catalog.chromosomes[usize::from(gene.chromosome)];
                ensure!(chromosome.alleles.iter().any(|a|a.compatible&&a.uid==gene.allele),"Gene is absent from native chromosome compatibility catalogue");
                for (id,registry) in [(&gene.serum,&domain.resources.serum),(&gene.genome,&domain.resources.genome),(&gene.sequence,&domain.resources.sequence)] {
                    let fact=items.get(id.as_str()).context("Missing native reference gene fact")?;
                    ensure!(&fact.registry==registry,"Native reference container item changed");
                    let decoded=genes(fact,&domain)?;ensure!(decoded.len()==1&&decoded[0].chromosome==gene.chromosome&&decoded[0].allele==gene.allele,"Reference gene NBT differs from catalogue");
                }
            }
        }
        for container in &self.containers {ensure!(items.contains_key(container.empty.as_str())&&!container.item.is_empty(),"Missing native empty serum/genome fact");
            let empty=items[container.empty.as_str()];let resources=&self.catalog.first().context("Native gene catalogue absent")?.resources;
            let (meta,registry)=if container.kind==ContainerKind::Serum{(6,&resources.serum)}else{(7,&resources.genome)};
            ensure!(empty.registry==resources.empty_sequence.item&&empty.meta==meta&&empty.nbt.is_none()&&container.item==*registry,"Native Database empty labware/container item differs");
            ensure!(matches!((container.kind,container.root_gate,container.gene_policy),
                (ContainerKind::Serum,AcceptorRootGate::Single,GenePolicy::Single)|(ContainerKind::Genome,AcceptorRootGate::Array,GenePolicy::Array)),"Native serum/root replacement policy changed");}
        ensure!(self.containers.iter().any(|c|c.kind==ContainerKind::Serum)&&self.containers.iter().any(|c|c.kind==ContainerKind::Genome),"Native Database container forms omitted");
        Ok(())
    }
}
impl GeneticsProcess {
    pub fn machine(&self)->&'static str {match self {Self::Acclimatiser(_)=>"Acclimatiser",Self::Sequencer(_)=>"Sequencer",Self::Analyser(_)=>"Analyser",Self::Splicer(_)=>"Splicer",Self::Isolator(_)=>"Isolator",Self::Inoculator(_)=>"Inoculator",Self::Genepool(_)=>"Genepool",Self::Incubator(_)=>"Incubator",Self::Polymeriser(_)=>"Polymeriser"}}
    pub fn domain(&self)->Option<&Domain>{Some(match self{Self::Acclimatiser(p)=>&p.domain,Self::Sequencer(p)=>&p.domain,Self::Analyser(p)=>&p.domain,Self::Splicer(p)|Self::Inoculator(p)=>&p.domain,Self::Isolator(p)=>&p.domain,Self::Genepool(p)=>&p.domain,Self::Polymeriser(p)=>&p.domain,Self::Incubator(_)=>return None})}
}

pub(super) fn validate_recipe(process:&GeneticsProcess,recipe:&Recipe,items:&BTreeMap<&str,&Item>,fluids:&BTreeMap<&str,&Fluid>)->Result<()> {
    ensure!(recipe.source.owner=="Genetics" && recipe.source.handler==format!("binnie.genetics.nei.{}RecipeHandler",process.machine())
        && recipe.duration.is_none()&&recipe.energy.is_none()&&recipe.grid.is_none()&&recipe.magic.is_none(),"Invalid Binnie native handler, owner or fixed timing");
    if let Some(domain)=process.domain(){domain.validate()?;}
    for input in &recipe.inputs {ensure!(input.choices.len()==1,"Native specimen must retain its single item/fluid identity");let c=&input.choices[0];
        ensure!(c.returns.is_empty(),"Genetic processor cannot invent a crafting container return");
        if input.kind==Kind::Item {ensure!(items.contains_key(c.id.as_str())&&c.amount=="1"&&matches!(c.rule,Match::Wildcard{nbt:true,..}),"Native genetics requires conditional NBT admission");}
        else {ensure!(fluids.contains_key(c.id.as_str())&&(matches!(c.rule,Match::Exact|Match::Wildcard{meta:false,nbt:true})
            ||matches!(process,GeneticsProcess::Incubator(p) if p.recipe.fluid_input.amount==0)
                &&matches!(c.rule,Match::NativeZeroFluid{policy:super::NativeZeroFluidPolicy::BinnieIncubator})),"Invalid native fluid admission");}
    }
    for output in &recipe.outputs {ensure!(matches!(output.role,OutputRole::Result)&&output.change.is_none()&&output.chance.numerator=="1"&&output.chance.denominator=="1","Genetic result must preserve its native conditional transformation");
        if output.kind==Kind::Item {ensure!(items.contains_key(output.id.as_str()),"Missing native genetics result specimen");}else{ensure!(fluids.contains_key(output.id.as_str()),"Missing native genetics fluid fact");}
        let dynamic=match process{GeneticsProcess::Sequencer(_)=>false,GeneticsProcess::Incubator(p)=>output.kind==Kind::Item&&p.recipe.callback==IncubatorCallback::Larvae,_=>true};
        if dynamic {ensure!(output.amount.is_none()&&matches!(&output.quantity,Some(Quantity::Potential{stat,nominal,condition:Some(condition),sample:Some(sample),parameters:None})
            if stat=="binnie:geneticTransform"&&condition=="nativeInputState"&&sample==nominal),"Native NBT/yield specimen became a fixed output");}
        else {ensure!(output.amount.is_some()&&output.quantity.is_none(),"Native fixed completion/task product was lost");}
    }
    match process {
        GeneticsProcess::Acclimatiser(p)=>{
            ensure!(p.rf_per_tick==2&&p.attempt_interval==100&&p.effect.is_finite()&&p.effect!=0.0&&p.tolerance_type!=ToleranceKind::Ph,"Invalid acclimatiser attempt/effect");
            let system=p.systems.get(p.system_ordinal as usize).context("Missing first-order native tolerance system")?;
            ensure!(system.root==p.domain.root&&system.chromosome==p.chromosome&&system.r#type==p.tolerance_type&&usize::from(p.chromosome)<p.domain.chromosomes.len(),"Acclimatiser system/root/chromosome mismatch");
            for effect in p.temperature.iter().chain(&p.humidity){ensure!(effect.effect.is_finite()&&!effect.item.item.is_empty(),"Invalid native reagent probability");}
            layout(recipe,[(Kind::Item,4,"staged"),(Kind::Item,0,"staged")],1)?;
        }
        GeneticsProcess::Analyser(p)=>{ensure!(p.base_ticks==300&&p.base_rf==9000&&p.dye_per_tick==0.002,"Invalid native analysis work/dye");layout(recipe,[(Kind::Item,6,"staged"),(Kind::Item,13,"buffer")],1)?;}
        GeneticsProcess::Sequencer(p)=>{ensure!(p.base_ticks==19200&&p.rf_per_tick==20&&(0..6).contains(&p.grade)&&p.owner_required&&p.dye_charge_per_cycle==0.4,"Invalid native sequencing grade, owner or dye gate");layout(recipe,[(Kind::Item,5,"staged"),(Kind::Item,0,"buffer")],1)?;}
        GeneticsProcess::Splicer(p)|GeneticsProcess::Inoculator(p)=>{
            let inoculator=matches!(process,GeneticsProcess::Inoculator(_));
            ensure!(p.base_ticks==if inoculator{12000}else{1200}&&p.base_rf==if inoculator{600000}else{12000000}
                &&p.time_scale==if inoculator{TimeScale::GeneCount}else{TimeScale::Mismatch}&&p.charges_gate==ChargesGate::Nonzero&&p.serum_damage==1&&p.allele_class_gate,"Invalid native splicing/inoculation work or gene gate");
            ensure!((p.bacteria,p.tank_gate,p.drain.is_some())==if inoculator{(Some(15),Some(1),true)}else{(None,None,false)},"Splicing vector/tank drain changed");
            layout(recipe,[(Kind::Item,9,"staged"),(Kind::Item,0,"wear")],1)?;
            ensure!(recipe.inputs.len()==if inoculator{3}else{2},"Unexpected native splicing resources");
            if inoculator{input(recipe,Kind::Fluid,0,"buffer")?;}
        }
        GeneticsProcess::Isolator(p)=>{ensure!(p.base_ticks==4800&&p.base_rf==192000&&p.enzyme_charge==0.5&&p.ethanol==10&&p.consumption_threshold.is_finite(),"Invalid native isolator work, enzyme, ethanol or consumption threshold");layout(recipe,[(Kind::Item,5,"staged"),(Kind::Item,1,"consume"),(Kind::Item,0,"buffer"),(Kind::Fluid,0,"staged")],1)?;}
        GeneticsProcess::Genepool(p)=>{ensure!(p.base_ticks==400&&p.base_rf==8000&&p.ethanol_per_dna==1.2&&p.enzyme_charge==0.25&&p.delete_whole_input_stack&&p.tank_gate==1,"Invalid native genepool yield/consumption");layout(recipe,[(Kind::Item,0,"staged"),(Kind::Item,7,"buffer"),(Kind::Fluid,1,"buffer")],1)?;}
        GeneticsProcess::Incubator(p)=>{
            ensure!(p.rf_per_tick==2&&p.attempt_interval==20&&p.earlier.len()==p.native_index as usize&&p.earlier.len()<=65536,"Invalid native incubator priority/attempt schedule");
            for native in p.earlier.iter().chain(std::iter::once(&p.recipe)){ensure!(native.chance.is_finite()&&native.loss_chance.is_finite()&&!native.input.item.is_empty()&&!native.fluid_input.fluid.is_empty()&&native.fluid_input.amount>=0,"Invalid native incubator callback/selector");}
            ensure!(!p.earlier.iter().any(|earlier|covers(earlier,&p.recipe)),"Shadowed native incubator recipe must not be consuming");
            let outputs=usize::from(p.recipe.output.is_some())+usize::from(p.recipe.fluid_output.is_some());
            layout(recipe,[(Kind::Item,3,"staged"),(Kind::Fluid,0,if p.recipe.fluid_input.amount==0{"keep"}else{"staged"})],outputs)?;
        }
        GeneticsProcess::Polymeriser(p)=>{ensure!(p.base_ticks==2400&&p.base_rf==96000&&p.charge_gate==ChargesGate::DamagePositive&&p.gold_optional&&p.gold_time_scale==0.2&&p.gold_charge==0.4&&p.dna_per_gene==50&&p.bacteria_per_gene==10&&p.tank_gate==1&&p.charge_change==1,"Invalid native polymerisation/catalyst/charge rules");layout(recipe,[(Kind::Item,0,"staged"),(Kind::Fluid,1,"buffer"),(Kind::Fluid,0,"buffer")],1)?;}
    }
    validate_specimen(process,recipe,items,fluids)?;
    Ok(())
}
fn input(recipe:&Recipe,kind:Kind,slot:u32,consume:&str)->Result<()> {
    let choice=recipe.inputs.iter().find(|i|i.kind==kind&&i.slot==slot).context("Missing native genetics material slot")?.choices.first().context("Missing native genetics material choice")?;
    let actual=match &choice.consume {Consumption::Staged=>"staged",Consumption::Buffer=>"buffer",Consumption::Wear=>"wear",Consumption::Consume=>"consume",Consumption::Keep=>"keep",_=>"unsupported"};
    ensure!(actual==consume,"Native genetics charge/retained/consumed resource policy changed");Ok(())
}
fn layout<const N:usize>(recipe:&Recipe,inputs:[(Kind,u32,&str);N],outputs:usize)->Result<()> {for (kind,slot,consume) in inputs {input(recipe,kind,slot,consume)?;}let extra=usize::from(recipe.source.handler.ends_with("InoculatorRecipeHandler"));ensure!(recipe.inputs.len()==N+extra&&recipe.outputs.len()==outputs,"Native genetics resource/product count changed");Ok(())}
fn covers(a:&IncubatorRecipe,b:&IncubatorRecipe)->bool{a.input.item==b.input.item&&(a.input.meta==32767||a.input.meta==b.input.meta)&&a.fluid_input.fluid==b.fluid_input.fluid&&a.fluid_input.nbt==b.fluid_input.nbt}

fn compound(value:&Nbt)->Result<&BTreeMap<String,Nbt>> { if let Nbt::Compound{value}=value {Ok(value)}else{anyhow::bail!("Native genetic NBT must be a compound")} }
fn tag(item:&Item)->Result<&BTreeMap<String,Nbt>> { compound(item.nbt.as_ref().context("Native genetic item lost its NBT")?) }
fn text(value:Option<&Nbt>)->Result<&str> {if let Some(Nbt::String{value})=value{Ok(value)}else{anyhow::bail!("Native genetic UID must be a string")}}
fn byte(value:Option<&Nbt>)->Result<i8>{if let Some(Nbt::Byte{value})=value{Ok(value.parse()?)}else{anyhow::bail!("Native genetic ordinal/flag/progress must be a byte")}}
#[derive(Clone,Debug,PartialEq,Eq)]
struct GeneData { chromosome:u8, allele:String }
fn gene(value:&Nbt,domain:&Domain)->Result<GeneData>{let fields=compound(value)?;let chromosome=byte(fields.get("chromo"))?;
    ensure!(text(fields.get("root"))?==domain.root&&chromosome>=0,"Native gene root or chromosome differs");
    let allele=text(fields.get("allele"))?.to_owned();let native=domain.chromosomes.get(chromosome as usize).context("Native gene chromosome absent")?;
    ensure!(native.alleles.iter().any(|a|a.compatible&&a.uid==allele),"Gene allele is not native chromosome-compatible");
    Ok(GeneData{chromosome:chromosome as u8,allele})}
fn genes(item:&Item,domain:&Domain)->Result<Vec<GeneData>>{let fields=tag(item)?;let resources=&domain.resources;
    if item.registry==resources.genome {let Some(Nbt::List{element,value})=fields.get("genes")else{anyhow::bail!("Native serum array genes missing")};
        ensure!(element=="compound"&&!value.is_empty()&&value.len()<=domain.chromosomes.len(),"Native serum array shape invalid");
        let mut seen=BTreeSet::new();let mut result=Vec::new();for value in value{let parsed=gene(value,domain)?;ensure!(seen.insert(parsed.chromosome),"Duplicate native serum chromosome");result.push(parsed);}Ok(result)
    }else{ensure!(item.registry==resources.serum||item.registry==resources.sequence,"Unknown native gene container item");Ok(vec![gene(fields.get("gene").context("Native single gene missing")?,domain)?])}}
fn genome_nbt(nbt:&Nbt)->Result<&Nbt>{compound(nbt)?.get("Genome").context("Native genome missing")}
fn chromosomes(nbt:&Nbt)->Result<&Vec<Nbt>>{let Some(Nbt::List{element,value})=compound(genome_nbt(nbt)?)?.get("Chromosomes")else{anyhow::bail!("Native chromosome list missing")};ensure!(element=="compound"&&!value.is_empty()&&value.len()<=128,"Native chromosome list shape invalid");Ok(value)}
fn homologs(nbt:&Nbt,ordinal:u8)->Result<(&str,&str)>{let row=chromosomes(nbt)?.iter().find(|row|compound(row).ok().and_then(|v|byte(v.get("Slot")).ok())==Some(ordinal as i8)).context("Native chromosome slot absent")?;
    let fields=compound(row)?;Ok((text(fields.get("UID0"))?,text(fields.get("UID1"))?))}
fn item_at<'a>(recipe:&Recipe,slot:u32,items:&BTreeMap<&str,&'a Item>)->Result<&'a Item>{let input=recipe.inputs.iter().find(|i|i.kind==Kind::Item&&i.slot==slot).context("Native item slot absent")?;items.get(input.choices[0].id.as_str()).copied().context("Native input fact absent")}
fn result<'a>(recipe:&Recipe,items:&BTreeMap<&str,&'a Item>)->Result<&'a Item>{let output=recipe.outputs.iter().find(|o|o.kind==Kind::Item).context("Native item output absent")?;items.get(output.id.as_str()).copied().context("Native result fact absent")}
fn same_stack(item:&Item,stack:&NativeStack)->bool{item.registry==stack.item&&item.meta==stack.meta&&item.nbt==stack.nbt}
fn resource(recipe:&Recipe,slot:u32,stack:&NativeStack,items:&BTreeMap<&str,&Item>)->Result<()>{ensure!(same_stack(item_at(recipe,slot,items)?,stack),"Native dye/enzyme/labware identity differs");Ok(())}
fn fluid_resource(recipe:&Recipe,slot:u32,fluid:&NativeFluid,amount:u32,fluids:&BTreeMap<&str,&Fluid>)->Result<()>{let choice=&recipe.inputs.iter().find(|i|i.kind==Kind::Fluid&&i.slot==slot).context("Native fluid resource absent")?.choices[0];let fact=fluids.get(choice.id.as_str()).context("Native fluid fact absent")?;
    ensure!(fact.registry==fluid.fluid&&fact.nbt==fluid.nbt&&choice.amount==amount.to_string(),"Native genetic fluid identity/amount differs");
    ensure!(if amount==0&&recipe.source.handler=="binnie.genetics.nei.IncubatorRecipeHandler"{matches!(choice.rule,Match::NativeZeroFluid{policy:super::NativeZeroFluidPolicy::BinnieIncubator})}
        else if fluid.fluid=="bioethanol"&&!recipe.source.handler.ends_with("IncubatorRecipeHandler"){matches!(choice.rule,Match::Wildcard{meta:false,nbt:true})}else{matches!(choice.rule,Match::Exact)},"Native genetic fluid admission/NBT policy differs");Ok(())}
fn nominal(output:&super::Output,expected:u32)->Result<()>{ensure!(matches!(&output.quantity,Some(Quantity::Potential{nominal,sample:Some(sample),..})if nominal==&expected.to_string()&&sample==nominal),"Native genetic conditional quantity exceeds its attainable sample count");Ok(())}
fn native_member<'a>(item:&Item,domain:&'a Domain,manipulable:bool)->Result<&'a NativeMember>{domain.members.iter().find(|m|same_stack(item,&m.stack)&&(!manipulable||m.manipulable)).context("Genetic specimen does not replay a native admitted member")}
fn set_homologs(nbt:&mut Nbt,gene:&GeneData,domain:&Domain)->Result<()>{let index=if domain.root_class=="forestry.apiculture.genetics.BeeHelper" {
    let humidity=domain.chromosomes.iter().find(|c|c.name.eq_ignore_ascii_case("humidity")).map(|c|c.ordinal);
    if humidity.is_some_and(|h|gene.chromosome>=h){usize::from(gene.chromosome).checked_sub(1).context("Native bee chromosome offset underflow")?}else{usize::from(gene.chromosome)}
    }else{usize::from(gene.chromosome)};
    let Nbt::Compound{value}=nbt else{anyhow::bail!("Native transformed member NBT missing")};let Some(Nbt::Compound{value})=value.get_mut("Genome")else{anyhow::bail!("Native transformed genome missing")};
    let Some(Nbt::List{value,..})=value.get_mut("Chromosomes")else{anyhow::bail!("Native transformed chromosome list missing")};let row=value.get_mut(index).context("Native setGene index absent")?;
    let Nbt::Compound{value}=row else{anyhow::bail!("Native transformed chromosome compound missing")};for key in ["UID0","UID1"]{value.insert(key.to_owned(),Nbt::String{value:gene.allele.clone()});}Ok(())}

/// Replays the exported native specimens, while typed process policies retain admission for arbitrary inserted NBT.
fn validate_specimen(process:&GeneticsProcess,recipe:&Recipe,items:&BTreeMap<&str,&Item>,fluids:&BTreeMap<&str,&Fluid>)->Result<()> {
    if let GeneticsProcess::Incubator(p)=process {
        let input=item_at(recipe,3,items)?;ensure!(input.registry==p.recipe.input.item&&(p.recipe.input.meta==32767||input.meta==p.recipe.input.meta),"Incubator native input selector differs");
        let rule=&recipe.inputs.iter().find(|i|i.kind==Kind::Item&&i.slot==3).context("Incubator item selector absent")?.choices[0].rule;
        ensure!(matches!(rule,Match::Wildcard{meta,nbt:true}if *meta==(p.recipe.input.meta==32767)),"Incubator native wildcard metadata admission differs");
        fluid_resource(recipe,0,&p.recipe.fluid_input,p.recipe.fluid_input.amount as u32,fluids)?;
        for output in &recipe.outputs{if output.kind==Kind::Fluid{let native=p.recipe.fluid_output.as_ref().context("Invented incubator fluid product")?;let fact=fluids.get(output.id.as_str()).context("Missing incubator fluid fact")?;
            ensure!(fact.registry==native.fluid&&fact.nbt==native.nbt&&output.amount.as_deref()==Some(native.amount.to_string().as_str()),"Incubator fixed fluid product differs");
        }else{let native=p.recipe.output.as_ref().context("Invented incubator item product")?;let fact=items.get(output.id.as_str()).context("Missing incubator item fact")?;
            if p.recipe.callback==IncubatorCallback::Fixed{ensure!(same_stack(fact,native)&&output.amount.as_deref()==Some(native.amount.to_string().as_str()),"Incubator fixed item task product differs");}
            else{nominal(output,1)?;ensure!(genome_nbt(fact.nbt.as_ref().context("Larvae drone genome absent")?)?==genome_nbt(input.nbt.as_ref().context("Larvae input genome absent")?)?,"Native larvae conversion lost actual input genome");}}
        }return Ok(());
    }
    let domain=process.domain().context("Native genetics domain absent")?;let resources=&domain.resources;
    for output in &recipe.outputs{if !matches!(process,GeneticsProcess::Sequencer(_)|GeneticsProcess::Genepool(_)){nominal(output,1)?;}}
    match process {
        GeneticsProcess::Analyser(p)=>{resource(recipe,13,&resources.dna_dye,items)?;let input=item_at(recipe,6,items)?;let output=result(recipe,items)?;
            match p.transformation {
                AnalysisTransform::Sequence=>{ensure!(p.conversion==Conversion::None&&input.registry==resources.sequence&&input.registry==output.registry&&input.meta==output.meta,"Native sequence analysis input differs");genes(input,domain)?;
                    let mut expected=input.nbt.clone().context("Sequence NBT absent")?;let Nbt::Compound{value}= &mut expected else{anyhow::bail!("Sequence NBT compound absent")};ensure!(byte(value.get("ana"))?==0,"Analysed sequence cannot enter analyser");value.insert("ana".to_owned(),Nbt::Byte{value:"1".to_owned()});ensure!(output.nbt.as_ref()==Some(&expected),"Analysis lost native sequence gene or byte progress");}
                AnalysisTransform::Individual=>{ensure!(p.conversion==Conversion::Native,"Native member conversion policy omitted");let expected=if let Some(conversion)=domain.conversions.iter().find(|c|input.registry==c.input.item&&input.meta==c.input.meta){&conversion.analysed}else{&native_member(input,domain,false)?.analysed};ensure!(same_stack(output,expected),"Native analysed/conversion specimen differs from native callback");}
            }}
        GeneticsProcess::Splicer(_)|GeneticsProcess::Inoculator(_)=>{let input=item_at(recipe,9,items)?;let member=native_member(input,domain,true)?;let serum=item_at(recipe,0,items)?;let decoded=genes(serum,domain)?;
            ensure!(serum.registry==resources.serum||serum.registry==resources.genome,"Native manipulation requires serum or genome array");ensure!(serum.meta!=serum.durability,"Zero-charge serum cannot manipulate DNA");
            let source=input.nbt.as_ref().context("Native manipulation input genome absent")?;
            ensure!(decoded.iter().any(|g|homologs(source,g.chromosome).is_ok_and(|(a,b)|a!=g.allele||b!=g.allele)),"Native specimen serum is defunct");
            let mut expected=member.analysed.nbt.clone().context("Native analysed manipulation specimen absent")?;for gene in decoded{set_homologs(&mut expected,&gene,domain)?;}
            let output=result(recipe,items)?;ensure!(output.registry==input.registry&&output.meta==input.meta&&output.nbt.as_ref()==Some(&expected),"Native gene transform changed unrelated NBT or failed both homologs");
            if matches!(process,GeneticsProcess::Inoculator(_)){fluid_resource(recipe,0,&resources.vector_bacteria,1,fluids)?;}}
        GeneticsProcess::Isolator(_)=>{let input=item_at(recipe,5,items)?;native_member(input,domain,false)?;resource(recipe,1,&resources.empty_sequence,items)?;resource(recipe,0,&resources.enzyme,items)?;fluid_resource(recipe,0,&resources.ethanol,10,fluids)?;
            let output=result(recipe,items)?;ensure!(output.registry==resources.sequence,"Native isolator must produce a gene sequence");let decoded=genes(output,domain)?;let gene=&decoded[0];let (a,b)=homologs(input.nbt.as_ref().context("Isolation input genome absent")?,gene.chromosome)?;
            ensure!(a==gene.allele||b==gene.allele,"Isolator fabricated an allele absent from both input homologs");}
        GeneticsProcess::Sequencer(p)=>{let input=item_at(recipe,5,items)?;ensure!(input.registry==resources.sequence&&input.meta%6==p.grade,"Native sequence quality specimen differs");genes(input,domain)?;let fields=tag(input)?;byte(fields.get("seq"))?;byte(fields.get("ana"))?;
            resource(recipe,0,&resources.fluorescent_dye,items)?;ensure!(same_stack(result(recipe,items)?,&resources.empty_sequence)&&recipe.outputs[0].amount.as_deref()==Some("1"),"Sequencer completion must return one native empty vial");}
        GeneticsProcess::Polymeriser(_)=>{let input=item_at(recipe,0,items)?;genes(input,domain)?;ensure!(input.meta>0,"Native charged item already full");let output=result(recipe,items)?;
            ensure!(input.registry==output.registry&&input.nbt==output.nbt&&output.meta==input.meta-1,"Polymerisation must preserve genes/NBT and add exactly one charge");fluid_resource(recipe,1,&resources.raw_dna,1,fluids)?;fluid_resource(recipe,0,&resources.polymer_bacteria,1,fluids)?;}
        GeneticsProcess::Genepool(_)=>{let input=item_at(recipe,0,items)?;let member=native_member(input,domain,false)?;resource(recipe,7,&resources.enzyme,items)?;fluid_resource(recipe,1,&resources.ethanol,1,fluids)?;
            let output=&recipe.outputs[0];let fact=fluids.get(output.id.as_str()).context("Native DNA fluid fact absent")?;ensure!(output.kind==Kind::Fluid&&fact.registry==resources.raw_dna.fluid&&fact.nbt==resources.raw_dna.nbt&&[10,30,50].contains(&member.dna_yield),"Genepool native DNA fluid or yield differs");nominal(output,member.dna_yield)?;}
        GeneticsProcess::Acclimatiser(p)=>{let input=item_at(recipe,4,items)?;native_member(input,domain,false)?;let reagent=item_at(recipe,0,items)?;
            let effects=if p.tolerance_type==ToleranceKind::Temperature{&p.temperature}else{&p.humidity};let effect=effects.iter().find(|e|e.item.item==reagent.registry&&e.item.meta==reagent.meta).context("Native acclimatiser reagent absent")?;ensure!(effect.effect==p.effect,"Native first reagent effect differs");
            ensure!(!p.systems.iter().take(p.system_ordinal as usize).any(|s|s.root==domain.root&&match s.r#type{ToleranceKind::Temperature=>p.temperature.iter().any(|e|e.item.item==reagent.registry&&e.item.meta==reagent.meta&&e.effect!=0.0),ToleranceKind::Humidity=>p.humidity.iter().any(|e|e.item.item==reagent.registry&&e.item.meta==reagent.meta&&e.effect!=0.0),ToleranceKind::Ph=>false}),"Earlier native tolerance system shadows this row");
            let (active,_)=homologs(input.nbt.as_ref().context("Acclimatiser genome absent")?,p.chromosome)?;let allele=domain.chromosomes[p.chromosome as usize].alleles.iter().find(|a|a.uid==active).context("Native active tolerance allele absent")?;
            let [low,high]=allele.tolerance_bounds.context("Native tolerance bounds absent")?;ensure!((p.effect>0.0&&high<5)||(p.effect<0.0&&low> -5),"Native tolerance specimen is saturated");
            // An unchanged specimen represents the native failed random alteration path; its quantity remains conditional.
            let output=result(recipe,items)?;ensure!(output.registry==input.registry&&output.meta==input.meta&&output.nbt==input.nbt,"Acclimatiser failed-attempt specimen changed its genome");}
        GeneticsProcess::Incubator(_)=>unreachable!(),
    }Ok(())
}

/// Native Java arithmetic: float multiplication and truncation, including resumed progress.
pub fn sequence_length(grade:i32)->i32{let quality=1.0-(grade%6) as f32/5.0;let strength=1.0-quality*quality*0.75;(19200.0*strength) as i32}
pub fn splice_length(mismatched_genes:u32)->u32{let mut scale=mismatched_genes as f32;if scale>1.0{scale=1.0+(scale-1.0)*0.5;}((1200.0*scale) as u32).max(1)}
pub fn polymerisation_length(genes:u32,gold_charge:f32)->u32{((2400u32.wrapping_mul(genes)) as f32*if gold_charge>0.0{0.2}else{1.0}) as u32}
pub fn dna_yield(bee:bool,drone:bool,mated:bool)->u32{if !bee||drone{10}else if mated{50}else{30}}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn native_timing_is_input_dependent(){assert_eq!(sequence_length(0),4800);assert_eq!(sequence_length(4),18624);assert_eq!(sequence_length(5),19200);assert_eq!(splice_length(0),1);assert_eq!(splice_length(1),1200);assert_eq!(splice_length(3),2400);assert_eq!(polymerisation_length(2,0.0),4800);assert_eq!(polymerisation_length(2,0.1),960);assert_eq!(dna_yield(false,false,false),10);assert_eq!(dna_yield(true,true,true),10);assert_eq!(dna_yield(true,false,false),30);assert_eq!(dna_yield(true,false,true),50);}
    #[test] fn all_native_callback_policies_are_closed_enums(){assert!(serde_json::from_str::<IncubatorCallback>("\"executeUnknownCallback\"").is_err());assert!(serde_json::from_str::<GeneTransform>("\"literalNeiExample\"").is_err());assert!(serde_json::from_str::<ChargesGate>("\"positiveOnly\"").is_err());}
}
