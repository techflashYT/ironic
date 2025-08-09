use crate::bus::*;
use crate::bus::prim::*;
use crate::bus::mmio::*;
use crate::bus::task::*;

use anyhow::bail;
use log::{error, warn, info};

/// One-time programmable [fused] memory.
pub mod otp;
/// Interface to GPIO pins.
pub mod gpio;
/// Flipper-compatible interfaces.
pub mod compat;
/// GDDR3 interface.
pub mod ddr;
/// Interrupt controller.
pub mod irq;
/// Inter-processor communication.
pub mod ipc;

/// Legacy Serial interface
#[derive(Default, Debug, Clone)]
pub struct SerialInterface {
    pub outbuf: [u32; 4],
    pub inbuf_h: [u32; 4],
    pub inbuf_l: [u32; 4],
    pub poll: u32,
    pub comcsr: u32,
    pub sr: u32,
    pub exilk: u32
}
impl MmioDevice for SerialInterface {
    type Width = u32;
    fn read(&self, off: usize) -> anyhow::Result<BusPacket> {
        let val = match off {
            0x00 | 0x0c | 0x18 | 0x24 => self.outbuf[off / 0xc],
            0x04 | 0x10 | 0x1c | 0x28 => self.inbuf_h[off / 0xc],
            0x08 | 0x14 | 0x20 | 0x2c => self.inbuf_l[off / 0xc],
            0x30 => self.poll,
            0x34 => self.comcsr,
            0x38 => self.sr,
            0x3c => self.exilk,
            _ => { bail!("SI read from undefined offset {off:x}"); },
        };
        Ok(BusPacket::Word(val))
    }
    fn write(&mut self, off: usize, val: u32) -> anyhow::Result<Option<BusTask>> {
        match off {
            0x00 | 0x0c | 0x18 | 0x24 => self.outbuf[off / 0xc] = val,
            0x04 | 0x10 | 0x1c | 0x28 => self.inbuf_h[off / 0xc] = val,
            0x08 | 0x14 | 0x20 | 0x2c => self.inbuf_l[off / 0xc] = val,
            0x30 => self.poll = val,
            0x34 => self.comcsr = val,
            0x38 => self.sr = val,
            0x3c => self.exilk = val,
            _ => { bail!("SI write {val:08x} to undefined offset {off:x}"); },
        }
        Ok(None)
    }
}

/// Legacy Audio interface
#[derive(Default, Debug, Clone)]
pub struct AudioInterface {
    pub control: u32,
    pub volume: u32,
    pub sample_cnt: u32,
    pub int_timing: u32
}
impl MmioDevice for AudioInterface {
    type Width = u32;
    fn read(&self, off: usize) -> anyhow::Result<BusPacket> {
        let val = match off {
            0x00 => self.control,
            0x04 => self.volume,
            0x08 => self.sample_cnt,
            0x0c => self.int_timing,
            _ => { bail!("AI read from undefined offset {off:x}"); },
        };
        Ok(BusPacket::Word(val))
    }
    fn write(&mut self, off: usize, val: u32) -> anyhow::Result<Option<BusTask>> {
        match off {
            0x00 => self.control = val,
            0x04 => self.volume = val,
            0x08 => self.sample_cnt = val,
            0x0c => self.int_timing = val,
            _ => { bail!("AI write {val:08x} to undefined offset {off:x}"); },
        }
        Ok(None)
    }
}

/// Legacy Video interface
#[derive(Default, Debug, Clone)]
pub struct VideoInterface {
    pub clk: u16
}
impl MmioDeviceMultiWidth for VideoInterface {
    fn read8(&self, off: usize) -> anyhow::Result<BusPacket> {
        bail!("VI unsupported 8-bit read from offset {off:x}");
    }

    fn read16(&self, off: usize) -> anyhow::Result<BusPacket> {
        let val = match off {
            0x6c => self.clk,
            _ => { bail!("VI 16-bit read from undefined offset {off:x}"); },
        };
        Ok(BusPacket::Half(val))
    }
    fn read32(&self, off: usize) -> anyhow::Result<BusPacket> {
        let val = match off {
            _ => { bail!("VI 32-bit read from undefined offset {off:x}"); },
        };
        Ok(BusPacket::Word(val))
    }


    fn write8(&mut self, off: usize, val: u8) -> anyhow::Result<Option<BusTask>> {
        bail!("VI unsupported 8-bit write {val:02x} to offset {off:x}");
    }

