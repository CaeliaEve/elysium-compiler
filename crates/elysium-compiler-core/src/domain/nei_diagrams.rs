//! Native finite diagram contents and query associations, never a pixel/GUI interpreter.
use super::{
    nei_information::{required_nullable, RequiredNullable},
    references::{GeneticReferenceRow, ReferenceData},
    Fluid, Item, Origin,
};
use crate::identity::{fluid_id, item_id, Nbt};
use anyhow::{ensure, Context, Result};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[schemars(rename = "NativeDiagramCounts")]
pub struct Counts {
    pub diagrams: u32,
    pub bindings: u32,
    pub actions: u32,
    pub lookups: u32,
    pub scanned_items: u32,
    pub scanned_fluids: u32,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(rename = "NativeDiagramMatcher")]
pub enum Matcher {
    ExplicitComponent,
    NativeMaterial,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(rename = "NativeDiagramQueryScope")]
pub enum QueryScope {
    CompleteNativeBindings,
    ReadyItemListRegistryFluidsAndDiagramComponents,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "camelCase", deny_unknown_fields)]
#[schemars(rename = "NativeDiagramEmptyPredicate")]
pub enum EmptyPredicate {
    Never,
    SlotInsertionsBelow { minimum: u32 },
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[schemars(rename = "NativeDiagramVisibility")]
pub enum Visibility {
    AlwaysShown,
    AlwaysHidden,
    ShowOnShift,
    HideOnShift,
    ShowOnCtrl,
    HideOnCtrl,
    Disabled,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[schemars(rename = "NativeDiagramSelection")]
pub struct Selection {
    pub show_empty_diagrams: bool,
    pub empty: EmptyPredicate,
    pub default_visibility: Visibility,
    #[serde(deserialize_with = "required_nullable")]
    #[schemars(with = "RequiredNullable<String>")]
    pub configured_visibility: Option<String>,
    pub visibility: Visibility,
}
impl Selection {
    fn validate(&self) -> Result<()> {
        if let EmptyPredicate::SlotInsertionsBelow { minimum } = self.empty {
            ensure!(minimum <= MAX as u32, "Native empty predicate exceeds budget");
        }
        let expected = match &self.configured_visibility {
            None => self.default_visibility.clone(),
            Some(raw) => {
                text(raw)?;
                match raw.as_str() {
                    "true" => Visibility::AlwaysShown,
                    "false" => Visibility::Disabled,
                    value => serde_json::from_value(serde_json::json!(value))
                        .unwrap_or(Visibility::AlwaysShown),
                }
            }
        };
        ensure!(expected == self.visibility, "Native effective visibility differs from its declaration");
        Ok(())
    }
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[schemars(rename = "NativeDiagramDefinition")]
pub struct Definition {
    pub source: Origin,
    pub group: String,
    pub name: String,
    pub description: String,
    pub ignore_nbt: bool,
    pub matcher: Matcher,
    pub query_scope: QueryScope,
    pub selection: Selection,
    pub ready: bool,
    pub counts: Counts,
    pub consuming: bool,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "camelCase", deny_unknown_fields)]
#[schemars(rename = "NativeDiagramComponent")]
pub enum Component {
    /// A concrete canonical item fact, shared by query bindings and original
    /// lookups. Wildcard templates still require the explicit Item variant.
    ItemFact { fact: String },
    Item {
        registry: String,
        meta: i32,
        #[serde(deserialize_with = "required_nullable")]
        #[schemars(with = "RequiredNullable<Nbt>")]
        nbt: Option<Nbt>,
        #[serde(deserialize_with = "required_nullable")]
        #[schemars(with = "RequiredNullable<String>")]
        fact: Option<String>,
    },
    Fluid {
        registry: String,
        #[serde(deserialize_with = "required_nullable")]
        #[schemars(with = "RequiredNullable<Nbt>")]
        nbt: Option<Nbt>,
        fact: String,
    },
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(rename = "NativeDiagramDisplay")]
pub struct Display {
    pub component: Component,
    #[serde(deserialize_with = "required_nullable")]
    #[schemars(with = "RequiredNullable<String>")]
    pub amount: Option<String>,
    #[serde(deserialize_with = "required_nullable")]
    #[schemars(with = "RequiredNullable<String>")]
    pub note: Option<String>,
    pub tooltip: Vec<Vec<TooltipElement>>,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "camelCase", deny_unknown_fields)]
#[schemars(rename = "NativeDiagramTooltipElement")]
pub enum TooltipElement {
    Text { text: String },
    Description { component: Component },
    Display { display: Display },
    Formatting { codes: Vec<String> },
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "camelCase", deny_unknown_fields)]
#[schemars(rename = "NativeDiagramLabel")]
pub enum Label {
    Text { text: String },
    Component { component: Component },
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(rename = "NativeDiagramQuery")]
pub enum Query {
    Crafting,
    Usage,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "camelCase", deny_unknown_fields)]
