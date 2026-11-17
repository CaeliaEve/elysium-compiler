//! ProjectBlue native dynamic crafting rules. Examples are not consuming recipes.
use super::{
    references::{GeneticReferenceRow, ReferenceData},
    Item, Origin,
};
use crate::identity::{item_id, Nbt};
use anyhow::{ensure, Context, Result};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(
    Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "camelCase")]
#[schemars(rename = "ProjectBlueCoreName")]
pub enum CoreName {
    ControlPanelItem,
    MiniatureLever,
    MiniatureButton,
    MiniatureLamp,
    MiniatureCover,
    SprayCan,
    EmptySprayCan,
    ItemMicroPart,
    ItemPartFixture,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(rename = "ProjectBlueMode")]
pub enum Mode {
    Panel,
    Miniature,
    Paint,
}
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "camelCase")]
#[schemars(rename = "ProjectBlueItemRole")]
pub enum Role {
    Saw,
    Spray,
    Lever,
    Button,
    FullLever,
    StoneButton,
    WoodButton,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(rename = "ProjectBluePickup")]
pub enum Pickup {
    VanillaWorkbenchAllOccupiedCells,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(rename = "ProjectBlueMaterialPolicy")]
pub enum MaterialPolicy {
    DefaultWood,
    CoverRequiresMaterial,
    NumericSprayColor,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(rename = "ProjectBlueCounts")]
