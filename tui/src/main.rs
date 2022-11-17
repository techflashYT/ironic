use clap::{Parser, ValueEnum};

use ironic_core::bus::*;
use ironic_backend::interp::*;
use ironic_backend::back::*;
use ironic_backend::ppc::*;

use std::panic;
use std::process;
use std::sync::{Arc, RwLock};
use std::thread::Builder;
use std::env::temp_dir;

/// User-specified backend type.
#[derive(Clone, Debug, ValueEnum)]
pub enum BackendType {
    Interp,
    JIT
}

impl std::fmt::Display for BackendType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Parser, Debug)]
struct Args {
    /// Emulator backend to use
    #[clap(short, long, default_value = "interp")]
    backend: BackendType,
    /// Path to a custom kernel ELF
    #[clap(short, long)]
    custom_kernel: Option<String>,
    /// Enable the PPC HLE server (default = False)
    #[clap(short, long)]
    ppc_hle: Option<bool>,
}

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

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let custom_kernel = args.custom_kernel.clone();
    let enable_ppc_hle = args.ppc_hle.unwrap_or(false);

    // The bus is shared between any threads we spin up
    let bus = match Bus::new() {
        Ok(val) => val,
        Err(reason) => {
            println!("Failed to construct emulator Bus: {reason}");
            process::exit(-1);
        }
    };

    let bus = Arc::new(RwLock::new(bus));

    // Fork off the backend thread
    let emu_bus = bus.clone();
    let emu_thread = match args.backend {
        BackendType::Interp => {
            Builder::new().name("EmuThread".to_owned()).spawn(move || {
                let mut back = InterpBackend::new(emu_bus, custom_kernel);
                if let Err(reason) = back.run() {
                    println!("InterpBackend returned an Err: {reason}");
                };
            }).unwrap()
        },
        _ => panic!("unimplemented backend"),
    };

    // Fork off the PPC HLE thread
    if enable_ppc_hle {
        let ppc_bus = bus.clone();
        let _ = Some(Builder::new().name("IpcThread".to_owned()).spawn(move || {
            let mut back = PpcBackend::new(ppc_bus);
            if let Err(reason) = back.run(){
                println!("PPC Backend returned an Err: {reason}");
            };
        }).unwrap());
    }

    let _ = emu_thread.join();

    let bus_ref = match bus.read() {
        Ok(val) => val,
        Err(reason) => {
            println!("Bus was poisoned during runtime: {reason}");
            reason.into_inner()
        }
    };
    dump_memory(&bus_ref)?;
    println!("Bus cycles elapsed: {}", bus_ref.cycle);
    process::exit(0);

}