    fn write16(&mut self, off: usize, val: u16) -> anyhow::Result<Option<BusTask>> {
        match off {
            0x6c => {
                if val == 1 {
                    info!(target: "VI", "Setting video clock to 54MHz");
                } else if val == 0 {
                    info!(target: "VI", "Setting video clock to 27MHz");
                } else {
                    bail!("Trying to set bogus VI clock speed {val:x}");
                }
                self.clk = val as u16;
            }
            _ => { bail!("VI 16-bit write {val:04x} to undefined offset {off:x}"); },
        }
        Ok(None)
    }

    fn write32(&mut self, off: usize, val: u32) -> anyhow::Result<Option<BusTask>> {
        match off {
            _ => { bail!("VI 32-bit write {val:08x} to undefined offset {off:x}"); },
        }
        Ok(None)
    }
}

/// Legacy Processor interface
#[derive(Default, Debug, Clone)]
pub struct ProcessorInterface {
    pub intsr: u32,
    pub intmr: u32,
    pub fifo_base_start: u32,
    pub fifo_base_end: u32,
    pub fifo_cur_write_ptr: u32,
    pub unk_18: u32,
    pub unk_1c: u32,
    pub unk_20: u32,
    pub reset: u32,
    pub unk_28: u32,
    pub unk_2c: u32,
}
impl MmioDevice for ProcessorInterface {
    type Width = u32;
    fn read(&self, off: usize) -> anyhow::Result<BusPacket> {
        let val = match off {
            0x00 => self.intsr,
            0x04 => self.intmr,
            0x08 => self.fifo_base_start,
            0x10 => self.fifo_base_end,
            0x14 => self.fifo_cur_write_ptr,
            0x18 => self.unk_18,
            0x1c => self.unk_1c,
            0x20 => self.unk_20,
            0x24 => self.reset,
            0x28 => self.unk_28,
            0x2c => self.unk_2c,
            _ => { bail!("PI read from undefined offset {off:x}"); },
        };
        Ok(BusPacket::Word(val))
    }
    fn write(&mut self, off: usize, val: u32) -> anyhow::Result<Option<BusTask>> {
        match off {
            0x00 => self.intsr = val,
            0x04 => self.intmr = val,
            0x08 => self.fifo_base_start = val,
            0x10 => self.fifo_base_end = val,
            0x14 => self.fifo_cur_write_ptr = val,
            0x18 => self.unk_18 = val,
            0x1c => self.unk_1c = val,
            0x20 => self.unk_20 = val,
            0x24 => self.reset = val,
            0x28 => self.unk_28 = val,
            0x2c => self.unk_2c = val,
            _ => { bail!("PI write {val:08x} to undefined offset {off:x}"); },
        }
        Ok(None)
    }
}