#[schemars(rename = "NativeDiagramNavigation")]
pub enum Navigation {
    #[serde(rename_all = "camelCase")]
    Group {
        group: String,
        crafting_opens: Query,
        usage_opens: Query,
    },
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(rename = "NativeDiagramAnnotation")]
pub struct Annotation {
    pub label: Label,
    pub tooltip: Vec<Vec<TooltipElement>>,
    #[serde(deserialize_with = "required_nullable")]
    #[schemars(with = "RequiredNullable<Navigation>")]
    pub action: Option<Navigation>,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(rename = "NativeDiagramLogicalGroup")]
pub struct Group {
    pub key: String,
    pub columns: u32,
    pub rows: u32,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(rename = "NativeDiagramCell")]
pub struct Cell {
    pub group: String,
    pub index: u32,
    pub alternatives: Vec<Display>,
    pub tooltip: Vec<Vec<TooltipElement>>,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(rename = "NativeDiagramContent")]
pub struct Content {
    pub groups: Vec<Group>,
    pub cells: Vec<Cell>,
    pub labels: Vec<Label>,
    pub annotations: Vec<Annotation>,
    pub relations: Vec<Relation>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(rename = "NativeDiagramRelationKind")]
pub enum RelationKind {
    CircuitRecipe,
    LensRecipe,
    OreStage,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(rename = "NativeDiagramRelation")]
