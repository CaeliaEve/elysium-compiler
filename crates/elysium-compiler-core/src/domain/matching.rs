use super::{change, Item, Match};
use crate::identity::Nbt;
use anyhow::{ensure, Context, Result};
use std::collections::{BTreeMap, BTreeSet};

impl Match {
    pub(super) fn validate_item(
        &self,
        item: &Item,
        items: &BTreeMap<&str, &Item>,
        member: bool,
        compound: bool,
    ) -> Result<()> {
        match self {
            Self::Exact => {}
            Self::Ae => {
                ensure!(
                    compound,
                    "AE precise matching cannot be nested in priority filters"
                );
                ensure!(
                    ae_nbt(item.nbt.as_ref(), item.nbt.as_ref()),
                    "AE NaN predicate depends on object identity"
                );
            }
            Self::Infusion { template, ores } => {
                ensure!(
                    compound,
                    "infusion predicate cannot be nested in priority filters"
                );
                let anchor = items
                    .get(template.as_str())
                    .context("missing infusion template")?;
                ensure!(
                    anchor.nbt.is_none() && ores == &anchor.tags,
                    "invalid infusion ore predicate"
                );
                ensure!(
                    item.registry == anchor.registry && item.meta == anchor.meta
                        || ores.iter().any(|ore| item.tags.contains(ore)),
                    "invalid native infusion sample"
                );
            }
            Self::Member { root, analyzed } => {
                ensure!(member, "member matching requires whole-stack analysis");
                change::member(item, root, *analyzed)?;
            }
            Self::Ore { name, exclusive } => ensure!(
                item.tags.contains(name) && (!exclusive || item.tags.len() == 1),
                "ore alternative lacks membership: {} in {name}",
                item.id
            ),
            Self::Wildcard { meta, nbt } => {
                ensure!(*meta || *nbt, "wildcard must ignore metadata or NBT")
            }
            Self::WithoutTags { keys } => {
                ensure!(
                    !keys.is_empty()
                        && keys.len() <= 4096
                        && keys.windows(2).all(|pair| pair[0] < pair[1])
                        && keys.iter().all(|key| key.len() <= 65535),
                    "removed tag keys must be bounded, sorted and unique"
                );
                if let Some(Nbt::Compound { value }) = &item.nbt {
                    ensure!(
                        !value.is_empty() && keys.iter().all(|key| !value.contains_key(key)),
                        "removed tag keys leave an unreachable reference NBT"
                    );
                }
            }
            Self::Tags {
                keys,
                present,
                absent,
                ..
            } => {
                let tags = compound_tags(item);
                let mut used = BTreeSet::new();
                for list in [keys, present, absent] {
                    ensure!(
                        list.len() <= 4096 && list.windows(2).all(|pair| pair[0] < pair[1]),
                        "tag constraints must be sorted and unique"
                    );
                    for key in list {
                        ensure!(
                            key.len() <= 65535 && used.insert(key),
                            "overlapping tag constraints"
                        );
                    }
                }
                ensure!(
                    keys.iter()
                        .chain(present)
                        .all(|key| tags.is_some_and(|tags| tags.contains_key(key)))
                        && absent
                            .iter()
                            .all(|key| tags.is_none_or(|tags| !tags.contains_key(key))),
                    "tag constraint rejects its representative item"
                );
            }
            Self::Except { base, exclude } => {
                ensure!(compound, "nested match exclusions are not permitted");
                ensure!(
                    !exclude.is_empty() && exclude.len() <= 4096,
                    "match exclusions must be nonempty and bounded"
                );
                base.validate_item(item, items, false, false)?;
                let mut unique = BTreeSet::new();
                for prior in exclude {
                    ensure!(
                        unique.insert(serde_json::to_string(prior)?),
                        "duplicate match exclusion"
                    );
                    let anchor = items
                        .get(prior.id.as_str())
                        .context("missing match exclusion item")?;
                    prior.rule.validate_item(anchor, items, false, false)?;
                }
                ensure!(
                    !matches!(base.as_ref(), Self::Exact)
                        || !exclude.iter().any(|prior| simple(
                            &prior.rule,
                            item,
                            items[prior.id.as_str()]
                        )),
                    "match exclusions reject the entire exact input"
                );
                // An excluded anchor is still a valid template for a nonempty
                // broad rule. It must not be promoted to an accepted example.
            }
        }
        Ok(())
    }

