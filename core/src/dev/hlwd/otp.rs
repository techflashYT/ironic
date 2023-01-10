
use std::io::Read;
use std::fs::File;
use crate::bus::prim::AccessWidth;

use log::{debug, trace, log_enabled};

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
        if log_enabled!(target: "OTP", log::Level::Trace) {
            trace!(target: "OTP", "Initial data: {} bytes", otp.data.len());
            for (word_idx, chunk )in otp.data.chunks(4).enumerate() {
                trace!(target: "OTP", "Word {word_idx:02}: {:02x}{:02x}{:02x}{:02x}", chunk[0], chunk[1], chunk[2], chunk[3]);
            }
            trace!(target: "OTP", "End Initial data");
        }
        Ok(otp)
    }
}

impl OtpInterface {
    /// Read a word from OTP memory.
    pub fn read(&self, word_idx: usize) -> u32 {
        let off = word_idx * 4;
        assert!(off + 4 <= self.data.len());
        if log_enabled!(target: "OTP", log::Level::Debug) {
            let mut tmp = [0u8;4];
            tmp.copy_from_slice(&self.data[off..off+4]);
            debug!(target: "OTP", "read {:02x}{:02x}{:02x}{:02x} @ idx={:x}", tmp[0], tmp[1], tmp[2], tmp[3], word_idx);
        }
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
