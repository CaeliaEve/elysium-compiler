//! Replay small rows emitted by the actual Java adapters through complete domain gates.
use super::*;
use serde_json::{json,Value};

#[test]
fn gt_native_probability_preserves_admission_slots_and_raw_arrays() {
    let mut fixture: Value = serde_json::from_str(include_str!("../../../../contracts/fixtures/gt-zero-fluid-contract.json")).unwrap();
    let native: Value = serde_json::from_str(include_str!("../../../../contracts/fixtures/gt-native-probability-facts.json")).unwrap();
    let choice = |stack: &Value, numerator, denominator| json!({"id":stack["id"],"amount":stack["amount"],
        "consume":{"kind":"chance","chance":{"numerator":numerator,"denominator":denominator}},
        "returns":[],"rule":{"kind":"wildcard","meta":false,"nbt":true}});
    fixture["recipe"]["gtNative"] = native.clone();
    fixture["recipe"]["inputs"] = json!([
        {"slot":0,"kind":"item","choices":[choice(&native["itemInputs"][0],"0","1")]},
        {"slot":2,"kind":"item","choices":[choice(&native["itemInputs"][2],"1","1")]},
        {"slot":0,"kind":"fluid","choices":[choice(&native["fluidAlternatives"][0][0],"1","2"),
            choice(&native["fluidAlternatives"][0][1],"1","2")]}
    ]);
    fixture["recipe"]["outputs"][0]["chance"] = json!({"numerator":"1","denominator":"10"});
    fixture["recipe"]["outputs"][1]["chance"] = json!({"numerator":"9","denominator":"10"});
    fixture["recipe"]["outputs"][2]["chance"] = json!({"numerator":"1","denominator":"4"});
    let (domain, source) = load(&fixture).unwrap();
    domain.validate(&source).unwrap();
    assert_eq!(serde_json::to_value(&domain.recipes[0]).unwrap()["gtNative"], native);
    let identity = recipe_id(&domain.recipes[0]).unwrap();
    let mut changed = domain.recipes[0].clone();
    changed.gt_native.as_mut().unwrap().fluid_input_chances = Some(vec![5000, 1]);
    assert_ne!(recipe_id(&changed).unwrap(), identity, "Unprojected native slot evidence must participate in recipe identity");
    changed = domain.recipes[0].clone();
    changed.gt_native.as_mut().unwrap().fake = true;
    changed.gt_native.as_mut().unwrap().policy.execution = gt_probability::GtExecution::ReferenceOnly;
    assert_ne!(recipe_id(&changed).unwrap(), identity, "Reference-only facts must not share executable recipe identity");
    let mut reference = fixture.clone();
    reference["recipe"]["gtNative"]["fake"] = json!(true);
    reference["recipe"]["gtNative"]["policy"]["execution"] = json!("referenceOnly");
    let (reference, source) = load(&reference).unwrap();
    reference.validate(&source).unwrap();
    assert_eq!(reference.recipes[0].inputs.len(), 3, "Reference facts lost their query/use edges");
    for (path, bad) in [
        ("/recipe/gtNative", Value::Null),
        ("/recipe/inputs/0/choices/0/amount", json!("1")),
        ("/recipe/inputs/0/choices/0/consume/kind", json!("keep")),
        ("/recipe/inputs/1/slot", json!(1)),
        ("/recipe/inputs/2/choices/1/amount", json!("1000")),
        ("/recipe/outputs/1/chance", json!({"numerator":"1","denominator":"1"})),
        ("/recipe/gtNative/inputChances", json!([-1])),
        ("/recipe/gtNative/fluidOutputChances", json!([10001])),
        ("/recipe/gtNative/fluidAlternatives", Value::Null),
        ("/recipe/gtNative/fluidAlternatives/1", Value::Null),
        ("/recipe/gtNative/fake", json!(true)),
        ("/recipe/gtNative/policy/execution", json!("exact")),
    ] {
        let mut changed = fixture.clone();
        *changed.pointer_mut(path).unwrap() = bad;
        assert!(load(&changed).and_then(|(domain, source)| domain.validate(&source)).is_err(), "{path}");
    }
}

#[test]
fn returned_native_workbench_rows_keep_logical_metadata_and_tool_consumption() {
    let mirror = include_str!("../../../../contracts/fixtures/native-metadata-mirror-contract.json");
    let value: Value = serde_json::from_str(mirror).unwrap();
    let recipe: Recipe = serde_json::from_value(value["recipe"].clone()).unwrap();
    assert_eq!(recipe.id, recipe_id(&recipe).unwrap());
    let (domain, source) = load(&value).unwrap();
    domain.validate(&source).unwrap();
    let input = &domain.recipes[0].inputs[0].choices[0];
    let item = domain.items.iter().find(|item| item.id == input.id).unwrap();
    assert_eq!(item.meta, 123, "The physical item identity became a logical getter value");
    assert!(matches!(input.rule, Match::NativeItemMetadata { value: 999, .. }));
    let items = domain.items.iter().map(|item| (item.id.as_str(), item)).collect();
    assert!(input.rule.indexes_example(item, &items));
    // A display anchor may fall outside a logical predicate; it must then lose
    // its reverse-use edge rather than having its physical metadata rewritten.
    let mut rejected = value.clone();
    rejected["recipe"]["inputs"][0]["choices"][0]["rule"]["value"] = json!(123);
    let (rejected, source) = load(&rejected).unwrap();
    rejected.validate(&source).unwrap();
    let rule = &rejected.recipes[0].inputs[0].choices[0].rule;
    assert!(!rule.indexes_example(item, &items));

    for raw in [
        include_str!("../../../../contracts/fixtures/native-blood-orb-tool-physical-contract.json"),
        include_str!("../../../../contracts/fixtures/native-blood-orb-tool-electric-contract.json"),
    ] {
        let value: Value = serde_json::from_str(raw).unwrap();
        let recipe: Recipe = serde_json::from_value(value["recipe"].clone()).unwrap();
        assert_eq!(recipe.id, recipe_id(&recipe).unwrap());
        let (domain, source) = load(&value).unwrap();
        domain.validate(&source).unwrap();
        let input = &domain.recipes[0].inputs[0].choices[0];
        assert!(matches!(input.consume, Consumption::GtTool { .. }));
        assert!(input.returns.is_empty());
        check_native("Blood Orb native version", raw, "/recipe/process/version", json!("1.7.51"));
    }
}

#[test]
fn native_gendustry_owned_catalogues_keep_dna_partitions_and_unobserved_results() {
    let fixtures=std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../contracts/fixtures");
    for name in ["imprinter-native-0","sampler-native-0","sampler-native-1","transposer-native-0",
        "transposer-native-1","mutatron-native-0","mutatron-locale","replicator-native-0","extractor-native-1","extractor-native-2"] {
        let mut id=None;
        for locale in ["en_US","zh_CN"] {
            let raw=std::fs::read_to_string(fixtures.join(format!("gendustry-{name}-{locale}-contract.json"))).unwrap();
            let value:Value=serde_json::from_str(&raw).unwrap();
            let recipe:Recipe=serde_json::from_value(value["recipe"].clone()).unwrap();
            assert_eq!(recipe.id,recipe_id(&recipe).unwrap());
            if let Some(ref previous)=id {assert_eq!(previous,&recipe.id);} else {id=Some(recipe.id.clone());}
            let(mut domain,source)=load(&value).unwrap();domain.recipes=vec![recipe];domain.validate(&source).unwrap();
            if name=="mutatron-locale" {
                let text=value["strings"].as_array().unwrap().iter().map(|row|row["text"].as_str().unwrap()).collect::<Vec<_>>().join(" ");
                assert!(text.contains(if locale=="zh_CN"{"原生生物群系条件"}else{"Native biome condition"}));
                check_native("Mutation display identity",&raw,"/recipe/process/roots/0/mutations/0/conditions",json!(["translated condition"]));
            }
            if name.starts_with("extractor") {
                check_native("DNA amount",&raw,"/recipe/outputs/0/amount",json!("1"));
                check_native("DNA exact blockers",&raw,"/recipe/inputs/0/choices/0/rule",json!({"kind":"wildcard","meta":true,"nbt":true}));
            }
        }
    }
}

#[test]
fn native_gendustry_template_keeps_original_identity_and_blank_sample_returns() {
    let raw=include_str!("../../../../contracts/fixtures/gendustry-template-gui-free-contract.json");
    let value:Value=serde_json::from_str(raw).unwrap();
    let recipe:Recipe=serde_json::from_value(value["recipe"].clone()).unwrap();
    assert_eq!(recipe.id,recipe_id(&recipe).unwrap());
    let translated:Recipe=serde_json::from_str::<Value>(include_str!("../../../../contracts/fixtures/gendustry-template-gui-free-zh_CN-contract.json"))
        .and_then(|v|serde_json::from_value(v["recipe"].clone())).unwrap();
    assert_eq!(recipe.id,translated.id);assert_eq!(translated.id,recipe_id(&translated).unwrap());
    let(mut domain,source)=load(&value).unwrap();domain.recipes=vec![recipe];
    domain.validate(&source).unwrap();
    for slot in [1,2] {
        check_native("Template sample container",raw,&format!("/recipe/inputs/{slot}/choices/0/returns"),json!([]));
    }
    check_native("Template native matching",raw,"/recipe/process",Value::Null);
    check_native("Template sample domain",raw,"/recipe/process/maximumSamples",json!(2));
    let old:Value=serde_json::from_str(include_str!("../../../../contracts/fixtures/gendustry-template-contract.json")).unwrap();
    assert!(load(&old).and_then(|(d,s)|d.validate(&s)).is_err(),"Historical missing-container sample was accepted as the corrected native rule");
}

#[test]
fn botania_pool_signed_mana_is_a_native_state_parameter_not_a_positive_duration() {
    let raw=include_str!("../../../../contracts/fixtures/botania-pool-native-0-en_US-contract.json");
    for mana in [i32::MIN,-50,0,i32::MAX] {
        let mut value:Value=serde_json::from_str(raw).unwrap();value["recipe"]["process"]["mana"]=json!(mana);
        let(domain,source)=load(&value).unwrap();domain.validate(&source).unwrap();
    }
    let root=std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../contracts/fixtures");
    for mana in [i32::MIN,-50,0,i32::MAX] {for state in 0..3 {
        let value:Value=serde_json::from_slice(&std::fs::read(root.join(format!("botania-pool-signed-{mana}-{state}-en_US-contract.json"))).unwrap()).unwrap();
        let translated:Value=serde_json::from_slice(&std::fs::read(root.join(format!("botania-pool-signed-{mana}-{state}-zh_CN-contract.json"))).unwrap()).unwrap();
        let recipe:Recipe=serde_json::from_value(value["recipe"].clone()).unwrap();
        assert_eq!(recipe.id,recipe_id(&recipe).unwrap());assert_eq!(recipe.id,translated["recipe"]["id"]);
        let(mut domain,source)=load(&value).unwrap();domain.recipes=vec![recipe];domain.validate(&source).unwrap();
    }}
}

#[test]
fn botania_native_readers_keep_typed_conditions_and_greedy_priority_without_gui() {
    for raw in [
        include_str!("../../../../contracts/fixtures/botania-pool-native-0-en_US-contract.json"),
        include_str!("../../../../contracts/fixtures/botania-pool-native-1-en_US-contract.json"),
        include_str!("../../../../contracts/fixtures/botania-pool-native-2-en_US-contract.json"),
        include_str!("../../../../contracts/fixtures/botania-pool-native-3-en_US-contract.json"),
        include_str!("../../../../contracts/fixtures/botania-runic-native-0-en_US-contract.json"),
        include_str!("../../../../contracts/fixtures/botania-runic-greedy-0-contract.json"),
        include_str!("../../../../contracts/fixtures/botania-runic-greedy-1-contract.json"),
        include_str!("../../../../contracts/fixtures/botania-runic-greedy-2-contract.json"),
    ] {
        let value:Value=serde_json::from_str(raw).unwrap();
        let recipe:Recipe=serde_json::from_value(value["recipe"].clone()).unwrap();
        assert_eq!(recipe.id,recipe_id(&recipe).unwrap());
        assert!(recipe.properties.is_empty());
        let(mut domain,source)=load(&value).unwrap();domain.recipes=vec![recipe];domain.validate(&source).unwrap();
        check_native("Botania typed process",raw,"/recipe/process",Value::Null);
        check_native("Botania no fixed duration",raw,"/recipe/duration",json!("60"));
        check_native("Botania pinned native version",raw,"/recipe/process/version",json!("unknown"));
        check_native("Botania no crafting containers",raw,"/recipe/inputs/0/choices/0/returns",json!([{"kind":"item","id":value["items"][0]["id"],"amount":"1"}]));
    }
}

#[test]
fn floating_native_transports_keep_all_36_count_branches_and_locale_independent_ids() {
    let root=std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../contracts/fixtures");
    for index in 0..36 {
        let value:Value=serde_json::from_slice(&std::fs::read(root.join(format!("botania-floating-native-{index}-en_US-contract.json"))).unwrap()).unwrap();
        let translated:Value=serde_json::from_slice(&std::fs::read(root.join(format!("botania-floating-native-{index}-zh_CN-contract.json"))).unwrap()).unwrap();
        let recipe:Recipe=serde_json::from_value(value["recipe"].clone()).unwrap();
        assert_eq!(recipe.id,recipe_id(&recipe).unwrap());assert_eq!(recipe.id,translated["recipe"]["id"]);
        let(mut domain,source)=load(&value).unwrap();domain.recipes=vec![recipe];domain.validate(&source).unwrap();
        let mut generic=value;generic["recipe"]["process"]=Value::Null;
        generic["recipe"]["outputs"][0]["change"]=Value::Null;
        for input in generic["recipe"]["inputs"].as_array_mut().unwrap(){for choice in input["choices"].as_array_mut().unwrap(){choice["rule"]=json!({"kind":"exact"});}}
        assert!(load(&generic).and_then(|(d,s)|d.validate(&s)).is_err(),"Native flower handler accepted a generic fixed craft");
    }
}

#[test]
fn native_inscriber_readers_preserve_java_identity_and_require_precise_machine_semantics() {
    for (en,zh) in [
        (include_str!("../../../../contracts/fixtures/inscriber-native-0-en_US-contract.json"),include_str!("../../../../contracts/fixtures/inscriber-native-0-zh_CN-contract.json")),
        (include_str!("../../../../contracts/fixtures/inscriber-native-1-en_US-contract.json"),include_str!("../../../../contracts/fixtures/inscriber-native-1-zh_CN-contract.json")),
        (include_str!("../../../../contracts/fixtures/inscriber-native-2-en_US-contract.json"),include_str!("../../../../contracts/fixtures/inscriber-native-2-zh_CN-contract.json"))
    ] {
        let value:Value=serde_json::from_str(en).unwrap();let translated:Value=serde_json::from_str(zh).unwrap();
        let recipe:Recipe=serde_json::from_value(value["recipe"].clone()).unwrap();
        assert_eq!(recipe.id,recipe_id(&recipe).unwrap());assert_eq!(recipe.id,translated["recipe"]["id"]);
        let(mut domain,source)=load(&value).unwrap();domain.recipes=vec![recipe];domain.validate(&source).unwrap();
        let mut generic=value.clone();generic["recipe"]["process"]=Value::Null;
        for input in generic["recipe"]["inputs"].as_array_mut().unwrap(){
            for choice in input["choices"].as_array_mut().unwrap(){choice["rule"]=json!({"kind":"exact"});}
        }
        assert!(load(&generic).and_then(|(d,s)|d.validate(&s)).is_err(),"Native inscriber accepted generic crafting semantics");
        check_native("inscriber native plate mode",en,"/recipe/process/mode",json!("invalid"));
        check_native("inscriber precise input",en,"/recipe/inputs/0/choices/0/rule",json!({"kind":"exact"}));
    }
}