    /// Reverse links index examples, not arbitrary dynamic NBT. Do not add a
    /// positive use edge for an example rejected by its own priority filter.
    /// Called only after Domain::validate; other existing rule indexing is unchanged.
    pub(crate) fn indexes_example(&self, item: &Item, items: &BTreeMap<&str, &Item>) -> bool {
        match self {
            Self::Except { base, exclude } => {
                simple(base, item, item)
                    && !exclude
                        .iter()
                        .any(|prior| simple(&prior.rule, item, items[prior.id.as_str()]))
            }
            _ => true,
        }
    }
}

fn compound_tags(item: &Item) -> Option<&BTreeMap<String, Nbt>> {
    match &item.nbt {
        Some(Nbt::Compound { value }) => Some(value),
        _ => None,
    }
}

/// Primitive, validated predicates. Neither an expression interpreter nor a
/// game simulator: these are the same finite identity operations as the contract.
fn simple(rule: &Match, offered: &Item, anchor: &Item) -> bool {
    if let Match::Ore { name, exclusive } = rule {
        return offered.tags.contains(name) && (!exclusive || offered.tags.len() == 1);
    }
    if offered.registry != anchor.registry {
        return false;
    }
    match rule {
        Match::Exact => offered.meta == anchor.meta && offered.nbt == anchor.nbt,
        Match::Ae => {
            offered.meta == anchor.meta && ae_nbt(offered.nbt.as_ref(), anchor.nbt.as_ref())
        }
        Match::Wildcard { meta, nbt } => {
            (*meta || offered.meta == anchor.meta) && (*nbt || offered.nbt == anchor.nbt)
        }
        Match::Tags {
            meta,
            keys,
            present,
            absent,
        } => {
            let actual = compound_tags(offered);
            let expected = compound_tags(anchor);
            (*meta || offered.meta == anchor.meta)
                && keys
                    .iter()
                    .all(|key| actual.and_then(|t| t.get(key)) == expected.and_then(|t| t.get(key)))
                && present
                    .iter()
                    .all(|key| actual.is_some_and(|t| t.contains_key(key)))
                && absent
                    .iter()
                    .all(|key| actual.is_none_or(|t| !t.contains_key(key)))
        }
        Match::WithoutTags { keys } => {
            if offered.meta != anchor.meta {
                return false;
            }
            match (&offered.nbt, &anchor.nbt) {
                (Some(Nbt::Compound { value }), expected) => {
                    let remaining = || {
                        value
                            .iter()
                            .filter(|(key, _)| keys.binary_search(key).is_err())
                    };
                    let count = remaining().count();
                    if count == 0 {
                        expected.is_none()
                    } else if let Some(Nbt::Compound { value }) = expected {
                        count == value.len()
                            && remaining().all(|(key, tag)| value.get(key) == Some(tag))
                    } else {
                        false
                    }
                }
                (actual, expected) => actual == expected,
            }
        }
        Match::Ore { .. }
        | Match::Member { .. }
        | Match::Except { .. }
        | Match::Infusion { .. } => {
            unreachable!("validated primitive match")
        }
    }
}

fn ae_nbt(a: Option<&Nbt>, b: Option<&Nbt>) -> bool {
    let empty = |value: Option<&Nbt>| {
        value.is_none() || matches!(value, Some(Nbt::Compound {value}) if value.is_empty())
    };
    if empty(a) && empty(b) {
        return true;
    }
    match (a, b) {
        (Some(a), Some(b)) => ae_tag(a, b),
        _ => false,
    }
}

