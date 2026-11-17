use elysium_compiler_core::domain::{content_id, origin_id, Domain, Origin};
use elysium_compiler_core::source::Source;
use serde_json::{json, Value};
use std::{collections::BTreeMap, path::PathBuf};

// Replay the current Java-owned native specimens through the public domain boundary.
fn fixture(name: &str) -> (Source, Domain) {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../contracts/fixtures");
    let raw: Value = serde_json::from_slice(&std::fs::read(root.join(name)).unwrap()).unwrap();
    let source = Source::open(&root.join("source-semantic-40")).unwrap();
    let mut domain = Domain::load(&source).unwrap();
    let base = serde_json::to_value(&domain.items[0]).unwrap();
    for item in raw["items"].as_array().unwrap() {
        if domain.items.iter().any(|old| old.id == item["id"].as_str().unwrap()) { continue; }
        let mut value = base.clone();
        for key in ["id", "registry", "meta", "nbt"] { value[key] = item[key].clone(); }
        value["icon"] = Value::Null; value["aspects"] = Value::Null; value["order"] = Value::Null;
        value["tags"] = json!([]);
        domain.items.push(serde_json::from_value(value).unwrap());
    }
    let groups: Vec<Vec<Value>> = if raw.get("domains").is_some() {
        serde_json::from_value(raw["domains"].clone()).unwrap()
    } else { vec![serde_json::from_value(raw["reference"]["records"].clone()).unwrap()] };
    for group in &groups {
        let origin: Origin = serde_json::from_value(group[0]["data"]["source"].clone()).unwrap();
        let mut category = domain.categories[0].clone();
        category.source = origin; category.id = origin_id("category", &category.source).unwrap();
        category.reference = Some(group[0]["domain"].as_str().unwrap().to_owned()); category.program = None; category.icon = None; category.machines.clear(); domain.categories.push(category);
    }
    domain.references = serde_json::from_value(json!(groups.into_iter().flatten().collect::<Vec<_>>())).unwrap();
    (source, domain)
}

// Keep hashes valid so negative cases reach semantic validation rather than checksum rejection.
fn rekey(domain: &mut Domain, kind: &str) {
    let mut groups = BTreeMap::<String, Vec<usize>>::new();
    for (index, row) in domain.references.iter().enumerate() { groups.entry(row.domain.clone()).or_default().push(index); }
    for (old, indices) in groups {
        let data: Vec<_> = indices.iter().map(|index| &domain.references[*index].data).collect();
        let id = content_id("reference", &json!({"kind": kind, "records": data})).unwrap();
        for index in indices {
            domain.references[index].domain = id.clone();
            domain.references[index].id = content_id("refrow", &domain.references[index]).unwrap();
        }
        for category in &mut domain.categories { if category.reference.as_deref() == Some(&old) { category.reference = Some(id.clone()); } }
    }
}

#[test]
fn fixed_fireworks_keep_actual_callback_quantity_and_flight_policy() {
    let (source, domain) = fixture("fireworks-fixed-contract.json");
    domain.validate(&source).unwrap();
    let mut changed = domain.clone();
    let mut row = serde_json::to_value(&changed.references[0]).unwrap(); row["data"]["outputAmount"] = json!("1");
    changed.references[0] = serde_json::from_value(row).unwrap();
    rekey(&mut changed, "fireworkRules");
    assert!(changed.validate(&source).is_err());
    let mut changed = domain.clone();
    for original in &mut changed.references {
        let mut row = serde_json::to_value(&original).unwrap();
        if row["data"]["label"] == "rocket-3-0" { row["data"]["output"]["amount"] = json!("1"); }
        *original = serde_json::from_value(row).unwrap();
    }
    rekey(&mut changed, "fireworkRules");
    assert!(changed.validate(&source).is_err());
}

#[test]
fn pinned_enderstorage_exposes_only_registered_global_and_personal_actions() {
    let (source, domain) = fixture("enderstorage-contract.json");
    domain.validate(&source).unwrap();
    let mut changed = domain.clone();
    let mut row = serde_json::to_value(&changed.references[0]).unwrap();
    let mut action = row["data"]["actions"][0].clone();
    action["key"] = json!(format!("{}-refreshView", action["key"].as_str().unwrap())); action["requiresNewSnapshot"] = json!(true);
    row["data"]["actions"].as_array_mut().unwrap().push(action); changed.references[0] = serde_json::from_value(row).unwrap();
    rekey(&mut changed, "enderStorageSnapshot");
    assert!(changed.validate(&source).is_err());
}

#[test]
fn malformed_firework_inputs_are_raw_state_without_unsafe_display_facts() {
    let (source,domain)=fixture("fireworks-fixed-contract.json");
    domain.validate(&source).unwrap();
    for slot in [9,1] {
        let mut changed=domain.clone();
        for original in &mut changed.references {
            let mut row=serde_json::to_value(&original).unwrap();
            if row["data"]["label"]=="fade-wrong-explosion" { row["data"]["failure"]["inputs"][0]["slot"]=json!(slot); }
            *original=serde_json::from_value(row).unwrap();
        }
        rekey(&mut changed,"fireworkRules");
        assert!(changed.validate(&source).is_err(),"raw failure state overwrote a real or out-of-bounds cell");
    }
}

#[test]
fn tgreg_display_binding_preserves_raw_template_and_rejects_unproved_nbt() {
    let (source, domain) = fixture("tconstruct-tgreg-display-contract.json");
    domain.validate(&source).unwrap();
    for material in [Some("Gold"), None] {
        let mut changed = domain.clone();
        let mut row = serde_json::to_value(&changed.references[2]).unwrap();
        if let Some(material) = material { row["data"]["displayBinding"]["material"] = json!(material); }
        else { row["data"].as_object_mut().unwrap().remove("displayBinding"); }
        changed.references[2] = serde_json::from_value(row).unwrap();
        rekey(&mut changed, "tconstructToolMaterials");
        assert!(changed.validate(&source).is_err());
    }
}
