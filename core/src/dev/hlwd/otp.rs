
use std::io::Read;
use std::fs::File;
use crate::bus::prim::AccessWidth;

/// One-time programmable memory device/interface.
pub struct OtpInterface {
    /// Bits fused to the device.
    data: [u8; 0x80],
    /// Command register.
    pub cmd: u32,
    /// Command output register.
    pub out: u32,
}
impl OtpInterface {
    pub fn new() -> Result<Self, std::io::Error> {
        let mut f = File::open("otp.bin")?;
        let mut otp = OtpInterface { data: [0; 0x80], cmd: 0, out: 0 };
        f.read_exact(&mut otp.data)?;
        Ok(otp)
    }
}

impl OtpInterface {
    /// Read a word from OTP memory.
    pub fn read(&self, word_idx: usize) -> u32 {
        let off = word_idx * 4;
        assert!(off + 4 <= self.data.len());
        //println!("OTP read {:08x} @ idx={:x}", res, word_idx);
        AccessWidth::from_be_bytes(&self.data[off..off+4])
    }

    /// Handle a command request.
    pub fn write_handler(&mut self, cmd: u32) {
        if cmd & 0x8000_0000 != 0 {
            let addr = (cmd & 0x0000_001f) as usize;
            let out = self.read(addr);
            self.cmd = cmd;
            self.out = out;
        }
    }
}
