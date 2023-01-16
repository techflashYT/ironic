#![allow(clippy::needless_range_loop)]
//! Minimal implementation of the SHA-1 algorithm.
//!
//! Apart from the fact that you shoud **not** use SHA-1, this is **not** an 
//! implementation that is suitable for, or otherwise intended for any kind of
//! practical cryptographic use outside of this particular application. 
//!
//! Do not even think about using this code somewhere else.
//!
//! For right now, we just assert that message size is a multiple of 64. I'm 
//! not sure what the hardware behavior is (either the SHA engine disregards 
//! messages which aren't a multiple of 64-bytes long, or it always performs 
//! DMA reads in 64-byte chunks).

use std::cell::OnceCell;

const K: [u32; 4] = [ 0x5a82_7999, 0x6ed9_eba1, 0x8f1b_bcdc, 0xca62_c1d6, ];

struct SyncSha1FnPtr {
    pub inner: OnceCell<unsafe fn(&mut Sha1State)>
}

impl SyncSha1FnPtr {
    const fn new() -> Self {
        Self { inner: OnceCell::new() }
    }
}

unsafe impl Sync for SyncSha1FnPtr {}

static PROCESS_MSG_FN: SyncSha1FnPtr = SyncSha1FnPtr::new();

pub struct Sha1State {
    pub digest: [u32; 5],
    pub buf: [u8; 64],
}

impl Default for Sha1State {
    fn default() -> Self {
        Self::new()
    }
}

impl Sha1State {
    pub fn new() -> Self {
        if PROCESS_MSG_FN.inner.get().is_none() {
            init_process_msg_fn_ptr();
        }
        Sha1State { digest: [0; 5], buf: [0; 64] }
    }
    pub fn reset(&mut self) {
        self.digest = [0; 5];
        self.buf = [0; 64];
    }
}

impl Sha1State {
    pub fn update(&mut self, input: &Vec<u8>) {
        assert!(input.len() % 64 == 0);
        for chunk in input.chunks(64) {
            self.buf.copy_from_slice(chunk);
            self.process_message();
        }
    }

    fn process_message(&mut self) {
        // SAFETY:
        //   for unwrap_unchecked - we could only be here if we have &mut self
        //     and the only constructor initialized the OnceCell or panics
        //   for calling the unsafe funtion ptr - these functions are unsafe
        //      due to the use of #[target_feature] to enable optimizations, the initialization
        //      of the OnceCell does the proper feature checks to ensure the target_feature is present
        //      on the current CPU
        unsafe { PROCESS_MSG_FN.inner.get().unwrap_unchecked()(self) };
    }

    #[cfg(target_arch = "aarch64")]
    #[target_feature(enable = "neon")]
    #[target_feature(enable = "sve2")]
    unsafe fn process_message_aarch_neon_sve2(&mut self) {
        self.process_message_scalar()
    }


    #[cfg(target_arch = "aarch64")]
    #[target_feature(enable = "neon")]
    unsafe fn process_message_aarch_neon(&mut self) {
        self.process_message_scalar()
    }

    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    #[target_feature(enable = "avx2")]
    #[target_feature(enable = "fma")]
    unsafe fn process_message_ia_avx2_and_fma(&mut self) {
        self.process_message_scalar()
    }

    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    #[target_feature(enable = "sse4.2")]
    unsafe fn process_message_ia_sse42(&mut self) {
        self.process_message_scalar()
    }

    #[inline(always)]
    fn process_message_scalar(&mut self) {
        let mut a = self.digest[0];
        let mut b = self.digest[1];
        let mut c = self.digest[2];
        let mut d = self.digest[3];
        let mut e = self.digest[4];

        let mut w = [0u32; 80];
        for (idx, wb) in self.buf.chunks(4).enumerate() {
            let mut word = [0u8; 4];
            word.copy_from_slice(wb);
            let word = u32::from_be_bytes(word);
            w[idx] = word;
        }

        for t in 16..80 {
            let word = w[t-3] ^ w[t-8] ^ w[t-14] ^ w[t-16];
            w[t] = word.rotate_left(1);
        }

        for t in 0..20 {
            let temp = a.rotate_left(5)
                .wrapping_add((b & c) | ((!b) & d))
                .wrapping_add(e)
                .wrapping_add(w[t])
                .wrapping_add(K[0]);

            e = d;
            d = c;
            c = b.rotate_left(30);
            b = a;
            a = temp;
        }

        for t in 20..40 {
            let temp = a.rotate_left(5)
                .wrapping_add(b ^ c ^ d)
                .wrapping_add(e)
                .wrapping_add(w[t])
                .wrapping_add(K[1]);

            e = d;
            d = c;
            c = b.rotate_left(30);
            b = a;
            a = temp;
        }

        for t in 40..60 {
            let temp = a.rotate_left(5)
                .wrapping_add((b & c) | (b & d) | (c & d))
                .wrapping_add(e)
                .wrapping_add(w[t])
                .wrapping_add(K[2]);

            e = d;
            d = c;
            c = b.rotate_left(30);
            b = a;
            a = temp;
        }

        for t in 60..80 {
            let temp = a.rotate_left(5)
                .wrapping_add(b ^ c ^ d)
                .wrapping_add(e)
                .wrapping_add(w[t])
                .wrapping_add(K[3]);

            e = d;
            d = c;
            c = b.rotate_left(30);
            b = a;
            a = temp;
        }

        self.digest[0] = self.digest[0].wrapping_add(a);
        self.digest[1] = self.digest[1].wrapping_add(b);
        self.digest[2] = self.digest[2].wrapping_add(c);
        self.digest[3] = self.digest[3].wrapping_add(d);
        self.digest[4] = self.digest[4].wrapping_add(e);

    }
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
fn init_process_msg_fn_ptr() {
    let err;
    if is_x86_feature_detected!("avx2") && is_x86_feature_detected!("fma") {
        err = PROCESS_MSG_FN.inner.set(Sha1State::process_message_ia_avx2_and_fma);
    }
    else if is_x86_feature_detected!("sse4.2") {
        err = PROCESS_MSG_FN.inner.set(Sha1State::process_message_ia_sse42);
    }
    else {
        err = PROCESS_MSG_FN.inner.set(Sha1State::process_message_scalar)
    }
    if err.is_err() {
        panic!("Failed to initialize Sha function pointer.");
    }
}


#[cfg(target_arch = "aarch64")]
fn init_process_msg_fn_ptr() {
    use std::arch::is_aarch64_feature_detected;
    let err;
    if is_aarch64_feature_detected!("sve2") {
        err = PROCESS_MSG_FN.inner.set(Sha1State::process_message_aarch_neon_sve2)
    }
    else if is_aarch64_feature_detected!("neon") {
        err = PROCESS_MSG_FN.inner.set(Sha1State::process_message_aarch_neon);
    }
    else {
        err = PROCESS_MSG_FN.inner.set(Sha1State::process_message_scalar);
    }
    if err.is_err() {
        panic!("Failed to initialize Sha function pointer.");
    }
}

#[cfg(not(any(target_arch = "x86", target_arch = "x86_64", target_arch = "aarch64")))]
fn init_process_msg_fn_ptr() {
    if PROCESS_MSG_FN.inner.set(Sha1State::process_message_scalar).is_err() {
        panic!("Failed to initialize Sha function pointer.");
    }
}