//! Focused replay of this batch's native observations; avoids compiling the full unit-test body.
use elysium_compiler_core::{domain::{self, Domain, Item, Recipe, Fluid, references::GeneticReferenceRow},source::Source};
use serde_json::{Value,json};
use std::{collections::BTreeMap,path::PathBuf};

fn fixtures()->PathBuf{PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../contracts/fixtures")}
fn baseline()->(Source,Domain){let s=Source::open(&fixtures().join("source-semantic-40")).unwrap();let d=Domain::load(&s).unwrap();(s,d)}
fn raw(name:&str)->Value{serde_json::from_slice(&std::fs::read(fixtures().join(name)).unwrap()).unwrap()}
fn facts<T:serde::de::DeserializeOwned>(values:&Value,base:&impl serde::Serialize)->Vec<T>{values.as_array().unwrap().iter().map(|row|{let mut value=serde_json::to_value(base).unwrap();for(k,v)in row.as_object().unwrap(){value[k]=v.clone();}serde_json::from_value(value).unwrap()}).collect()}

#[test]
fn native_trade_zero_is_an_observation_not_a_consumption_rule(){
    let (_,domain)=baseline();let fixture=raw("villager-trades-contract.json");
    let rows:Vec<GeneticReferenceRow>=serde_json::from_value(fixture["records"].clone()).unwrap();
    let items:Vec<Item>=facts(&fixture["items"],&domain.items[0]);
    let map:BTreeMap<_,_>=items.iter().map(|i|(i.id.as_str(),i)).collect();
    assert!(fixture["records"][1]["data"]["inputs"][0]["sizes"].as_array().unwrap().contains(&json!(0)));
    domain::references::validate(&rows,&map).unwrap();
}

#[test]
fn native_cell_fault_is_preserved_as_a_fault_not_an_empty_cell(){
    let (_,domain)=baseline();let fixture=raw("native-usage-query-contract.json");
    let items:Vec<Item>=facts(&fixture["items"],&domain.items[0]);
    let map:BTreeMap<_,_>=items.iter().map(|i|(i.id.as_str(),i)).collect();
    let domains:Vec<Vec<GeneticReferenceRow>>=serde_json::from_value(fixture["domains"].clone()).unwrap();
    assert_eq!(fixture["domains"][0][3]["data"]["result"]["kind"],"cellFault");
    for rows in &domains{domain::references::validate(rows,&map).unwrap();}
}

#[test]
fn native_incubator_zero_keeps_exact_fluid_matching_and_no_debit(){
    let(source,mut domain)=baseline();let mut fixture=raw("binnie-genetics-incubator-semantic-contract.json");
    fixture["recipe"]["process"]["recipe"]["fluidInput"]["amount"]=json!(0);
    let c=&mut fixture["recipe"]["inputs"][1]["choices"][0];
    c["amount"]=json!("0");c["consume"]=json!({"kind":"keep"});c["rule"]=json!({"kind":"nativeZeroFluid","policy":"binnieIncubator"});
    let mut recipe:Recipe=serde_json::from_value(fixture["recipe"].clone()).unwrap();
    let items:Vec<Item>=facts(&fixture["items"],&domain.items[0]);let fluids:Vec<Fluid>=facts(&fixture["fluids"],&domain.fluids[0]);
    for mut item in items{if let Some(old)=domain.items.iter().find(|old|old.id==item.id){item.tags.extend(old.tags.clone());item.tags.sort();item.tags.dedup();}domain.items.retain(|old|old.id!=item.id);domain.items.push(item);}
    for fluid in fluids{domain.fluids.retain(|old|old.id!=fluid.id);domain.fluids.push(fluid);}
    let mut category=domain.categories[0].clone();category.source=recipe.source.clone();category.id=domain::origin_id("category",&category.source).unwrap();
    category.program=None;category.reference=None;category.icon=None;category.machines.clear();
    recipe.category=category.id.clone();recipe.id=domain::recipe_id(&recipe).unwrap();
    domain.categories.push(category);domain.recipes.push(recipe);
    domain.validate(&source).unwrap();
}