/// Legacy DSP
#[derive(Default, Debug, Clone)]
pub struct DSP {
    pub mailbox_in_h: u16,
    pub mailbox_in_l: u16,
    pub mailbox_out_h: u16,
    pub mailbox_out_l: u16,
    pub unk_08: u16,
    pub control_status: u16,
    pub unk_0c: u16,
    pub unk_0e: u16,
    pub unk_10: u16,
    pub ar_size: u16,
    pub unk_14: u16,
    pub ar_mode: u16,
    pub unk_18: u16,
    pub ar_refresh: u16,
    pub unk_1c: u16,
    pub unk_1e: u16,
    pub ar_dma_mmaddr_h: u16,
    pub ar_dma_mmaddr_l: u16,
    pub ar_dma_araddr_h: u16,
    pub ar_dma_araddr_l: u16,
    pub ar_dma_size_h: u16,
    pub ar_dma_size_l: u16,
    pub unk_2c: u16,
    pub unk_2e: u16,
    pub dma_start_addr_h: u16,
    pub dma_start_addr_l: u16,
    pub unk_34: u16,
    pub dma_control_length: u16,
    pub unk_38: u16,
    pub dma_bytes_left: u16
}
impl MmioDeviceMultiWidth for DSP {
    fn read8(&self, off: usize) -> anyhow::Result<BusPacket> {
        bail!("DSP unsupported 8-bit read from offset {off:x}");
    }
    fn read16(&self, off: usize) -> anyhow::Result<BusPacket> {
        let val = match off {
            0x00 => self.mailbox_in_h,
            0x02 => self.mailbox_in_l,
            0x04 => self.mailbox_out_h,
            0x06 => self.mailbox_out_l,
            0x08 => self.unk_08,
            0x0a => self.control_status,
            0x0c => self.unk_0c,
            0x0e => self.unk_0e,
            0x10 => self.unk_10,
            0x12 => self.ar_size,
            0x14 => self.unk_14,
            0x16 => self.ar_mode,
            0x18 => self.unk_18,
            0x1a => self.ar_refresh,
            0x1c => self.unk_1c,
            0x1e => self.unk_1e,
            0x20 => self.ar_dma_mmaddr_h,
            0x22 => self.ar_dma_mmaddr_l,
            0x24 => self.ar_dma_araddr_h,
            0x26 => self.ar_dma_araddr_l,
            0x28 => self.ar_dma_size_h,
            0x2a => self.ar_dma_size_l,
            0x2c => self.unk_2c,
            0x2e => self.unk_2e,
            0x30 => self.dma_start_addr_h,
            0x32 => self.dma_start_addr_l,
            0x34 => self.unk_34,
            0x36 => self.dma_control_length,
            0x38 => self.unk_38,
            0x3a => self.dma_bytes_left,
            _ => { bail!("DSP 16-bit read from undefined offset {off:x}"); },
        };
        Ok(BusPacket::Half(val))
    }
    fn read32(&self, off: usize) -> anyhow::Result<BusPacket> {
        let val: u32 = match off {
            0x00 => (self.mailbox_in_h as u32) << 16 & self.mailbox_in_l as u32,
            0x04 => (self.mailbox_out_h as u32) << 16 & self.mailbox_out_l as u32,
            0x20 => (self.ar_dma_mmaddr_h as u32) << 16 & self.ar_dma_mmaddr_l as u32,
            0x24 => (self.ar_dma_araddr_h as u32) << 16 & self.ar_dma_araddr_l as u32,
            0x30 => (self.dma_start_addr_h as u32) << 16 & self.dma_start_addr_l as u32,
            _ => { bail!("DSP 32-bit read from undefined offset {off:x}"); },
        };
        Ok(BusPacket::Word(val))
    }

    fn write8(&mut self, off: usize, val: u8) -> anyhow::Result<Option<BusTask>> {
        bail!("DSP unsupported 8-bit write {val:02x} to offset {off:x}");
    }
    fn write16(&mut self, off: usize, val: u16) -> anyhow::Result<Option<BusTask>> {
        match off {
            0x00 => self.mailbox_in_h = val,
            0x02 => self.mailbox_in_l = val,
            0x04 => self.mailbox_out_h = val,
            0x06 => self.mailbox_out_l = val,
            0x08 => self.unk_08 = val,
            0x0a => self.control_status = val & 0xfffe, // so that RES stays 0
            0x0c => self.unk_0c = val,
            0x0e => self.unk_0e = val,
            0x10 => self.unk_10 = val,
            0x12 => self.ar_size = val,
            0x14 => self.unk_14 = val,
            0x16 => self.ar_mode = val,
            0x18 => self.unk_18 = val,
            0x1a => self.ar_refresh = val,
            0x1c => self.unk_1c = val,
            0x1e => self.unk_1e = val,
            0x20 => self.ar_dma_mmaddr_h = val,
            0x22 => self.ar_dma_mmaddr_l = val,
            0x24 => self.ar_dma_araddr_h = val,
            0x26 => self.ar_dma_araddr_l = val,
            0x28 => self.ar_dma_size_h = val,
            0x2a => self.ar_dma_size_l = val,
            0x2c => self.unk_2c = val,
            0x2e => self.unk_2e = val,
            0x30 => self.dma_start_addr_h = val,
            0x32 => self.dma_start_addr_l = val,
            0x34 => self.unk_34 = val,
            0x36 => self.dma_control_length = val,
            0x38 => self.unk_38 = val,
            0x3a => self.dma_bytes_left = val,
            _ => { bail!("DSP 16-bit write {val:04x} to undefined offset {off:x}"); },
        }
        Ok(None)
    }
    fn write32(&mut self, off: usize, val: u32) -> anyhow::Result<Option<BusTask>> {
        let val = match off {
            0x00 => {
                self.mailbox_in_h = (val & 0xffff0000) as u16;
                self.mailbox_in_l = (val & 0x0000ffff) as u16;
            },
            0x04 => {
                self.mailbox_out_h = (val & 0xffff0000) as u16;
                self.mailbox_out_l = (val & 0x0000ffff) as u16;
            },
            0x20 => {
                self.ar_dma_mmaddr_h = (val & 0xffff0000) as u16;
                self.ar_dma_mmaddr_l = (val & 0x0000ffff) as u16;
            },
            0x24 => {
                self.ar_dma_araddr_h = (val & 0xffff0000) as u16;
                self.ar_dma_araddr_l = (val & 0x0000ffff) as u16;
            },
            0x28 => {
                self.ar_dma_size_h = (val & 0xffff0000) as u16;
                self.ar_dma_size_l = (val & 0x0000ffff) as u16;
            },
            0x30 => {
                self.dma_start_addr_h = (val & 0xffff0000) as u16;
                self.dma_start_addr_l = (val & 0x0000ffff) as u16;
            },
            _ => { bail!("DSP 32-bit write {val:08x} to undefined offset {off:x}"); },
        };
        Ok(None)
    }
}


