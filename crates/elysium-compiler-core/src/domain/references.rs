//! Native read-only Database catalogues, transported as bounded independently identified leaves.
use super::{binnie_genetics::{Allele, ContainerReference, GeneReference, GeneticsReference, ReferenceDiscovery, Tracker, ContainerKind, AcceptorRootGate, GenePolicy}, Item, Origin};
use crate::identity::Nbt;
use anyhow::{ensure, Context, Result};
use schemars::JsonSchema;
use serde::{Deserialize,Serialize};
use std::collections::{BTreeMap,BTreeSet};

#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize,JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ReferenceCounts {pub roots:u32,pub chromosomes:u32,pub alleles:u32,pub genes:u32,pub containers:u32}
#[derive(Clone,Debug,Serialize,Deserialize,JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ReferenceExplanation {pub key:String,pub text:String}
#[derive(Clone,Debug,Serialize,Deserialize,JsonSchema)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct ReferenceRoot {pub root:String,pub root_class:String,pub system_class:String,pub member_types:Vec<u8>,pub serum:String,pub genome:String,pub sequence:String}
#[derive(Clone,Debug,Serialize,Deserialize,JsonSchema)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct ReferenceChromosome {pub ordinal:u8,pub name:String,pub allele_class:String,pub invalid:bool}
#[derive(Clone,Debug,Serialize,Deserialize,JsonSchema)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct InteractiveGroupReference {pub source:Origin,pub group:String,pub page:u32,pub width:u32,pub height:u32,pub diagrams_per_page:u32,pub consuming:bool}
#[derive(Clone,Debug,Serialize,Deserialize,JsonSchema)]
#[serde(tag="kind",rename_all="camelCase",deny_unknown_fields)]
pub enum ReferenceData {
    CropsNhDefinition(super::cropsnh::CropsNhDefinition),
    CropsNhCrop(super::cropsnh::CropsNhCrop),
    CropsNhAlternateSeed(super::cropsnh::CropsNhAlternateSeed),
    CropsNhMutation(super::cropsnh::CropsNhMutation),
    CropsNhMutationPool(super::cropsnh::CropsNhMutationPool),
    UsageDefinition(super::usage_queries::UsageDefinition),
    UsageObservation(super::usage_queries::UsageObservation),
    UsageDocument(super::usage_queries::UsageDocument),
    UsageResource(super::usage_queries::UsageResource),
    ProjectBlueDefinition(super::projectblue::Definition),
    ProjectBlueRule(super::projectblue::Rule),
    ProjectBlueItem(super::projectblue::ItemRoles),
    ProjectBlueCachedMaterial(super::projectblue::CachedMaterial),
    ProjectBlueRegisteredMaterial(super::projectblue::RegisteredMaterial),
    ProjectBlueMaterialAlias(super::projectblue::Alias),
    ProjectBlueExample(super::projectblue::Example),
    LatheDefinition(super::lathe::Definition),
    LatheSample(super::lathe::Sample),
    LatheWorkpiece(super::lathe::Workpiece),
    LatheState(super::lathe::State),
    LatheTool(super::lathe::Tool),
    LatheToolState(super::lathe::ToolState),
    FireworkDefinition(super::fireworks::Definition),
    FireworkExample(super::fireworks::Example),
    FurnaceFuelDefinition(super::furnace_fuels::Definition),
    FurnaceFuel(super::furnace_fuels::Fuel),
    StructureCatalogDefinition(super::structure_references::Definition),
    StructureCatalogController(super::structure_references::Controller),
    StructureComponentLookup(super::structure_references::StructureComponentLookup),
    QuestsDefinition(super::betterquesting::Definition),
    QuestDeclaration(super::betterquesting::Declaration),
    BartworksOresDefinition(super::bartworks_ores::Definition),
    BartworksOreLayer(super::bartworks_ores::Layer),
    BartworksOreMaterial(super::bartworks_ores::Material),
    BartworksOreLookup(super::bartworks_ores::Lookup),
    GtMineralsDefinition(super::gt_minerals::Definition),
    GtMineralEntry(super::gt_minerals::Entry),
    GtMineralDimensionShare(super::gt_minerals::DimensionShare),
    UndergroundFluidsDefinition(super::underground_fluids::Definition),
    UndergroundFluidDimension(super::underground_fluids::Dimension),
    UndergroundFluidEntry(super::underground_fluids::Entry),
    UndergroundFluidDisplay(super::underground_fluids::Display),
    BeeTreeDefinition(super::bee_tree::Definition),
    BeeTreeSpecies(super::bee_tree::Species),
    BeeTreeMutation(super::bee_tree::Mutation),
    ForestryAddonsDefinition(super::forestry_addons::Definition),
    ForestryAddonsSpecies(super::forestry_addons::Species),
    ForestryAddonsOutput(super::forestry_addons::Output),
    ForestryAddonsMutation(super::forestry_addons::Mutation),
    DraconicReactorDefinition(super::draconic_reactor::Definition),
    InfernalDropsDefinition(super::infernal_drops::Definition),
    InfernalTier(super::infernal_drops::TierDefinition),
    InfernalPoolItem(super::infernal_drops::PoolItem),
    #[serde(rename_all="camelCase")]
    MobDropsDefinition {source:Origin,mobs:u32,drops:u32,hidden_mode:bool,ready:bool,consuming:bool},
    MobDefinition(super::mob_drops::MobDefinition),
    MobDrop(super::mob_drops::MobDrop),
    LexiconDefinition {source:Origin,entries:u32,pages:u32,consuming:bool},
    LexiconEntry {entry:String,name:super::lexicon::LexiconText,knowledge:super::lexicon::LexiconKnowledge,tagline:Option<super::lexicon::LexiconText>,items:Vec<super::lexicon::LexiconItem>},
    ItemAspectsDefinition {source:Origin,scans:u32,visibility:super::aspect_combinations::Visibility,ready:bool,consuming:bool},
    ItemAspectObservation {scan:String,status:super::item_aspects::ScanStatus,item:Option<String>,aspects:Option<Vec<super::AspectAmount>>},
    AspectCombinationsDefinition {source:Origin,aspects:u32,visibility:super::aspect_combinations::Visibility,consuming:bool},
    AspectCombination {aspect:String,components:Vec<String>,visible:bool},
    VillagerProfession {source:Origin,profession:i32,name:String,#[serde(rename="mod")] mod_name:String,offers:u32},
    #[serde(rename_all="camelCase")]
    VillagerOffer {source:Origin,profession:i32,offer:u32,native_frequency:String,inputs:Vec<super::villager_trades::TradeItem>,output:super::villager_trades::TradeItem},
    Definition {source:Origin,explanation:ReferenceExplanation,containers:Vec<ContainerReference>,tracker:Tracker,discovery:ReferenceDiscovery,consuming:bool,counts:ReferenceCounts},
    Root {definition:ReferenceRoot},
    Chromosome {root:String,chromosome:ReferenceChromosome},
    Allele {root:String,chromosome:u8,allele:Allele},
    Gene {gene:GeneReference},
    #[serde(rename_all="camelCase")]
    ToolMaterialsDefinition {source:Origin,version:String,craft_metal_tools:bool,materials:u32,parts:u32,native_pages:u32,consuming:bool},
    ToolMaterial {material:super::tool_materials::ToolMaterial},
    #[serde(rename_all="camelCase")]
    ToolMaterialPart {material_id:i32,index:u32,template:super::OreTemplate,display:Option<String>,
        #[serde(default,skip_serializing_if="Option::is_none")] display_binding:Option<super::tool_materials::PartDisplayBinding>},
    ForgeRegistryDefinition(super::forge_registry::Definition),
    #[serde(rename="oreGroup")]
    ForgeOreGroup(super::forge_registry::OreGroup),
    #[serde(rename="fluid")]
    ForgeFluid(super::forge_registry::FluidEntry),
    NeiInformationDefinition(super::nei_information::Definition),
    NeiInformationPage(super::nei_information::Page),
    NeiDiagramDefinition(super::nei_diagrams::Definition),
    NeiDiagram(super::nei_diagrams::Diagram),
    NeiDiagramBinding(super::nei_diagrams::Binding),
    NeiDiagramLookup(super::nei_diagrams::Lookup),
    NeiDiagramAction(super::nei_diagrams::Action),
    EnderStorageDefinition(super::ender_storage::Definition),
    EnderStorageState(super::ender_storage::State),
    #[serde(rename="neiInteractiveGroup")]
    NeiInteractiveGroup {source:Origin,group:String,page:u32,width:u32,height:u32,diagrams_per_page:u32,consuming:bool},
}
#[derive(Clone,Debug,Serialize,Deserialize,JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct GeneticReferenceRow {pub id:String,pub domain:String,pub order:u32,pub data:ReferenceData}

#[test]
fn native_usage_definitions_are_typed() {
    let valid=serde_json::json!({"kind":"usageDefinition","source":{"owner":"appliedenergistics2","handler":"appeng.integration.modules.NEIHelpers.NEICellViewHandler","key":"appeng.integration.modules.NEIHelpers.NEICellViewHandler"},
        "backend":"aeItemCell","nativeVersion":"rv3-beta-695-GTNH","locale":"en_US","scope":"catalogObservations","ready":true,"consuming":false,
        "counts":{"scanned":0,"queries":0,"matched":0,"unmatched":0,"duplicates":0,"documents":0,"resources":0}});
    serde_json::from_value::<ReferenceData>(valid.clone()).expect("Dedicated native usage definitions must be accepted");
    for field in ["backend","scope","extra"] {
        let mut changed=valid.clone();changed[field]=serde_json::json!("unverifiedGenericCallback");
        assert!(serde_json::from_value::<ReferenceData>(changed).is_err(),"Unknown native usage contract {field} accepted");
    }
}

#[test]
fn java_native_projectblue_reference_contract_is_accepted() {
    let raw:serde_json::Value=serde_json::from_str(include_str!("../../../../contracts/fixtures/native-projectblue-reference-contract.json")).unwrap();
    let rows:Vec<GeneticReferenceRow>=serde_json::from_value(raw["references"].clone()).unwrap();
    let base=crate::source::Source::open(&std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../contracts/fixtures/source-semantic-40")).unwrap();
    let baseline=super::Domain::load(&base).unwrap();
    let items:Vec<Item>=raw["items"].as_array().unwrap().iter().map(|raw|{let mut item=serde_json::to_value(&baseline.items[0]).unwrap();for(key,value)in raw.as_object().unwrap(){item[key]=value.clone();}serde_json::from_value(item).unwrap()}).collect();
    let item_map=items.iter().map(|item|(item.id.as_str(),item)).collect();
    validate(&rows,&item_map).unwrap();
    // Recompute content IDs to ensure these fail for semantics, not merely hashes.
    for mutation in 0..5 {
        let mut changed=rows.clone();
        if mutation==0 {if let ReferenceData::ProjectBlueDefinition(def)=&mut changed[0].data{def.consuming=true;}}
        if mutation==1 {for row in &mut changed{if let ReferenceData::ProjectBlueRule(rule)=&mut row.data{if rule.mode==super::projectblue::Mode::Panel{rule.required_empty.clear();break;}}}}
        if mutation==2 {for row in &mut changed{if let ReferenceData::ProjectBlueRule(rule)=&mut row.data{rule.ignored.clear();break;}}}
        if mutation==3 {for row in &mut changed{if let ReferenceData::ProjectBlueExample(example)=&mut row.data{example.output.amount="99".into();break;}}}
        if mutation==4 {for row in &mut changed{if let ReferenceData::ProjectBlueExample(example)=&mut row.data{if example.material_key.is_some(){example.material_key=Some("wrong-native-key".into());break;}}}}
        let data:Vec<_>=changed.iter().map(|row|&row.data).collect();let domain=super::value_id("reference",&serde_json::json!({"kind":"projectBlueRules","records":data})).unwrap();
        for row in &mut changed{row.domain=domain.clone();row.id=row.content_id().unwrap();}
        assert!(validate(&changed,&item_map).is_err(),"ProjectBlue semantic mutation {mutation} was accepted");
    }
}

#[test]
fn java_native_forge_registry_contract_is_accepted() {
    let raw:serde_json::Value=serde_json::from_str(include_str!("../../../../contracts/fixtures/native-forge-registry-contract.json")).unwrap();
    let base=crate::source::Source::open(&std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../contracts/fixtures/source-semantic-40")).unwrap();
    let baseline=super::Domain::load(&base).unwrap();
    let items:Vec<Item>=raw["items"].as_array().unwrap().iter().map(|raw|{let mut item=serde_json::to_value(&baseline.items[0]).unwrap();for(key,value)in raw.as_object().unwrap(){item[key]=value.clone();}serde_json::from_value(item).unwrap()}).collect();
    let fluids:Vec<super::Fluid>=raw["fluids"].as_array().unwrap().iter().map(|raw|{let mut item=serde_json::to_value(&baseline.fluids[0]).unwrap();for(key,value)in raw.as_object().unwrap(){item[key]=value.clone();}serde_json::from_value(item).unwrap()}).collect();
    let item_map=items.iter().map(|item|(item.id.as_str(),item)).collect();let fluid_map=fluids.iter().map(|fluid|(fluid.id.as_str(),fluid)).collect();
    let domains:Vec<Vec<GeneticReferenceRow>>=serde_json::from_value(raw["domains"].clone()).unwrap();
    for rows in &domains {validate(rows,&item_map).unwrap();super::forge_registry::validate_facts(rows,&fluid_map).unwrap();}
    let disabled:Vec<GeneticReferenceRow>=serde_json::from_value(raw["disabled"].clone()).unwrap();
    for row in disabled{validate(&[row],&item_map).unwrap();}
    fn seal(rows:&mut [GeneticReferenceRow]) {let data:Vec<_>=rows.iter().map(|row|&row.data).collect();let domain=super::value_id("reference",&serde_json::json!({"kind":"forgeRegistry","records":data})).unwrap();for row in rows{row.domain=domain.clone();row.id=row.content_id().unwrap();}}
    for mutation in 0..5 {
        let mut rows=domains[0].clone();
        if mutation==0{if let ReferenceData::ForgeRegistryDefinition(def)=&mut rows[0].data{def.consuming=true;}}
        if mutation==1{if let ReferenceData::ForgeRegistryDefinition(def)=&mut rows[0].data{def.enabled=false;}}
        if mutation==2{if let ReferenceData::ForgeRegistryDefinition(def)=&mut rows[0].data{def.source.handler="foreign.Handler".into();}}
        if mutation==3{for row in &mut rows{if let ReferenceData::ForgeOreGroup(group)=&mut row.data{if let Some(member)=group.members.first_mut(){member.registry="minecraft:foreign".into();break;}}}}
        if mutation==4{for row in &mut rows{if let ReferenceData::ForgeOreGroup(group)=&mut row.data{if group.items.len()>1{group.items[1]=group.items[0].clone();break;}}}}
        seal(&mut rows);assert!(validate(&rows,&item_map).is_err(),"Forge mutation {mutation} accepted");
    }
    let mut missing=domains[1].clone();if let ReferenceData::ForgeFluid(entry)=&mut missing[1].data{entry.containers[0].fluid.nbt=Some(crate::identity::Nbt::Compound{value:BTreeMap::new()});}
    seal(&mut missing);assert!(validate(&missing,&item_map).is_err());
    assert!(super::forge_registry::validate_facts(&domains[1],&BTreeMap::new()).is_err());
}

#[test]
fn native_enderstorage_transport_is_typed() {
    let raw:serde_json::Value=serde_json::from_str(include_str!("../../../../contracts/fixtures/enderstorage-contract.json")).unwrap();
    let domains:Vec<Vec<GeneticReferenceRow>>=serde_json::from_value(raw["domains"].clone()).unwrap();
    assert_eq!(domains.len(),2);assert_eq!(domains[0].len(),3);assert_eq!(domains[1].len(),2);
}

#[test]
fn java_native_structure_reference_contract_is_accepted() {
    let raw: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../contracts/fixtures/structure-reference-contract.json")
    ).unwrap()).unwrap();
    let _: Vec<GeneticReferenceRow> = serde_json::from_value(raw["records"].clone()).unwrap();
}

#[test]
fn java_aspect_reference_contract_is_accepted() {
    let raw: serde_json::Value = serde_json::from_str(include_str!("../../../../contracts/fixtures/aspect-combinations-contract.json")).unwrap();
    let rows: Vec<GeneticReferenceRow> = serde_json::from_value(raw["records"].clone()).unwrap();
    validate(&rows, &BTreeMap::new()).unwrap();
}

#[test]
fn java_item_aspect_contract_is_accepted() {
    let raw: serde_json::Value = serde_json::from_str(include_str!("../../../../contracts/fixtures/item-aspects-contract.json")).unwrap();
    let _: Vec<GeneticReferenceRow> = serde_json::from_value(raw["records"].clone()).unwrap();
}

#[test]
fn java_lexicon_contract_is_accepted() {
    let raw: serde_json::Value = serde_json::from_str(include_str!("../../../../contracts/fixtures/botania-lexicon-contract.json")).unwrap();
    let _: Vec<GeneticReferenceRow> = serde_json::from_value(raw["records"].clone()).unwrap();
}

#[test]
fn java_mob_drops_contract_is_accepted() {
    let raw: serde_json::Value = serde_json::from_str(include_str!("../../../../contracts/fixtures/mob-drops-contract.json")).unwrap();
    let _: Vec<GeneticReferenceRow> = serde_json::from_value(raw["records"].clone()).unwrap();
}

#[test]
fn java_infernal_pools_contract_is_accepted() {
    let raw: serde_json::Value = serde_json::from_str(include_str!("../../../../contracts/fixtures/infernal-drops-contract.json")).unwrap();
    for field in ["records", "disabled"] {
        let _: Vec<GeneticReferenceRow> = serde_json::from_value(raw[field].clone()).unwrap();
    }
}

#[test]
fn java_bee_tree_contract_is_accepted() {
    let raw: serde_json::Value = serde_json::from_str(include_str!("../../../../contracts/fixtures/bee-tree-contract.json")).unwrap();
    let _: Vec<GeneticReferenceRow> = serde_json::from_value(raw["records"].clone()).unwrap();
}

#[test]
fn java_forestry_addons_contract_is_accepted() {
    let raw:serde_json::Value=serde_json::from_slice(&std::fs::read(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../contracts/fixtures/forestry-addons-contract.json")
    ).unwrap()).unwrap();
    let _:Vec<Vec<GeneticReferenceRow>>=serde_json::from_value(raw["domains"].clone()).unwrap();
}

#[test]
fn java_underground_fluid_contract_is_accepted() {
    let raw: serde_json::Value = serde_json::from_str(include_str!("../../../../contracts/fixtures/underground-fluids-contract.json")).unwrap();
    let _: Vec<GeneticReferenceRow> = serde_json::from_value(raw["records"].clone()).unwrap();
}

#[test]
fn java_bartworks_ore_contract_is_accepted() {
    let raw: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../contracts/fixtures/bartworks-ores-contract.json")
    ).unwrap()).unwrap();
    let _: Vec<GeneticReferenceRow> = serde_json::from_value(raw["records"].clone()).unwrap();
}

