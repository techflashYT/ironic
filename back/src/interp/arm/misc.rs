use ironic_core::cpu::Cpu;
use crate::bits::arm::*;
use crate::interp::DispatchRes;

pub fn bkpt(_cpu: &mut Cpu, op: BkptBits) -> DispatchRes {
    println!("Breakpoint intsruction: {:#x}", op.imm16());
    DispatchRes::FatalErr
}
