use elysium_compiler_core::{domain::{self,Domain,Recipe,Item,Text},source::Source};
use serde_json::Value;
#[test]
fn actual_special_botania_recipes_are_typed_and_validate(){
 let root=std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../contracts/fixtures");
 for stem in ["botania-mini-special","botania-mini-priority","botania-head-special","botania-head-priority"]{
  let raw:Value=serde_json::from_slice(&std::fs::read(root.join(format!("{stem}-contract.json"))).unwrap()).unwrap();
  let source=Source::open(&root.join("source-semantic-40")).unwrap();let mut domain=Domain::load(&source).unwrap();
  let mut recipe:Recipe=serde_json::from_value(raw["recipe"].clone()).unwrap();
  for row in raw["items"].as_array().unwrap(){let mut value=serde_json::to_value(&domain.items[0]).unwrap();for(k,v)in row.as_object().unwrap(){value[k]=v.clone();}
   let item:Item=serde_json::from_value(value).unwrap();domain.items.retain(|old|old.id!=item.id);domain.items.push(item);}
  if let Some(strings)=raw["strings"].as_array(){for row in strings{let text:Text=serde_json::from_value(row.clone()).unwrap();domain.strings.retain(|old|old.id!=text.id);domain.strings.push(text);}}
  let mut category=domain.categories[0].clone();category.source=recipe.source.clone();category.id=domain::origin_id("category",&category.source).unwrap();category.program=None;category.reference=None;category.icon=None;category.machines.clear();
  recipe.category=category.id.clone();recipe.id=domain::recipe_id(&recipe).unwrap();domain.categories.push(category);domain.recipes.push(recipe);domain.validate(&source).unwrap();
  let index=domain.recipes.len()-1;let mut bad=serde_json::to_value(&domain.recipes[index]).unwrap();
  match stem {
   "botania-mini-special"=>bad["process"]["selection"]["coercions"][0]["value"]=serde_json::json!("unobserved key"),
   "botania-mini-priority"=>bad["process"]["selection"]["earlier"][0]["key"]=serde_json::json!("unobserved key"),
   "botania-head-special"=>bad["process"]["head"]["defaultNameKey"]=serde_json::json!("invented translation"),
   _=>bad["process"]["earlierHeads"][0]=serde_json::json!(99999),
  }
  let mut bad:Recipe=serde_json::from_value(bad).unwrap();bad.id=domain::recipe_id(&bad).unwrap();domain.recipes[index]=bad;
  assert!(domain.validate(&source).is_err(),"{stem}: changed native Botania law was accepted");
 }
}
