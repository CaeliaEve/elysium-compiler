//! Logical getter values never replace the physical metadata used by item ids.
use super::{change, Item, NativeItemMetadataCallback};
use crate::identity::Nbt;

pub(super) fn registry(callback: NativeItemMetadataCallback) -> &'static str {
    match callback {
        NativeItemMetadataCallback::BotaniaManaMirror => "Botania:manaMirror",
    }
}

pub(super) fn value(item: &Item) -> i32 {
    if item.registry != registry(NativeItemMetadataCallback::BotaniaManaMirror) {
        return item.meta;
    }
    let mana = match &item.nbt {
        Some(Nbt::Compound { value }) => change::number(value.get("mana")).unwrap_or(0),
        _ => 0,
    };
    // Separate f32 operations reproduce Java fdiv/fmul/f2i, including narrowing.
    let fraction = mana as f32 / 1_000_000.0_f32;
    1000_i32.wrapping_sub((fraction * 1000.0_f32) as i32)
}