/// The timer/alarm interface.
#[derive(Default, Debug, Clone)]
pub struct TimerInterface {
    pub timer: u32,
    pub alarm: u32,

    pub cpu_cycle_prev: usize,
}
impl TimerInterface {
    /// Timer period (some fraction of the CPU clock).
    pub const CPU_CLK_DIV: usize = 128;

    pub fn step(&mut self, current_cpu_cycle: usize) -> bool {
        // Fine as long as bus steps are interleaved with CPU steps I guess?
        if current_cpu_cycle - self.cpu_cycle_prev >= Self::CPU_CLK_DIV {
            self.timer += 1;
            self.cpu_cycle_prev = current_cpu_cycle;
            if self.timer == self.alarm {
                info!(target: "HLWD", "alarm IRQ {:08x}", self.timer);
                return true;
            } else {
                return false;
            }
        }
        false
    }
}

/// Various clocking registers.
#[derive(Debug, Clone)]
pub struct ClockInterface {
    pub sys: u32,       // 0x1b0
    pub sys_ext: u32,   // 0x1b4
    pub ddr: u32,       // 0x1bc
    pub ddr_ext: u32,   // 0x1c0
    pub vi_ext: u32,    // 0x1c8
    pub ai: u32,        // 0x1cc
    pub ai_ext: u32,    // 0x1d0
    pub usb_ext: u32,   // 0x1d8
}
impl Default for ClockInterface {
    fn default() -> Self {
        ClockInterface {
            sys: 0x0040_11c0,
            sys_ext: 0x1800_0018,
            ddr: 0,
            ddr_ext: 0,
            vi_ext: 0,
            ai: 0,
            ai_ext: 0,
            usb_ext: 0
        }
    }
}


/// Various bus control registers (?)
#[derive(Default, Debug, Clone)]
pub struct BusCtrlInterface {
    pub srnprot: u32,
    pub ahbprot: u32,
    pub aipprot: u32,
}

#[derive(Default, Debug, Clone)]
pub struct ArbCfgInterface {
    pub m0: u32,
    pub m1: u32,
    pub m2: u32,
    pub m3: u32,
    pub m4: u32,
    pub m5: u32,
    pub m6: u32,
    pub m7: u32,
    pub m8: u32,
    pub m9: u32,
    pub ma: u32,
    pub mb: u32,
    pub mc: u32,
    pub md: u32,
    pub me: u32,
    pub mf: u32,
    pub cpu: u32,
    pub dma: u32,
}
impl ArbCfgInterface {
    fn read_handler(&self, off: usize) -> anyhow::Result<u32>  {
        Ok(match off {
            0x00 => self.m0,
            0x04 => self.m1,
            0x08 => self.m2,
            0x0c => self.m3,
            0x10 => self.m4,
            0x14 => self.m5,
            0x18 => self.m6,
            0x1c => self.m7,
            0x20 => self.m8,
            0x24 => self.m9,
            0x30 => 0x0000_0400,
            0x34 => self.md,
            0x38 => 0x0000_0400,
            _ => { bail!("ARB_CFG read to undefined offset {off:x}"); },
        })
    }
    fn write_handler(&mut self, off: usize, val: u32) -> anyhow::Result<()> {
        match off {
            0x00 => self.m0 = val, 
            0x04 => self.m1 = val,
            0x08 => self.m2 = val, 
            0x0c => self.m3 = val, 
            0x10 => self.m4 = val, 
            0x14 => self.m5 = val, 
            0x18 => self.m6 = val, 
            0x1c => self.m7 = val, 
            0x20 => self.m8 = val, 
            0x24 => self.m9 = val, 
            0x30 => {},
            0x34 => self.md = val, 
            _ => { bail!("ARB_CFG write {val:08x} to undefined offset {off:x}"); },
        };
        Ok(())
    }
}


