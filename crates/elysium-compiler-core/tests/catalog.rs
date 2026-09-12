use elysium_compiler_core::catalog::{compile, Catalog, Table};
use elysium_compiler_core::domain::{content_id, recipe_id, Domain, SpeciesKind};
use elysium_compiler_core::identity::{fluid_id, item_id};
use elysium_compiler_core::source::Source;
use std::fs;
use std::path::PathBuf;

fn fixture() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../contracts/fixtures/source")
}

#[test]
fn java_facts_compile_into_deterministic_queryable_catalogs() {
    let directory = tempfile::tempdir().unwrap();
    let first = compile(&fixture(), directory.path()).unwrap();
    let repeated = compile(&fixture(), directory.path()).unwrap();
    assert_eq!(first.id, repeated.id);
    assert_eq!(first.path, repeated.path);
    let catalog = Catalog::current(directory.path()).unwrap();
    catalog.verify().unwrap();
    assert_eq!(catalog.manifest.id, first.id);
    assert_eq!(catalog.manifest.counts["recipes"], 14);
    assert_eq!(
        catalog.manifest.counts["index"],
        catalog.manifest.counts["recipes"]
    );
    assert_eq!(catalog.manifest.counts["browse"], 29);
    assert_eq!(catalog.manifest.counts["materials"], 2);
    assert_eq!(catalog.manifest.counts["circuits"], 1);
    assert_eq!(catalog.manifest.counts["species"], 3);
    assert_eq!(catalog.manifest.counts["mutations"], 2);
    assert_eq!(catalog.manifest.counts["topics"], 12);
    assert_eq!(catalog.manifest.counts["aspects"], 3);
    assert_eq!(catalog.manifest.counts["research"], 2);
    assert_eq!(catalog.manifest.counts["structures"], 1);
    assert_eq!(catalog.manifest.counts["shapes"], 7);
    assert_eq!(catalog.manifest.counts["blocks"], 2);
    assert_eq!(catalog.manifest.counts["models"], 4);
    assert_eq!(catalog.manifest.counts["builds"], 2);
    let stone = item_id("minecraft:stone", 0, None).unwrap();
    let water = fluid_id("water", None).unwrap();
    let mut recipe_id = String::new();
    for file in &catalog.manifest.files {
        if file.kind == "image" {
            continue;
        }
        let table: Table = rmp_serde::from_slice(&catalog.read(file).unwrap()).unwrap();
        match table {
            Table::Materials(rows) => {
                let alloy = rows.iter().find(|row| row.source.key == "alloy").unwrap();
                assert_eq!(alloy.components[0].amount, "9007199254740993");
                assert_eq!(alloy.parts[0].content.as_ref().unwrap().denominator, "9");
                assert_eq!(alloy.color, 0xff807060);
                assert!(rows
                    .iter()
                    .any(|row| row.id == alloy.components[0].material));
            }
            Table::Circuits(rows) => {
                assert_eq!(rows[0].steps[0].item, stone);
                assert_eq!(rows[0].steps[0].tier.as_ref().unwrap().voltage, "32");
            }
            Table::Aspects(rows) => {
                let light = rows.iter().find(|row| row.source.key == "light").unwrap();
                assert_eq!(light.components.len(), 2);
                assert_eq!(light.discovered, Some(false));
                assert_eq!(
                    rows.iter()
                        .find(|row| row.source.key == "air")
                        .unwrap()
                        .discovered,
                    None
                );
            }
            Table::Research(rows) => {
                let study = rows.iter().find(|row| row.source.key == "ALCHEMY").unwrap();
                assert_eq!(study.completed, Some(false));
                assert_eq!(study.parents[0].completed, None);
                assert_eq!(study.hidden_parents[0].key, "@fixture_scanned");
                assert_eq!(study.hidden_parents[0].id, None);
                assert_eq!(study.item_triggers.as_slice(), std::slice::from_ref(&stone));
            }
            Table::Items(rows) => {
                assert_eq!(
                    rows.iter()
                        .find(|row| row.id == stone)
                        .unwrap()
                        .aspects
                        .as_ref()
                        .unwrap()[0]
                        .amount,
                    "3"
                );
            }
            Table::Tracks(rows) => {
                assert_eq!(rows.len(), 1);
                let frames = &rows[0].frames;
                assert_eq!(frames.iter().map(|frame| frame.ticks).sum::<u32>(), 16);
                assert!(frames[0].areas.is_empty());
                assert_eq!(frames[2].areas.len(), 2);
            }
            Table::Structures(rows) => {
                assert_eq!(rows[0].controller, stone);
                assert_eq!(rows[0].pieces[0].size, [3, 3, 3]);
                assert_eq!(rows[0].pieces[0].anchors, [[1, 1, 0]]);
                assert_eq!(rows[0].pieces[0].cells, 26);
                assert_eq!(rows[0].variants.len(), 2);
                assert_eq!(rows[0].variants[0].probe.count, 1);
                assert_eq!(rows[0].variants[1].probe.channels["coil"], 2);
                assert!(rows[0]
                    .variants
                    .iter()
                    .all(|variant| variant.build.is_some() && variant.problem.is_none()));
            }
            Table::Blocks(rows) => {
                let controller = rows
                    .iter()
                    .find(|block| block.registry == "fixture:controller")
                    .unwrap();
                let nbt = serde_json::to_value(&controller.nbt).unwrap();
                assert_eq!(nbt["value"]["energy"]["value"], "9007199254740993");
            }
            Table::Builds(rows) => {
                for row in rows {
                    assert!(row.rendered);
                    assert!(row
                        .palette
                        .iter()
                        .all(|entry| entry.model.is_some() && entry.problem.is_none()));
                    let expanded = row.probe.count == 4;
                    assert_eq!(row.cells, if expanded { 34 } else { 26 });
                    assert_eq!(row.size, [3, 3, if expanded { 4 } else { 3 }]);
                    assert_eq!(row.origin, [-1, 65, 0]);
                    assert_eq!(row.controller, [1, 1, 0]);
                    assert_eq!(row.result, Some(-1));
                    assert_eq!(
                        row.probe.channels.get("coil").copied(),
                        if expanded { Some(2) } else { None }
                    );
                }
            }
            Table::Shapes(rows) => {
                assert_eq!(rows.iter().map(|row| row.cells.len()).sum::<usize>(), 60);
                assert!(rows
                    .iter()
                    .flat_map(|row| &row.cells)
                    .any(|cell| cell.at == [1, 1, 1] && cell.index == 1));
            }
            Table::Models(rows) => {
                assert_eq!(
                    rows.iter()
                        .filter(|model| model.hidden && model.faces.is_empty())
                        .count(),
                    1
                );
                assert_eq!(
                    rows.iter().map(|model| model.faces.len()).sum::<usize>(),
                    19
                );
                assert!(rows.iter().filter(|model| !model.hidden).any(|model| model
                    .faces
                    .iter()
                    .all(|face| face
                        .vertices
                        .iter()
                        .all(|vertex| vertex.at[1].parse::<f32>().unwrap() >= 0.5))));
                assert!(rows
                    .iter()
                    .any(|model| model.faces.iter().any(|face| face.pass
                        == elysium_compiler_core::domain::Pass::Blend
                        && face.vertices[0].color == 0x4be8db80)));
            }
            Table::Topics(rows) => {
                assert!(rows.iter().any(|row| row.terms.contains("hejin")));
                assert!(rows.iter().any(|row| row.terms.contains("dianlu")));
                assert!(rows.iter().any(|row| row.terms.contains("mifeng")));
                assert!(rows.iter().any(|row| row.terms.contains("shumu")));
                assert!(rows.iter().any(|row| row.terms.contains("duofangkuai")));
            }
            Table::Species(rows) => {
                let bee = rows
                    .iter()
                    .find(|row| row.kind == SpeciesKind::Bee)
                    .unwrap();
                assert_eq!(bee.products[0].chance.as_ref().unwrap().numerator, "3");
                assert_eq!(bee.products[0].chance.as_ref().unwrap().denominator, "10");
                assert!(bee.genes[0].value.is_none());
                let tree = rows
                    .iter()
                    .find(|row| row.kind == SpeciesKind::Tree)
                    .unwrap();
                assert!(tree.products[0].chance.is_none());
                assert!(tree.blacklisted);
                assert_eq!(tree.fruit_compatible, Some(false));
            }
            Table::Mutations(rows) => {
                assert!(rows
                    .iter()
                    .any(|row| row.chance.numerator == "3" && row.chance.denominator == "40"));
                assert!(rows.iter().any(|row| row.parents[0] == row.parents[1]));
                assert_eq!(rows[0].genes[0].allele, "fixture.fast");
            }
            Table::Lineage(rows) => {
                assert_eq!(rows.len(), 3);
                assert_eq!(rows.iter().map(|row| row.origins.len()).sum::<usize>(), 2);
                assert_eq!(rows.iter().map(|row| row.crosses.len()).sum::<usize>(), 3);
            }
            Table::Index(rows) => {
                let machine = rows
                    .iter()
                    .find(|row| row.handler == "fixture:machine")
                    .unwrap();
                assert_eq!(machine.owner, "fixture");
                assert_eq!(machine.targets, [water.clone(), stone.clone()]);
            }
            Table::Recipes(rows) => {
                let recipe = rows.iter().find(|row| row.source.key == "machine").unwrap();
                recipe_id = recipe.id.clone();
                assert_eq!(recipe.inputs[0].choices[0].amount, "9007199254740993");
                assert_eq!(recipe.energy.as_deref(), Some("9223372036854775807"));
                assert_eq!(recipe.outputs[0].chance.numerator, "1");
                assert_eq!(recipe.outputs[0].chance.denominator, "3");
                let arcane = rows.iter().find(|row| row.source.key == "arcane").unwrap();
                assert_eq!(
                    arcane.grid.as_ref().unwrap().cells,
                    [Some(0), None, None, None]
                );
                assert_eq!(arcane.magic.as_ref().unwrap().aspects[0].amount, "7");
                assert_eq!(arcane.inputs[0].choices.len(), 2);
                let mut changed = arcane.clone();
                changed.magic.as_mut().unwrap().research[0].completed = Some(true);
                assert_eq!(
                    elysium_compiler_core::domain::recipe_id(&changed).unwrap(),
                    arcane.id
                );
                changed.magic.as_mut().unwrap().aspects[0].amount = "8".to_owned();
                assert_ne!(
                    elysium_compiler_core::domain::recipe_id(&changed).unwrap(),
                    arcane.id
                );
                changed = arcane.clone();
                changed.grid.as_mut().unwrap().cells.swap(0, 3);
                assert_ne!(
                    elysium_compiler_core::domain::recipe_id(&changed).unwrap(),
                    arcane.id
                );
                let imprint = rows.iter().find(|row| row.source.key == "imprint").unwrap();
                let mut observed = imprint.clone();
                observed.outputs[0].id = "another_example".to_owned();
                observed.outputs[0].amount = "2".to_owned();
                observed.outputs[0].change.as_mut().unwrap().samples[0].amount = "2".to_owned();
                assert_eq!(
                    elysium_compiler_core::domain::recipe_id(&observed).unwrap(),
                    imprint.id
                );
                if let elysium_compiler_core::domain::Edit::Patch { set, .. } =
                    &mut observed.outputs[0].change.as_mut().unwrap().action
                {
                    let value = set.remove("mark").unwrap();
                    set.insert("different".to_owned(), value);
                }
                assert_ne!(
                    elysium_compiler_core::domain::recipe_id(&observed).unwrap(),
                    imprint.id
                );
                let wand = rows
                    .iter()
                    .find(|row| row.source.key == "wand_core")
                    .unwrap();
                assert_eq!(
                    wand.inputs.len(),
                    8,
                    "screws and conductors occupy separate slots"
                );
                assert!(wand
                    .inputs
                    .iter()
                    .flat_map(|input| &input.choices)
                    .all(|choice| choice.amount == "1"));
                let payment = wand.magic.as_ref().unwrap().payment.as_ref().unwrap();
                assert_eq!(payment.capacity, 400);
                assert!(payment.preserve);
                assert!(rows
                    .iter()
                    .find(|row| row.source.key == "wand_staff")
                    .unwrap()
                    .magic
                    .as_ref()
                    .unwrap()
                    .payment
                    .is_none());
                let creative = rows
                    .iter()
                    .find(|row| row.source.key == "wand_creative")
                    .unwrap()
                    .magic
                    .as_ref()
                    .unwrap();
                assert!(creative.creative && creative.aspects.is_empty());
            }
            Table::Browse(rows) => {
                let item = rows.iter().find(|row| row.id == stone).unwrap();
                assert_eq!(item.name, "Stone 石头");
                assert!(item.terms.contains("shitou"));
                assert!(item.group.is_some());
            }
            Table::Textures(rows) => {
                assert_eq!(rows.len(), 1);
                let sprite = &rows[0].frames[0];
                let file = catalog
                    .manifest
                    .files
                    .iter()
                    .find(|file| file.path == sprite.path)
                    .unwrap();
                let image = image::load_from_memory(&catalog.read(file).unwrap())
                    .unwrap()
                    .into_rgba8();
                assert_eq!(
                    image.get_pixel(sprite.x, sprite.y).0,
                    [0x70, 0x60, 0x50, 0xff]
                );
            }
            _ => {}
        }
    }
    let descriptor = catalog
        .manifest
        .files
        .iter()
        .find(|file| file.kind == "links")
        .unwrap();
    let Table::Links(links) = rmp_serde::from_slice(&catalog.read(descriptor).unwrap()).unwrap()
    else {
        panic!("links table")
    };
    assert_eq!(
        links.iter().find(|links| links.id == stone).unwrap().uses,
        [recipe_id.clone()]
    );
    assert_eq!(
        links
            .iter()
            .find(|links| links.id == stone)
            .unwrap()
            .topics
            .len(),
        8
    );
    assert_eq!(
        links
            .iter()
            .find(|links| links.id == water)
            .unwrap()
            .recipes,
        [recipe_id]
    );
    assert!(fs::read_dir(directory.path().join("catalogs"))
        .unwrap()
        .all(|entry| !entry
            .unwrap()
            .file_name()
            .to_string_lossy()
            .starts_with(".capture-")));
}

