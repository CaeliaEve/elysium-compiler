//! Native GT5 5.09.51.482 in-world pumping, not input-free guaranteed production.
use super::{Kind,OutputRole,Quantity,Recipe};
use crate::identity::integer;
use anyhow::{ensure,Result};
use schemars::JsonSchema;
use serde::{Deserialize,Serialize};
#[derive(Clone,Debug,Serialize,Deserialize,JsonSchema)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct Acquisition {
    pub operation:Operation,pub planet:String,pub selector:i32,pub fluid:String,pub base_amount:String,
    pub tool_debit:String,pub duration_law:String,pub environment_law:String,pub power_law:String,
    pub output_law:String,pub selection_law:String,pub modules:Vec<Module>,pub energy_per_parallel:Option<String>,
    pub batch_limit:Option<u32>,
    #[serde(default,skip_serializing_if="Option::is_none")]pub pipe_law:Option<String>,
    #[serde(default,skip_serializing_if="Option::is_none")]pub circuit_item_role:Option<String>,
    #[serde(default,skip_serializing_if="Option::is_none")]pub pipe_item_role:Option<String>,
}
#[derive(Clone,Copy,Debug,PartialEq,Eq,Serialize,Deserialize,JsonSchema)]
#[serde(rename_all="lowercase")]pub enum Operation{Pump,Siphon}
#[derive(Clone,Debug,PartialEq,Eq,Serialize,Deserialize,JsonSchema)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct Module{pub tier:u32,pub voltage_tier:u32,pub motor_tier:u32,pub parallels:u32,pub slots:u32}
pub(super) fn validate(p:&Acquisition,r:&Recipe)->Result<()> {
    let pump=p.operation==Operation::Pump;
    ensure!(r.source.handler==if pump{"gtnhintergalactic.nei.SpacePumpModuleRecipeHandler"}else{"gtnhintergalactic.nei.GasSiphonRecipeHandler"}
        &&r.inputs.is_empty()&&r.outputs.len()==1&&r.grid.is_none()&&r.magic.is_none()&&r.energy.is_none()&&r.duration.is_none(),"Space acquisition must retain its native conditional process");
    ensure!(!p.planet.is_empty()&&p.planet.len()<=4096&&p.tool_debit=="none","Space tooling/selector changed");
    if pump {
        ensure!(p.planet.parse::<i32>().is_ok_and(|n|n.to_string()==p.planet),"Pump planet is not its native numeric selector");
        ensure!(p.duration_law=="20TimesMaximumClampedBatch"&&p.environment_law=="connectedElevatorAndValidModuleStructure"
            &&p.power_law=="maximumModuleCapacityEnergyBufferGateAndSignedIntEUt"
            &&p.output_law=="firstMatchingLockedFilledHatchUnlessMEOrSafeVoid"&&p.selection_law=="moduleSettingsPlanetGas"
            &&p.energy_per_parallel.as_deref()==Some("1966080")&&p.batch_limit==Some(128)
            &&p.pipe_law.is_none()&&p.circuit_item_role.is_none()&&p.pipe_item_role.is_none(),"Native pump settings/power/output laws changed");
        ensure!(p.modules==vec![Module{tier:1,voltage_tier:9,motor_tier:2,parallels:4,slots:1},
            Module{tier:2,voltage_tier:10,motor_tier:3,parallels:4,slots:4},
            Module{tier:3,voltage_tier:13,motor_tier:4,parallels:64,slots:4}],"Native module capacity changed");
    }else {
        ensure!(p.duration_law=="20Ticks"&&p.environment_law=="orbitProviderAndValidSiphonStructure"
            &&p.power_law=="orbitTierDepthSignedIntLogFourOverclock"&&p.output_law=="baseFluidSpaceBeforeOverclock"
            &&p.selection_law=="lastCircuitControllerThenFirstValidBus"&&p.modules.is_empty()
            &&p.energy_per_parallel.is_none()&&p.batch_limit.is_none()
            &&p.pipe_law.as_deref()==Some("signedIntSumAtLeastDepthTimes64")
            &&p.circuit_item_role.as_deref()==Some("gregtech.api.enums.ItemList.Circuit_Integrated")
            &&p.pipe_item_role.as_deref()==Some("IC2:miningPipe"),"Native orbit/pipe/overclock laws changed");
    }
    let o=&r.outputs[0];
    ensure!(o.id==p.fluid&&o.kind==Kind::Fluid&&matches!(o.role,OutputRole::Result)&&o.slot==0&&o.amount.is_none()&&o.change.is_none()
        &&o.chance.numerator=="1"&&o.chance.denominator=="1"
        &&matches!(&o.quantity,Some(Quantity::SpaceAcquisition{nominal}) if nominal==&p.base_amount),"Acquisition output must preserve nominal and conditional yield");
    integer(&p.base_amount,1,i64::from(i32::MAX))?;Ok(())
}
/// Evaluate native signed Java power/output arithmetic once orbit and voltage are supplied.
/// Structure, stock, maintenance and output acceptance remain separate world gates.
pub fn siphon_overclock(depth:i32,tier:i32,voltage:i64,base:i32)->Option<(i32,i32)> {
    let shift=tier.wrapping_mul(2).wrapping_add(2) as u32 &31;
    let eut=depth.wrapping_mul(4i32.wrapping_shl(shift));
    let log=((voltage as f64)/(eut as f64)).log10()/4f64.log10();
    let trunc=log as i32;let mut oc=if log<(trunc as f64){trunc.wrapping_sub(1)}else{trunc};
    if oc<0{return None;}if oc==0{return Some((base,eut.wrapping_neg()));}
    oc=oc.wrapping_sub(1);
    Some((base.wrapping_mul(2i32.wrapping_shl(oc as u32&31)),
        eut.wrapping_neg().wrapping_mul(4i32.wrapping_shl(oc.wrapping_mul(2) as u32&31))))
}
/// One configured pump lane. `space` is the first matching filled/locked hatch,
/// after applying native ME/safe-void bypass; absent matching hatches impose no limit.
/// A division fault is distinct from a lane that produces nothing.
pub fn pump_branch(base:i32,limit:i32,setting:i32,batch:f64,space:Option<i32>)->Result<Option<(i64,i32,i32)>> {
    let mut batch=if batch.is_nan(){0}else{batch.max(1.0).min(128.0) as i32};
    let mut parallel=setting.min(limit);
    if let Some(space)=space {
        if space<base{return Ok(None);}
        ensure!(base!=0,"Native pump division by zero base");
        parallel=parallel.min(space/base);
        let denominator=base.wrapping_mul(parallel);
        ensure!(denominator!=0,"Native pump division by zero parallel");
        batch=batch.min(space.wrapping_div(denominator));
    }
    if parallel<=0||batch<=0{return Ok(None);}
    Ok(Some((i64::from(base)*i64::from(parallel)*i64::from(batch),batch,(1966080i64*i64::from(parallel)) as i32)))
}
#[cfg(test)]mod tests {
    #[test]fn pump_clamps_batch_and_limits_parallel_to_matching_output_space(){
        assert_eq!(super::pump_branch(15000,64,99,200.0,None).unwrap(),Some((122880000,128,125829120)));
        assert_eq!(super::pump_branch(15000,4,4,10.0,Some(60000)).unwrap(),Some((60000,1,7864320)));
        assert_eq!(super::pump_branch(15000,4,4,10.0,Some(14999)).unwrap(),None);
        assert_eq!(super::pump_branch(15000,4,4,-2.0,None).unwrap(),Some((60000,1,7864320)));
        // Native divides by fluid*parallel before checking positive parallels.
        assert!(super::pump_branch(15000,4,0,10.0,Some(60000)).is_err());
    }
    #[test]fn depth_and_orbit_tier_control_voltage_and_signed_overclock(){
        assert_eq!(super::siphon_overclock(2,3,1024,15000),None);
        assert_eq!(super::siphon_overclock(2,3,2048,15000),Some((15000,-2048)));
        assert_eq!(super::siphon_overclock(2,3,8192,15000),Some((30000,-8192)));
        assert_eq!(super::siphon_overclock(2,3,32768,15000),Some((60000,-32768)));
        assert_eq!(super::siphon_overclock(2,3,8192,i32::MAX),Some((-2,-8192)));
    }
}
