#![deny(unsafe_op_in_unsafe_fn)]

use ironic_core::bus::*;
use ironic_backend::interp::*;
use ironic_backend::back::*;
use ironic_backend::ppc::*;
use log::{debug, error};
use strum::VariantNames;

use std::panic;
use std::process;
use std::sync::{Arc, RwLock};
use std::thread::Builder;

use clap::{Parser, ValueEnum};

const LOGGING_EXAMPLE_TXT: &str = "
Example usage for --logging
 Set base log level to INFO: `--logging info`
 Set base log level to WARN but override SHA to DEBUG: --logging warn,sha:debug
 Set base log level to ERROR but override SHA to TRACE and AES to DEBUG: --logging ERROR,sha:trace,aes:DEBUG";

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
    #[clap(long, default_value="info")]
    logging: String,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    handle_logging_argument(args.logging)?;
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
            let ppc_early_on = custom_kernel.is_some() && enable_ppc_hle;
            Builder::new().name("EmuThread".to_owned()).spawn(move || {
                let mut back = InterpBackend::new(emu_bus, custom_kernel, ppc_early_on);
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
    match bus_ref.dump_memory("bin") {
        Ok(path) => {
            debug!(target: "Other", "Dumped ram to {}", path.to_string_lossy())
        }
        Err(e) => {
            error!(target: "Other", "Failed to dump ram: {e:?}");
        }
    }
    println!("Bus cycles elapsed: {}", bus_ref.cycle);
    process::exit(0);

}

#[derive(Debug, Clone, Copy, PartialEq, Eq, strum::AsRefStr, strum::Display, strum::EnumVariantNames, strum::EnumString)]
#[strum(ascii_case_insensitive)]
#[allow(non_camel_case_types, clippy::upper_case_acronyms)]
enum LogTarget {
    AES,
    DEBUG_PORT,
    EXI,
    HLWD,
    IPC,
    IRQ,
    NAND,
    OTP,
    PPC,
    SEEPROM,
    SHA,
    SYSCALL,
    xHCI,
    Other,
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
    }).chain(std::io::stdout());
    Ok(config.apply()?)
}

// I'm sorry for this monster
fn handle_logging_argument(log_string: String) -> anyhow::Result<()> {
    if !log_string.contains(',') {
        if let Ok(base_only) = log_string.parse::<log::LevelFilter>() {
            return setup_logger(base_only, &[]);
        }
        anyhow::bail!(
            "Failed to parse --logging argument: Base-level must be `off`, `error`, `warn`, `info`, `debug`, or `trace`. You supplied \"{log_string}\"{LOGGING_EXAMPLE_TXT}"
        );
    }
    let mut split = log_string.split(',');
    let maybe_base_level = split.next().expect("BUG parsing logging argument! First call to Split.next() should always be Some(...)");
    if let Ok(base_level) = maybe_base_level.parse::<log::LevelFilter>() {
        let mut target_level_overrides: Vec<(LogTarget, log::LevelFilter)> = Vec::new();
        for part in split {
            let mut inner = part.split(':');
            let maybe_target = inner.next().expect("BUG parsing logging argument! First call to Split.next() should always be Some(...)");
            if let Ok(target) = maybe_target.parse::<LogTarget>()
            {
                let maybe_specific_override = inner.next().expect("BUG parsing logging argument! If we got this far, this call to Split.next() should return Some(...)");
                if let Ok(specific_override) = maybe_specific_override.parse::<log::LevelFilter>()
                {
                    target_level_overrides.push((target, specific_override));
                }
                else {
                    //parsing level failed
                    anyhow::bail!(
                        "Failed to parse --logging argument: Log level for target: {target} is not valid. Log level must be be `off`, `error`, `warn`, `info`, `debug`, or `trace`. You supplied \"{maybe_specific_override}\"{LOGGING_EXAMPLE_TXT}"
                    )
                }
            }
            else {
                // parsing target failed
                anyhow::bail!(
                    "Failed to parse --logging argument: Not a valid logging subsystem/target! You specified: \"{maybe_target}\"\nValid options are:\n{:#?}{LOGGING_EXAMPLE_TXT}",
                    LogTarget::VARIANTS
                );
            }
        }
        return setup_logger(base_level, target_level_overrides.as_slice());
    }
    else {
        // Failed to parse base level
        anyhow::bail!(
            "Failed to parse --logging argument: Base-level must be `off`, `error`, `warn`, `info`, `debug`, or `trace`. You supplied \"{maybe_base_level}\"{LOGGING_EXAMPLE_TXT}"
        );
    }
}