#[test]
fn java_gt_mineral_contract_is_accepted() {
    let raw: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../contracts/fixtures/gt-minerals-contract.json")
    ).unwrap()).unwrap();
    for field in ["small","veins","disabled"] {
        let _: Vec<GeneticReferenceRow> = serde_json::from_value(raw[field].clone()).unwrap();
    }
}

#[test]
fn java_draconic_reactor_contract_is_accepted() {
    let raw: serde_json::Value = serde_json::from_str(include_str!("../../../../contracts/fixtures/draconic-reactor-contract.json")).unwrap();
    let _: Vec<GeneticReferenceRow> = serde_json::from_value(raw["records"].clone()).unwrap();
}

impl GeneticReferenceRow {pub fn content_id(&self)->Result<String>{super::content_id("refrow",self)}}

#[test]
fn cropsnh_pools_are_nonconsuming_memberships_with_native_seed_identity() {
    let source = crate::source::Source::open(&std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../contracts/fixtures/source-semantic-40")).unwrap();
    let mut seed = super::Domain::load(&source).unwrap().items.remove(0);
    seed.registry = "cropsnh:genericSeed".into();
    seed.meta = 0;
    seed.nbt = Some(Nbt::Compound { value: BTreeMap::from([
        ("crop".into(), Nbt::String { value: "cropsnh:wheat".into() }),
    ]) });
    let raw = serde_json::json!([
        {"kind":"cropsNhDefinition","source":{"owner":"cropsnh",
            "handler":"com.gtnewhorizon.cropsnh.compatibility.NEI.NEICropsNHMutationPoolHandler",
            "key":"cropsnh.mutationPools"},"nativeVersion":"2.0.140",
            "semantics":"cropsnhRegistriesV1","section":"mutationPools","entries":1,"consuming":false},
        {"kind":"cropsNhMutationPool","pool":"food","members":[
            {"crop":"cropsnh:wheat","numericId":1,"seed":seed.id}
        ]}
    ]);
    let data: Vec<ReferenceData> = serde_json::from_value(raw.clone())
        .expect("CropsNH registry leaves need a typed reference contract, not executable fake recipes");
    let seal = |data: Vec<ReferenceData>| {
        let domain = super::value_id("reference", &serde_json::json!({"kind":"cropsnh","records":data})).unwrap();
        data.into_iter().enumerate().map(|(order,data)| {
            let mut row = GeneticReferenceRow {id:String::new(),domain:domain.clone(),order:order as u32,data};
            row.id = row.content_id().unwrap(); row
        }).collect::<Vec<_>>()
    };
    let rows = seal(data);
    let facts = BTreeMap::from([(seed.id.as_str(), &seed)]);
    validate(&rows, &facts).unwrap();
    let mut consuming = raw.clone();
    consuming[0]["consuming"] = true.into();
    assert!(validate(&seal(serde_json::from_value(consuming).unwrap()), &facts).is_err());
    let mut probability = raw.clone();
    probability[1]["chance"] = 0.5.into();
    assert!(serde_json::from_value::<Vec<ReferenceData>>(probability).is_err());
    let mut foreign = raw;
    foreign[0]["source"]["owner"] = "gregtech".into();
    assert!(validate(&seal(serde_json::from_value(foreign).unwrap()), &facts).is_err());
    seed.nbt = None;
    assert!(validate(&rows, &BTreeMap::from([(seed.id.as_str(), &seed)])).is_err());
}

#[test]
fn native_diagrams_require_real_content_and_bindings() {
    let raw: serde_json::Value = serde_json::from_str(include_str!("../../../../contracts/fixtures/nei-diagram-contract.json")).unwrap();
    let rows: Vec<GeneticReferenceRow> = serde_json::from_value(raw["records"].clone()).unwrap();
    assert_eq!(rows.len(),11);
    assert_eq!(rows[0].id, rows[0].content_id().unwrap());
}

/// A deterministic normalization of the real native fixture; Source emits this same representation.
pub fn normalize(reference:&GeneticsReference)->Result<Vec<GeneticReferenceRow>> {
    normalize_with(reference,ReferenceExplanation{key:"genetics.nei.tip.databaseDesc".into(),text:"genetics.nei.tip.databaseDesc".into()})
}
pub fn normalize_with(reference:&GeneticsReference,explanation:ReferenceExplanation)->Result<Vec<GeneticReferenceRow>> {
    let counts=ReferenceCounts{roots:reference.catalog.len() as u32,chromosomes:reference.catalog.iter().map(|r|r.chromosomes.len() as u32).sum(),alleles:reference.catalog.iter().flat_map(|r|&r.chromosomes).map(|c|c.alleles.len() as u32).sum(),genes:reference.catalog.iter().map(|r|r.genes.len() as u32).sum(),containers:reference.containers.len() as u32};
    let mut data=vec![ReferenceData::Definition{source:reference.source.clone(),explanation,containers:reference.containers.clone(),tracker:reference.tracker,discovery:reference.discovery,consuming:reference.consuming,counts}];
    for root in &reference.catalog {
        data.push(ReferenceData::Root{definition:ReferenceRoot{root:root.root.clone(),root_class:root.root_class.clone(),system_class:root.system_class.clone(),member_types:root.member_types.clone(),serum:root.resources.serum.clone(),genome:root.resources.genome.clone(),sequence:root.resources.sequence.clone()}});
        for chromosome in &root.chromosomes {data.push(ReferenceData::Chromosome{root:root.root.clone(),chromosome:ReferenceChromosome{ordinal:chromosome.ordinal,name:chromosome.name.clone(),allele_class:chromosome.allele_class.clone(),invalid:chromosome.invalid}});
            for allele in &chromosome.alleles{data.push(ReferenceData::Allele{root:root.root.clone(),chromosome:chromosome.ordinal,allele:allele.clone()});}}
        for gene in &root.genes{data.push(ReferenceData::Gene{gene:gene.clone()});}
    }
    let domain=super::value_id("reference",&serde_json::json!({"kind":"binnieGenetics","records":data}))?;
    data.into_iter().enumerate().map(|(order,data)|{let mut row=GeneticReferenceRow{id:String::new(),domain:domain.clone(),order:order as u32,data};row.id=row.content_id()?;ensure!(serde_json::to_vec(&row)?.len()<=1024*1024,"Native reference leaf exceeds 1 MiB");Ok(row)}).collect()
}

pub fn validate(rows:&[GeneticReferenceRow],items:&BTreeMap<&str,&Item>)->Result<()> {
    ensure!(rows.len()<=1_000_000,"Native reference row budget exceeded");let mut domains=BTreeMap::<&str,Vec<&GeneticReferenceRow>>::new();let mut ids=BTreeSet::new();
    for row in rows {ensure!(row.id==row.content_id()?&&ids.insert(&row.id)&&serde_json::to_vec(row)?.len()<=1024*1024,"Invalid, duplicate or oversized native reference row");domains.entry(&row.domain).or_default().push(row);}
    for (id,mut rows) in domains {rows.sort_by_key(|r|r.order);for(index,row)in rows.iter().enumerate(){ensure!(row.order as usize==index,"Incomplete or repeated native reference order");}
        if matches!(&rows[0].data,ReferenceData::CropsNhDefinition(_)) {super::cropsnh::validate(id,&rows,items)?;continue;}
        if matches!(&rows[0].data,ReferenceData::UsageDefinition(_)) {super::usage_queries::validate(id,&rows,items)?;continue;}
        if matches!(&rows[0].data,ReferenceData::ProjectBlueDefinition(_)) {super::projectblue::validate(id,&rows,items)?;continue;}
        if matches!(&rows[0].data,ReferenceData::LatheDefinition(_)) {super::lathe::validate(id,&rows,items)?;continue;}
        if matches!(&rows[0].data,ReferenceData::FireworkDefinition(_)) {super::fireworks::validate(id,&rows,items)?;continue;}
        if matches!(&rows[0].data,ReferenceData::FurnaceFuelDefinition(_)) {super::furnace_fuels::validate(id,&rows,items)?;continue;}
        if matches!(&rows[0].data,ReferenceData::StructureCatalogDefinition(_)) {super::structure_references::validate(id,&rows,items)?;continue;}
        if matches!(&rows[0].data,ReferenceData::QuestsDefinition(_)) {super::betterquesting::validate(id,&rows,items)?;continue;}
        if matches!(&rows[0].data,ReferenceData::BartworksOresDefinition(_)) {super::bartworks_ores::validate(id,&rows,items)?;continue;}
        if matches!(&rows[0].data,ReferenceData::GtMineralsDefinition(_)) {super::gt_minerals::validate(id,&rows,items)?;continue;}
        if matches!(&rows[0].data,ReferenceData::UndergroundFluidsDefinition(_)) {super::underground_fluids::validate(id,&rows)?;continue;}
        if matches!(&rows[0].data,ReferenceData::BeeTreeDefinition(_)) {super::bee_tree::validate(id,&rows,items)?;continue;}
        if matches!(&rows[0].data,ReferenceData::ForestryAddonsDefinition(_)) {super::forestry_addons::validate(id,&rows,items)?;continue;}
        if matches!(&rows[0].data,ReferenceData::NeiInformationDefinition(_)) {super::nei_information::validate(id,&rows,items)?;continue;}
        if matches!(&rows[0].data,ReferenceData::NeiDiagramDefinition(_)) {super::nei_diagrams::validate(id,&rows,items)?;continue;}
        if matches!(&rows[0].data,ReferenceData::EnderStorageDefinition(_)) {super::ender_storage::validate(id,&rows,items)?;continue;}
        if matches!(&rows[0].data,ReferenceData::DraconicReactorDefinition(_)) {super::draconic_reactor::validate(id,&rows,items)?;continue;}
        if matches!(&rows[0].data,ReferenceData::InfernalDropsDefinition(_)) {super::infernal_drops::validate(id,&rows,items)?;continue;}
        if matches!(&rows[0].data,ReferenceData::MobDropsDefinition{..}) {super::mob_drops::validate(id,&rows,items)?;continue;}
        if matches!(&rows[0].data,ReferenceData::LexiconDefinition{..}) {super::lexicon::validate(id,&rows,items)?;continue;}
        if matches!(&rows[0].data,ReferenceData::ItemAspectsDefinition{..}) {super::item_aspects::validate(id,&rows,items)?;continue;}
        if matches!(&rows[0].data,ReferenceData::AspectCombinationsDefinition{..}) {super::aspect_combinations::validate(id,&rows)?;continue;}
        if matches!(&rows[0].data,ReferenceData::VillagerProfession{..}) {super::villager_trades::validate(id,&rows,items)?;continue;}
        if matches!(&rows[0].data,ReferenceData::ToolMaterialsDefinition{..}) {super::tool_materials::validate(id,&rows,items)?;continue;}
        if matches!(&rows[0].data,ReferenceData::ForgeRegistryDefinition(_)) {super::forge_registry::validate(id,&rows,items)?;continue;}
        let data:Vec<_>=rows.iter().map(|r|&r.data).collect();ensure!(id==super::value_id("reference",&serde_json::json!({"kind":"binnieGenetics","records":data}))?,"Native reference catalogue content identity differs");
        let ReferenceData::Definition{source,explanation,containers,consuming,counts,..}=&rows[0].data else{anyhow::bail!("Native reference definition must be first")};
        ensure!(!consuming&&source.owner=="Genetics"&&source.handler=="binnie.genetics.nei.DatabaseRecipeHandler"&&source.key=="native-gene-domain"&&containers.len()==2,"Database must be native read-only reference data");
        ensure!(explanation.key=="genetics.nei.tip.databaseDesc"&&!explanation.text.is_empty()&&explanation.text.len()<=65535,"Native Database localized explanation is absent or invalid");
        let mut actual=ReferenceCounts{roots:0,chromosomes:0,alleles:0,genes:0,containers:containers.len() as u32};
        let mut roots=BTreeMap::new();let mut chromosomes=BTreeMap::new();let mut alleles=BTreeMap::new();let mut genes=BTreeSet::new();
        for row in &rows[1..] {match &row.data {
            ReferenceData::CropsNhDefinition(_)|ReferenceData::CropsNhCrop(_)|ReferenceData::CropsNhAlternateSeed(_)|ReferenceData::CropsNhMutation(_)|ReferenceData::CropsNhMutationPool(_) => anyhow::bail!("Mixed CropsNH reference domain"),
            ReferenceData::UsageDefinition(_)|ReferenceData::UsageObservation(_)|ReferenceData::UsageDocument(_)|ReferenceData::UsageResource(_)=>anyhow::bail!("Mixed native usage domain"),
            ReferenceData::ProjectBlueDefinition(_)|ReferenceData::ProjectBlueRule(_)|ReferenceData::ProjectBlueItem(_)|ReferenceData::ProjectBlueCachedMaterial(_)|ReferenceData::ProjectBlueRegisteredMaterial(_)|ReferenceData::ProjectBlueMaterialAlias(_)|ReferenceData::ProjectBlueExample(_) => anyhow::bail!("Mixed native ProjectBlue domain"),
            ReferenceData::LatheDefinition(_)|ReferenceData::LatheSample(_)|ReferenceData::LatheWorkpiece(_)|ReferenceData::LatheState(_)|ReferenceData::LatheTool(_)|ReferenceData::LatheToolState(_) => anyhow::bail!("Mixed native lathe domain"),
            ReferenceData::FireworkDefinition(_)|ReferenceData::FireworkExample(_) => anyhow::bail!("Mixed native firework domain"),
            ReferenceData::FurnaceFuelDefinition(_)|ReferenceData::FurnaceFuel(_) => anyhow::bail!("Mixed native fuel domain"),
            ReferenceData::StructureCatalogDefinition(_)|ReferenceData::StructureCatalogController(_)|ReferenceData::StructureComponentLookup(_) => anyhow::bail!("Mixed native structure domain"),
            ReferenceData::QuestsDefinition(_)|ReferenceData::QuestDeclaration(_) => anyhow::bail!("Mixed native quest domain"),
            ReferenceData::BartworksOresDefinition(_)|ReferenceData::BartworksOreLayer(_)|ReferenceData::BartworksOreMaterial(_)|ReferenceData::BartworksOreLookup(_) => anyhow::bail!("Mixed native Bartworks ore domain"),
            ReferenceData::GtMineralsDefinition(_)|ReferenceData::GtMineralEntry(_)|ReferenceData::GtMineralDimensionShare(_) => anyhow::bail!("Mixed native GT mineral domain"),
            ReferenceData::UndergroundFluidsDefinition(_)|ReferenceData::UndergroundFluidDimension(_)|ReferenceData::UndergroundFluidEntry(_)|ReferenceData::UndergroundFluidDisplay(_) => anyhow::bail!("Mixed native underground-fluid domain"),
            ReferenceData::BeeTreeDefinition(_)|ReferenceData::BeeTreeSpecies(_)|ReferenceData::BeeTreeMutation(_) => anyhow::bail!("Mixed native bee tree domain"),
            ReferenceData::ForestryAddonsDefinition(_)|ReferenceData::ForestryAddonsSpecies(_)|ReferenceData::ForestryAddonsOutput(_)|ReferenceData::ForestryAddonsMutation(_) => anyhow::bail!("Mixed native Forestry Addons domain"),
            ReferenceData::NeiDiagramDefinition(_)|ReferenceData::NeiDiagram(_)|ReferenceData::NeiDiagramBinding(_)|ReferenceData::NeiDiagramLookup(_)|ReferenceData::NeiDiagramAction(_) => anyhow::bail!("Mixed native diagram domain"),
            ReferenceData::EnderStorageDefinition(_)|ReferenceData::EnderStorageState(_) => anyhow::bail!("Mixed native EnderStorage snapshot"),
            ReferenceData::DraconicReactorDefinition(_) => anyhow::bail!("Mixed native reactor domain"),
            ReferenceData::InfernalDropsDefinition(_)|ReferenceData::InfernalTier(_)|ReferenceData::InfernalPoolItem(_) => anyhow::bail!("Mixed native infernal domain"),
            ReferenceData::MobDropsDefinition{..}|ReferenceData::MobDefinition(_)|ReferenceData::MobDrop(_) => anyhow::bail!("Mixed native mob domain"),
            ReferenceData::LexiconDefinition{..}|ReferenceData::LexiconEntry{..} => anyhow::bail!("Mixed native lexicon domain"),
            ReferenceData::ItemAspectsDefinition{..}|ReferenceData::ItemAspectObservation{..} => anyhow::bail!("Mixed scanned-item reference domain"),
            ReferenceData::Definition{..}=>anyhow::bail!("Repeated native reference definition"),
            ReferenceData::Root{definition:root}=>{validate_root(root)?;ensure!(roots.insert(root.root.as_str(),root).is_none(),"Duplicate native reference root");actual.roots+=1;},
            ReferenceData::Chromosome{root,chromosome}=>{ensure!(roots.contains_key(root.as_str())&&!chromosome.name.is_empty()&&!chromosome.allele_class.is_empty()&&chromosomes.insert((root.as_str(),chromosome.ordinal),chromosome).is_none(),"Invalid/duplicate native reference chromosome");actual.chromosomes+=1;},
            ReferenceData::Allele{root,chromosome,allele}=>{ensure!(chromosomes.contains_key(&(root.as_str(),*chromosome))&&!allele.uid.is_empty()&&!allele.class.is_empty()&&alleles.insert((root.as_str(),*chromosome,allele.uid.as_str()),allele).is_none(),"Invalid/duplicate native reference allele");if let Some([low,high])=allele.tolerance_bounds{ensure!((-5..=0).contains(&low)&&(0..=5).contains(&high),"Invalid native reference tolerance bounds");}actual.alleles+=1;},
            ReferenceData::Gene{gene}=>{let root=roots.get(gene.root.as_str()).context("Native reference gene root absent")?;let allele=alleles.get(&(gene.root.as_str(),gene.chromosome,gene.allele.as_str())).context("Native reference gene allele absent")?;
                ensure!(allele.compatible&&genes.insert((gene.root.as_str(),gene.chromosome,gene.allele.as_str())),"Incompatible or duplicated native reference gene");
                for (id,registry,array,sequence) in [(&gene.serum,&root.serum,false,false),(&gene.genome,&root.genome,true,false),(&gene.sequence,&root.sequence,false,true)]{let fact=items.get(id.as_str()).context("Native reference gene item fact absent")?;ensure!(&fact.registry==registry,"Native reference gene container differs");validate_gene_nbt(fact,gene,array,sequence)?;}actual.genes+=1;},
            ReferenceData::AspectCombinationsDefinition{..}|ReferenceData::AspectCombination{..}|ReferenceData::VillagerProfession{..}|ReferenceData::VillagerOffer{..}|ReferenceData::ToolMaterialsDefinition{..}|ReferenceData::ToolMaterial{..}|ReferenceData::ToolMaterialPart{..}|ReferenceData::ForgeRegistryDefinition(_)|ReferenceData::ForgeOreGroup(_)|ReferenceData::ForgeFluid(_)|ReferenceData::NeiInformationDefinition(_)|ReferenceData::NeiInformationPage(_)|ReferenceData::NeiInteractiveGroup{..}=>anyhow::bail!("Typed reference domains must be validated independently"),
        }}
        ensure!(&actual==counts&&counts.roots>0&&counts.roots<=4&&counts.genes>0,"Native reference catalogue is incomplete or count evidence differs");
        for root in roots.keys(){let ordinals:Vec<_>=chromosomes.keys().filter(|(r,_)|r==root).map(|(_,c)|*c).collect();ensure!(!ordinals.is_empty()&&ordinals.iter().enumerate().all(|(i,c)|i==usize::from(*c)),"Native reference karyotype is incomplete");}
        ensure!(alleles.iter().filter(|(_,a)|a.compatible).all(|(key,_)|genes.contains(key)),"Native compatible allele catalogue lost gene facts");
        let mut kinds=BTreeSet::new();for container in containers {let serum=container.kind==ContainerKind::Serum;ensure!(kinds.insert(serum)&&matches!((container.root_gate,container.gene_policy),(AcceptorRootGate::Single,GenePolicy::Single)|(AcceptorRootGate::Array,GenePolicy::Array))&&serum==(container.root_gate==AcceptorRootGate::Single),"Native empty container policy differs");
            ensure!(roots.values().all(|r|container.item==if serum{r.serum.clone()}else{r.genome.clone()}),"Native serum container root identity differs");let empty=items.get(container.empty.as_str()).context("Native empty container fact absent")?;ensure!(empty.meta==if serum{6}else{7}&&empty.nbt.is_none(),"Native empty serum/genome specimen differs");}
    }Ok(())
}
fn validate_root(root:&ReferenceRoot)->Result<()> {let system=match root.root_class.as_str(){"forestry.apiculture.genetics.BeeHelper"=>"binnie.core.genetics.BeeBreedingSystem","forestry.arboriculture.genetics.TreeHelper"=>"binnie.core.genetics.TreeBreedingSystem","forestry.lepidopterology.genetics.ButterflyHelper"=>"binnie.core.genetics.MothBreedingSystem","binnie.botany.genetics.FlowerHelper"=>"binnie.core.genetics.FlowerBreedingSystem",_=>anyhow::bail!("Unknown native reference root callback")};ensure!(!root.root.is_empty()&&root.system_class==system&&!root.member_types.is_empty()&&root.member_types.len()<=128&&root.member_types.iter().copied().collect::<BTreeSet<_>>().len()==root.member_types.len()&&!root.serum.is_empty()&&!root.genome.is_empty()&&!root.sequence.is_empty()&&root.serum!=root.genome&&root.serum!=root.sequence&&root.genome!=root.sequence,"Native reference root identities/types differ");Ok(())}
fn fields(nbt:&Nbt)->Result<&BTreeMap<String,Nbt>>{nbt.validate()?;if let Nbt::Compound{value}=nbt{Ok(value)}else{anyhow::bail!("Native gene NBT must be compound")}}
fn validate_gene_nbt(item:&Item,gene:&GeneReference,array:bool,sequence:bool)->Result<()> {let nbt=fields(item.nbt.as_ref().context("Native reference gene NBT absent")?)?;let value=if array{let Some(Nbt::List{element,value})=nbt.get("genes")else{anyhow::bail!("Native reference genome array absent")};ensure!(element=="compound"&&value.len()==1,"Reference catalogue genome must contain its one native gene");&value[0]}else{nbt.get("gene").context("Native reference single gene absent")?};let fields=fields(value)?;
    ensure!(matches!(fields.get("root"),Some(Nbt::String{value})if value==&gene.root)&&matches!(fields.get("allele"),Some(Nbt::String{value})if value==&gene.allele)&&matches!(fields.get("chromo"),Some(Nbt::Byte{value})if value==&gene.chromosome.to_string()),"Native reference gene NBT differs from root/chromosome/allele");
    if sequence{ensure!(item.meta==5&&item.durability==5&&matches!(nbt.get("seq"),Some(Nbt::Byte{value})if value=="0")&&matches!(nbt.get("ana"),Some(Nbt::Byte{value})if value=="0"),"Native unsequenced reference specimen differs");}else{ensure!(item.meta==item.durability&&item.meta>0,"Native reference serum must have zero charges");}Ok(())}

#[cfg(test)] mod tests {
    use super::*;
    #[test]fn java_native_tool_material_leaves_are_typed_and_preserved(){
        let fixture:serde_json::Value=serde_json::from_slice(&std::fs::read(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../contracts/fixtures/tconstruct-tool-materials-contract.json")).unwrap()).unwrap();
        let rows:Vec<GeneticReferenceRow>=serde_json::from_value(fixture["reference"]["records"].clone()).expect("Java native tool material rows must have a typed Compiler contract");
        let source=crate::source::Source::open(&std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../contracts/fixtures/source-semantic-40")).unwrap();
        let base=serde_json::to_value(super::super::Domain::load(&source).unwrap().items.remove(0)).unwrap();
        let items:Vec<Item>=fixture["items"].as_array().unwrap().iter().map(|raw|{let mut item=base.clone();for key in ["id","registry","meta","nbt","tags","durability"]{item[key]=raw[key].clone();}serde_json::from_value(item).unwrap()}).collect();
        let facts=items.iter().map(|item|(item.id.as_str(),item)).collect();
        validate(&rows,&facts).unwrap();
        let mut missing=rows.clone();missing.pop();assert!(validate(&missing,&facts).is_err());
        let mut mutated=items.clone();mutated[0].nbt=None;assert!(validate(&rows,&mutated.iter().map(|item|(item.id.as_str(),item)).collect()).is_err());
        let mut drift=rows.clone();drift[1].order=0;drift[1].id=drift[1].content_id().unwrap();assert!(validate(&drift,&facts).is_err());
    }
    fn native_transport()->(Vec<GeneticReferenceRow>,Vec<Item>) {
        let fixtures=std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../contracts/fixtures");
        let fixture:serde_json::Value=serde_json::from_slice(&std::fs::read(fixtures.join("binnie-genetics-reference-rows-contract.json")).unwrap()).unwrap();
        let raw:serde_json::Value=serde_json::from_slice(&std::fs::read(fixtures.join("binnie-genetics-reference-contract.json")).unwrap()).unwrap();
        let rows:Vec<GeneticReferenceRow>=serde_json::from_value(fixture["reference"]["records"].clone()).unwrap();
        let reference:GeneticsReference=serde_json::from_value(raw["reference"].clone()).unwrap();
        let ReferenceData::Definition{explanation,..}=&rows[0].data else{panic!("native definition absent")};
        let normalized=normalize_with(&reference,explanation.clone()).unwrap();
        assert_eq!(serde_json::to_value(&rows).unwrap(),serde_json::to_value(normalized).unwrap(),"Java/Rust native catalogue normalization differs");
        let source=crate::source::Source::open(&fixtures.join("source-semantic-40")).unwrap();
        let base=serde_json::to_value(super::super::Domain::load(&source).unwrap().items.remove(0)).unwrap();
        let items=fixture["items"].as_array().unwrap().iter().map(|raw| {
            assert!(raw.get("durability").is_some(),"native charge evidence must be captured");
            let mut item=base.clone();for key in ["id","registry","meta","nbt","tags","durability"] {item[key]=raw[key].clone();}
            serde_json::from_value(item).unwrap()
        }).collect();
        (rows,items)
    }
    #[test]fn actual_java_native_leaves_validate_and_preserve_gene_facts(){
        let(rows,items)=native_transport();let facts=items.iter().map(|item|(item.id.as_str(),item)).collect();
        validate(&rows,&facts).unwrap();
        let gene=rows.iter().find_map(|row|if let ReferenceData::Gene{gene}=&row.data{Some(gene)}else{None}).unwrap();
        for id in [&gene.serum,&gene.genome,&gene.sequence] {
            let mut altered=items.clone();let item=altered.iter_mut().find(|item|&item.id==id).unwrap();item.nbt=None;
            assert!(validate(&rows,&altered.iter().map(|item|(item.id.as_str(),item)).collect()).is_err(),"Missing native gene NBT accepted");
        }
        let mut altered=items.clone();altered.iter_mut().find(|item|item.id==gene.serum).unwrap().meta=0;
        assert!(validate(&rows,&altered.iter().map(|item|(item.id.as_str(),item)).collect()).is_err(),"Charged serum reference accepted");
        let mut altered=items.clone();altered.iter_mut().find(|item|item.id==gene.sequence).unwrap().meta=0;
        assert!(validate(&rows,&altered.iter().map(|item|(item.id.as_str(),item)).collect()).is_err(),"Charged sequence reference accepted");
        let mut altered=items.clone();let sequence=altered.iter_mut().find(|item|item.id==gene.sequence).unwrap();
        if let Some(Nbt::Compound{value})=&mut sequence.nbt{value.insert("ana".into(),Nbt::Byte{value:"1".into()});}
        assert!(validate(&rows,&altered.iter().map(|item|(item.id.as_str(),item)).collect()).is_err(),"Analysed sequence reference accepted");
        let mut missing=rows.clone();missing.pop();assert!(validate(&missing,&facts).is_err());
        let mut duplicate=rows.clone();duplicate.push(rows[0].clone());assert!(validate(&duplicate,&facts).is_err());
        let mut changed=rows.clone();if let ReferenceData::Definition{explanation,..}=&mut changed[0].data{explanation.text.push_str(" drift");}
        assert!(validate(&changed,&facts).is_err(),"Changed native explanation escaped content binding");
    }
    #[test]fn actual_native_reference_normalizes_and_rejects_truncation(){let value:serde_json::Value=serde_json::from_slice(&std::fs::read(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../contracts/fixtures/binnie-genetics-reference-contract.json")).unwrap()).unwrap();let reference:GeneticsReference=serde_json::from_value(value["reference"].clone()).unwrap();let rows=normalize(&reference).unwrap();assert!(rows.len()>reference.catalog.len());assert!(rows.iter().all(|r|serde_json::to_vec(r).unwrap().len()<=1024*1024));let mut truncated=rows.clone();truncated.pop();assert!(validate(&truncated,&BTreeMap::new()).is_err());let mut duplicate=rows.clone();duplicate.push(rows[0].clone());assert!(validate(&duplicate,&BTreeMap::new()).is_err());}
    #[test]fn native_database_transport_obeys_complete_domain_gates(){
        let source=crate::source::Source::open(&std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../contracts/fixtures/source-semantic-40")).unwrap();
        let mut domain=super::super::Domain::load(&source).unwrap();
        let (rows,items)=native_transport();
        let ReferenceData::Definition{source:native_source,..}=&rows[0].data else{panic!("native definition absent")};
        let native_source=native_source.clone();
        let mut category=domain.categories[0].clone();category.source=native_source.clone();category.id=super::super::origin_id("category",&native_source).unwrap();
        category.reference=Some(rows[0].domain.clone());category.program=None;category.icon=None;category.machines.clear();
        domain.categories.push(category.clone());domain.references=rows;
        for item in items{domain.items.retain(|existing|existing.id!=item.id);domain.items.push(item);}
        domain.validate(&source).unwrap();
        let mut orphan=domain.clone();orphan.categories.pop();
        assert!(orphan.validate(&source).unwrap_err().to_string().contains("Unreferenced native Database domain"));
        let mut wrong=domain.clone();wrong.categories.last_mut().unwrap().reference=Some(format!("reference_{}","0".repeat(64)));
        assert!(wrong.validate(&source).unwrap_err().to_string().contains("Category refers to a missing native reference domain"));
        let mut consuming=domain.clone();let mut recipe=consuming.recipes[0].clone();recipe.category=category.id;recipe.source=native_source.clone();
        recipe.id=super::super::recipe_id(&recipe).unwrap();consuming.recipes.push(recipe);
        assert!(consuming.validate(&source).unwrap_err().to_string().contains("Native reference category became consuming recipes"));
    }
    #[test]fn dimensions_only_interactive_reference_is_rejected(){
        let origin=Origin{owner:"neicustomdiagram".into(),handler:"com.github.dcysteine.neicustomdiagram.api.diagram.CustomDiagramGroup".into(),key:"neicustomdiagram.diagramgroup.gregtech.circuits".into()};
        let first=ReferenceData::NeiInteractiveGroup{source:origin.clone(),group:origin.key.clone(),page:0,width:166,height:65,diagrams_per_page:1,consuming:false};
        let second=ReferenceData::NeiInteractiveGroup{source:origin,group:"neicustomdiagram.diagramgroup.gregtech.circuits".into(),page:1,width:166,height:65,diagrams_per_page:1,consuming:false};
        let data=vec![first,second];
        let domain_id=super::super::value_id("reference",&serde_json::json!({"kind":"neiInteractiveGroup","records":data})).unwrap();
        let rows=data.into_iter().enumerate().map(|(order,data)|{let mut row=GeneticReferenceRow{id:String::new(),domain:domain_id.clone(),order:order as u32,data};row.id=row.content_id().unwrap();row}).collect::<Vec<_>>();
        assert!(validate(&rows,&BTreeMap::new()).is_err());
        let mut altered=rows.clone(); if let ReferenceData::NeiInteractiveGroup{consuming,..}=&mut altered[0].data{*consuming=true;} altered[0].id=altered[0].content_id().unwrap();
        assert!(validate(&altered,&BTreeMap::new()).is_err());
        let mut gap=rows.clone(); if let ReferenceData::NeiInteractiveGroup{page,..}=&mut gap[1].data{*page=2;} gap[1].id=gap[1].content_id().unwrap();
        assert!(validate(&gap,&BTreeMap::new()).is_err());
    }
}
