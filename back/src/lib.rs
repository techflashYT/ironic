
// For evaluating lookup tables at compile-time.
#![feature(const_mut_refs)]

#![deny(unsafe_op_in_unsafe_fn)]

pub mod back;
pub mod bits;
pub mod decode;

pub mod interp;

pub mod ipc;
pub mod ppc;
