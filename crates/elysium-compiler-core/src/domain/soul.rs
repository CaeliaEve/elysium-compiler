use super::{
    Consumption, Edit, Item, Kind, Match, MatchCase, OutputRole, Process, Quantity, Recipe,
};
use crate::identity::Nbt;
use anyhow::{ensure, Context, Result};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SoulFilter {
    /// Empty native vessel item reference; metadata does not constrain input.
    pub vessel: String,
    /// Sorted unique native mob strings, with explicit null extraction result.
    pub names: Vec<Option<String>>,
    /// Complement for spawner blacklist; includes null in names.
    pub exclude: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SoulSelector {
    pub soul: SoulFilter,
    pub material: MatchCase,
}

impl SoulFilter {
    pub(super) fn validate(&self, items: &BTreeMap<&str, &Item>) -> Result<()> {
        let vessel = items
            .get(self.vessel.as_str())
            .context("missing native soul vessel")?;
        ensure!(
            vessel.meta == 0 && vessel.nbt.is_none(),
            "soul vessel anchor must be empty"
        );
        ensure!(
            self.names.len() <= 65536
                && self.names.windows(2).all(|p| match (&p[0], &p[1]) {
                    (None, Some(_)) => true,
                    (Some(a), Some(b)) => a.encode_utf16().cmp(b.encode_utf16()).is_lt(),
                    _ => false,
                })
                && self.names.iter().flatten().all(|s| s.len() <= 65535),
            "soul names must be bounded, sorted and unique"
        );
        ensure!(
            !self.exclude || self.names.first() == Some(&None),
            "spawner blacklist must reject null extraction"
        );
        Ok(())
    }
    /// Validate canonical examples only. The native rule also accepts non-string
    /// id tags through Minecraft's text coercion; this is not an arbitrary-NBT emulator.
    pub(super) fn sample_name<'a>(
        &self,
        item: &'a Item,
        items: &BTreeMap<&str, &Item>,
    ) -> Result<Option<&'a str>> {
        let vessel = items
            .get(self.vessel.as_str())
            .context("missing native soul vessel")?;
        if item.registry != vessel.registry {
            return Ok(None);
        }
        let Some(Nbt::Compound { value }) = &item.nbt else {
            return Ok(None);
        };
        match value.get("id") {
            None=>Ok(None),
            Some(Nbt::String{value})=>Ok(Some(value)),
            _=>anyhow::bail!("soul examples require canonical string ids; native input coercion remains symbolic"),
        }
    }
    pub(super) fn accepts_sample(
        &self,
        item: &Item,
        items: &BTreeMap<&str, &Item>,
    ) -> Result<bool> {
        let name = self.sample_name(item, items)?;
        Ok(self.names.iter().any(|n| n.as_deref() == name) != self.exclude)
    }
}

