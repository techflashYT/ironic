use std::env::temp_dir;
use ironic_core::bus::Bus;
use ironic_core::cpu::Cpu;
use crate::bits::arm::*;
use crate::interp::DispatchRes;
use anyhow::anyhow;

fn dump_memory(bus: &Bus) -> anyhow::Result<()> {
    let dir = temp_dir();

    let mut sram0_dir = temp_dir();
    sram0_dir.push("sram0-bkpt");
    bus.sram0.dump(&sram0_dir)?;

    let mut sram1_dir = temp_dir();
    sram1_dir.push("sram1-bkpt");
    bus.sram1.dump(&sram1_dir)?;

    let mut mem1_dir = temp_dir();
    mem1_dir.push("mem1-bkpt");
    bus.mem1.dump(&mem1_dir)?;

    let mut mem2_dir = dir;
    mem2_dir.push("mem2-bkpt");
    bus.mem2.dump(&mem2_dir)?;
    Ok(())
}

/// Breakpoint instruction:
/// ffff = Immediately stop emulator (dumps RAM)
/// fffe = cpu debug print toggle
/// fffd = cpu debug print on
/// fffc = cpu debug print off
/// fffb = dump RAM and continue
/// All other values, store in scratch reg and wait for debugger
pub fn bkpt(cpu: &mut Cpu, op: BkptBits) -> DispatchRes {
    let cmd = op.imm16() as u16;
    println!("Breakpoint instruction: {cmd:#x}");

    match cmd {
        0xffff => { return DispatchRes::FatalErr(anyhow!("Breakpoint 0xffff - stopping emulation")) }
        0xfffc..=0xfffe => {
            match cmd & 0x3 {
                0b10 => { cpu.dbg_on = !cpu.dbg_on; }
                0b01 => { cpu.dbg_on = true; }
                0b00 => {cpu.dbg_on = false; }
                _ => { unreachable!() }
            }
            return DispatchRes::RetireOk;
        },
        0xfffb => {
            let bus = cpu.bus.read().expect("breakpoint instruction - bus access");
            if let Err(e) = dump_memory(&bus) {
                return DispatchRes::FatalErr(e);
            };
            return DispatchRes::RetireOk;
        },
        _      => {},
    }
    cpu.scratch = cmd as u32;
    DispatchRes::Breakpoint
}
