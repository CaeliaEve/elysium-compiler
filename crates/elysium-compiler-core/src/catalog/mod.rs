//! Deterministic, validated projections encoded with standard MessagePack maps.
mod atlas;
pub mod cli;
mod store;

use crate::domain::*;
use crate::source::{Source, SOURCE_REVISION};
use crate::text::{build_pinyin_fields, normalize_text};
use anyhow::{ensure, Result};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::Path;

pub use store::{Catalog, File, Manifest, Pointer, Publication};

pub const FORMAT: &str = "elysium.catalog";
pub const REVISION: u32 = 10;
pub const FILE_LIMIT: usize = 16 * 1024 * 1024;
pub const IMAGE_LIMIT: usize = 80 * 1024 * 1024;
pub const TABLE_ROWS: usize = 4096;

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Entry {
    pub id: String,
    pub kind: Kind,
    pub registry: String,
    pub meta: Option<i32>,
    pub name: String,
    pub tooltip: Vec<String>,
    pub icon: Option<String>,
    pub order: Option<u32>,
    pub group: Option<String>,
    /// Precomputed search text includes registry, display name, tooltip, ore tags and pinyin.
    pub terms: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Links {
    pub id: String,
    pub recipes: Vec<String>,
    pub uses: Vec<String>,
    pub topics: Vec<String>,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum TopicKind {
    Material,
    Circuit,
    Bee,
    Tree,
    Structure,
    Aspect,
    Research,
}

/// Mutation adjacency, so one species page never decodes the entire breeding graph.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Lineage {
    pub id: String,
    pub origins: Vec<String>,
    pub crosses: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Topic {
    pub id: String,
    pub kind: TopicKind,
    pub name: String,
    pub terms: String,
    pub icon: Option<Reference>,
    /// A standalone atlas image when this topic has no item/fluid icon.
    pub image: Option<String>,
    pub order: u32,
}

/// Compact recipe metadata for filtering and counts without decoding full ingredient payloads.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RecipeIndex {
    pub id: String,
    pub category: String,
    pub owner: String,
    pub handler: String,
    pub targets: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Sprite {
    pub path: String,
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
    pub ticks: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Texture {
    pub id: String,
    pub width: u32,
    pub height: u32,
    pub frames: Vec<Sprite>,
    pub interpolate: bool,
}

// One declaration owns schema variants, the required table set and record inspection.
macro_rules! tables {
    ($($variant:ident($record:ty) => $name:literal),+ $(,)?) => {
        #[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
        #[serde(tag = "kind", content = "records", deny_unknown_fields)]
        pub enum Table {
            $(#[serde(rename = $name)] $variant(Vec<$record>)),+
        }

        impl Table {
            const KINDS: &'static [&'static str] = &[$($name),+];
            fn keys(&self) -> (&'static str, Vec<&str>) {
                match self { $(Self::$variant(rows) => ($name, rows.iter().map(|row| row.id.as_str()).collect())),+ }
            }
        }
    };
}

tables! {
    Aspects(Aspect) => "aspects",
    Blocks(Block) => "blocks",
    Browse(Entry) => "browse",
    Builds(Build) => "builds",
    Categories(Category) => "categories",
    Circuits(Circuit) => "circuits",
    Fluids(Fluid) => "fluids",
    Groups(Group) => "groups",
    Index(RecipeIndex) => "index",
    Items(Item) => "items",
    Lineage(Lineage) => "lineage",
    Links(Links) => "links",
    Materials(Material) => "materials",
    Models(Model) => "models",
    Mutations(Mutation) => "mutations",
    Recipes(Recipe) => "recipes",
    Research(Research) => "research",
    Shapes(Shape) => "shapes",
    Species(Species) => "species",
    Strings(Text) => "strings",
    Structures(Structure) => "structures",
    Textures(Texture) => "textures",
    Topics(Topic) => "topics",
    Tracks(Track) => "tracks",
    Views(View) => "views",
}

/// Owns the schema exported to consumers; it is not a runtime payload itself.
#[derive(JsonSchema)]
pub struct Contract {
    pub environment: crate::source::Environment,
    pub source: crate::source::SourceManifest,
    pub domain: Domain,
    pub catalog: Manifest,
    pub pointer: Pointer,
    pub table: Table,
}

pub fn compile(input: &Path, output: &Path) -> Result<Publication> {
    let source = Source::open(input)?;
    ensure!(
        source.manifest.revision == SOURCE_REVISION,
        "unsupported source revision"
    );
    let domain = Domain::load(&source)?;
    let mut writer = store::Writer::new(output, &source)?;
    let textures = atlas::pack(&source, &domain.assets, &mut writer)?;
    macro_rules! write {
        ($kind:literal, $records:expr, $variant:ident) => {
            writer.table(
                $kind,
                $records,
                |rows| Table::$variant(rows.to_vec()),
                |row| row.id.as_str(),
            )?;
        };
    }
    write!("items", &domain.items, Items);
    write!("fluids", &domain.fluids, Fluids);
    write!("recipes", &domain.recipes, Recipes);
    write!("categories", &domain.categories, Categories);
    write!("groups", &domain.groups, Groups);
    write!("views", &domain.views, Views);
    write!("tracks", &domain.tracks, Tracks);
    write!("strings", &domain.strings, Strings);
    write!("textures", &textures, Textures);
    let entries = entries(&domain);
    write!("browse", &entries, Browse);
    let links = links(&domain);
    write!("links", &links, Links);
    let index: Vec<_> = domain
        .recipes
        .iter()
        .map(|recipe| RecipeIndex {
            id: recipe.id.clone(),
            category: recipe.category.clone(),
            owner: recipe.source.owner.clone(),
            handler: recipe.source.handler.clone(),
            targets: recipe
                .inputs
                .iter()
                .flat_map(|input| input.choices.iter().map(|choice| choice.id.clone()))
                .chain(
                    recipe
                        .outputs
                        .iter()
                        .flat_map(Output::ids)
                        .map(str::to_owned),
                )
                .collect::<std::collections::BTreeSet<_>>()
                .into_iter()
                .collect(),
        })
        .collect();
    write!("index", &index, Index);
    write!("materials", &domain.materials, Materials);
    write!("circuits", &domain.circuits, Circuits);
    write!("species", &domain.species, Species);
    write!("mutations", &domain.mutations, Mutations);
    write!("structures", &domain.structures, Structures);
    write!("blocks", &domain.blocks, Blocks);
    write!("models", &domain.models, Models);
    write!("builds", &domain.builds, Builds);
    write!("shapes", &domain.shapes, Shapes);
    write!("aspects", &domain.aspects, Aspects);
    write!("research", &domain.research, Research);
    write!("lineage", &lineage(&domain), Lineage);
    write!("topics", &topics(&domain), Topics);
    writer.publish()
}

fn topics(domain: &Domain) -> Vec<Topic> {
    let strings: BTreeMap<_, _> = domain
        .strings
        .iter()
        .map(|row| (&row.id, &row.text))
        .collect();
    let mut rows = Vec::new();
    for material in &domain.materials {
        let name = strings[&material.name].clone();
        rows.push(Topic {
            id: material.id.clone(),
            kind: TopicKind::Material,
            terms: terms(
                &name,
                &material.source.key,
                std::slice::from_ref(&material.formula),
                &[],
            ),
            name,
            icon: material.parts.first().map(|part| part.target.clone()),
            image: None,
            order: 0,
        });
    }
    for circuit in &domain.circuits {
        let name = strings[&circuit.name].clone();
        rows.push(Topic {
            id: circuit.id.clone(),
            kind: TopicKind::Circuit,
            terms: terms(&name, &circuit.source.key, &[], &[]),
            name,
            icon: circuit.steps.first().map(|step| Reference {
                kind: Kind::Item,
                id: step.item.clone(),
            }),
            image: None,
            order: circuit.order,
        });
    }
    for species in &domain.species {
        let name = strings[&species.name].clone();
        rows.push(Topic {
            id: species.id.clone(),
            kind: match species.kind {
                SpeciesKind::Bee => TopicKind::Bee,
                SpeciesKind::Tree => TopicKind::Tree,
            },
            terms: terms(
                &name,
                &species.source.key,
                std::slice::from_ref(&species.binomial),
                &[],
            ),
            name,
            icon: species.members.first().map(|member| Reference {
                kind: Kind::Item,
                id: member.item.clone(),
            }),
            image: None,
            order: 0,
        });
    }
    for structure in &domain.structures {
        let name = strings[&structure.name].clone();
        rows.push(Topic {
            id: structure.id.clone(),
            kind: TopicKind::Structure,
            terms: terms(&name, &structure.source.key, &[], &[]),
            name,
            icon: Some(Reference {
                kind: Kind::Item,
                id: structure.controller.clone(),
            }),
            image: None,
            order: 0,
        });
    }
    for aspect in &domain.aspects {
        let name = strings[&aspect.name].clone();
        rows.push(Topic {
            id: aspect.id.clone(),
            kind: TopicKind::Aspect,
            terms: terms(
                &name,
                &aspect.source.key,
                &[strings[&aspect.description].clone()],
                &[],
            ),
            name,
            icon: None,
            image: aspect.icon.clone(),
            order: 0,
        });
    }
    for study in &domain.research {
        let name = strings[&study.name].clone();
        rows.push(Topic {
            id: study.id.clone(),
            kind: TopicKind::Research,
            terms: terms(
                &name,
                &study.source.key,
                &[
                    strings[&study.category_name].clone(),
                    study.category.clone(),
                ],
                &[],
            ),
            name,
            icon: study.icon.as_ref().map(|id| Reference {
                kind: Kind::Item,
                id: id.clone(),
            }),
            image: study.texture.clone(),
            order: 0,
        });
    }
    rows.sort_by(|left, right| left.id.cmp(&right.id));
    rows
}

fn lineage(domain: &Domain) -> Vec<Lineage> {
    let mut rows: BTreeMap<_, _> = domain
        .species
        .iter()
        .map(|species| {
            (
                species.id.as_str(),
                Lineage {
                    id: species.id.clone(),
                    origins: Vec::new(),
                    crosses: Vec::new(),
                },
            )
        })
        .collect();
    for mutation in &domain.mutations {
        rows.get_mut(mutation.result.as_str())
            .expect("validated mutation result")
            .origins
            .push(mutation.id.clone());
        for parent in mutation
            .parents
            .iter()
            .collect::<std::collections::BTreeSet<_>>()
        {
            rows.get_mut(parent.as_str())
                .expect("validated mutation parent")
                .crosses
                .push(mutation.id.clone());
        }
    }
    rows.into_values().collect()
}

fn entries(domain: &Domain) -> Vec<Entry> {
    let strings: BTreeMap<_, _> = domain
        .strings
        .iter()
        .map(|text| (text.id.as_str(), text.text.as_str()))
        .collect();
    let groups: BTreeMap<_, _> = domain
        .groups
        .iter()
        .flat_map(|group| group.members.iter().map(move |id| (id.as_str(), &group.id)))
        .collect();
    let mut entries = Vec::with_capacity(domain.items.len() + domain.fluids.len());
    for item in &domain.items {
        let name = strings[item.name.as_str()].to_owned();
        let tooltip: Vec<_> = item
            .tooltip
            .iter()
            .map(|line| strings[line.as_str()].to_owned())
            .collect();
        entries.push(Entry {
            terms: terms(&name, &item.registry, &tooltip, &item.tags),
            id: item.id.clone(),
            kind: Kind::Item,
            registry: item.registry.clone(),
            meta: Some(item.meta),
            name,
            tooltip,
            icon: item.icon.clone(),
            order: item.order,
            group: groups.get(item.id.as_str()).map(|id| (*id).clone()),
        });
    }
    for fluid in &domain.fluids {
        let name = strings[fluid.name.as_str()].to_owned();
        entries.push(Entry {
            terms: terms(&name, &fluid.registry, &[], &[]),
            id: fluid.id.clone(),
            kind: Kind::Fluid,
            registry: fluid.registry.clone(),
            meta: None,
            name,
            tooltip: Vec::new(),
            icon: fluid.icon.clone(),
            order: None,
            group: None,
        });
    }
    entries.sort_by(|left, right| left.id.cmp(&right.id));
    entries
}

fn terms(name: &str, registry: &str, tooltip: &[String], tags: &[String]) -> String {
    let (pinyin, initials) = build_pinyin_fields(name);
    let mut parts = vec![name.to_owned(), registry.to_owned(), pinyin, initials];
    parts.extend_from_slice(tooltip);
    parts.extend_from_slice(tags);
    let mut plain = String::new();
    let mut chars = parts.join(" ").chars().collect::<Vec<_>>().into_iter();
    while let Some(character) = chars.next() {
        if character == '§' {
            chars.next();
        } else {
            plain.push(character);
        }
    }
    normalize_text(&plain)
}

fn links(domain: &Domain) -> Vec<Links> {
    let mut links: BTreeMap<&str, Links> = domain
        .items
        .iter()
        .map(|item| item.id.as_str())
        .chain(domain.fluids.iter().map(|fluid| fluid.id.as_str()))
        .map(|id| {
            (
                id,
                Links {
                    id: id.to_owned(),
                    recipes: Vec::new(),
                    uses: Vec::new(),
                    topics: Vec::new(),
                },
            )
        })
        .collect();
    for structure in &domain.structures {
        let ids = std::iter::once(&structure.controller).chain(
            structure
                .pieces
                .iter()
                .flat_map(|piece| &piece.rules)
                .flat_map(|rule| rule.placements.iter().flatten()),
        );
        for id in ids.collect::<std::collections::BTreeSet<_>>() {
            links
                .get_mut(id.as_str())
                .expect("validated structure item")
                .topics
                .push(structure.id.clone());
        }
    }
    for item in &domain.items {
        for amount in item.aspects.iter().flatten() {
            links
                .get_mut(item.id.as_str())
                .expect("validated item")
                .topics
                .push(amount.aspect.clone());
        }
    }
    for study in &domain.research {
        for id in study
            .item_triggers
            .iter()
            .chain(study.icon.iter())
            .collect::<std::collections::BTreeSet<_>>()
        {
            links
                .get_mut(id.as_str())
                .expect("validated research item")
                .topics
                .push(study.id.clone());
        }
    }
    for material in &domain.materials {
        for id in material
            .parts
            .iter()
            .map(|part| &part.target.id)
            .collect::<std::collections::BTreeSet<_>>()
        {
            links
                .get_mut(id.as_str())
                .expect("validated material part")
                .topics
                .push(material.id.clone());
        }
    }
    for circuit in &domain.circuits {
        for id in circuit
            .boards
            .iter()
            .chain(circuit.steps.iter().map(|step| &step.item))
            .collect::<std::collections::BTreeSet<_>>()
        {
            links
                .get_mut(id.as_str())
                .expect("validated circuit item")
                .topics
                .push(circuit.id.clone());
        }
    }
    for species in &domain.species {
        for id in species
            .members
            .iter()
            .map(|member| &member.item)
            .chain(
                species
                    .products
                    .iter()
                    .chain(&species.specialties)
                    .map(|product| &product.item),
            )
            .collect::<std::collections::BTreeSet<_>>()
        {
            links
                .get_mut(id.as_str())
                .expect("validated species item")
                .topics
                .push(species.id.clone());
        }
    }
    for entry in links.values_mut() {
        entry.topics.sort();
    }
    // Category order follows NEI; recipe order is local to its category, with ids breaking ties.
    let mut recipes: Vec<_> = domain.recipes.iter().collect();
    let machines: BTreeMap<_, _> = domain
        .categories
        .iter()
        .map(|category| (&category.id, category))
        .collect();
    recipes.sort_by_key(|recipe| {
        (
            machines[&recipe.category].order,
            &recipe.category,
            recipe.order,
            &recipe.id,
        )
    });
    for recipe in recipes {
        let mut used = std::collections::BTreeSet::new();
        for machine in &machines[&recipe.category].machines {
            if used.insert(&machine.id) {
                links
                    .get_mut(machine.id.as_str())
                    .expect("validated machine")
                    .uses
                    .push(recipe.id.clone());
            }
        }
        for input in &recipe.inputs {
            for choice in &input.choices {
                if used.insert(&choice.id) {
                    links
                        .get_mut(choice.id.as_str())
                        .expect("validated reference")
                        .uses
                        .push(recipe.id.clone());
                }
            }
        }
        let mut produced = std::collections::BTreeSet::new();
        for output in &recipe.outputs {
            if matches!(output.role, OutputRole::Result) && output.chance.numerator != "0" {
                for id in output.ids() {
                    if produced.insert(id) {
                        links
                            .get_mut(id)
                            .expect("validated reference")
                            .recipes
                            .push(recipe.id.clone());
                    }
                }
            }
        }
    }
    links.into_values().collect()
}
