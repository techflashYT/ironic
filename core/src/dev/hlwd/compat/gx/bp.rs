/// GX Blitting Processor (BP)
///
/// Accessed only by GX FIFO commands by the CP

/// Cross-unit events that can result from a BP register write, which the
/// caller (which has access to the rest of the GX pipeline) is responsible
/// for actually acting on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BpEvent {
    // Fire the PE Finish IRQ
    PeFinish,
}

#[derive(Debug, Clone)]
pub struct BlittingProcessor {
    // Internal register file, indexed by BP register address.
    pub regs: [u32; 0x100],
}
impl Default for BlittingProcessor {
    fn default() -> Self {
        Self { regs: [0; 0x100] }
    }
}
impl BlittingProcessor {
    pub fn read_reg(&self, addr: u8) -> u32 {
        self.regs[addr as usize]
    }
    pub fn write_reg(&mut self, addr: u8, val: u32) -> Option<BpEvent> {
        self.regs[addr as usize] = val;
        if addr == 0x45 && (val & 2) == 2 {
            return Some(BpEvent::PeFinish);
        }
        None
    }
}
