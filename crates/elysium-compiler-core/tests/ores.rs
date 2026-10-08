use elysium_compiler_core::catalog::Table;
use elysium_compiler_core::domain::{origin_id, Domain, Origin};
use elysium_compiler_core::source::Source;
use serde_json::{json, Value};
use std::path::PathBuf;

#[test]
fn ore_groups_retain_raw_templates_and_positions_separately_from_display_items() {
    let source = Source::open(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../contracts/fixtures/source"),
    )
    .unwrap();
    let original = Domain::load(&source).unwrap();
    let item = original
        .items
        .iter()
        .find(|row| row.registry == "minecraft:paper" && row.meta == 0 && row.nbt.is_none())
        .unwrap();
    let make_group = |name: &str, order, members| {
        let origin = Origin {
            owner: "Forge".into(),
            handler: "net.minecraftforge.oredict.OreDictionary".into(),
            key: name.into(),
        };
        json!({"id":origin_id("oregroup", &origin).unwrap(), "source":origin,"name":name,"order":order,"members":members})
    };
    let group = make_group("oreFixture", 0, 3);
    let empty = make_group("oreVacant", 1, 0);
    let gid = group["id"].as_str().unwrap();
    let entries: Vec<_> = (0..3).map(|index| json!({"id":format!("{gid}.member_{index:08x}"), "group":gid,"index":index,
        "template":{"registry":item.registry,"meta":if index==0 {32767} else {0},
            "nbt":if index==0 {json!({"type":"compound","value":{"raw":{"type":"string","value":"wildcard"}}})} else {Value::Null},
            "amount":if index==0 {"0"} else {"-4"}}, "display":if index==2 {Value::Null} else {json!(item.id)}})).collect();
    let mut value = serde_json::to_value(original).unwrap();
    value["oreGroups"] = json!([group.clone(), empty]);
    value["oreMembers"] = json!(entries);
    let valid: Domain = serde_json::from_value(value.clone()).unwrap();
    valid.validate(&source).unwrap();
    for (kind, rows) in [
        ("ore-groups", value["oreGroups"].clone()),
        ("ore-members", value["oreMembers"].clone()),
    ] {
        let table: Table = serde_json::from_value(json!({"kind":kind,"records":rows})).unwrap();
        let bytes = rmp_serde::to_vec_named(&table).unwrap();
        assert_eq!(
            serde_json::to_value(rmp_serde::from_slice::<Table>(&bytes).unwrap()).unwrap(),
            serde_json::to_value(table).unwrap()
        );
    }
    for (field, invalid) in [
        ("index", json!(8)),
        ("group", json!("oregroup_missing")),
        ("display", json!("item_missing")),
    ] {
        let mut broken = value.clone();
        broken["oreMembers"][0][field] = invalid;
        assert!(
            serde_json::from_value::<Domain>(broken)
                .unwrap()
                .validate(&source)
                .is_err(),
            "accepted invalid {field}"
        );
    }
    for (field, invalid) in [
        ("meta", json!(5)),
        ("amount", json!("1.5")),
        ("amount", json!("2147483648")),
        ("amount", json!("-2147483649")),
        ("registry", json!("minecraft:stone")),
        ("nbt", json!({"type":"compound","value":{}})),
    ] {
        let mut broken = value.clone();
        broken["oreMembers"][1]["template"][field] = invalid;
        assert!(
            serde_json::from_value::<Domain>(broken)
                .unwrap()
                .validate(&source)
                .is_err(),
            "accepted invalid template {field}"
        );
    }
    let mut missing = value.clone();
    missing["oreMembers"].as_array_mut().unwrap().pop();
    assert!(serde_json::from_value::<Domain>(missing)
        .unwrap()
        .validate(&source)
        .is_err());
    let mut count = value.clone();
    count["oreGroups"][0]["members"] = json!(2);
    assert!(serde_json::from_value::<Domain>(count)
        .unwrap()
        .validate(&source)
        .is_err());
    let mut order = value;
    order["oreGroups"][1]["order"] = json!(0);
    assert!(serde_json::from_value::<Domain>(order)
        .unwrap()
        .validate(&source)
        .is_err());
}