/// Unknown interface (probably related to the AHB).
#[derive(Default, Debug, Clone)]
pub struct AhbInterface {
    pub unk_08: u32,
    pub unk_10: u32,
}
impl MmioDevice for AhbInterface {
    type Width = u32;
    fn read(&self, off: usize) -> anyhow::Result<BusPacket> {
        let val = match off {
            0x08 => 0,
            0x10 => self.unk_10,
            0x3fe4 => {
                error!(target: "HLWD", "FIXME: AHB Read from weird (0x3fe4) - returning 0");
                0
            }
            _ => { bail!("AHB read to undefined offset {off:x}"); },
        };
        Ok(BusPacket::Word(val))
    }
    fn write(&mut self, off: usize, val: u32) -> anyhow::Result<Option<BusTask>> {
        match off {
            0x08 => {
                self.unk_08 = val;
            },
            0x10 => self.unk_10 = val,
            0x3fe4..=0x3fe8 => {
                error!(target: "HLWD", "FIXME: AHB write to weird ({off:x}) offset: {val:x}")
            }
            _ => { bail!("AHB write {val:08x} to undefined offset {off:x}"); },
        }
        Ok(None)
    }
}


/// Hollywood memory-mapped registers
pub struct Hollywood {
    pub task: Option<HlwdTask>,

    pub si: SerialInterface,
    pub vi: VideoInterface,
    pub ai: AudioInterface,
    pub pi: ProcessorInterface,
    pub dsp: DSP,

    pub ipc: ipc::IpcInterface,
    pub timer: TimerInterface,
    pub busctrl: BusCtrlInterface,
    pub pll: ClockInterface,
    pub otp: otp::OtpInterface,
    pub gpio: gpio::GpioInterface,
    pub irq: irq::IrqInterface,

    pub exi: compat::exi::EXInterface,
    pub di: compat::di::DriveInterface,
    pub mi: compat::mem::MemInterface,
    pub ahb: AhbInterface,
    pub ddr: ddr::DdrInterface,

    pub arb: ArbCfgInterface,
    pub reset_ahb: u32,
    pub clocks: u32,
    pub resets: u32,
    pub compat: u32,
    pub spare0: u32,
    pub spare1: u32,

    pub io_str_ctrl0: u32,
    pub io_str_ctrl1: u32,

    pub usb_frc_rst: u32,
    pub ppc_on: bool,
}
impl Hollywood {
    pub fn new() -> anyhow::Result<Self> {
        // TODO: Where do the initial values for these registers matter?
        Ok(Hollywood {
            task: None,
            si: SerialInterface::default(),
            vi: VideoInterface::default(),
            ai: AudioInterface::default(),
            pi: ProcessorInterface::default(),
            dsp: DSP::default(),
            ipc: ipc::IpcInterface::new(),
            busctrl: BusCtrlInterface::default(),
            timer: TimerInterface::default(),
            irq: irq::IrqInterface::default(),
            otp: otp::OtpInterface::new()?,
            gpio: gpio::GpioInterface::new()?,
            pll: ClockInterface::default(),

            ahb: AhbInterface::default(),
            di: compat::di::DriveInterface::default(),
            exi: compat::exi::EXInterface::new(),
            mi: compat::mem::MemInterface::new(),
            ddr: ddr::DdrInterface::new(),

            usb_frc_rst: 0,
            arb: ArbCfgInterface::default(),
            reset_ahb: 0x0000_ffff,
            resets: 0x0000_0008,
            clocks: 0,
            compat: 0,
            spare0: 0,
            spare1: 0,
            io_str_ctrl0: 0,
            io_str_ctrl1: 0,
            ppc_on: false,
        })
    }
}


