//! The interpreter backend.

pub mod arm;
pub mod thumb;
pub mod dispatch;
pub mod lut;

use std::sync::mpsc::Receiver;
use std::sync::{Arc, RwLock, mpsc::Sender};
use std::fs;
use std::io::Seek;
use std::io::Read;

extern crate elf;

use crate::back::*;
use crate::debug::DebugPacket;
use crate::interp::lut::*;
use crate::interp::dispatch::DispatchRes;

use crate::decode::arm::*;
use crate::decode::thumb::*;

use ironic_core::bus::*;
use ironic_core::cpu::{Cpu, CpuRes};
use ironic_core::cpu::reg::Reg;
use ironic_core::cpu::excep::ExceptionType;

/// Current stage in the platform's boot process.
#[derive(PartialEq)]
pub enum BootStatus { 
    /// Execution in the mask ROM.
    Boot0, 
    /// Execution in the first-stage bootloader.
    Boot1, 
    /// Execution in the second-stage bootloader stub.
    Boot2Stub, 
    /// Execution in the second-stage bootloader.
    Boot2, 
    /// Execution in the kernel.
    IOSKernel, 

    /// Execution in a user-loaded foreign kernel.
    UserKernelStub, 
    UserKernel, 
}

/// Backend for interpreting-style emulation. 
///
/// Right now, the main loop works like this:
///
/// - Execute all pending work on the bus
/// - Update the state of any signals from the bus to the CPU
/// - Decode/dispatch an instruction, mutating the CPU state
///
/// For now it's sufficient to perfectly interleave bus and CPU cycles, but
/// maybe at some point it will become more efficient to let dispatched
/// instructions return some hint to the backend (requesting that a bus cycle
/// should be completed before the next instruction).

pub struct InterpBackend {
    /// Reference to a bus (attached to memories and devices).
    pub bus: Arc<RwLock<Bus>>,

    /// The CPU state.
    pub cpu: Cpu,

    /// Number of CPU cycles elapsed.
    pub cpu_cycle: usize,
    /// Number of bus cycles elapsed.
    pub bus_cycle: usize,

    /// Buffer for semi-hosting debug writes.
    pub svc_buf: String,
    /// Current stage in the platform boot process.
    pub boot_status: BootStatus,
    pub custom_kernel: Option<String>,
    emu_send: Sender<DebugPacket>,
    emu_recv: Receiver<DebugPacket>,
    debugger_attached: bool,
    debug_cycles: u32,
}
impl InterpBackend {
    pub fn new(bus: Arc<RwLock<Bus>>, custom_kernel: Option<String>, emu_send: Sender<DebugPacket>, emu_recv: Receiver<DebugPacket>) -> Self {
        InterpBackend {
            svc_buf: String::new(),
            cpu: Cpu::new(bus.clone()),
            boot_status: BootStatus::Boot0,
            cpu_cycle: 0,
            bus_cycle: 0,
            bus,
            custom_kernel,
            emu_send,
            emu_recv,
            debugger_attached: false,
            debug_cycles: 0,
        }
    }
}

impl InterpBackend {
    /// Check if we need to update the current boot stage.
    pub fn update_boot_status(&mut self) {
        match self.boot_status {
            BootStatus::Boot0 => {
                if self.cpu.read_fetch_pc() == 0xfff0_0000 { 
                    println!("Entered boot1");
                    self.boot_status = BootStatus::Boot1;
                }
            }
            BootStatus::Boot1 => {
                if self.cpu.read_fetch_pc() == 0xfff0_0058 { 
                    println!("Entered boot2 stub");
                    self.boot_status = BootStatus::Boot2Stub;
                }
            }
            BootStatus::Boot2Stub => {
                if self.cpu.read_fetch_pc() == 0xffff_0000 { 
                    println!("Entered boot2");
                    self.boot_status = BootStatus::Boot2;
                }
            }
            BootStatus::Boot2 => {
                if self.cpu.read_fetch_pc() == 0xffff_2224 { 
                    println!("Entered kernel");
                    self.boot_status = BootStatus::IOSKernel;
                }
            }
            BootStatus::IOSKernel => {
                if self.cpu.read_fetch_pc() == 0x0001_0000 { 
                    println!("Entered foreign kernel stub");
                    self.boot_status = BootStatus::UserKernelStub;
                }
            }
            BootStatus::UserKernelStub=> {
                if self.cpu.read_fetch_pc() == 0xffff_0000 {
                    println!("Entered foreign kernel");
                    self.boot_status = BootStatus::UserKernel;
                }
            },
            _ => {},
        }
    }

