use elysium_compiler_core::{domain::forestry::*,identity::Nbt};

#[test]
fn gtpp_pump_initializes_native_nbt_only_when_init_tag_is_absent() {
    let callback: SqueezerCallback = serde_json::from_value(serde_json::json!({
        "registry":"fixture:pump", "kind":"noContainerRule",
        "drainedRegistries":["fixture:pump"], "readEffect":"gtppPump",
        "minMeta":-2147483648i32, "maxMeta":2147483647i32, "compoundTags":[]
    })).expect("GT++ pump needs an explicit initialization effect");
    let program = SqueezerProgram { ordinary:vec![], containers:vec![], filled:vec![], dynamic:vec![callback] };
    for (meta, expected_meta, capacity) in [(0,0,2000),(1000,0,2000),(1001,1,8000),(1004,4,512000),(i32::MAX,4,512000)] {
        let stack = ForestryStack { registry:"fixture:pump".into(), meta, amount:3, nbt:None, ores:vec![] };
        let result = program.select_observed(&[Some(stack.clone())], None).unwrap();
        let expected = std::collections::BTreeMap::from([
            ("mMeta".into(), Nbt::Int {value:expected_meta.to_string()}),
            ("mInit".into(), Nbt::Byte {value:"1".into()}),
            ("mFluid".into(), Nbt::String {value:"@@@@@".into()}),
            ("mFluidAmount".into(), Nbt::Int {value:"0".into()}),
            ("mCapacity".into(), Nbt::Int {value:capacity.to_string()}),
            ("capacityInit".into(), Nbt::Byte {value:"1".into()}),
        ]);
        assert!(result.selected.is_none());
        assert_eq!(result.stock, vec![Some(ForestryStack {nbt:Some(Nbt::Compound{value:expected.clone()}), ..stack.clone()})]);
        for marker in [Nbt::Byte{value:"0".into()},Nbt::String{value:"present-wrong-type".into()}] {
            let mut tags=expected.clone(); tags.insert("mInit".into(),marker);tags.remove("mCapacity");
            let existing=ForestryStack{nbt:Some(Nbt::Compound{value:tags}),..stack.clone()};
            assert_eq!(program.select_observed(&[Some(existing.clone())],None).unwrap().stock,vec![Some(existing)],"Native hasKey does not inspect the value/type of mInit");
        }
        let retained=SqueezerRecipe {time:1,requirements:vec![Some(stack.clone())],fluid:None,remnant:None,chance:"00000000".into()};
        assert_eq!(program.select_observed(&[Some(stack.clone())],Some(&retained)).unwrap().stock,vec![Some(stack)]);
    }
}
    #[test]
    fn squeezer_new_callbacks_keep_all_drain_targets_disjoint_and_native_read_effects() {
        let callback: SqueezerCallback = serde_json::from_value(serde_json::json!({
            "registry":"fixture:plasma", "kind":"noContainerRule",
            "drainedRegistries":["fixture:plasma","fixture:empty"],
            "readEffect":"ensureCompound", "minMeta":1, "maxMeta":4,
            "compoundTags":["tankContents"]
        })).expect("Source callback descriptor must be admitted after its native closure is modeled");
        let offered=ForestryStack { registry:"fixture:plasma".into(),meta:2,amount:1,nbt:None,ores:vec![] };
        let program=SqueezerProgram {ordinary:vec![],containers:vec![],filled:vec![],dynamic:vec![callback]};
        let observed=program.select_observed(&[Some(offered.clone())],None).unwrap();
        assert!(observed.selected.is_none());
        assert_eq!(observed.stock[0].as_ref().unwrap().nbt,Some(Nbt::Compound{value:Default::default()}));
        let retained=SqueezerRecipe {time:1,requirements:vec![Some(offered.clone())],fluid:None,remnant:None,chance:"00000000".into()};
        assert_eq!(program.select_observed(&[Some(offered.clone())],Some(&retained)).unwrap().stock,vec![Some(offered.clone())]);
        for target in ["fixture:plasma","fixture:empty"] {
            let empty=ForestryStack{registry:target.into(),..offered.clone()};
            for key in [ContainerKey::Item{registry:target.into()},ContainerKey::Stack{stack:empty.clone()},ContainerKey::Ore{members:vec![empty.clone()]}] {
                let mut conflict=program.clone();conflict.containers.push(SqueezerContainer{key,empty:empty.clone(),time:1,remnant:None,chance:"00000000".into()});
                assert!(conflict.validate().is_err(),"Every possible copied-item target must stay disjoint");
            }
        }
        let outside=ForestryStack{meta:0,..offered.clone()};
        assert!(program.select(&[Some(outside)],None).is_err());
        let malformed=ForestryStack{nbt:Some(Nbt::Compound{value:std::collections::BTreeMap::from([("tankContents".into(),Nbt::String{value:"wrong native tag type".into()})])}),..offered};
        assert!(program.select(&[Some(malformed)],None).is_err());
    }

    #[test]
    fn squeezer_new_callbacks_gt_table_keeps_short_metadata_original_nbt_and_zero_drain() {
        let raw=serde_json::json!({"registry":"fixture:bw", "kind":"gtFixed", "rows":[{
            "meta":17,"fluid":{"registry":"water","amount":600,"nbt":null},
            "emptyRegistry":"minecraft:paper","emptyMeta":-3
        }]});
        let callback: SqueezerCallback=serde_json::from_value(raw.clone()).expect("Finite native GT callback tables must be modeled");
        let nbt=Some(Nbt::Compound{value:std::collections::BTreeMap::from([("retained".into(),Nbt::String{value:"input-nbt".into()})])});
        let offered=ForestryStack{registry:"fixture:bw".into(),meta:65553,amount:2,nbt:nbt.clone(),ores:vec![]};
        let empty=ForestryStack{registry:"minecraft:paper".into(),meta:0,amount:1,nbt,ores:vec![]};
        let container=SqueezerContainer{key:ContainerKey::Stack{stack:empty.clone()},empty,time:5,remnant:None,chance:"00000000".into()};
        let program=SqueezerProgram{ordinary:vec![],containers:vec![container],filled:vec![],dynamic:vec![callback]};
        let observed=program.select_observed(&[Some(offered.clone())],None).unwrap();
        let selected=observed.selected.unwrap();
        assert_eq!(selected.fluid.unwrap().amount,600);
        assert_eq!(selected.requirements,vec![Some(ForestryStack{amount:1,..offered.clone()})]);
        assert_eq!(observed.stock,vec![Some(offered.clone())]);
        assert!(program.select(&[Some(ForestryStack{amount:-1,..offered.clone()})],None).unwrap().is_none());
        let mut zero=raw.clone();zero["rows"][0]["fluid"]["amount"]=serde_json::json!(0);
        let mut changed=program.clone();changed.dynamic=vec![serde_json::from_value(zero).unwrap()];
        assert!(changed.select(&[Some(offered.clone())],None).unwrap().is_none(),"Native zero drain uses original BW item before any target replacement");
        let mut duplicate=raw.clone();duplicate["rows"].as_array_mut().unwrap().push(raw["rows"][0].clone());
        changed.dynamic=vec![serde_json::from_value(duplicate).unwrap()];assert!(changed.validate().is_err());
        let mut invalid=raw;invalid["rows"][0]["meta"]=serde_json::json!(65553);
        assert!(serde_json::from_value::<SqueezerCallback>(invalid).is_err());
        changed=program;changed.containers[0].key=ContainerKey::Item{registry:"fixture:bw".into()};
        assert!(changed.validate().is_err(),"GT source registry must be disjoint before a zero drain or short alias can be admitted");
    }


