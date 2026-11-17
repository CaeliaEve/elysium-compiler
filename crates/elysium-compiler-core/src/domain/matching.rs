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
            Self::NativePoolInput=>ensure!(compound,"Native pool display requires its ordered session"),
            Self::MatterManipulatorInput|Self::MatterManipulatorUpgrade=>ensure!(compound,"Matter input requires its native state process"),
            Self::NativeMetadataKey{code}=>ensure!(*code>0&&item.meta.wrapping_add(1) as u16==*code,"Invalid native baked metadata-key example"),
            Self::EnchanterBook=>ensure!(compound,"Enchanter presence predicate requires native pickup process"),
            Self::FmpMaterial{program,native_text,..}=>ensure!(compound
                &&program.strip_prefix("program_").is_some_and(crate::source::is_digest)
                &&native_text.as_ref().is_none_or(|s|s.len()<=524288),"Invalid shared FMP material predicate"),
            Self::NativeZeroFluid { .. } | Self::LumbermillTank => anyhow::bail!("native fluid gate is not an item predicate"),
            Self::CalInput{role,imprint_ids}=>{
                ensure!(compound,"CAL input requires its native process");super::cal_imprint::validate_ids(imprint_ids)?;
                ensure!(super::cal_imprint::accepts(*role,item,imprint_ids)?,"CAL input rejects its native example");
            }
            Self::WitcheryOvenNbt { product, .. } => ensure!(compound && matches!(product,
                super::witchery::OvenByproduct::None | super::witchery::OvenByproduct::Foul
                | super::witchery::OvenByproduct::Exhale | super::witchery::OvenByproduct::Hint
                | super::witchery::OvenByproduct::Breath), "invalid native oven NBT partition"),
            Self::BannerInput{max_patterns} => ensure!(compound && *max_patterns==6 && super::banner::pattern_count(item.nbt.as_ref())<6,"Invalid native banner input"),
            Self::FacadeTexture => ensure!(compound && item.nbt.is_none(), "native facade texture must have absent NBT"),
            Self::AeCrystalSeed { start, end } => {
                ensure!(compound && matches!((*start,*end),(0,600)|(600,1200)|(1200,1800))
                    && simple(self,item,item), "native crystal seed progress differs from its growth band");
            }
            Self::AeEnderDust { ore, metadata } => ensure!(compound
                && (ore=="dustEnder"||ore=="dustEnderPearl") && *metadata>=0
                && (*metadata==32767 || item.meta==*metadata),
                "invalid native AE2 ender-dust template"),
            Self::BloodAltar { tagged, keys } => {
                ensure!(keys.len() <= 65536 && keys.iter().all(|key| key.len() <= 65535)
                    && keys.windows(2).all(|pair| pair[0] < pair[1])
                    && (*tagged || keys.is_empty()), "Invalid native altar tag predicate");
            }
            Self::Exact => {}
            Self::ChiselSource { nbt, .. } => {
                ensure!(compound && *nbt, "Chisel source requires native process lookup and ignored NBT");
            }
            Self::ChiselTool { .. } => {
                ensure!(compound, "Chisel tool requires correlated source lookup");
            }
            Self::CreativeCore { .. } | Self::DistilleryJar | Self::Integration | Self::Forestry => ensure!(
                compound,
                "integration cannot be nested in an independent predicate"
            ),
            Self::Untagged { .. } => ensure!(
                item.nbt.is_none()
                    || matches!(&item.nbt, Some(Nbt::Compound {value}) if value.is_empty()),
                "untagged predicate has a nonempty anchor"
            ),
            Self::Buildcraft { wildcard, .. } => {
                ensure!(
                    compound,
                    "BuildCraft predicate cannot be nested in priority filters"
                );
                ensure!(
                    *wildcard
                        || item
                            .nbt
                            .as_ref()
                            .is_none_or(|tag| native_tag(tag, tag, true)),
                    "BuildCraft NaN predicate depends on object identity"
                );
            }
            Self::Soul { filter } => {
                ensure!(
                    compound,
                    "soul predicates cannot be nested in priority filters"
                );
                filter.validate(items)?;
                ensure!(
                    filter.accepts_sample(item, items)?,
                    "soul filter rejects its example"
                );
            }
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
            Self::StringTag { key } => {
                ensure!(
                    !key.is_empty() && key.len() <= 65535,
                    "invalid string tag key"
                );
                ensure!(
                    simple(self, item, item),
                    "string tag predicate rejects its example"
                );
            }
            Self::Metadata { value, nbt, absent } => {
                ensure!(*value >= -1, "literal metadata only permits -1 as a negative sentinel");
                ensure!(
                    absent.len() <= 4096
                        && absent.windows(2).all(|pair| pair[0] < pair[1])
                        && absent.iter().all(|key| key.len() <= 65535),
                    "absent tag keys must be bounded, sorted and unique"
                );
                ensure!(
                    *nbt || absent
                        .iter()
                        .all(|key| compound_tags(item).is_none_or(|tags| !tags.contains_key(key))),
                    "literal metadata predicate has conflicting NBT constraints"
                );
            }
            Self::MetadataSet { values, .. } => ensure!(
                !values.is_empty() && values.len() <= 65536
                    && values.windows(2).all(|pair| pair[0] < pair[1]),
                "metadata partition values must be nonempty, bounded, sorted and unique"
            ),
            Self::NativeItemMetadata { callback, .. } => ensure!(
                item.registry == super::native_item_metadata::registry(*callback),
                "native logical metadata callback item identity differs"
            ),
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
            Self::NativePoolInput=>true, // Index the constructor role for discovery; the shared session still owns acceptance.
            Self::FmpMaterial{..}=>true, // Full-domain program validation already admitted this exact witness.
            Self::NativeItemMetadata { .. } | Self::MetadataSet { .. } | Self::NativeMetadataKey{..}|Self::CalInput { .. } | Self::BannerInput { .. } | Self::Metadata { .. } | Self::BloodAltar { .. } | Self::FacadeTexture | Self::AeCrystalSeed { .. } | Self::AeEnderDust { .. }
                | Self::WitcheryOvenNbt { .. } => simple(self, item, item),
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
pub(super) fn simple(rule: &Match, offered: &Item, anchor: &Item) -> bool {
    if let Match::Ore { name, exclusive } = rule {
        return offered.tags.contains(name) && (!exclusive || offered.tags.len() == 1);
    }
    if offered.registry != anchor.registry {
        return false;
    }
    match rule {
        Match::NativePoolInput=>false,
        Match::NativeMetadataKey{code}=>offered.meta.wrapping_add(1) as u16==*code,
        Match::FmpMaterial{..}=>false, // Requires its shared native lookup; never guess in a generic matcher.
        Match::CalInput{role,imprint_ids}=>offered.meta==anchor.meta&&super::cal_imprint::accepts(*role,offered,imprint_ids).unwrap_or(false),
        Match::WitcheryOvenNbt { meta, product } => (*meta || offered.meta==anchor.meta)
            && super::witchery::oven_nbt_product(offered.nbt.as_ref())==*product,
        Match::BannerInput { max_patterns } => super::banner::pattern_count(offered.nbt.as_ref())<*max_patterns as usize,
        Match::FacadeTexture => offered.meta==anchor.meta && offered.nbt.is_none(),
        Match::AeEnderDust { metadata, .. } => offered.meta==32767||offered.meta==*metadata,
        Match::AeCrystalSeed { start,end } => {
            let progress=if let Some(tags)=compound_tags(offered) {
                change::number(tags.get("progress")).unwrap_or(0)
            } else if offered.nbt.is_none() {offered.meta} else {0};
            progress>=*start && progress<*end
        }
        Match::BloodAltar { tagged, keys } => {
            offered.meta == anchor.meta && (!*tagged || compound_tags(offered).is_some_and(|tags|
                keys.iter().all(|key| !matches!(tags.get(key), Some(Nbt::Compound { .. })))))
        }
        Match::Exact => offered.meta == anchor.meta && offered.nbt == anchor.nbt,
        Match::Untagged { meta } => {
            (*meta || offered.meta == anchor.meta)
                && (offered.nbt.is_none()
                    || matches!(&offered.nbt, Some(Nbt::Compound {value}) if value.is_empty()))
        }
        Match::Buildcraft { wildcard, subtypes } => {
            *wildcard
                || matches!(offered.meta, -1 | 32767)
                || ((!subtypes || offered.meta == anchor.meta)
                    && match (&offered.nbt, &anchor.nbt) {
                        (None, None) => true,
                        (Some(a), Some(b)) => native_tag(a, b, true),
                        _ => false,
                    })
        }
        Match::Ae => {
            offered.meta == anchor.meta && ae_nbt(offered.nbt.as_ref(), anchor.nbt.as_ref())
        }
        Match::Wildcard { meta, nbt } => {
            (*meta || offered.meta == anchor.meta) && (*nbt || offered.nbt == anchor.nbt)
        }
        Match::StringTag { key } => compound_tags(offered)
            .and_then(|tags| tags.get(key))
            .is_none_or(|tag| matches!(tag, Nbt::String { .. })),
        Match::Metadata { value, nbt, absent } => {
            offered.meta == *value
                && (*nbt || offered.nbt == anchor.nbt)
                && absent
                    .iter()
                    .all(|key| compound_tags(offered).is_none_or(|tags| !tags.contains_key(key)))
        }
        Match::MetadataSet { values, exclude, nbt } =>
            (values.binary_search(&offered.meta).is_ok() != *exclude)
                && (*nbt || offered.nbt == anchor.nbt),
        Match::NativeItemMetadata { callback, value, nbt } =>
            offered.registry == super::native_item_metadata::registry(*callback)
                && super::native_item_metadata::value(offered) == *value
                && (*nbt || offered.nbt == anchor.nbt),
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
        Match::NativeZeroFluid { .. }
        | Match::MatterManipulatorInput
        | Match::MatterManipulatorUpgrade
        | Match::Integration
        | Match::ChiselSource { .. }
        | Match::ChiselTool { .. }
        | Match::DistilleryJar
        | Match::CreativeCore { .. }
        | Match::EnchanterBook
        | Match::LumbermillTank
        | Match::Forestry
        | Match::Ore { .. }
        | Match::Soul { .. }
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
    native_tag(a, b, false)
}

pub(super) fn native_tag(a: &Nbt, b: &Nbt, list_type: bool) -> bool {
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
        (
            Nbt::List {
                value: a,
                element: at,
            },
            Nbt::List {
                value: b,
                element: bt,
            },
        ) => {
            (!list_type || at == bt)
                && a.len() == b.len()
                && a.iter().zip(b).all(|(a, b)| native_tag(a, b, list_type))
        }
        (Nbt::Compound { value: a }, Nbt::Compound { value: b }) => {
            a.len() == b.len()
                && a.iter()
                    .all(|(key, a)| b.get(key).is_some_and(|b| native_tag(a, b, list_type)))
        }
        _ => a == b,
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn returned_metadata_sets_keep_exact_partitions_and_reject_malformed_domains() {
        let source=Source::open(&std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../contracts/fixtures/source-semantic-40")).unwrap();
        let domain=Domain::load(&source).unwrap();
        let mut anchor=domain.items[0].clone();anchor.meta=2;anchor.nbt=None;
        let items=BTreeMap::from([(anchor.id.as_str(),&anchor)]);
        for exclude in [false,true] {
            let rule:Match=serde_json::from_value(json!({"kind":"metadataSet","values":[0,2,32767],"exclude":exclude,"nbt":false})).unwrap();
            rule.validate_item(&anchor,&items,false,true).unwrap();
            for (meta,included) in [(-1,false),(0,true),(1,false),(2,true),(32767,true),(i32::MAX,false)] {
                let mut offered=anchor.clone();offered.meta=meta;
                assert_eq!(simple(&rule,&offered,&anchor),included!=exclude);
                offered.nbt=Some(Nbt::Compound{value:BTreeMap::new()});
                assert!(!simple(&rule,&offered,&anchor),"exact NBT still distinguishes absent and empty");
                offered.registry="fixture:other".into();assert!(!simple(&rule,&offered,&anchor));
            }
            assert_eq!(rule.indexes_example(&anchor,&items),!exclude);
        }
        for values in [json!([]),json!([2,1]),json!([1,1])] {
            let rule:Match=serde_json::from_value(json!({"kind":"metadataSet","values":values,"exclude":false,"nbt":true})).unwrap();
            assert!(rule.validate_item(&anchor,&items,false,true).is_err());
        }
    }
    #[test]
    fn returned_native_metadata_uses_java_float_values_and_keeps_physical_identity() {
        let source=Source::open(&std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../contracts/fixtures/source-semantic-40")).unwrap();
        let domain=Domain::load(&source).unwrap();
        let mut anchor=domain.items[0].clone();anchor.registry="Botania:manaMirror".into();anchor.meta=123;anchor.nbt=None;
        for (mana,logical) in [(0,1000),(1000,999),(1_000_000,0),(16_777_217,-15777),(i32::MAX,-2146483),(i32::MIN,2148483)] {
            let mut offered=anchor.clone();offered.meta=9;
            offered.nbt=Some(Nbt::Compound{value:BTreeMap::from([("mana".into(),Nbt::Int{value:mana.to_string()})])});
            let rule:Match=serde_json::from_value(serde_json::json!({"kind":"nativeItemMetadata","callback":"botaniaManaMirror","value":logical,"nbt":true})).unwrap();
            assert!(simple(&rule,&offered,&anchor),"{mana}");
            assert_eq!(offered.meta,9);
            offered.registry="fixture:unreviewed".into();assert!(!simple(&rule,&offered,&anchor));
        }
        assert!(serde_json::from_value::<super::super::NativeContainerCallback>(serde_json::json!("thaumicMortar")).is_ok());
        assert!(serde_json::from_value::<super::super::NativeContainerCallback>(serde_json::json!("railFirestone")).is_ok());
    }
    #[test]
    fn ae2_ender_dust_keeps_native_offered_first_wildcard_direction() {
        let source=Source::open(&std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../contracts/fixtures/source-semantic-40")).unwrap();
        let domain=Domain::load(&source).unwrap();let mut anchor=domain.items[0].clone();anchor.registry="minecraft:ender_pearl".into();anchor.meta=0;anchor.tags.clear();anchor.nbt=None;
        let cases:serde_json::Value=serde_json::from_str(include_str!("../../../../contracts/fixtures/ae-world-ender-matching.json")).unwrap();
        for case in cases.as_array().unwrap(){
            let rule:Match=serde_json::from_value(serde_json::json!({"kind":"ae2EnderDust","ore":"dustEnder","metadata":case["registered"]})).unwrap();
            let mut offered=anchor.clone();offered.meta=case["offered"].as_i64().unwrap()as i32;
            if case["tagged"]==true{offered.nbt=Some(Nbt::Compound{value:BTreeMap::new()});}
            assert_eq!(simple(&rule,&offered,&anchor),case["accepted"].as_bool().unwrap(),"{case}");
            offered.registry="fixture:other".into();assert!(!simple(&rule,&offered,&anchor));
        }
    }
    #[test]
    fn blood_altar_preserves_native_presence_recursion_and_literal_metadata() {
        let source = Source::open(&std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../contracts/fixtures/source-semantic-40")).unwrap();
        let domain = Domain::load(&source).unwrap();
        let anchor = domain.items.iter().find(|i| i.registry == "minecraft:stone" && i.meta == 0 && i.nbt.is_none()).unwrap();
        let rule = Match::BloodAltar { tagged: true, keys: vec!["native".into()] };
        let mut offered = anchor.clone();
        assert!(!simple(&rule, &offered, anchor));
        offered.nbt = Some(Nbt::Compound { value: BTreeMap::new() });
        assert!(simple(&rule, &offered, anchor), "Native ignores absent required keys but requires root tag");
        if let Some(Nbt::Compound { value }) = &mut offered.nbt {
            value.insert("native".into(), Nbt::Int { value: "99".into() });
        }
        assert!(simple(&rule, &offered, anchor), "Native ignores leaf values");
        if let Some(Nbt::Compound { value }) = &mut offered.nbt {
            value.insert("native".into(), Nbt::Compound { value: BTreeMap::new() });
        }
        assert!(!simple(&rule, &offered, anchor), "Native recursive failure must not be admitted");
        offered.nbt = None; offered.meta = 32767;
        let ignore = Match::BloodAltar { tagged: false, keys: vec![] };
        assert!(!simple(&ignore, &offered, anchor), "32767 is literal, not wildcard");
        offered.meta = 0; assert!(simple(&ignore, &offered, anchor));
    }
    #[test]
    fn literal_metadata_keeps_display_and_priority_independent() {
        let rule: Match = serde_json::from_value(serde_json::json!({
            "kind":"metadata", "value":32767, "nbt":false, "absent":["synthetic"]
        }))
        .expect("literal metadata predicate must deserialize");
        let source = Source::open(
            &std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../contracts/fixtures/source-semantic-40"),
        )
        .unwrap();
        let domain = Domain::load(&source).unwrap();
        let anchor = domain
            .items
            .iter()
            .find(|i| i.registry == "minecraft:stone" && i.meta == 0 && i.nbt.is_none())
            .unwrap();
        let items = BTreeMap::from([(anchor.id.as_str(), anchor)]);
        rule.validate_item(anchor, &items, false, true).unwrap();
        assert!(
            !rule.indexes_example(anchor, &items),
            "display-only metadata must not create a false usage link"
        );
        let mut offered = anchor.clone();
        for meta in [0, 3, 7, 32767] {
            offered.meta = meta;
            assert_eq!(simple(&rule, &offered, anchor), meta == 32767);
        }
        offered.nbt = Some(Nbt::Compound {
            value: BTreeMap::new(),
        });
        assert!(
            !simple(&rule, &offered, anchor),
            "empty NBT is not absent NBT"
        );
        let ignored: Match = serde_json::from_value(serde_json::json!({
            "kind":"metadata", "value":32767, "nbt":true, "absent":["synthetic"]
        }))
        .unwrap();
        assert!(simple(&ignored, &offered, anchor));
        if let Some(Nbt::Compound { value }) = &mut offered.nbt {
            value.insert("synthetic".into(), Nbt::Byte { value: "0".into() });
        }
        assert!(!simple(&ignored, &offered, anchor));
        for invalid in [
            serde_json::json!({"kind":"metadata","value":-2,"nbt":true,"absent":[]}),
            serde_json::json!({"kind":"metadata","value":32767,"nbt":true,"absent":["x","x"]}),
        ] {
            assert!(serde_json::from_value::<Match>(invalid)
                .unwrap()
                .validate_item(anchor, &items, false, true)
                .is_err());
        }
        let excluded: Match = serde_json::from_value(serde_json::json!({
            "kind":"except", "base":{"kind":"wildcard","meta":true,"nbt":true},
            "exclude":[{"id":anchor.id,"rule":rule}]
        }))
        .unwrap();
        excluded.validate_item(anchor, &items, false, true).unwrap();
        let negative_sentinel: Match = serde_json::from_value(serde_json::json!({
            "kind":"metadata", "value":-1, "nbt":true, "absent":[]
        })).unwrap();
        negative_sentinel.validate_item(anchor, &items, false, true).unwrap();
        assert!(!negative_sentinel.indexes_example(anchor, &items));
        for meta in [-2, -1, 0, 32767] {
            offered.meta = meta;
            assert_eq!(simple(&negative_sentinel, &offered, anchor), meta == -1);
        }
        assert!(
            excluded.indexes_example(anchor, &items),
            "sentinel exclusion must not shadow the concrete display"
        );
    }
    use super::*;
    use crate::{domain::Domain, identity::item_id, source::Source};
    use serde_json::json;

    #[test]
    fn buildcraft_wildcard_shortcuts_preserve_native_nbt() {
        let rule: Match =
            serde_json::from_value(json!({"kind":"buildcraft","wildcard":false,"subtypes":true}))
                .unwrap();
        let source = Source::open(
            &std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../contracts/fixtures/source-semantic-40"),
        )
        .unwrap();
        let domain = Domain::load(&source).unwrap();
        let mut anchor = domain.items[0].clone();
        anchor.meta = 0;
        anchor.nbt = None;
        let mut offered = anchor.clone();
        offered.nbt = serde_json::from_value(json!({"type":"compound","value":{}})).unwrap();
        assert!(
            !simple(&rule, &offered, &anchor),
            "native BuildCraft distinguishes absent and empty NBT"
        );
        offered.meta = 32767;
        assert!(
            simple(&rule, &offered, &anchor),
            "offered wildcard skips NBT"
        );
        offered.registry = "different:item".into();
        assert!(
            !simple(&rule, &offered, &anchor),
            "wildcard must retain item identity"
        );
        offered = anchor.clone();
        offered.meta = 1;
        assert!(!simple(&rule, &offered, &anchor));
        let broad: Match =
            serde_json::from_value(json!({"kind":"buildcraft","wildcard":true,"subtypes":true}))
                .unwrap();
        assert!(simple(&broad, &offered, &anchor));
        let plain: Match =
            serde_json::from_value(json!({"kind":"buildcraft","wildcard":false,"subtypes":false}))
                .unwrap();
        assert!(simple(&plain, &offered, &anchor));
        let tag = |bits: &str| {
            serde_json::from_value(
                json!({"type":"compound","value":{"x":{"type":"float","value":bits}}}),
            )
            .unwrap()
        };
        anchor.nbt = tag("80000000");
        offered.meta = 0;
        offered.nbt = tag("00000000");
        assert!(
            simple(&rule, &offered, &anchor),
            "native signed zeros compare numerically"
        );
        let list = |element: &str| {
            serde_json::from_value(json!({"type":"compound","value":{"list":{"type":"list","element":element,"value":[]}}})).unwrap()
        };
        anchor.nbt = list("end");
        offered.nbt = list("string");
        assert!(
            !simple(&rule, &offered, &anchor),
            "native empty lists retain element type"
        );
        anchor.nbt = tag("7fc00000");
        assert!(
            rule.validate_item(&anchor, &BTreeMap::new(), false, true)
                .is_err(),
            "NaN predicates depend on native reference identity"
        );
    }

    #[test]
    fn ae_precise_preserves_native_nbt_comparison() {
        let rule: Match = serde_json::from_value(json!({"kind":"ae"})).unwrap();
        let source = Source::open(
            &std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../contracts/fixtures/source-semantic-40"),
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
                .join("../../contracts/fixtures/source-semantic-40"),
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
        let air: Match = serde_json::from_value(json!({"kind":"untagged","meta":true})).unwrap();
        assert!(simple(&air, &offered, anchor));
        offered.meta = 7;
        offered.nbt = Some(Nbt::Compound {
            value: BTreeMap::new(),
        });
        assert!(
            simple(&air, &offered, anchor),
            "empty root and non-subtype metadata must be accepted"
        );
        assert!(!simple(
            &serde_json::from_value(json!({"kind":"untagged","meta":false})).unwrap(),
            &offered,
            anchor
        ));
        offered.nbt = Some(Nbt::Compound {
            value: BTreeMap::from([("owner".into(), Nbt::Int { value: "0".into() })]),
        });
        assert!(
            !simple(&air, &offered, anchor),
            "nonempty NBT must not match an untagged air cell"
        );
        assert!(air.validate_item(&offered, &items, false, true).is_err());
        offered = anchor.clone();
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