#[test]
fn native_ae_reader_transports_preserve_original_java_ids_cells_and_alternatives() {
    for raw in [include_str!("../../../../contracts/fixtures/ae-native-shaped-contract.json"),
        include_str!("../../../../contracts/fixtures/ae-native-shapeless-contract.json"),
        include_str!("../../../../contracts/fixtures/ae-native-wireless-contract.json"),
        include_str!("../../../../contracts/fixtures/ae-native-greedy-contract.json")] {
        let value:Value=serde_json::from_str(raw).unwrap();
        let recipe:Recipe=serde_json::from_value(value["recipe"].clone()).unwrap();
        assert_eq!(recipe.id,recipe_id(&recipe).unwrap());
        assert_eq!(recipe.inputs.len(),if recipe.source.key=="ae-native-greedy"{2}else{3});
        assert_eq!(recipe.inputs[0].choices.len(),2);
        let (mut domain,source)=load(&value).unwrap();domain.recipes=vec![recipe];
        domain.validate(&source).unwrap();
        check_native("AE native greedy selection",raw,"/recipe/process",Value::Null);
        check_native("AE input ignores NBT",raw,"/recipe/inputs/0/choices/0/rule/nbt",json!(false));
        check_native("AE exact native matcher",raw,"/recipe/process/matching",json!("genericPermutation"));
    }
}

#[test]
fn native_binnie_semantic_rows_preserve_original_java_ids_and_nine_machine_processes() {
    for raw in [include_str!("../../../../contracts/fixtures/binnie-genetics-acclimatiser-batch-contract.json"),
        include_str!("../../../../contracts/fixtures/binnie-genetics-acclimatiser-semantic-contract.json"),
        include_str!("../../../../contracts/fixtures/binnie-genetics-sequencer-semantic-contract.json"),
        include_str!("../../../../contracts/fixtures/binnie-genetics-analyser-semantic-contract.json"),
        include_str!("../../../../contracts/fixtures/binnie-genetics-splicer-semantic-contract.json"),
        include_str!("../../../../contracts/fixtures/binnie-genetics-isolator-semantic-contract.json"),
        include_str!("../../../../contracts/fixtures/binnie-genetics-inoculator-semantic-contract.json"),
        include_str!("../../../../contracts/fixtures/binnie-genetics-genepool-semantic-contract.json"),
        include_str!("../../../../contracts/fixtures/binnie-genetics-incubator-semantic-contract.json"),
        include_str!("../../../../contracts/fixtures/binnie-genetics-polymeriser-semantic-contract.json")] {
        let value:Value=serde_json::from_str(raw).unwrap();
        let recipe:Recipe=serde_json::from_value(value["recipe"].clone()).unwrap();
        assert_eq!(recipe.id,recipe_id(&recipe).unwrap(),"{}",recipe.source.handler);
        let(mut domain,source)=load(&value).unwrap();domain.recipes=vec![recipe];domain.validate(&source).unwrap();
        check_native("Native genetics resource law",raw,"/recipe/process",Value::Null);
    }
}

#[test]
fn native_blood_semantic_readers_keep_network_charging_and_world_rituals_without_gui() {
    for raw in [include_str!("../../../../contracts/fixtures/blood-altar-semantic-contract.json"),
        include_str!("../../../../contracts/fixtures/blood-binding-semantic-contract.json"),
        include_str!("../../../../contracts/fixtures/blood-meteor-semantic-contract.json"),
        include_str!("../../../../contracts/fixtures/blood-charging-semantic-contract.json")] {
        let value:Value=serde_json::from_str(raw).unwrap();
        let recipe:Recipe=serde_json::from_value(value["recipe"].clone()).unwrap();
        assert_eq!(recipe.id,recipe_id(&recipe).unwrap());
        let (mut domain,source)=load(&value).unwrap();domain.recipes=vec![recipe];domain.validate(&source).unwrap();
        check_native("Blood native process",raw,"/recipe/process",Value::Null);
        if value["recipe"]["process"]["canBeFilled"]==json!(true) {
            assert!(value["recipe"]["outputs"].as_array().unwrap().is_empty());
            check_native("Charging keeps orb",raw,"/recipe/inputs/0/choices/0/consume/kind",json!("consume"));
            check_native("Charging requires owner",raw,"/recipe/process/orb/requireOwner",json!(false));
        }
    }
}

#[test]
fn native_lumbermill_preserves_runtime_admission_tank_volume_and_conditional_merge(){
    let en:Value=serde_json::from_str(include_str!("../../../../contracts/fixtures/binnie-lumbermill-semantic-contract.json")).unwrap();
    let zh:Value=serde_json::from_str(include_str!("../../../../contracts/fixtures/binnie-lumbermill-semantic-zh_CN-contract.json")).unwrap();
    let translated:Recipe=serde_json::from_value(zh["recipe"].clone()).unwrap();assert_eq!(en["recipe"]["id"],translated.id);assert_eq!(translated.id,recipe_id(&translated).unwrap());
    assert_ne!(en["recipe"]["notes"][0]["text"],zh["recipe"]["notes"][0]["text"]);
    for raw in [include_str!("../../../../contracts/fixtures/binnie-lumbermill-semantic-contract.json"),
        include_str!("../../../../contracts/fixtures/native-lumbermill-lazy-contract.json"),
        include_str!("../../../../contracts/fixtures/native-lumbermill-saved-input-contract.json")]{
        let value:Value=serde_json::from_str(raw).unwrap();let direct:Recipe=serde_json::from_value(value["recipe"].clone()).unwrap();
        assert_eq!(direct.id,recipe_id(&direct).unwrap());let(mut d,s)=load(&value).unwrap();d.recipes=vec![direct];d.validate(&s).unwrap();
        for(path,bad)in[("/recipe/process",Value::Null),("/recipe/inputs/1/choices/0/rule",json!({"kind":"exact"})),
            ("/recipe/inputs/1/choices/0/consume",json!({"kind":"consume"})),("/recipe/process/workTicks",json!(31)),
            ("/recipe/outputs/0/quantity/condition",json!("always"))]{check_native("Lumbermill runtime",raw,path,bad);}
        check_native("Lumbermill byproduct role",raw,"/recipe/outputs/1/id",value["recipe"]["outputs"][2]["id"].clone());
    }
}

#[test]
fn native_enchanter_pickup_keeps_count_intervals_whole_book_slot_and_signed_xp(){
    let en:Value=serde_json::from_str(include_str!("../../../../contracts/fixtures/native-enchanter-level-2-contract.json")).unwrap();
    let zh:Value=serde_json::from_str(include_str!("../../../../contracts/fixtures/native-enchanter-level-2-zh_CN-contract.json")).unwrap();
    let translated:Recipe=serde_json::from_value(zh["recipe"].clone()).unwrap();
    assert_eq!(en["recipe"]["id"],translated.id);assert_eq!(translated.id,recipe_id(&translated).unwrap());
    assert_ne!(en["recipe"]["notes"][0]["text"],zh["recipe"]["notes"][0]["text"]);
    for raw in [include_str!("../../../../contracts/fixtures/native-enchanter-level-1-contract.json"),
        include_str!("../../../../contracts/fixtures/native-enchanter-level-2-contract.json"),
        include_str!("../../../../contracts/fixtures/native-enchanter-level-5-contract.json"),
        include_str!("../../../../contracts/fixtures/native-enchanter-stored-divisor-contract.json"),
        include_str!("../../../../contracts/fixtures/native-enchanter-stored-65-contract.json"),
        include_str!("../../../../contracts/fixtures/native-enchanter-ore-contract.json"),
        include_str!("../../../../contracts/fixtures/native-enchanter-ore-alias-contract.json"),
        include_str!("../../../../contracts/fixtures/native-enchanter-signed-fffffff9-contract.json"),
        include_str!("../../../../contracts/fixtures/native-enchanter-signed-7fffffff-contract.json"),
        include_str!("../../../../contracts/fixtures/native-enchanter-signed-80000000-contract.json")]{
        let value:Value=serde_json::from_str(raw).unwrap();let direct:Recipe=serde_json::from_value(value["recipe"].clone()).unwrap();
        assert_eq!(direct.id,recipe_id(&direct).unwrap());let(mut d,s)=load(&value).unwrap();d.recipes=vec![direct];d.validate(&s).unwrap();
        for(path,bad)in[("/recipe/process",Value::Null),("/recipe/process/bookPolicy",json!("consumeOneWritableBook")),
            ("/recipe/process/offeredMaximum",json!("0")),("/recipe/inputs/0/choices/0/consume",json!({"kind":"consume"})),
            ("/recipe/outputs/0/quantity/condition",json!("always")),("/recipe/process/cost",json!(1234567))]{check_native("Enchanter pickup",raw,path,bad);}
    }
}

#[test]
fn native_creativecore_keeps_stock_thresholds_baked_ores_and_ordered_equality(){
    for raw in [include_str!("../../../../contracts/fixtures/creativecore-owned-shape-contract.json"),
        include_str!("../../../../contracts/fixtures/creativecore-owned-stock-contract.json"),
        include_str!("../../../../contracts/fixtures/creativecore-owned-greedy-contract.json"),
        include_str!("../../../../contracts/fixtures/creativecore-owned-ore-contract.json"),
        include_str!("../../../../contracts/fixtures/creativecore-owned-stackinfoblock-contract.json"),
        include_str!("../../../../contracts/fixtures/creativecore-owned-stackinfomaterial-contract.json"),
        include_str!("../../../../contracts/fixtures/creativecore-owned-remove-equals-contract.json"),
        include_str!("../../../../contracts/fixtures/creativecore-owned-tail-contract.json")]{
        let value:Value=serde_json::from_str(raw).unwrap();let direct:Recipe=serde_json::from_value(value["recipe"].clone()).unwrap();
        assert_eq!(direct.id,recipe_id(&direct).unwrap());let(mut d,s)=load(&value).unwrap();d.recipes=vec![direct];d.validate(&s).unwrap();
        for(path,bad)in[("/recipe/process",Value::Null),("/recipe/inputs/0/choices/0/rule/input",json!(99)),
            ("/recipe/inputs/0/choices/0/amount",json!("3")),("/recipe/process/consumption",json!("consumeMinimum")),
            ("/recipe/outputs/0/chance/numerator",json!("0"))]{check_native("CreativeCore",raw,path,bad);}
    }
    let value:Value=serde_json::from_str(include_str!("../../../../contracts/fixtures/creativecore-owned-empty-contract.json")).unwrap();
    let direct:Recipe=serde_json::from_value(value["recipe"].clone()).unwrap();assert_eq!(direct.id,recipe_id(&direct).unwrap());
    let(mut d,s)=load(&value).unwrap();d.recipes=vec![direct];d.validate(&s).unwrap();
    let Some(Process::CreativeCoreCrafting(p))=&d.recipes[0].process else{panic!("missing law")};assert_eq!(creativecore::select(p,[None;9]).unwrap(),Some([None;9]));
    let value:Value=serde_json::from_str(include_str!("../../../../contracts/fixtures/creativecore-owned-remove-equals-contract.json")).unwrap();
    let recipe:Recipe=serde_json::from_value(value["recipe"].clone()).unwrap();let(d,_)=load(&value).unwrap();
    let Some(Process::CreativeCoreCrafting(p))=&recipe.process else{panic!("missing law")};
    let b=d.items.iter().find(|item|item.registry=="minecraft:paper"&&matches!(&item.nbt,Some(crate::identity::Nbt::Compound{value})if matches!(value.get("marker"),Some(crate::identity::Nbt::String{value})if value=="b"))).unwrap();
    let mut offered=[None;9];offered[0]=Some((b,1));offered[1]=Some((b,1));let result=creativecore::select(p,offered).unwrap().unwrap();assert_eq!(&result[..2],&[Some(0),Some(1)]);
    let en:Value=serde_json::from_str(include_str!("../../../../contracts/fixtures/creativecore-owned-stock-contract.json")).unwrap();
    let zh:Value=serde_json::from_str(include_str!("../../../../contracts/fixtures/creativecore-owned-stock-zh_CN-contract.json")).unwrap();
    let translated:Recipe=serde_json::from_value(zh["recipe"].clone()).unwrap();assert_eq!(en["recipe"]["id"],translated.id);assert_eq!(translated.id,recipe_id(&translated).unwrap());
    assert_ne!(en["recipe"]["notes"][0]["text"],zh["recipe"]["notes"][0]["text"]);
}

#[test]
fn native_distillery_preserves_swapped_allocation_priority_and_conditional_inventory_outputs(){
    for raw in [include_str!("../../../../contracts/fixtures/native-distillery-production-jar-contract.json"),
        include_str!("../../../../contracts/fixtures/native-distillery-empty-slot-contract.json"),
        include_str!("../../../../contracts/fixtures/native-distillery-zero-output-contract.json"),
        include_str!("../../../../contracts/fixtures/native-distillery-negative-output-contract.json"),
        include_str!("../../../../contracts/fixtures/native-distillery-signed-jars-contract.json")]{
        let value:Value=serde_json::from_str(raw).unwrap();let direct:Recipe=serde_json::from_value(value["recipe"].clone()).unwrap();
        assert_eq!(direct.id,recipe_id(&direct).unwrap());let(mut domain,source)=load(&value).unwrap();domain.recipes=vec![direct];domain.validate(&source).unwrap();
    }
    for raw in [include_str!("../../../../contracts/fixtures/native-distillery-contract.json"),
        include_str!("../../../../contracts/fixtures/native-distillery-priority-contract.json")]{
        let value:Value=serde_json::from_str(raw).unwrap();let direct:Recipe=serde_json::from_value(value["recipe"].clone()).unwrap();
        assert_eq!(direct.id,recipe_id(&direct).unwrap());let(mut domain,source)=load(&value).unwrap();domain.recipes=vec![direct];domain.validate(&source).unwrap();
        let Some(Process::WitcheryDistillery(p))=&domain.recipes[0].process else{panic!("native law missing")};
        let items=domain.items.iter().map(|item|(item.id.as_str(),item)).collect();
        let first=domain.items.iter().find(|item|item.id==p.reagents[0].as_ref().unwrap().item).unwrap();
        let second=domain.items.iter().find(|item|item.id==p.reagents[1].as_ref().unwrap().item).unwrap();
        assert!(distillery::selected(p,[Some(second),Some(first)],Some(p.jars),&items));
        if !p.earlier.is_empty(){assert!(!distillery::selected(p,[Some(second),Some(first)],Some(i32::MAX),&items));}
        assert!(!distillery::selected(p,[Some(first),Some(second)],None,&items));
        let prototype=items[p.outputs[0].as_ref().unwrap().id.as_str()];
        let count:i32=p.outputs[0].as_ref().unwrap().amount.parse().unwrap();
        assert_eq!(distillery::placed_count(Some((first,5,64)),prototype,count),Some(5));
        assert_eq!(distillery::placed_count(Some((prototype,1,64)),prototype,count),Some(1+count));
        assert_eq!(distillery::placed_count(Some((prototype,64,64)),prototype,count),None);
        for(path,bad)in[("/recipe/process",Value::Null),("/recipe/process/cookTicks",json!(200)),
          ("/recipe/process/powerPerWorkTickBits",json!("3f199999")),("/recipe/duration",json!("800")),
          ("/recipe/inputs/0/choices/0/consume",json!({"kind":"consume"})),("/recipe/outputs/0/amount",json!("3"))]{check_native("Distillery",raw,path,bad);}
    }
    let en:Value=serde_json::from_str(include_str!("../../../../contracts/fixtures/native-distillery-contract.json")).unwrap();
    let zh:Value=serde_json::from_str(include_str!("../../../../contracts/fixtures/native-distillery-zh_CN-contract.json")).unwrap();
    let row:Recipe=serde_json::from_value(zh["recipe"].clone()).unwrap();assert_eq!(row.id,recipe_id(&row).unwrap());assert_eq!(en["recipe"]["id"],zh["recipe"]["id"]);
}

