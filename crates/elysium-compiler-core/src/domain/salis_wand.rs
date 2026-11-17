//! Salis' finite inventory partitions and two distinct charge-clamping moments.
//! Player discounts, loaded mixins and world/research execution remain conditions.
use super::{Aspect, Consumption, Edit, Item, Kind, MagicKind, Match, OutputRole, Recipe};
use crate::identity::Nbt;
use anyhow::{bail, ensure, Context, Result};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum Part { Rod, Cap }
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum Role { Center, Part, Screw, Conductor }
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub enum PaymentCondition {
    #[serde(rename = "loadedSalisWorkbenchMixinAndNativePlayerPayment")]
    LoadedSalisWorkbenchMixinAndNativePlayerPayment,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct WandMaterial { pub screw: String, pub conductor: super::infusion::Template, pub screws: u8 }
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SalisReplacement {
    pub native_recipe: String,
    pub part: Part,
    pub staff: bool,
    pub sceptre: bool,
    pub enforce_staff: bool,
    pub target_staff: bool,
    pub staff_attributes: Option<Nbt>,
    pub roles: Vec<Role>,
    pub material: Option<WandMaterial>,
    pub cost: i32,
    pub preserve: bool,
    pub charge_capacity: i32,
    pub result_capacity: i32,
    pub self_payment: bool,
    pub payment_condition: PaymentCondition,
}

// Only the identity predicates produced by the pinned Salis reader are admitted.
// Exclusions are one level deep, so this cannot become an arbitrary game evaluator.
fn primitive(rule: &Match) -> bool {
    matches!(rule, Match::Wildcard { .. } | Match::Tags { .. } | Match::Ore { .. })
}
fn domain<'a>(r: &'a Recipe, items: &BTreeMap<&str, &'a Item>) -> Result<Vec<Vec<(&'a Match, &'a Item)>>> {
    r.inputs.iter().map(|input| {
        let mut cases = BTreeMap::new();
        for choice in &input.choices {
            let anchor = *items.get(choice.id.as_str()).context("missing Salis role anchor")?;
            let base = match &choice.rule { Match::Except { base, .. } => base.as_ref(), rule => rule };
            let state = match base {
                Match::Ore { .. } => serde_json::Value::Null,
                Match::Wildcard { meta, nbt } => serde_json::json!([anchor.registry, if *meta { None } else { Some(anchor.meta) }, if *nbt { None } else { anchor.nbt.as_ref() }]),
                Match::Tags { meta, keys, .. } => {
                    let tags = match &anchor.nbt { Some(Nbt::Compound { value }) => Some(value), _ => None };
                    serde_json::json!([anchor.registry, if *meta { None } else { Some(anchor.meta) }, keys.iter().map(|key|tags.and_then(|t|t.get(key))).collect::<Vec<_>>()])
                }
                _ => bail!("unknown Salis inventory predicate"),
            };
            cases.insert(serde_json::to_string(&(&choice.rule, state))?, (&choice.rule, anchor));
        }
        Ok(cases.into_values().collect())
    }).collect()
}
fn accepts(rule: &Match, offered: &Item, anchor: &Item, items: &BTreeMap<&str, &Item>) -> Result<bool> {
    if let Match::Except { base, exclude } = rule {
        ensure!(primitive(base) && !exclude.is_empty() && exclude.len() <= 4096, "invalid Salis role exclusions");
        if !super::matching::simple(base, offered, anchor) { return Ok(false); }
        for entry in exclude {
            ensure!(matches!(entry.rule, Match::Wildcard { nbt: true, .. }), "Salis exclusions must ignore NBT");
            let excluded = items.get(entry.id.as_str()).context("missing Salis classification anchor")?;
            if super::matching::simple(&entry.rule, offered, excluded) { return Ok(false); }
        }
        return Ok(true);
    }
    ensure!(primitive(rule), "unknown Salis inventory predicate");
    Ok(super::matching::simple(rule, offered, anchor))
}

