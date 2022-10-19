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
    fn read(&self, off: usize) -> Result<BusPacket, String> {
        if off == 0xfc {
            return Ok(BusPacket::Word(REPORTED_SDHC_VER));
        }
        return Err(format!("SDHC0 read at {:x} unimplemented", off));
    }
    fn write(&mut self, off: usize, val: u32) -> Result<Option<BusTask>, String> {
        Err(format!("SDHC0 write {:08x} at {:x} unimpl", val, off))
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
    fn read(&self, off: usize) -> Result<BusPacket, String> {
        let val = match off {
            0x24 => self.unk_24,
            //0x24 => 0x0001_0000, //self.unk_24,
            //0x40 => 0x0040_0000, //self.unk_24,
            //0xfc => self.unk_fc,
            _ => { return Err(format!("SDHC1 read at {:x} unimpl", off)); },
        };
        Ok(BusPacket::Word(val))
    }
    fn write(&mut self, off: usize, val: u32) -> Result<Option<BusTask>, String> {
        Err(format!("SDHC1 write {:08x} at {:x} unimpl", val, off))
    }
}