    /// Write semihosting debug strings to stdout.
    pub fn svc_read(&mut self) {
        use ironic_core::cpu::mmu::prim::{TLBReq, Access};

        // On the SVC calls, r1 should contain a pointer to some buffer.
        // They might be virtual addresses, so we need to do an out-of-band
        // request to MMU code in order to resolve the actual location.
        let paddr = self.cpu.translate(
            TLBReq::new(self.cpu.reg.r[1], Access::Debug)
        );

        // Pull the buffer out of guest memory
        let mut line_buf = [0u8; 16];
        self.bus.write().unwrap().dma_read(paddr, &mut line_buf);

        let s = std::str::from_utf8(&line_buf).unwrap()
            .trim_matches(char::from(0));
        self.svc_buf += s;

        if self.svc_buf.find('\n').is_some() {
            let string: String = self.svc_buf.chars()
                .take(self.svc_buf.find('\n').unwrap()).collect();
            println!("SVC {}", string);
            self.svc_buf.clear();
        }
    }

    /// Log IOS syscalls to stdout.
    pub fn syscall_log(&mut self, opcd: u32) {
        println!("IOS syscall {:08x}, lr={:08x}", opcd, self.cpu.reg[Reg::Lr]);
    }

    /// Write the current instruction to stdout.
    pub fn dbg_print(&mut self) {
        let pc = self.cpu.read_fetch_pc();
        if self.cpu.dbg_on {
            if self.cpu.reg.cpsr.thumb() {
                let opcd = self.cpu.read16(pc);
                let inst = ThumbInst::decode(opcd);
                match inst {
                    ThumbInst::BlImmSuffix => return,
                    _ => {}
                }
                let name = format!("{:?}", ThumbInst::decode(opcd));
                println!("({:08x}) {:12} {:x?}", opcd, name, self.cpu.reg);
                //println!("{:?}", self.cpu.reg);
            } else {
                let opcd = self.cpu.read32(pc);
                let name = format!("{:?}", ArmInst::decode(opcd));
                println!("({:08x}) {:12} {:x?}", opcd, name, self.cpu.reg);
                //println!("{:?}", self.cpu.reg);
            };
        }
    }

    /// Patch containing a call to ThreadCancel()
    const THREAD_CANCEL_PATCH: [u8; 0x8] = [
        // e3a00000 mov     r0, #0
        //0xe3, 0xa0, 0x00, 0x00,
        // e3a01006 mov     r1, #6
        //0xe3, 0xa0, 0x10, 0x06,
        // e6000050 .word   0xe6000050
        0xe6, 0x00, 0x00, 0x50,
        // e12fff1e bx      lr
        0xe1, 0x2f, 0xff, 0x1e,
    ];

    /// Skyeye intentionally kills a bunch of threads, specifically NCD, KD,
    /// WL, and WD; presumably to avoid having to deal with emulating WLAN.
    pub fn hotpatch_check(&mut self) {
        use ironic_core::cpu::mmu::prim::{TLBReq, Access};
        if self.boot_status == BootStatus::IOSKernel {
            let pc = self.cpu.read_fetch_pc();
            let vaddr = match pc {
                0x13d9_0024 | // NCD
                0x13db_0024 | // KD
                0x13ed_0024 | // WL
                0x13eb_0024 => Some(pc), // WD
                _ => None
            };
            if vaddr.is_none() { 
                return; 
            } else {
                let paddr = self.cpu.translate(
                    TLBReq::new(vaddr.unwrap(), Access::Debug)
                );
                println!("DBG hotpatching module entrypoint {:08x}", paddr);
                println!("{:?}", self.cpu.reg);
                self.bus.write().unwrap().dma_write(paddr, 
                    &Self::THREAD_CANCEL_PATCH);
            }
        }
    }

    /// Do a single step of the CPU.
    pub fn cpu_step(&mut self) -> CpuRes {
        assert!((self.cpu.read_fetch_pc() & 1) == 0);

        // Sample the IRQ line. If the IRQ line is high and IRQs are not 
        // disabled in the CPSR, take an IRQ exception. 
        if !self.cpu.reg.cpsr.irq_disable() && self.cpu.irq_input {
            self.cpu.generate_exception(ExceptionType::Irq);
        }

        // Fetch/decode/execute an ARM or Thumb instruction depending on
        // the state of the Thumb flag in the CPSR.
        let disp_res = if self.cpu.reg.cpsr.thumb() {
            self.dbg_print();
            let opcd = self.cpu.read16(self.cpu.read_fetch_pc());
            let func = INTERP_LUT.thumb.lookup(opcd);
            func.0(&mut self.cpu, opcd)
        } else {
            self.dbg_print();
            let opcd = self.cpu.read32(self.cpu.read_fetch_pc());
            if self.cpu.reg.cond_pass(opcd) {
                let func = INTERP_LUT.arm.lookup(opcd);
                func.0(&mut self.cpu, opcd)
            } else {
                DispatchRes::CondFailed
            }
        };

        // Depending on the instruction, adjust the program counter
        let cpu_res = match disp_res {
            DispatchRes::RetireBranch => { CpuRes::StepOk },
            DispatchRes::RetireOk | 
            DispatchRes::CondFailed => {
                self.cpu.increment_pc(); 
                CpuRes::StepOk
            },

            // NOTE: Skyeye doesn't take SWI exceptions at all, but I wonder
            // why this is permissible. What does the hardware actually do?
            DispatchRes::Exception(e) => {
                if e == ExceptionType::Swi {
                    self.cpu.increment_pc();
                    CpuRes::Semihosting
                } else {
                    self.cpu.generate_exception(e);
                    CpuRes::StepException(e)
                }
            },

            DispatchRes::FatalErr => {
                println!("CPU halted at pc={:08x}", self.cpu.read_fetch_pc());
                CpuRes::HaltEmulation
            },
        };

        self.update_boot_status();
        cpu_res
    }
}