#[test]
fn real_gt_container_observations_replay(){
 let rows:serde_json::Value=serde_json::from_str(include_str!("../../../contracts/fixtures/forestry-gt-container-observations.json")).unwrap();
 for row in rows.as_array().unwrap(){
  let program:SqueezerProgram=serde_json::from_value(row["program"].clone()).unwrap();
  let before:Vec<Option<ForestryStack>>=serde_json::from_value(row["before"].clone()).unwrap();
  let observed=program.select_observed(&before,None).unwrap();
  assert_eq!(serde_json::to_value(observed.selected).unwrap(),row["selected"],"{}",row["name"]);
  assert_eq!(serde_json::to_value(observed.stock).unwrap(),row["after"],"{}",row["name"]);
 }
}

#[test]
fn real_gtpp_pump_read_effects_replay() {
    let rows: serde_json::Value = serde_json::from_str(include_str!("../../../contracts/fixtures/forestry-pump-observations.json")).unwrap();
    for row in rows.as_array().unwrap() {
        let program: SqueezerProgram = serde_json::from_value(row["program"].clone()).unwrap();
        let before: Vec<Option<ForestryStack>> = serde_json::from_value(row["before"].clone()).unwrap();
        let observed = program.select_observed(&before, None).unwrap();
        assert_eq!(serde_json::to_value(observed.stock).unwrap(), row["after"], "{}", row["name"]);
        assert!(observed.selected.is_none(), "{}", row["name"]);
    }
}
