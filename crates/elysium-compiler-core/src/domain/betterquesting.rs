//! Pinned BetterQuesting declarations; player progression is not a consuming recipe.

use super::{
    references::{GeneticReferenceRow, ReferenceData},
    Fluid, Item, Origin,
};
use crate::identity::Nbt;
use anyhow::{ensure, Context, Result};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

macro_rules! record {
    ($name:ident, $schema:literal, {$($field:ident:$ty:ty),* $(,)?}) => {
        #[derive(Clone,Debug,Serialize,Deserialize,JsonSchema)]
        #[serde(rename_all="camelCase",deny_unknown_fields)]
        #[schemars(rename=$schema)]
        pub struct $name {$(pub $field:$ty),*}
    };
}
record!(Counts,"QuestDeclarationCounts",{quests:u32,tasks:u32,rewards:u32,items:u32,fluids:u32});
record!(Settings,"QuestWorldSettings",{no_rewards:bool,view_mode:bool,party_enable:bool,fully_sync_quests:bool,edit_mode:bool,hardcore:bool,lives_default:i32,lives_max:i32});
record!(Definition,"QuestsDefinition",{source:Origin,ready:bool,consuming:bool,scope:Scope,registered:u32,registry:Vec<String>,visible:Vec<String>,settings:Settings,counts:Counts});
record!(Text,"QuestDeclaredText",{key:String,raw:String,value:String});
record!(Policy,"QuestDeclaredPolicy",{main:bool,task_logic:Logic,quest_logic:Logic,repeat_ticks:i32,repeat_relative:bool,global:bool,global_share:bool,auto_claim:bool,locked_progress:bool,simultaneous:bool,silent:bool});
record!(Prerequisite,"QuestPrerequisite",{quest_uuid:String,r#type:RequirementType,state:TargetState});
record!(Declaration,"QuestDeclaration",{quest_uuid:String,ordinal:u32,name:Text,description:Text,visibility:Visibility,icon:Option<BigStack>,policy:Policy,prerequisites:Vec<Prerequisite>,tasks:Vec<Task>,rewards:Vec<Reward>});
record!(Task,"QuestTaskDeclaration",{native_id:u32,native_class:String,factory:String,label_key:String,rule:TaskRule});
record!(Reward,"QuestRewardDeclaration",{native_id:u32,native_class:String,factory:String,label_key:String,rule:RewardRule});
record!(BigStack,"QuestBigItemStack",{base:Pattern,amount:i32,ore_key:String,ore_active:bool,candidates:Vec<Pattern>});
record!(Pattern,"QuestItemPattern",{registry:String,meta:i32,nbt:Option<Nbt>,damageable:bool,examples:Vec<String>});
record!(Block,"QuestBlockRequirement",{registry:Option<String>,meta:i32,amount:i32,ore_key:Option<String>,nbt:Option<Nbt>,specimen:Option<Pattern>});
record!(FluidRequirement,"QuestFluidRequirement",{fluid:String,amount:i32});

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(rename = "QuestCaptureScope")]
pub enum Scope {
    NativeCurrentPlayerVisibility,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[schemars(rename = "QuestLogicalOperation")]
pub enum Logic {
    And,
    Nand,
    Or,
    Nor,
    Xor,
    Xnor,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[schemars(rename = "QuestVisibilityRule")]
pub enum Visibility {
    Hidden,
    Secret,
    Unlocked,
    Normal,
    Completed,
    Chain,
    Always,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[schemars(rename = "QuestPrerequisiteType")]
pub enum RequirementType {
    Normal,
    Implicit,
    Hidden,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(rename = "QuestTargetState")]
pub enum TargetState {
    Visible,
    Hidden,
    Missing,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(rename = "QuestRewardGrantPolicy")]
pub enum GrantPolicy {
    BaseStackSplit,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[schemars(rename = "QuestScoreOperation")]
pub enum ScoreOperation {
    Equal,
    LessThan,
    MoreThan,
    LessOrEqual,
    MoreOrEqual,
    Not,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(
    tag = "type",
    rename_all = "camelCase",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
#[schemars(rename = "QuestTaskRule")]
pub enum TaskRule {
    Retrieval {
        optional: bool,
        items: Vec<BigStack>,
        #[serde(rename = "partialMatch")]
        partial_match: bool,
        #[serde(rename = "ignoreNBT")]
        ignore_nbt: bool,
        consume: bool,
        #[serde(rename = "groupDetect")]
        group_detect: bool,
        #[serde(rename = "autoConsume")]
        auto_consume: bool,
    },
    Crafting {
        items: Vec<BigStack>,
        #[serde(rename = "partialMatch")]
        partial_match: bool,
        #[serde(rename = "ignoreNBT")]
        ignore_nbt: bool,
        #[serde(rename = "allowAnvil")]
        allow_anvil: bool,
        #[serde(rename = "allowSmelt")]
        allow_smelt: bool,
        #[serde(rename = "allowCraft")]
        allow_craft: bool,
        #[serde(rename = "allowCraftedFromStatistics")]
        allow_crafted_from_statistics: bool,
    },
    Fluid {
        fluids: Vec<FluidRequirement>,
        #[serde(rename = "ignoreNBT")]
        ignore_nbt: bool,
        consume: bool,
        #[serde(rename = "groupDetect")]
        group_detect: bool,
        #[serde(rename = "autoConsume")]
        auto_consume: bool,
    },
    Meeting {
        entity: String,
        range: i32,
        amount: i32,
        #[serde(rename = "targetNBT")]
        target_nbt: Option<Nbt>,
        #[serde(rename = "ignoreNBT")]
        ignore_nbt: bool,
        subtypes: bool,
    },
    Hunt {
        entity: String,
        required: i32,
        #[serde(rename = "damageType")]
        damage_type: String,
        #[serde(rename = "targetNBT")]
        target_nbt: Option<Nbt>,
        #[serde(rename = "ignoreNBT")]
        ignore_nbt: bool,
        subtypes: bool,
    },
    InteractItem {
        item: Option<BigStack>,
        block: Block,
        required: i32,
        #[serde(rename = "partialMatch")]
        partial_match: bool,
        #[serde(rename = "ignoreNBT")]
        ignore_nbt: bool,
        #[serde(rename = "onInteract")]
        on_interact: bool,
        #[serde(rename = "onHit")]
        on_hit: bool,
    },
    InteractEntity {
        item: Option<BigStack>,
        entity: String,
        #[serde(rename = "targetNBT")]
        target_nbt: Option<Nbt>,
        required: i32,
        #[serde(rename = "ignoreItemNBT")]
        ignore_item_nbt: bool,
        #[serde(rename = "partialItemMatch")]
        partial_item_match: bool,
        #[serde(rename = "entitySubtypes")]
        entity_subtypes: bool,
        #[serde(rename = "ignoreEntityNBT")]
        ignore_entity_nbt: bool,
        #[serde(rename = "onInteract")]
        on_interact: bool,
        #[serde(rename = "onHit")]
        on_hit: bool,
    },
    Checkbox {},
    Location {
        name: String,
        structure: String,
        biome: i32,
        x: i32,
        y: i32,
        z: i32,
        dim: i32,
        range: i32,
        visible: bool,
        #[serde(rename = "hideInfo")]
        hide_info: bool,
        invert: bool,
        #[serde(rename = "taxiCab")]
        taxi_cab: bool,
    },
    Xp {
        levels: bool,
        amount: i32,
        consume: bool,
    },
    BlockBreak {
        blocks: Vec<Block>,
    },
    Scoreboard {
        #[serde(rename = "scoreName")]
        score_name: String,
        #[serde(rename = "scoreDisp")]
        score_disp: String,
        criteria: String,
        target: i32,
        #[serde(rename = "conversionBits")]
        conversion_bits: String,
        suffix: String,
        operation: ScoreOperation,
    },
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(
    tag = "type",
    rename_all = "camelCase",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
#[schemars(rename = "QuestRewardRule")]
pub enum RewardRule {
    Item {
        items: Vec<BigStack>,
        #[serde(rename = "grantPolicy")]
        grant_policy: GrantPolicy,
        #[serde(rename = "ignoreDisabled")]
        ignore_disabled: bool,
    },
    Choice {
        items: Vec<BigStack>,
        #[serde(rename = "selectionCount")]
        selection_count: u32,
        #[serde(rename = "grantPolicy")]
        grant_policy: GrantPolicy,
        #[serde(rename = "ignoreDisabled")]
        ignore_disabled: bool,
    },
    Xp {
        levels: bool,
        amount: i32,
        #[serde(rename = "ignoreDisabled")]
        ignore_disabled: bool,
    },
    Command {
        command: String,
        #[serde(rename = "hideCommand")]
        hide_command: bool,
        #[serde(rename = "viaPlayer")]
        via_player: bool,
        #[serde(rename = "ignoreDisabled")]
        ignore_disabled: bool,
    },
    Scoreboard {
        score: String,
        criteria: String,
        relative: bool,
        value: i32,
        #[serde(rename = "ignoreDisabled")]
        ignore_disabled: bool,
    },
    QuestCompletion {
        #[serde(rename = "questUuid")]
        quest_uuid: Option<String>,
        #[serde(rename = "ignoreDisabled")]
        ignore_disabled: bool,
    },
}

fn uuid(value: &str) -> bool {
    value.len() == 36
        && value.bytes().enumerate().all(|(i, b)| {
            if matches!(i, 8 | 13 | 18 | 23) {
                b == b'-'
            } else {
                b.is_ascii_digit() || (b'a'..=b'f').contains(&b)
            }
        })
}
fn text(value: &str) -> Result<()> {
    ensure!(value.len() <= 524288, "Quest string exceeds budget");
    Ok(())
}
fn nbt(value: Option<&Nbt>) -> Result<()> {
    if let Some(value) = value {
        value.validate()?;
    }
    Ok(())
}

fn pattern(
    value: &Pattern,
    items: &BTreeMap<&str, &Item>,
    seen: &mut BTreeSet<String>,
) -> Result<()> {
    crate::identity::item_id(&value.registry, value.meta, value.nbt.as_ref())?;
    nbt(value.nbt.as_ref())?;
    ensure!(
        value.examples.len() <= 65536,
        "Quest item example budget exceeded"
    );
    for id in &value.examples {
        let item = items
            .get(id.as_str())
            .context("Native quest item example is absent")?;
        ensure!(
            item.registry == value.registry
                && item.nbt == value.nbt
                && if value.meta == 32767 {
                    item.meta >= 0 && item.meta != 32767
                } else {
                    item.meta == value.meta
                },
            "Quest example identity differs from native pattern"
        );
        seen.insert(id.clone());
    }
    ensure!(
        value.meta == 32767 || value.examples.len() == 1,
        "Concrete quest pattern needs its native example"
    );
    Ok(())
}
fn stack(
    value: &BigStack,
    items: &BTreeMap<&str, &Item>,
    seen: &mut BTreeSet<String>,
) -> Result<()> {
    text(&value.ore_key)?;
    pattern(&value.base, items, seen)?;
    ensure!(
        !value.candidates.is_empty() && value.candidates.len() <= 65536,
        "Quest candidate budget differs"
    );
    if value.ore_active {
        ensure!(
            !value.ore_key.is_empty(),
            "Active quest ore has no native key"
        );
    } else {
        ensure!(
            value.candidates.len() == 1
                && serde_json::to_value(&value.candidates[0])?
                    == serde_json::to_value(&value.base)?,
            "Inactive quest ore must use the native base"
        );
    }
    for candidate in &value.candidates {
        pattern(candidate, items, seen)?;
    }
    Ok(())
}
fn block(value: &Block, items: &BTreeMap<&str, &Item>, seen: &mut BTreeSet<String>) -> Result<()> {
    if let Some(registry) = &value.registry {
        text(registry)?;
        ensure!(!registry.is_empty(), "Empty native quest block key");
    }
    if let Some(ore) = &value.ore_key {
        text(ore)?;
    }
    nbt(value.nbt.as_ref())?;
    if let Some(sample) = &value.specimen {
        ensure!(
            value.registry.is_some()
                && sample.meta == if value.meta < 0 { 32767 } else { value.meta },
            "Quest block display metadata differs"
        );
        pattern(sample, items, seen)?;
    }
    Ok(())
}
fn task(
    rule: &TaskRule,
    items: &BTreeMap<&str, &Item>,
    seen: &mut BTreeSet<String>,
    fluids: &mut BTreeSet<String>,
) -> Result<(&'static str, &'static str)> {
    let identity = match rule {
        TaskRule::Retrieval {
            optional,
            items: requirements,
            ..
        } => {
            ensure!(
                requirements.len() <= 65536,
                "Quest requirement budget exceeded"
            );
            for item in requirements {
                stack(item, items, seen)?;
            }
            if *optional {
                ("OptionalRetrieval", "optional_retrieval")
            } else {
                ("Retrieval", "retrieval")
            }
        }
        TaskRule::Crafting {
            items: requirements,
            ..
        } => {
            ensure!(
                requirements.len() <= 65536,
                "Quest crafting requirement budget exceeded"
            );
            for item in requirements {
                stack(item, items, seen)?;
            }
            ("Crafting", "crafting")
        }
        TaskRule::Fluid {
            fluids: requirements,
            ..
        } => {
            ensure!(
                requirements.len() <= 65536,
                "Quest fluid requirement budget exceeded"
            );
            for fluid in requirements {
                ensure!(
                    fluid.fluid.starts_with("fluid_") && fluid.fluid.len() == 70,
                    "Invalid quest fluid identity"
                );
                fluids.insert(fluid.fluid.clone());
            }
            ("Fluid", "fluid")
        }
        TaskRule::Meeting {
            entity, target_nbt, ..
        } => {
            text(entity)?;
            nbt(target_nbt.as_ref())?;
            ("Meeting", "meeting")
        }
        TaskRule::Hunt {
            entity,
            damage_type,
            target_nbt,
            ..
        } => {
            text(entity)?;
            text(damage_type)?;
            nbt(target_nbt.as_ref())?;
            ("Hunt", "hunt")
        }
        TaskRule::InteractItem {
            item,
            block: target,
            ..
        } => {
            if let Some(item) = item {
                stack(item, items, seen)?;
            }
            block(target, items, seen)?;
            ("InteractItem", "interact_item")
        }
        TaskRule::InteractEntity {
            item,
            entity,
            target_nbt,
            ..
        } => {
            if let Some(item) = item {
                stack(item, items, seen)?;
            }
            text(entity)?;
            nbt(target_nbt.as_ref())?;
            ("InteractEntity", "interact_entity")
        }
        TaskRule::Checkbox {} => ("Checkbox", "checkbox"),
        TaskRule::Location {
            name, structure, ..
        } => {
            text(name)?;
            text(structure)?;
            ("Location", "location")
        }
        TaskRule::Xp { .. } => ("XP", "xp"),
        TaskRule::BlockBreak { blocks } => {
            ensure!(blocks.len() <= 65536, "Quest block rule budget exceeded");
            for value in blocks {
                block(value, items, seen)?;
            }
            ("BlockBreak", "block_break")
        }
        TaskRule::Scoreboard {
            score_name,
            score_disp,
            criteria,
            conversion_bits,
            suffix,
            ..
        } => {
            for value in [score_name, score_disp, criteria, suffix] {
                text(value)?;
            }
            ensure!(
                conversion_bits.len() == 8
                    && conversion_bits
                        .bytes()
                        .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b)),
                "Invalid native IEEE float conversion"
            );
            ("Scoreboard", "scoreboard")
        }
    };
    Ok(identity)
}
fn reward(
    rule: &RewardRule,
    items: &BTreeMap<&str, &Item>,
    seen: &mut BTreeSet<String>,
) -> Result<(&'static str, &'static str)> {
    Ok(match rule {
        RewardRule::Item { items: grants, .. } | RewardRule::Choice { items: grants, .. } => {
            ensure!(grants.len() <= 65536, "Quest reward item budget exceeded");
            for item in grants {
                stack(item, items, seen)?;
            }
            if let RewardRule::Choice {
                selection_count, ..
            } = rule
            {
                ensure!(
                    *selection_count == 1,
                    "Native choice reward must select one entry"
                );
                ("Choice", "choice")
            } else {
                ("Item", "item")
            }
        }
        RewardRule::Xp { .. } => ("XP", "xp"),
        RewardRule::Command { command, .. } => {
            text(command)?;
            ("Command", "command")
        }
        RewardRule::Scoreboard {
            score, criteria, ..
        } => {
            text(score)?;
            text(criteria)?;
            ("Scoreboard", "scoreboard")
        }
        RewardRule::QuestCompletion { quest_uuid, .. } => {
            if let Some(id) = quest_uuid {
                ensure!(uuid(id), "Invalid completion reward quest UUID");
            }
            ("QuestCompletion", "questcompletion")
        }
    })
}
fn callback(
    native_class: &str,
    factory: &str,
    label: &str,
    suffix: &str,
    key: &str,
    is_reward: bool,
) -> Result<()> {
    let group = if is_reward {
        "rewards.Reward"
    } else {
        "tasks.Task"
    };
    ensure!(
        native_class == format!("bq_standard.{group}{suffix}")
            && factory == format!("bq_standard:{key}")
            && label
                == format!(
                    "bq_standard.{}.{key}",
                    if is_reward { "reward" } else { "task" }
                ),
        "Quest callback owner/factory/label differs from declared rule"
    );
    Ok(())
}

pub fn validate(
    id: &str,
    rows: &[&GeneticReferenceRow],
    items: &BTreeMap<&str, &Item>,
) -> Result<()> {
    let ReferenceData::QuestsDefinition(def) = &rows[0].data else {
        anyhow::bail!("Missing native quest definition")
    };
    ensure!(
        def.ready
            && !def.consuming
            && !def.source.owner.is_empty()
            && def.source.handler == "bq_standard.integration.nei.QuestRecipeHandler"
            && def.source.key == def.source.handler,
        "Native quest source/readiness differs"
    );
    let data: Vec<_> = rows.iter().map(|r| &r.data).collect();
    ensure!(
        id == super::value_id(
            "reference",
            &serde_json::json!({"kind":"betterQuesting","records":data})
        )?,
        "Native quest domain identity differs"
    );
    let c = &def.counts;
    ensure!(
        def.registered <= 65536
            && c.quests <= def.registered
            && c.tasks <= 262144
            && c.rewards <= 262144
            && c.items <= 262144
            && c.fluids <= 65536
            && rows.len() == c.quests as usize + 1
            && def.registry.len() == def.registered as usize
            && def.visible.len() == c.quests as usize,
        "Incomplete native quest declaration counts"
    );
    let registry: BTreeSet<_> = def.registry.iter().collect();
    ensure!(
        registry.len() == def.registry.len() && registry.iter().all(|id| uuid(id)),
        "Invalid or duplicate native quest registry UUID"
    );
    let mut visible = BTreeSet::new();
    for row in &rows[1..] {
        let ReferenceData::QuestDeclaration(quest) = &row.data else {
            anyhow::bail!("Mixed native quest domain")
        };
        ensure!(
            registry.contains(&quest.quest_uuid) && visible.insert(quest.quest_uuid.as_str()),
            "Quest is duplicate or absent from native registry"
        );
    }
    let mut task_count = 0usize;
    let mut reward_count = 0usize;
    let mut item_ids = BTreeSet::new();
    let mut fluid_ids = BTreeSet::new();
    for (ordinal, row) in rows[1..].iter().enumerate() {
        let ReferenceData::QuestDeclaration(quest) = &row.data else {
            unreachable!()
        };
        ensure!(
            quest.ordinal as usize == ordinal && def.visible[ordinal] == quest.quest_uuid,
            "Native quest declaration order differs"
        );
        for (field, value) in [("name", &quest.name), ("desc", &quest.description)] {
            text(&value.raw)?;
            text(&value.value)?;
            // Native UUID keys are Base64 URL-safe UUID bytes, with padding removed.
            let bytes: Vec<u8> = quest
                .quest_uuid
                .bytes()
                .filter(|b| *b != b'-')
                .collect::<Vec<_>>()
                .chunks(2)
                .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap())
                .collect();
            use base64::Engine;
            let encoded = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(bytes);
            ensure!(
                value.key == format!("betterquesting.quest.{encoded}.{field}"),
                "Native quest translation key differs"
            );
        }
        if let Some(icon) = &quest.icon {
            stack(icon, items, &mut item_ids)?;
        }
        let mut prereqs = BTreeSet::new();
        ensure!(
            quest.prerequisites.len() <= 65536,
            "Quest prerequisite budget exceeded"
        );
        for prereq in &quest.prerequisites {
            ensure!(
                uuid(&prereq.quest_uuid) && prereqs.insert(&prereq.quest_uuid),
                "Invalid/duplicate native prerequisite"
            );
            let state = if visible.contains(prereq.quest_uuid.as_str()) {
                TargetState::Visible
            } else if registry.contains(&prereq.quest_uuid) {
                TargetState::Hidden
            } else {
                TargetState::Missing
            };
            ensure!(
                prereq.state == state,
                "Native quest prerequisite visibility/availability differs"
            );
        }
        ensure!(
            quest.tasks.len() <= 65536 && quest.rewards.len() <= 65536,
            "Quest callback budget exceeded"
        );
        let mut previous = None;
        for entry in &quest.tasks {
            ensure!(
                previous.is_none_or(|id| entry.native_id > id),
                "Quest task IDs are unsorted/duplicated"
            );
            previous = Some(entry.native_id);
            let (suffix, key) = task(&entry.rule, items, &mut item_ids, &mut fluid_ids)?;
            callback(
                &entry.native_class,
                &entry.factory,
                &entry.label_key,
                suffix,
                key,
                false,
            )?;
        }
        previous = None;
        for entry in &quest.rewards {
            ensure!(
                previous.is_none_or(|id| entry.native_id > id),
                "Quest reward IDs are unsorted/duplicated"
            );
            previous = Some(entry.native_id);
            let (suffix, key) = reward(&entry.rule, items, &mut item_ids)?;
            callback(
                &entry.native_class,
                &entry.factory,
                &entry.label_key,
                suffix,
                key,
                true,
            )?;
        }
        task_count += quest.tasks.len();
        reward_count += quest.rewards.len();
    }
    ensure!(
        task_count == c.tasks as usize
            && reward_count == c.rewards as usize
            && item_ids.len() == c.items as usize
            && fluid_ids.len() == c.fluids as usize,
        "Native quest declaration/fact counts differ"
    );
    Ok(())
}

pub fn validate_facts(rows: &[GeneticReferenceRow], fluids: &BTreeMap<&str, &Fluid>) -> Result<()> {
    for row in rows {
        if let ReferenceData::QuestDeclaration(quest) = &row.data {
            for task in &quest.tasks {
                if let TaskRule::Fluid {
                    fluids: requirements,
                    ..
                } = &task.rule
                {
                    for requirement in requirements {
                        ensure!(
                            fluids.contains_key(requirement.fluid.as_str()),
                            "Native quest fluid fact is absent"
                        );
                    }
                }
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn fixture() -> serde_json::Value {
        serde_json::from_str(include_str!(
            "../../../../contracts/fixtures/betterquesting-rules-contract.json"
        ))
        .unwrap()
    }
    fn facts() -> (Vec<Item>, Vec<Fluid>) {
        let raw = fixture();
        let source = crate::source::Source::open(
            &std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../contracts/fixtures/source-semantic-40"),
        )
        .unwrap();
        let domain = super::super::Domain::load(&source).unwrap();
        let items = raw["items"]
            .as_array()
            .unwrap()
            .iter()
            .map(|raw| {
                let mut item = domain.items[0].clone();
                item.id = raw["id"].as_str().unwrap().into();
                item.registry = raw["registry"].as_str().unwrap().into();
                item.meta = raw["meta"].as_i64().unwrap() as i32;
                item.nbt = serde_json::from_value(raw["nbt"].clone()).unwrap();
                item.icon = None;
                item.order = None;
                item.durability = 0;
                item.tools.clear();
                item.tags.clear();
                item.armor = false;
                item.aspects = None;
                item
            })
            .collect();
        let fluids = raw["fluids"]
            .as_array()
            .unwrap()
            .iter()
            .map(|raw| {
                let mut value = raw.clone();
                value
                    .as_object_mut()
                    .unwrap()
                    .insert("name".into(), serde_json::json!(domain.items[0].name));
                value
                    .as_object_mut()
                    .unwrap()
                    .insert("icon".into(), serde_json::Value::Null);
                serde_json::from_value(value).unwrap()
            })
            .collect();
        (items, fluids)
    }
    fn reseal(rows: &mut [GeneticReferenceRow]) {
        let data: Vec<_> = rows.iter().map(|row| &row.data).collect();
        let domain = super::super::value_id(
            "reference",
            &serde_json::json!({"kind":"betterQuesting","records":data}),
        )
        .unwrap();
        for row in rows {
            row.domain = domain.clone();
            row.id = row.content_id().unwrap();
        }
    }
    #[test]
    fn actual_java_quest_declarations_have_a_typed_compiler_contract() {
        let raw: serde_json::Value = serde_json::from_str(include_str!(
            "../../../../contracts/fixtures/betterquesting-rules-contract.json"
        ))
        .unwrap();
        let rows: Vec<super::super::references::GeneticReferenceRow> =
            serde_json::from_value(raw["records"].clone())
                .expect("Native quest records need a typed contract");
        assert_eq!(rows.len(), 2);
    }
    #[test]
    fn native_quest_semantics_reject_wrong_choice_optional_scope_and_missing_facts() {
        let raw = fixture();
        let (items, fluids) = facts();
        let index = items.iter().map(|item| (item.id.as_str(), item)).collect();
        let fluid_index = fluids
            .iter()
            .map(|fluid| (fluid.id.as_str(), fluid))
            .collect();
        let rows: Vec<GeneticReferenceRow> =
            serde_json::from_value(raw["records"].clone()).unwrap();
        super::super::references::validate(&rows, &index).unwrap();
        validate_facts(&rows, &fluid_index).unwrap();
        assert!(super::super::references::validate(&rows, &BTreeMap::new()).is_err());
        assert!(validate_facts(&rows, &BTreeMap::new()).is_err());
        let mut wrong = rows.clone();
        if let ReferenceData::QuestDeclaration(quest) = &mut wrong[1].data {
            if let RewardRule::Choice {
                selection_count, ..
            } = &mut quest.rewards[1].rule
            {
                *selection_count = 2;
            }
        }
        reseal(&mut wrong);
        assert!(super::super::references::validate(&wrong, &index).is_err());
        let mut wrong = rows.clone();
        if let ReferenceData::QuestDeclaration(quest) = &mut wrong[1].data {
            if let TaskRule::Retrieval { optional, .. } = &mut quest.tasks[1].rule {
                *optional = false;
            }
        }
        reseal(&mut wrong);
        assert!(super::super::references::validate(&wrong, &index).is_err());
        let mut wrong = rows.clone();
        if let ReferenceData::QuestDeclaration(quest) = &mut wrong[1].data {
            quest.prerequisites[0].state = TargetState::Visible;
        }
        reseal(&mut wrong);
        assert!(super::super::references::validate(&wrong, &index).is_err());
        let mut extra = raw["records"].clone();
        extra[1]["data"]["tasks"][6]["rule"]["consume"] = serde_json::json!(true);
        assert!(serde_json::from_value::<Vec<GeneticReferenceRow>>(extra).is_err());
        let mut empty = vec![rows[0].clone()];
        if let ReferenceData::QuestsDefinition(def) = &mut empty[0].data {
            def.registry.clear();
            def.visible.clear();
            def.registered = 0;
            def.counts = Counts {
                quests: 0,
                tasks: 0,
                rewards: 0,
                items: 0,
                fluids: 0,
            };
        }
        reseal(&mut empty);
        super::super::references::validate(&empty, &BTreeMap::new()).unwrap();
        if let ReferenceData::QuestsDefinition(def) = &mut empty[0].data {
            def.ready = false;
        }
        reseal(&mut empty);
        assert!(super::super::references::validate(&empty, &BTreeMap::new()).is_err());
    }
    #[test]
    fn complete_quest_domain_requires_category_and_native_fluid_facts() {
        let raw = fixture();
        let (items, fluids) = facts();
        let source = crate::source::Source::open(
            &std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../contracts/fixtures/source-semantic-40"),
        )
        .unwrap();
        let mut domain = super::super::Domain::load(&source).unwrap();
        let rows: Vec<GeneticReferenceRow> =
            serde_json::from_value(raw["records"].clone()).unwrap();
        let ReferenceData::QuestsDefinition(def) = &rows[0].data else {
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
        domain.references = rows;
        for item in items {
            domain.items.retain(|existing| existing.id != item.id);
            domain.items.push(item);
        }
        domain.fluids.extend(fluids);
        domain.validate(&source).unwrap();
        domain.fluids.clear();
        assert!(domain.validate(&source).is_err());
    }
}
