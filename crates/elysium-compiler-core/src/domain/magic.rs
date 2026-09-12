use super::{check::origin, origin_id, Domain, Kind, Origin};
use crate::identity::integer;
use anyhow::{ensure, Result};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet, VecDeque};

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AspectAmount {
    pub aspect: String,
    pub amount: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Aspect {
    pub id: String,
    pub source: Origin,
    pub name: String,
    pub description: String,
    pub color: u32,
    /// Empty for a primal aspect; otherwise the two components used by the game, including repeats.
    pub components: Vec<String>,
    pub icon: Option<String>,
    /// Observed player knowledge; null when its client cache has not arrived.
    pub discovered: Option<bool>,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ResearchLink {
    pub key: String,
    /// Null is permitted only for an unregistered @ knowledge flag, as defined by Thaumcraft.
    pub id: Option<String>,
    pub completed: Option<bool>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum MagicKind {
    Arcane,
    Crucible,
    Infusion,
}

/// Registered recipe costs before equipment discounts and observed research prerequisites.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct MagicRecipe {
    pub kind: MagicKind,
    /// Vis for arcane crafting; essentia for crucible and infusion recipes.
    pub aspects: Vec<AspectAmount>,
    pub research: Vec<ResearchLink>,
    /// Base infusion instability, independent of the player's altar and stabilizers.
    pub instability: Option<i32>,
    /// Item input slot containing the central infusion ingredient.
    pub central: Option<u32>,
    /// Optional Salis self-payment. Ordinary arcane output samples assume a
    /// separate wand in the workbench's supply slot; this mode requires it empty.
    pub payment: Option<Payment>,
    /// The configured creative-mode waiver bypasses Vis checks/consumption.
    /// An empty arcane cost list is usable only with this waiver, not free in survival.
    pub creative: bool,
}

/// The consumed input pays the arcane cost using its OLD components and the
/// player's equipment discount. After payment, the new wand retains the remaining
/// charge up to capacity, or zeros these tags. No fixed paid-output sample is implied.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Payment {
    pub input: u32,
    /// Primal aspect id -> root NBT charge key. Charge/capacity are hundredths of Vis.
    pub charges: BTreeMap<String, String>,
    pub capacity: i32,
    pub preserve: bool,
}

#[derive(
    Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "lowercase")]
pub enum ResearchFlag {
    Auto,
    Concealed,
    Hidden,
    Lost,
    Round,
    Secondary,
    Special,
    Stub,
    Virtual,
}

/// Research prerequisites and observed knowledge, not a claim that the player can currently unlock it.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Research {
    pub id: String,
    pub source: Origin,
    pub name: String,
    pub text: String,
    pub category: String,
    pub category_name: String,
    pub position: [i32; 2],
    pub complexity: u32,
    pub warp: i32,
    pub flags: Vec<ResearchFlag>,
    pub completed: Option<bool>,
    pub parents: Vec<ResearchLink>,
    pub hidden_parents: Vec<ResearchLink>,
    pub siblings: Vec<ResearchLink>,
    pub aspects: Vec<AspectAmount>,
    pub item_triggers: Vec<String>,
    pub entity_triggers: Vec<String>,
    pub aspect_triggers: Vec<String>,
    pub icon: Option<String>,
    pub texture: Option<String>,
}