/// Counts physical occupied cells in the first nine slots, never stack amounts.
fn inventory_match(r: &Recipe, offered: &[Option<&Item>], items: &BTreeMap<&str, &Item>) -> Result<bool> {
    let groups = domain(r, items)?;
    let mut remaining: Vec<usize> = (0..r.inputs.len()).collect();
    for item in offered.iter().take(9).flatten() {
        let mut selected = None;
        for (at, slot) in remaining.iter().enumerate() {
            for (rule, anchor) in &groups[*slot] {
                if accepts(rule, item, anchor, items)? { selected = Some(at); break; }
            }
            if selected.is_some() { break; }
        }
        let Some(at) = selected else { return Ok(false); };
        remaining.remove(at);
    }
    Ok(remaining.is_empty())
}

pub(super) fn validate(p: &SalisReplacement, r: &Recipe, items: &BTreeMap<&str, &Item>, aspects: &[Aspect]) -> Result<()> {
    let native = match p.part { Part::Rod => "ReplaceWandCoreRecipe", Part::Cap => "ReplaceWandCapsRecipe" };
    ensure!(p.native_recipe == format!("dev.rndmorris.salisarcana.common.recipes.{native}")
        && super::magic::ari_source(&r.source, super::magic::ARI_SHAPELESS)
        && r.grid.is_none() && r.duration.is_none() && r.energy.is_none() && r.properties.is_empty()
        && p.charge_capacity >= 0 && p.result_capacity >= 0 && (!p.self_payment || !p.staff)
        && (!p.target_staff || p.part == Part::Rod)
        && (p.part != Part::Rod || !p.enforce_staff || p.staff == p.target_staff), "invalid native Salis replacement");
    let mut roles = vec![Role::Center, Role::Part];
    if p.part == Part::Rod {
        if let Some(material) = &p.material {
            ensure!(!material.screw.is_empty() && material.screw.len() <= 4096 && material.screws == if p.sceptre { 2 } else { 4 }, "invalid Salis screw law");
            crate::identity::registry_name(&material.conductor.registry)?;
            if let Some(nbt) = &material.conductor.nbt { nbt.validate()?; }
            roles.extend(std::iter::repeat_n(Role::Screw, material.screws as usize));
            roles.extend([Role::Conductor, Role::Conductor]);
        }
    } else {
        ensure!(p.material.is_none(), "cap replacement cannot require core materials");
        roles.extend(std::iter::repeat_n(Role::Part, if p.sceptre { 2 } else { 1 }));
    }
    ensure!(roles == p.roles && roles.len() == r.inputs.len() && r.outputs.len() == 1, "Salis physical cell roles differ");
    let magic = r.magic.as_ref().context("Salis magic costs missing")?;
    ensure!(magic.kind == MagicKind::Arcane && magic.central.is_none() && magic.instability.is_none()
        && !magic.research.is_empty() && magic.research.len() <= 2 && magic.payment.is_some() == p.self_payment,
        "invalid Salis research or payment condition");
    if p.cost < 0 { ensure!(magic.aspects.is_empty() && magic.creative, "negative native cost requires creative waiver"); }
    else {
        let mut keys = BTreeSet::new();
        for cost in &magic.aspects {
            let aspect = aspects.iter().find(|a| a.id == cost.aspect).context("Salis aspect missing")?;
            ensure!(aspect.components.is_empty() && aspect.source.owner == "Thaumcraft" && cost.amount == p.cost.to_string()
                && keys.insert(aspect.source.key.as_str()), "Salis primal cost differs");
        }
        ensure!(keys == BTreeSet::from(["aer", "aqua", "ignis", "ordo", "perditio", "terra"]), "Salis six primal costs missing");
    }
    if let Some(payment) = &magic.payment {
        ensure!(payment.input == 0 && payment.capacity == p.result_capacity && payment.preserve == p.preserve
            && payment.charges.len() == 6, "Salis post-payment clamp differs");
    }
    let groups = domain(r, items)?;
    for (slot, input) in r.inputs.iter().enumerate() {
        ensure!(input.kind == Kind::Item && input.slot == slot as u32 && !input.choices.is_empty(), "invalid Salis input group");
        let role = p.roles[slot];
        let prior = p.roles[..slot].iter().position(|v| *v == role);
        if let Some(prior) = prior {
            ensure!(serde_json::to_value(&input.choices)? == serde_json::to_value(&r.inputs[prior].choices)?, "repeated Salis role has a different domain");
        }
        for choice in &input.choices {
            ensure!(choice.amount == "1" && matches!(choice.consume, Consumption::Consume | Consumption::Keep | Consumption::Damage { .. } | Consumption::GtTool { .. } | Consumption::NativeContainer { .. }), "Salis consumes one per occupied cell");
            let item = items.get(choice.id.as_str()).context("Salis input item missing")?;
            let base = match &choice.rule { Match::Except { base, .. } => base.as_ref(), rule => rule };
            match role {
                Role::Center => ensure!(matches!(base, Match::Wildcard { meta: false, nbt: true } | Match::Tags { meta: true, .. }), "invalid Salis center state"),
                Role::Part | Role::Conductor => ensure!(matches!(choice.rule, Match::Wildcard { meta: false, nbt: true }), "Salis component metadata must be literal"),
                Role::Screw => ensure!(matches!(base, Match::Ore { name, exclusive: false } if p.material.as_ref().is_some_and(|m|m.screw == *name))
                    && matches!(choice.rule, Match::Except { .. }), "Salis screw classification partition missing"),
            }
            ensure!(accepts(&choice.rule, item, item, items)?, "Salis role rejects its native specimen");
            if role == Role::Conductor {
                let conductor = &p.material.as_ref().context("Salis conductor rule missing")?.conductor;
                ensure!(item.registry == conductor.registry && item.meta == conductor.metadata, "Salis conductor differs from native item/metadata");
            }
            // Distinct role partitions retain the native component/wand/conductor
            // precedence without depending on an assignment solver's ordering.
            for (other, group) in groups.iter().enumerate().filter(|(i,_)|p.roles[*i] != role) {
                for (rule, anchor) in group {
                    ensure!(!accepts(rule, item, anchor, items)?, "Salis specimen overlaps distinct physical roles {slot}/{other}");
                }
            }
        }
    }
    let offered: Vec<_> = r.inputs.iter().map(|i| items.get(i.choices[0].id.as_str()).copied()).collect();
    ensure!(inventory_match(r, &offered, items)?, "Salis first inventory specimen does not match");
    let output = &r.outputs[0];
    let change = output.change.as_ref().context("Salis output edit missing")?;
    let Edit::SalisWand { set, limits, metadata } = &change.action else { bail!("Salis output requires native replacement edit"); };
    ensure!(change.input == 0 && change.bindings.is_none() && *metadata == p.cost.max(0) && output.kind == Kind::Item
        && output.slot == 0 && output.role == OutputRole::Result && output.amount.as_deref() == Some("1") && output.quantity.is_none()
        && output.chance.numerator == "1" && output.chance.denominator == "1", "invalid Salis output sample");
    let component = if p.part == Part::Rod { "rod" } else { "cap" };
    ensure!(matches!(set.get(component), Some(Nbt::String { value }) if !value.is_empty()), "Salis replaced component missing");
    let mut written = BTreeSet::from([component]);
    if p.target_staff {
        let attributes = p.staff_attributes.as_ref().context("Salis staff attributes missing")?;
        ensure!(matches!(attributes, Nbt::List { element, value } if element == "compound" && value.len() == 1)
            && set.get("AttributeModifiers") == Some(attributes), "Salis staff must replace the complete native attribute list");
        written.insert("AttributeModifiers");
    } else { ensure!(p.staff_attributes.is_none(), "non-staff component cannot supply native staff attributes"); }
    let primal: BTreeSet<_> = ["aer", "aqua", "ignis", "ordo", "perditio", "terra"].into_iter().collect();
    if !p.preserve {
        written.extend(primal.iter().copied());
        ensure!(limits.is_empty() && primal.iter().all(|key|matches!(set.get(*key), Some(Nbt::Int { value }) if value == "0")), "Salis clearing differs");
    } else {
        ensure!(if p.part == Part::Rod {
            limits.len() == 6 && primal.iter().all(|key|limits.get(*key) == Some(&p.charge_capacity))
        } else { limits.is_empty() }, "Salis pre-metadata charge clamp differs");
    }
    ensure!(set.keys().map(String::as_str).collect::<BTreeSet<_>>() == written, "Salis edit changes unrelated fields");
    Ok(())
}
