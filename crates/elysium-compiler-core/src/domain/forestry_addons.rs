//! Native species production and mutation information, without consuming recipe semantics.
use super::{
    references::{GeneticReferenceRow, ReferenceData},
    Item, Origin,
};
use anyhow::{ensure, Context, Result};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum Mode {
    Produce,
    Breeding,
}
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    Producer,
    Parent1,
    Parent2,
    Offspring,
}
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum OutputRole {
    Product,
    Specialty,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum Probability {
    NominalSpeciesValues,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum OutputOrder {
    StableRoleItemAmountFloatBits,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum Specimens {
    NativeAnalyzedTemplates,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Scalar {
    pub bits: String,
    pub decimal: String,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Counts {
    pub native_entries: u32,
    pub species: u32,
    pub outputs: u32,
    pub mutations: u32,
    pub eligible: u32,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Definition {
    pub source: Origin,
    pub root: String,
    pub mode: Mode,
    pub ready: bool,
    pub consuming: bool,
    pub show_secret: Option<bool>,
    pub probability: Probability,
    pub output_order: OutputOrder,
    pub specimens: Specimens,
    pub member_types: BTreeMap<Role, u8>,
    pub counts: Counts,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Specimen {
    pub role: Role,
    pub native_type: u8,
    pub item: String,
    pub amount: String,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Species {
    pub ordinal: u32,
    pub species: String,
    pub name: String,
    pub template: Vec<Option<String>>,
    pub specimens: Vec<Specimen>,
    pub listed: bool,
    pub outputs: u32,
    pub jubilance_provider: Option<String>,
    pub jubilance: Option<String>,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Output {
    pub species: String,
    pub ordinal: u32,
    pub role: OutputRole,
    pub item: String,
    pub amount: String,
    pub rate: Scalar,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Mutation {
    pub ordinal: u32,
    pub owner: String,
    pub parents: [String; 2],
    pub result: String,
    pub template: Vec<Option<String>>,
    pub base: Scalar,
    pub secret: bool,
    pub eligible: bool,
    pub conditions: Option<Vec<String>>,
}
fn uid(text: &str) -> bool {
    !text.is_empty() && text.len() <= 256
}
fn text(text: &str) -> bool {
    !text.is_empty() && text.len() <= 65536
}
fn amount(text: &str) -> Result<u32> {
    ensure!(
        text.len() <= 10 && text.bytes().all(|v| v.is_ascii_digit()) && !text.starts_with('0'),
        "Invalid native specimen/output amount"
    );
    let amount = text.parse::<u32>()?;
    ensure!(
        amount <= i32::MAX as u32,
        "Native amount overflows Java int"
    );
    Ok(amount)
}
fn scalar(value: &Scalar) -> Result<()> {
    ensure!(
        value.bits.len() == 8
            && value
                .bits
                .bytes()
                .all(|v| v.is_ascii_digit() || (b'a'..=b'f').contains(&v))
            && value.decimal.len() <= 64,
        "Invalid native Float encoding"
    );
    let expected = u32::from_str_radix(&value.bits, 16)?;
    let native = f32::from_bits(expected);
    let parsed = value
        .decimal
        .parse::<f32>()
        .context("Native Float decimal is invalid")?;
    ensure!(
        native.is_finite() && native >= 0.0 && parsed.to_bits() == expected,
        "Native Float rate was clamped, rounded or invalid"
    );
    Ok(())
}
fn template(values: &[Option<String>], species: &str) -> Result<()> {
    ensure!(
        !values.is_empty()
            && values.len() <= 256
            && values[0].as_deref() == Some(species)
            && values.iter().flatten().all(|s| uid(s)),
        "Incomplete native genotype or species binding"
    );
    Ok(())
}
fn binding(handler: &str) -> Option<(&'static str, Mode)> {
    Some(match handler {
        "net.bdew.neiaddons.forestry.bees.BeeProduceHandler" => ("rootBees", Mode::Produce),
        "net.bdew.neiaddons.forestry.trees.TreeProduceHandler" => ("rootTrees", Mode::Produce),
        "net.bdew.neiaddons.forestry.bees.BeeBreedingHandler" => ("rootBees", Mode::Breeding),
        "net.bdew.neiaddons.forestry.trees.TreeBreedingHandler" => ("rootTrees", Mode::Breeding),
        "net.bdew.neiaddons.forestry.butterflies.ButterflyBreedingHandler" => {
            ("rootButterflies", Mode::Breeding)
        }
        "net.bdew.neiaddons.botany.flowers.FlowerBreedingHandler" => {
            ("rootFlowers", Mode::Breeding)
        }
        _ => return None,
    })
}
pub fn validate(
    id: &str,
    rows: &[&GeneticReferenceRow],
    items: &BTreeMap<&str, &Item>,
) -> Result<()> {
    let ReferenceData::ForestryAddonsDefinition(def) = &rows[0].data else {
        anyhow::bail!("Native NEIAddons definition absent")
    };
    let (root, mode) = binding(&def.source.handler).context("Unknown native NEIAddons handler")?;
    ensure!(
        def.ready
            && !def.consuming
            && def.root == root
            && def.mode == mode
            && !def.source.owner.is_empty()
            && !def.source.key.is_empty(),
        "Native NEIAddons identity, readiness or mode differs"
    );
    let data: Vec<_> = rows.iter().map(|row| &row.data).collect();
    ensure!(
        id == super::value_id(
            "reference",
            &serde_json::json!({"kind":"forestryAddons","records":data})
        )?,
        "Native NEIAddons content identity differs"
    );
    let count = &def.counts;
    ensure!(
        count.species <= 65536
            && count.outputs <= 262144
            && count.mutations <= 262144
            && rows.len()
                == 1 + count.species as usize + count.outputs as usize + count.mutations as usize,
        "Native NEIAddons counts or closure incomplete"
    );
    if mode == Mode::Produce {
        ensure!(
            def.show_secret.is_none()
                && count.native_entries == count.species
                && count.mutations == 0
                && count.eligible == 0
                && def.member_types.keys().eq([Role::Producer].iter()),
            "Producer became a mutation/consuming domain"
        );
    } else {
        ensure!(
            def.show_secret.is_some()
                && count.native_entries == count.mutations
                && count.outputs == 0
                && def
                    .member_types
                    .keys()
                    .eq([Role::Parent1, Role::Parent2, Role::Offspring].iter()),
            "Mutation native enumeration or specimen role map differs"
        );
    }
    let mut species = BTreeMap::new();
    for (ordinal, row) in rows[1..1 + count.species as usize].iter().enumerate() {
        let ReferenceData::ForestryAddonsSpecies(value) = &row.data else {
            anyhow::bail!("Native species leaf absent")
        };
        ensure!(
            value.ordinal as usize == ordinal
                && uid(&value.species)
                && text(&value.name)
                && species.insert(value.species.as_str(), value).is_none(),
            "Invalid or duplicate native species enumeration"
        );
        template(&value.template, &value.species)?;
        ensure!(
            value.specimens.len() == def.member_types.len(),
            "Native parent/member form omitted"
        );
        let mut roles = BTreeSet::new();
        for form in &value.specimens {
            ensure!(
                roles.insert(form.role.clone())
                    && def.member_types.get(&form.role) == Some(&form.native_type)
                    && items.contains_key(form.item.as_str()),
                "Native analyzed specimen/role fact missing"
            );
            amount(&form.amount)?;
        }
        ensure!(
            value.jubilance.as_deref().map_or(true, text)
                && value.jubilance_provider.as_deref().map_or(true, text)
                && !(value.jubilance.is_some() && value.jubilance_provider.is_none()),
            "Native jubilance description lost its owner"
        );
        if mode == Mode::Breeding {
            ensure!(
                value.listed
                    && value.outputs == 0
                    && value.jubilance.is_none()
                    && value.jubilance_provider.is_none(),
                "Breeding species gained production semantics"
            );
        }
    }
    let mut outputs = BTreeMap::<&str, Vec<&Output>>::new();
    let mut previous_species = 0u32;
    for row in
        &rows[1 + count.species as usize..1 + count.species as usize + count.outputs as usize]
    {
        let ReferenceData::ForestryAddonsOutput(output) = &row.data else {
            anyhow::bail!("Native product/specialty leaf absent")
        };
        let value = species
            .get(output.species.as_str())
            .context("Output producer species missing")?;
        ensure!(
            value.ordinal >= previous_species && items.contains_key(output.item.as_str()),
            "Native output order or item fact differs"
        );
        previous_species = value.ordinal;
        amount(&output.amount)?;
        scalar(&output.rate)?;
        let values = outputs.entry(output.species.as_str()).or_default();
        ensure!(
            output.ordinal as usize == values.len(),
            "Native output ordinal incomplete"
        );
        if let Some(prior) = values.last() {
            ensure!(
                (
                    prior.role.clone(),
                    prior.item.as_str(),
                    amount(&prior.amount)?,
                    prior.rate.bits.as_str()
                ) <= (
                    output.role.clone(),
                    output.item.as_str(),
                    amount(&output.amount)?,
                    output.rate.bits.as_str()
                ),
                "Stable native output ordering differs"
            );
        }
        values.push(output);
    }
    if mode == Mode::Produce {
        for (uid, value) in &species {
            let size = outputs.get(uid).map_or(0, Vec::len);
            ensure!(
                value.outputs as usize == size && value.listed == (size != 0),
                "Native specialty/output coverage or empty listing differs"
            );
        }
    }
    let mut eligible = 0;
    for (ordinal, row) in rows[1 + count.species as usize + count.outputs as usize..]
        .iter()
        .enumerate()
    {
        let ReferenceData::ForestryAddonsMutation(m) = &row.data else {
            anyhow::bail!("Native mutation leaf missing")
        };
        ensure!(
            m.ordinal as usize == ordinal
                && text(&m.owner)
                && m.parents
                    .iter()
                    .chain([&m.result])
                    .all(|uid| species.contains_key(uid.as_str()))
                && m.eligible == (!m.secret || def.show_secret.unwrap()),
            "Native parents/result/order/secret policy differs"
        );
        template(&m.template, &m.result)?;
        scalar(&m.base)?;
        if let Some(conditions) = &m.conditions {
            ensure!(
                conditions.len() <= 256 && conditions.iter().all(|value| text(value)),
                "Invalid native mutation condition descriptions"
            );
        }
        if m.eligible {
            eligible += 1;
        }
    }
    ensure!(
        eligible == count.eligible,
        "Native eligible mutation count differs"
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn native_forestry_addons_domains_reject_missing_specialty_parent_and_rate_drift() {
        let fixtures =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../contracts/fixtures");
        let raw: serde_json::Value = serde_json::from_slice(
            &std::fs::read(fixtures.join("forestry-addons-contract.json")).unwrap(),
        )
        .unwrap();
        let source = crate::source::Source::open(&fixtures.join("source-semantic-40")).unwrap();
        let mut domain = super::super::Domain::load(&source).unwrap();
        let base = serde_json::to_value(&domain.items[0]).unwrap();
        for item in raw["items"].as_array().unwrap() {
            let mut value = base.clone();
            for key in ["id", "registry", "meta", "nbt", "tags"] {
                value[key] = item[key].clone();
            }
            let item: Item = serde_json::from_value(value).unwrap();
            domain.items.retain(|old| old.id != item.id);
            domain.items.push(item);
        }
        let groups: Vec<Vec<GeneticReferenceRow>> =
            serde_json::from_value(raw["domains"].clone()).unwrap();
        for rows in &groups {
            let ReferenceData::ForestryAddonsDefinition(def) = &rows[0].data else {
                panic!()
            };
            let mut category = domain.categories[0].clone();
            category.source = def.source.clone();
            category.id = super::super::origin_id("category", &category.source).unwrap();
            category.reference = Some(rows[0].domain.clone());
            category.program = None;
            category.icon = None;
            category.machines.clear();
            domain.categories.push(category);
        }
        domain.references = groups.iter().flatten().cloned().collect();
        domain.validate(&source).unwrap();
        let facts = domain
            .items
            .iter()
            .map(|item| (item.id.as_str(), item))
            .collect();
        let reject = |mut rows: Vec<GeneticReferenceRow>| {
            let data: Vec<_> = rows.iter().map(|row| &row.data).collect();
            let id = super::super::value_id(
                "reference",
                &serde_json::json!({"kind":"forestryAddons","records":data}),
            )
            .unwrap();
            for row in &mut rows {
                row.domain = id.clone();
                row.id = row.content_id().unwrap();
            }
            assert!(super::super::references::validate(&rows, &facts).is_err());
        };
        let mut missing = groups[0].clone();
        missing.pop();
        reject(missing);
        let mut parent = groups[2].clone();
        if let ReferenceData::ForestryAddonsSpecies(s) = &mut parent[1].data {
            s.specimens.pop();
        }
        reject(parent);
        let mut rate = groups[0].clone();
        for row in &mut rate {
            if let ReferenceData::ForestryAddonsOutput(o) = &mut row.data {
                o.rate.decimal = "0.0".into();
                break;
            }
        }
        reject(rate);
        let mut hidden = groups[2].clone();
        for row in &mut hidden {
            if let ReferenceData::ForestryAddonsMutation(m) = &mut row.data {
                if m.secret {
                    m.eligible = true;
                    break;
                }
            }
        }
        reject(hidden);
    }
}