impl MmioDevice for Hollywood {
    type Width = u32;
    fn read(&self, off: usize) -> anyhow::Result<BusPacket> {
        let val = match off {
            0x000..=0x00c   => self.ipc.read_handler(off)?,
            0x010           => self.timer.timer,
            0x014           => self.timer.alarm,
            0x030..=0x05c   => self.irq.read_handler(off - 0x30)?,
            0x060           => self.busctrl.srnprot,
            0x064           => self.busctrl.ahbprot,
            0x070           => self.busctrl.aipprot,
            0x0c0..=0x0d8   => self.gpio.ppc.read_handler(off - 0xc0)?,
            0x0dc..=0x0fc   => self.gpio.arm.read_handler(off - 0xdc)?,
            0x100..=0x13c   => self.arb.read_handler(off - 0x100)?,
            0x180           => self.compat,
            0x184           => self.reset_ahb,
            0x188           => self.spare0,
            0x18c           => self.spare1,
            0x190           => self.clocks,
            0x194           => self.resets,
            0x1b0           => self.pll.sys,
            0x1b4           => self.pll.sys_ext,
            0x1bc           => self.pll.ddr,
            0x1c0           => self.pll.ddr_ext,
            0x1c8           => self.pll.vi_ext,
            0x1cc           => self.pll.ai,
            0x1d0           => self.pll.ai_ext,
            0x1d8           => self.pll.usb_ext,
            0x1e0           => self.io_str_ctrl0,
            0x1e4           => self.io_str_ctrl1,
            0x1ec           => self.otp.cmd,
            0x1f0           => self.otp.out,
            0x214           => 0x0000_0000,
            _ => { bail!("Unimplemented Hollywood read at {off:x}"); },
        };
        Ok(BusPacket::Word(val))
    }