fn ae_tag(a: &Nbt, b: &Nbt) -> bool {
    match (a, b) {
        (Nbt::Float { value: a }, Nbt::Float { value: b }) => {
            match (u32::from_str_radix(a, 16), u32::from_str_radix(b, 16)) {
                (Ok(a), Ok(b)) => f32::from_bits(a) == f32::from_bits(b),
                _ => false,
            }
        }
        (Nbt::Double { value: a }, Nbt::Double { value: b }) => {
            match (u64::from_str_radix(a, 16), u64::from_str_radix(b, 16)) {
                (Ok(a), Ok(b)) => f64::from_bits(a) == f64::from_bits(b),
                _ => false,
            }
        }
        (Nbt::List { value: a, .. }, Nbt::List { value: b, .. }) => {
            a.len() == b.len() && a.iter().zip(b).all(|(a, b)| ae_tag(a, b))
        }
        (Nbt::Compound { value: a }, Nbt::Compound { value: b }) => {
            a.len() == b.len()
                && a.iter()
                    .all(|(key, a)| b.get(key).is_some_and(|b| ae_tag(a, b)))
        }
        _ => a == b,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{domain::Domain, identity::item_id, source::Source};
    use serde_json::json;

    #[test]
    fn ae_precise_preserves_native_nbt_comparison() {
        let rule: Match = serde_json::from_value(json!({"kind":"ae"})).unwrap();
        let source = Source::open(
            &std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../contracts/fixtures/source"),
        )
        .unwrap();
        let domain = Domain::load(&source).unwrap();
        let mut a = domain.items[0].clone();
        a.nbt = None;
        let mut b = a.clone();
        b.nbt = serde_json::from_value(json!({"type":"compound","value":{}})).unwrap();
        assert!(simple(&rule, &a, &b));
        assert!(!simple(&Match::Exact, &a, &b));
        assert!(rule
            .validate_item(&a, &BTreeMap::from([(a.id.as_str(), &a)]), false, false)
            .is_err());
        let tags = |bits: &str, element: &str| {
            serde_json::from_value(json!({"type":"compound","value":{
            "zero":{"type":"float","value":bits},"list":{"type":"list","element":element,"value":[]}}})).unwrap()
        };
        a.nbt = tags("80000000", "string");
        b.nbt = tags("00000000", "end");
        assert!(simple(&rule, &a, &b));
        b.meta = 32767;
        assert!(!simple(&rule, &a, &b));
        b.meta = a.meta;
        b.nbt = tags("3f800000", "end");
        assert!(!simple(&rule, &a, &b));
        a.nbt = tags("7fc00000", "end");
        b.nbt = a.nbt.clone();
        assert!(!simple(&rule, &a, &b));
        let items = BTreeMap::from([(a.id.as_str(), &a)]);
        assert!(
            rule.validate_item(&a, &items, false, true).is_err(),
            "NaN cannot model reference-only equality"
        );
    }

    #[test]
    fn priority_retains_unshadowed_variants_and_strict_nbt() {
        let source = Source::open(
            &std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../contracts/fixtures/source"),
        )
        .unwrap();
        let domain = Domain::load(&source).unwrap();
        let anchor = domain
            .items
            .iter()
            .find(|i| i.id == item_id("minecraft:stone", 0, None).unwrap())
            .unwrap();
        let rule: Match = serde_json::from_value(
            json!({"kind":"except","base":{"kind":"wildcard","meta":true,"nbt":true},
            "exclude":[{"id":anchor.id,"rule":{"kind":"exact"}}]}),
        )
        .unwrap();
        let items = BTreeMap::from([(anchor.id.as_str(), anchor)]);
        assert!(!rule.indexes_example(anchor, &items));
        let mut offered = anchor.clone();
        offered.meta = 1;
        assert!(rule.indexes_example(&offered, &items));
        offered.meta = 0;
        offered.nbt = serde_json::from_value(json!({"type":"compound","value":{}})).unwrap();
        assert!(
            rule.indexes_example(&offered, &items),
            "empty compound is not exact null NBT"
        );
        let tags: Match = serde_json::from_value(
            json!({"kind":"tags","meta":false,"keys":[],"present":["synthetic"],"absent":[]}),
        )
        .unwrap();
        offered.nbt = serde_json::from_value(
            json!({"type":"compound","value":{"synthetic":{"type":"byte","value":"0"}}}),
        )
        .unwrap();
        assert!(
            simple(&tags, &offered, &offered),
            "false still counts as present"
        );
        let removal: Match =
            serde_json::from_value(json!({"kind":"without_tags","keys":["synthetic"]})).unwrap();
        assert!(simple(&removal, &offered, anchor));
        if let Some(Nbt::Compound { value }) = &mut offered.nbt {
            value.insert("owner".into(), Nbt::Int { value: "1".into() });
        }
        assert!(
            !simple(&removal, &offered, anchor),
            "unremoved NBT remains significant"
        );
    }
}
