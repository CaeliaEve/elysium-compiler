//! Shared game facts. Source and catalog use these meanings without compatibility aliases.
mod change;
mod check;
mod genetics;
mod industry;
mod magic;
mod model;
mod structure;
pub use change::{Change, Edit, Stack};
pub use genetics::*;
pub use industry::*;
pub use magic::*;
pub use model::*;
pub use structure::*;

use crate::identity::Nbt;
use crate::source::{Source, CORE_COLLECTIONS};
use anyhow::{ensure, Context, Result};
use schemars::JsonSchema;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Origin {
    pub owner: String,
    pub handler: String,
    pub key: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Item {
    pub id: String,
    pub registry: String,
    pub meta: i32,
    pub nbt: Option<Nbt>,
    /// Reference to a localized, Minecraft-formatted string.
    pub name: String,
    pub tooltip: Vec<String>,
    pub stack_limit: u32,
    pub durability: u32,
    pub tools: BTreeMap<String, i32>,
    /// Actual ItemArmor classification; durability alone does not identify armor or tools.
    pub armor: bool,
    /// Exact OreDictionary membership reported by the game.
    pub tags: Vec<String>,
    pub icon: Option<String>,
    /// NEI order; null denotes an item discovered only through a recipe.
    pub order: Option<u32>,
    /// Thaumcraft's returned object tags; null means it provided no value, not zero aspects.
    pub aspects: Option<Vec<AspectAmount>>,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Fluid {
    pub id: String,
    /// The actual global Forge fluid key, which need not contain a namespace.
    pub registry: String,
    pub nbt: Option<Nbt>,
    pub name: String,
    /// Kelvin.
    pub temperature: i32,
    pub density: i32,
    pub viscosity: i32,
    pub luminosity: u8,
    pub gaseous: bool,
    pub icon: Option<String>,
}

#[derive(
    Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "lowercase")]