pub(super) fn validate(recipe: &Recipe, items: &BTreeMap<&str, &Item>) -> Result<()> {
    let Some(Process::Soul {
        levels,
        experience,
        capacity,
        spawner,
        earlier,
        ..
    }) = &recipe.process
    else {
        unreachable!()
    };
    ensure!(
        recipe.inputs.len() == 2
            && recipe.outputs.len() == 2
            && recipe.duration.is_none()
            && recipe.energy.is_none()
            && recipe.grid.is_none()
            && recipe.magic.is_none()
            && earlier.len() <= 4096
            && (0..=i32::MAX / 20).contains(capacity),
        "invalid Soul Binder shape or capacity"
    );
    let level = f64::from(*levels);
    let xp = if *levels == 0 {
        0
    } else if (1..=15).contains(levels) {
        levels * 17
    } else if (16..=30).contains(levels) {
        (1.5 * level * level - 29.5 * level + 360.0) as i32
    } else {
        (3.5 * level * level - 151.5 * level + 2220.0) as i32
    };
    ensure!(
        *experience == xp,
        "soul XP gate differs from the native level curve"
    );
    let Some(Match::Soul { filter }) = recipe.inputs[0].choices.first().map(|c| &c.rule) else {
        anyhow::bail!("missing soul input predicate")
    };
    filter.validate(items)?;
    ensure!(
        filter.exclude == *spawner,
        "soul allow/deny mode differs from process"
    );
    let material = recipe.inputs[1]
        .choices
        .first()
        .context("missing soul material")?;
    for (index, input) in recipe.inputs.iter().enumerate() {
        ensure!(
            input.kind == Kind::Item && input.slot == index as u32 && !input.choices.is_empty(),
            "invalid Soul Binder input slots"
        );
        for choice in &input.choices {
            ensure!(
                choice.amount == "1"
                    && matches!(choice.consume, Consumption::Consume)
                    && choice.returns.is_empty(),
                "Soul Binder consumes one item from each slot"
            );
            if index == 0 {
                ensure!(
                    serde_json::to_value(&choice.rule)?
                        == serde_json::to_value(&recipe.inputs[0].choices[0].rule)?,
                    "mixed soul predicates"
                );
                let item = items
                    .get(choice.id.as_str())
                    .context("missing soul sample")?;
                ensure!(
                    filter.accepts_sample(item, items)?,
                    "soul sample fails its predicate"
                );
            } else {
                ensure!(
                    input.choices.len() == 1
                        && matches!(choice.rule,Match::Wildcard{meta,nbt:true} if meta==*spawner),
                    "invalid soul material predicate"
                );
            }
        }
    }
    for prior in earlier {
        prior.soul.validate(items)?;
        ensure!(
            prior.soul.vessel == filter.vessel,
            "prior soul selector uses another vessel"
        );
        ensure!(
            matches!(prior.material.rule, Match::Wildcard { nbt: true, .. }),
            "invalid prior Soul material predicate"
        );
        let item = items
            .get(prior.material.id.as_str())
            .context("missing prior soul material")?;
        prior
            .material
            .rule
            .validate_item(item, items, false, false)?;
    }
    let vessel = items[filter.vessel.as_str()];
    for (index, output) in recipe.outputs.iter().enumerate() {
        ensure!(
            output.kind == Kind::Item
                && output.slot == index as u32
                && matches!(output.role, OutputRole::Result)
                && output.chance.numerator == "1"
                && output.chance.denominator == "1",
            "invalid soul output slots or independent chance"
        );
        if index == 0 {
            ensure!(output.id == vessel.id, "soul return is not an empty vessel");
        }
        if *spawner {
            if index == 0 {
                ensure!(
                    output.amount.as_deref() == Some("1")
                        && output.quantity.is_none()
                        && output.change.is_none(),
                    "invalid spawner vessel return"
                );
            } else {
                let change = output
                    .change
                    .as_ref()
                    .context("missing dynamic spawner output")?;
                let Edit::Soul { base } = &change.action else {
                    anyhow::bail!("wrong spawner reconstruction")
                };
                let base = items.get(base.as_str()).context("missing spawner base")?;
                ensure!(
                    base.registry == items[material.id.as_str()].registry
                        && base.registry != vessel.registry
                        && change.input == 0
                        && output.quantity.is_none(),
                    "spawner item differs from native material"
                );
            }
        } else {
            let Some(Quantity::Soul { nominal }) = &output.quantity else {
                anyhow::bail!("fixed Soul outputs must share the completion gate")
            };
            ensure!(index != 0 || nominal == "1", "invalid empty vessel count");
            super::quantity_bounds(recipe, output)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn soul_examples_do_not_erase_gates_or_reconstruct_entity_nbt() {
        let source = crate::source::Source::open(
            &std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../contracts/fixtures/source"),
        )
        .unwrap();
        let domain = super::super::Domain::load(&source).unwrap();
        let items = domain
            .items
            .iter()
            .map(|i| (i.id.as_str(), i))
            .collect::<BTreeMap<_, _>>();
        let recipes = domain
            .recipes
            .iter()
            .filter(|r| matches!(r.process, Some(Process::Soul { .. })))
            .collect::<Vec<_>>();
        assert_eq!(recipes.len(), 2);
        for recipe in recipes {
            validate(recipe, &items).unwrap();
            for output in &recipe.outputs {
                super::super::change::validate(recipe, output, &items).unwrap();
            }
            let mut wrong = recipe.clone();
            if let Some(Process::Soul { experience, .. }) = &mut wrong.process {
                *experience = 16;
            }
            assert!(
                validate(&wrong, &items).is_err(),
                "XP levels were substituted for raw XP"
            );
            wrong = recipe.clone();
            wrong.inputs[1].choices[0].amount = "7".into();
            assert!(
                validate(&wrong, &items).is_err(),
                "example size became consumption"
            );
            let Match::Soul { filter } = &recipe.inputs[0].choices[0].rule else {
                unreachable!()
            };
            let mut unicode = filter.clone();
            unicode.exclude = false;
            unicode.names = vec![Some("\u{10000}".into()), Some("\u{e000}".into())];
            assert!(
                unicode.validate(&items).is_ok(),
                "Java UTF-16 name order rejected by compiler"
            );
            let mut arbitrary = items[recipe.inputs[0].choices[0].id.as_str()].clone();
            if filter.exclude {
                arbitrary.nbt = Some(Nbt::Compound {
                    value: BTreeMap::from([(
                        "id".into(),
                        Nbt::String {
                            value: "Not.Registered".into(),
                        },
                    )]),
                });
                assert!(
                    filter.accepts_sample(&arbitrary, &items).unwrap(),
                    "blacklist complement narrowed to examples"
                );
                arbitrary.nbt = None;
                assert!(!filter.accepts_sample(&arbitrary, &items).unwrap());
                let mut output = recipe.outputs[1].clone();
                output.change.as_mut().unwrap().samples.swap(0, 1);
                assert!(
                    super::super::change::validate(recipe, &output, &items).is_err(),
                    "spawner samples not associated with input mob"
                );
            } else {
                arbitrary.registry = "fixture:unrelated".into();
                assert!(
                    filter.accepts_sample(&arbitrary, &items).unwrap(),
                    "allowed null extraction narrowed to vessel items"
                );
                assert_eq!(
                    super::super::quantity_bounds(recipe, &recipe.outputs[1]).unwrap(),
                    (0, 2)
                );
                wrong = recipe.clone();
                wrong.outputs[0].quantity = None;
                wrong.outputs[0].amount = Some("1".into());
                assert!(
                    validate(&wrong, &items).is_err(),
                    "last-vessel gate lost from empty vial return"
                );
            }
        }
    }
}
