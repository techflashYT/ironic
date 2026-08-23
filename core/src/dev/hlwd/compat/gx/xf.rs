/// GX Transform Unit (XF)

#[derive(Debug, Clone)]
pub struct TransformUnit {
    /// Internal register file, indexed by XF register address
    pub regs: [u32; 0x1058],
}
impl Default for TransformUnit {
    fn default() -> Self {
        Self { regs: [0; 0x1058] }
    }
}
impl TransformUnit {
    pub fn read_reg(&self, addr: u16) -> u32 {
        self.regs[addr as usize]
    }
    pub fn write_reg(&mut self, addr: u16, val: u32) {
        self.regs[addr as usize] = val;
    }
}
