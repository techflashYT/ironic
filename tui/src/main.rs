use clap::{ArgEnum, Parser};

use ironic_core::bus::*;
use ironic_backend::interp::*;
use ironic_backend::back::*;
use ironic_backend::ppc::*;
use ironic_backend::debug::*;

use std::sync::mpsc::Receiver;
use std::sync::{Arc, RwLock, mpsc, mpsc::Sender};
use std::thread::Builder;
use std::env;

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
    /// Enable the PPC HLE server (default = True)
    #[clap(short, long)]
    ppc_hle: Option<bool>,
    /// Enable the Debug server (default = False)
    #[clap(short, long)]
    debug_server: Option<bool>,
}

fn dump_memory(bus: &Bus) {
    bus.sram0.dump("/tmp/sram0.bin");
    bus.sram1.dump("/tmp/sram1.bin");
    bus.mem1.dump("/tmp/mem1.bin");
    bus.mem2.dump("/tmp/mem2.bin");
}

fn main() {
    let args = Args::parse();
    let custom_kernel = args.custom_kernel.clone();
    let enable_ppc_hle = args.ppc_hle.unwrap_or(true);
    let enable_debug_server = args.debug_server.unwrap_or(false);


    // The bus is shared between any threads we spin up
    let bus = Arc::new(RwLock::new(Bus::new()));
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
        let ppc_thread = Builder::new().name("IpcThread".to_owned()).spawn(move || {
            let mut back = PpcBackend::new(ppc_bus);
            back.run();
        }).unwrap();
    }

    // Finally fork the DEBUG thread
    if enable_debug_server {
        let debug_bus = bus.clone();
        let debug_thread = Builder::new().name("DebugThread".to_owned()).spawn( move || {
            println!("DEBUG");
            let mut back = DebugBackend::new(dbg_send, dbg_recv);
            back.run();
        });
    }

    //ppc_thread.join().unwrap();
    emu_thread.join().unwrap();

    let bus_ref = bus.write().unwrap();
    dump_memory(&bus_ref);
    println!("Bus cycles elapsed: {}", bus_ref.cycle);
}