#[test]
fn native_chisel_gui_free_transport_preserves_conditional_tool_and_native_partitions(){
    for raw in [include_str!("../../../../contracts/fixtures/chisel-semantic-contract.json"),
        include_str!("../../../../contracts/fixtures/chisel-ore-single-semantic-contract.json"),
        include_str!("../../../../contracts/fixtures/chisel-ore-stack-semantic-contract.json"),
        include_str!("../../../../contracts/fixtures/chisel-wildcard-semantic-contract.json"),
        include_str!("../../../../contracts/fixtures/chisel-literal-metadata-semantic-contract.json")]{
        let value:Value=serde_json::from_str(raw).unwrap();let row:Recipe=serde_json::from_value(value["recipe"].clone()).unwrap();
        assert_eq!(row.id,recipe_id(&row).unwrap());let(mut d,s)=load(&value).unwrap();d.recipes=vec![row];d.validate(&s).unwrap();
        assert_eq!(d.recipes[0].inputs.len(),2);
        if let Some(Process::Chisel(process)) = &d.recipes[0].process {
            let items = d.items.iter().map(|item| (item.id.as_str(), item)).collect();
            if process.sources.iter().any(|source| !source.native_wildcard && d.items.iter().any(|item| item.id == source.item && item.meta == 32767)) {
                let mut offered = d.items.iter().find(|item| item.id == process.sources[0].item).unwrap().clone();
                assert!(chisel::accepts_source(process, &offered, &items));
                offered.meta = 0;
                assert!(!chisel::accepts_source(process, &offered, &items));
            }
        }
    }
    let en:Value=serde_json::from_str(include_str!("../../../../contracts/fixtures/chisel-semantic-contract.json")).unwrap();
    let zh:Value=serde_json::from_str(include_str!("../../../../contracts/fixtures/chisel-semantic-zh_CN-contract.json")).unwrap();
    let row:Recipe=serde_json::from_value(zh["recipe"].clone()).unwrap();
    assert_eq!(row.id,recipe_id(&row).unwrap());assert_eq!(en["recipe"]["id"],zh["recipe"]["id"]);
}

#[test]
fn native_refinery_transport_keeps_normalization_countdown_and_drain_before_rejected_fill(){
    for raw in [include_str!("../../../../contracts/fixtures/native-refinery-contract.json"),
        include_str!("../../../../contracts/fixtures/native-refinery-alternate-contract.json"),
        include_str!("../../../../contracts/fixtures/native-refinery-legacy-contract.json"),
        include_str!("../../../../contracts/fixtures/native-refinery-legacy-alternate-contract.json")]{
        let value:Value=serde_json::from_str(raw).unwrap();let direct:Recipe=serde_json::from_value(value["recipe"].clone()).unwrap();
        assert_eq!(direct.id,recipe_id(&direct).unwrap());let(mut d,s)=load(&value).unwrap();d.recipes=vec![direct];d.validate(&s).unwrap();
        for(path,bad)in[("/recipe/process",Value::Null),("/recipe/process/normalizedOil",json!("missing")),
            ("/recipe/process/tankCapacity",json!(24001)),("/recipe/process/countdownTicks",json!(3)),
            ("/recipe/process/maxExtractGj",json!(20)),("/recipe/process/outputPolicy",json!("alwaysProducesOne")),
            ("/recipe/inputs/0/choices/0/rule/nbt",json!(false)),("/recipe/outputs/0/amount",json!("1")),
            ("/recipe/outputs/0/quantity/condition",json!("always")),("/recipe/duration",json!("2"))]{check_native("Refinery",raw,path,bad);}
    }
    let en:Value=serde_json::from_str(include_str!("../../../../contracts/fixtures/native-refinery-contract.json")).unwrap();
    let zh:Value=serde_json::from_str(include_str!("../../../../contracts/fixtures/native-refinery-zh_CN-contract.json")).unwrap();
    let translated:Recipe=serde_json::from_value(zh["recipe"].clone()).unwrap();assert_eq!(translated.id,recipe_id(&translated).unwrap());assert_eq!(en["recipe"]["id"],zh["recipe"]["id"]);
}

#[test]
fn native_circuit_transport_keeps_five_slots_raw_quick_capacity_and_drop_law(){
    for raw in [include_str!("../../../../contracts/fixtures/native-circuit-law-contract.json"),
        include_str!("../../../../contracts/fixtures/native-circuit-quick-contract.json"),
        include_str!("../../../../contracts/fixtures/native-circuit-amunra-contract.json"),
        include_str!("../../../../contracts/fixtures/native-circuit-empty-contract.json")]{
        let value:Value=serde_json::from_str(raw).unwrap();
        let direct:Recipe=serde_json::from_value(value["recipe"].clone()).unwrap();assert_eq!(direct.id,recipe_id(&direct).unwrap());
        let(mut domain,source)=load(&value).unwrap();domain.recipes=vec![direct];domain.validate(&source).unwrap();
        for(path,bad)in[("/recipe/process",Value::Null),("/recipe/process/slots/4",json!("missing")),
            ("/recipe/process/outputAmount",json!("999")),("/recipe/process/processingTicks",json!(200)),
            ("/recipe/process/maxExtractGj",json!(99)),("/recipe/process/capacity",json!("adjustedCount")),
            ("/recipe/process/overflow",json!("copiesPrototypeTags")),("/recipe/duration",json!("300")),
            ("/recipe/outputs/0/slot",json!(0)),("/recipe/outputs/0/amount",json!("5")),
            ("/recipe/outputs/0/quantity/condition",json!("always")),("/recipe/notes/1/source",json!("unknown"))]{
            check_native("Circuit fabricator",raw,path,bad);
        }
        if !domain.recipes[0].inputs.is_empty(){
            check_native("Circuit literal metadata",raw,"/recipe/inputs/0/choices/0/rule/meta",json!(true));
            check_native("Circuit consume one",raw,"/recipe/inputs/0/choices/0/amount",json!("2"));
        }
    }
    let en:Value=serde_json::from_str(include_str!("../../../../contracts/fixtures/native-circuit-law-contract.json")).unwrap();
    let zh:Value=serde_json::from_str(include_str!("../../../../contracts/fixtures/native-circuit-law-zh_CN-contract.json")).unwrap();
    let translated:Recipe=serde_json::from_value(zh["recipe"].clone()).unwrap();assert_eq!(translated.id,recipe_id(&translated).unwrap());
    assert_eq!(en["recipe"]["id"],zh["recipe"]["id"]);
    let old:Value=serde_json::from_str(include_str!("../../../../contracts/fixtures/native-circuit-gui-free-contract.json")).unwrap();
    assert!(load(&old).and_then(|(d,s)|d.validate(&s)).is_err(),"Generic circuit shape must not bypass typed family gate");
}

#[test]
fn native_solar_transport_preserves_total_energy_priority_zero_presence_and_merge_state(){
    for raw in [include_str!("../../../../contracts/fixtures/native-solar-contract.json"),include_str!("../../../../contracts/fixtures/native-solar-zero-contract.json")]{
        let value:Value=serde_json::from_str(raw).unwrap();let direct:Recipe=serde_json::from_value(value["recipe"].clone()).unwrap();
        assert_eq!(direct.id,recipe_id(&direct).unwrap());let(mut domain,source)=load(&value).unwrap();domain.recipes=vec![direct];domain.validate(&source).unwrap();
        for(path,bad)in[("/recipe/process",Value::Null),("/recipe/process/energyPerOperation",json!(-1)),
            ("/recipe/process/inputAmount",json!("999")),("/recipe/inputs/0/choices/0/rule/meta",json!(true)),
            ("/recipe/outputs/0/quantity/condition",json!("always")),("/recipe/outputs/0/quantity/nominal",json!("999")),
            ("/recipe/duration",json!("100")),("/recipe/notes/0/source",json!("unknown"))]{check_native("Molecular transformer",raw,path,bad);}
    }
    let english:Value=serde_json::from_str(include_str!("../../../../contracts/fixtures/native-solar-contract.json")).unwrap();
    let chinese:Value=serde_json::from_str(include_str!("../../../../contracts/fixtures/native-solar-zh_CN-contract.json")).unwrap();
    let en:Recipe=serde_json::from_value(english["recipe"].clone()).unwrap();let zh:Recipe=serde_json::from_value(chinese["recipe"].clone()).unwrap();
    assert_eq!(en.id,zh.id);assert_eq!(zh.id,recipe_id(&zh).unwrap());assert_ne!(en.notes[0].text,zh.notes[0].text);
}

#[test]
fn native_decay_saved_state_transport_keeps_conditional_result_and_raw_locale_identity(){
    let raw=include_str!("../../../../contracts/fixtures/native-decay-state-contract.json");
    let value:Value=serde_json::from_str(raw).unwrap();let direct:Recipe=serde_json::from_value(value["recipe"].clone()).unwrap();
    assert_eq!(direct.id,recipe_id(&direct).unwrap());
    let(mut domain,source)=load(&value).unwrap();domain.recipes=vec![direct.clone()];domain.validate(&source).unwrap();
    assert!(direct.outputs[0].amount.is_none());
    let chinese:Value=serde_json::from_str(include_str!("../../../../contracts/fixtures/native-decay-state-zh_CN-contract.json")).unwrap();
    let translated:Recipe=serde_json::from_value(chinese["recipe"].clone()).unwrap();assert_eq!(translated.id,direct.id);assert_eq!(translated.id,recipe_id(&translated).unwrap());
    assert_ne!(translated.notes[0].text,direct.notes[0].text);
    let(mut translated_domain,translated_source)=load(&chinese).unwrap();translated_domain.recipes=vec![translated];translated_domain.validate(&translated_source).unwrap();
    for(path,bad)in[("/recipe/process/stateLaw",Value::Null),("/recipe/process/scope",json!("freshUntagged")),
        ("/recipe/inputs/0/choices/0/rule/nbt",json!(false)),("/recipe/outputs/0/quantity/condition",json!("always")),
        ("/recipe/outputs/0/quantity/nominal",json!("2")),("/recipe/outputs/0/quantity",Value::Null)]{
        check_native("Decay saved native state law",raw,path,bad);
    }
}

#[test]
fn native_decay_fresh_transport_keeps_batch_process_and_locale_independent_identity(){
    let value:Value=serde_json::from_str(include_str!("../../../../contracts/fixtures/native-decay-fresh-contract.json")).unwrap();
    let direct:Recipe=serde_json::from_value(value["recipe"].clone()).unwrap();
    assert_eq!(direct.id,recipe_id(&direct).unwrap(),"Native decay row must not need consumer-side identity repair");
    let(mut domain,source)=load(&value).unwrap();domain.recipes=vec![direct];domain.validate(&source).unwrap();
    let row=&domain.recipes[0];assert!(row.duration.is_none());
    assert!(matches!(row.inputs[0].choices[0].consume,Consumption::Stack));assert_eq!(row.outputs[0].amount.as_deref(),Some("1"));
    assert!(row.properties.is_empty());assert_eq!(row.notes.len(),4);
    let Some(Process::DecayChest(process))=&row.process else{panic!()};assert_eq!(process.fresh_batches,"107374183");
    let chinese:Value=serde_json::from_str(include_str!("../../../../contracts/fixtures/native-decay-fresh-zh_CN-contract.json")).unwrap();
    let translated:Recipe=serde_json::from_value(chinese["recipe"].clone()).unwrap();
    assert_eq!(translated.id,recipe_id(&translated).unwrap());assert_eq!(translated.id,row.id);assert_ne!(translated.notes[0].text,row.notes[0].text);
    let(mut translated_domain,translated_source)=load(&chinese).unwrap();translated_domain.recipes=vec![translated];translated_domain.validate(&translated_source).unwrap();
    let raw=include_str!("../../../../contracts/fixtures/native-decay-fresh-contract.json");
    for(path,bad)in[("/recipe/process",Value::Null),("/recipe/process/maxTicks",json!(0)),
        ("/recipe/process/freshBatches",json!("107374182")),("/recipe/process/batchSteps",json!(21)),
        ("/recipe/inputs/0/choices/0/consume",json!({"kind":"consume"})),("/recipe/outputs/0/amount",json!("2")),
        ("/recipe/duration",json!("2147483647")),("/recipe/inputs/0/choices/0/rule/nbt",json!(true))]{check_native("Decay native fresh scope",raw,path,bad);}
    check_native("Decay source note",raw,"/recipe/notes/0/source",json!("unrelated"));
}

#[test]
fn native_fmp_external_saw_does_not_replace_cached_registry_maximum(){
    let raw=include_str!("../../../../contracts/fixtures/fmp-stronger-saw-contract.json");
    let value:Value=serde_json::from_str(raw).unwrap();let(domain,source)=load(&value).unwrap();domain.validate(&source).unwrap();
    let Some(Process::FmpMicroblock(p))=&domain.recipes[0].process else{panic!()};
    let programs=programs::reconstruct(&domain.programs).unwrap();let programs::SharedProgram::Fmp(rules)=&programs[&p.program]else{panic!()};
    assert!(rules.saws.iter().any(|saw|saw.max_strength>rules.settings().max_saw_strength));
    check_native("FMP native cached strength",raw,"/recipe/process/materialId",json!(1));
}

