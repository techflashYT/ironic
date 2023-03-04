use anyhow::bail;

use crate::bus::prim::*;
use crate::bus::mmio::*;
use crate::bus::task::*;

/// The version reported when someone asks what SD Host Controller Version we are.
/// For some reason it's a 32 bit value, but only the upper 16 are used according to mini.
/// Of those, the lower 8 is the host controller version, the upper 8 is the vendor version.
/// 0 seems to work ok for now TODO: find out what real hardware reports.
const REPORTED_SDHC_VER:u32 = 0;

#[derive(Default)]
pub struct SDInterface {
    /// Destination address for DMA.
    pub dma_addr: u32,
    /// SDHC Block Control Register
    pub bcon: u32,
    /// SDHC Argument Register
    pub arg: u32,
    /// SDHC Mode Register
    pub mode: u32,
    /// SDHC Response Register
    pub resp: [u32; 4],
    /// SDHC Data Register
    pub data: u32,
    /// SDHC Status Register 1
    pub stat1: u32,
    /// SDHC Control Register 1
    pub ctrl1: u32,
    /// SDHC Control Register 2
    pub ctrl2: u32,
    /// SDHC Interrupt Status Register
    pub intstat: u32,
    /// SDHC Interrupt Flag Enable Register
    pub inten: u32,
    /// SDHC Interrupt Signal Enable Register
    pub intsen: u32,
    /// SDHC Status Register 2
    pub stat2: u32,
    /// SDHC Capabilities Register
    pub cap: u32,
    /// SDHC Maximum Current Capabilities Register 
    pub maxcap: u32,
}

impl MmioDevice for SDInterface {
    type Width = u32;
    fn read(&self, off: usize) -> anyhow::Result<BusPacket> {
        match off {
            0xfc => Ok(BusPacket::Word(REPORTED_SDHC_VER)),
            0x38 => Ok(BusPacket::Word(self.intsen)), // Interrupt Signal Enable

            // 2.2.17 Software Reset Register
            // Software will read this to know if the SDHC is in the middle of a reset previously requested from a write to this register
            // or as part of a load-store operation
            // We process resets synchronously, so always read out 0.
            0x2c => Ok(BusPacket::Word(0)),

            // 2.2.16 Timeout Control Register
            // We aren't going to need this (probably) so lets start at 0 and if anything freaks out we can deal with that as it comes
            0x34 => Ok(BusPacket::Word(0)),
            // TODO: add spec section
            // Capabilities register
            0x40 => Ok(BusPacket::Word(self.cap)),
            // Max Capabilities register
            0x48 => Ok(BusPacket::Word(self.maxcap)),
            _ => bail!("SDHC0 read at {off:x} unimplemented")
        }
    }
    fn write(&mut self, off: usize, val: u32) -> anyhow::Result<Option<BusTask>> {
        match off {
            0x38 => { // Interrupt Signal Enable
                self.intsen = val;
                Ok(None)
            },
            0x2c => { // 2.2.17 Software Reset Register
                let val = val >> 24; // Data comes in shifted
                if val == 0 {
                    return Ok(None); // Nothing we need to do
                }
                if val != 1 {
                    bail!("SDHC0 Software Reset only supports full reset at this time");
                }
                // Note: this is wrong!
                self.dma_addr = 0;
                self.bcon = 0;
                self.arg = 0;
                self.mode = 0;
                self.resp = [0;4];
                self.data = 0;
                self.stat1 = 0;
                self.ctrl1 = 0;
                self.ctrl2 = 0;
                self.intstat = 0;
                self.intsen = 0;
                self.stat2 = 0;
                self.cap = 0; // What are our default capabilities?
                self.maxcap = 0; // What are our maximum capabilities?
                Ok(None)
            },

            // 2.2.16 Timeout Control Register
            // Just swallow any writes for now.
            0x34 => Ok(None),

            // TODO: spec section
            0x40 => {
                self.cap = val;
                Ok(None)
            }
            0x48 => {
                self.maxcap = val;
                Ok(None)
            }
            _ => bail!("SDHC0 write {val:08x} at {off:x} unimpl")
        }
    }
}

#[derive(Default)]
pub struct WLANInterface {
    pub unk_24: u32,
    pub unk_40: u32,
    pub unk_fc: u32,
}

impl MmioDevice for WLANInterface {
    type Width = u32;
    fn read(&self, off: usize) -> anyhow::Result<BusPacket> {
        let val = match off {
            0x24 => self.unk_24,
            //0x24 => 0x0001_0000, //self.unk_24,
            //0x40 => 0x0040_0000, //self.unk_24,
            //0xfc => self.unk_fc,
            _ => { bail!("SDHC1 read at {off:x} unimpl"); },
        };
        Ok(BusPacket::Word(val))
    }
    fn write(&mut self, off: usize, val: u32) -> anyhow::Result<Option<BusTask>> {
        bail!("SDHC1 write {val:08x} at {off:x} unimpl")
    }
}
