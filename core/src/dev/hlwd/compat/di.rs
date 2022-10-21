use anyhow::bail;

use crate::bus::mmio::*;
use crate::bus::prim::*;
use crate::bus::task::*;

/// Legacy disc drive interface.
#[derive(Default, Debug, Clone)]
#[allow(dead_code)]
pub struct DriveInterface {
    disr: u32,
    dicvr: u32,
    dicmdbuf: [u32; 3],
    dimar: u32,
    dilength: u32,
    dicr: u32,
    diimmbuf: u32,
    dicfg: u32,
}
impl MmioDevice for DriveInterface {
    type Width = u32;
    fn read(&self, off: usize) -> anyhow::Result<BusPacket> {
        let val = match off {
            0x00 => self.disr,
            0x04 => self.dicvr,
            0x24 => self.dicfg,
            _ => { bail!("DI read to undefined offset {off:x}"); },
        };
        Ok(BusPacket::Word(val))
    }
    fn write(&mut self, off: usize, val: u32) -> anyhow::Result<Option<BusTask>> {
        match off {
            0x00 => self.disr = val,
            0x04 => self.dicvr = val,
            _ => { bail!("DI write {val:08x?} to undefined offset {off:x}"); },
        }
        Ok(None)
    }
}