#[test]
fn native_ic2_advanced_preserves_compact_masks_and_double_result_states(){
    for raw in [include_str!("../../../../contracts/fixtures/native-ic2-advanced-shaped-contract.json"),include_str!("../../../../contracts/fixtures/native-ic2-advanced-electric-contract.json")] {
    let value:Value=serde_json::from_str(raw).unwrap();let(domain,source)=load(&value).unwrap();domain.validate(&source).unwrap();
    for(path,bad)in[("/recipe/process",Value::Null),
        ("/recipe/process/visibility/hidden",json!(true)),("/recipe/process/charges/0/0",json!("0")),
        ("/recipe/process/states/0/positions/1",json!(0)),("/recipe/inputs/0/choices/0/amount",json!("7")),
        ("/recipe/process/scope",json!("arbitraryInventory"))]{check_native("IC2 Advanced",raw,path,bad);}
    if !value["recipe"]["grid"].is_null(){check_native("IC2 compact mask",raw,"/recipe/process/layout/masks/0",json!(1));check_native("IC2 logical holes",raw,"/recipe/grid/cells/4",Value::Null);}
    let mut missing=value.clone();missing["recipe"]["process"]["states"].as_array_mut().unwrap().pop();missing["recipe"]["outputs"][0]["change"]["samples"].as_array_mut().unwrap().pop();assert!(load(&missing).and_then(|(domain,source)|domain.validate(&source)).is_err(),"IC2 finite coverage omitted a reachable double state");
    }
}

#[test]
fn native_galaxy_assembly_preserves_repair_interception() {
    let raw=include_str!("../../../../contracts/fixtures/native-galaxy-repair-contract.json");
    for(path,bad)in[("/recipe/process",Value::Null),
        ("/recipe/process/repair",json!([])),
        ("/recipe/process/repair/0/item",json!("missing")),
        ("/recipe/process/repair/0/maximum",json!(0)),
        ("/recipe/inputs/0/choices/0/amount",json!("2")),
        ("/recipe/source/handler",json!("codechicken.nei.recipe.ShapedRecipeHandler")),
        ("/recipe/duration",json!("20"))] {
        check_native("Galaxy assembly repair",raw,path,bad);
    }
}

#[test]
fn native_qed_nbt_keeps_flux_priority_and_last_tag_transfer() {
    let raw=include_str!("../../../../contracts/fixtures/native-qed-nbt-contract.json");
    for(path,bad)in[("/recipe/process",Value::Null), ("/recipe/process/copyNbt",json!(false)),
        ("/recipe/outputs/0/change",Value::Null),
        ("/recipe/process/enderFlux",json!("1")),
        ("/recipe/inputs/0/choices/0/returns",json!([{"kind":"item","id":"missing","amount":"1"}])),
        ("/recipe/source/handler",json!("codechicken.nei.recipe.ShapedRecipeHandler"))] {
        check_native("QED NBT",raw,path,bad);
    }
}

#[test]
fn native_extreme_registry_keeps_nine_grid_priorities_and_repair() {
    for raw in [include_str!("../../../../contracts/fixtures/native-extreme-shaped-contract.json"),
        include_str!("../../../../contracts/fixtures/native-extreme-shapeless-contract.json"),
        include_str!("../../../../contracts/fixtures/native-extreme-priority-contract.json"),
        include_str!("../../../../contracts/fixtures/native-extreme-repair-contract.json")] {
        let value:Value=serde_json::from_str(raw).unwrap();
        let (domain,source)=load(&value).unwrap();domain.validate(&source).unwrap();
        for (path,bad) in [("/recipe/process",Value::Null),
            ("/recipe/process/nativeIndex",json!(262144)),
            ("/recipe/inputs/0/choices/0/amount",json!("2")),
            ("/recipe/inputs/0/choices/0/consume",json!({"kind":"allocated"})),
            ("/recipe/source/handler",json!("codechicken.nei.recipe.ShapedRecipeHandler")),
            ("/recipe/duration",json!("20"))] {
            check_native("Extreme",raw,path,bad);
        }
        if !value["recipe"]["process"]["earlier"].as_array().unwrap().is_empty() {
            for(path,bad)in[("/recipe/process/earlier/0/inputs/0/0/id",json!("missing")),
                ("/recipe/process/earlier/0/inputs",json!([])),
                ("/recipe/process/nativeIndex",json!(0))] {check_native("Extreme priority",raw,path,bad);}
        }
        if !value["recipe"]["process"]["repair"].as_array().unwrap().is_empty() {
            for(path,bad)in[("/recipe/process/repair",json!([])),
                ("/recipe/process/repair/0/item",json!("missing")),
                ("/recipe/process/repair/0/maximum",json!(0))] {check_native("Extreme repair",raw,path,bad);}
        }
    }
}

#[test]
fn native_rail_blast_fuel_is_independent_of_recipe_materials() {
    let raw=include_str!("../../../../contracts/fixtures/native-rail-blast-fuel-contract.json");
    let value:Value=serde_json::from_str(raw).unwrap();
    let (domain,source)=load(&value).unwrap();domain.validate(&source).unwrap();
    let Some(Process::RailcraftBlast(process))=&domain.recipes[0].process else {panic!()};
    let items=domain.items.iter().map(|item|(item.id.as_str(),item)).collect();
    let uses=rail_blast::uses(process,&items).unwrap();
    assert!(uses.contains(&process.fuel.allowed[0].item));
    let tagged=value["items"].as_array().unwrap().iter().find(|item|item["nbt"].is_object()).unwrap();
    assert!(!uses.iter().any(|id|id==tagged["id"].as_str().unwrap()));
    for(path,bad)in [("/recipe/process/registeredTime",json!(26)),
        ("/recipe/process/fuel/allowed/0/metadata",json!(1)),
        ("/recipe/process/fuel/filter",json!("ignoreNbt")),
        ("/recipe/process/fuel/reload",json!("everyRecipe")),
        ("/recipe/inputs/0/choices/0/amount",json!("2")),
        ("/recipe/source/handler",json!("tonius.neiintegration.mods.railcraft.RecipeHandlerCokeOven"))] {
        check_native("Rail blast fuel",raw,path,bad);
    }
}

#[test]
fn native_rolling_nbt_keeps_machine_rules_and_whole_tag_transfer() {
    let raw=include_str!("../../../../contracts/fixtures/native-rolling-nbt-contract.json");
    for(path,bad)in [("/recipe/process/copyNbt",json!(false)),
        ("/recipe/outputs/0/change",Value::Null),
        ("/recipe/source/handler",json!("tonius.neiintegration.mods.railcraft.RecipeHandlerRollingMachineShapeless")),
        ("/recipe/inputs/0/choices/0/returns",json!([{"kind":"item","id":"missing","amount":"1"}])),
        ("/recipe/outputs/0/change/samples/1/id",json!("missing"))] {
        check_native("rolling NBT",raw,path,bad);
    }
}

#[test]
fn native_ae_facade_keeps_non_idempotent_texture_metadata() {
    let raw=include_str!("../../../../contracts/fixtures/ae-facade-normalized-contract.json");
    let value:Value=serde_json::from_str(raw).unwrap();
    assert_eq!(value["recipe"]["process"]["textureMetadata"],json!(4));
    check_native("AE2 normalized facade",raw,"/recipe/process/textureMetadata",json!(0));
    check_native("AE2 normalized facade",raw,"/recipe/process/textureMetadata",json!(-1));
}

#[test]
fn native_railcraft_literal_wildcards_keep_concrete_visuals() {
    let raw=include_str!("../../../../contracts/fixtures/native-rail-literal-wildcards-contract.json");
    let value:Value=serde_json::from_str(raw).unwrap();
    assert!(value["items"].as_array().unwrap().iter().all(|item|item["meta"].as_i64().unwrap()>=0));
    let choices=value["recipe"]["inputs"][0]["choices"].as_array().unwrap();
    assert_eq!(choices[1]["rule"]["value"],json!(-1));
    assert_eq!(choices[2]["rule"]["value"],json!(32767));
    check_native("Railcraft literal wildcards",raw,"/recipe/inputs/0/choices/1/rule/value",json!(-2));
}

#[test]
fn native_gt_crafting_retains_complete_finite_order_states() {
    for raw in [include_str!("../../../../contracts/fixtures/native-gt-crafting-overwrite-contract.json"),
        include_str!("../../../../contracts/fixtures/native-gt-crafting-keeping-contract.json"),
        include_str!("../../../../contracts/fixtures/native-gt-crafting-shaped-contract.json"),
        include_str!("../../../../contracts/fixtures/native-gt-crafting-neutral-contract.json")] {
        let value:Value=serde_json::from_str(raw).unwrap();
        let check=|value:&Value|->anyhow::Result<()> {let(domain,source)=load(value)?;domain.validate(&source)};
        check(&value).unwrap();
        for(path,bad)in [("/recipe/process/version",json!("future-version")),
            ("/recipe/process/scope",json!("arbitraryInventory")),
            ("/recipe/outputs/0/change/action/kind",json!("craftingNbt")),
            ("/recipe/process/states/0/positions/1",json!(0)),
            ("/recipe/process/states/0/choices/0",json!(999)),
            ("/recipe/outputs/0/change/samples/0/amount",json!("99"))] {
            let mut changed=value.clone();*changed.pointer_mut(path).unwrap()=bad;
            assert!(check(&changed).is_err(),"GT finite observation lost {path}");
        }
        let mut changed=value.clone();changed["recipe"]["process"]["states"].as_array_mut().unwrap().pop();
        changed["recipe"]["outputs"][0]["change"]["samples"].as_array_mut().unwrap().pop();
        assert!(check(&changed).is_err(),"GT result coverage omitted a reachable first-tagged/overwrite state");
    }
}

#[test]
fn native_tag_equality_uses_numeric_float_equality_and_keeps_list_types() {
    let cases: Value = serde_json::from_str(include_str!("../../../../contracts/fixtures/gt-native-tag-equality.json")).unwrap();
    for case in cases.as_array().unwrap() {
        let left: crate::identity::Nbt = serde_json::from_value(case["left"].clone()).unwrap();
        let right: crate::identity::Nbt = serde_json::from_value(case["right"].clone()).unwrap();
        assert_eq!(left.native_equal(&right).unwrap(), case["nativeEqual"].as_bool().unwrap(), "{}",case["name"]);
    }
}

#[test]
fn native_cache_key_hash_matches_the_installed_java_nbt_oracle() {
    let cases: Value = serde_json::from_str(include_str!("../../../../contracts/fixtures/native-nbt-hash-law.json")).unwrap();
    for case in cases.as_array().unwrap() {
        let tag: crate::identity::Nbt = serde_json::from_value(case["tag"].clone()).unwrap();
        assert_eq!(tag.native_hash().unwrap(),case["nativeHash"].as_i64().unwrap() as i32,"case {}",case["case"]);
    }
}

#[test]
fn native_pool_cache_snapshot_replays_stored_hash_routing_and_expiry() {
    let fixture: Value=serde_json::from_str(include_str!("../../../../contracts/fixtures/magicbees-pool-cache-lookup.json")).unwrap();
    for case in fixture["cases"].as_array().unwrap() {
        let cache:super::botania_cache::Snapshot=serde_json::from_value(case["cache"].clone()).unwrap();
        let query:crate::identity::Nbt=serde_json::from_value(case["query"].clone()).unwrap();
        let now=case["now"].as_str().unwrap().parse::<i64>().unwrap();
        let result=cache.lookup(&query,now).unwrap();
        assert_eq!(serde_json::to_value(result).unwrap(),case["result"],"{}",case["case"]);
        let mut changed=cache.clone();changed.mask=127;
        assert!(changed.lookup(&query,now).is_err(),"Malformed cache routing was accepted");
        let mut changed=cache.clone();changed.segments[0].count+=1;
        assert!(changed.lookup(&query,now).is_err(),"Malformed cache count was accepted");
    }
}

#[test]
fn native_pool_cache_transitions_replay_installed_guava17() {
    use super::botania_cache_machine::{Load, Machine, State};
    let fixture: Value=serde_json::from_str(include_str!("../../../../contracts/fixtures/magicbees-pool-cache-transitions.json")).unwrap();
    for case in fixture["observations"].as_array().unwrap() {
        let mut machine=Machine::new(serde_json::from_value(case["initial"].clone()).unwrap()).unwrap();
        for (i,step) in case["steps"].as_array().unwrap().iter().enumerate() {
            let key=step["key"].as_u64().unwrap() as usize;
            let ticks=step["ticks"].as_array().unwrap();
            let mut at=0;
            let mut clock=|| {let now=ticks.get(at).ok_or_else(||anyhow::anyhow!("Extra cache ticker read"))?;
                at+=1;Ok(now.as_str().unwrap().parse::<i64>()?)};
            let mut loads=0;
            let mut loader=|key:&mut crate::identity::Nbt| {
                loads+=1;*key=serde_json::from_value(step["keyAfter"].clone())?;
                Ok(if step["result"]["kind"]=="nativeError"{Load::NativeError}
                    else{Load::Loaded(serde_json::from_value(step["result"]["chromosome"].clone())?)})
            };
            let actual=machine.get(key,&mut clock,&mut loader).unwrap();
            assert_eq!(serde_json::to_value(actual).unwrap(),step["result"],"{}/{i}",case["case"]);
            assert_eq!(serde_json::to_value(machine.key(key).unwrap()).unwrap(),step["keyAfter"]);
            assert_eq!(at,ticks.len(),"Exact native ticker consumption");
            assert_eq!(loads,if step["result"]["kind"]=="hit"{0}else{1},"Exact native loader count");
        }
        assert_eq!(serde_json::to_value(machine.snapshot()).unwrap(),case["state"],"{}",case["case"]);
        let state:State=serde_json::from_value(case["state"].clone()).unwrap();
        assert_eq!(serde_json::to_value(Machine::new(state.clone()).unwrap().snapshot()).unwrap(),case["state"]);
        let mut bad=state.clone();bad.segments[0].total_weight+=1;
        assert!(Machine::new(bad).is_err(),"Invalid weight/count state was accepted");
    }
}

#[test]
fn native_bee_pool_production_rows_bind_ordered_program_and_conditional_prototypes() {
    for raw in [
        include_str!("../../../../contracts/fixtures/botania-native-pool-0-contract.json"),
        include_str!("../../../../contracts/fixtures/botania-native-pool-1-contract.json"),
        include_str!("../../../../contracts/fixtures/botania-native-pool-2-contract.json"),
        include_str!("../../../../contracts/fixtures/botania-native-pool-3-contract.json"),
    ] {
        let original:Value=serde_json::from_str(raw).unwrap();
        let (domain,source)=load(&original).unwrap();
        domain.validate(&source).unwrap();
        let items=domain.items.iter().map(|i|(i.id.as_str(),i)).collect::<std::collections::BTreeMap<_,_>>();
        let choice=&domain.recipes[0].inputs[0].choices[0];let item=items[choice.id.as_str()];
        assert!(choice.rule.indexes_example(item,&items),"Conditional pool display must remain discoverable under input uses");
        assert!(!matching::simple(&choice.rule,item,item),"Discoverability is not independent native acceptance");
        for (path,bad) in [
            ("/recipe/process",Value::Null),("/recipe/process/index",json!(4095)),
            ("/recipe/process/mana",json!(9999)),("/recipe/process/nativeRecipe",json!("foreign.Callback")),
            ("/recipe/outputs/0/change",Value::Null),
            ("/recipe/outputs/0/change/samples",json!([{"id":original["recipe"]["outputs"][0]["id"],"amount":"1"}])),
            ("/recipe/inputs/0/choices/0/rule",json!({"kind":"exact"})),
        ] {
            let mut changed=original.clone();*changed.pointer_mut(path).unwrap()=bad;
            assert!(load(&changed).and_then(|(d,s)|d.validate(&s)).is_err(),"Accepted broken pool binding {path}");
        }
    }
}

