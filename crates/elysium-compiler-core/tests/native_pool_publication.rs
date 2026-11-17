//! Exercise constructor discovery through the public Source -> Catalog boundary.
use elysium_compiler_core::{
    catalog::{self, Catalog, Table},
    domain::{self, Domain, Process, Recipe},
    source::Source,
};
use flate2::{write::GzEncoder, Compression};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::{collections::BTreeSet, fs, io::Write, path::Path};

fn specimen(index: usize) -> (Source, Domain) {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../contracts/fixtures");
    let raw: Value = serde_json::from_slice(
        &fs::read(root.join(format!("botania-native-pool-{index}-contract.json"))).unwrap(),
    ).unwrap();
    let source = Source::open(&root.join("source-semantic-40")).unwrap();
    let mut domain = Domain::load(&source).unwrap();
    let base = serde_json::to_value(&domain.items[0]).unwrap();
    for item in raw["items"].as_array().unwrap() {
        let mut value = base.clone();
        for key in ["id", "registry", "meta", "nbt", "tags"] {
            value[key] = item[key].clone();
        }
        let item: domain::Item = serde_json::from_value(value).unwrap();
        domain.items.retain(|old| old.id != item.id);
        domain.items.push(item);
    }
    domain.programs.extend(
        serde_json::from_value::<Vec<domain::programs::ProgramChunk>>(raw["programs"].clone()).unwrap(),
    );
    let mut recipe: Recipe = serde_json::from_value(raw["recipe"].clone()).unwrap();
    let mut category = domain.categories[0].clone();
    category.source = recipe.source.clone();
    category.id = domain::origin_id("category", &category.source).unwrap();
    let Some(Process::NativeBotaniaPool(pool)) = &recipe.process else { panic!("not a pool row") };
    category.program = Some(pool.program.clone());
    recipe.category = category.id.clone();
    recipe.id = domain::recipe_id(&recipe).unwrap();
    domain.categories.push(category);
    domain.recipes = vec![recipe];
    domain.validate(&source).unwrap();
    (source, domain)
}

fn write_specimen(source: &Source, domain: &Domain, target: &Path) {
    let mut manifest = source.manifest.clone();
    let records = serde_json::to_value(domain).unwrap();
    for file in &mut manifest.files {
        let bytes = if ["items", "categories", "recipes", "programs"].contains(&file.kind.as_str()) {
            let mut rows = records[&file.kind].as_array().unwrap().iter().collect::<Vec<_>>();
            rows.sort_by(|left, right| left["id"].as_str().cmp(&right["id"].as_str()));
            let mut decoded = Vec::new();
            for row in &rows {
                serde_json::to_writer(&mut decoded, row).unwrap();
                decoded.push(b'\n');
            }
            let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
            encoder.write_all(&decoded).unwrap();
            let bytes = encoder.finish().unwrap();
            file.rows = rows.len() as u64;
            file.decoded_bytes = decoded.len() as u64;
            file.bytes = bytes.len() as u64;
            file.sha256 = format!("{:x}", Sha256::digest(&bytes));
            bytes
        } else {
            source.read_file(file).unwrap()
        };
        let destination = target.join(&file.path);
        fs::create_dir_all(destination.parent().unwrap()).unwrap();
        fs::write(destination, bytes).unwrap();
    }
    manifest.id = manifest.digest().unwrap();
    fs::write(target.join("manifest.json"), serde_json::to_vec(&manifest).unwrap()).unwrap();
}

#[test]
fn conditional_constructor_roles_remain_discoverable_after_catalog_publication() {
    for index in 0..4 {
        let (source, domain) = specimen(index);
        let temporary = tempfile::tempdir().unwrap();
        let input = temporary.path().join("source");
        let output = temporary.path().join("catalog");
        write_specimen(&source, &domain, &input);
        catalog::compile(&input, &output).unwrap();
        let catalog = Catalog::current(&output).unwrap();
        catalog.verify().unwrap();
        let recipe = &domain.recipes[0];
        let expected: BTreeSet<_> = recipe.inputs[0].choices.iter().map(|c| c.id.as_str()).collect();
        let mut actual = BTreeSet::new();
        for file in catalog.manifest.files.iter().filter(|f| f.kind == "links") {
            let Table::Links(rows) = rmp_serde::from_slice(&catalog.read(file).unwrap()).unwrap() else { unreachable!() };
            for row in rows {
                if row.uses.contains(&recipe.id) { actual.insert(row.id); }
            }
        }
        assert!(expected.iter().all(|id| actual.contains(*id)), "pool row {index} lost input discovery");
        let output_id = &recipe.outputs[0].id;
        assert!(catalog.manifest.files.iter().filter(|f| f.kind == "recipes").any(|file| {
            let Table::Recipes(rows) = rmp_serde::from_slice(&catalog.read(file).unwrap()).unwrap() else { unreachable!() };
            rows.iter().any(|r| r.id == recipe.id && r.outputs[0].id == *output_id
                && r.outputs[0].change.as_ref().unwrap().samples.is_empty())
        }), "conditional constructor was replaced by a guaranteed output");
    }
}
