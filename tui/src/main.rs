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
        write!(f, "{self:?}")
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
    /// Define log levels for the program
    #[clap(long)]
    logging: Option<String>,
}

fn dump_memory(bus: &Bus) -> anyhow::Result<()> {
    let dir = temp_dir();

    let mut sram0_dir = temp_dir();
    sram0_dir.push("sram0");
    bus.sram0.dump(&sram0_dir)?;

    let mut sram1_dir = temp_dir();
    sram1_dir.push("sram1");
    bus.sram1.dump(&sram1_dir)?;

    let mut mem1_dir = temp_dir();
    mem1_dir.push("mem1");
    bus.mem1.dump(&mem1_dir)?;

    let mut mem2_dir = dir;
    mem2_dir.push("mem2");
    bus.mem2.dump(&mem2_dir)?;
    Ok(())
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    handle_logging_argument(args.logging.unwrap_or("error".to_string()))?;
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
#[allow(non_camel_case_types, clippy::upper_case_acronyms)]
enum LogTarget {
    AES,
    EXI,
    NAND,
    SHA,
    xHCI,
    Other,
}

impl std::fmt::Display for LogTarget {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AES        => f.write_str("AES"),
            Self::EXI        => f.write_str("EXI"),
            Self::NAND       => f.write_str("NAND"),
            Self::SHA        => f.write_str("SHA"),
            Self::xHCI       => f.write_str("xHCI"),
            Self::Other      => f.write_str("Other"),
        }
    }
}

impl std::str::FromStr for LogTarget {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "aes" => Ok(Self::AES),
            "exi" => Ok(Self::EXI),
            "nand" => Ok(Self::NAND),
            "sha" => Ok(Self::SHA),
            "xhci" => Ok(Self::xHCI),
            "other" => Ok(Self::Other),
            _ => Err(anyhow::anyhow!("No such variant: {s}"))
        }
    }
}

fn setup_logger(base_level: log::LevelFilter, target_level_overrides: &[(LogTarget, log::LevelFilter)]) -> anyhow::Result<()> {
    use fern::colors::{Color, ColoredLevelConfig};
    let colors = ColoredLevelConfig::default().debug(Color::Cyan).trace(Color::BrightCyan);
    let mut config = fern::Dispatch::new().level(base_level);
    for specific_override in target_level_overrides {
        config = config.level_for(specific_override.0.to_string(), specific_override.1);
    }
    config = config.format(move |out, message, record| {
        out.finish(format_args!(
            "[{}][{}] {}",
            record.target(),
            colors.color(record.level()),
            message
        ))
    }).chain(std::io::stderr());
    Ok(config.apply()?)
}

fn handle_logging_argument(log_string: String) -> anyhow::Result<()> {
    if !log_string.contains(',') {
        let base_only = log_string.parse::<log::LevelFilter>()?;
        return setup_logger(base_only, &[]);
    }
    let mut split = log_string.split(',');
    let base_level = split.next().ok_or_else(|| anyhow::anyhow!("Log string improper format"))?.parse::<log::LevelFilter>()?;
    let mut target_level_overrides: Vec<(LogTarget, log::LevelFilter)> = Vec::new();
    for part in split {
        let mut inner = part.split(':');
        let target = inner.next().ok_or_else(|| anyhow::anyhow!("Log string improper format"))?.parse::<LogTarget>()?;
        let specific_override = inner.next().ok_or_else(|| anyhow::anyhow!("Log string improper format"))?.parse::<log::LevelFilter>()?;
        target_level_overrides.push((target, specific_override));
    }
    setup_logger(base_level, target_level_overrides.as_slice())
}