pub enum Kind {
    Item,
    Fluid,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Reference {
    pub kind: Kind,
    pub id: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Choice {
    pub id: String,
    /// Positive signed-64-bit integer: item count or fluid millibuckets.
    pub amount: String,
    pub consume: Consumption,
    pub returns: Vec<Remainder>,
    pub rule: Match,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Remainder {
    pub kind: Kind,
    pub id: String,
    pub amount: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Match {
    Exact,
    Ore {
        name: String,
        exclusive: bool,
    },
    Wildcard {
        meta: bool,
        nbt: bool,
    },
    /// Compare only the named root tags with the choice; other tags are allowed.
    /// Presence checks ignore the tag's type and value, including false/zero.
    Tags {
        meta: bool,
        keys: Vec<String>,
        present: Vec<String>,
        absent: Vec<String>,
    },
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "lowercase", deny_unknown_fields)]
pub enum Consumption {
    Consume,
    Keep,
    Damage { points: u32 },
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Input {
    pub slot: u32,
    pub kind: Kind,
    pub choices: Vec<Choice>,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Chance {
    pub numerator: String,
    pub denominator: String,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum OutputRole {
    Result,
    Return,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Output {
    pub slot: u32,
    pub kind: Kind,
    pub id: String,
    pub amount: String,
    pub chance: Chance,
    pub role: OutputRole,
    /// With a change, id/amount are the first input choice's example, not a fixed result.
    pub change: Option<Change>,
}

impl Output {
    pub fn ids(&self) -> impl Iterator<Item = &str> {
        std::iter::once(self.id.as_str()).chain(
            self.change
                .iter()
                .flat_map(|change| change.samples.iter().map(|sample| sample.id.as_str())),
        )
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Unit {
    Count,
    Tick,
    Eu,
    EuPerTick,
    Mb,
    Kelvin,
    Pascal,
    Rpm,
    Mana,
    Lp,
    Vis,
    Percent,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "lowercase", deny_unknown_fields)]
pub enum PropertyValue {
    Integer {
        value: String,
    },
    Decimal {
        value: String,
    },
    Quantity {
        amount: String,
        unit: Unit,
    },
    Text {
        text: String,
    },
    Flag {
        value: bool,
    },
    Reference {
        target: Reference,
    },
    List {
        values: Vec<PropertyValue>,
    },
    Map {
        values: BTreeMap<String, PropertyValue>,
    },
    Symbol {
        namespace: String,
        value: String,
    },
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Property {
    pub name: String,
    pub value: PropertyValue,
}

/// Crafting arrangement is recipe data and remains available when no view is captured.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Grid {
    pub width: u32,
    pub height: u32,
    /// Row-major item input slot references, with explicit empty cells.
    pub cells: Vec<Option<u32>>,
    pub mirror: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Recipe {
    pub id: String,
    pub source: Origin,
    pub category: String,
    pub inputs: Vec<Input>,
    pub outputs: Vec<Output>,
    /// Ticks, when the source defines a duration.
    pub duration: Option<String>,
    /// Signed EU/t; negative values denote generation.
    pub energy: Option<String>,
    /// Namespaced properties with an explicit value type and unit.
    pub properties: BTreeMap<String, Property>,
    pub grid: Option<Grid>,
    pub magic: Option<MagicRecipe>,
    pub view: Option<String>,
    pub order: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Category {
    pub id: String,
    pub name: String,
    pub source: Origin,
    pub icon: Option<Reference>,
    pub machines: Vec<Reference>,
    pub view: Option<String>,
    pub order: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Group {
    pub id: String,
    pub source: Origin,
    pub name: Option<String>,
    pub members: Vec<String>,
    pub representative: String,
    pub collapsed: bool,
    pub order: u32,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum Direction {
    Input,
    Output,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum Align {
    Left,
    Center,
    Right,
}

/// A discrete native UI state. Areas reveal matching fractions of the texture and destination.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Motion {
    pub ticks: u32,
    /// Non-overlapping [left, top, right, bottom] rectangles in [0,1], as finite float32 decimal strings.
    /// An empty list hides the element for this state.
    pub areas: Vec<[String; 4]>,
}

/// Shared clipping timeline, independent of the texture or recipe using it.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Track {
    pub id: String,
    /// One repeating native UI cycle, compressed by consecutive equal states; never recipe duration.
    pub frames: Vec<Motion>,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "lowercase", deny_unknown_fields)]
pub enum Element {
    Clip {
        asset: String,
        x: i32,
        y: i32,
        width: u32,
        height: u32,
        z: i32,
        track: String,
    },
    Sprite {
        asset: String,
        x: i32,
        y: i32,
        width: u32,
        height: u32,
        z: i32,
    },
    Slot {
        direction: Direction,
        substance: Kind,
        slot: u32,
        x: i32,
        y: i32,
        width: u32,
        height: u32,
        z: i32,
    },
    Cost {
        /// Index into the owning recipe's magic aspect costs, never an inventory item.
        index: u32,
        x: i32,
        y: i32,
        width: u32,
        height: u32,
        z: i32,
    },
    Text {
        text: String,
        x: i32,
        y: i32,
        color: u32,
        align: Align,
        z: i32,
    },
    Rectangle {
        x: i32,
        y: i32,
        width: u32,
        height: u32,
        color: u32,
        z: i32,
    },
    Tooltip {
        x: i32,
        y: i32,
        width: u32,
        height: u32,
        lines: Vec<String>,
        z: i32,
    },
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct View {
    pub id: String,
    pub width: u32,
    pub height: u32,
    pub elements: Vec<Element>,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Frame {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
    pub ticks: u32,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum AssetKind {
    Resource,
    Capture,
    Block,
    Fluid,
    Entity,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AssetOrigin {
    pub kind: AssetKind,
    pub location: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Asset {
    pub id: String,
    pub path: String,
    pub width: u32,
    pub height: u32,
    /// Empty means a still image; otherwise every frame specifies its crop and tick duration.
    pub frames: Vec<Frame>,
    pub interpolate: bool,
    pub source: AssetOrigin,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Text {
    pub id: String,
    pub locale: String,
    pub text: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Domain {
    pub items: Vec<Item>,
    pub fluids: Vec<Fluid>,
    pub recipes: Vec<Recipe>,
    pub categories: Vec<Category>,
    pub groups: Vec<Group>,
    pub views: Vec<View>,
    pub tracks: Vec<Track>,
    pub assets: Vec<Asset>,
    pub strings: Vec<Text>,
    pub materials: Vec<Material>,
    pub circuits: Vec<Circuit>,
    pub species: Vec<Species>,
    pub mutations: Vec<Mutation>,
    pub structures: Vec<Structure>,
    pub blocks: Vec<Block>,
    pub models: Vec<Model>,
    pub builds: Vec<Build>,
    pub shapes: Vec<Shape>,
    pub aspects: Vec<Aspect>,
    pub research: Vec<Research>,
}

impl Domain {
    pub fn load(source: &Source) -> Result<Self> {
        source.environment()?;
        ensure!(source.manifest.scope.collections.iter().map(String::as_str).eq(CORE_COLLECTIONS.iter().copied()),
            "a catalog requires all domain collections; unsupported extensions or partial collections cannot be published");
        let result = Self {
            items: read(source, "items")?,
            fluids: read(source, "fluids")?,
            recipes: read(source, "recipes")?,
            categories: read(source, "categories")?,
            groups: read(source, "groups")?,
            views: read(source, "views")?,
            tracks: read(source, "tracks")?,
            assets: read(source, "assets")?,
            strings: read(source, "strings")?,
            materials: read(source, "materials")?,
            circuits: read(source, "circuits")?,
            species: read(source, "species")?,
            mutations: read(source, "mutations")?,
            structures: read(source, "structures")?,
            blocks: read(source, "blocks")?,
            models: read(source, "models")?,
            builds: read(source, "builds")?,
            shapes: read(source, "shapes")?,
            aspects: read(source, "aspects")?,
            research: read(source, "research")?,
        };
        result.validate(source)?;
        Ok(result)
    }
}

fn read<T: DeserializeOwned>(source: &Source, kind: &str) -> Result<Vec<T>> {
    let mut rows = Vec::new();
    source.visit(kind, |value| {
        let id = value["id"].as_str().unwrap_or_default().to_owned();
        rows.push(serde_json::from_value(value).with_context(|| format!("decode {kind}/{id}"))?);
        Ok(())
    })?;
    Ok(rows)
}

pub fn content_id(prefix: &str, record: &impl Serialize) -> Result<String> {
    let mut value = serde_json::to_value(record)?;
    value
        .as_object_mut()
        .context("record must be an object")?
        .remove("id");
    value_id(prefix, &value)
}

pub fn origin_id(prefix: &str, source: &Origin) -> Result<String> {
    value_id(prefix, &serde_json::to_value(source)?)
}

pub fn recipe_id(recipe: &Recipe) -> Result<String> {
    let values: BTreeMap<_, _> = recipe
        .properties
        .iter()
        .map(|(key, property)| (key, &property.value))
        .collect();
    let mut magic = serde_json::to_value(&recipe.magic)?;
    if let Some(studies) = magic.get_mut("research").and_then(Value::as_array_mut) {
        for study in studies {
            study
                .as_object_mut()
                .expect("research link object")
                .remove("completed");
        }
    }
    let mut outputs = serde_json::to_value(&recipe.outputs)?;
    for output in outputs.as_array_mut().expect("output array") {
        if output.get("change").is_some_and(Value::is_object) {
            output.as_object_mut().expect("output object").remove("id");
            output
                .as_object_mut()
                .expect("output object")
                .remove("amount");
            output["change"]
                .as_object_mut()
                .expect("change object")
                .remove("samples");
        }
    }
    value_id(
        "recipe",
        &json!({
            "source": recipe.source, "category": recipe.category,
            "inputs": recipe.inputs, "outputs": outputs,
            "duration": recipe.duration, "energy": recipe.energy, "properties": values,
            "grid": recipe.grid, "magic": magic,
        }),
    )
}

fn value_id(prefix: &str, value: &Value) -> Result<String> {
    Ok(format!(
        "{prefix}_{:x}",
        Sha256::digest(serde_json::to_vec(value)?)
    ))
}
