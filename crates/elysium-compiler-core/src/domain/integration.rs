//! BuildCraft integration's owned native sample semantics and correlated recipe validation.
//! Display observations are examples, not an enumeration of the full input domain.
use super::{change::number, matching::native_tag};
use crate::identity::Nbt;
use anyhow::{bail, ensure, Context, Result};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct IntegrationStack {
    pub registry: String,
    pub meta: i32,
    pub amount: i32,
    pub nbt: Option<Nbt>,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Chip {
    pub registry: String,
    pub meta: i32,
    pub amount: i32,
    pub subtypes: bool,
    pub nbt: Option<Nbt>,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct GateExpansion {
    pub id: String,
    pub chip: Chip,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "lowercase", deny_unknown_fields)]
pub enum IntegrationRule {
    Gate {
        energy: i32,
        maximum: i32,
        primary: Vec<String>,
        red: Chip,
        expansions: Vec<GateExpansion>,
    },
    Facade {
        energy: i32,
        maximum: i32,
        primary: Vec<String>,
        wires: Vec<String>,
        facade: String,
        wire: String,
        plug: String,
        blocks: BTreeMap<String, String>,
    },
    Robot {
        energy: i32,
        maximum: i32,
        primary: Vec<String>,
        robot: String,
        empty: String,
        boards: BTreeMap<String, String>,
    },
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct IntegrationObservation {
    pub input: IntegrationStack,
    pub expansions: Vec<Option<IntegrationStack>>,
    pub output: Option<IntegrationStack>,
}
impl IntegrationRule {
    /// Replays independent physical slots on owned state. Native arbitrary string
    /// coercion remains symbolic: non-string registry/board names need native evidence,
    /// rather than a guessed Java NBT-to-text conversion. No Java callbacks execute.
    pub fn observe(
        &self,
        input: &IntegrationStack,
        expansions: &[Option<IntegrationStack>],
        preview: bool,
    ) -> Result<IntegrationObservation> {
        self.validate()?;
        self.observe_validated(input, expansions, preview)
    }
    fn observe_validated(
        &self,
        input: &IntegrationStack,
        expansions: &[Option<IntegrationStack>],
        preview: bool,
    ) -> Result<IntegrationObservation> {
        ensure!(
            expansions.len() <= 8,
            "integration exceeds eight expansion slots"
        );
        for stack in std::iter::once(input).chain(expansions.iter().flatten()) {
            stack.validate()?;
        }
        let primary = match self {
            Self::Gate { primary, .. }
            | Self::Facade { primary, .. }
            | Self::Robot { primary, .. } => primary,
        };
        ensure!(
            primary.contains(&input.registry),
            "invalid integration primary item"
        );
        let mut result = IntegrationObservation {
            input: input.clone(),
            expansions: expansions.to_vec(),
            output: None,
        };
        result.expansions.resize(8, None);
        if !result.expansions.iter().any(Option::is_some) {
            return Ok(result);
        }
        result.output = match self {
            Self::Gate {
                red, expansions, ..
            } => gate(
                &result.input,
                &mut result.expansions,
                red,
                expansions,
                preview,
            )?,
            Self::Facade {
                primary,
                wires,
                facade,
                wire,
                blocks,
                ..
            } => facade_output(
                &mut result.input,
                &mut result.expansions,
                primary,
                wires,
                facade,
                wire,
                blocks,
                preview,
            )?,
            Self::Robot {
                robot,
                empty,
                boards,
                ..
            } => {
                let expansion = result
                    .expansions
                    .iter_mut()
                    .flatten()
                    .next()
                    .context("missing board")?;
                if !preview {
                    expansion.amount -= 1;
                }
                let tags = expansion.tags_mut()?;
                tags.entry("id".into()).or_insert_with(|| text_tag(empty));
                let lookup = text(tags.get("id"))?;
                let board = boards.get(&lookup).unwrap_or(empty);
                let tags = result.input.tags_mut()?;
                if !tags.contains_key("board") {
                    tags.insert("id".into(), text_tag(empty));
                }
                let energy = number(tags.get("energy"))?;
                Some(IntegrationStack {
                    registry: robot.clone(),
                    meta: 0,
                    amount: 1,
                    nbt: Some(Nbt::Compound {
                        value: BTreeMap::from([
                            (
                                "board".into(),
                                Nbt::Compound {
                                    value: BTreeMap::from([("id".into(), text_tag(board))]),
                                },
                            ),
                            (
                                "energy".into(),
                                Nbt::Int {
                                    value: if energy == 0 { 20000 } else { energy }.to_string(),
                                },
                            ),
                        ]),
                    }),
                })
            }
        };
        Ok(result)
    }
    pub fn validate(&self) -> Result<()> {
        let primary = match self {
            Self::Gate { primary, .. }
            | Self::Facade { primary, .. }
            | Self::Robot { primary, .. } => primary,
        };
        names(primary)?;
        match self {
            Self::Gate {
                red, expansions, ..
            } => {
                ensure!(expansions.len() <= 4096, "too many gate expansions");
                red.validate()?;
                for expansion in expansions {
                    name(&expansion.id)?;
                    expansion.chip.validate()?;
                }
            }
            Self::Facade {
                wires,
                facade,
                wire,
                plug,
                blocks,
                ..
            } => {
                names(wires)?;
                name(facade)?;
                name(wire)?;
                name(plug)?;
                ensure!(
                    primary.contains(facade) && wires.contains(wire),
                    "facade item family differs from global items"
                );
                ensure!(
                    blocks.len() <= 65536 && blocks.get("0").is_some_and(|b| b == "minecraft:air"),
                    "missing defaulted native block registry"
                );
                let mut unique = BTreeSet::new();
                for (id, block) in blocks {
                    let n = id.parse::<u16>()?;
                    ensure!(
                        id == &n.to_string() && unique.insert(block),
                        "invalid block registry"
                    );
                    name(block)?;
                }
            }
            Self::Robot {
                robot,
                empty,
                boards,
                ..
            } => {
                name(robot)?;
                name(empty)?;
                ensure!(
                    boards.len() <= 4096 && primary.contains(robot),
                    "invalid robot registry"
                );
                for (key, id) in boards {
                    name(key)?;
                    name(id)?;
                }
            }
        }
        Ok(())
    }
}
pub(super) fn validate_recipe(recipe: &super::Recipe, rule: &IntegrationRule) -> Result<()> {
    use super::{Consumption, Edit, Kind, Match, OutputRole};
    rule.validate()?;
    ensure!(
        recipe.duration.is_none()
            && recipe.energy.is_none()
            && recipe.grid.is_none()
            && recipe.magic.is_none(),
        "integration has a native energy gate, not fixed duration or EU"
    );
    ensure!(
        (2..=9).contains(&recipe.inputs.len())
            && recipe.inputs[0].slot == 0
            && recipe.inputs.windows(2).all(|p| p[0].slot < p[1].slot),
        "integration inputs must use ordered native slots"
    );
    for input in &recipe.inputs {
        ensure!(
            input.kind == Kind::Item
                && input.slot <= 8
                && !input.choices.is_empty()
                && input.choices.len() <= 65536,
            "invalid integration input slot"
        );
        for choice in &input.choices {
            ensure!(
                choice.amount == "1"
                    && matches!(choice.consume, Consumption::Allocated)
                    && matches!(choice.rule, Match::Integration)
                    && choice.returns.is_empty(),
                "integration choices are one-item examples governed by the shared process"
            );
        }
    }
    ensure!(
        recipe.outputs.len() == 1,
        "integration requires one transformed output"
    );
    let output = &recipe.outputs[0];
    ensure!(
        output.kind == Kind::Item
            && output.slot == 0
            && matches!(output.role, OutputRole::Result)
            && output.quantity.is_none()
            && output.chance.numerator == "1"
            && output.chance.denominator == "1",
        "invalid integration output"
    );
    let change = output
        .change
        .as_ref()
        .context("integration needs a transformed result")?;
    ensure!(
        change.input == 0
            && matches!(change.action, Edit::Integration)
            && change.bindings.is_some(),
        "integration needs correlated input bindings"
    );
    Ok(())
}

pub(super) fn validate_change(
    recipe: &super::Recipe,
    output: &super::Output,
    items: &BTreeMap<&str, &super::Item>,
) -> Result<()> {
    use super::Process;
    let Some(Process::BuildcraftIntegration { rule }) = &recipe.process else {
        bail!("integration output requires its native process")
    };
    validate_recipe(recipe, rule)?;
    let change = output
        .change
        .as_ref()
        .context("missing integration output change")?;
    let bindings = change
        .bindings
        .as_ref()
        .context("missing integration tuples")?;
    ensure!(
        !bindings.is_empty() && bindings.len() <= 65536 && bindings.len() == change.samples.len(),
        "integration tuples differ from output samples"
    );
    // Resolve each factual sample once; validate the block map once per rule.
    let choices = recipe
        .inputs
        .iter()
        .map(|input| {
            input
                .choices
                .iter()
                .map(|choice| {
                    let item = items
                        .get(choice.id.as_str())
                        .context("missing integration input fact")?;
                    Ok(IntegrationStack {
                        registry: item.registry.clone(),
                        meta: item.meta,
                        amount: 1,
                        nbt: item.nbt.clone(),
                    })
                })
                .collect::<Result<Vec<_>>>()
        })
        .collect::<Result<Vec<_>>>()?;
    let mut covered = vec![BTreeSet::new(); recipe.inputs.len()];
    for (binding, sample) in bindings.iter().zip(&change.samples) {
        ensure!(
            binding.len() == recipe.inputs.len() && binding[0].is_some(),
            "integration tuple must bind the primary and every sample slot"
        );
        let mut offered = vec![None; 9];
        for (index, selection) in binding.iter().enumerate() {
            if let Some(selection) = selection {
                let stack = choices[index]
                    .get(*selection as usize)
                    .context("integration binding refers to a missing choice")?;
                covered[index].insert(*selection as usize);
                offered[recipe.inputs[index].slot as usize] = Some(stack.clone());
            }
        }
        let input = offered[0].as_ref().context("missing integration primary")?;
        let preview = rule.observe_validated(input, &offered[1..], true)?;
        let completed = rule.observe_validated(input, &offered[1..], false)?;
        let expected = completed
            .output
            .context("integration tuple has no native output")?;
        ensure!(
            preview.output.as_ref() == Some(&expected),
            "integration preview differs from its completion result"
        );
        let id =
            crate::identity::item_id(&expected.registry, expected.meta, expected.nbt.as_ref())?;
        ensure!(
            sample.id == id
                && sample.amount == expected.amount.to_string()
                && items.contains_key(sample.id.as_str()),
            "integration sample differs from the complete input tuple"
        );
    }
    ensure!(
        covered
            .iter()
            .zip(&choices)
            .all(|(seen, choices)| seen.len() == choices.len()),
        "integration includes an input choice with no correlated sample"
    );
    let first = &change.samples[0];
    ensure!(
        output.id == first.id && output.amount.as_deref() == Some(first.amount.as_str()),
        "default integration output differs from its first tuple"
    );
    Ok(())
}
impl IntegrationStack {
    fn validate(&self) -> Result<()> {
        name(&self.registry)?;
        ensure!(self.amount > 0, "invalid integration input amount");
        if let Some(nbt) = &self.nbt {
            nbt.validate()?;
            ensure!(
                matches!(nbt, Nbt::Compound { .. }),
                "integration root must be a compound"
            );
        }
        Ok(())
    }
    fn tags_mut(&mut self) -> Result<&mut BTreeMap<String, Nbt>> {
        let nbt = self.nbt.get_or_insert_with(|| Nbt::Compound {
            value: BTreeMap::new(),
        });
        let Nbt::Compound { value } = nbt else {
            bail!("integration root must be a compound")
        };
        Ok(value)
    }
}
impl Chip {
    fn validate(&self) -> Result<()> {
        name(&self.registry)?;
        if let Some(nbt) = &self.nbt {
            nbt.validate()?;
            ensure!(matches!(nbt, Nbt::Compound { .. }), "invalid chipset root");
        }
        Ok(())
    }
    fn matches(&self, stack: &IntegrationStack) -> bool {
        self.registry == stack.registry
            && (matches!(self.meta, -1 | 32767)
                || matches!(stack.meta, -1 | 32767)
                || (!self.subtypes || self.meta == stack.meta)
                    && match (&self.nbt, &stack.nbt) {
                        (None, None) => true,
                        (Some(a), Some(b)) => native_tag(a, b, true),
                        _ => false,
                    })
    }
}
fn gate(
    input: &IntegrationStack,
    slots: &mut [Option<IntegrationStack>],
    red: &Chip,
    expansions: &[GateExpansion],
    preview: bool,
) -> Result<Option<IntegrationStack>> {
    let mut output = input.clone();
    output.amount = 1;
    let mut changes = 0;
    for stack in slots.iter_mut().flatten() {
        if red.matches(stack) {
            let tags = output.tags_mut()?;
            // Invalid ordinal -> AND; only native ordinal one denotes OR.
            let logic = number(tags.get("logic"))? as i8;
            tags.insert("logic".into(), byte_tag(if logic == 1 { 0 } else { 1 }));
            changes += 1;
            continue;
        }
        for expansion in expansions {
            if !expansion.chip.matches(stack) {
                continue;
            }
            let tags = output.tags_mut()?;
            let (mut element, mut values) = list(tags.get("ex"), "string");
            if values
                .iter()
                .any(|value| matches!(value,Nbt::String{value} if value==&expansion.id))
            {
                continue;
            }
            if !preview {
                stack.amount -= 1;
            }
            if element == "end" {
                element = "string".into();
            }
            // MC getTagList returns a typed empty list even when its element type
            // differs. appendTag then refuses the string; native craft still counts it.
            if element == "string" {
                values.push(text_tag(&expansion.id));
            }
            tags.insert(
                "ex".into(),
                Nbt::List {
                    element,
                    value: values,
                },
            );
            changes += 1;
            break;
        }
    }
    Ok((changes > 0).then_some(output))
}
#[allow(clippy::too_many_arguments)]
fn facade_output(
    input: &mut IntegrationStack,
    slots: &mut [Option<IntegrationStack>],
    facades: &[String],
    wires: &[String],
    base: &str,
    pipe_wire: &str,
    blocks: &BTreeMap<String, String>,
    preview: bool,
) -> Result<Option<IntegrationStack>> {
    let mut selected_wire = None;
    let mut selected_facade = None;
    for (index, stack) in slots.iter_mut().enumerate() {
        let Some(stack) = stack else { continue };
        if selected_wire.is_none() && wires.contains(&stack.registry) {
            selected_wire = Some(wire(stack.meta));
            if !preview {
                stack.amount -= 1;
            }
        } else if selected_facade.is_none()
            && (facades.contains(&stack.registry) || stack.registry == pipe_wire)
        {
            selected_facade = Some(index);
            if !preview {
                stack.amount -= 1;
            }
        }
    }
    let (Some(wire), Some(index)) = (selected_wire, selected_facade) else {
        return Ok(None);
    };
    let mut states = facade_states(input, base, blocks)?;
    let mut added = facade_states(
        slots[index].as_mut().context("missing selected facade")?,
        base,
        blocks,
    )?
    .into_iter()
    .next()
    .context("native facade selection has no state")?;
    added.wire = Some(wire);
    added.transparent = false;
    if let Some(index) = states.iter().position(|state| state.wire == Some(wire)) {
        states[index] = added;
    } else {
        states.push(added);
    }
    Ok(Some(facade_stack(base, &states)))
}
#[derive(Clone)]
struct FacadeState {
    block: Option<String>,
    meta: i8,
    wire: Option<i8>,
    transparent: bool,
    hollow: bool,
}
fn facade_states(
    stack: &mut IntegrationStack,
    base: &str,
    blocks: &BTreeMap<String, String>,
) -> Result<Vec<FacadeState>> {
    if stack.nbt.is_none() {
        return Ok(Vec::new());
    }
    let alt_meta = stack.meta & 15;
    let tags = stack.tags_mut()?;
    let block = if tags.contains_key("id") {
        Some(
            blocks
                .get(&number(tags.get("id"))?.to_string())
                .cloned()
                .unwrap_or_else(|| "minecraft:air".into()),
        )
    } else if tags.contains_key("name") {
        Some(block_name(&text(tags.get("name"))?, blocks))
    } else {
        None
    };
    if let Some(block) = block {
        let mut migrated = vec![FacadeState {
            block: Some(block),
            meta: number(tags.get("meta"))? as i8,
            wire: None,
            transparent: false,
            hollow: false,
        }];
        if tags.contains_key("name_alt") && tags.contains_key("wire") {
            migrated.push(FacadeState {
                block: Some(block_name(&text(tags.get("name_alt"))?, blocks)),
                meta: if tags.contains_key("meta_alt") {
                    number(tags.get("meta_alt"))?
                } else {
                    alt_meta
                } as i8,
                wire: Some(wire(number(tags.get("wire"))?)),
                transparent: false,
                hollow: false,
            });
        }
        // Migration writes fresh tags back to the offered stack even in preview.
        stack.nbt = facade_stack(base, &migrated).nbt;
    }
    let (_, values) = list(stack.tags_mut()?.get("states"), "compound");
    values
        .iter()
        .map(|tag| {
            let Nbt::Compound { value: tags } = tag else {
                bail!("invalid facade state")
            };
            Ok(FacadeState {
                block: if tags.contains_key("block") {
                    Some(block_name(&text(tags.get("block"))?, blocks))
                } else {
                    None
                },
                meta: number(tags.get("metadata"))? as i8,
                wire: if tags.contains_key("wire") {
                    Some(wire((number(tags.get("wire"))? as i8).into()))
                } else {
                    None
                },
                transparent: number(tags.get("transparent"))? as i8 != 0,
                hollow: number(tags.get("hollow"))? as i8 != 0,
            })
        })
        .collect()
}
fn facade_stack(base: &str, states: &[FacadeState]) -> IntegrationStack {
    let values = states
        .iter()
        .map(|state| {
            let mut tags = BTreeMap::from([
                ("metadata".into(), byte_tag(state.meta)),
                ("transparent".into(), byte_tag(state.transparent.into())),
                ("hollow".into(), byte_tag(state.hollow.into())),
            ]);
            if let Some(block) = &state.block {
                tags.insert("block".into(), text_tag(block));
            }
            if let Some(wire) = state.wire {
                tags.insert("wire".into(), byte_tag(wire));
            }
            Nbt::Compound { value: tags }
        })
        .collect();
    IntegrationStack {
        registry: base.into(),
        meta: 0,
        amount: 1,
        nbt: Some(Nbt::Compound {
            value: BTreeMap::from([
                (
                    "type".into(),
                    byte_tag(i8::from(!(states.len() == 1 && states[0].wire.is_none()))),
                ),
                (
                    "states".into(),
                    Nbt::List {
                        element: "compound".into(),
                        value: values,
                    },
                ),
            ]),
        }),
    }
}
fn wire(value: i32) -> i8 {
    if (0..4).contains(&value) {
        value as i8
    } else {
        0
    }
}
fn block_name(value: &str, blocks: &BTreeMap<String, String>) -> String {
    // RegistryNamespaced accepts an implicit minecraft namespace.
    let value = if value.contains(':') {
        value.into()
    } else {
        format!("minecraft:{value}")
    };
    if blocks.values().any(|name| name == &value) {
        value
    } else {
        "minecraft:air".into()
    }
}
fn list(tag: Option<&Nbt>, kind: &str) -> (String, Vec<Nbt>) {
    match tag {
        Some(Nbt::List { element, value }) if value.is_empty() || element == kind => {
            (element.clone(), value.clone())
        }
        _ => ("end".into(), Vec::new()),
    }
}
fn text(tag: Option<&Nbt>) -> Result<String> {
    match tag {
        None => Ok(String::new()),
        Some(Nbt::String { value }) => Ok(value.clone()),
        _ => bail!(
            "integration examples require canonical string names; native coercion remains symbolic"
        ),
    }
}
fn text_tag(value: &str) -> Nbt {
    Nbt::String {
        value: value.into(),
    }
}
fn byte_tag(value: i8) -> Nbt {
    Nbt::Byte {
        value: value.to_string(),
    }
}
fn name(value: &str) -> Result<()> {
    ensure!(
        !value.is_empty() && value.len() <= 65535,
        "invalid integration name"
    );
    Ok(())
}
fn names(values: &[String]) -> Result<()> {
    ensure!(
        !values.is_empty() && values.len() <= 65536,
        "invalid integration item family"
    );
    let mut seen = BTreeSet::new();
    for value in values {
        name(value)?;
        ensure!(seen.insert(value), "duplicate integration item");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[derive(Deserialize)]
    struct Cases {
        cases: Vec<Case>,
    }
    #[derive(Deserialize)]
    struct Case {
        rule: IntegrationRule,
        input: IntegrationStack,
        expansions: Vec<Option<IntegrationStack>>,
        preview: bool,
        after: Option<IntegrationObservation>,
        error: Option<String>,
    }
    #[test]
    fn integration_contract_requires_complete_correlated_tuples() {
        use crate::{
            domain::{Domain, Recipe},
            identity::item_id,
            source::Source,
        };
        use serde_json::json;
        let source = Source::open(
            &std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../contracts/fixtures/source"),
        )
        .unwrap();
        let domain = Domain::load(&source).unwrap();
        let make = |registry: &str, nbt: serde_json::Value| {
            let mut item = domain.items[0].clone();
            item.registry = registry.into();
            item.meta = 0;
            item.nbt = serde_json::from_value(nbt).unwrap();
            item.id = item_id(registry, 0, item.nbt.as_ref()).unwrap();
            item
        };
        let input = make(
            "fixture:robot",
            json!({"type":"compound","value":{"energy":{"type":"int","value":"12"}}}),
        );
        let red = make(
            "fixture:board",
            json!({"type":"compound","value":{"id":{"type":"string","value":"red"}}}),
        );
        let blue = make(
            "fixture:board",
            json!({"type":"compound","value":{"id":{"type":"string","value":"blue"}}}),
        );
        let product = |id: &str| json!({"type":"compound","value":{"energy":{"type":"int","value":"12"},"board":{"type":"compound","value":{"id":{"type":"string","value":id}}}}});
        let red_product = make("fixture:robot", product("red"));
        let blue_product = make("fixture:robot", product("blue"));
        let all = [
            input.clone(),
            red.clone(),
            blue.clone(),
            red_product.clone(),
            blue_product.clone(),
        ];
        let items = all.iter().map(|i| (i.id.as_str(), i)).collect();
        let choice = |id: &str| json!({"id":id,"amount":"1","rule":{"kind":"integration"},"consume":{"kind":"allocated"},"returns":[]});
        let mut value = json!({"id":"recipe_test","source":{"owner":"fixture","handler":"integration","key":"robot"},"category":"category_test","order":0,"duration":null,"energy":null,"grid":null,"magic":null,"view":null,"properties":{},
            "process":{"kind":"buildcraftIntegration","rule":{"kind":"robot","energy":50000,"maximum":1,"primary":["fixture:robot"],"robot":"fixture:robot","empty":"empty","boards":{"red":"red","blue":"blue"}}},
            "inputs":[{"kind":"item","slot":0,"choices":[choice(&input.id)]},{"kind":"item","slot":1,"choices":[choice(&red.id),choice(&blue.id)]},{"kind":"item","slot":8,"choices":[choice(&blue.id)]}],
            "outputs":[{"kind":"item","slot":0,"id":red_product.id,"amount":"1","quantity":null,"role":"result","chance":{"numerator":"1","denominator":"1"},
                "change":{"input":0,"action":{"kind":"integration"},"bindings":[[0,0,null],[0,1,null],[0,null,0]],"samples":[{"id":red_product.id,"amount":"1"},{"id":blue_product.id,"amount":"1"},{"id":blue_product.id,"amount":"1"}]}}]});
        let check = |v: serde_json::Value| -> Result<()> {
            let r: Recipe = serde_json::from_value(v)?;
            super::super::process::validate(&r, &Default::default(), &items, &[])?;
            super::super::change::validate(&r, &r.outputs[0], &items)
        };
        check(value.clone()).unwrap();
        for (path, bad) in [
            ("/outputs/0/change/bindings", json!(null)),
            ("/outputs/0/change/bindings/0", json!([0, 0])),
            ("/outputs/0/change/bindings/0/0", json!(null)),
            ("/outputs/0/change/bindings/0/1", json!(5)),
            ("/outputs/0/change/samples/1/id", json!(red_product.id)),
            ("/outputs/0/change/action", json!({"kind":"runic"})),
            ("/inputs/1/choices/0/rule", json!({"kind":"exact"})),
            ("/inputs/1/choices/0/consume", json!({"kind":"consume"})),
            ("/inputs/1/choices/0/amount", json!("3")),
            ("/duration", json!("16")),
            ("/energy", json!("50000")),
            ("/process", json!(null)),
        ] {
            let mut bad_value = value.clone();
            *bad_value.pointer_mut(path).unwrap() = bad;
            assert!(check(bad_value).is_err(), "{path}");
        }
        value["outputs"][0]["change"]["bindings"][0] = json!([0, null, null]);
        assert!(check(value).is_err());
    }
    #[test]
    fn replay_pinned_native_integration_observations() {
        let evidence: Cases = serde_json::from_str(include_str!(
            "../../../../contracts/fixtures/integration-observations.json"
        ))
        .unwrap();
        let mut checked = 0;
        for (index, case) in evidence.cases.into_iter().enumerate() {
            // Registry replacement is verified at the mutable Java capture boundary.
            if case.error.as_deref() == Some("recipe_changed") {
                continue;
            }
            let result = case
                .rule
                .observe(&case.input, &case.expansions, case.preview);
            match case.after {
                Some(expected) => assert_eq!(
                    result.unwrap_or_else(|e| panic!("case {index}: {e}")),
                    expected,
                    "case {index}"
                ),
                None => assert!(
                    result.is_err(),
                    "native failed case {index} unexpectedly passed"
                ),
            }
            checked += 1;
        }
        assert_eq!(checked, 23);
        // These are actual production-adapter records, including its last partial
        // facade batch. Recompute their correlated outputs using the same validator
        // used by source compilation, independently of Java's native craft calls.
        let records: serde_json::Value = serde_json::from_str(include_str!(
            "../../../../contracts/fixtures/integration-adapter-records.json"
        ))
        .unwrap();
        let source = crate::source::Source::open(
            &std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../contracts/fixtures/source"),
        )
        .unwrap();
        let domain = crate::domain::Domain::load(&source).unwrap();
        let facts: Vec<_> = records["items"]
            .as_array()
            .unwrap()
            .iter()
            .map(|value| {
                let stack: IntegrationStack = serde_json::from_value(value.clone()).unwrap();
                let mut item = domain.items[0].clone();
                item.registry = stack.registry;
                item.meta = stack.meta;
                item.nbt = stack.nbt;
                item.id =
                    crate::identity::item_id(&item.registry, item.meta, item.nbt.as_ref()).unwrap();
                item
            })
            .collect();
        let items = facts.iter().map(|item| (item.id.as_str(), item)).collect();
        let recipes = records["recipes"].as_array().unwrap();
        assert_eq!(recipes.len(), 4);
        for value in recipes {
            let recipe: crate::domain::Recipe = serde_json::from_value(value.clone()).unwrap();
            super::super::process::validate(&recipe, &Default::default(), &items, &[]).unwrap();
            super::super::change::validate(&recipe, &recipe.outputs[0], &items).unwrap();
        }
    }
}