#[test]
fn native_bee_pool_session_shares_cache_and_persistent_recipe_outputs() {
    use super::botania_session::Session;
    let fixture:Value=serde_json::from_str(include_str!("../../../../contracts/fixtures/magicbees-pool-session.json")).unwrap();
    let mut session=Session::new(serde_json::from_value(fixture["source"].clone()).unwrap(),
        serde_json::from_value(fixture["entries"].clone()).unwrap(),serde_json::from_value(fixture["cache"].clone()).unwrap()).unwrap();
    for (i,row) in fixture["observations"].as_array().unwrap().iter().enumerate() {
        let ticks=row["ticks"].as_array().unwrap();let mut at=0;
        let mut clock=||{let tick=ticks.get(at).ok_or_else(||anyhow::anyhow!("Extra session ticker read"))?;
            at+=1;Ok(tick.as_str().unwrap().parse::<i64>()?)};
        let actual=session.select(serde_json::from_value(row["input"].clone()).unwrap(),row["alchemy"].as_bool().unwrap(),
            row["conjuration"].as_bool().unwrap(),row["mana"].as_i64().unwrap() as i32,&mut clock).unwrap();
        assert_eq!(serde_json::to_value(actual).unwrap(),row["selection"],"invocation {i}");
        assert_eq!(serde_json::to_value(session.outputs().unwrap()).unwrap(),row["outputs"],"output caches {i}");
        assert_eq!(serde_json::to_value(session.cache_state().unwrap()).unwrap(),row["cache"],"native genome cache {i}");
        assert_eq!(at,ticks.len(),"Exact native clock-read sequence");
    }
    let last=fixture["observations"].as_array().unwrap().last().unwrap();
    assert_eq!(serde_json::to_value(session.take_output(0).unwrap()).unwrap(),last["outputs"][0]);
    assert_eq!(serde_json::to_value(session.take_output(0).unwrap()).unwrap(),fixture["entries"][0]["prototype"]);
    assert!(session.select(serde_json::from_value(fixture["observations"][0]["input"].clone()).unwrap(),
        true,false,100,&mut ||anyhow::bail!("ticker unavailable")).is_err());
    assert!(session.cache_state().is_err()&&session.outputs().is_err()&&session.take_output(0).is_err(),
        "Aborted cache transitions must invalidate the detached session");
}

#[test]
fn native_bee_pool_snapshot_law_preserves_repair_identity_and_dynamic_output() {
    for raw in [include_str!("../../../../contracts/fixtures/magicbees-pool-snapshot-law.json"),
        include_str!("../../../../contracts/fixtures/magicbees-pool-cache-law.json")] {
        let fixture:Value=serde_json::from_str(raw).unwrap();
        let source:super::botania_bee::Source=serde_json::from_value(fixture["source"].clone()).unwrap();
        for case in fixture["observations"].as_array().unwrap() {
            let offered=serde_json::from_value(case["input"].clone()).unwrap();
            let reads=serde_json::from_value(case["reads"].clone()).unwrap();
            let result=source.evaluate(fixture["form"].as_str().unwrap(),fixture["inputSpecies"].as_str().unwrap(),
                fixture["outputSpecies"].as_str().unwrap(),offered,reads).unwrap();
            assert_eq!(serde_json::to_value(result).unwrap(),case["result"],"{}",case["case"]);
        }
    }
}

#[test]
fn native_bee_pool_transition_keeps_repairs_after_mismatch_and_failure() {
    let fixture:Value=serde_json::from_str(include_str!("../../../../contracts/fixtures/magicbees-pool-transition-law.json")).unwrap();
    let source:super::botania_bee::Source=serde_json::from_value(fixture["source"].clone()).unwrap();
    for case in fixture["observations"].as_array().unwrap() {
        let offered=serde_json::from_value(case["input"].clone()).unwrap();
        let reads=serde_json::from_value(case["reads"].clone()).unwrap();
        let actual=source.transition(fixture["form"].as_str().unwrap(),fixture["inputSpecies"].as_str().unwrap(),
            fixture["outputSpecies"].as_str().unwrap(),offered,reads).unwrap();
        assert_eq!(serde_json::to_value(actual).unwrap(),case["transition"],"{}",case["case"]);
    }
}

#[test]
fn native_bee_pool_selection_preserves_callback_order_catalysts_and_mana_blocking() {
    let fixture:Value=serde_json::from_str(include_str!("../../../../contracts/fixtures/magicbees-pool-selection-law.json")).unwrap();
    let source:super::botania_bee::Source=serde_json::from_value(fixture["source"].clone()).unwrap();
    for case in fixture["observations"].as_array().unwrap() {
        let entries:Vec<super::botania_selection::Entry>=serde_json::from_value(case["entries"].clone()).unwrap();
        let offered=serde_json::from_value(case["input"].clone()).unwrap();
        let traces:Vec<super::botania_selection::Trace>=serde_json::from_value(case["traces"].clone()).unwrap();
        let actual=source.select(&entries,offered,case["alchemy"].as_bool().unwrap(),case["conjuration"].as_bool().unwrap(),
            case["mana"].as_i64().unwrap() as i32,&traces).unwrap();
        assert_eq!(serde_json::to_value(actual).unwrap(),case["selection"],"{}",case["case"]);
    }
}

#[test]
fn native_crafting_preserves_grids_and_consumes_one_per_cell() {
    for raw in [
        include_str!("../../../../contracts/fixtures/native-crafting-shaped-contract.json"),
        include_str!("../../../../contracts/fixtures/native-crafting-forge-contract.json"),
        include_str!("../../../../contracts/fixtures/native-crafting-shapeless-contract.json"),
        include_str!("../../../../contracts/fixtures/native-crafting-map-contract.json"),
        include_str!("../../../../contracts/fixtures/native-crafting-nbt-contract.json"),
    ] {
        let value: Value = serde_json::from_str(raw).unwrap();
        let check = |value: &Value| -> anyhow::Result<()> { let (domain,source)=load(value)?; domain.validate(&source) };
        check(&value).unwrap();
        let mut bad = value.clone(); bad["recipe"]["inputs"][0]["choices"][0]["amount"] = json!("2");
        assert!(check(&bad).is_err(), "crafting accepted a display stack amount as native per-cell consumption");
        let mut bad = value.clone(); bad["recipe"]["duration"] = json!("200");
        assert!(check(&bad).is_err(), "manual crafting acquired a machine execution cost");
        if !value["recipe"]["grid"].is_null() {
            let mut bad = value.clone(); bad["recipe"]["grid"]["cells"][0] = json!(99);
            assert!(check(&bad).is_err(), "native grid lost its logical input binding");
        } else {
            let mut bad = value.clone(); bad["recipe"]["grid"] = json!({"width":2,"height":1,"cells":[0,1],"mirror":true});
            assert!(check(&bad).is_err(), "shapeless multiplicity became a shaped grid");
        }
    }
}

#[test]
fn native_crafting_tool_wear_keeps_conditional_energy_and_native_units() {
    for raw in [include_str!("../../../../contracts/fixtures/native-crafting-tool-physical-contract.json"),
        include_str!("../../../../contracts/fixtures/native-crafting-tool-electric-contract.json")] {
        let value: Value=serde_json::from_str(raw).unwrap();
        let check=|value:&Value|->anyhow::Result<()> {let(domain,source)=load(value)?;domain.validate(&source)};
        check(&value).unwrap();
        for (field,bad) in [("cost",json!(400)),("owner",json!("unknown.ToolCallback"))] {
            let mut changed=value.clone();changed["recipe"]["inputs"][0]["choices"][0]["consume"][field]=bad;
            assert!(check(&changed).is_err(),"native tool consumption lost {field}");
        }
        for(field,bad) in [("electric",json!(!value["recipe"]["inputs"][0]["choices"][0]["consume"]["sample"]["electric"].as_bool().unwrap())),
            ("damage",json!("11")),("limit",json!("999"))] {
            let mut changed=value.clone();changed["recipe"]["inputs"][0]["choices"][0]["consume"]["sample"][field]=bad;
            assert!(check(&changed).is_err(),"native tool sample lost {field}");
        }
        let mut changed=value.clone();changed["recipe"]["source"]["handler"]=json!("unknown.Machine");
        assert!(check(&changed).is_err(),"GT tool consumption escaped the native crafting family");
    }
}

#[test]
fn native_container_returns_remain_input_dependent_and_crafting_only() {
    for raw in [include_str!("../../../../contracts/fixtures/native-container-cutter-contract.json"),
        include_str!("../../../../contracts/fixtures/native-container-cosmetic-contract.json")] {
        let value:Value=serde_json::from_str(raw).unwrap();
        let check=|v:&Value|->anyhow::Result<()> {let(d,s)=load(v)?;d.validate(&s)};
        check(&value).unwrap();
        for (path,bad) in [
            ("/recipe/inputs/0/choices/0/amount",json!("2")),
            ("/recipe/inputs/0/choices/0/consume/callback",json!("unknownCallback")),
            ("/recipe/inputs/0/choices/0/returns",json!([{"kind":"item","id":value["items"][0]["id"],"amount":"1"}])),
            ("/recipe/source/handler",json!("unknown.Machine")),
        ] {
            let mut changed=value.clone();*changed.pointer_mut(path).unwrap()=bad;
            assert!(check(&changed).is_err(),"native container lost {path}");
        }
        let mut changed=value.clone();
        changed["recipe"]["inputs"][0]["choices"][0]["consume"]["callback"]=json!(if raw.contains("ic2Cutter") {"botaniaCosmetic"} else {"ic2Cutter"});
        assert!(check(&changed).is_err(),"native container callback accepted the wrong item registry");
    }
}

#[test]
fn gtpp_angle_grinder_wear_uses_its_native_400_unit_callback() {
    let mut value:Value=serde_json::from_str(include_str!("../../../../contracts/fixtures/native-crafting-tool-physical-contract.json")).unwrap();
    value["recipe"]["inputs"][0]["choices"][0]["consume"]["owner"]=json!("gtPlusPlus.xmod.gregtech.common.tools.ToolAngleGrinder");
    value["recipe"]["inputs"][0]["choices"][0]["consume"]["cost"]=json!(400);
    let check=|v:&Value|->anyhow::Result<()> {let(d,s)=load(v)?;d.validate(&s)};
    check(&value).unwrap();
    value["recipe"]["inputs"][0]["choices"][0]["consume"]["cost"]=json!(200);
    assert!(check(&value).is_err(),"angle grinder accepted saw wear");
}

#[test]
fn native_crafting_nbt_projections_keep_empty_tags_and_base_identity() {
    let value: Value = serde_json::from_str(include_str!("../../../../contracts/fixtures/native-crafting-nbt-contract.json")).unwrap();
    let check = |value: &Value| -> anyhow::Result<()> { let (domain, source) = load(value)?; domain.validate(&source) };
    check(&value).unwrap();
    for (path, bad) in [
        ("/recipe/outputs/0/change/samples/2/id", value["recipe"]["outputs"][0]["change"]["samples"][1]["id"].clone()),
        ("/recipe/outputs/0/change/samples/1/amount", json!("1")),
        ("/recipe/outputs/0/change/input", json!(1)),
        ("/recipe/process", Value::Null),
        ("/recipe/grid", Value::Null),
    ] {
        let mut bad_value = value.clone();
        *bad_value.pointer_mut(path).unwrap_or_else(|| panic!("missing {path}")) = bad;
        assert!(check(&bad_value).is_err(), "native tag projection lost {path}");
    }
    let mut missing = value.clone(); missing["recipe"]["outputs"][0]["change"]["samples"].as_array_mut().unwrap().pop();
    assert!(check(&missing).is_err(), "missing native tag projection accepted");
    let mut tuple = value.clone(); tuple["recipe"]["outputs"][0]["change"]["bindings"] = json!([[0, 0], [0, 0], [0, 0]]);
    assert!(check(&tuple).is_err(), "independent tag projections became matching tuples");
}

#[test]
fn native_zero_fluid_keeps_family_policy_without_inventing_positive_amounts() {
    for (raw, slot) in [
        (include_str!("../../../../contracts/fixtures/gt-zero-fluid-contract.json"), 0),
        (include_str!("../../../../contracts/fixtures/gt-presence-fluid-contract.json"), 0),
        (include_str!("../../../../contracts/fixtures/vat-zero-fluid-contract.json"), 1),
    ] {
        let value: Value = serde_json::from_str(raw).unwrap();
        let check = |value: &Value| -> anyhow::Result<()> { let (domain, source) = load(value)?; domain.validate(&source) };
        check(&value).unwrap_or_else(|error| panic!("{}: {error:#}",value["recipe"]["source"]["key"]));
        for (field, bad) in [("amount", json!("1")), ("rule", json!({"kind":"exact"})), ("consume", json!({"kind":"consume"}))] {
            let mut changed = value.clone();
            changed["recipe"]["inputs"][slot]["choices"][0][field] = bad;
            assert!(check(&changed).is_err(), "native policy lost {field}");
        }
        let mut changed = value.clone();
        changed["recipe"]["source"]["handler"] = json!("unknown.NativeRecipe");
        assert!(check(&changed).is_err(), "zero rule accepted outside its native family");
        if value["recipe"]["source"]["key"].as_str().unwrap().starts_with("gtpp.recipe.multiblockrockbreaker/") {
            let mut changed = value.clone(); changed["recipe"]["inputs"].as_array_mut().unwrap().pop();
            assert!(check(&changed).is_err(), "missing rock-breaker lava condition accepted");
        }
    }
}

#[test]
fn ic2_crop_prediction_keeps_parents_and_exact_conditional_weight() {
    let raw = include_str!("../../../../contracts/fixtures/ic2-crop-prediction-contract.json");
    let value: Value = serde_json::from_str(raw).unwrap();
    let check = |value: Value| -> anyhow::Result<()> {
        let (domain, source) = load(&value)?;
        domain.validate(&source)
    };
    check(value.clone()).unwrap();
    for (path, bad) in [
        ("/recipe/process", Value::Null),
        ("/recipe/process/total", json!(0)),
        ("/recipe/process/parents", json!([])),
        ("/recipe/process/usageListedFor", json!([{"owner":"IC2","name":"unregistered-parent"}])),
        ("/recipe/outputs/0/chance", json!({"numerator":"1","denominator":"1"})),
        ("/recipe/inputs/0/choices/0/consume", json!({"kind":"consume"})),
        ("/recipe/duration", json!("20")),
    ] {
        let mut changed = value.clone();
        *changed.pointer_mut(path).unwrap() = bad;
        assert!(check(changed).is_err(), "{path}");
    }
}