pub(super) fn validate(
    domain: &Domain,
    text: &impl Fn(&str) -> Result<()>,
    reference: &impl Fn(Kind, &str) -> Result<()>,
    asset: &impl Fn(&str) -> Result<()>,
) -> Result<()> {
    ensure!(
        domain.aspects.len() <= 4096,
        "aspect registry exceeds its budget"
    );
    let aspects: BTreeMap<_, _> = domain
        .aspects
        .iter()
        .map(|row| (row.id.as_str(), row))
        .collect();
    let studies: BTreeMap<_, _> = domain
        .research
        .iter()
        .map(|row| (row.source.key.as_str(), row))
        .collect();
    ensure!(
        studies.len() == domain.research.len(),
        "duplicate research registry key"
    );
    let aspect = |id: &str| -> Result<()> {
        ensure!(aspects.contains_key(id), "missing aspect: {id}");
        Ok(())
    };
    let amounts = |rows: &[AspectAmount]| -> Result<()> {
        ensure!(rows.len() <= 4096, "aspect list exceeds its budget");
        let mut used = BTreeSet::new();
        for row in rows {
            aspect(&row.aspect)?;
            ensure!(used.insert(&row.aspect), "duplicate aspect amount");
            integer(&row.amount, 0, i32::MAX as i64)?;
        }
        Ok(())
    };
    let mut dependencies = BTreeMap::new();
    let mut parents: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
    let mut ready = VecDeque::new();
    for row in &domain.aspects {
        origin(&row.source)?;
        ensure!(
            row.id == origin_id("aspect", &row.source)?,
            "aspect identity mismatch"
        );
        text(&row.name)?;
        text(&row.description)?;
        if let Some(icon) = &row.icon {
            asset(icon)?;
        }
        ensure!(
            row.components.is_empty() || row.components.len() == 2,
            "invalid aspect composition"
        );
        dependencies.insert(row.id.as_str(), row.components.len());
        if row.components.is_empty() {
            ready.push_back(row.id.as_str());
        }
        for component in &row.components {
            aspect(component)?;
            parents.entry(component).or_default().push(&row.id);
        }
    }
    let mut visited = 0;
    while let Some(id) = ready.pop_front() {
        visited += 1;
        for parent in parents.get(id).into_iter().flatten() {
            let remaining = dependencies.get_mut(parent).expect("validated aspect");
            *remaining -= 1;
            if *remaining == 0 {
                ready.push_back(parent);
            }
        }
    }
    ensure!(visited == domain.aspects.len(), "cyclic aspect composition");
    for item in &domain.items {
        if let Some(rows) = &item.aspects {
            amounts(rows)?;
        }
    }
    let links = |rows: &[ResearchLink]| -> Result<()> {
        ensure!(rows.len() <= 512, "research links exceed their budget");
        for row in rows {
            ensure!(
                !row.key.is_empty() && row.key.len() <= 256,
                "invalid research key"
            );
            match studies.get(row.key.as_str()) {
                Some(study) => {
                    ensure!(
                        row.id.as_deref() == Some(study.id.as_str()),
                        "research reference does not match its key"
                    );
                    ensure!(
                        row.completed == study.completed,
                        "contradictory research knowledge within one snapshot"
                    );
                }
                None => ensure!(
                    row.id.is_none() && row.key.starts_with('@'),
                    "missing research definition: {}",
                    row.key
                ),
            }
        }
        Ok(())
    };
    for recipe in &domain.recipes {
        if let Some(magic) = &recipe.magic {
            amounts(&magic.aspects)?;
            links(&magic.research)?;
            if let Some(payment) = &magic.payment {
                ensure!(
                    magic.kind == MagicKind::Arcane
                        && (!magic.aspects.is_empty() || magic.creative),
                    "self-payment requires payable arcane costs"
                );
                ensure!(
                    payment.capacity >= 0
                        && !payment.charges.is_empty()
                        && payment.charges.len() <= 4096,
                    "invalid self-payment capacity or charge map"
                );
                let input = recipe
                    .inputs
                    .iter()
                    .find(|input| input.kind == Kind::Item && input.slot == payment.input)
                    .ok_or_else(|| anyhow::anyhow!("self-payment input is missing"))?;
                ensure!(
                    input.choices.iter().all(|choice| choice.amount == "1"
                        && matches!(choice.consume, super::Consumption::Consume)),
                    "self-payment must consume one wand"
                );
                let changes: Vec<_> = recipe
                    .outputs
                    .iter()
                    .filter_map(|output| output.change.as_ref())
                    .filter(|change| change.input == payment.input)
                    .collect();
                ensure!(
                    changes.len() == 1,
                    "self-payment must bind one output patch"
                );
                let super::Edit::Patch { set, limits } = &changes[0].action else {
                    anyhow::bail!("self-payment requires an output patch");
                };
                let mut keys = BTreeSet::new();
                for (id, key) in &payment.charges {
                    let row = aspects
                        .get(id.as_str())
                        .ok_or_else(|| anyhow::anyhow!("missing payment aspect"))?;
                    ensure!(
                        row.components.is_empty() && key == &row.source.key && keys.insert(key),
                        "invalid primal charge key"
                    );
                    if payment.preserve {
                        ensure!(
                            !set.contains_key(key)
                                && (limits.is_empty()
                                    || limits.get(key) == Some(&payment.capacity)),
                            "preserved charge contradicts the output patch"
                        );
                    } else {
                        ensure!(
                            matches!(set.get(key), Some(crate::identity::Nbt::Int { value }) if value == "0")
                                && limits.is_empty(),
                            "cleared charge contradicts the output patch"
                        );
                    }
                }
                ensure!(
                    magic
                        .aspects
                        .iter()
                        .all(|amount| payment.charges.contains_key(&amount.aspect)),
                    "payment omits a charged aspect"
                );
            }
            ensure!(
                !magic.creative || magic.kind == MagicKind::Arcane,
                "creative Vis waiver requires arcane crafting"
            );
            ensure!(
                magic.kind != MagicKind::Arcane || !magic.aspects.is_empty() || magic.creative,
                "empty arcane cost list cannot be paid in survival"
            );
            ensure!(
                recipe.inputs.iter().all(|input| input.kind == Kind::Item)
                    && recipe
                        .outputs
                        .iter()
                        .all(|output| output.kind == Kind::Item),
                "Thaumcraft recipes require item slots"
            );
            let mut keys = BTreeSet::new();
            ensure!(
                magic.research.iter().all(|link| keys.insert(&link.key)),
                "duplicate recipe research prerequisite"
            );
            if magic.kind == MagicKind::Infusion {
                let central = magic
                    .central
                    .ok_or_else(|| anyhow::anyhow!("infusion has no central input"))?;
                ensure!(
                    recipe.inputs.iter().any(|input| input.slot == central),
                    "infusion central input is missing"
                );
                ensure!(
                    magic.instability.is_some_and(|value| value >= 0),
                    "invalid infusion instability"
                );
            } else {
                ensure!(
                    magic.central.is_none() && magic.instability.is_none(),
                    "non-infusion recipe has altar fields"
                );
            }
            ensure!(
                magic.kind == MagicKind::Arcane || recipe.grid.is_none(),
                "non-arcane magic recipe has a crafting grid"
            );
        }
    }
    for row in &domain.research {
        origin(&row.source)?;
        ensure!(
            row.id == origin_id("research", &row.source)?,
            "research identity mismatch"
        );
        text(&row.name)?;
        text(&row.text)?;
        text(&row.category_name)?;
        ensure!(
            !row.category.is_empty() && row.category.len() <= 256,
            "invalid research category"
        );
        ensure!(row.complexity <= 3, "invalid research complexity");
        ensure!(
            row.flags.windows(2).all(|pair| pair[0] < pair[1]),
            "research flags must be sorted and unique"
        );
        links(&row.parents)?;
        links(&row.hidden_parents)?;
        links(&row.siblings)?;
        amounts(&row.aspects)?;
        ensure!(
            row.item_triggers.len() <= 4096
                && row.aspect_triggers.len() <= 4096
                && row.entity_triggers.len() <= 4096,
            "research triggers exceed their budget"
        );
        for item in &row.item_triggers {
            reference(Kind::Item, item)?;
        }
        for id in &row.aspect_triggers {
            aspect(id)?;
        }
        for entity in &row.entity_triggers {
            ensure!(
                !entity.is_empty() && entity.len() <= 256,
                "invalid entity trigger"
            );
        }
        ensure!(
            row.icon.is_none() || row.texture.is_none(),
            "ambiguous research icon"
        );
        if let Some(icon) = &row.icon {
            reference(Kind::Item, icon)?;
        }
        if let Some(texture) = &row.texture {
            asset(texture)?;
        }
    }
    Ok(())
}
