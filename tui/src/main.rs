use clap::{ArgEnum, Parser};

use ironic_core::bus::*;
use ironic_backend::interp::*;
use ironic_backend::back::*;
use ironic_backend::ppc::*;
use ironic_backend::debug::*;

use std::panic;
use std::process;
use std::sync::mpsc::Receiver;
use std::sync::{Arc, RwLock, mpsc, mpsc::Sender};
use std::thread::Builder;
use std::env::temp_dir;

/// User-specified backend type.
#[derive(ArgEnum, Clone, Debug)]
pub enum BackendType {
    Interp,
    JIT
}

#[derive(Parser, Debug)]
struct Args {
    /// Emulator backend to use
    #[clap(short, long, arg_enum, default_value_t = BackendType::Interp)]
    backend: BackendType,
    /// Path to a custom kernel ELF
    #[clap(short, long)]
    custom_kernel: Option<String>,
    /// Enable the PPC HLE server (default = False)
    #[clap(short, long)]
    ppc_hle: Option<bool>,
    /// Enable the Debug server (default = False)
    #[clap(short, long)]
    debug_server: Option<bool>,
}

fn dump_memory(bus: &Bus) {
    let dir = temp_dir();

    let mut sram0_dir = temp_dir().clone();
    sram0_dir.push("sram0");
    bus.sram0.dump(&sram0_dir);

    let mut sram1_dir = temp_dir().clone();
    sram1_dir.push("sram1");
    bus.sram1.dump(&sram1_dir);

    let mut mem1_dir = temp_dir().clone();
    mem1_dir.push("mem1");
    bus.mem1.dump(&mem1_dir);

    let mut mem2_dir = dir;
    mem2_dir.push("mem2");
    bus.mem2.dump(&mem2_dir);
}

fn main() {
    let args = Args::parse();
    let custom_kernel = args.custom_kernel.clone();
    let enable_ppc_hle = args.ppc_hle.unwrap_or(false);
    let enable_debug_server = args.debug_server.unwrap_or(false);

    // The bus is shared between any threads we spin up
    let bus = match Bus::new() {
        Ok(val) => val,
        Err(reason) => {
            println!("Failed to construct emulator Bus: {}", reason);
            process::exit(-1);
        }
    };

    let bus = Arc::new(RwLock::new(bus));

    let (dbg_send, emu_recv): (Sender<DebugPacket>, Receiver<DebugPacket>) = mpsc::channel();
    let (emu_send, dbg_recv): (Sender<DebugPacket>, Receiver<DebugPacket>) = mpsc::channel();

    // Fork off the backend thread
    let emu_bus = bus.clone();
    let emu_thread = match args.backend {
        BackendType::Interp => {
            Builder::new().name("EmuThread".to_owned()).spawn(move || {
                let mut back = InterpBackend::new(emu_bus, custom_kernel, emu_send, emu_recv);
                back.run();
            }).unwrap()
        },
        _ => panic!("unimplemented backend"),
    };

    // Fork off the PPC HLE thread
    if enable_ppc_hle {
        let ppc_bus = bus.clone();
        let _ = Some(Builder::new().name("IpcThread".to_owned()).spawn(move || {
            let mut back = PpcBackend::new(ppc_bus);
            back.run();
        }).unwrap());
    }

    // Finally fork the DEBUG thread
    if enable_debug_server {
        let _ = Some(Builder::new().name("DebugThread".to_owned()).spawn( move || {
            let mut back = DebugBackend::new(dbg_send, dbg_recv);
            back.run();
        }).unwrap());
    }

    let _ = emu_thread.join();

    match bus.write() {
        Ok(bus_ref) => {
            dump_memory(&bus_ref);
            println!("Bus cycles elapsed: {}", bus_ref.cycle);
        },
        Err(reason) => {
            println!("Could not dump Bus memory because it is poisoned: {}", reason);
        }
    };
    process::exit(0);

}