impl Backend for InterpBackend {
    fn run(&mut self) {
        if self.custom_kernel.is_some() {
            // Read the user supplied kernel file
            let mut kernel_file = fs::File::open(&self.custom_kernel.as_ref().unwrap()).unwrap();
            let mut kernel_bytes:Vec<u8> = Vec::with_capacity(kernel_file.metadata().expect("Kernel elf file-metadata").len() as usize);
            kernel_file.read_to_end(&mut kernel_bytes).expect("Failed to read custom kernel ELF");
            // Reuse the file for the ELF parser
            kernel_file.rewind().unwrap();
            let kernel_elf = match elf::File::open_stream(&mut kernel_file) {
                Ok(res) => res,
                Err(e)  => panic!("Custom Kernel ELF error: {:?}", e),
            };
            let headers = kernel_elf.phdrs;
            // We have a valid ELF (probably)
            let mut bus = self.bus.write().expect("RW bus access for custom kernel");
            // We are relying on the mirror being available
            // Or else we would be writing to mask ROM.
            bus.rom_disabled = true;
            bus.mirror_enabled = true;
            // A basic ELF loader
            for header in headers.iter() {
                if header.progtype == elf::types::ProgType(1) && header.filesz > 0 { // progtype 1 == PT_LOAD
                    let start = header.offset as usize;
                    let end = start + header.filesz as usize;
                    println!("CUSTOM KERNEL: LOADING offset: {:#10x}  phys addr: {:#10x} filesz: {:#10x}", header.offset, header.paddr, header.filesz);
                    bus.dma_write(header.paddr as u32, &kernel_bytes[start..end]);
                }
            }
            self.boot_status = BootStatus::UserKernelStub;
        }
        'outer: loop {
            'check_for_debug: {
                let maybe_packet = self.emu_recv.try_recv();
                if maybe_packet.is_ok(){
                    self.debugger_attached = true;
                    self.debug_cycles = 0;
                }
                if self.debugger_attached && self.debug_cycles == 0 {
                    if maybe_packet.is_err(){
                        continue 'outer;
                    }
                }
                else if self.debug_cycles > 0 {
                    self.debug_cycles -= 1;
                    break 'check_for_debug;
                }
                else {
                    break 'check_for_debug;
                }
                self.handle_debug_packet(maybe_packet.unwrap());
            }
            // Take ownership of the bus to deal with any pending tasks
            {
                let mut bus = self.bus.write().unwrap();
                bus.step(self.cpu_cycle);
                self.bus_cycle += 1;
                self.cpu.irq_input = bus.hlwd.irq.arm_irq_output;
            }

            // Before each CPU step, check if we need to patch any close code
            self.hotpatch_check();

            let res = self.cpu_step();
            match res {
                CpuRes::StepOk => {},
                CpuRes::HaltEmulation => break,
                CpuRes::StepException(e) => {
                    match e {
                        ExceptionType::Undef(_) => {},
                        ExceptionType::Irq => {},
                        _ => panic!("Unimplemented exception type {:?}", e),
                    }
                },
                CpuRes::Semihosting => {
                    self.svc_read();
                }
            }
            self.cpu_cycle += 1;
        }
        println!("CPU stopped at pc={:08x}", self.cpu.read_fetch_pc());
    }
}

impl InterpBackend {
    fn handle_debug_packet(&mut self, packet: DebugPacket){

        match packet.new_step {
            Some(new_step) => {
                self.debug_cycles = new_step;
                return;
            },
            None => {},
        }

        match packet.reg {
            Some(reg) => {
                match packet.write {
                    Some(write) => {
                        match write {
                            true => self.cpu.reg[reg] = packet.value.expect("DebugPacket reg->write->value"),
                            false => todo!("Debug reg read reply"),
                        }
                    },
                    None => panic!("DebugPacket Reg but no read/write bool"),
                }
                return;
            },
            None => {},
        }

        match packet.addr {
            Some(addr) => {
                match packet.write {
                    Some(write) => {
                        match write {
                            true => todo!("Bus write, u32 to [u8]"),
                            false => todo!("Bus read reply"),
                        }
                    },
                    None => panic!("DebugPacket Addr but no read/write bool"),
                }
                return;
            },
            None => {}
        }
        panic!("Unhndled debug packet {:#?}", &packet);
    }
}