pub struct Relation {
    pub kind: RelationKind,
    pub from: String,
    pub to: String,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[schemars(rename = "NativeDiagramRecord")]
pub struct Diagram {
    pub source: Origin,
    pub group: String,
    pub diagram: u32,
    pub native_slot_insertions: u32,
    pub content: Content,
    pub consuming: bool,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(rename = "NativeDiagramBinding")]
pub struct Binding {
    pub source: Origin,
    pub group: String,
    pub binding: u32,
    pub query: Query,
    pub component: Component,
    pub diagrams: Vec<u32>,
    pub consuming: bool,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(rename = "NativeDiagramAction")]
pub struct Action {
    pub source: Origin,
    pub group: String,
    pub action: u32,
    pub key: String,
    pub diagrams: Vec<u32>,
    pub consuming: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(rename = "NativeDiagramDamageNormalization")]
pub enum DamageNormalization {
    Preserved,
    ZeroForDamageableItem,
    NotApplicable,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(rename = "NativeDiagramLookup")]
pub struct Lookup {
    pub source: Origin,
    pub group: String,
    pub lookup: u32,
    pub original: Component,
    pub damage: DamageNormalization,
    pub bindings: Vec<u32>,
    pub consuming: bool,
}
impl Lookup {
    pub fn normalized(&self, ignore_nbt: bool, items: &BTreeMap<&str, &Item>) -> Result<Component> {
        let mut value = self.original.expanded(items)?;
        match &mut value {
            Component::ItemFact { .. } => unreachable!(),
            Component::Item {
                registry,
                meta,
                nbt,
                fact,
            } => {
                ensure!(
                    self.damage != DamageNormalization::NotApplicable,
                    "Native item normalization policy missing"
                );
                if self.damage == DamageNormalization::ZeroForDamageableItem {
                    *meta = 0;
                }
                if ignore_nbt {
                    *nbt = None;
                }
                *fact = if *meta == 32767 {
                    None
                } else {
                    Some(item_id(registry, *meta, nbt.as_ref())?)
                };
            }
            Component::Fluid {
                registry,
                nbt,
                fact,
            } => {
                ensure!(
                    self.damage == DamageNormalization::NotApplicable,
                    "Fluid has invented damage normalization"
                );
                if ignore_nbt {
                    *nbt = None;
                }
                *fact = fluid_id(registry, nbt.as_ref())?;
            }
        }
        Ok(value)
    }
}

const MAX: usize = 262144;
fn text(value: &str) -> Result<()> {
    ensure!(
        value.len() <= 65536,
        "Native diagram text exceeds UTF-8 budget"
    );
    Ok(())
}
fn source(source: &Origin, group: &str, consuming: bool, definition: &Definition) -> Result<()> {
    ensure!(
        !consuming
            && group == definition.group
            && serde_json::to_value(source)? == serde_json::to_value(&definition.source)?,
        "Native diagram origin/group/consumption differs"
    );
    Ok(())
}
impl Component {
    pub(crate) fn expanded(&self, items: &BTreeMap<&str, &Item>) -> Result<Self> {
        if let Self::ItemFact { fact } = self {
            let item = items.get(fact.as_str()).context("Compact native diagram item fact absent")?;
            ensure!(item.meta != 32767, "A wildcard cannot become a concrete item fact reference");
            return Ok(Self::Item { registry:item.registry.clone(),meta:item.meta,
                nbt:item.nbt.clone(),fact:Some(fact.clone()) });
        }
        Ok(self.clone())
    }
    pub(crate) fn validate(&self, items: &BTreeMap<&str, &Item>) -> Result<()> {
        match self {
            Self::ItemFact { .. } => self.expanded(items)?.validate(items)?,
            Self::Item {
                registry,
                meta,
                nbt,
                fact,
            } => {
                ensure!(
                    !registry.is_empty() && *meta >= 0,
                    "Invalid native diagram item template"
                );
                let id = item_id(registry, *meta, nbt.as_ref())?;
                if *meta == 32767 {
                    ensure!(
                        fact.is_none(),
                        "Native wildcard has invented concrete appearance"
                    );
                } else {
                    ensure!(
                        fact.as_deref() == Some(&id),
                        "Native item component identity differs"
                    );
                    let item = items
                        .get(id.as_str())
                        .context("Native diagram item fact absent")?;
                    ensure!(
                        item.registry == *registry && item.meta == *meta && item.nbt == *nbt,
                        "Native diagram item specimen differs"
                    );
                }
            }
            Self::Fluid {
                registry,
                nbt,
                fact,
            } => {
                ensure!(
                    !registry.is_empty() && *fact == fluid_id(registry, nbt.as_ref())?,
                    "Native fluid component identity differs"
                );
            }
        }
        Ok(())
    }
    pub(crate) fn fluid(&self, fluids: &BTreeMap<&str, &Fluid>) -> Result<()> {
        if let Self::Fluid {
            registry,
            nbt,
            fact,
        } = self
        {
            let fluid = fluids
                .get(fact.as_str())
                .context("Native diagram fluid fact absent")?;
            ensure!(
                fluid.registry == *registry && fluid.nbt == *nbt,
                "Native diagram fluid specimen differs"
            );
        }
        Ok(())
    }
}
fn displays(
    displays: &[Display],
    visit: &mut impl FnMut(&Component) -> Result<()>,
    depth: u32,
) -> Result<()> {
    ensure!(
        depth <= 16 && displays.len() <= MAX,
        "Native diagram display budget exceeded"
    );
    for d in displays {
        visit(&d.component)?;
        if let Some(amount) = &d.amount {
            let n = amount.parse::<i32>()?;
            ensure!(
                n > 0 && n.to_string() == *amount,
                "Invalid native display quantity"
            );
        }
        if let Some(note) = &d.note {
            text(note)?;
        }
        tooltip(&d.tooltip, visit, depth + 1)?;
    }
    Ok(())
}
fn tooltip(
    lines: &[Vec<TooltipElement>],
    visit: &mut impl FnMut(&Component) -> Result<()>,
    depth: u32,
) -> Result<()> {
    ensure!(
        depth <= 16 && lines.len() <= MAX,
        "Native tooltip recursion/count exceeds budget"
    );
    for line in lines {
        ensure!(
            line.len() <= MAX,
            "Native tooltip element count exceeds budget"
        );
        for element in line {
            match element {
                TooltipElement::Text { text: value } => text(value)?,
                TooltipElement::Description { component } => visit(component)?,
                TooltipElement::Display { display } => {
                    displays(std::slice::from_ref(display), visit, depth + 1)?
                }
                TooltipElement::Formatting { codes } => {
                    let mut seen = BTreeSet::new();
                    for code in codes {
                        ensure!(
                            matches!(
                                code.as_str(),
                                "BLACK"
                                    | "DARK_BLUE"
                                    | "DARK_GREEN"
                                    | "DARK_AQUA"
                                    | "DARK_RED"
                                    | "DARK_PURPLE"
                                    | "GOLD"
                                    | "GRAY"
                                    | "DARK_GRAY"
                                    | "BLUE"
                                    | "GREEN"
                                    | "AQUA"
                                    | "RED"
                                    | "LIGHT_PURPLE"
                                    | "YELLOW"
                                    | "WHITE"
                                    | "OBFUSCATED"
                                    | "BOLD"
                                    | "STRIKETHROUGH"
                                    | "UNDERLINE"
                                    | "ITALIC"
                                    | "RESET"
                            ) && seen.insert(code),
                            "Invalid native warning/formatting code"
                        );
                    }
                }
            }
        }
    }
    Ok(())
}
fn label(value: &Label, visit: &mut impl FnMut(&Component) -> Result<()>) -> Result<()> {
    match value {
        Label::Text { text: value } => text(value),
        Label::Component { component } => visit(component),
    }
}
fn content(
    value: &Content,
    visit: &mut impl FnMut(&Component) -> Result<()>,
    actions: &BTreeSet<&str>,
) -> Result<()> {
    let mut groups = BTreeMap::new();
    let mut expected = Vec::new();
    ensure!(
        value.groups.len() <= MAX
            && value.cells.len() <= MAX
            && value.labels.len() <= MAX
            && value.annotations.len() <= MAX,
        "Native diagram content exceeds count budget"
    );
    for group in &value.groups {
        text(&group.key)?;
        let count = u64::from(group.columns) * u64::from(group.rows);
        ensure!(
            !group.key.is_empty()
                && count > 0
                && count <= MAX as u64
                && groups.insert(&group.key, count).is_none(),
            "Invalid native logical group"
        );
        ensure!(
            expected.len() + count as usize <= MAX,
            "Native diagram cell budget exceeded"
        );
        expected.extend((0..count as u32).map(|i| (&group.key, i)));
    }
    ensure!(
        expected.len() == value.cells.len(),
        "Native logical empty cells were omitted"
    );
    for (cell, (group, index)) in value.cells.iter().zip(expected) {
        ensure!(
            cell.group == *group && cell.index == index,
            "Native logical cell order differs"
        );
        displays(&cell.alternatives, visit, 0)?;
        tooltip(&cell.tooltip, visit, 0)?;
    }
    for value in &value.labels {
        label(value, visit)?;
    }
    for annotation in &value.annotations {
        label(&annotation.label, visit)?;
        tooltip(&annotation.tooltip, visit, 0)?;
        if let Some(Navigation::Group {
            group,
            crafting_opens,
            usage_opens,
        }) = &annotation.action
        {
            ensure!(
                actions.contains(group.as_str())
                    && *crafting_opens == Query::Usage
                    && *usage_opens == Query::Crafting,
                "Unknown/altered native navigation callback"
            );
        }
    }
    Ok(())
}
fn relations(family: &str, content: &Content) -> Vec<Relation> {
    let keys: BTreeSet<_> = content
        .cells
        .iter()
        .filter(|cell| !cell.alternatives.is_empty())
        .map(|cell| cell.group.as_str())
        .collect();
    let mut result = Vec::new();
    let mut add = |from: &str, to: &str, kind: RelationKind| {
        if keys.contains(from) && keys.contains(to) {
            result.push(Relation {
                kind,
                from: from.into(),
                to: to.into(),
            });
        }
    };
    match family {
        "circuits" => {
            for key in &keys {
                if key.starts_with("recipe-item-inputs-") || *key == "recipe-fluid-inputs" {
                    add(key, "recipe-output", RelationKind::CircuitRecipe);
                }
            }
        }
        "lenses" => {
            for key in &keys {
                if let Some(index) = key.strip_prefix("recipe-input-") {
                    add(
                        key,
                        &format!("recipe-output-{index}"),
                        RelationKind::LensRecipe,
                    );
                }
            }
        }
        "oreprocessing" => {
            for (from, to) in [
                ("raw-ore", "raw-ore-macerate"),
                ("true-raw-ore", "true-raw-ore-macerate"),
                ("raw-ore-macerate", "crushed-ore-macerate"),
                ("raw-ore-macerate", "crushed-ore-wash"),
                ("raw-ore-macerate", "crushed-ore-bath-mercury"),
                ("raw-ore-macerate", "crushed-ore-bath-sodium-persulfate"),
                ("crushed-ore-wash", "purified-ore-macerate"),
                ("crushed-ore-wash", "purified-ore-sift"),
                ("crushed-ore-macerate", "impure-dust-centrifuge"),
                ("purified-ore-macerate", "purified-dust-centrifuge"),
                (
                    "purified-ore-macerate",
                    "purified-dust-electromagnetic-separate",
                ),
                ("raw-ore-macerate", "ore-thermal-centrifuge"),
                ("crushed-ore-wash", "ore-thermal-centrifuge"),
                ("raw-ore-macerate", "ore-thermal-centrifuge-crushed"),
                ("crushed-ore-wash", "ore-thermal-centrifuge-purified"),
                ("ore-thermal-centrifuge", "ore-thermal-centrifuge-macerate"),
                (
                    "ore-thermal-centrifuge-crushed",
                    "ore-thermal-centrifuge-macerate-extra",
                ),
                (
                    "ore-thermal-centrifuge-purified",
                    "ore-thermal-centrifuge-macerate-extra",
                ),
            ] {
                add(from, to, RelationKind::OreStage);
            }
        }
        _ => {}
    }
    result
}
pub fn validate(
    id: &str,
    rows: &[&GeneticReferenceRow],
    items: &BTreeMap<&str, &Item>,
) -> Result<()> {
    let ReferenceData::NeiDiagramDefinition(def) = &rows[0].data else {
        anyhow::bail!("Native diagram definition missing")
    };
    let group = def
        .group
        .strip_prefix("neicustomdiagram.diagramgroup.gregtech.")
        .context("Unknown native diagram family")?;
    ensure!(
        matches!(
            group,
            "circuits" | "lenses" | "materialparts" | "oreprocessing" | "materialtools"
        ) && def.ready
            && !def.consuming
            && def.source.key == def.group
            && !def.source.owner.is_empty()
            && matches!(
                def.source.handler.as_str(),
                "com.github.dcysteine.neicustomdiagram.api.diagram.DiagramGroup"
                    | "com.github.dcysteine.neicustomdiagram.api.diagram.CustomDiagramGroup"
            ),
        "Invalid native diagram source/readiness"
    );
    let custom = matches!(group, "materialparts" | "materialtools");
    ensure!(
        custom == (def.matcher == Matcher::NativeMaterial)
            && def.query_scope == QueryScope::ReadyItemListRegistryFluidsAndDiagramComponents,
        "Native matcher semantics differ"
    );
    text(&def.name)?;
    text(&def.description)?;
    def.selection.validate()?;
    let c = &def.counts;
    ensure!(
        [
            c.diagrams,
            c.bindings,
            c.actions,
            c.lookups,
            c.scanned_items,
            c.scanned_fluids
        ]
        .into_iter()
        .all(|n| n <= MAX as u32)
            && c.actions == 1
            && rows.len()
                == 1 + c.diagrams as usize
                    + c.bindings as usize
                    + c.lookups as usize
                    + c.actions as usize,
        "Native diagram counts differ"
    );
    let data: Vec<_> = rows.iter().map(|r| &r.data).collect();
    ensure!(
        id == super::value_id(
            "reference",
            &serde_json::json!({"kind":"neiDiagrams","records":data})
        )? && serde_json::to_vec(&data)?.len() <= 32 * 1024 * 1024,
        "Native diagram identity/budget differs"
    );
    let mut actions = BTreeSet::new();
    let lookup_start = 1 + c.diagrams as usize + c.bindings as usize;
    let action_start = lookup_start + c.lookups as usize;
    for row in &rows[action_start..] {
        let ReferenceData::NeiDiagramAction(action) = &row.data else {
            anyhow::bail!("Native action order differs")
        };
        source(&action.source, &action.group, action.consuming, def)?;
        ensure!(
            action.action == 0 && action.key == def.group && actions.insert(action.key.as_str()),
            "Native all-diagram action differs"
        );
        let mut seen = BTreeSet::new();
        ensure!(
            action
                .diagrams
                .iter()
                .all(|i| *i < c.diagrams && seen.insert(*i)),
            "Native action has duplicate/unknown diagram"
        );
        if def.source.handler.ends_with(".DiagramGroup") {
            ensure!(
                action.diagrams == (0..c.diagrams).collect::<Vec<_>>(),
                "Native all-diagram closure incomplete"
            );
        } else {
            ensure!(
                group == "circuits" && !action.diagrams.is_empty(),
                "Unknown/empty native overview action"
            );
        }
    }
    for (index, row) in rows[1..1 + c.diagrams as usize].iter().enumerate() {
        let ReferenceData::NeiDiagram(diagram) = &row.data else {
            anyhow::bail!("Native diagram order differs")
        };
        source(&diagram.source, &diagram.group, diagram.consuming, def)?;
        ensure!(
            diagram.diagram as usize == index && diagram.native_slot_insertions <= MAX as u32,
            "Native diagram index differs"
        );
        content(
            &diagram.content,
            &mut |component| component.validate(items),
            &actions,
        )?;
        ensure!(
            diagram.content.relations == relations(group, &diagram.content),
            "Native factory process associations omitted/altered"
        );
    }
    let mut seen = BTreeSet::new();
    let mut destinations = BTreeSet::<u32>::new();
    let mut edges = 0;
    for (index, row) in rows[1 + c.diagrams as usize..1 + c.diagrams as usize + c.bindings as usize]
        .iter()
        .enumerate()
    {
        let ReferenceData::NeiDiagramBinding(binding) = &row.data else {
            anyhow::bail!("Native binding order differs")
        };
        source(&binding.source, &binding.group, binding.consuming, def)?;
        ensure!(
            binding.binding as usize == index
                && !binding.diagrams.is_empty()
                && binding.diagrams.len() <= c.diagrams as usize
                && binding.diagrams.iter().all(|target| *target < c.diagrams)
                && binding.diagrams.iter().copied().collect::<BTreeSet<_>>().len() == binding.diagrams.len()
                && seen.insert(serde_json::to_string(&(&binding.query, binding.component.expanded(items)?))?),
            "Duplicate/invalid native query binding"
        );
        binding.component.validate(items)?;
        edges += binding.diagrams.len();
        ensure!(edges <= MAX, "Native diagram destination budget exceeded");
        destinations.extend(binding.diagrams.iter().copied());
    }
    let bindings = &rows[1 + c.diagrams as usize..lookup_start];
    let mut binding_groups = BTreeMap::<String, Vec<u32>>::new();
    for (index, row) in bindings.iter().enumerate() {
        if let ReferenceData::NeiDiagramBinding(binding) = &row.data {
            binding_groups
                .entry(serde_json::to_string(&binding.component.expanded(items)?)?)
                .or_default()
                .push(index as u32);
        }
    }
    let mut queries = BTreeSet::new();
    let mut covered = BTreeSet::new();
    let mut damage_policies = BTreeMap::new();
    for (index, row) in rows[lookup_start..action_start].iter().enumerate() {
        let ReferenceData::NeiDiagramLookup(lookup) = &row.data else {
            anyhow::bail!("Native lookup order differs")
        };
        source(&lookup.source, &lookup.group, lookup.consuming, def)?;
        ensure!(
            lookup.lookup as usize == index
                && !lookup.bindings.is_empty()
                && lookup.bindings.len() <= c.bindings as usize
                && queries.insert(serde_json::to_string(&lookup.original.expanded(items)?)?),
            "Invalid/duplicate original native query"
        );
        lookup.original.validate(items)?;
        if let Component::Item { registry, .. } = lookup.original.expanded(items)? {
            if let Some(prior) = damage_policies.insert(registry, &lookup.damage) {
                ensure!(
                    prior == &lookup.damage,
                    "Native isDamageable policy changed within an item registry"
                );
            }
        }
        let expected = serde_json::to_string(&lookup.normalized(def.ignore_nbt,items)?)?;
        let mut indices = BTreeSet::new();
        for index in &lookup.bindings {
            ensure!(
                *index < c.bindings && indices.insert(*index),
                "Native lookup binding absent/repeated"
            );
            let ReferenceData::NeiDiagramBinding(binding) = &bindings[*index as usize].data else {
                unreachable!()
            };
            ensure!(
                serde_json::to_string(&binding.component.expanded(items)?)? == expected,
                "Original native query normalization/target differs"
            );
            covered.insert(*index);
        }
        ensure!(
            binding_groups.get(&expected) == Some(&lookup.bindings),
            "Original query omitted a native crafting/usage target"
        );
    }
    ensure!(
        covered.len() == c.bindings as usize,
        "Native bindings lost verified original queries"
    );
    for row in &rows[action_start..] {
        if let ReferenceData::NeiDiagramAction(action) = &row.data {
            destinations.extend(action.diagrams.iter().copied());
        }
    }
    ensure!(
        destinations.len() == c.diagrams as usize,
        "Unreachable native diagram"
    );
    Ok(())
}
pub fn validate_facts(rows: &[GeneticReferenceRow], fluids: &BTreeMap<&str, &Fluid>) -> Result<()> {
    for row in rows {
        match &row.data {
            ReferenceData::NeiDiagram(diagram) => {
                let actions = diagram
                    .content
                    .annotations
                    .iter()
                    .filter_map(|a| match &a.action {
                        Some(Navigation::Group { group, .. }) => Some(group.as_str()),
                        _ => None,
                    })
                    .collect();
                content(
                    &diagram.content,
                    &mut |component| component.fluid(fluids),
                    &actions,
                )?;
            }
            ReferenceData::NeiDiagramBinding(binding) => binding.component.fluid(fluids)?,
            ReferenceData::NeiDiagramLookup(lookup) => lookup.original.fluid(fluids)?,
            _ => {}
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::super::{origin_id, Domain};
    use super::*;
    use serde_json::{json, Value};

    #[test]
    fn actual_five_gt_generators_keep_material_queries_and_distinct_overview() {
        let raw: Value = serde_json::from_str(include_str!(
            "../../../../contracts/fixtures/gt-diagram-generators-contract.json"
        )).unwrap();
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../contracts/fixtures/source-semantic-40");
        let source = crate::source::Source::open(&path).unwrap();
        let mut domain = Domain::load(&source).unwrap();
        let base_item = serde_json::to_value(&domain.items[0]).unwrap();
        for item in raw["items"].as_array().unwrap() {
            let mut value = domain.items.iter().find(|old| item["id"].as_str() == Some(old.id.as_str()))
                .map(|old| serde_json::to_value(old).unwrap()).unwrap_or_else(|| base_item.clone());
            let prior_tags = domain.items.iter().find(|old| item["id"].as_str() == Some(old.id.as_str()))
                .map(|old| serde_json::to_value(&old.tags).unwrap()).unwrap_or_else(|| json!([]));
            for key in ["id", "registry", "meta", "nbt", "tags", "durability"] { value[key] = item[key].clone(); }
            // Preserve the shared golden fixture's separate recipe memberships. Native diagram rows are unchanged.
            for tag in prior_tags.as_array().unwrap() {
                if !value["tags"].as_array().unwrap().contains(tag) { value["tags"].as_array_mut().unwrap().push(tag.clone()); }
            }
            let fact: Item = serde_json::from_value(value).unwrap();
            domain.items.retain(|old| old.id != fact.id);
            domain.items.push(fact);
        }
        let base_fluid = serde_json::to_value(&domain.fluids[0]).unwrap();
        for fluid in raw["fluids"].as_array().unwrap() {
            let mut value = base_fluid.clone();
            for key in ["id", "registry", "nbt", "temperature", "density", "viscosity", "luminosity", "gaseous"] { value[key] = fluid[key].clone(); }
            let fact: Fluid = serde_json::from_value(value).unwrap();
            domain.fluids.retain(|old| old.id != fact.id);
            domain.fluids.push(fact);
        }
        domain.references.clear();
        assert_eq!(raw["domains"].as_array().unwrap().len(), 5);
        let mut groups = BTreeSet::new();
        for raw_rows in raw["domains"].as_array().unwrap() {
            let rows: Vec<GeneticReferenceRow> = serde_json::from_value(raw_rows.clone()).unwrap();
            let ReferenceData::NeiDiagramDefinition(def) = &rows[0].data else { panic!() };
            assert!(groups.insert(def.group.clone()));
            assert!(def.counts.diagrams > 0);
            let mut category = domain.categories[0].clone();
            category.source = def.source.clone();
            category.id = origin_id("category", &category.source).unwrap();
            category.reference = Some(rows[0].domain.clone());
            category.program = None; category.icon = None; category.machines.clear();
            domain.categories.push(category);
            domain.references.extend(rows);
        }
        domain.validate(&source).unwrap();
        let circuit = domain.references.iter().find(|row| matches!(&row.data,
            ReferenceData::NeiDiagramAction(action) if action.group.ends_with(".circuits"))).unwrap();
        let ReferenceData::NeiDiagramAction(action) = &circuit.data else { panic!() };
        assert_eq!(action.diagrams.len(), 1);
    }

    #[test]
    fn actual_grouped_query_keeps_full_destination_closure() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../contracts/fixtures");
        let raw: Value = serde_json::from_slice(&std::fs::read(root.join("nei-diagram-grouped-contract.json")).unwrap()).unwrap();
        let source = crate::source::Source::open(&root.join("source-semantic-40")).unwrap();
        let domain = Domain::load(&source).unwrap();
        let base = serde_json::to_value(&domain.items[0]).unwrap();
        let items: Vec<Item> = raw["items"].as_array().unwrap().iter().map(|raw| {
            let mut value = base.clone();
            for key in ["id","registry","meta","nbt","tags"] { value[key] = raw[key].clone(); }
            serde_json::from_value(value).unwrap()
        }).collect();
        let facts = items.iter().map(|item| (item.id.as_str(),item)).collect();
        let rows: Vec<GeneticReferenceRow> = serde_json::from_value(raw["records"].clone()).unwrap();
        let check = |mut rows: Vec<GeneticReferenceRow>| {
            let data: Vec<_> = rows.iter().map(|row| &row.data).collect();
            let id = super::super::value_id("reference", &json!({"kind":"neiDiagrams","records":data})).unwrap();
            for row in &mut rows { row.domain = id.clone(); row.id = row.content_id().unwrap(); }
            super::validate(&id, &rows.iter().collect::<Vec<_>>(), &facts)
        };
        check(rows.clone()).unwrap();
        let mut compact: Value=serde_json::to_value(&rows).unwrap();
        let mut compacted=0;
        for row in compact.as_array_mut().unwrap() {
            let data=&mut row["data"];
            for key in ["component","original"] {
                if data[key]["kind"]=="item" && data[key]["fact"].is_string() {
                    data[key]=json!({"kind":"itemFact","fact":data[key]["fact"]});
                    compacted+=1;
                }
            }
        }
        assert!(compacted>0);
        check(serde_json::from_value(compact).expect("Canonical item references must deserialize")).unwrap();
        let bindings: Vec<_> = rows.iter().filter_map(|row| if let ReferenceData::NeiDiagramBinding(binding) = &row.data {Some(binding)} else {None}).collect();
        assert_eq!(bindings.len(),2); assert!(bindings.iter().all(|binding| binding.diagrams.len()==120));
        for targets in [vec![], vec![0,0], vec![120]] {
            let mut changed = rows.clone();
            if let ReferenceData::NeiDiagramBinding(binding) = &mut changed[121].data { binding.diagrams = targets; } else {panic!()}
            assert!(check(changed).is_err());
        }
        let mut changed = rows.clone();
        if let ReferenceData::NeiDiagramLookup(lookup) = &mut changed[123].data { lookup.bindings.pop(); } else {panic!()}
        assert!(check(changed).is_err());
    }

    #[test]
    fn binding_stores_one_component_for_all_native_destinations() {
        let raw = serde_json::json!({"source":{"owner":"neicustomdiagram","handler":"handler","key":"group"},
            "group":"group","binding":0,"query":"usage","component":{"kind":"item","registry":"minecraft:paper","meta":0,"nbt":null,"fact":null},
            "diagrams":[0,3,9],"consuming":false});
        let binding: Binding = serde_json::from_value(raw.clone()).expect("grouped native destinations");
        assert_eq!(serde_json::to_value(binding).unwrap(), raw);
    }

    #[test]
    fn native_diagram_contract_keeps_empty_cells_nullable_quantities_and_binding_closure() {
        let path =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../contracts/fixtures");
        let raw: Value = serde_json::from_str(include_str!(
            "../../../../contracts/fixtures/nei-diagram-contract.json"
        ))
        .unwrap();
        let source = crate::source::Source::open(&path.join("source-semantic-40")).unwrap();
        let mut domain = Domain::load(&source).unwrap();
        let base = serde_json::to_value(&domain.items[0]).unwrap();
        for item in raw["items"].as_array().unwrap() {
            let mut value = domain
                .items
                .iter()
                .find(|old| old.id == item["id"].as_str().unwrap())
                .map(|old| serde_json::to_value(old).unwrap())
                .unwrap_or_else(|| base.clone());
            for key in ["id", "registry", "meta", "nbt"] {
                value[key] = item[key].clone();
            }
            let fact: Item = serde_json::from_value(value).unwrap();
            domain.items.retain(|old| old.id != fact.id);
            domain.items.push(fact);
        }
        domain.references = serde_json::from_value(raw["records"].clone()).unwrap();
        let base_fluid = serde_json::to_value(&domain.fluids[0]).unwrap();
        for fluid in raw["fluids"].as_array().unwrap() {
            let mut value = domain
                .fluids
                .iter()
                .find(|old| old.id == fluid["id"].as_str().unwrap())
                .map(|old| serde_json::to_value(old).unwrap())
                .unwrap_or_else(|| base_fluid.clone());
            for key in ["id", "registry", "nbt"] {
                value[key] = fluid[key].clone();
            }
            let fact: Fluid = serde_json::from_value(value).unwrap();
            domain.fluids.retain(|old| old.id != fact.id);
            domain.fluids.push(fact);
        }
        let ReferenceData::NeiDiagramDefinition(def) = &domain.references[0].data else {
            panic!()
        };
        let mut category = domain.categories[0].clone();
        category.source = def.source.clone();
        category.id = origin_id("category", &category.source).unwrap();
        category.reference = Some(domain.references[0].domain.clone());
        category.program = None;
        category.icon = None;
        category.machines.clear();
        domain.categories.push(category);
        domain.validate(&source).unwrap();
        assert!(super::validate_facts(&domain.references, &BTreeMap::new()).is_err());
        let mut missing_fluid = domain.clone();
        assert_eq!(raw["fluids"].as_array().unwrap().len(), 1);
        missing_fluid
            .fluids
            .retain(|old| old.id != raw["fluids"][0]["id"].as_str().unwrap());
        assert!(missing_fluid.validate(&source).is_err());
        let reject = |mut rows: Vec<GeneticReferenceRow>| {
            let data: Vec<_> = rows.iter().map(|r| &r.data).collect();
            let id =
                super::super::value_id("reference", &json!({"kind":"neiDiagrams","records":data}))
                    .unwrap();
            for row in &mut rows {
                row.domain = id.clone();
                row.id = row.content_id().unwrap();
            }
            let mut changed = domain.clone();
            changed.references = rows;
            let category = changed.categories.last_mut().unwrap();
            category.reference = Some(id);
            assert!(changed.validate(&source).is_err());
        };
        let mut missing = domain.references.clone();
        let mut selection = domain.references.clone();
        if let ReferenceData::NeiDiagramDefinition(d) = &mut selection[0].data {
            d.selection.configured_visibility = Some("false".into());
        }
        reject(selection);
        let mut insertion = domain.references.clone();
        if let ReferenceData::NeiDiagram(d) = &mut insertion[1].data {
            d.native_slot_insertions = MAX as u32 + 1;
        }
        reject(insertion);
        let mut threshold = domain.references.clone();
        if let ReferenceData::NeiDiagramDefinition(d) = &mut threshold[0].data {
            d.selection.empty = EmptyPredicate::SlotInsertionsBelow { minimum: MAX as u32 + 1 };
        }
        reject(threshold);
        if let ReferenceData::NeiDiagram(d) = &mut missing[1].data {
            d.content.cells.remove(1);
        }
        reject(missing);
        let mut quantity = domain.references.clone();
        if let ReferenceData::NeiDiagram(d) = &mut quantity[1].data {
            d.content.cells[0].alternatives[0].amount = Some("0".into());
        }
        reject(quantity);
        let mut wildcard = domain.references.clone();
        if let ReferenceData::NeiDiagram(d) = &mut wildcard[1].data {
            if let Component::Item { fact, .. } = &mut d.content.cells[2].alternatives[0].component
            {
                *fact = Some("invented".into());
            }
        }
        reject(wildcard);
        let mut binding = domain.references.clone();
        if let ReferenceData::NeiDiagramBinding(b) = &mut binding[2].data {
            b.diagrams = vec![99];
        }
        reject(binding);
        let mut relation = domain.references.clone();
        if let ReferenceData::NeiDiagram(d) = &mut relation[1].data {
            assert_eq!(d.content.relations.len(), 1);
            d.content.relations.clear();
        }
        reject(relation);
        let mut action = domain.references.clone();
        if let ReferenceData::NeiDiagramAction(a) = &mut action.last_mut().unwrap().data {
            a.diagrams.clear();
        }
        reject(action);
        let mut changed_lookup = domain.references.clone();
        if let ReferenceData::NeiDiagramLookup(lookup) = &mut changed_lookup[7].data {
            assert_eq!(lookup.damage, DamageNormalization::ZeroForDamageableItem);
            lookup.damage = DamageNormalization::Preserved;
        }
        reject(changed_lookup);
        let mut missing_target = domain.references.clone();
        if let ReferenceData::NeiDiagramLookup(lookup) = &mut missing_target[6].data {
            lookup.bindings.pop();
        }
        reject(missing_target);
        let mut unready = domain.references.clone();
        if let ReferenceData::NeiDiagramDefinition(d) = &mut unready[0].data {
            d.ready = false;
        }
        reject(unready);
        let mut unknown = raw["records"][1].clone();
        unknown["data"]["content"]["nativeBackground"] = json!("native.png");
        assert!(serde_json::from_value::<GeneticReferenceRow>(unknown).is_err());
        let mut nullable = raw["records"][1].clone();
        nullable["data"]["content"]["cells"][0]["alternatives"][1]
            .as_object_mut()
            .unwrap()
            .remove("amount");
        assert!(serde_json::from_value::<GeneticReferenceRow>(nullable).is_err());
        let schema = schemars::schema_for!(Display);
        assert!(schema
            .schema
            .object
            .as_ref()
            .unwrap()
            .required
            .contains("amount"));
    }
}
