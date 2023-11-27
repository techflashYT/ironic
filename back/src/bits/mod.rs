//! Wrapper types for representing ARM/Thumb instructions as bitfields.

pub mod arm;
pub mod thumb;

use std::any::{Any, TypeId};
#[allow(non_camel_case_types, unused_variables)]
/// Like std::fmt::Display but provides facilities for a 'context' that can provide
/// additional information to the formatter.
/// Implement xDisplay on the opcode bits to enable disassembly like printing.
/// The context feature is used to provide additional information such as a base
/// for relative addressing modes.
pub trait xDisplay{
    /// See xDisplay::required_context for what to pass to ctx.
    fn fmt(&self, f: &mut String, ctx: Option<Box<dyn Any>>) -> anyhow::Result<()> {
        anyhow::bail!("Unimplemented")
    }
    fn required_context(&self) -> Option<TypeId> {
        None
    }
}

pub mod disassembly {
    use anyhow::bail;
    use crate::decode::thumb::*;
    use std::any::{Any, TypeId};

    pub fn disassmble_thumb(op: u16, address: u32) -> anyhow::Result<String> {
        let instrution = ThumbInst::decode(op);
        if instrution == crate::decode::thumb::ThumbInst::Undefined {
            bail!("Failed to decode opcde: {op:x}");
        }
        let bits = instrution.bits_for_display(op);
        let ctx = if bits.required_context() == Some(TypeId::of::<u32>()) {
            match instrution {
                // These instructions want a base register as context
                ThumbInst::StrImmAlt => Some(Box::new(13) as Box<dyn Any>),
                ThumbInst::LdrImmAlt => Some(Box::new(13) as Box<dyn Any>),
                ThumbInst::LdrLit => Some(Box::new(15) as Box<dyn Any>),
                _ => Some(Box::new(address) as Box<dyn Any>) // Otherwise context is PC

            }
        } else { None };
        let mut res = format!("{instrution:#}");
        bits.fmt(&mut res, ctx)?;
        Ok(res)
    }
    pub fn disassmble_arm(_op: u32, _address: u32) {
        todo!();
    }
}