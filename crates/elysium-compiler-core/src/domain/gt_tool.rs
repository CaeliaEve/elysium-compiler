//! Pinned GT 5.09.51.482 container tool declaration, without sampling its RNG.
use super::{Choice, GtToolSample, Item, Recipe};
use crate::identity::{integer,Nbt};
use anyhow::{ensure,Result};
use std::collections::BTreeMap;

pub(super) fn validate(recipe:&Recipe,choice:&Choice,item:&Item,cost:u32,owner:&str,sample:&GtToolSample)->Result<()> {
    ensure!(super::native_crafting::container_runtime(recipe),
        "GT tool container is outside the native crafting reader");
    ensure!(choice.amount=="1" && choice.returns.is_empty() && item.meta%2==0,
        "native GT tool keeps one conditional tool in the grid, not a fixed container return");
    let name=owner.strip_prefix("gregtech.common.tools.").unwrap_or("");
    let expected=match name {
        "" if owner=="gtPlusPlus.xmod.gregtech.common.tools.ToolAngleGrinder"=>400,
        "ToolButcheryKnife"|"ToolBuzzSaw"|"ToolCrowbar"|"ToolDrillLV"|"ToolKnife"|"ToolRollingPin"=>100,
        "ToolSaw"|"ToolScoop"|"ToolScrewdriverLV"=>200,
        "ToolFile"|"ToolHardHammer"|"ToolMortar"|"ToolScrewdriver"|"ToolWireCutter"=>400,
        "GTTool"|"ToolChainsawLV"=>800,
        "ToolSolderingIron"=>1000,
        "ToolChainsawMV"|"ToolDrillMV"|"ToolJackHammer"|"ToolWrenchMV"=>3200,
        "ToolChainsawHV"|"ToolDrillHV"|"ToolWrenchHV"=>12800,
        _=>anyhow::bail!("unadapted native GT tool wear owner"),
    };
    ensure!(cost==expected,"native GT crafting cost differs from pinned callback");
    let stats=compound(item.nbt.as_ref()).and_then(|root|root.get("GT.ToolStats")).and_then(|tag|compound(Some(tag)));
    let get=|key:&str|stats.and_then(|tags|tags.get(key));
    ensure!(sample.electric==((super::change::number(get("Electric"))? as i8)!=0),"native electric-tool sample flag differs");
    ensure!(integer(&sample.damage,i64::MIN,i64::MAX)?==long(get("Damage"))?
        && integer(&sample.limit,i64::MIN,i64::MAX)?==long(get("MaxDamage"))?,"native tool damage/limit sample differs");
    Ok(())
}
fn compound(tag:Option<&Nbt>)->Option<&BTreeMap<String,Nbt>> {
    if let Some(Nbt::Compound{value})=tag{Some(value)}else{None}
}
/// Minecraft NBTPrimitive.getLong: floating-point values truncate with Java saturation.
fn long(tag:Option<&Nbt>)->Result<i64> {
    Ok(match tag {
        Some(Nbt::Byte{value}|Nbt::Short{value}|Nbt::Int{value}|Nbt::Long{value})=>value.parse()?,
        Some(Nbt::Float{value})=>f32::from_bits(u32::from_str_radix(value,16)?) as i64,
        Some(Nbt::Double{value})=>f64::from_bits(u64::from_str_radix(value,16)?) as i64,
        _=>0,
    })
}