    fn write(&mut self, off: usize, val: u32) -> anyhow::Result<Option<BusTask>> {
        match off {
            0x000..=0x00c => self.ipc.write_handler(off, val)?,
            0x014 => {
                info!(target: "HLWD", "alarm={val:08x} (timer={:08x})", self.timer.timer);
                self.timer.alarm = val;
            },
            0x030..=0x05c => self.irq.write_handler(off - 0x30, val)?,
            0x060 => {
                info!(target: "HLWD", "SRNPROT={val:08x}");
                let diff = self.busctrl.srnprot ^ val;
                self.busctrl.srnprot = val;
                let task = if (diff & 0x0000_0020) != 0 {
                    Some(BusTask::SetMirrorEnabled((val & 0x0000_0020) != 0))
                } else {
                    None
                };
                return Ok(task);
            }
            0x064 => self.busctrl.ahbprot = val,
            0x070 => self.busctrl.aipprot = val,
            0x088 => self.usb_frc_rst = val,
            0x0c0..=0x0d8 => self.gpio.ppc.write_handler(off - 0xc0, val)?,
            0x0dc..=0x0fc => {
                self.task = self.gpio.arm.write_handler(off - 0xdc, val)?;
            },
            0x100..=0x13c => self.arb.write_handler(off - 0x100, val)?,
            0x180 => self.compat = val,
            0x184 => {
                info!(target: "HLWD", "reset_ahb={val:08x}");
                self.reset_ahb = val;
            },
            0x188 => {
                self.spare0 = val;
                // AHB flushing code seems to check these bits?
                if (val & 0x0001_0000) != 0 {
                    self.spare1 &= 0xffff_fff6;
                } else {
                    self.spare1 |= 0x0000_0009;
                }
            },
            0x18c => {
                info!(target: "HLWD", "SPARE1={val:08x}");
                // Potentially toggle the boot ROM mapping
                let diff = self.spare1 ^ val;
                self.spare1 = val;
                let task = if (diff & 0x0000_1000) != 0 {
                    Some(BusTask::SetRomDisabled((val & 0x0000_1000) != 0))
                } else { 
                    None
                };
                return Ok(task);
            },
            0x190 => {
                info!(target: "HLWD", "clocks={val:08x}");
                // Perform sanity checks on the clock change.
                // Doing things in the wrong order on a real hardware results in a crash.
                // The correct order to switch to 486/162 is exactly:
                // 1. Set FX (bit 0) in HW_CLOCKS
                // 2. Clear RSTB_DSKPLL (bit 3) in HW_RESETS
                // 3. Set SPEED (bit 1) in HW_CLOCKS
                // 4. Set RSTB_DSKPLL (bit 3) in HW_RESETS
                // 5. Clear FX (bit 0) in HW_CLOCKS
                // ... and very similar (clear SPEED instead of setting it) to switch
                // back to 729/243.
                //
                // Source: https://wiibrew.org/wiki/Broadway/Clock_Speed_Control
                //
                // So, this should enforce that order here, and complain if the
                // software tries to do it incorrectly.
                // This would all have real side-effects on the actual speed in
                // reality, but I'm not sure that's possible here, so it's not attempted.
                // TODO: This might also affect other registers (PLLs?)

                // Prevent modifying SPEED whilst either FX is not *already* cleared, or whilst
                // RSTB_DSKPLL is not *already* cleared
                if (val & 0x0000_0002) != (self.clocks & 0x0000_0002) {
                    // We're modifying SPEED, ensure valid state
                    if (self.clocks & 0x0000_0001) == 0 {
                        bail!("Trying to modify HW_CLOCKS[SPEED] whilst HW_CLOCKS[FX] is unset, which would crash the system");
                    }

                    if (self.resets & 0x0000_0008) == 1 {
                        bail!("Trying to modify HW_CLOCKS[SPEED] whilst HW_RESETS[RSTB_DSKPLL] is set, which would crash the system");
                    }

                    if (val & 0x0000_0002) == 0 {
                        warn!(target: "HLWD", "Switching clocks to 729MHz (PPC core) / 243MHz (Bus / Hollywood); this is not reflected in actual timing");
                    } else {
                        warn!(target: "HLWD", "Switching clocks to 486MHz (PPC core) / 162MHz (Bus / Hollywood); this is not reflected in actual timing");
                    }
                }

                self.clocks = val;
            },
            0x194 => {
                let diff = self.resets ^ val;
                if diff & 0x0000_0030 != 0 {
                    if (val & 0x0000_0020 != 0) && (val & 0x0000_0010 != 0) {
                        info!(target: "HLWD", "Broadway power on");
                        self.ppc_on = true;
                    } else {
                        info!(target: "HLWD", "Broadway power off");
                        self.ppc_on = false;
                    }
                }

                info!(target: "HLWD", "resets={val:08x}");

                // Prevent invalid state for clocking, see comment of HW_CLOCKS
                // Prevent RSTB_DSKPLL (bit 3) from being cleared whilst FX is unset
                if (val & 0x0000_0008) == 0 && (self.clocks & 0x0000_0001) == 0 {
                    bail!("Trying to clear HW_RESETS[RSTB_DSKPLL] whilst HW_CLOCKS[FX] is unset, which would crash the system");
                }

                self.resets = val;
            },
            0x1b0 => self.pll.sys = val,
            0x1b4 => self.pll.sys_ext = val,
            0x1bc => self.pll.ddr = val,
            0x1c0 => self.pll.ddr_ext = val,
            0x1c8 => self.pll.vi_ext = val,
            0x1cc => self.pll.ai = val,
            0x1d0 => self.pll.ai_ext = val,
            0x1d8 => self.pll.usb_ext = val,
            0x1e0 => self.io_str_ctrl0 = val,
            0x1e4 => self.io_str_ctrl1 = val,
            0x1ec => self.otp.write_handler(val),
            _ => { bail!("Unimplemented Hollywood write at {off:x}"); },
        }
        Ok(None)
    }

}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum HlwdTask { 
    GpioOutput(u32) 
}

impl Bus {
    pub fn handle_step_hlwd(&mut self, cpu_cycle: usize) -> anyhow::Result<()> {

        // Potentially assert an IRQ
        let timer_irq = self.hlwd.timer.step(cpu_cycle);
        if timer_irq {
            self.hlwd.irq.assert(irq::HollywoodIrq::Timer);
        }
        if self.hlwd.ipc.assert_ppc_irq() {
            self.hlwd.irq.assert(irq::HollywoodIrq::PpcIpc);
        }
        if self.hlwd.ipc.assert_arm_irq() {
            self.hlwd.irq.assert(irq::HollywoodIrq::ArmIpc);
        }

        if self.hlwd.task.is_some() {
            match self.hlwd.task.unwrap() {
                HlwdTask::GpioOutput(val) => self.hlwd.gpio.handle_output(val)?,
            }
            self.hlwd.task = None;
        }
        Ok(())
    }
}