#[test]
fn fmp_native_rule_keeps_procedure_and_correlated_result() {
    let raw=include_str!("../../../../contracts/fixtures/fmp-microblock-contract.json");
    check_native("fmp procedure",raw,"/recipe/process",Value::Null);
    check_native("fmp branch",raw,"/recipe/process/branch",json!("hollow"));
    check_native("fmp material",raw,"/recipe/process/materialId",json!(1));
    check_native("fmp result",raw,"/recipe/outputs/0/change",Value::Null);
}

#[test]
fn fmp_shared_lookup_keeps_id_zero_and_rejects_rehashed_semantic_changes(){
    let raw=include_str!("../../../../contracts/fixtures/fmp-missing-material-contract.json");
    let value:Value=serde_json::from_str(raw).unwrap();let(domain,source)=load(&value).unwrap();domain.validate(&source).unwrap();
    assert!(domain.recipes[0].inputs[0].choices.iter().all(|choice|matches!(&choice.rule,Match::FmpMaterial{material_id:0,native_text:None,..})));
    for(path,bad)in[("/recipe/process/materialId",json!(1)),("/recipe/inputs/0/choices/0/rule",json!({"kind":"microMaterial","material":"a_dirt","aliases":["a_dirt"]})),
        ("/recipe/inputs/0/choices/0/rule/nativeText",json!("a_dirt")),("/recipe/outputs/0/amount",json!("2"))]{
        let mut changed=value.clone();if path.ends_with("nativeText"){changed["recipe"]["inputs"][0]["choices"][0]["rule"]["nativeText"]=bad;}else{*changed.pointer_mut(path).unwrap()=bad;}
        assert!(load(&changed).and_then(|(domain,source)|domain.validate(&source)).is_err(),"FMP semantic change accepted: {path}");
    }
    let raw=include_str!("../../../../contracts/fixtures/fmp-microblock-contract.json");
    for(field,bad)in[("strength",json!(99)),("owner",json!("unadapted.Saw")),("nbt",json!("preserve"))]{
        let mut changed:Value=serde_json::from_str(raw).unwrap();
        changed["programs"][3]["data"]["rows"][0][field]=bad;
        rebind_fmp_program(&mut changed);
        assert!(load(&changed).and_then(|(domain,source)|domain.validate(&source)).is_err(),"Rehashed FMP saw semantic change accepted: {field}");
    }
    let mut missing:Value=serde_json::from_str(raw).unwrap();missing["programs"].as_array_mut().unwrap().remove(1);
    assert!(load(&missing).and_then(|(domain,source)|domain.validate(&source)).is_err());
}
fn rebind_fmp_program(value:&mut Value){
    let mut rules=json!({});
    for chunk in value["programs"].as_array().unwrap(){let key=match chunk["data"]["kind"].as_str().unwrap(){
        "fmpMaterials"=>"materials","fmpNames"=>"names","fmpAliases"=>"aliases","fmpSaws"=>"saws","fmpSettings"=>"settings",_=>panic!()};
        rules[key]=chunk["data"]["rows"].clone();
    }
    let program=content_id("program",&json!({"kind":"fmpMicroblock","rules":rules})).unwrap();
    for chunk in value["programs"].as_array_mut().unwrap(){chunk["program"]=json!(program);let mut row:programs::ProgramChunk=serde_json::from_value(chunk.clone()).unwrap();row.id=row.content_id().unwrap();*chunk=serde_json::to_value(row).unwrap();}
    value["recipe"]["process"]["program"]=json!(program);
    for input in value["recipe"]["inputs"].as_array_mut().unwrap(){for choice in input["choices"].as_array_mut().unwrap(){if choice["rule"]["kind"]=="fmpMaterial"{choice["rule"]["program"]=json!(program);}}}
}

struct FixtureSource { source: Source, _root: Option<tempfile::TempDir> }
impl std::ops::Deref for FixtureSource { type Target=Source; fn deref(&self)->&Source { &self.source } }

