
use std::path::Path;
use std::fmt;
use std::fs::File;
use std::io::Read;
use std::io::Write;
use std::mem;

use anyhow::Context;
use anyhow::bail;

use crate::bus::prim::AccessWidth;

/// An abstract, generic memory device.
pub struct BigEndianMemory {
    /// Vector of bytes with the contents of this memory device.
    pub data: Vec<u8>,
}
impl BigEndianMemory {
    pub fn new(len: usize, init_fn: Option<&str>) -> anyhow::Result<Self> {
        let data = if let Some(filename) = init_fn {
            let mut f = File::open(filename)?;
            let mut data = vec![0u8; len];
            let _ = f.read(&mut data)?; // ignore partial read
            data
        } else {
            vec![0u8; len]
        };
        Ok(BigEndianMemory { data })
    }

    pub fn dump(&self, filename: &impl AsRef<Path>) -> anyhow::Result<()> {
        let mut f = File::create(filename).context("BigEndianMemory: Couldn't create dump file")?;
        let res = f.write(self.data.as_slice())?;
        println!("Dumped memory to {} ({res:?})", filename.as_ref().display());
        Ok(())
    }
}

impl fmt::Debug for BigEndianMemory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("BigEndianMemory").finish()
    }
}

/// Generic reads and writes.
impl BigEndianMemory {
    pub fn read<T: AccessWidth>(&self, off: usize) -> anyhow::Result<T> {
        let src_len = mem::size_of::<T>();
        if off + src_len > self.data.len() {
            bail!("Out-of-bounds read at {off:x}");
        }
        Ok(T::from_be_bytes(&self.data[off..off + src_len]))
    }
    pub fn write<T: AccessWidth>(&mut self, off: usize, val: T) -> anyhow::Result<()> {
        let data = val.to_be();
        let src_slice: &[u8] = data.as_bytes() ;
        if off + src_slice.len() > self.data.len() {
            bail!("Out-of-bounds write at {off:x}");
        }
        self.data[off..off + src_slice.len()].copy_from_slice(src_slice);
        Ok(())
    }
}

/// Bulk reads and writes.
impl BigEndianMemory {
    pub fn read_buf(&self, off: usize, dst: &mut [u8]) -> anyhow::Result<()> {
        if off + dst.len() > self.data.len() {
            bail!("OOB bulk read on BigEndianMemory, offset {off:x}");
        }
        dst.copy_from_slice(&self.data[off..off + dst.len()]);
        Ok(())
    }
    pub fn write_buf(&mut self, off: usize, src: &[u8]) -> anyhow::Result<()> {
        if off + src.len() > self.data.len() {
            bail!("OOB bulk write on BigEndianMemory, offset {off:x}");
        }
        self.data[off..off + src.len()].copy_from_slice(src);
        Ok(())
    }
    pub fn memset(&mut self, off: usize, len: usize, val: u8) -> anyhow::Result<()> {
        if off + len > self.data.len() {
            bail!("OOB memset on BigEndianMemory, offset {off:x}");
        }
        for d in &mut self.data[off..off+len] {
            *d = val;
        }
        Ok(())
    }
}
