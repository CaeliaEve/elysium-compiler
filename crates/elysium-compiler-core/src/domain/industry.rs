use super::{check::origin, origin_id, Domain, Kind, Origin, Reference};
use crate::identity::integer;
use anyhow::{ensure, Result};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

/// An exact amount of base material, independent of item counts or fluid volume.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Ratio {
    pub numerator: String,
    pub denominator: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct MaterialPart {
    /// Registry form, e.g. plate, wireGt01, molten. It is not inferred from a name.
    pub key: String,
    pub target: Reference,
    /// Material units in one item; null when the source does not define a conversion.
    pub content: Option<Ratio>,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Component {
    pub material: String,
    /// Relative component count from the material definition, not a recipe quantity.
    pub amount: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Material {
    pub id: String,
    pub source: Origin,
    pub name: String,
    /// The source's formula verbatim; not parsed or guessed from an item tooltip.
    pub formula: String,
    /// ARGB color reported by the registry.
    pub color: u32,
    pub components: Vec<Component>,
    pub parts: Vec<MaterialPart>,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Tier {
    pub level: u32,
    pub name: String,
    /// Nominal voltage in EU per packet.
    pub voltage: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct CircuitStep {
    pub item: String,
    pub tier: Option<Tier>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum CircuitKind {
    Line,
    Parts,
}

/// A circuit family or component family defined by the game's diagram provider.
/// Adjacent steps describe progression, not a fabricated crafting recipe.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Circuit {
    pub id: String,
    pub source: Origin,
    pub name: String,
    pub kind: CircuitKind,
    pub boards: Vec<String>,
    pub steps: Vec<CircuitStep>,
    pub order: u32,
}

pub(super) fn validate(
    domain: &Domain,
    text: &impl Fn(&str) -> Result<()>,
    reference: &impl Fn(Kind, &str) -> Result<()>,
) -> Result<()> {
    let materials: BTreeMap<_, _> = domain
        .materials
        .iter()
        .map(|row| (row.id.as_str(), row))
        .collect();
    for material in &domain.materials {
        origin(&material.source)?;
        ensure!(
            material.id == origin_id("material", &material.source)?,
            "material identity mismatch: {}",
            material.id
        );
        text(&material.name)?;
        ensure!(
            material.formula.len() <= 65536,
            "material formula exceeds 64 KiB"
        );
        ensure!(
            material.parts.len() <= 4096 && material.components.len() <= 1024,
            "material exceeds its reference budget"
        );
        let mut keys = BTreeSet::new();
        for part in &material.parts {
            ensure!(
                !part.key.is_empty() && part.key.len() <= 128 && part.key.is_ascii(),
                "invalid material form"
            );
            ensure!(
                keys.insert((part.target.kind, &part.key)),
                "duplicate material form: {}",
                part.key
            );
            reference(part.target.kind, &part.target.id)?;
            if let Some(content) = &part.content {
                ensure!(
                    part.target.kind == Kind::Item,
                    "fluid volume has no implicit material conversion"
                );
                let numerator = integer(&content.numerator, 1, i64::MAX)?;
                let denominator = integer(&content.denominator, 1, i64::MAX)?;
                ensure!(
                    gcd(numerator, denominator) == 1,
                    "material ratio must be reduced"
                );
            }
        }
        let mut components = BTreeSet::new();
        for component in &material.components {
            ensure!(
                materials.contains_key(component.material.as_str()),
                "missing material component: {}",
                component.material
            );
            ensure!(
                components.insert(&component.material),
                "duplicate material component"
            );
            integer(&component.amount, 1, i64::MAX)?;
        }
    }
    let mut orders = BTreeSet::new();
    for circuit in &domain.circuits {
        origin(&circuit.source)?;
        ensure!(
            circuit.id == origin_id("circuit", &circuit.source)?,
            "circuit identity mismatch: {}",
            circuit.id
        );
        text(&circuit.name)?;
        ensure!(orders.insert(circuit.order), "duplicate circuit order");
        ensure!(
            !circuit.steps.is_empty() && circuit.steps.len() <= 256 && circuit.boards.len() <= 256,
            "invalid circuit family size"
        );
        ensure!(
            circuit.kind != CircuitKind::Parts || circuit.boards.is_empty(),
            "component families cannot declare circuit boards"
        );
        let mut boards = BTreeSet::new();
        for board in &circuit.boards {
            reference(Kind::Item, board)?;
            ensure!(boards.insert(board), "duplicate circuit board");
        }
        let mut previous = None;
        let mut steps = BTreeSet::new();
        for step in &circuit.steps {
            reference(Kind::Item, &step.item)?;
            ensure!(steps.insert(&step.item), "duplicate circuit step");
            ensure!(
                (circuit.kind == CircuitKind::Line) == step.tier.is_some(),
                "circuit tier does not match family kind"
            );
            if let Some(tier) = &step.tier {
                ensure!(
                    tier.level <= 255 && previous.is_none_or(|level| tier.level == level + 1),
                    "invalid circuit tier progression"
                );
                text(&tier.name)?;
                integer(&tier.voltage, 1, i64::MAX)?;
                previous = Some(tier.level);
            }
        }
    }
    Ok(())
}

fn gcd(mut left: i64, mut right: i64) -> i64 {
    while right != 0 {
        (left, right) = (right, left % right);
    }
    left
}
