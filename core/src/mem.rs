
use std::path::Path;
use std::fmt;
use std::fs::File;
use std::io::Read;
use std::io::Write;
use std::mem;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering::Relaxed;



use anyhow::{bail, Context};
use log::{error, debug};
use bincode::{config, Decode, Encode};

use crate::bus::prim::AccessWidth;

/// An abstract, generic memory device.
pub struct BigEndianMemory {
    /// Vector of bytes with the contents of this memory device.
    pub data: Vec<u8>,
    /// Hash of initial data (used for write tracking)
    hash: u32,
    /// Holds all writes so they can be replayed next time the emulator launches
    writes: Option<Vec<MemoryPatch>>,
    /// write_index
    pub write_index: u8,
    already_wrote: AtomicBool,
}
impl BigEndianMemory {
    pub fn new(len: usize, init_fn: Option<&str>, track_writes: bool) -> anyhow::Result<Self> {
        let hash: u32;
        let data = if let Some(filename) = init_fn {
            let mut f = File::open(filename)?;
            let mut data = vec![0u8; len];
            let _ = f.read(&mut data)?; // ignore partial read
            hash = crc32fast::hash(&data);
            data
        } else {
            hash = 0xDEADC0DE;
            vec![0u8; len]
        };
        let writes: Option<Vec<MemoryPatch>> = if track_writes {
            debug!(target: "MEMSAVE", "BEMemory: Writes Enabled, hash: {hash}");
            Some(Vec::new())
        }
        else {
            None
        };
        let mut res = BigEndianMemory { data, hash, writes, write_index: 0, already_wrote: AtomicBool::new(true)};
        if track_writes {
            if let Ok((write_index, mpfs)) = BigEndianMemory::get_patchfiles(hash) {
                res.write_index = write_index.checked_add(1).unwrap();
                for mpf in mpfs {
                    res.patch(mpf)?;
                }
            }
        }
        Ok(res)
    }

    /// Get the patches to apply persistent writes
    /// Returns the highest numbered patch file, so this time around we can write to n+1
    fn get_patchfiles(hash: u32) -> anyhow::Result<(u8, Vec<MemoryPatchFile>)> {
        let dir = match std::fs::read_dir(format!("./saved-writes/{hash}/")) {
            Ok(dir) => dir,
            Err(err) => {
                // handle no directory by creating it and trying again
                match err.raw_os_error() {
                    Some(2) => {
                        std::fs::create_dir_all(format!("./saved-writes/{hash}/"))?;
                        std::fs::read_dir(format!("./saved-writes/{hash}/"))?
                    },
                    Some(_) | None => { return Err(err).context(format!("Failed to open directory ./saved-writes/{hash}/ for get_patchfiles")) }
                }
            },
        };
        // Get all files saved to the directory that matches our hash and who's filenames can be parsed into a u8
        // deserialize them, and save their filename with
        let mut pfs: Vec<(u8, MemoryPatchFile)> = dir.filter_map(|maybe_entry|{
            if let Ok(entry) = maybe_entry {
                debug!(target: "MEMSAVE", "Candidate entry: {:?}", entry.path());
                if let Ok(num) = entry.file_name().to_string_lossy().into_owned().parse::<u8>() {
                    let mpf = MemoryPatchFile::from_file(entry.path());
                    debug!(target: "MEMSAVE", "Found MemoryPatchFile: {num} {:?}", entry.path());
                    Some((num, mpf))
                }
                else {
                    None
                }
            }
            else {
                error!(target: "MEMSAVE", "Unable to read ./saved-writes/{hash}/");
                None
            }
        }).collect();
        // Sort by filename
        pfs.sort_by(|a, b| {
            a.0.cmp(&b.0)
        });
        let max = pfs.iter().map(|x|x.0).max().unwrap_or(0);
        Ok((max, pfs.iter().map(|x|x.1.clone()).collect()))
    }

    pub fn dump(&self, filename: &impl AsRef<Path>) -> anyhow::Result<()> {
        let filename = filename.as_ref();
        let mut f = File::create(filename).context(format!("BigEndianMemory: Couldn't create dump file: {}", filename.to_string_lossy()))?;
        let res = f.write(self.data.as_slice())?;
        debug!(target: "Other", "Dumped memory to {} ({res:?})", filename.display());
        Ok(())
    }

