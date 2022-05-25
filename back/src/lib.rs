
// For evaluating lookup tables at compile-time.
#![feature(const_mut_refs,label_break_value, const_eval_limit)]
// The new LUT needs more iterations beyond what the compiler is comfortable doing by default.
#![const_eval_limit = "0"]

pub mod back;
pub mod bits;
pub mod decode;

pub mod interp;

pub mod ipc;
pub mod ppc;
pub mod debug;
