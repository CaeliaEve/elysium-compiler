//! Read-only native cell observations, manual content and static OC Callback documentation.
use super::{references::{GeneticReferenceRow, ReferenceData}, Item, Origin};
use anyhow::{ensure, Context, Result};
use base64::{engine::general_purpose::STANDARD, Engine};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use sha2::{Digest,Sha256};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all="camelCase")]
pub enum UsageBackend { AeItemCell, AeFluidCell, OcManual, OcCallbacks }
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all="camelCase")]
pub enum UsageScope { CatalogObservations }
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct UsageCounts {pub scanned:u32,pub queries:u32,pub matched:u32,pub unmatched:u32,pub duplicates:u32,pub documents:u32,pub resources:u32}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct UsageDefinition {
    pub source:Origin,pub backend:UsageBackend,pub native_version:String,pub locale:String,pub scope:UsageScope,
    pub ready:bool,pub consuming:bool,pub counts:UsageCounts,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct UsageInput {pub item:String,pub amount:String}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all="camelCase")]
pub enum UsageEntryKind {Item, Fluid}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct UsageEntry {pub kind:UsageEntryKind,pub id:String,pub amount:String}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all="camelCase")]
pub enum UsageChannel {Items, Fluids}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub enum CellFaultKind {#[serde(rename="missingCellInventory")]MissingCellInventory}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub enum CellNativeException {#[serde(rename="java.lang.NullPointerException")]NullPointerException}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(tag="kind",rename_all="camelCase",rename_all_fields="camelCase",deny_unknown_fields)]
pub enum UsageResult {
    /// An observed native viewer fault. This is neither an empty cell nor a non-match.
    CellFault {channel:UsageChannel,provider:String,inventory:String,fault:CellFaultKind,#[serde(rename="nativeException")]native_exception:CellNativeException},
    Cell {channel:UsageChannel,provider:String,inventory:String,
        #[serde(rename="totalBytes")]total_bytes:String,
        #[serde(rename="usedBytes")]used_bytes:String,
        #[serde(rename="totalTypes")]total_types:String,
        #[serde(rename="usedTypes")]used_types:String,entries:Vec<UsageEntry>},
    Manual {path:String,document:String},
    Callbacks {documents:Vec<String>},
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct UsageObservation {pub native_index:u32,pub input:UsageInput,pub result:UsageResult}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all="camelCase")]
pub enum UsageLinkKind {Document, External}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct UsageLink {pub kind:UsageLinkKind,pub label:String,pub href:String,pub target:String,pub anchor:String}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub enum UsageMissingWarning {
    #[serde(rename="oc:gui.Manual.Warning.ImageMissing")]Image,
    #[serde(rename="oc:gui.Manual.Warning.ItemMissing")]Item,
    #[serde(rename="oc:gui.Manual.Warning.BlockMissing")]Block,
    #[serde(rename="oc:gui.Manual.Warning.OreDictMissing")]OreDict,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(tag="kind",rename_all="camelCase",rename_all_fields="camelCase",deny_unknown_fields)]
pub enum UsageImage {
    Png {href:String,label:String,resource:String},
    Items {href:String,label:String,items:Vec<String>},
    Missing {href:String,label:String,#[serde(rename="nativeWarning")]native_warning:UsageMissingWarning},
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all="camelCase")]
pub enum UsageDocumentStatus {Found, Missing}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all="camelCase")]
pub enum UsageAnalysis {NativeStaticAnnotations}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct UsageCallback {pub name:String,pub declaring_class:String,pub java_name:String,pub doc:String,pub direct:bool,pub limit:i32,pub getter:bool,pub setter:bool}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(tag="kind",rename_all="camelCase",rename_all_fields="camelCase",deny_unknown_fields)]
pub enum UsageDocumentContent {
    Manual {
        path:String,#[serde(rename="resolvedPath")]resolved_path:String,
        #[serde(rename="requestedLocale")]requested_locale:String,
        #[serde(rename="fallbackAttempted")]fallback_attempted:bool,status:UsageDocumentStatus,
        #[serde(deserialize_with="super::nei_information::required_nullable")]
        #[schemars(with="super::nei_information::RequiredNullable<String>")]
        provider:Option<String>,redirects:Vec<String>,lines:Vec<String>,links:Vec<UsageLink>,images:Vec<UsageImage>,
    },
    Callbacks {environment:String,analysis:UsageAnalysis,#[serde(rename="ignoredAnnotations")]ignored_annotations:u32,methods:Vec<UsageCallback>},
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct UsageDocument {pub key:String,pub content:UsageDocumentContent}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub enum UsageMime {#[serde(rename="image/png")]Png}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct UsageResource {pub key:String,pub location:String,pub mime:UsageMime,pub width:u32,pub height:u32,pub sha256:String,pub bytes:u32,pub base64:String}

const AE:&str="appeng.integration.modules.NEIHelpers.NEICellViewHandler";
const FC:&str="com.glodblock.github.nei.NEICellViewHandler";
const MANUAL:&str="li.cil.oc.integration.nei.ManualUsageHandler";
const CALLBACKS:&str="li.cil.oc.integration.nei.CallbackDocHandler";
fn text(value:&str,limit:usize)->Result<()> {ensure!(value.len()<=limit&&!value.contains('\0'),"Native usage text exceeds its budget");Ok(())}
fn path(value:&str)->Result<()> {text(value,4096)?;ensure!(!value.is_empty()&&!value.contains('\\')&&!value.split('/').any(|part|part==".."),"Invalid native usage path");Ok(())}
fn integer(value:&str)->Result<i64> {let number=value.parse::<i64>().context("Invalid signed native usage counter")?;ensure!(number.to_string()==value,"Noncanonical native usage counter");Ok(number)}
fn document_key(path:&str)->Result<String>{super::value_id("usage_doc",&serde_json::json!({"backend":"ocManual","path":path}))}

pub fn validate(id:&str,rows:&[&GeneticReferenceRow],items:&BTreeMap<&str,&Item>)->Result<()> {
    let ReferenceData::UsageDefinition(def)=&rows[0].data else {anyhow::bail!("Native usage definition must be first")};
    let (handler,owner,version)=match def.backend {
        UsageBackend::AeItemCell=>(AE,"appliedenergistics2","rv3-beta-695-GTNH"),UsageBackend::AeFluidCell=>(FC,"ae2fc","1.4.120-gtnh"),
        UsageBackend::OcManual=>(MANUAL,"OpenComputers","1.11.20-GTNH"),UsageBackend::OcCallbacks=>(CALLBACKS,"OpenComputers","1.11.20-GTNH"),
    };
    ensure!(def.source.handler==handler&&def.source.key==handler&&def.source.owner==owner&&def.native_version==version
        &&matches!(def.locale.as_str(),"zh_CN"|"en_US")&&def.ready&&!def.consuming,"Native usage source/version/locale differs");
    let c=&def.counts;
    ensure!([c.scanned,c.queries,c.matched,c.unmatched,c.duplicates,c.documents,c.resources].iter().all(|n|*n<=262144)
        &&u64::from(c.scanned)==u64::from(c.queries)+u64::from(c.duplicates)
        &&u64::from(c.queries)==u64::from(c.matched)+u64::from(c.unmatched)
        &&rows.len()==1+c.matched as usize+c.documents as usize+c.resources as usize,"Native usage disposition counts differ");
    let data:Vec<_>=rows.iter().map(|row|&row.data).collect();
    ensure!(serde_json::to_vec(&data)?.len()<=64*1024*1024&&id==super::value_id("reference",&serde_json::json!({"kind":"usageQueries","records":data}))?,"Native usage content identity/budget differs");
    let mut documents=BTreeMap::new();let mut resources=BTreeMap::new();
    for row in &rows[1+c.matched as usize..1+c.matched as usize+c.documents as usize] {
        let ReferenceData::UsageDocument(doc)=&row.data else {anyhow::bail!("Native usage document order differs")};
        ensure!(documents.insert(doc.key.as_str(),doc).is_none(),"Duplicate native usage document");
    }
    for row in &rows[1+c.matched as usize+c.documents as usize..] {
        let ReferenceData::UsageResource(resource)=&row.data else {anyhow::bail!("Native usage resource order differs")};
        validate_resource(resource)?;ensure!(resources.insert(resource.key.as_str(),resource).is_none(),"Duplicate native usage resource");
    }
    let mut seen=BTreeSet::new();let mut prior=None;
    for row in &rows[1..1+c.matched as usize] {
        let ReferenceData::UsageObservation(observation)=&row.data else {anyhow::bail!("Native usage observation order differs")};
        let amount=integer(&observation.input.amount)?;
        ensure!(amount>0&&amount<=i64::from(i32::MAX)&&items.contains_key(observation.input.item.as_str())
            &&observation.native_index<c.scanned&&prior.is_none_or(|index|observation.native_index>index)
            &&seen.insert((&observation.input.item,&observation.input.amount)),"Invalid/duplicate native usage input/order");prior=Some(observation.native_index);
        match &observation.result {
            UsageResult::CellFault{channel,provider,inventory,..}=>{
                ensure!(def.backend==UsageBackend::AeItemCell&&*channel==UsageChannel::Items
                    &&inventory=="appeng.me.storage.CellInventoryHandler"
                    &&["appeng.core.features.registries.entries.BasicCellHandler","appeng.core.features.registries.entries.CreativeCellHandler",
                        "appeng.core.features.registries.entries.VoidCellHandler","com.glodblock.github.crossmod.extracells.storage.ProxyItemCellHandler",
                        "com.glodblock.github.crossmod.extracells.storage.ProxyVoidCellHandler"].contains(&provider.as_str()),
                    "Unadapted native cell inventory fault");
            }
            UsageResult::Cell{channel,provider,inventory,total_bytes,used_bytes,total_types,used_types,entries}=>{
                let fluid=def.backend==UsageBackend::AeFluidCell;
                ensure!(matches!(def.backend,UsageBackend::AeItemCell|UsageBackend::AeFluidCell)
                    &&*channel==if fluid{UsageChannel::Fluids}else{UsageChannel::Items}
                    &&inventory==if fluid{"com.glodblock.github.common.storage.FluidCellInventoryHandler"}else{"appeng.me.storage.CellInventoryHandler"},"Native cell channel/inventory differs");
                ensure!(["appeng.core.features.registries.entries.BasicCellHandler","appeng.core.features.registries.entries.CreativeCellHandler",
                    "appeng.core.features.registries.entries.VoidCellHandler","com.glodblock.github.common.storage.FluidCellHandler",
                    "com.glodblock.github.crossmod.extracells.storage.ProxyItemCellHandler","com.glodblock.github.crossmod.extracells.storage.ProxyVoidCellHandler"].contains(&provider.as_str()),"Unadapted native cell provider");
                ensure!(integer(total_bytes)?>0&&integer(total_types)?>=0,"Invalid native cell capacity");integer(used_bytes)?;integer(used_types)?;
                let mut last=i64::MAX;let mut contents=BTreeSet::new();ensure!(entries.len()<=262144,"Native cell entry budget exceeded");
                for entry in entries {
                    let amount=integer(&entry.amount)?;ensure!(amount>0&&amount<=last&&contents.insert(entry.id.as_str())
                        &&entry.kind==if fluid{UsageEntryKind::Fluid}else{UsageEntryKind::Item},"Native cell contents/order differ");last=amount;
                    if !fluid{ensure!(items.contains_key(entry.id.as_str()),"Native cell item is missing");}
                    else{ensure!(entry.id.starts_with("fluid_"),"Native cell fluid identity is invalid");}
                }
            }
            UsageResult::Manual{path:query,document}=>{
                ensure!(def.backend==UsageBackend::OcManual&&document==&document_key(query)?&&documents.contains_key(document.as_str()),"Native manual query target is missing");
                path(query)?;
            }
            UsageResult::Callbacks{documents:links}=>{
                ensure!(def.backend==UsageBackend::OcCallbacks&&!links.is_empty()&&links.len()<=262144,"Invalid native API query");
                for key in links{ensure!(documents.contains_key(key.as_str()),"Native API query document is missing");}
            }
        }
    }
    for doc in documents.values(){validate_document(doc,def,&documents,&resources,items)?;}
    ensure!(def.backend==UsageBackend::OcManual||resources.is_empty(),"Non-manual usage domain has image resources");
    ensure!(matches!(def.backend,UsageBackend::OcManual|UsageBackend::OcCallbacks)||documents.is_empty(),"Cell domain has documentation");
    Ok(())
}

fn validate_document(doc:&UsageDocument,def:&UsageDefinition,documents:&BTreeMap<&str,&UsageDocument>,resources:&BTreeMap<&str,&UsageResource>,items:&BTreeMap<&str,&Item>)->Result<()> {
    match &doc.content {
        UsageDocumentContent::Manual{path:logical,resolved_path,requested_locale,fallback_attempted,status,provider,redirects,lines,links,images}=>{
            ensure!(def.backend==UsageBackend::OcManual&&doc.key==document_key(logical)?&&requested_locale==&def.locale,"Native manual identity/locale differs");
            path(logical)?;path(resolved_path)?;
            let language=if *fallback_attempted {ensure!(def.locale=="zh_CN"&&logical.contains("%LANGUAGE%"),"Invalid native fallback");"en_US"}else{def.locale.as_str()};
            ensure!(!redirects.is_empty()&&redirects.len()<=64&&redirects.first()==Some(&logical.replace("%LANGUAGE%",language))
                &&redirects.last()==Some(resolved_path)&&redirects.iter().collect::<BTreeSet<_>>().len()==redirects.len(),"Native manual redirect chain differs");
            for location in redirects{path(location)?;}
            ensure!(lines.len()<=65536&&links.len()<=262144&&images.len()<=262144,"Native manual page budget exceeded");
            for line in lines{text(line,262144)?;}
            if *status==UsageDocumentStatus::Missing {ensure!(provider.is_none()&&lines.is_empty()&&links.is_empty()&&images.is_empty(),"Missing native manual page acquired content");}
            else{ensure!(provider.as_ref().is_some_and(|p|!p.is_empty()),"Native manual resource provenance missing");}
            for link in links {
                text(&link.label,65536)?;text(&link.href,4096)?;text(&link.anchor,4096)?;
                match link.kind {
                    UsageLinkKind::Document=>{path(&link.target)?;ensure!(documents.contains_key(document_key(&link.target)?.as_str()),"Native manual link closure is incomplete");}
                    UsageLinkKind::External=>ensure!((link.href.starts_with("https://")||link.href.starts_with("http://"))&&link.target==link.href&&link.anchor.is_empty(),"Invalid external native manual link"),
                }
            }
            for image in images {match image {
                UsageImage::Png{href,label,resource}=>{text(href,4096)?;text(label,65536)?;ensure!(resources.contains_key(resource.as_str()),"Manual image resource is absent");}
                UsageImage::Items{href,label,items:choices}=>{text(href,4096)?;text(label,65536)?;ensure!(!choices.is_empty()&&choices.len()<=262144&&choices.iter().all(|id|items.contains_key(id.as_str())),"Manual image item closure is incomplete");}
                UsageImage::Missing{href,label,..}=>{text(href,4096)?;text(label,65536)?;}
            }}
        }
        UsageDocumentContent::Callbacks{environment,methods,ignored_annotations,..}=>{
            ensure!(def.backend==UsageBackend::OcCallbacks&&doc.key==super::value_id("usage_doc",&serde_json::json!({"backend":"ocCallbacks","environment":environment}))?
                &&!environment.is_empty()&&environment.len()<=1024&&!methods.is_empty()&&methods.len()<=65536&&*ignored_annotations<=65536,"Invalid native API document");
            let mut names=BTreeSet::new();
            for method in methods {
                ensure!(!method.name.is_empty()&&names.insert(method.name.as_str())&&!method.java_name.is_empty()&&!method.declaring_class.is_empty(),"Invalid/duplicate native API callback");
                text(&method.name,4096)?;text(&method.java_name,4096)?;text(&method.declaring_class,4096)?;text(&method.doc,262144)?;
            }
        }
    }
    Ok(())
}
fn validate_resource(resource:&UsageResource)->Result<()> {
    ensure!(resource.bytes<=768*1024&&resource.width>0&&resource.height>0&&resource.width<=4096&&resource.height<=4096,"Native manual image budget exceeded");
    text(&resource.location,4096)?;
    let raw=STANDARD.decode(&resource.base64).context("Invalid native manual image encoding")?;
    ensure!(raw.len()==resource.bytes as usize&&STANDARD.encode(&raw)==resource.base64&&format!("{:x}",Sha256::digest(&raw))==resource.sha256,"Native manual image bytes/digest differ");
    ensure!(raw.len()>=24&&raw[..8]==[137,80,78,71,13,10,26,10]&&u32::from_be_bytes(raw[16..20].try_into().unwrap())==resource.width
        &&u32::from_be_bytes(raw[20..24].try_into().unwrap())==resource.height,"Native manual image header differs");
    let image=image::load_from_memory_with_format(&raw,image::ImageFormat::Png)?;
    ensure!(image.width()==resource.width&&image.height()==resource.height&&resource.key==super::value_id("usage_resource",&serde_json::json!({"location":resource.location,"sha256":resource.sha256}))?,"Native manual image identity/dimensions differ");
    Ok(())
}
pub fn validate_facts(rows:&[GeneticReferenceRow],fluids:&BTreeMap<&str,&super::Fluid>)->Result<()> {
    for row in rows{if let ReferenceData::UsageObservation(UsageObservation{result:UsageResult::Cell{entries,..},..})=&row.data{
        for entry in entries{if entry.kind==UsageEntryKind::Fluid{ensure!(fluids.contains_key(entry.id.as_str()),"Native cell fluid fact is absent");}}
    }}Ok(())
}
pub fn validate_locale(rows:&[GeneticReferenceRow],locale:&str)->Result<()> {
    for row in rows{if let ReferenceData::UsageDefinition(def)=&row.data{
        ensure!(def.locale==locale,"Native usage locale differs from Source environment");
    }}Ok(())
}

#[test]
fn java_native_usage_queries_preserve_content_and_reject_inconsistent_domains() {
    let raw:serde_json::Value=serde_json::from_str(include_str!("../../../../contracts/fixtures/native-usage-query-contract.json")).unwrap();
    let source=crate::source::Source::open(&std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../contracts/fixtures/source-semantic-40")).unwrap();
    let baseline=super::Domain::load(&source).unwrap();
    let items:Vec<Item>=raw["items"].as_array().unwrap().iter().map(|row|{
        let mut value=serde_json::to_value(&baseline.items[0]).unwrap();
        for(key,field)in row.as_object().unwrap(){value[key]=field.clone();}serde_json::from_value(value).unwrap()
    }).collect();
    let fluids:Vec<super::Fluid>=raw["fluids"].as_array().unwrap().iter().map(|row|{
        let mut value=serde_json::to_value(&baseline.fluids[0]).unwrap();
        for(key,field)in row.as_object().unwrap(){value[key]=field.clone();}serde_json::from_value(value).unwrap()
    }).collect();
    let item_map=items.iter().map(|item|(item.id.as_str(),item)).collect();
    let fluid_map=fluids.iter().map(|fluid|(fluid.id.as_str(),fluid)).collect();
    let domains:Vec<Vec<GeneticReferenceRow>>=serde_json::from_value(raw["domains"].clone()).unwrap();
    assert_eq!(domains.len(),5,"Two cell channels including native creative-cell fault, two manual locales and native EEPROM callbacks");
    for rows in &domains{super::references::validate(rows,&item_map).unwrap();validate_facts(rows,&fluid_map).unwrap();}
    validate_locale(&domains[2],"zh_CN").unwrap();assert!(validate_locale(&domains[2],"en_US").is_err());
    assert!(validate_facts(&domains[1],&BTreeMap::new()).is_err());
    fn seal(rows:&mut [GeneticReferenceRow]){
        let data:Vec<_>=rows.iter().map(|row|&row.data).collect();
        let domain=super::value_id("reference",&serde_json::json!({"kind":"usageQueries","records":data})).unwrap();
        for row in rows{row.domain=domain.clone();row.id=row.content_id().unwrap();}
    }
    for mutation in 0..7 {
        let mut rows=domains[if mutation<3{0}else if mutation<6{2}else{4}].clone();
        match mutation {
            0=>if let ReferenceData::UsageDefinition(def)=&mut rows[0].data{def.consuming=true;},
            1=>if let ReferenceData::UsageObservation(UsageObservation{result:UsageResult::Cell{entries,..},..})=&mut rows[1].data{entries[0].amount="9223372036854775808".into();},
            2=>if let ReferenceData::UsageObservation(UsageObservation{result:UsageResult::Cell{channel,..},..})=&mut rows[1].data{*channel=UsageChannel::Fluids;},
            3=>if let ReferenceData::UsageObservation(UsageObservation{result:UsageResult::Manual{document,..},..})=&mut rows[1].data{*document="usage_doc_missing".into();},
            4=>for row in &mut rows{if let ReferenceData::UsageResource(resource)=&mut row.data{resource.base64.push('A');}},
            5=>for row in &mut rows{if let ReferenceData::UsageDocument(UsageDocument{content:UsageDocumentContent::Manual{requested_locale,..},..})=&mut row.data{*requested_locale="en_US".into();}},
            6=>for row in &mut rows{if let ReferenceData::UsageDocument(UsageDocument{content:UsageDocumentContent::Callbacks{environment,..},..})=&mut row.data{*environment="foreign.Environment".into();}},
            _=>unreachable!(),
        }
        seal(&mut rows);assert!(super::references::validate(&rows,&item_map).is_err(),"Native usage semantic mutation {mutation} was accepted");
    }
}