#[test]
fn invalid_facts_and_damaged_catalogs_cannot_replace_a_published_snapshot() {
    let source = Source::open(&fixture()).unwrap();
    let domain = Domain::load(&source).unwrap();
    let mut registered = domain.clone();
    let mut repeated = registered.mutations[0].clone();
    repeated.occurrence = 1;
    repeated.id = content_id("mutation", &repeated).unwrap();
    assert_ne!(repeated.id, registered.mutations[0].id);
    registered.mutations.push(repeated);
    registered.validate(&source).unwrap();
    let repeated = registered.mutations.last_mut().unwrap();
    repeated.occurrence = 2;
    repeated.id = content_id("mutation", repeated).unwrap();
    assert!(
        registered.validate(&source).is_err(),
        "accepted a missing mutation registration"
    );
    for issue in [
        "sample", "amount", "count", "default", "input", "tag", "tools", "filter", "base",
    ] {
        let mut invalid = domain.clone();
        let key = if matches!(issue, "tools" | "filter" | "base") {
            "inherit"
        } else {
            "imprint"
        };
        let recipe = invalid
            .recipes
            .iter_mut()
            .find(|recipe| recipe.source.key == key)
            .unwrap();
        let input_id = recipe.inputs[0].choices[0].id.clone();
        let output = &mut recipe.outputs[0];
        let change = output.change.as_mut().unwrap();
        match issue {
            "sample" => change.samples[1].id = change.samples[0].id.clone(),
            "amount" => change.samples[1].amount = "2".to_owned(),
            "count" => {
                change.samples.pop();
            }
            "default" => output.id = change.samples[1].id.clone(),
            "input" => change.input = 99,
            "tag" => {
                if let elysium_compiler_core::domain::Edit::Patch { set, .. } = &mut change.action {
                    let value = set.remove("mark").unwrap();
                    set.insert("other".to_owned(), value);
                }
            }
            "tools" => {
                if let elysium_compiler_core::domain::Edit::Merge { tools, .. } = &mut change.action
                {
                    *tools = false;
                }
            }
            "filter" => {
                if let elysium_compiler_core::domain::Edit::Merge { keys, .. } = &mut change.action
                {
                    *keys = Some(Vec::new());
                }
            }
            "base" => {
                if let elysium_compiler_core::domain::Edit::Merge { base, .. } = &mut change.action
                {
                    base.id = input_id;
                }
            }
            _ => unreachable!(),
        }
        recipe.id = recipe_id(recipe).unwrap();
        assert!(
            invalid.validate(&source).is_err(),
            "accepted invalid changed output {issue}"
        );
    }
    for issue in [
        "limit", "overlap", "keys", "present", "absent", "payment", "charge", "capacity",
        "preserve", "creative",
    ] {
        let mut invalid = domain.clone();
        let recipe = invalid
            .recipes
            .iter_mut()
            .find(|recipe| {
                recipe.source.key
                    == if issue == "creative" {
                        "wand_creative"
                    } else {
                        "wand_core"
                    }
            })
            .unwrap();
        match issue {
            "limit" | "overlap" => {
                if let elysium_compiler_core::domain::Edit::Patch { set, limits } =
                    &mut recipe.outputs[0].change.as_mut().unwrap().action
                {
                    if issue == "limit" {
                        limits.insert("fire".to_owned(), 10);
                    } else {
                        set.insert(
                            "fire".to_owned(),
                            elysium_compiler_core::identity::Nbt::Int {
                                value: "400".to_owned(),
                            },
                        );
                    }
                }
            }
            "keys" | "present" | "absent" => {
                if let elysium_compiler_core::domain::Match::Tags {
                    keys,
                    present,
                    absent,
                    ..
                } = &mut recipe.inputs[0].choices[0].rule
                {
                    match issue {
                        "keys" => keys.push("missing".to_owned()),
                        "present" => present.push("sceptre".to_owned()),
                        _ => absent.insert(0, "cap".to_owned()),
                    }
                }
            }
            "payment" => {
                recipe
                    .magic
                    .as_mut()
                    .unwrap()
                    .payment
                    .as_mut()
                    .unwrap()
                    .input = 1
            }
            "charge" => {
                recipe
                    .magic
                    .as_mut()
                    .unwrap()
                    .payment
                    .as_mut()
                    .unwrap()
                    .charges
                    .values_mut()
                    .for_each(|key| *key = "wrong".to_owned());
            }
            "capacity" => {
                recipe
                    .magic
                    .as_mut()
                    .unwrap()
                    .payment
                    .as_mut()
                    .unwrap()
                    .capacity += 1
            }
            "preserve" => {
                recipe
                    .magic
                    .as_mut()
                    .unwrap()
                    .payment
                    .as_mut()
                    .unwrap()
                    .preserve = false
            }
            "creative" => recipe.magic.as_mut().unwrap().creative = false,
            _ => unreachable!(),
        }
        recipe.id = recipe_id(recipe).unwrap();
        assert!(
            invalid.validate(&source).is_err(),
            "accepted invalid wand {issue}"
        );
    }
    for issue in ["quantity", "fraction", "reference", "slot", "view"] {
        let mut invalid = domain.clone();
        match issue {
            "quantity" => {
                invalid.recipes[0].inputs[0].choices[0].amount = "9007199254740993.0".to_owned()
            }
            "fraction" => invalid.recipes[0].outputs[0].chance.denominator = "0".to_owned(),
            "reference" => invalid.recipes[0].outputs[0].id = "fluid_missing".to_owned(),
            "slot" => {
                let repeated = invalid.recipes[0].outputs[0].clone();
                invalid.recipes[0].outputs.push(repeated);
            }
            "view" => invalid.recipes[0].view = Some("view_missing".to_owned()),
            _ => unreachable!(),
        }
        invalid.recipes[0].id = recipe_id(&invalid.recipes[0]).unwrap();
        assert!(
            invalid.validate(&source).is_err(),
            "accepted invalid {issue}"
        );
    }
    for issue in [
        "empty", "duration", "budget", "bounds", "nan", "overlap", "repeat", "missing", "orphan",
    ] {
        let mut invalid = domain.clone();
        let old = invalid.tracks[0].id.clone();
        let frames = &mut invalid.tracks[0].frames;
        match issue {
            "empty" => frames.clear(),
            "duration" => frames[0].ticks = 0,
            "budget" => frames[0].ticks = 72000,
            "bounds" => frames[1].areas[0][2] = "1.1".to_owned(),
            "nan" => frames[1].areas[0][2] = "NaN".to_owned(),
            "overlap" => {
                let repeated = frames[1].areas[0].clone();
                frames[1].areas.push(repeated);
            }
            "repeat" => frames.insert(1, frames[0].clone()),
            "orphan" => {
                let mut orphan = invalid.tracks[0].clone();
                orphan.frames[0].ticks += 1;
                orphan.id = content_id("track", &orphan).unwrap();
                invalid.tracks.push(orphan);
            }
            "missing" => {}
            _ => unreachable!(),
        }
        invalid.tracks[0].id = content_id("track", &invalid.tracks[0]).unwrap();
        let replacement = if issue == "missing" {
            "track_missing"
        } else {
            &invalid.tracks[0].id
        };
        for view in &mut invalid.views {
            let old_view = view.id.clone();
            for element in &mut view.elements {
                if let elysium_compiler_core::domain::Element::Clip { track, .. } = element {
                    if track == &old {
                        *track = replacement.to_owned();
                    }
                }
            }
            view.id = content_id("view", view).unwrap();
            for recipe in &mut invalid.recipes {
                if recipe.view.as_ref() == Some(&old_view) {
                    recipe.view = Some(view.id.clone());
                }
            }
        }
        assert!(
            invalid.validate(&source).is_err(),
            "accepted invalid UI motion {issue}"
        );
    }
    for issue in ["component", "ratio", "fluid_ratio", "circuit_item", "tier"] {
        let mut invalid = domain.clone();
        let alloy = invalid
            .materials
            .iter_mut()
            .find(|row| row.source.key == "alloy")
            .unwrap();
        match issue {
            "component" => alloy.components[0].material = "material_missing".to_owned(),
            "ratio" => alloy.parts[0].content.as_mut().unwrap().denominator = "0".to_owned(),
            "fluid_ratio" => alloy.parts[1].content = alloy.parts[0].content.clone(),
            "circuit_item" => invalid.circuits[0].steps[0].item = "item_missing".to_owned(),
            "tier" => invalid.circuits[0].steps[0].tier = None,
            _ => unreachable!(),
        }
        assert!(
            invalid.validate(&source).is_err(),
            "accepted invalid industry {issue}"
        );
    }
    let output = tempfile::tempdir().unwrap();
    for issue in [
        "parent",
        "root",
        "condition",
        "rate",
        "tree_rate",
        "gene",
        "member",
    ] {
        let mut invalid = domain.clone();
        let tree = invalid
            .species
            .iter_mut()
            .find(|row| row.kind == SpeciesKind::Tree)
            .unwrap();
        match issue {
            "parent" => invalid.mutations[0].parents[0] = "species_missing".to_owned(),
            "root" => invalid.mutations[0].result = tree.id.clone(),
            "condition" => invalid.mutations[0].conditions[0] = "text_missing".to_owned(),
            "rate" => invalid.mutations[0].chance.numerator = "101".to_owned(),
            "tree_rate" => tree.products[0].chance = Some(invalid.mutations[0].chance.clone()),
            "gene" => {
                let repeated = tree.genes[0].clone();
                tree.genes.push(repeated);
            }
            "member" => tree.members[0].item = "item_missing".to_owned(),
            _ => unreachable!(),
        }
        invalid.mutations[0].parents.sort();
        invalid.mutations[0].id = content_id("mutation", &invalid.mutations[0]).unwrap();
        assert!(
            invalid.validate(&source).is_err(),
            "accepted invalid genetics {issue}"
        );
    }
    for issue in ["cycle", "aspect", "research", "knowledge"] {
        let mut invalid = domain.clone();
        let index = invalid
            .research
            .iter()
            .position(|row| row.source.key == "ALCHEMY")
            .unwrap();
        match issue {
            "cycle" => {
                let aspect = &mut invalid.aspects[0];
                aspect.components = vec![aspect.id.clone(), aspect.id.clone()];
            }
            "aspect" => {
                invalid
                    .items
                    .iter_mut()
                    .find(|row| row.registry == "minecraft:stone")
                    .unwrap()
                    .aspects
                    .as_mut()
                    .unwrap()[0]
                    .aspect = "aspect_missing".to_owned()
            }
            "research" => invalid.research[index].parents[0].key = "MISSING".to_owned(),
            "knowledge" => invalid.research[index].parents[0].completed = Some(true),
            _ => unreachable!(),
        }
        assert!(
            invalid.validate(&source).is_err(),
            "accepted invalid magic {issue}"
        );
    }
    for issue in [
        "cost",
        "research",
        "central",
        "grid",
        "candidate",
        "cost_view",
    ] {
        let mut invalid = domain.clone();
        let recipe = invalid
            .recipes
            .iter_mut()
            .find(|row| {
                row.source.key
                    == if issue == "central" {
                        "infusion"
                    } else {
                        "arcane"
                    }
            })
            .unwrap();
        match issue {
            "cost" => {
                recipe.magic.as_mut().unwrap().aspects[0].aspect = "aspect_missing".to_owned()
            }
            "research" => recipe.magic.as_mut().unwrap().research[0].completed = Some(true),
            "central" => recipe.magic.as_mut().unwrap().central = Some(9),
            "grid" => recipe.grid.as_mut().unwrap().cells[1] = Some(0),
            "candidate" => {
                recipe.inputs[0].choices[1].rule = recipe.inputs[0].choices[0].rule.clone()
            }
            "cost_view" => {
                let view = invalid
                    .views
                    .iter_mut()
                    .find(|view| Some(&view.id) == recipe.view.as_ref())
                    .unwrap();
                for element in &mut view.elements {
                    if let elysium_compiler_core::domain::Element::Cost { index, .. } = element {
                        *index = 99;
                    }
                }
                view.id = content_id("view", view).unwrap();
                recipe.view = Some(view.id.clone());
            }
            _ => unreachable!(),
        }
        recipe.id = recipe_id(recipe).unwrap();
        assert!(
            invalid.validate(&source).is_err(),
            "accepted invalid recipe {issue}"
        );
    }
    for issue in [
        "controller",
        "chunk",
        "rule",
        "bounds",
        "duplicate",
        "count",
        "anchor",
    ] {
        let mut invalid = domain.clone();
        let old = invalid.shapes[0].id.clone();
        match issue {
            "controller" => invalid.structures[0].controller = "item_missing".to_owned(),
            "chunk" => invalid.structures[0].pieces[0].chunks[0] = "shape_missing".to_owned(),
            "rule" => invalid.shapes[0].cells[0].index = 4096,
            "bounds" => invalid.structures[0].pieces[0].size = [1, 1, 1],
            "duplicate" => {
                let cell = invalid.shapes[0].cells[0].clone();
                invalid.shapes[0].cells.push(cell);
            }
            "count" => invalid.structures[0].pieces[0].cells += 1,
            "anchor" => invalid.structures[0].pieces[0].anchors[0] = invalid.shapes[0].cells[0].at,
            _ => unreachable!(),
        }
        invalid.shapes[0].id = content_id("shape", &invalid.shapes[0]).unwrap();
        for chunk in &mut invalid.structures[0].pieces[0].chunks {
            if *chunk == old {
                chunk.clone_from(&invalid.shapes[0].id);
            }
        }
        for build in &mut invalid.builds {
            for chunk in &mut build.chunks {
                if *chunk == old {
                    chunk.clone_from(&invalid.shapes[0].id);
                }
            }
            let previous = build.id.clone();
            build.id = content_id("build", build).unwrap();
            for variant in &mut invalid.structures[0].variants {
                if variant.build.as_ref() == Some(&previous) {
                    variant.build = Some(build.id.clone());
                }
            }
        }
        assert!(
            invalid.validate(&source).is_err(),
            "accepted invalid structure {issue}"
        );
    }
    for issue in [
        "owner",
        "frame",
        "overflow",
        "controller",
        "result",
        "palette",
        "count",
        "probe",
    ] {
        let mut invalid = domain.clone();
        let build = &mut invalid.builds[0];
        let previous = build.id.clone();
        match issue {
            "owner" => build.structure = "structure_missing".to_owned(),
            "frame" => build.origin[0] += 1,
            "overflow" => {
                build.origin[0] = i32::MIN;
                build.controller[0] = 1 << 31;
            }
            "controller" => {
                build.controller = [1, 1, 1];
                build.origin[2] = 1;
            }
            "result" => build.result = Some(1),
            "palette" => build.palette[0].block = "block_missing".to_owned(),
            "count" => build.cells += 1,
            "probe" => build.probe.count += 1,
            _ => unreachable!(),
        }
        build.id = content_id("build", build).unwrap();
        for variant in &mut invalid.structures[0].variants {
            if variant.build.as_ref() == Some(&previous) {
                variant.build = Some(build.id.clone());
            }
        }
        assert!(
            invalid.validate(&source).is_err(),
            "accepted invalid construction {issue}"
        );
    }
    for issue in [
        "missing",
        "duplicate",
        "order",
        "parameters",
        "definition",
        "no_outcome",
        "both_outcomes",
    ] {
        let mut invalid = domain.clone();
        let structure = &mut invalid.structures[0];
        match issue {
            "missing" => {
                structure.variants.pop();
            }
            "duplicate" => structure.variants[1] = structure.variants[0].clone(),
            "order" => structure.variants.swap(0, 1),
            "parameters" => {
                structure.variants[1]
                    .probe
                    .channels
                    .insert("coil".to_owned(), 3);
            }
            "definition" => structure.probe.count = 4,
            "no_outcome" => structure.variants[1].build = None,
            "both_outcomes" => {
                structure.variants[1].problem = Some(structure.description[0].clone())
            }
            _ => unreachable!(),
        }
        assert!(
            invalid.validate(&source).is_err(),
            "accepted invalid variant {issue}"
        );
    }
    let mut partial = domain.clone();
    for issue in ["empty", "coordinate", "finite", "uv", "texture", "hidden"] {
        let mut invalid = domain.clone();
        let model = invalid
            .models
            .iter_mut()
            .find(|model| !model.hidden)
            .unwrap();
        let previous = model.id.clone();
        match issue {
            "empty" => model.faces.clear(),
            "coordinate" => model.faces[0].vertices[0].at[0] = "257".to_owned(),
            "finite" => model.faces[0].vertices[0].at[0] = "NaN".to_owned(),
            "uv" => model.faces[0].vertices[0].uv[0] = "1.1".to_owned(),
            "texture" => model.faces[0].texture = "asset_missing".to_owned(),
            "hidden" => model.hidden = true,
            _ => unreachable!(),
        }
        model.id = content_id("model", model).unwrap();
        for build in &mut invalid.builds {
            for entry in &mut build.palette {
                if entry.model.as_ref() == Some(&previous) {
                    entry.model = Some(model.id.clone());
                }
            }
        }
        refresh_builds(&mut invalid);
        assert!(
            invalid.validate(&source).is_err(),
            "accepted invalid model {issue}"
        );
    }
    for issue in ["omitted", "reference", "outcome", "failure"] {
        let mut invalid = domain.clone();
        let build = &mut invalid.builds[0];
        match issue {
            "omitted" => build.rendered = false,
            "reference" => build.palette[0].model = Some("model_missing".to_owned()),
            "outcome" => build.palette[0].model = None,
            "failure" => build.palette[0].problem = Some(build.notes[0].clone()),
            _ => unreachable!(),
        }
        refresh_builds(&mut invalid);
        assert!(
            invalid.validate(&source).is_err(),
            "accepted incomplete appearance {issue}"
        );
    }
    let structure = &mut partial.structures[0];
    let failed = structure.variants[1].build.take().unwrap();
    structure.variants[1].problem = Some(structure.description[0].clone());
    partial.builds.retain(|build| build.id != failed);
    let used: std::collections::BTreeSet<_> = partial
        .structures
        .iter()
        .flat_map(|structure| &structure.pieces)
        .flat_map(|piece| &piece.chunks)
        .chain(partial.builds.iter().flat_map(|build| &build.chunks))
        .cloned()
        .collect();
    partial.shapes.retain(|shape| used.contains(&shape.id));
    let used_models: std::collections::BTreeSet<_> = partial
        .builds
        .iter()
        .flat_map(|build| &build.palette)
        .filter_map(|entry| entry.model.clone())
        .collect();
    partial
        .models
        .retain(|model| used_models.contains(&model.id));
    assert!(partial
        .validate(&source)
        .unwrap_err()
        .to_string()
        .contains("unavailable construction variant"));
    let mut selection = Source::open(&fixture()).unwrap();
    selection.manifest.scope.mode = "selection".to_owned();
    selection.manifest.id = selection.manifest.digest().unwrap();
    partial.validate(&selection).unwrap();
    let mut geometry = partial.clone();
    for build in &mut geometry.builds {
        build.rendered = false;
        for entry in &mut build.palette {
            entry.model = None;
            entry.problem = None;
        }
    }
    geometry.models.clear();
    refresh_builds(&mut geometry);
    geometry.validate(&selection).unwrap();
    let mut incomplete = domain.clone();
    incomplete.builds[0].palette[0].problem = Some(incomplete.builds[0].notes[0].clone());
    refresh_builds(&mut incomplete);
    incomplete.validate(&selection).unwrap();
    compile(&fixture(), output.path()).unwrap();
    let pointer = fs::read(output.path().join("current.json")).unwrap();
    let catalog = Catalog::current(output.path()).unwrap();
    let descriptor = &catalog.manifest.files[0];
    let mut bytes = catalog.read(descriptor).unwrap();
    bytes[0] ^= 1;
    fs::write(catalog.root.join(&descriptor.path), bytes).unwrap();
    assert!(catalog.verify().is_err());
    assert!(compile(&fixture(), output.path()).is_err());
    assert_eq!(
        fs::read(output.path().join("current.json")).unwrap(),
        pointer
    );
}

fn refresh_builds(domain: &mut Domain) {
    for build in &mut domain.builds {
        let previous = build.id.clone();
        build.id = content_id("build", build).unwrap();
        for structure in &mut domain.structures {
            for variant in &mut structure.variants {
                if variant.build.as_ref() == Some(&previous) {
                    variant.build = Some(build.id.clone());
                }
            }
        }
    }
}