    fn patch(&mut self, patchfile: MemoryPatchFile) -> anyhow::Result<()> {
        if self.hash != patchfile.hash {
            bail!("Mismatched patch file!");
        }
        let temp = std::mem::take(&mut self.writes); // Or else the subsequent calls to write_buf will double-count these writes.
        for range in patchfile.ranges {
            debug!(target: "MEMSAVE", "Patching memory at offset {}: #{} bytes", range.offset, &range.data.len());
            self.write_buf(range.offset, &range.data)?;
        }
        self.writes = temp;
        Ok(())
    }

    pub fn dump_writes(&self) -> anyhow::Result<()> {
        if self.writes.is_none() {
            bail!("dump_writes but writes not enabled!");
        }
        if self.already_wrote.load(Relaxed) {
            bail!("dump_writes but already wrote the latest changes!");
        }
        self.already_wrote.store(true, Relaxed);
        let mut mpf = MemoryPatchFile {
            hash: self.hash,
            ranges: self.writes.clone().unwrap(),
        };
        mpf.merge_adjacent_ranges();
        mpf.to_file(format!("./saved-writes/{}/{}", self.hash, self.write_index).into())?;
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
        if let Some(ref mut writes) = &mut self.writes {
            self.already_wrote.store(false, Relaxed);
            writes.push(
                MemoryPatch { offset: off, data: src_slice.to_vec() }
            );
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
        if let Some(ref mut writes) = &mut self.writes {
            self.already_wrote.store(false, Relaxed);
            writes.push(
                MemoryPatch { offset: off, data: src.to_vec() }
            );
        }
        self.data[off..off + src.len()].copy_from_slice(src);
        Ok(())
    }
    pub fn memset(&mut self, off: usize, len: usize, val: u8) -> anyhow::Result<()> {
        if off + len > self.data.len() {
            bail!("OOB memset on BigEndianMemory, offset {off:x}");
        }
        if let Some(ref mut writes) = &mut self.writes {
            self.already_wrote.store(false, Relaxed);
            writes.push(
                MemoryPatch { offset: off, data: vec![val; len] }
            );
        }
        for d in &mut self.data[off..off+len] {
            *d = val;
        }
        Ok(())
    }
}

#[derive(Encode, Decode, PartialEq, Debug, Clone)]
pub struct MemoryPatch {
    pub offset: usize,
    pub data: Vec<u8>,
}

#[derive(Encode, Decode, PartialEq, Debug, Clone)]
pub struct MemoryPatchFile {
    hash: u32,
    ranges: Vec<MemoryPatch>,
}

impl MemoryPatchFile {
    pub fn from_file(path: std::path::PathBuf) -> Self {
        use lz4_flex::frame::*;
        let mut bytes: Vec<u8> = Vec::new();
        FrameDecoder::new(std::fs::File::open(path).unwrap()).read_to_end(&mut bytes).unwrap();
        let res: MemoryPatchFile = bincode::decode_from_slice(&bytes, config::standard()).unwrap().0;
        debug!(target: "MEMSAVE", "decoded MemoryPatchFile: hash: {} # of ranges: {}", res.hash, res.ranges.len());
        res
    }

    pub fn to_file(&self, path: std::path::PathBuf) -> anyhow::Result<()> {
        use std::io::BufWriter;
        use lz4_flex::frame::*;
        let bytes = bincode::encode_to_vec(self, config::standard())?;
        let mut file = std::fs::File::create(&path)?;
        let mut writer = BufWriter::new(&mut file);
        let mut encoder = FrameEncoder::new(&mut writer);
        encoder.write_all(&bytes)?;
        encoder.finish()?;
        drop(writer);
        file.flush()?;
        let real_size = bytes.len() as f64;
        let written = file.metadata()?.len() as f64;
        debug!(target: "MEMSAVE", "encoded MemoryPatchFile to {path:?}, size {:.1}k compressed to {:.1}k. ({:.2}%)", (real_size/1024f64), (written/1024f64), (written/real_size));
        Ok(())
    }
    pub fn merge_adjacent_ranges(&mut self) {
        fn adjacent(a: &MemoryPatch, b: &MemoryPatch) -> bool {
            a.offset + a.data.len() == b.offset
        }
        let mut done = false;
        while !done {
            let mut found = false;
            for i in 0..(self.ranges.len()-1) {
                if adjacent(&self.ranges[i], &self.ranges[i+1]) {
                    found = true;
                    let extend = std::mem::take(&mut self.ranges[i+1].data);
                    self.ranges[i+1].offset = 0xDEADC0DE;
                    self.ranges[i].data.extend(extend.iter());
                }
            }
            if !found { done = true; }
        }
        self.ranges.retain(|x|x.offset != 0xDEADC0DE);
    }
}