pub struct Counts {
    pub rules: u32,
    pub roles: u32,
    pub materials: u32,
    pub examples: u32,
    pub items: u32,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[schemars(rename = "ProjectBlueDefinition")]
pub struct Definition {
    pub source: Origin,
    pub ready: bool,
    pub consuming: bool,
    pub grid_width: u8,
    pub grid_height: u8,
    pub pickup: Pickup,
    pub native_rules: u32,
    pub core: BTreeMap<CoreName, String>,
    pub counts: Counts,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[schemars(rename = "ProjectBlueRule")]
pub struct Rule {
    pub index: u32,
    pub recipe_class: String,
    pub mode: Mode,
    pub inputs: Vec<u8>,
    pub required_empty: Vec<u8>,
    pub ignored: Vec<u8>,
    pub output_amount: u8,
    pub material_policy: MaterialPolicy,
    pub anchored: bool,
    pub mirrored: bool,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[schemars(rename = "ProjectBlueItem")]
pub struct ItemRoles {
    pub registry: String,
    pub roles: Vec<Role>,
    pub max_damage: Option<i32>,
    pub damageable: Option<bool>,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(rename = "ProjectBlueCachedMaterial")]
pub struct CachedMaterial {
    pub key: String,
    pub name: Option<String>,
    pub block: Option<String>,
    pub metadata: Option<i32>,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[schemars(rename = "ProjectBlueRegisteredMaterial")]
pub struct RegisteredMaterial {
    pub key: String,
    pub owner: Option<String>,
    pub source_block: Option<String>,
    pub source_metadata: i32,
    pub item_block: bool,
    pub block: String,
    pub metadata: i32,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(rename = "ProjectBlueMaterialAlias")]
pub struct Alias {
    pub key: String,
    pub target: String,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(rename = "ProjectBlueRawStack")]
pub struct RawStack {
    pub registry: String,
    pub meta: i32,
    pub amount: i32,
    pub nbt: Option<Nbt>,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(rename = "ProjectBlueStack")]
pub struct Stack {
    pub item: String,
    pub amount: String,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(rename = "ProjectBlueExample")]
#[serde(rename_all = "camelCase")]
pub struct Example {
    pub rule: u32,
    pub cells: Vec<Option<Stack>>,
    pub material_key: Option<String>,
    pub output: Stack,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "camelCase", deny_unknown_fields)]
pub enum Outcome {
    Unmatched,
    Output { stack: RawStack },
    Error { phase: String, reason: String },
}

#[derive(Default)]
pub struct Materials {
    pub cache: BTreeMap<String, CachedMaterial>,
    pub registry: BTreeMap<String, RegisteredMaterial>,
    pub aliases: BTreeMap<String, String>,
}
impl Materials {
    fn resolve(&self, key: &str) -> (String, i32) {
        if let Some(cached) = self.cache.get(key) {
            if let Some(name) = &cached.name {
                return (name.clone(), cached.metadata.unwrap());
            }
        }
        let key_alias = self.aliases.get(key).map_or(key, String::as_str);
        (
            key.into(),
            self.registry
                .get(key_alias)
                .map_or(0, |entry| entry.metadata),
        )
    }
}
fn tag<'a>(stack: &'a RawStack, key: &str) -> Option<&'a Nbt> {
    if let Some(Nbt::Compound { value }) = &stack.nbt {
        value.get(key)
    } else {
        None
    }
}
fn native_material_key(
    stack: &RawStack,
    default: Option<&str>,
    projection: Option<&str>,
) -> Result<String> {
    let value = match tag(stack, "mat") {
        None => default
            .context("Native cover requires a material tag")?
            .to_owned(),
        Some(Nbt::String { value }) => value.clone(),
        // Non-string NBT uses NBTBase's native text conversion. Typed NBT sorts
        // compound keys for identity and cannot reconstruct native HashMap order.
        // Java observes this projection on the exact offered specimen; do not
        // guess it or substitute an empty string during compilation.
        Some(_) => projection
            .context("Native material string projection is absent")?
            .to_owned(),
    };
    key(&value)?;
    ensure!(
        projection.is_none_or(|observed| observed == value),
        "Native ProjectBlue material projection differs"
    );
    Ok(value)
}
fn floor_i32(value: f64) -> i32 {
    let int = value as i32;
    if value < (int as f64) {
        int.wrapping_sub(1)
    } else {
        int
    }
}
fn native_integer(value: Option<&Nbt>) -> Result<i32> {
    Ok(match value {
        Some(Nbt::Byte { value }) | Some(Nbt::Short { value }) | Some(Nbt::Int { value }) => {
            value.parse()?
        }
        Some(Nbt::Long { value }) => value.parse::<i64>()? as i32,
        Some(Nbt::Float { value }) => {
            floor_i32(f32::from_bits(u32::from_str_radix(value, 16)?) as f64)
        }
        Some(Nbt::Double { value }) => floor_i32(f64::from_bits(u64::from_str_radix(value, 16)?)),
        _ => 0,
    })
}
fn mat_tag(value: String) -> Option<Nbt> {
    Some(Nbt::Compound {
        value: BTreeMap::from([("mat".into(), Nbt::String { value })]),
    })
}
fn output(registry: &str, meta: i32, amount: i32, nbt: Option<Nbt>) -> Outcome {
    // The native ItemStack constructor turns negative initial damage into zero.
    Outcome::Output {
        stack: RawStack {
            registry: registry.into(),
            meta: meta.max(0),
            amount,
            nbt,
        },
    }
}
pub fn evaluate(
    mode: Mode,
    cells: &[Option<RawStack>],
    core: &BTreeMap<CoreName, String>,
    roles: &BTreeMap<String, Vec<Role>>,
    materials: &Materials,
    material_key: Option<&str>,
) -> Result<Outcome> {
    ensure!(cells.len() == 9, "ProjectBlue requires nine logical cells");
    for stack in cells.iter().flatten() {
        item_id(&stack.registry, stack.meta, stack.nbt.as_ref())?;
    }
    let get = |name| core.get(&name).context("ProjectBlue core item is absent");
    let has = |stack: &RawStack, role| {
        roles
            .get(&stack.registry)
            .is_some_and(|roles| roles.contains(&role))
    };
    let Some(first) = &cells[0] else {
        return Ok(Outcome::Unmatched);
    };
    if mode == Mode::Paint {
        let Some(spray) = &cells[1] else {
            return Ok(Outcome::Unmatched);
        };
        return Ok(
            if (has(first, Role::Lever) || has(first, Role::Button)) && has(spray, Role::Spray) {
                output(
                    &first.registry,
                    native_integer(tag(spray, "color"))?,
                    1,
                    None,
                )
            } else {
                Outcome::Unmatched
            },
        );
    }
    if !has(first, Role::Saw) || cells[1].is_some() || cells[3].is_some() {
        return Ok(Outcome::Unmatched);
    }
    let Some(input) = &cells[4] else {
        return Ok(Outcome::Unmatched);
    };
    if mode == Mode::Panel {
        if &input.registry != get(CoreName::ItemMicroPart)? || input.meta != 257 {
            return Ok(Outcome::Unmatched);
        }
        let material = native_material_key(input, Some("tile.wood"), material_key)?;
        let (name, meta) = materials.resolve(&material);
        return Ok(output(
            get(CoreName::ControlPanelItem)?,
            meta,
            1,
            mat_tag(name),
        ));
    }
    for (role, name, meta) in [
        (Role::FullLever, CoreName::MiniatureLever, 16),
        (Role::StoneButton, CoreName::MiniatureButton, 16),
        (Role::WoodButton, CoreName::MiniatureButton, 17),
    ] {
        if has(input, role) {
            return Ok(output(get(name)?, meta, 8, None));
        }
    }
    if &input.registry == get(CoreName::ItemPartFixture)? {
        return Ok(output(get(CoreName::MiniatureLamp)?, input.meta, 8, None));
    }
    if &input.registry == get(CoreName::ItemMicroPart)? && input.meta == 1 {
        if tag(input, "mat").is_none() {
            return Ok(Outcome::Error {
                phase: "match".into(),
                reason: "materialTagAbsent".into(),
            });
        }
        let (name, meta) = materials.resolve(&native_material_key(input, None, material_key)?);
        return Ok(output(
            get(CoreName::MiniatureCover)?,
            meta,
            8,
            mat_tag(name),
        ));
    }
    Ok(Outcome::Unmatched)
}
fn checked_stack(stack: &Stack, items: &BTreeMap<&str, &Item>) -> Result<RawStack> {
    let item = items
        .get(stack.item.as_str())
        .context("ProjectBlue example item fact absent")?;
    let amount = stack.amount.parse::<i32>()?;
    ensure!(
        amount.to_string() == stack.amount,
        "ProjectBlue amount is not canonical"
    );
    Ok(RawStack {
        registry: item.registry.clone(),
        meta: item.meta,
        amount,
        nbt: item.nbt.clone(),
    })
}
fn key(value: &str) -> Result<()> {
    ensure!(
        value.encode_utf16().count() <= 65536,
        "ProjectBlue material name exceeds budget"
    );
    Ok(())
}
pub(super) fn validate(
    id: &str,
    rows: &[&GeneticReferenceRow],
    items: &BTreeMap<&str, &Item>,
) -> Result<()> {
    let ReferenceData::ProjectBlueDefinition(def) = &rows[0].data else {
        anyhow::bail!("ProjectBlue definition must be first")
    };
    ensure!(
        def.source.handler == "gcewing.projectblue.nei.NEIRecipeHandler"
            && !def.source.owner.is_empty()
            && !def.source.key.is_empty()
            && def.ready
            && !def.consuming
            && def.grid_width == 3
            && def.grid_height == 3
            && def.native_rules <= 262144
            && def.counts.rules == def.native_rules,
        "ProjectBlue source/readiness/grid/count differs"
    );
    let data: Vec<_> = rows.iter().map(|row| &row.data).collect();
    ensure!(
        id == super::value_id(
            "reference",
            &serde_json::json!({"kind":"projectBlueRules","records":data})
        )?,
        "ProjectBlue domain identity differs"
    );
    ensure!(
        def.core.len() == if def.native_rules == 0 { 0 } else { 9 },
        "ProjectBlue core item census differs"
    );
    for registry in def.core.values() {
        item_id(registry, 0, None)?;
    }
    let mut materials = Materials::default();
    let mut roles = BTreeMap::new();
    let mut rules = BTreeMap::new();
    let mut examples = BTreeMap::<u32, u32>::new();
    let mut used_items = BTreeSet::new();
    let mut material_count = 0;
    let mut phase = 0;
    let mut last_cache = None;
    let mut last_registry = None;
    let mut last_alias = None;
    let mut last_item = None;
    for row in &rows[1..] {
        match &row.data {
            ReferenceData::ProjectBlueCachedMaterial(value) => {
                ensure!(
                    phase == 0
                        && value.name.is_some() == value.block.is_some()
                        && value.name.is_some() == value.metadata.is_some(),
                    "ProjectBlue cached material fields/order differ"
                );
                key(&value.key)?;
                if let Some(name) = &value.name {
                    key(name)?;
                    item_id(value.block.as_ref().unwrap(), 0, None)?;
                }
                ensure!(
                    last_cache.as_ref().is_none_or(|prior: &String| prior
                        .encode_utf16()
                        .cmp(value.key.encode_utf16())
                        .is_lt()),
                    "ProjectBlue cache key order differs"
                );
                last_cache = Some(value.key.clone());
                ensure!(
                    materials
                        .cache
                        .insert(value.key.clone(), value.clone())
                        .is_none(),
                    "Repeated ProjectBlue cached material"
                );
                material_count += 1;
            }
            ReferenceData::ProjectBlueRegisteredMaterial(value) => {
                ensure!(phase <= 1, "ProjectBlue material registry order differs");
                phase = 1;
                key(&value.key)?;
                item_id(&value.block, 0, None)?;
                ensure!(
                    value.owner.is_some() == value.source_block.is_some()
                        && (!value.item_block
                            || (value.source_block.as_deref() == Some(value.block.as_str())
                                && value.metadata == value.source_metadata.max(0)))
                        && (value.item_block
                            || (value.block == "minecraft:planks" && value.metadata == 0)),
                    "ProjectBlue native material/fallback differs"
                );
                if let Some(block) = &value.source_block {
                    item_id(block, 0, None)?;
                }
                ensure!(
                    last_registry.as_ref().is_none_or(|prior: &String| prior
                        .encode_utf16()
                        .cmp(value.key.encode_utf16())
                        .is_lt()),
                    "ProjectBlue material key order differs"
                );
                last_registry = Some(value.key.clone());
                ensure!(
                    materials
                        .registry
                        .insert(value.key.clone(), value.clone())
                        .is_none(),
                    "Repeated ProjectBlue registry material"
                );
                material_count += 1;
            }
            ReferenceData::ProjectBlueMaterialAlias(value) => {
                ensure!(phase <= 2, "ProjectBlue alias order differs");
                phase = 2;
                key(&value.key)?;
                key(&value.target)?;
                ensure!(
                    last_alias.as_ref().is_none_or(|prior: &String| prior
                        .encode_utf16()
                        .cmp(value.key.encode_utf16())
                        .is_lt()),
                    "ProjectBlue alias key order differs"
                );
                last_alias = Some(value.key.clone());
                ensure!(
                    materials
                        .aliases
                        .insert(value.key.clone(), value.target.clone())
                        .is_none(),
                    "Repeated ProjectBlue material alias"
                );
                material_count += 1;
            }
            ReferenceData::ProjectBlueItem(value) => {
                ensure!(phase <= 3, "ProjectBlue item role order differs");
                phase = 3;
                item_id(&value.registry, 0, None)?;
                ensure!(
                    !value.roles.is_empty()
                        && value.roles.len() == value.roles.iter().collect::<BTreeSet<_>>().len(),
                    "ProjectBlue roles empty or repeated"
                );
                ensure!(
                    value.max_damage.is_some()
                        == (value.roles.contains(&Role::Saw) || value.roles.contains(&Role::Spray))
                        && value.damageable.is_some() == value.roles.contains(&Role::Saw)
                        && (!value.roles.contains(&Role::Spray) || value.max_damage == Some(256)),
                    "ProjectBlue native container parameters differ"
                );
                ensure!(
                    last_item.as_ref().is_none_or(|prior: &String| prior
                        .encode_utf16()
                        .cmp(value.registry.encode_utf16())
                        .is_lt()),
                    "ProjectBlue item registry order differs"
                );
                last_item = Some(value.registry.clone());
                ensure!(
                    roles
                        .insert(value.registry.clone(), value.roles.clone())
                        .is_none(),
                    "Repeated ProjectBlue item role"
                );
            }
            ReferenceData::ProjectBlueRule(value) => {
                ensure!(
                    phase <= 4
                        && value.index as usize == rules.len()
                        && value.anchored
                        && !value.mirrored,
                    "ProjectBlue native rule order/anchor differs"
                );
                phase = 4;
                let (paint, suffix, policy, amount) = match value.mode {
                    Mode::Panel => (false, "CraftControlPanel", MaterialPolicy::DefaultWood, 1),
                    Mode::Miniature => (
                        false,
                        "CraftMiniatureItem",
                        MaterialPolicy::CoverRequiresMaterial,
                        8,
                    ),
                    Mode::Paint => (true, "PaintControl", MaterialPolicy::NumericSprayColor, 1),
                };
                ensure!(
                    value.recipe_class
                        == format!("gcewing.projectblue.ControlPanelRecipes${suffix}")
                        && value.inputs == if paint { vec![0, 1] } else { vec![0, 4] }
                        && value.required_empty == if paint { vec![] } else { vec![1, 3] }
                        && value.ignored
                            == if paint {
                                vec![2, 3, 4, 5, 6, 7, 8]
                            } else {
                                vec![2, 5, 6, 7, 8]
                            }
                        && value.material_policy == policy
                        && value.output_amount == amount,
                    "ProjectBlue native predicate/output policy differs"
                );
                rules.insert(value.index, value);
            }
            ReferenceData::ProjectBlueExample(value) => {
                ensure!(
                    phase == 4 && value.rule as usize + 1 == rules.len(),
                    "ProjectBlue example is detached from its native rule"
                );
                let rule = rules
                    .get(&value.rule)
                    .context("ProjectBlue example native rule absent")?;
                let cells = value
                    .cells
                    .iter()
                    .map(|stack| {
                        stack
                            .as_ref()
                            .map(|stack| checked_stack(stack, items))
                            .transpose()
                    })
                    .collect::<Result<Vec<_>>>()?;
                let uses_material = rule.mode == Mode::Panel
                    || (rule.mode == Mode::Miniature
                        && cells.get(4).and_then(Option::as_ref).is_some_and(|stack| {
                            def.core.get(&CoreName::ItemMicroPart) == Some(&stack.registry)
                                && stack.meta == 1
                        }));
                ensure!(
                    value.material_key.is_some() == uses_material,
                    "ProjectBlue example material observation differs"
                );
                let expected = evaluate(
                    rule.mode,
                    &cells,
                    &def.core,
                    &roles,
                    &materials,
                    value.material_key.as_deref(),
                )?;
                let output = checked_stack(&value.output, items)?;
                ensure!(
                    expected == Outcome::Output { stack: output },
                    "ProjectBlue example differs from native dynamic result"
                );
                for stack in value
                    .cells
                    .iter()
                    .flatten()
                    .chain(std::iter::once(&value.output))
                {
                    used_items.insert(stack.item.as_str());
                }
                *examples.entry(value.rule).or_default() += 1;
                ensure!(
                    examples[&value.rule] <= 32,
                    "ProjectBlue per-rule example budget exceeded"
                );
            }
            _ => anyhow::bail!("ProjectBlue domain mixes foreign reference rows"),
        }
    }
    ensure!(
        def.counts.materials == material_count
            && def.counts.roles as usize == roles.len()
            && def.native_rules as usize == rules.len()
            && def.counts.examples == examples.values().sum::<u32>()
            && def.counts.items as usize == used_items.len(),
        "ProjectBlue reference census differs"
    );
    ensure!(
        def.native_rules > 0 || rows.len() == 1,
        "Ready-empty ProjectBlue has residual records"
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn java_projectblue_three_rule_results_are_reproduced() {
        let raw: serde_json::Value = serde_json::from_str(include_str!(
            "../../../../contracts/fixtures/native-projectblue-logic-contract.json"
        ))
        .unwrap();
        let core = serde_json::from_value(raw["core"].clone()).unwrap();
        let mut materials = Materials::default();
        for row in raw["materials"]["cache"].as_array().unwrap() {
            let entry: CachedMaterial = serde_json::from_value(row.clone()).unwrap();
            materials.cache.insert(entry.key.clone(), entry);
        }
        for row in raw["materials"]["registry"].as_array().unwrap() {
            let entry: RegisteredMaterial = serde_json::from_value(row.clone()).unwrap();
            materials.registry.insert(entry.key.clone(), entry);
        }
        for row in raw["materials"]["aliases"].as_array().unwrap() {
            materials.aliases.insert(
                row["key"].as_str().unwrap().into(),
                row["target"].as_str().unwrap().into(),
            );
        }
        let roles = raw["items"]
            .as_array()
            .unwrap()
            .iter()
            .map(|row| {
                (
                    row["registry"].as_str().unwrap().into(),
                    serde_json::from_value(row["roles"].clone()).unwrap(),
                )
            })
            .collect();
        for case in raw["cases"].as_array().unwrap() {
            let mode = serde_json::from_value(case["mode"].clone()).unwrap();
            let cells: Vec<Option<RawStack>> =
                serde_json::from_value(case["inputs"].clone()).unwrap();
            let expected: Outcome = serde_json::from_value(case["outcome"].clone()).unwrap();
            assert_eq!(
                evaluate(
                    mode,
                    &cells,
                    &core,
                    &roles,
                    &materials,
                    case["materialKey"].as_str()
                )
                .unwrap(),
                expected,
                "{}",
                case["label"]
            );
        }
    }
}
