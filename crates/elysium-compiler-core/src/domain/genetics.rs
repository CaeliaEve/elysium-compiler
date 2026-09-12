use super::{check, content_id, origin_id, Chance, Domain, Kind, Origin, PropertyValue};
use crate::identity::integer;
use anyhow::{ensure, Context, Result};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum SpeciesKind {
    Bee,
    Tree,
}

/// A populated chromosome of a registered template, not a player's individual genome.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Gene {
    pub key: String,
    pub allele: String,
    pub name: String,
    pub dominant: bool,
    /// An API-provided scalar or vector. Providers with executable behavior have no invented value.
    pub value: Option<PropertyValue>,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Member {
    /// Actual item form, e.g. queen, drone, sapling, pollen.
    pub form: String,
    pub item: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Produce {
    pub item: String,
    pub amount: String,
    /// Bee base rate per production cycle. Tree product lists supply no probability: null.
    pub chance: Option<Chance>,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Species {
    pub id: String,
    /// The root UID is the handler; the species allele UID is the key.
    pub source: Origin,
    pub kind: SpeciesKind,
    pub name: String,
    pub description: String,
    pub binomial: String,
    pub authority: String,
    /// Native climate symbols from Forestry, not guessed biome or numeric ranges.
    pub temperature: String,
    pub humidity: String,
    pub dominant: bool,
    pub secret: bool,
    pub blacklisted: bool,
    pub counted: bool,
    /// Natural bee activity; distinct from the template's nocturnal tolerance chromosome.
    pub nocturnal: Option<bool>,
    /// Whether the tree species supports its default template's fruit family.
    pub fruit_compatible: Option<bool>,
    pub members: Vec<Member>,
    pub genes: Vec<Gene>,
    pub products: Vec<Produce>,
    pub specialties: Vec<Produce>,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Mutation {
    /// Content identity: Forestry exposes no mutation registry key.
    pub id: String,
    /// Actual implementation class; conditions remain descriptive, not executable rules.
    pub handler: String,
    /// Zero-based occurrence among identical registrations, retaining independent attempts.
    pub occurrence: u32,
    /// An unordered pair, canonically sorted. Self-crosses retain two equal entries.
    pub parents: [String; 2],
    pub result: String,
    /// Unmodified base rate in 0..1; housing, mode, research and world conditions are not applied.
    pub chance: Chance,
    /// Localized descriptions from the provider, not executable eligibility rules.
    pub conditions: Vec<String>,
    pub secret: bool,
    /// The mutation's actual result template can differ from the species' registered default.
    pub genes: Vec<Gene>,
}

pub(super) fn validate(
    domain: &Domain,
    text: &impl Fn(&str) -> Result<()>,
    reference: &impl Fn(Kind, &str) -> Result<()>,
) -> Result<()> {
    let species: BTreeMap<_, _> = domain
        .species
        .iter()
        .map(|row| (row.id.as_str(), row))
        .collect();
    for row in &domain.species {
        check::origin(&row.source)?;
        ensure!(
            row.id == origin_id("species", &row.source)?,
            "species identity mismatch: {}",
            row.id
        );
        text(&row.name)?;
        text(&row.description)?;
        ensure!(
            row.binomial.len() <= 1024 && row.authority.len() <= 1024,
            "species attribution exceeds its budget"
        );
        symbol(&row.temperature, 64)?;
        symbol(&row.humidity, 64)?;
        ensure!(
            row.nocturnal.is_some() == (row.kind == SpeciesKind::Bee)
                && row.fruit_compatible.is_some() == (row.kind == SpeciesKind::Tree),
            "species traits do not match its kind"
        );
        ensure!(
            !row.members.is_empty() && row.members.len() <= 16,
            "invalid species member count"
        );
        let mut forms = BTreeSet::new();
        for member in &row.members {
            symbol(&member.form, 64)?;
            ensure!(forms.insert(&member.form), "duplicate species member form");
            reference(Kind::Item, &member.item)?;
        }
        genes(&row.genes, text, reference)?;
        ensure!(
            row.products.len() <= 4096 && row.specialties.len() <= 4096,
            "species products exceed their budget"
        );
        for product in row.products.iter().chain(&row.specialties) {
            reference(Kind::Item, &product.item)?;
            integer(&product.amount, 1, i64::MAX)?;
            ensure!(
                product.chance.is_some() == (row.kind == SpeciesKind::Bee),
                "tree possibilities and bee rates must remain distinct"
            );
            if let Some(chance) = &product.chance {
                check::chance(chance)?;
            }
        }
    }
    let mut registrations: BTreeMap<String, BTreeSet<u32>> = BTreeMap::new();
    for mutation in &domain.mutations {
        symbol(&mutation.handler, 512)?;
        ensure!(
            mutation.occurrence < 262144,
            "mutation occurrence exceeds the registry budget"
        );
        let mut definition = serde_json::to_value(mutation)?;
        definition
            .as_object_mut()
            .expect("mutation object")
            .remove("occurrence");
        ensure!(
            registrations
                .entry(content_id("mutation", &definition)?)
                .or_default()
                .insert(mutation.occurrence),
            "duplicate mutation occurrence"
        );
        ensure!(
            mutation.id == content_id("mutation", mutation)?,
            "mutation identity mismatch: {}",
            mutation.id
        );
        ensure!(
            mutation.parents[0] <= mutation.parents[1],
            "mutation parents must be sorted"
        );
        let result = species
            .get(mutation.result.as_str())
            .context("missing mutation result species")?;
        for parent in &mutation.parents {
            let parent = species
                .get(parent.as_str())
                .context("missing mutation parent species")?;
            ensure!(
                parent.kind == result.kind
                    && parent.source.owner == result.source.owner
                    && parent.source.handler == result.source.handler,
                "mutation crosses incompatible species roots"
            );
        }
        check::chance(&mutation.chance)?;
        ensure!(
            mutation.conditions.len() <= 256,
            "too many mutation conditions"
        );
        ensure!(
            mutation
                .conditions
                .windows(2)
                .all(|pair| pair[0] <= pair[1]),
            "mutation conditions must be sorted"
        );
        for condition in &mutation.conditions {
            text(condition)?;
        }
        genes(&mutation.genes, text, reference)?;
    }
    for occurrences in registrations.values() {
        ensure!(
            occurrences.iter().copied().eq(0..occurrences.len() as u32),
            "mutation occurrences must be consecutive"
        );
    }
    Ok(())
}

fn genes(
    genes: &[Gene],
    text: &impl Fn(&str) -> Result<()>,
    reference: &impl Fn(Kind, &str) -> Result<()>,
) -> Result<()> {
    ensure!(
        !genes.is_empty() && genes.len() <= 256,
        "invalid template chromosome count"
    );
    ensure!(
        genes.windows(2).all(|pair| pair[0].key < pair[1].key),
        "chromosomes must be unique and sorted"
    );
    for gene in genes {
        symbol(&gene.key, 128)?;
        symbol(&gene.allele, 512)?;
        ensure!(
            gene.key != "species",
            "the species chromosome is represented by its species reference"
        );
        text(&gene.name)?;
        if let Some(value) = &gene.value {
            check::property_value(value, text, reference, 0)?;
        }
    }
    Ok(())
}

fn symbol(value: &str, limit: usize) -> Result<()> {
    ensure!(
        !value.is_empty() && value.len() <= limit && !value.chars().any(char::is_control),
        "invalid genetics symbol"
    );
    Ok(())
}
