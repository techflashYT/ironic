
use std::env::temp_dir;
use crate::bits::thumb::*;
use crate::interp::DispatchRes;
use ironic_core::bus::Bus;
use ironic_core::cpu::Cpu;
use ironic_core::cpu::excep::ExceptionType;

pub fn svc(_cpu: &mut Cpu, _op: MiscBits) -> DispatchRes {
    DispatchRes::Exception(ExceptionType::Swi)
}

fn dump_memory(bus: &Bus) {
    let dir = temp_dir();

    let mut sram0_dir = temp_dir();
    sram0_dir.push("sram0-bkpt");
    bus.sram0.dump(&sram0_dir);

    let mut sram1_dir = temp_dir();
    sram1_dir.push("sram1-bkpt");
    bus.sram1.dump(&sram1_dir);

    let mut mem1_dir = temp_dir();
    mem1_dir.push("mem1-bkpt");
    bus.mem1.dump(&mem1_dir);

    let mut mem2_dir = dir;
    mem2_dir.push("mem2-bkpt");
    bus.mem2.dump(&mem2_dir);
}

/// Breakpoint instruction:
/// ff = Immediately stop emulator (dumps RAM)
/// fe = cpu debug print toggle
/// fd = cpu debug print on
/// fc = cpu debug print off
/// fb = dump RAM and continue
/// All other values, store in scratch reg and wait for debugger
pub fn bkpt(cpu: &mut Cpu, op: BkptBits) -> DispatchRes {
    let cmd = op.imm8() as u8;
    println!("Breakpoint instruction: {:#x}", cmd);

    match cmd {
        0xff => { return DispatchRes::FatalErr("Breakpoint 0xff - stopping emulation".to_string()) }
        0xfc..=0xfe => {
            match cmd & 0x3 {
                0b10 => { cpu.dbg_on = !cpu.dbg_on; }
                0b01 => { cpu.dbg_on = true; }
                0b00 => {cpu.dbg_on = false; }
                _ => { unreachable!() }
            }
            return DispatchRes::RetireOk;
        },
        0xfb => {
            let bus = cpu.bus.read().expect("breakpoint instruction - bus access");
            dump_memory(&bus);
            return DispatchRes::RetireOk;
        },
        _      => {},
    }
    cpu.scratch = cmd as u32;
    DispatchRes::Breakpoint
}