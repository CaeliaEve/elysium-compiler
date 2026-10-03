//! Forestry 4.10.17 factory inventory semantics on independently owned stack values.
pub mod recipes;
pub mod still;
pub mod work;
use crate::identity::Nbt;
use anyhow::{ensure, Result};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ForestryStack {
    pub registry: String,
    pub meta: i32,
    pub amount: i32,
    pub nbt: Option<Nbt>,
    pub ores: Vec<String>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct StockObservation {
    pub direct: i32,
    pub ore: i32,
    pub removed: bool,
    pub remaining: Vec<Option<ForestryStack>>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ForestryFluid {
    pub registry: String,
    pub amount: i32,
    pub nbt: Option<Nbt>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SqueezerRecipe {
    pub time: i32,
    pub requirements: Vec<Option<ForestryStack>>,
    pub fluid: Option<ForestryFluid>,
    pub remnant: Option<ForestryStack>,
    /// Raw IEEE-754 float bits. NaN must not turn into a guaranteed remnant.
    pub chance: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "lowercase", deny_unknown_fields)]
pub enum ContainerKey {
    Stack { stack: ForestryStack },
    Item { registry: String },
    Ore { members: Vec<ForestryStack> },
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SqueezerContainer {
    pub key: ContainerKey,
    pub empty: ForestryStack,
    pub time: i32,
    pub remnant: Option<ForestryStack>,
    pub chance: String,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct FilledContainer {
    pub filled: ForestryStack,
    pub empty: ForestryStack,
    pub fluid: ForestryFluid,
}

/// Snapshot in native iteration order. This model is for registered fixed
/// containers; callers must reject IFluidContainerItem overrides at extraction.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SqueezerProgram {
    pub ordinary: Vec<SqueezerRecipe>,
    pub containers: Vec<SqueezerContainer>,
    pub filled: Vec<FilledContainer>,
    pub dynamic: Vec<String>,
}

/// Stable positions in one content-addressed native registry snapshot.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "lowercase", deny_unknown_fields)]
pub enum SqueezerSelector {
    Ordinary { index: u32 },
    Container { container: u32, filled: u32 },
}

impl SqueezerProgram {
    /// Resolve a cursor entry after whole-context validation at the import boundary.
    /// Validate the selected recipe without rescanning the global rules per row.
    pub fn entry(&self, selector: &SqueezerSelector) -> Result<SqueezerRecipe> {
        let recipe = match selector {
            SqueezerSelector::Ordinary { index } => self
                .ordinary
                .get(*index as usize)
                .ok_or_else(|| anyhow::anyhow!("Squeezer ordinary selector is out of range"))?
                .clone(),
            SqueezerSelector::Container { container, filled } => {
                let fixed = self.filled.get(*filled as usize).ok_or_else(|| {
                    anyhow::anyhow!("Squeezer fixed-fluid selector is out of range")
                })?;
                ensure!(
                    !self.dynamic.contains(&fixed.filled.registry),
                    "Squeezer selector requires an unmodeled Java callback"
                );
                let (index, recipe) = self.fixed_recipe(&fixed.filled, fixed).ok_or_else(|| {
                    anyhow::anyhow!("Squeezer fixed-fluid selector has no native recipe")
                })?;
                ensure!(
                    index == *container as usize,
                    "Squeezer selector bypasses first matching container rule"
                );
                recipe
            }
        };
        recipe.validate()?;
        Ok(recipe)
    }

    fn fixed_recipe(
        &self,
        stack: &ForestryStack,
        filled: &FilledContainer,
    ) -> Option<(usize, SqueezerRecipe)> {
        // FluidHelper returns the original container for a zero drain;
        // negative capacity cannot be drained through the fixed registry.
        let empty = if filled.fluid.amount == 0 {
            stack
        } else if filled.fluid.amount > 0 {
            &filled.empty
        } else {
            return None;
        };
        let (index, container) = self
            .containers
            .iter()
            .enumerate()
            .find(|(_, c)| c.key.matches(empty))?;
        let mut demand = stack.clone();
        demand.amount = 1;
        Some((
            index,
            SqueezerRecipe {
                time: container.time,
                requirements: vec![Some(demand)],
                fluid: Some(filled.fluid.clone()),
                remnant: container.remnant.clone(),
                chance: container.chance.clone(),
            },
        ))
    }

    pub fn select(
        &self,
        stock: &[Option<ForestryStack>],
        retained: Option<&SqueezerRecipe>,
    ) -> Result<Option<SqueezerRecipe>> {
        self.validate()?;
        ensure!(stock.len() <= 9, "Squeezer exceeds nine input slots");
        for stack in stock.iter().flatten() {
            stack.validate()?;
        }
        if let Some(current) = retained {
            current.validate()?;
            if contains_sets(&current.requirements, stock, true) > 0 {
                return Ok(Some(current.clone()));
            }
        }
        // Containers are searched by physical input slot before any ordinary rule.
        for stack in stock.iter().flatten() {
            ensure!(
                !self.dynamic.contains(&stack.registry),
                "Squeezer container requires an unmodeled Java callback: {}",
                stack.registry
            );
            if let Some(filled) = self.filled.iter().find(|c| c.filled.same_type(stack)) {
                if let Some((_, recipe)) = self.fixed_recipe(stack, filled) {
                    return Ok(Some(recipe));
                }
            }
        }
        Ok(self
            .ordinary
            .iter()
            .find(|r| contains_sets(&r.requirements, stock, false) > 0)
            .cloned())
    }

    pub fn validate(&self) -> Result<()> {
        ensure!(
            self.ordinary.len() <= 262144
                && self.containers.len() <= 4096
                && self.filled.len() <= 65536
                && self.dynamic.len() <= 65536,
            "Squeezer registry exceeds budget"
        );
        for registry in &self.dynamic {
            registry_name(registry)?;
        }
        for recipe in &self.ordinary {
            recipe.validate()?;
        }
        for container in &self.containers {
            chance(&container.chance)?;
            if let Some(remnant) = &container.remnant {
                remnant.validate()?;
            }
            container.empty.validate()?;
            match &container.key {
                ContainerKey::Stack { stack } => stack.validate()?,
                ContainerKey::Item { registry } => registry_name(registry)?,
                ContainerKey::Ore { members } => {
                    ensure!(members.len() <= 65536, "Squeezer ore key exceeds budget");
                    for stack in members {
                        stack.validate()?;
                    }
                }
            }
        }
        let mut keys = std::collections::BTreeSet::new();
        for container in &self.filled {
            container.filled.validate()?;
            container.empty.validate()?;
            container.fluid.validate()?;
            ensure!(
                keys.insert((&container.filled.registry, container.filled.meta)),
                "Duplicate fixed fluid container key"
            );
        }
        Ok(())
    }
}

impl ContainerKey {
    fn matches(&self, empty: &ForestryStack) -> bool {
        let exact = |stack: &ForestryStack| empty.same_type(stack) && empty.same_tags(stack);
        match self {
            Self::Stack { stack } => exact(stack),
            Self::Item { registry } => &empty.registry == registry,
            Self::Ore { members } => members.iter().any(exact),
        }
    }
}

impl SqueezerRecipe {
    fn validate(&self) -> Result<()> {
        ensure!(
            self.requirements.len() <= 9,
            "Squeezer demands exceed native display slots"
        );
        for stack in self
            .requirements
            .iter()
            .flatten()
            .chain(self.remnant.iter())
        {
            stack.validate()?;
        }
        if let Some(fluid) = &self.fluid {
            fluid.validate()?;
        }
        chance(&self.chance)?;
        Ok(())
    }
}
impl ForestryFluid {
    fn validate(&self) -> Result<()> {
        registry_name(&self.registry)?;
        compound(&self.nbt)
    }
}
fn registry_name(name: &str) -> Result<()> {
    ensure!(
        !name.is_empty() && name.len() <= 65535,
        "invalid Forestry registry name"
    );
    Ok(())
}
fn compound(nbt: &Option<Nbt>) -> Result<()> {
    if let Some(nbt) = nbt {
        ensure!(
            matches!(nbt, Nbt::Compound { .. }),
            "Forestry stack NBT root must be a compound"
        );
        nbt.validate()?;
    }
    Ok(())
}
fn chance(bits: &str) -> Result<f32> {
    ensure!(
        bits.len() == 8
            && bits
                .bytes()
                .all(|v| v.is_ascii_digit() || (b'a'..=b'f').contains(&v)),
        "invalid Forestry probability bits"
    );
    Ok(f32::from_bits(u32::from_str_radix(bits, 16)?))
}

/// A single Squeezer removal attempt. No container-item return callback runs.
pub fn observe_stock(
    requirements: &[Option<ForestryStack>],
    stock: &[Option<ForestryStack>],
) -> Result<StockObservation> {
    ensure!(
        requirements.len() <= 9 && stock.len() <= 9,
        "Forestry stock exceeds nine slots"
    );
    for stack in requirements.iter().chain(stock).flatten() {
        stack.validate()?;
    }
    let direct = contains_sets(requirements, stock, false);
    let ore = contains_sets(requirements, stock, true);
    let mut remaining = stock.to_vec();
    remaining.resize(9, None);
    let removed = ore >= 1 && !requirements.is_empty();
    if removed {
        // removeSets preflights the complete set only once. It then returns a
        // set.length array, even when an individual removal failed part-way.
        for requirement in requirements.iter().flatten() {
            let mut wanted = requirement.clone();
            if !remove_stack(&mut remaining, &mut wanted, false) {
                remove_stack(&mut remaining, &mut wanted, true);
            }
        }
    }
    Ok(StockObservation {
        direct,
        ore,
        removed,
        remaining,
    })
}

impl ForestryStack {
    fn validate(&self) -> Result<()> {
        registry_name(&self.registry)?;
        ensure!(
            self.ores.len() <= 4096 && self.ores.iter().all(|v| !v.is_empty() && v.len() <= 65535),
            "invalid Forestry ore membership"
        );
        compound(&self.nbt)
    }
    fn same_type(&self, other: &Self) -> bool {
        self.registry == other.registry && self.meta == other.meta
    }
    fn same_tags(&self, other: &Self) -> bool {
        match (&self.nbt, &other.nbt) {
            (None, None) => true,
            (Some(a), Some(b)) => super::matching::native_tag(a, b, true),
            _ => false,
        }
    }
    fn empty_tags(&self) -> bool {
        self.nbt
            .as_ref()
            .is_none_or(|v| matches!(v, Nbt::Compound {value} if value.is_empty()))
    }
    fn equivalent(&self, offered: &Self, ore: bool) -> bool {
        if self.registry == offered.registry
            && (self.meta == 32767 || self.meta == offered.meta)
            && (self.empty_tags() || self.same_tags(offered))
        {
            return true;
        }
        if !(self.empty_tags()
            || self.same_type(offered) && self.amount == offered.amount && self.same_tags(offered))
        {
            return false;
        }
        ore && self.ores.iter().any(|id| offered.ores.contains(id))
    }
}

fn condense(values: &[Option<ForestryStack>]) -> Vec<ForestryStack> {
    let mut condensed: Vec<ForestryStack> = Vec::new();
    for value in values.iter().flatten().filter(|s| s.amount > 0) {
        let mut matched = false;
        for cached in &mut condensed {
            if cached.same_type(value) && cached.same_tags(value) {
                cached.amount = cached.amount.wrapping_add(value.amount);
                matched = true;
            }
        }
        if !matched {
            condensed.push(value.clone());
        }
    }
    condensed
}

fn contains_sets(
    requirements: &[Option<ForestryStack>],
    stock: &[Option<ForestryStack>],
    ore: bool,
) -> i32 {
    let wanted = condense(requirements);
    let offered = condense(stock);
    let mut total = 0;
    for requirement in &wanted {
        let mut count = 0;
        for value in &offered {
            if requirement.equivalent(value, ore) {
                // Native divides float counts, floors the result, then casts to
                // int. Rust's saturating float cast also maps NaN to zero.
                let batches = (value.amount as f32 / requirement.amount as f32).floor() as i32;
                count = count.max(batches);
            }
        }
        if count == 0 {
            return 0;
        }
        if total == 0 || count < total {
            total = count;
        }
    }
    total
}

fn remove_stack(
    stock: &mut [Option<ForestryStack>],
    wanted: &mut ForestryStack,
    ore: bool,
) -> bool {
    for slot in stock {
        let Some(value) = slot.as_mut() else {
            continue;
        };
        if !wanted.equivalent(value, ore) {
            continue;
        }
        // Matches InventoryCrafting/Forestry InventoryAdapter decrStackSize,
        // including malformed zero/negative demands among otherwise valid sets.
        let removed = if value.amount <= wanted.amount {
            let amount = value.amount;
            *slot = None;
            amount
        } else {
            value.amount = value.amount.wrapping_sub(wanted.amount);
            wanted.amount
        };
        wanted.amount = wanted.amount.wrapping_sub(removed);
        if wanted.amount == 0 {
            return true;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    #[test]
    fn native_squeezer_rows_keep_shared_process_and_conditional_quantities() {
        let observed: serde_json::Value = serde_json::from_str(include_str!(
            "../../../../contracts/fixtures/squeezer-adapter-records.json"
        ))
        .unwrap();
        let rows: Vec<super::super::Recipe> =
            serde_json::from_value(observed["recipes"].clone()).unwrap();
        assert_eq!(rows.len(), 6);
        for row in rows {
            assert_eq!(
                serde_json::to_value(&row).unwrap()["process"]["program"],
                observed["programId"]
            );
        }
    }
    use super::*;
    #[derive(Deserialize)]
    struct Case {
        name: String,
        requirements: Vec<Option<ForestryStack>>,
        stock: Vec<Option<ForestryStack>>,
        #[serde(flatten)]
        expected: StockObservation,
    }
    #[derive(Deserialize)]
    struct Cases {
        cases: Vec<Case>,
    }
    #[test]
    fn stock_replays_real_forestry_matching_and_removal() {
        let cases: Cases = serde_json::from_str(include_str!(
            "../../../../contracts/fixtures/forestry-stock-observations.json"
        ))
        .unwrap();
        for case in cases.cases {
            assert_eq!(
                observe_stock(&case.requirements, &case.stock).unwrap(),
                case.expected,
                "{}",
                case.name
            );
        }
    }
    #[test]
    fn squeezer_replays_native_priority_retention_and_fixed_containers() {
        #[derive(Deserialize)]
        struct SelectionCase {
            name: String,
            program: SqueezerProgram,
            stock: Vec<Option<ForestryStack>>,
            retained: Option<SqueezerRecipe>,
            selected: Option<SqueezerRecipe>,
        }
        #[derive(Deserialize)]
        struct SelectionCases {
            cases: Vec<SelectionCase>,
        }
        let cases: SelectionCases = serde_json::from_str(include_str!(
            "../../../../contracts/fixtures/forestry-selection-observations.json"
        ))
        .unwrap();
        for case in cases.cases {
            assert_eq!(
                case.program
                    .select(&case.stock, case.retained.as_ref())
                    .unwrap(),
                case.selected,
                "{}",
                case.name
            );
        }
    }
    #[test]
    fn cursor_selectors_resolve_exact_native_entries() {
        #[derive(Deserialize)]
        struct Entry {
            selector: SqueezerSelector,
            recipe: SqueezerRecipe,
        }
        #[derive(Deserialize)]
        struct Observations {
            program: SqueezerProgram,
            entries: Vec<Entry>,
        }
        let observations: Observations = serde_json::from_str(include_str!(
            "../../../../contracts/fixtures/forestry-entry-observations.json"
        ))
        .unwrap();
        for entry in &observations.entries {
            assert_eq!(
                observations.program.entry(&entry.selector).unwrap(),
                entry.recipe
            );
        }
        assert!(observations
            .program
            .entry(&SqueezerSelector::Ordinary { index: u32::MAX })
            .is_err());
        let fixed = observations
            .entries
            .iter()
            .find_map(|entry| match entry.selector {
                SqueezerSelector::Container { container, filled } => Some((container, filled)),
                _ => None,
            })
            .unwrap();
        assert!(observations
            .program
            .entry(&SqueezerSelector::Container {
                container: u32::MAX,
                filled: fixed.1
            })
            .is_err());
        assert!(observations
            .program
            .entry(&SqueezerSelector::Container {
                container: fixed.0,
                filled: u32::MAX
            })
            .is_err());
        let mut changed = observations.program.clone();
        changed
            .containers
            .insert(0, changed.containers[fixed.0 as usize].clone());
        assert!(
            changed
                .entry(&SqueezerSelector::Container {
                    container: fixed.0 + 1,
                    filled: fixed.1
                })
                .is_err(),
            "Selector bypassed native first-key precedence"
        );
        let mut callback = observations.program.clone();
        callback
            .dynamic
            .push(callback.filled[fixed.1 as usize].filled.registry.clone());
        assert!(
            callback
                .entry(&SqueezerSelector::Container {
                    container: fixed.0,
                    filled: fixed.1
                })
                .is_err(),
            "Selector turned a callback into a fixed recipe"
        );
    }
    #[test]
    fn exported_snapshot_is_owned_and_does_not_flatten_java_callbacks() {
        let mut program: SqueezerProgram = serde_json::from_str(include_str!(
            "../../../../contracts/fixtures/forestry-squeezer-context.json"
        ))
        .unwrap();
        let filled = program
            .filled
            .iter()
            .find(|c| c.filled.registry == "minecraft:water_bucket")
            .unwrap()
            .filled
            .clone();
        let selected = program
            .select(&[Some(filled.clone())], None)
            .unwrap()
            .unwrap();
        assert_eq!(selected.fluid.unwrap().amount, 1000);
        assert_eq!(selected.requirements[0].as_ref().unwrap().amount, 1);
        program.dynamic.push(filled.registry.clone());
        assert!(
            program.select(&[Some(filled.clone())], None).is_err(),
            "dynamic callbacks were evaluated as fixed containers"
        );
        // A retained valid recipe is checked before dispatch to container callbacks.
        let retained = SqueezerRecipe {
            time: 1,
            requirements: vec![Some(filled.clone())],
            fluid: None,
            remnant: None,
            chance: "00000000".into(),
        };
        assert!(program
            .select(&[Some(filled)], Some(&retained))
            .unwrap()
            .is_some());
    }
}
