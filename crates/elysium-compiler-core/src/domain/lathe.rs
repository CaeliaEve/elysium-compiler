//! Native IC2 workpiece and tool state graphs; no fixed consuming recipe or NEI GUI.
use super::{
    references::{GeneticReferenceRow, ReferenceData},
    Item, Origin,
};
use crate::identity::Nbt;
use anyhow::{ensure, Context, Result};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet, VecDeque};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[schemars(rename = "LatheCounts")]
pub struct Counts {
    pub samples: u32,
    pub workpieces: u32,
    pub tools: u32,
    pub states: u32,
    pub tool_states: u32,
    pub transitions: u32,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum HardnessGate {
    StrictlyGreater,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum SpaceCheck {
    BeforeStateChange,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum ByproductStage {
    AfterStateChange,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum BufferLeakCast {
    TruncateToInt,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum BufferLeakPhase {
    BeforeIntake,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum IntakeSide {
    Opposite,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum IntakeSource {
    FirstNonzeroAdvertisedSource,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[schemars(rename = "LatheDefinition")]
pub struct Definition {
    pub source: Origin,
    pub ready: bool,
    pub consuming: bool,
    pub api_class: String,
    pub tool_api_class: String,
    pub machine_class: String,
    pub energy_per_cut: u32,
    pub buffer_capacity: u32,
    pub tool_wear: u32,
    pub hardness_gate: HardnessGate,
    pub output_space_check: SpaceCheck,
    pub byproduct: ByproductStage,
    pub minimum_radius: u32,
    pub buffer_leak_divisor: u32,
    pub buffer_leak_bias_bits: String,
    pub buffer_leak_cast: BufferLeakCast,
    pub buffer_leak_phase: BufferLeakPhase,
    pub intake_directions: Vec<String>,
    pub intake_side: IntakeSide,
    pub intake_source: IntakeSource,
    pub counts: Counts,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum SampleRole {
    Workpiece,
    Tool,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[schemars(rename = "LatheSample")]
pub struct Sample {
    pub panel_order: u32,
    pub role: SampleRole,
    pub item: String,
    pub raw_meta: i32,
    pub raw_nbt: Option<Nbt>,
    pub amount: String,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[schemars(rename = "LatheWorkpiece")]
pub struct Workpiece {
    pub workpiece: String,
    pub callback_class: String,
    pub width: i32,
    pub hardness: i32,
    pub initial: Vec<i32>,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(rename = "LatheOutput")]
pub struct Output {
    pub item: String,
    pub amount: String,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum Comparison {
    LessThan,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[schemars(rename = "LatheRoll")]
pub struct Roll {
    pub threshold_bits: String,
    pub tickets: u32,
    pub samples: u32,
    pub comparison: Comparison,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[schemars(rename = "LatheTransition")]
pub struct Transition {
    pub lane: u32,
    pub after: String,
    pub preflight_output: Output,
    pub byproduct: Output,
    pub roll: Roll,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(rename = "LatheState")]
pub struct State {
    pub workpiece: String,
    pub item: String,
    pub width: i32,
    pub hardness: i32,
    pub lanes: Vec<i32>,
    pub transitions: Vec<Transition>,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[schemars(rename = "LatheTool")]
pub struct Tool {
    pub tool: String,
    pub callback_class: String,
    pub damage: i32,
    pub maximum: i32,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(rename = "LatheToolState")]
pub struct ToolState {
    pub tool: String,
    pub item: String,
    pub damage: i32,
    pub maximum: i32,
    pub hardness: i32,
    pub after: Option<String>,
}

pub fn roll_tickets(bits: &str) -> Result<u32> {
    ensure!(
        bits.len() == 8
            && bits
                .bytes()
                .all(|value| value.is_ascii_digit() || (b'a'..=b'f').contains(&value)),
        "Non-canonical native Float threshold bits"
    );
    let chance = f32::from_bits(u32::from_str_radix(bits, 16)?);
    Ok(if chance.is_nan() || chance <= 0.0 {
        0
    } else if chance >= 1.0 {
        1 << 24
    } else {
        ((chance as f64) * (1 << 24) as f64).ceil() as u32
    })
}
fn integer_nbt(value: &Nbt) -> Result<i32> {
    Ok(match value {
        Nbt::Byte { value } | Nbt::Short { value } | Nbt::Int { value } => value.parse()?,
        Nbt::Long { value } => value.parse::<i64>()? as i32,
        Nbt::Float { value } => f32::from_bits(u32::from_str_radix(value, 16)?) as i32,
        Nbt::Double { value } => f64::from_bits(u64::from_str_radix(value, 16)?) as i32,
        _ => 0,
    })
}
fn default_lanes(nbt: Option<&Nbt>, width: i32) -> Result<Vec<i32>> {
    let Some(Nbt::Compound { value }) = nbt else {
        anyhow::bail!("Normalized native blank lacks state NBT")
    };
    let Some(Nbt::Compound { value }) = value.get("state") else {
        anyhow::bail!("Native workpiece state NBT is not a compound")
    };
    (0..5)
        .map(|lane| {
            value
                .get(&format!("l{lane}"))
                .map(integer_nbt)
                .unwrap_or(Ok(width))
        })
        .collect()
}
fn numeric_state(lanes: &[i32]) -> i32 {
    lanes.iter().fold(0, |value, lane| (value << 4) + lane)
}
fn normalize_default(sample: &Sample, width: i32) -> Result<(i32, Option<Nbt>)> {
    let mut root = match &sample.raw_nbt {
        None => BTreeMap::new(),
        Some(Nbt::Compound { value }) => value.clone(),
        _ => anyhow::bail!("Raw ItemStack NBT must be compound"),
    };
    if !root.contains_key("state") {
        let state = (0..5)
            .map(|lane| {
                let value = if sample.raw_meta == 0 {
                    width
                } else if sample.raw_meta == -1 {
                    1
                } else {
                    ((sample.raw_meta >> (4 * (4 - lane))) & 15)
                        .min(width)
                        .max(1)
                };
                (
                    format!("l{lane}"),
                    Nbt::Int {
                        value: value.to_string(),
                    },
                )
            })
            .collect();
        root.insert("state".into(), Nbt::Compound { value: state });
    }
    let nbt = Nbt::Compound { value: root };
    let lanes = default_lanes(Some(&nbt), width)?;
    Ok((numeric_state(&lanes), Some(nbt)))
}
fn shape(width: i32, lanes: &[i32]) -> Result<()> {
    ensure!(
        (1..=64).contains(&width)
            && (1..=16).contains(&lanes.len())
            && lanes.iter().all(|radius| (1..=width).contains(radius)),
        "Invalid native lathe geometry"
    );
    Ok(())
}
fn class(value: &str) -> Result<()> {
    ensure!(
        !value.is_empty() && value.len() <= 1024,
        "Missing/oversized native lathe callback class"
    );
    Ok(())
}

pub fn validate(
    id: &str,
    rows: &[&GeneticReferenceRow],
    items: &BTreeMap<&str, &Item>,
) -> Result<()> {
    let ReferenceData::LatheDefinition(def) = &rows
        .first()
        .context("Missing native lathe definition")?
        .data
    else {
        anyhow::bail!("Native lathe definition must be first")
    };
    ensure!(
        def.ready
            && !def.consuming
            && def.source.handler == "ic2.neiIntegration.core.recipehandler.LatheRecipeHandler"
            && !def.source.owner.is_empty()
            && !def.source.key.is_empty()
            && def.api_class == "ic2.api.item.ILatheItem"
            && def.tool_api_class == "ic2.api.item.ILatheItem$ILatheTool"
            && def.machine_class == "ic2.core.block.machine.tileentity.TileEntityLathe"
            && def.energy_per_cut == 1000
            && def.buffer_capacity == 10000
            && def.tool_wear == 1
            && def.minimum_radius == 1
            && def.buffer_leak_divisor == 100
            && def.buffer_leak_bias_bits == "3f000000"
            && (1..=7).contains(&def.intake_directions.len())
            && def.intake_directions.iter().all(|value| matches!(
                value.as_str(),
                "down" | "up" | "north" | "south" | "west" | "east" | "unknown"
            )),
        "Invalid native lathe policy"
    );
    let counts = &def.counts;
    ensure!(
        [
            counts.samples,
            counts.workpieces,
            counts.tools,
            counts.states,
            counts.tool_states,
            counts.transitions
        ]
        .iter()
        .all(|count| *count <= 262144),
        "Native lathe graph exceeds count budget"
    );
    ensure!(
        rows.len() as u64
            == 1 + [
                counts.samples,
                counts.workpieces,
                counts.tools,
                counts.states,
                counts.tool_states
            ]
            .iter()
            .map(|count| *count as u64)
            .sum::<u64>(),
        "Incomplete native lathe declaration"
    );
    let data: Vec<_> = rows.iter().map(|row| &row.data).collect();
    ensure!(
        id == super::value_id(
            "reference",
            &serde_json::json!({"kind":"latheRules","records":data})
        )?,
        "Native lathe graph identity differs"
    );
    let mut actual = Counts {
        samples: 0,
        workpieces: 0,
        tools: 0,
        states: 0,
        tool_states: 0,
        transitions: 0,
    };
    let mut samples = Vec::new();
    let mut workpieces = BTreeMap::new();
    let mut tools = BTreeMap::new();
    let mut states = BTreeMap::new();
    let mut tool_states = BTreeMap::new();
    let mut panel_order = None;
    for row in &rows[1..] {
        match &row.data {
            ReferenceData::LatheSample(sample) => {
                ensure!(
                    sample.panel_order < 1_000_000
                        && panel_order.is_none_or(|order| sample.panel_order > order),
                    "Repeated/unordered native panel sample"
                );
                panel_order = Some(sample.panel_order);
                crate::identity::integer(&sample.amount, 1, i32::MAX as i64)?;
                if let Some(nbt) = &sample.raw_nbt {
                    nbt.validate()?;
                }
                ensure!(
                    items.contains_key(sample.item.as_str()),
                    "Lathe panel sample fact absent"
                );
                samples.push(sample);
                actual.samples += 1;
            }
            ReferenceData::LatheWorkpiece(work) => {
                class(&work.callback_class)?;
                shape(work.width, &work.initial)?;
                ensure!(
                    items.contains_key(work.workpiece.as_str())
                        && workpieces.insert(work.workpiece.as_str(), work).is_none(),
                    "Missing/duplicate native workpiece"
                );
                actual.workpieces += 1;
            }
            ReferenceData::LatheTool(tool) => {
                class(&tool.callback_class)?;
                ensure!(
                    tool.damage >= 0
                        && (1..=65536).contains(&tool.maximum)
                        && items.contains_key(tool.tool.as_str())
                        && tools.insert(tool.tool.as_str(), tool).is_none(),
                    "Missing/invalid native lathe tool"
                );
                actual.tools += 1;
            }
            ReferenceData::LatheState(state) => {
                shape(state.width, &state.lanes)?;
                ensure!(
                    items.contains_key(state.item.as_str())
                        && states
                            .insert((state.workpiece.as_str(), state.item.as_str()), state)
                            .is_none(),
                    "Missing/duplicate native lathe state"
                );
                actual.states += 1;
                actual.transitions = actual
                    .transitions
                    .checked_add(state.transitions.len() as u32)
                    .context("Lathe transition count overflow")?;
            }
            ReferenceData::LatheToolState(state) => {
                ensure!(
                    state.damage >= 0
                        && (1..=65536).contains(&state.maximum)
                        && items.contains_key(state.item.as_str())
                        && tool_states
                            .insert((state.tool.as_str(), state.item.as_str()), state)
                            .is_none(),
                    "Invalid/missing/duplicate lathe tool state"
                );
                actual.tool_states += 1;
            }
            _ => anyhow::bail!("Mixed native lathe domain"),
        }
    }
    ensure!(
        &actual == counts,
        "Native lathe totals differ from complete state graph"
    );
    let mut sampled_work = BTreeSet::new();
    let mut sampled_tools = BTreeSet::new();
    for sample in samples {
        let item = items[sample.item.as_str()];
        match sample.role {
            SampleRole::Workpiece => {
                let work = workpieces
                    .get(sample.item.as_str())
                    .context("Native panel workpiece declaration absent")?;
                sampled_work.insert(sample.item.as_str());
                if work.callback_class == "ic2.core.item.resources.ItemLatheDefault" {
                    let (meta, nbt) = normalize_default(sample, work.width)?;
                    ensure!(
                        item.meta == meta && item.nbt == nbt,
                        "Native blank metadata/NBT normalization differs from raw panel sample"
                    );
                }
            }
            SampleRole::Tool => {
                let tool = tools
                    .get(sample.item.as_str())
                    .context("Native panel tool declaration absent")?;
                sampled_tools.insert(sample.item.as_str());
                if tool.callback_class == "ic2.core.item.tool.ItemLathingTool" {
                    ensure!(
                        item.meta == sample.raw_meta && item.nbt == sample.raw_nbt,
                        "Native tool changed raw panel metadata/NBT"
                    );
                }
            }
        }
    }
    ensure!(
        sampled_work.len() == workpieces.len() && sampled_tools.len() == tools.len(),
        "Native lathe declarations lack panel provenance"
    );
    let mut grouped_states = BTreeMap::<&str, usize>::new();
    let output = |value: &Output| -> Result<()> {
        ensure!(
            items.contains_key(value.item.as_str()),
            "Native lathe byproduct fact absent"
        );
        crate::identity::integer(&value.amount, 1, i32::MAX as i64)?;
        Ok(())
    };
    for ((work_id, item_id), state) in &states {
        let work = workpieces
            .get(work_id)
            .context("Native lathe state owner absent")?;
        let initial = items[work.workpiece.as_str()];
        let item = items[item_id];
        ensure!(
            state.width == work.width
                && state.lanes.len() == work.initial.len()
                && item.registry == initial.registry,
            "Native state changed workpiece geometry/registry"
        );
        *grouped_states.entry(work_id).or_default() += 1;
        if work.callback_class == "ic2.core.item.resources.ItemLatheDefault" {
            ensure!(
                state.lanes.len() == 5
                    && (state.width == 3 && state.hardness == 0
                        || state.width == 5 && state.hardness == 2),
                "Native default workpiece material policy differs"
            );
            ensure!(
                item.meta == numeric_state(&state.lanes)
                    && default_lanes(item.nbt.as_ref(), state.width)? == state.lanes,
                "Native state geometry differs from item metadata/NBT"
            );
        }
        let mut lanes = BTreeSet::new();
        for edge in &state.transitions {
            ensure!(
                (edge.lane as usize) < state.lanes.len()
                    && state.lanes[edge.lane as usize] > 1
                    && lanes.insert(edge.lane),
                "Repeated/illegal native lathe cutting lane"
            );
            let next = states
                .get(&(*work_id, edge.after.as_str()))
                .context("Native cut target state absent")?;
            let mut expected = state.lanes.clone();
            expected[edge.lane as usize] -= 1;
            ensure!(
                next.lanes == expected,
                "Native cut is not an exact single-lane decrement"
            );
            if work.callback_class == "ic2.core.item.resources.ItemLatheDefault" {
                let mut expected_nbt = item.nbt.clone();
                let Some(Nbt::Compound { value }) = &mut expected_nbt else {
                    anyhow::bail!("Native blank lacks state NBT")
                };
                let Some(Nbt::Compound { value }) = value.get_mut("state") else {
                    anyhow::bail!("Native blank state type differs")
                };
                value.insert(
                    format!("l{}", edge.lane),
                    Nbt::Int {
                        value: expected[edge.lane as usize].to_string(),
                    },
                );
                ensure!(
                    items[edge.after.as_str()].nbt == expected_nbt,
                    "Native cut modified unrelated workpiece NBT"
                );
            }
            output(&edge.preflight_output)?;
            output(&edge.byproduct)?;
            ensure!(
                edge.roll.samples == 1 << 24
                    && edge.roll.tickets == roll_tickets(&edge.roll.threshold_bits)?,
                "Native nextFloat threshold and tickets disagree"
            );
            if work.callback_class == "ic2.core.item.resources.ItemLatheDefault" {
                ensure!(
                    edge.preflight_output.item == edge.byproduct.item
                        && edge.preflight_output.amount == edge.byproduct.amount
                        && edge.roll.threshold_bits
                            == if state.width == 3 {
                                "3dcccccd"
                            } else {
                                "3f000000"
                            },
                    "Native default byproduct or probability changed"
                );
                if state.width == 3 {
                    let drop = items[edge.byproduct.item.as_str()];
                    ensure!(
                        drop.registry == "minecraft:stick"
                            && drop.meta == 0
                            && drop.nbt.is_none()
                            && edge.byproduct.amount == "1",
                        "Native WOOD byproduct differs"
                    );
                }
            }
        }
        ensure!(
            lanes.len() == state.lanes.iter().filter(|radius| **radius > 1).count(),
            "Native state omitted a cutting lane"
        );
    }
    for (work_id, work) in &workpieces {
        let initial = states
            .get(&(*work_id, *work_id))
            .context("Initial native workpiece state absent")?;
        ensure!(
            initial.lanes == work.initial && initial.hardness == work.hardness,
            "Native initial state differs"
        );
        let mut seen = BTreeSet::new();
        let mut pending = VecDeque::from([*work_id]);
        while let Some(item) = pending.pop_front() {
            if seen.insert(item) {
                ensure!(
                    seen.len() <= 32768,
                    "Native workpiece graph exceeds state budget"
                );
                for edge in &states[&(*work_id, item)].transitions {
                    pending.push_back(edge.after.as_str());
                }
            }
        }
        ensure!(
            seen.len()
                == *grouped_states
                    .get(work_id)
                    .context("Workpiece state group absent")?,
            "Native graph contains unreachable/orphan workpiece states"
        );
    }
    let mut grouped_tools = BTreeMap::<&str, usize>::new();
    for ((tool_id, item_id), state) in &tool_states {
        let tool = tools
            .get(tool_id)
            .context("Native tool state owner absent")?;
        let item = items[item_id];
        let initial = items[tool.tool.as_str()];
        ensure!(
            state.maximum == tool.maximum
                && item.registry == initial.registry
                && state.damage < i32::MAX,
            "Native tool bounds/registry changed"
        );
        *grouped_tools.entry(tool_id).or_default() += 1;
        if tool.callback_class == "ic2.core.item.tool.ItemLathingTool" {
            ensure!(
                state.damage == item.meta && state.hardness == 3,
                "Native lathing tool metadata/hardness differs"
            );
        }
        if state.damage + 1 >= state.maximum {
            ensure!(state.after.is_none(), "Native exhausted tool survived");
        } else {
            let id = state
                .after
                .as_deref()
                .context("Native usable tool was removed")?;
            let next = tool_states
                .get(&(*tool_id, id))
                .context("Native next tool state absent")?;
            ensure!(
                next.damage == state.damage + 1,
                "Native tool wear is not one custom damage unit"
            );
            if tool.callback_class == "ic2.core.item.tool.ItemLathingTool" {
                ensure!(
                    items[id].nbt == item.nbt,
                    "Native tool wear modified unrelated NBT"
                );
            }
        }
    }
    for (tool_id, tool) in &tools {
        let first = tool_states
            .get(&(*tool_id, *tool_id))
            .context("Initial native tool state absent")?;
        ensure!(
            first.damage == tool.damage,
            "Native initial tool damage differs"
        );
        let mut seen = BTreeSet::new();
        let mut next = Some(*tool_id);
        while let Some(id) = next {
            ensure!(
                seen.insert(id) && seen.len() <= 262144,
                "Native tool state cycle/budget exceeded"
            );
            next = tool_states[&(*tool_id, id)].after.as_deref();
        }
        ensure!(
            seen.len()
                == *grouped_tools
                    .get(tool_id)
                    .context("Native tool state group absent")?,
            "Native graph contains orphan tool states"
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn java_lathe_state_closure_crosses_full_domain_and_rejects_drift() {
        let fixtures =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../contracts/fixtures");
        let raw: serde_json::Value = serde_json::from_slice(
            &std::fs::read(fixtures.join("lathe-reference-contract.json")).unwrap(),
        )
        .unwrap();
        let rows: Vec<GeneticReferenceRow> =
            serde_json::from_value(raw["records"].clone()).unwrap();
        let source = crate::source::Source::open(&fixtures.join("source-semantic-40")).unwrap();
        let mut domain = super::super::Domain::load(&source).unwrap();
        let base = serde_json::to_value(&domain.items[0]).unwrap();
        for item in raw["items"].as_array().unwrap() {
            if domain
                .items
                .iter()
                .any(|existing| existing.id == item["id"].as_str().unwrap())
            {
                continue;
            }
            let mut value = base.clone();
            for key in ["id", "registry", "meta", "nbt"] {
                value[key] = item[key].clone();
            }
            value["icon"] = serde_json::Value::Null;
            value["aspects"] = serde_json::Value::Null;
            value["order"] = serde_json::Value::Null;
            value["tags"] = serde_json::json!([]);
            domain.items.push(serde_json::from_value(value).unwrap());
        }
        let ReferenceData::LatheDefinition(def) = &rows[0].data else {
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
        domain.references = rows.clone();
        domain.validate(&source).unwrap();
        let items = domain
            .items
            .iter()
            .map(|item| (item.id.as_str(), item))
            .collect();
        let check = |mut altered: Vec<GeneticReferenceRow>| {
            let data: Vec<_> = altered.iter().map(|row| &row.data).collect();
            let id = super::super::value_id(
                "reference",
                &serde_json::json!({"kind":"latheRules","records":data}),
            )
            .unwrap();
            for row in &mut altered {
                row.domain = id.clone();
                row.id = row.content_id().unwrap();
            }
            super::super::references::validate(&altered, &items)
        };
        let mut missing = rows.clone();
        missing.pop();
        assert!(check(missing).is_err());
        let mut probability = rows.clone();
        if let ReferenceData::LatheState(state) = &mut probability[5].data {
            state.transitions[0].roll.tickets = 1677721;
        }
        assert!(check(probability).is_err());
        let mut wrong = rows.clone();
        if let ReferenceData::LatheState(state) = &mut wrong[5].data {
            state.transitions[0].after = state.item.clone();
        }
        assert!(check(wrong).is_err());
        let mut tool = rows.clone();
        if let ReferenceData::LatheToolState(state) = &mut tool.last_mut().unwrap().data {
            state.after = Some(state.item.clone());
        }
        assert!(check(tool).is_err());
        let mut raw_nbt = rows.clone();
        if let ReferenceData::LatheSample(sample) = &mut raw_nbt[1].data {
            sample.raw_meta = 0x22222;
        }
        assert!(check(raw_nbt).is_err());
        // A correctly re-identified child with unrelated NBT inserted must still be rejected.
        let ReferenceData::LatheState(first) = &rows[5].data else {
            panic!()
        };
        let old = first.transitions[0].after.clone();
        let mut changed_item = (*items[old.as_str()]).clone();
        if let Some(Nbt::Compound { value }) = &mut changed_item.nbt {
            value.insert(
                "owner".into(),
                Nbt::String {
                    value: "unexpected-injection".into(),
                },
            );
        }
        changed_item.id = crate::identity::item_id(
            &changed_item.registry,
            changed_item.meta,
            changed_item.nbt.as_ref(),
        )
        .unwrap();
        let mut changed_rows = rows.clone();
        for row in &mut changed_rows {
            if let ReferenceData::LatheState(state) = &mut row.data {
                if state.item == old {
                    state.item = changed_item.id.clone();
                }
                for edge in &mut state.transitions {
                    if edge.after == old {
                        edge.after = changed_item.id.clone();
                    }
                }
            }
        }
        let mut changed_facts = items.clone();
        changed_facts.insert(changed_item.id.as_str(), &changed_item);
        let borrowed: Vec<_> = changed_rows.iter().collect();
        let changed_id=super::super::value_id("reference",&serde_json::json!({"kind":"latheRules","records":changed_rows.iter().map(|row|&row.data).collect::<Vec<_>>()})).unwrap();
        assert!(validate(&changed_id, &borrowed, &changed_facts).is_err());
        assert_eq!(roll_tickets("3dcccccd").unwrap(), 1677722);
        assert_eq!(roll_tickets("7fc00000").unwrap(), 0);
        assert_eq!(roll_tickets("7f800000").unwrap(), 1 << 24);
        assert!(roll_tickets("3DCCCCCD").is_err());
    }
}