fn load(value:&Value) -> anyhow::Result<(Domain,FixtureSource)> {
    let mut source=Source::open(&std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../contracts/fixtures/source-semantic-40"))?;
    let mut domain=Domain::load(&source)?;
    let mut locale_root=None;
    if let Some(locale)=value.get("strings").and_then(Value::as_array).and_then(|rows|rows.first()).and_then(|row|row["locale"].as_str()) {
        if locale!=source.environment()?.locale {
            // Own a separate language fixture. Do not add Chinese native text to the English
            // Source or relabel its English prose as a translation.
            let mut remap=std::collections::BTreeMap::new();
            for (i,text) in domain.strings.iter_mut().enumerate(){
                let old=text.id.clone();text.locale=locale.to_owned();
                text.text=if locale=="zh_CN" { format!("本地测试标签{i}") } else { format!("Local fixture label {i}") };
                text.id=content_id("text",text)?;remap.insert(old,text.id.clone());
            }
            fn replace(value:&mut Value,map:&std::collections::BTreeMap<String,String>){match value {
                Value::String(s)=>if let Some(id)=map.get(s){*s=id.clone()},
                Value::Array(rows)=>for row in rows {replace(row,map)},
                Value::Object(rows)=>for row in rows.values_mut(){replace(row,map)},_=>{}
            }}
            let mut raw=serde_json::to_value(&domain)?;replace(&mut raw,&remap);domain=serde_json::from_value(raw)?;
            for mutation in &mut domain.mutations { mutation.id=content_id("mutation",mutation)?; }
            // These recipe probes do not exercise environment-bound construction previews.
            domain.builds.clear();domain.shapes.clear();domain.models.clear();domain.structures.clear();domain.blocks.clear();
            let root=tempfile::tempdir()?;
            for file in &source.manifest.files {let target=root.path().join(&file.path);std::fs::create_dir_all(target.parent().unwrap())?;std::fs::write(target,source.read_file(file)?)?;}
            let mut env=serde_json::to_value(source.environment()?)?;env["locale"]=json!(locale);let bytes=serde_json::to_vec(&env)?;
            let hash=format!("{:x}",<sha2::Sha256 as sha2::Digest>::digest(&bytes));let mut manifest=source.manifest.clone();manifest.environment=hash.clone();
            let file=manifest.files.iter_mut().find(|f|f.kind=="environment").unwrap();file.bytes=bytes.len()as u64;file.decoded_bytes=file.bytes;file.sha256=hash;
            std::fs::write(root.path().join(&file.path),bytes)?;manifest.id=manifest.digest()?;std::fs::write(root.path().join("manifest.json"),serde_json::to_vec(&manifest)?)?;
            source=Source::open(root.path())?;locale_root=Some(root);
        }
    }
    if let Some(strings)=value.get("strings").and_then(Value::as_array) {
        for raw in strings {
            let text: Text=serde_json::from_value(raw.clone())?;
            domain.strings.retain(|prior|prior.id!=text.id);domain.strings.push(text);
        }
    }
    let item=domain.items[0].clone();
    for raw in value["items"].as_array().unwrap() {
        let mut next=serde_json::to_value(&item)?;
        for key in ["id","registry","meta","nbt","tags"] {next[key]=raw[key].clone();}
        if let Some(durability)=raw.get("durability") {next["durability"]=durability.clone();}
        let next:Item=serde_json::from_value(next)?;domain.items.retain(|i|i.id!=next.id);domain.items.push(next);
    }
    let fluid=domain.fluids[0].clone();
    for raw in value["fluids"].as_array().unwrap() {
        let mut next=serde_json::to_value(&fluid)?;
        for key in ["id","registry","nbt"] {next[key]=raw[key].clone();}
        let next:Fluid=serde_json::from_value(next)?;domain.fluids.retain(|i|i.id!=next.id);domain.fluids.push(next);
    }
    if let Some(programs) = value.get("programs").filter(|value| !value.is_null()) {
        domain.programs.extend(serde_json::from_value::<Vec<programs::ProgramChunk>>(programs.clone())?);
    }
    if let Some(aspects)=value.get("aspects").and_then(Value::as_array) {
        for raw in aspects {
            // Native identity is observed by Java; labels/icons belong to this test Source.
            let mut next=serde_json::to_value(&domain.aspects[0])?;
            next["id"]=raw["id"].clone();next["source"]=raw["source"].clone();
            next["components"]=json!([]);
            let next:Aspect=serde_json::from_value(next)?;
            domain.aspects.retain(|a|a.id!=next.id);domain.aspects.push(next);
        }
    }
    let mut recipe:Recipe=serde_json::from_value(value["recipe"].clone())?;
    recipe.properties.clear();recipe.category=domain.categories[0].id.clone();
    let mut category=domain.categories[0].clone();category.source=recipe.source.clone();category.id=origin_id("category",&category.source)?;category.program=None;recipe.category=category.id.clone();
    if let Some(Process::ForestryFabricator(p))=&recipe.process {category.program=Some(p.program.clone());}
    if let Some(Process::ForestryProcessing(p))=&recipe.process {category.program=Some(p.program.clone());}
    if let Some(Process::FmpMicroblock(p))=&recipe.process {category.program=Some(p.program.clone());}
    if let Some(Process::XuMicroblock(p))=&recipe.process {category.program=Some(p.program.clone());}
    if let Some(Process::NativeBotaniaPool(p))=&recipe.process {category.program=Some(p.program.clone());}
    domain.categories.push(category);
    recipe.id=recipe_id(&recipe)?;domain.recipes=vec![recipe];
    Ok((domain,FixtureSource{source,_root:locale_root}))
}

fn check_native(name:&str,raw:&str,path:&str,bad:Value) {
    let value:Value=serde_json::from_str(raw).unwrap();
    let (domain,source)=load(&value).unwrap();
    domain.validate(&source).unwrap_or_else(|error|panic!("{name}: {error:#}"));
    let mut changed=value;*changed.pointer_mut(path).unwrap()=bad;
    assert!(load(&changed).and_then(|(d,s)|d.validate(&s)).is_err(),"{name}: native semantic change was accepted");
}

#[test]
fn targeted_botania_containers_keep_native_identity_and_dynamic_returns() {
    for raw in [
        include_str!("../../../../contracts/fixtures/targeted-container-planet-contract.json"),
        include_str!("../../../../contracts/fixtures/targeted-container-blackhole-contract.json"),
    ] {
        check_native("targeted Botania container",raw,"/items/0/registry",json!("minecraft:paper"));
        check_native("targeted Botania remainder",raw,"/recipe/inputs/0/choices/0/returns",json!([{"kind":"item","id":"item_unknown","amount":"1"}]));
    }
}

#[test]
fn targeted_wireless_upgrade_keeps_physical_cells_and_offered_stack_copy() {
    let raw=include_str!("../../../../contracts/fixtures/targeted-wireless-upgrade-contract.json");
    let (domain,source)=load(&serde_json::from_str(raw).unwrap()).unwrap();
    domain.validate(&source).unwrap();
    assert_eq!(super::quantity::quantity_bounds(&domain.recipes[0],&domain.recipes[0].outputs[0]).unwrap(),(1,i64::from(i32::MAX)));
    for (path,bad) in [
        ("/recipe/process/version",json!("unknown")),
        ("/recipe/process/flag",json!("differentFlag")),
        ("/recipe/grid/cells/1",Value::Null),
        ("/recipe/outputs/0/change",Value::Null),
        ("/recipe/outputs/0/change/action/set/infinityBoosterCard/value",json!("0")),
        ("/recipe/inputs/0/choices/0/rule/meta",json!(false)),
    ] {check_native("wireless fixed-cell upgrade",raw,path,bad);}
}

#[test]
fn ari_infusion_requires_current_handler_and_native_owner() {
    let mut fixture: Value = serde_json::from_str(include_str!(
        "../../../../contracts/fixtures/targeted-random-infusion-false-contract.json"
    )).unwrap();
    let recipe: Recipe = serde_json::from_value(fixture["recipe"].clone()).unwrap();
    assert_eq!(recipe.id, recipe_id(&recipe).unwrap());
    fixture["recipe"]["source"]["owner"] = json!("aspectrecipeindex");
    fixture["recipe"]["source"]["handler"] =
        json!("com.gtnewhorizons.aspectrecipeindex.nei.InfusionRecipeHandler");
    let (domain, source) = load(&fixture).unwrap();
    domain.validate(&source).unwrap();
    for (owner, handler) in [
        ("foreign", "com.gtnewhorizons.aspectrecipeindex.nei.InfusionRecipeHandler"),
        ("aspectrecipeindex", "com.gtnewhorizons.aspectrecipeindex.nei.AlchemyRecipeHandler"),
        ("tcneiadditions", "ru.timeconqueror.tcneiadditions.nei.TCNAInfusionRecipeHandler"),
    ] {
        let mut changed = fixture.clone();
        changed["recipe"]["source"]["owner"] = json!(owner);
        changed["recipe"]["source"]["handler"] = json!(handler);
        let (domain, source) = load(&changed).unwrap();
        assert!(domain.validate(&source).is_err(), "{owner}:{handler}");
    }
}

#[test]
fn targeted_random_infusion_retains_definition_instead_of_sampled_cost() {
    for raw in [
        include_str!("../../../../contracts/fixtures/targeted-random-infusion-false-contract.json"),
        include_str!("../../../../contracts/fixtures/targeted-random-infusion-true-contract.json"),
    ] {
        for (path,bad) in [
            ("/recipe/process/randomAspects/semantics",json!("unknown")),
            ("/recipe/process/randomAspects/refreshMillis",json!(0)),
            ("/recipe/process/randomAspects/bases/0/amount",json!("0")),
            ("/recipe/process/randomAspects/bases/0/aspect",json!("aspect_unknown")),
            ("/recipe/process/baseAspects",json!([{"aspect":"aspect_unknown","amount":"9"}])),
            ("/recipe/magic/aspects",json!([{"aspect":"aspect_unknown","amount":"4"}])),
        ] {check_native("random infusion cost law",raw,path,bad);}
    }
}

#[test]
fn cal_imprint_retains_native_admission_and_dynamic_id_copy() {
    let raw=include_str!("../../../../contracts/fixtures/cal-imprint-contract.json");
    for (path,bad) in [("/recipe/process",Value::Null),("/recipe/process/tag",json!("CircuitImprint")),
        ("/recipe/process/occupiedLimit",json!(3)),("/recipe/outputs/0/change",Value::Null),
        ("/recipe/inputs/0/choices/0/rule",json!({"kind":"wildcard","meta":false,"nbt":true}))] {
        check_native("CAL imprint",raw,path,bad);
    }
}

#[test]
fn xu_microblock_keeps_material_correlation_and_entire_input_nbt() {
    let raw=include_str!("../../../../contracts/fixtures/xu-microblock-contract.json");
    for (path,bad) in [("/recipe/process",Value::Null),("/recipe/process/materialId",json!(0)),
        ("/recipe/outputs/0/change",Value::Null),("/recipe/process/cells/0",json!(0)),
        ("/recipe/inputs/0/choices/0/rule",json!({"kind":"exact"}))] {
        check_native("XU microblock",raw,path,bad);
    }
}

#[test]
fn xu_native_literal_micro_supplies_first_tagged_physical_nbt(){
    for raw in [include_str!("../../../../contracts/fixtures/xu-literal-first-contract.json"),
        include_str!("../../../../contracts/fixtures/xu-literal-only-contract.json"),
        include_str!("../../../../contracts/fixtures/xu-untagged-first-contract.json"),
        include_str!("../../../../contracts/fixtures/xu-untagged-only-contract.json")] {
        let value:Value=serde_json::from_str(raw).unwrap();let(domain,source)=load(&value).unwrap();domain.validate(&source).unwrap();
        for(path,bad)in[("/recipe/process/states/1/offeredMirrored",json!(false)),
            ("/recipe/process/states/0/matchedMirrored",json!(!value["recipe"]["process"]["states"][0]["matchedMirrored"].as_bool().unwrap())),
            ("/recipe/process/states/0/firstPart",json!(99)),("/recipe/process/materialTexts",json!([])),
            ("/recipe/inputs/0/choices/0/rule",json!({"kind":"exact"})),
            ("/recipe/inputs/0/choices/0/consume",json!({"kind":"allocated"}))] {
            check_native("XU physical result",raw,path,bad);
        }
    }
}

#[test]
fn cal_imprint_29_uses_enum_ids_and_preserves_legacy_type() {
    use super::cal_imprint::{Role,accepts,apply};
    use crate::identity::{item_id,Nbt};
    use std::collections::BTreeMap;
    let mut machine:Item=serde_json::from_value(json!({"id":"item_test","registry":"gregtech:machine","meta":120,
        "nbt":{"type":"compound","value":{"owner":{"type":"string","value":"retained"}}},
        "name":"text_test","tooltip":[],"stackLimit":64,"durability":0,"tools":{},"armor":false,"tags":[]})).unwrap();
    let mut imprint=machine.clone();imprint.registry="bartworks:CircuitParts".into();imprint.meta=4;imprint.nbt=None;
    assert!(accepts(Role::Machine,&machine,&[4,5]).unwrap());
    assert!(accepts(Role::Imprint,&imprint,&[4,5]).unwrap(),"Enum metadata, not serialized item NBT, selects the imprint");
    let Nbt::Compound{value:tags}=machine.nbt.as_mut().unwrap()else{unreachable!()};
    tags.insert("id_imprint".into(),Nbt::Long{value:"4294967300".into()});
    assert!(!accepts(Role::Machine,&machine,&[4,5]).unwrap(),"Native getInteger narrows long IDs");
    let Nbt::Compound{value:tags}=machine.nbt.as_mut().unwrap()else{unreachable!()};
    tags.insert("Type".into(),Nbt::String{value:"legacy has precedence".into()});
    assert!(accepts(Role::Machine,&machine,&[4,5]).is_err(),"Legacy Type needs native lookup, not an unmatched/default result");
    let mut expected=if let Some(Nbt::Compound{value})=&machine.nbt{value.clone()}else{BTreeMap::new()};
    expected.insert("id_imprint".into(),Nbt::Int{value:"4".into()});
    let output=apply(&machine,&imprint).unwrap();
    assert_eq!(output.id,item_id(&machine.registry,machine.meta,Some(&Nbt::Compound{value:expected})).unwrap());
    assert_eq!(output.amount,"1");
}

#[test]
fn cal_imprint_admits_unknown_enum_ids_but_not_existing_imprints_or_legacy_type() {
    use super::cal_imprint::{Role,accepts,apply};use crate::identity::Nbt;
    let value:Value=serde_json::from_str(include_str!("../../../../contracts/fixtures/cal-imprint-contract.json")).unwrap();
    let (domain,_)=load(&value).unwrap();let r=&domain.recipes[0];
    let Process::CalImprint(p)=r.process.as_ref().unwrap()else{panic!("native process missing")};
    let mut machine=domain.items.iter().find(|i|i.id==r.inputs[0].choices[0].id).unwrap().clone();
    let imprint=domain.items.iter().find(|i|i.id==r.inputs[1].choices[0].id).unwrap();
    assert!(accepts(Role::Machine,&machine,&p.imprint_ids).unwrap());
    let result=apply(&machine,imprint).unwrap();assert_eq!(result.id,r.outputs[0].id);assert_eq!(result.amount,"1");
    let Nbt::Compound{value:tags}=machine.nbt.as_mut().unwrap()else{unreachable!()};
    tags.insert("id_imprint".into(),Nbt::Int{value:"30000".into()});
    assert!(accepts(Role::Machine,&machine,&p.imprint_ids).unwrap());
    let Nbt::Compound{value:tags}=machine.nbt.as_mut().unwrap()else{unreachable!()};
    tags.insert("id_imprint".into(),Nbt::Int{value:imprint.meta.to_string()});
    assert!(!accepts(Role::Machine,&machine,&p.imprint_ids).unwrap());
    let Nbt::Compound{value:tags}=machine.nbt.as_mut().unwrap()else{unreachable!()};
    tags.insert("Type".into(),Nbt::String{value:"broken".into()});
    assert!(accepts(Role::Machine,&machine,&p.imprint_ids).is_err());
}

#[test]
fn space_acquisition_cannot_become_unconditional_fluid_generation() {
    for raw in [include_str!("../../../../contracts/fixtures/space-pump-contract.json"),
        include_str!("../../../../contracts/fixtures/gas-siphon-contract.json")] {
        for (path,bad) in [
            ("/recipe/process",Value::Null),
            ("/recipe/process/baseAmount",json!("1")),
            ("/recipe/process/toolDebit",json!("consume")),
            ("/recipe/process/environmentLaw",json!("anywhere")),
            ("/recipe/process/powerLaw",json!("none")),
            ("/recipe/outputs/0/amount",json!("15000")),
            ("/recipe/outputs/0/quantity",Value::Null),
        ] {check_native("space conditional acquisition",raw,path,bad);}
    }
    check_native("native pump capacity",include_str!("../../../../contracts/fixtures/space-pump-contract.json"),
        "/recipe/process/modules/2/parallels",json!(4));
    check_native("native siphon tooling",include_str!("../../../../contracts/fixtures/gas-siphon-contract.json"),
        "/recipe/process/pipeLaw",json!("consume64"));
}

#[test]
fn loot_bag_opening_keeps_correlated_limits_and_conditional_debit() {
    let raw=include_str!("../../../../contracts/fixtures/lootbag-contract.json");
    for (path,bad) in [
        ("/recipe/process",Value::Null),
        ("/recipe/process/retryLimit",json!(1)),
        ("/recipe/process/budgetLaw",json!("itemUnits")),
        ("/recipe/process/retryState",json!("clearPerRetry")),
        ("/recipe/process/limitLaw",json!("afterItemResolution")),
        ("/recipe/process/debitLaw",json!("beforeOpening")),
        ("/recipe/process/emptyDraw",json!("discardEarlierDrops")),
        ("/recipe/inputs/0/choices/0/consume",json!({"kind":"consume"})),
        ("/recipe/inputs/0/choices/0/rule",json!({"kind":"exact"})),
        ("/recipe/outputs/0/quantity",Value::Null),
        ("/recipe/outputs/0/id",json!("item_missing")),
    ] {check_native("lootbag opening",raw,path,bad);}
    let raw=include_str!("../../../../contracts/fixtures/lootbag-unresolved-contract.json");
    check_native("unresolved native loot",raw,"/recipe/process/groups/0/drops/2/resolved",json!("item_missing"));
}

#[test]
fn gendustry_sampler_binds_observation_to_the_native_sample_item() {
    let raw=include_str!("../../../../contracts/fixtures/gendustry-sampler-contract.json");
    check_native("sampler native item",raw,"/recipe/process/outputPolicy/sampleRegistry",json!("minecraft:paper"));
    check_native("sampler native meta",raw,"/recipe/process/outputPolicy/sampleMeta",json!(1));
    check_native("sampler mandatory item binding",raw,"/recipe/process/outputPolicy",json!({"kind":"direct"}));
}

#[test]
fn extrautils_soul_retains_pending_result_and_personal_inventory_lifecycle() {
    let value:Value=serde_json::from_str(include_str!("../../../../contracts/fixtures/native-extrautils-soul-contract.json")).unwrap();
    let direct:Recipe=serde_json::from_value(value["recipe"].clone()).unwrap();
    assert_eq!(direct.id,recipe_id(&direct).unwrap(),"Actual Java Soul recipe must arrive with its original canonical identity, before any test rebinding");
    for raw in [include_str!("../../../../contracts/fixtures/extrautils-soul-contract.json"),
        include_str!("../../../../contracts/fixtures/native-extrautils-soul-contract.json")] {
    for (path,bad) in [
        ("/recipe/process/inventorySize",json!(9)),
        ("/recipe/process/healthDecrement",json!("0")),
        ("/recipe/process/minimumHealth",json!("0")),
        ("/recipe/process",Value::Null),
        ("/recipe/inputs/0/choices/0/rule",json!({"kind":"exact"})),
        ("/recipe/outputs/0/quantity/condition",json!("unconditional")),
    ] {check_native("Soul lifecycle",raw,path,bad);}
    let value:Value=serde_json::from_str(raw).unwrap();
    let id=&value["recipe"]["outputs"][0]["id"];
    let position=value["items"].as_array().unwrap().iter().position(|item|&item["id"]==id).unwrap();
    check_native("pending Soul meta",raw,&format!("/items/{position}/meta"),json!(0));
    }
}

#[test]
fn gendustry_replicator_replays_native_lifespan_and_full_queen_serialization() {
    for raw in [
        include_str!("../../../../contracts/fixtures/gendustry-replicator-0-contract.json"),
        include_str!("../../../../contracts/fixtures/gendustry-replicator-1-contract.json"),
        include_str!("../../../../contracts/fixtures/gendustry-replicator-2-contract.json"),
    ] {
        check_native("replicator lifespan",raw,"/recipe/process/roots/0/lifespan",Value::Null);
        let value:Value=serde_json::from_str(raw).unwrap();
        let output_id=&value["recipe"]["outputs"][0]["id"];
        let position=value["items"].as_array().unwrap().iter().position(|item|&item["id"]==output_id).unwrap();
        check_native("replicator changed health",raw,&format!("/items/{position}/nbt/value/Health/value"),json!("999"));
        check_native("replicator changed maximum health",raw,&format!("/items/{position}/nbt/value/MaxH/value"),json!("999"));
    }
}

#[test]
fn gendustry_mutatron_display_keeps_conditional_policy_without_fabricated_observations() {
    let raw=include_str!("../../../../contracts/fixtures/gendustry-mutatron-contract.json");
    check_native("mutatron",raw,"/recipe/process/outputPolicy/emptyCandidatesReturnNull",json!(false));
    check_native("mutatron",raw,"/recipe/outputs/0/change",Value::Null);
    let mut value:Value=serde_json::from_str(raw).unwrap();
    assert_eq!(value["recipe"]["outputs"][0]["change"]["samples"],json!([]));
    value["recipe"]["process"]["config"]["labwareConsumeChance"]=json!(100);
    value["recipe"]["process"]["labwareThreshold"]=json!(100);
    value["recipe"]["inputs"][2]["choices"][0]["consume"]=json!({"kind":"consume"});
    let (domain,source)=load(&value).unwrap();domain.validate(&source).unwrap();
    value["recipe"]["process"]=Value::Null;
    value["recipe"]["outputs"][0]["change"]=Value::Null;
    value["recipe"]["outputs"][0]["amount"]=json!("1");
    assert!(load(&value).and_then(|(d,s)|d.validate(&s)).is_err(),"Deleting mutation policy admitted a guaranteed display result");
}

#[test]
fn witchery_oven_cannot_change_nbt_classification_without_changing_admission() {
    let raw=include_str!("../../../../contracts/fixtures/witchery-oven-contract.json");
    check_native("oven byproduct classification",raw,"/recipe/process/byproduct",json!("exhale"));
    check_native("oven unconstrained NBT",raw,"/recipe/inputs/0/choices/0/rule",json!({"kind":"wildcard","meta":false,"nbt":true}));
    let value:Value=serde_json::from_str(raw).unwrap();
    check_native("wrong oven byproduct",raw,"/recipe/outputs/1/id",value["recipe"]["outputs"][0]["id"].clone());
    for raw in [
        include_str!("../../../../contracts/fixtures/witchery-oven-foul-contract.json"),
        include_str!("../../../../contracts/fixtures/witchery-oven-exhale-contract.json"),
        include_str!("../../../../contracts/fixtures/witchery-oven-hint-contract.json"),
        include_str!("../../../../contracts/fixtures/witchery-oven-breath-contract.json"),
        include_str!("../../../../contracts/fixtures/witchery-oven-none-contract.json"),
    ] {
        let value:Value=serde_json::from_str(raw).unwrap();
        let (domain,source)=load(&value).unwrap();domain.validate(&source).unwrap();
        let choice=&domain.recipes[0].inputs[0].choices[0];
        let items=domain.items.iter().map(|item|(item.id.as_str(),item)).collect();
        let anchor=domain.items.iter().find(|item|item.id==choice.id).unwrap();
        assert_eq!(choice.rule.indexes_example(anchor,&items),value["recipe"]["process"]["byproduct"]=="foul",
            "only the untagged foul branch may index this display anchor");
    }
}

#[test]
fn witchery_adapter_rows_survive_full_domain_and_reject_native_policy_changes() {
    for (name,raw,path,bad) in [
        ("oven",include_str!("../../../../contracts/fixtures/witchery-oven-contract.json"),"/recipe/process/baseTicks",json!(1)),
        ("kettle",include_str!("../../../../contracts/fixtures/witchery-kettle-contract.json"),"/recipe/process/waterMillibuckets",json!(1)),
        ("ritual",include_str!("../../../../contracts/fixtures/witchery-ritual-contract.json"),"/recipe/process/waterMillibuckets",json!(1)),
        ("infernal",include_str!("../../../../contracts/fixtures/witchery-infernal-contract.json"),"/recipe/process/visDivisor",json!(1)),
        ("infernal wildcard",include_str!("../../../../contracts/fixtures/witchery-infernal-wildcard-contract.json"),"/recipe/process/visDivisor",json!(1)),
    ] { check_native(name,raw,path,bad); }
}

#[test]
fn ae2_adapter_rows_preserve_facade_grid_and_all_nine_world_actions() {
    let facade=include_str!("../../../../contracts/fixtures/ae-facade-contract.json");
    check_native("facade",facade,"/recipe/outputs/0/amount",json!("1"));
    check_native("facade",facade,"/recipe/inputs/2/choices/0/rule/kind",json!("exact"));
    for (name,raw) in [
        ("charged ore",include_str!("../../../../contracts/fixtures/ae-world-locale-0-en_US-contract.json")),
        ("logic press",include_str!("../../../../contracts/fixtures/ae-world-locale-1-en_US-contract.json")),
        ("calculation press",include_str!("../../../../contracts/fixtures/ae-world-locale-2-en_US-contract.json")),
        ("engineering press",include_str!("../../../../contracts/fixtures/ae-world-locale-3-en_US-contract.json")),
        ("fluix",include_str!("../../../../contracts/fixtures/ae-world-locale-4-en_US-contract.json")),
        ("entangled singularity",include_str!("../../../../contracts/fixtures/ae-world-locale-5-en_US-contract.json")),
        ("certus growth",include_str!("../../../../contracts/fixtures/ae-world-locale-6-en_US-contract.json")),
        ("nether growth",include_str!("../../../../contracts/fixtures/ae-world-locale-7-en_US-contract.json")),
        ("fluix growth",include_str!("../../../../contracts/fixtures/ae-world-locale-8-en_US-contract.json")),
    ] {
        check_native(name,raw,"/recipe/outputs/0/quantity/condition",json!("unconditional"));
        let value:Value=serde_json::from_str(raw).unwrap();
        let (domain,_)=load(&value).unwrap();
        assert_eq!(domain.recipes[0].id,value["recipe"]["id"].as_str().unwrap(),"Java world recipe identity is not canonical");
        let other=value["items"].as_array().unwrap().iter()
            .find(|item|item["id"]!=value["recipe"]["outputs"][0]["id"]).unwrap();
        check_native(name,raw,"/recipe/outputs/0/id",other["id"].clone());
    }
    let raw=include_str!("../../../../contracts/fixtures/ae-world-locale-4-en_US-contract.json");
    let value:Value=serde_json::from_str(raw).unwrap();
    check_native("wrong fluix reagent",raw,"/recipe/inputs/1/choices/0/id",value["recipe"]["outputs"][0]["id"].clone());
    check_native("missing native world explanation",raw,"/recipe/notes",json!([]));
    check_native("wrong native explanation source",raw,"/recipe/notes/0/source",json!("invented translation"));
    let translated:Value=serde_json::from_str(include_str!("../../../../contracts/fixtures/ae-world-locale-4-zh_CN-contract.json")).unwrap();
    let (translated_domain,source)=load(&translated).unwrap();translated_domain.validate(&source).unwrap();
    assert_eq!(translated_domain.recipes[0].id,value["recipe"]["id"].as_str().unwrap());
    assert_ne!(translated["recipe"]["notes"][0]["text"],value["recipe"]["notes"][0]["text"]);
    let mut old=value.clone();old["recipe"]["process"]["description"]=json!("localized prose is not a process rule");
    assert!(load(&old).is_err(),"Unversioned localized process field was silently accepted");
    let raw=include_str!("../../../../contracts/fixtures/ae-world-locale-6-en_US-contract.json");
    let value:Value=serde_json::from_str(raw).unwrap();
    check_native("wrong seed registry",raw,"/recipe/inputs/0/choices/0/id",value["recipe"]["outputs"][0]["id"].clone());
}

#[test]
fn ae2_registered_wildcard_is_a_symbolic_input_not_a_positive_use_of_its_display() {
    let raw=include_str!("../../../../contracts/fixtures/ae-world-ender-notes-contract.json");
    let value:Value=serde_json::from_str(raw).unwrap();let(domain,source)=load(&value).unwrap();domain.validate(&source).unwrap();
    let choice=&domain.recipes[0].inputs[1].choices[0];
    let item=domain.items.iter().find(|item|item.id==choice.id).unwrap();
    assert_eq!(item.meta,0);assert!(!choice.rule.indexes_example(item,&BTreeMap::new()));
    check_native("AE2 literal ender input",raw,"/recipe/inputs/1/choices/0/rule/ore",json!("ingotIron"));
    check_native("AE2 literal ender input",raw,"/recipe/inputs/1/choices/0/rule/metadata",json!(-1));
    check_native("AE2 literal ender input",raw,"/recipe/process",Value::Null);
}

#[test]
fn world_acquisition_cannot_become_a_free_guaranteed_recipe() {
    let value:Value=serde_json::from_str(include_str!("../../../../contracts/fixtures/ae-world-locale-0-en_US-contract.json")).unwrap();
    let (domain,source)=load(&value).unwrap();
    domain.validate(&source).unwrap();
    let mut guaranteed=value.clone();
    guaranteed["recipe"]["outputs"][0]["quantity"]=Value::Null;
    guaranteed["recipe"]["outputs"][0]["amount"]=json!("1");
    assert!(load(&guaranteed).and_then(|(d,s)|d.validate(&s)).is_err(),"Native generation became guaranteed free production");
    let mut stripped=guaranteed.clone();stripped["recipe"]["process"]=Value::Null;
    assert!(load(&stripped).and_then(|(d,s)|d.validate(&s)).is_err(),"Deleting the native action bypassed acquisition conditions");
    let mut injected=value;
    injected["recipe"]["process"]["action"]=json!({"kind":"fluix","liquidTicks":61});
    assert!(load(&injected).and_then(|(d,s)|d.validate(&s)).is_err(),"A different native action bypassed its entity inputs");
}

#[test]
fn ae2_world_native_identities_survive_reidentified_item_facts() {
    for (name,raw,output,field,bad) in [
        ("wrong redstone",include_str!("../../../../contracts/fixtures/ae-world-locale-4-en_US-contract.json"),false,"registry",json!("minecraft:apple")),
        ("wrong seed",include_str!("../../../../contracts/fixtures/ae-world-locale-6-en_US-contract.json"),false,"registry",json!("minecraft:paper")),
        ("swapped press",include_str!("../../../../contracts/fixtures/ae-world-locale-1-en_US-contract.json"),true,"registry",json!("minecraft:book")),
        ("tagged growth result",include_str!("../../../../contracts/fixtures/ae-world-locale-6-en_US-contract.json"),true,"nbt",json!({"type":"compound","value":{"owner":{"type":"string","value":"must-not-survive"}}})),
    ] {
        let mut value:Value=serde_json::from_str(raw).unwrap();
        let slot=if name=="wrong redstone" {1} else {0};
        let path=if output {"/recipe/outputs/0/id".to_owned()} else {format!("/recipe/inputs/{slot}/choices/0/id")};
        let old=value.pointer(&path).unwrap().clone();
        let fact=value["items"].as_array_mut().unwrap().iter_mut().find(|i|i["id"]==old).unwrap();
        fact[field]=bad;
        let nbt:Option<crate::identity::Nbt>=serde_json::from_value(fact["nbt"].clone()).unwrap();
        let id=crate::identity::item_id(fact["registry"].as_str().unwrap(),fact["meta"].as_i64().unwrap() as i32,nbt.as_ref()).unwrap();
        fact["id"]=json!(id);
        *value.pointer_mut(&path).unwrap()=json!(id);
        if output {value["recipe"]["process"]["result"]=json!(id);}
        else {value["recipe"]["process"]["sources"][slot][0]=json!(id);}
        // The changed fact has a valid content identity; rejection must come
        // from the sealed native role, not the generic item-hash gate.
        let (domain,source)=load(&value).unwrap();
        let error=domain.validate(&source).unwrap_err();
        assert!(format!("{error:#}").contains("identity differs from native definition"),"{name}: {error:#}");
    }
    let raw=include_str!("../../../../contracts/fixtures/ae-world-locale-6-en_US-contract.json");
    check_native("growth liquid condition",raw,"/recipe/process/action/liquidMaterial",json!("water"));
}

#[test]
fn forestry_gui_free_owned_rows_replay_without_rewriting_java_recipe_identity() {
    let fixtures=std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../contracts/fixtures");
    for name in ["bottler","bottler-dynamic","fabricator","fermenter","moistener","carpenter"] {
        let raw=std::fs::read(fixtures.join(format!("forestry-{name}-semantic-contract.json"))).unwrap();
        let value:Value=serde_json::from_slice(&raw).unwrap();
        let recipe:Recipe=serde_json::from_value(value["recipe"].clone()).unwrap();
        assert_eq!(recipe.id,recipe_id(&recipe).unwrap(),"{name}: original Java identity differs");
        assert!(!recipe.notes.is_empty(),"{name}: GUI-free row lost native notes");
        let(mut domain,source)=load(&value).unwrap();
        domain.recipes=vec![recipe];
        domain.validate(&source).unwrap_or_else(|error|panic!("{name}: {error:#}"));
    }
}

#[test]
fn forestry_processing_adapter_rows_keep_native_allocation_and_conditions() {
    for (name,raw) in [
        ("fermenter",include_str!("../../../../contracts/fixtures/forestry-fermenter-contract.json")),
        ("moistener",include_str!("../../../../contracts/fixtures/forestry-moistener-contract.json")),
        ("carpenter",include_str!("../../../../contracts/fixtures/forestry-carpenter-contract.json")),
    ] {
        check_native(name,raw,"/recipe/process/selector",json!(65535));
        check_native(name,raw,"/recipe/inputs/0/choices/0/amount",json!("999"));
        check_native(name,raw,"/recipe/source/handler",json!("unadapted.Handler"));
        check_native(name,raw,"/recipe/duration",json!("100"));
        check_native(name,raw,"/recipe/energy",json!("100"));
    }
}

#[test]
fn chisel_adapter_rows_keep_native_tool_return_and_selection() {
    let raw=include_str!("../../../../contracts/fixtures/chisel-contract.json");
    check_native("chisel",raw,"/recipe/process/damagePoints",json!(2));
    check_native("chisel",raw,"/recipe/inputs/0/choices/0/rule/kind",json!("exact"));
    let value:Value=serde_json::from_str(raw).unwrap();
    let (domain,source)=load(&value).unwrap();domain.validate(&source).unwrap();
    let recipe=&domain.recipes[0];
    let returned=recipe.outputs.iter().find(|o|matches!(o.role,OutputRole::Return)).unwrap();
    assert_eq!(quantity_bounds(recipe,returned).unwrap(),(0,i64::from(i32::MAX)));
}

#[test]
fn binnie_genepool_quantity_bounds_cover_other_valid_genomes() {
    let raw:Value=serde_json::from_str(include_str!("../../../../contracts/fixtures/binnie-genetics-genepool-contract.json")).unwrap();
    let mut recipe:Recipe=serde_json::from_value(raw["recipe"].clone()).unwrap();
    let output=&mut recipe.outputs[0];
    if let Some(Quantity::Potential{nominal,sample,..})=&mut output.quantity {
        *nominal="10".into();*sample=Some("10".into());
    }
    assert_eq!(quantity_bounds(&recipe,&recipe.outputs[0]).unwrap(),(0,10));
    if let Some(Process::BinnieGenetics(binnie_genetics::GeneticsProcess::Genepool(process)))=&mut recipe.process {
        process.domain.root_class="forestry.apiculture.genetics.BeeHelper".into();
        process.domain.root="rootBees".into();
        process.domain.system_class="binnie.core.genetics.BeeBreedingSystem".into();
    }
    // A drone specimen displays 10 mB; other admitted bee members can yield 30 or 50 mB.
    assert_eq!(quantity_bounds(&recipe,&recipe.outputs[0]).unwrap(),(0,50));
}

#[test]
fn binnie_genetics_adapter_rows_keep_nine_native_resource_policies() {
    for (name,raw) in [
        ("acclimatiser",include_str!("../../../../contracts/fixtures/binnie-genetics-acclimatiser-contract.json")),
        ("analyser",include_str!("../../../../contracts/fixtures/binnie-genetics-analyser-contract.json")),
        ("sequencer",include_str!("../../../../contracts/fixtures/binnie-genetics-sequencer-contract.json")),
        ("splicer",include_str!("../../../../contracts/fixtures/binnie-genetics-splicer-contract.json")),
        ("isolator",include_str!("../../../../contracts/fixtures/binnie-genetics-isolator-contract.json")),
        ("inoculator",include_str!("../../../../contracts/fixtures/binnie-genetics-inoculator-contract.json")),
        ("genepool",include_str!("../../../../contracts/fixtures/binnie-genetics-genepool-contract.json")),
        ("incubator",include_str!("../../../../contracts/fixtures/binnie-genetics-incubator-contract.json")),
        ("polymeriser",include_str!("../../../../contracts/fixtures/binnie-genetics-polymeriser-contract.json")),
    ] {
        check_native(name,raw,"/recipe/inputs/0/choices/0/consume/kind",json!("keep"));
        check_native(name,raw,"/recipe/inputs/0/choices/0/rule/kind",json!("exact"));
    }
}

#[test]
fn native_adapter_rows_survive_full_domain_checks_and_reject_semantic_changes() {
    let cases=[
        ("gendustry template",include_str!("../../../../contracts/fixtures/gendustry-template-gui-free-contract.json"),"/recipe/process/maximumSamples",json!(9)),
        ("altar",include_str!("../../../../contracts/fixtures/blood-altar-contract.json"),"/recipe/outputs/0/quantity/nominal",json!("0")),
        ("binding",include_str!("../../../../contracts/fixtures/blood-binding-contract.json"),"/recipe/process/cooldown",json!(0)),
        ("meteor",include_str!("../../../../contracts/fixtures/blood-meteor-contract.json"),"/recipe/process/reagentCost",json!(0)),
        ("bottler",include_str!("../../../../contracts/fixtures/forestry-bottler-contract.json"),"/recipe/inputs/1/choices/0/amount",json!("999")),
        ("dynamic bottle",include_str!("../../../../contracts/fixtures/forestry-bottler-dynamic-contract.json"),"/recipe/process/callback/owner",json!("unadapted.Callback")),
        ("fabricator",include_str!("../../../../contracts/fixtures/forestry-fabricator-contract.json"),"/recipe/inputs/1/choices/0/consume/kind",json!("consume")),
    ];
    for(name,raw,path,bad)in cases {
        let value:Value=serde_json::from_str(raw).unwrap();let(domain,source)=load(&value).unwrap();
        domain.validate(&source).unwrap_or_else(|error|panic!("{name}: {error:#}"));
        let mut changed=value;*changed.pointer_mut(path).unwrap()=bad;
        let result=load(&changed).and_then(|(domain,source)|domain.validate(&source));
        assert!(result.is_err(),"{name}: native semantic change was accepted");
    }
}

#[test]
fn banner_native_rows_preserve_color_correlation_and_input_history() {
    for raw in [
        include_str!("../../../../contracts/fixtures/banner-mask-contract.json"),
        include_str!("../../../../contracts/fixtures/banner-catalyst-contract.json"),
        include_str!("../../../../contracts/fixtures/banner-no-dye-contract.json"),
        include_str!("../../../../contracts/fixtures/native-banner-mask-contract.json"),
        include_str!("../../../../contracts/fixtures/native-banner-catalyst-contract.json"),
        include_str!("../../../../contracts/fixtures/native-banner-no-dye-contract.json"),
    ] {
        check_native("banner",raw,"/recipe/process",Value::Null);
        check_native("banner",raw,"/recipe/process/maxPatterns",json!(7));
        check_native("banner",raw,"/recipe/outputs/0/change/action/color",json!(9));
        check_native("banner",raw,"/recipe/outputs/0/change",Value::Null);
        check_native("banner",raw,"/recipe/grid",json!({"width":3,"height":3,"mirror":false,"cells":[null,null,null,null,null,null,null,null,null]}));
        check_native("banner",raw,"/recipe/process/palette/0/color",json!(8));
        let v:Value=serde_json::from_str(raw).unwrap();
        let slot=v["recipe"]["process"]["banner"].as_u64().unwrap();
        check_native("banner cap",raw,&format!("/recipe/inputs/{slot}/choices/0/rule"),json!({"kind":"wildcard","meta":true,"nbt":true}));
    }
}
