//! Wrapper types for representing ARM/Thumb instructions as bitfields.

pub mod arm;
pub mod thumb;

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum DisassemblyContext {
    /// PC for offset calculations
    PC(u32),
    /// Register number for formatting
    BaseRegister(u32),
    /// True if Blx, false if other + PC for offset calculation
    BlxDiscriminantAndPC((bool, u32)),
    /// No context required
    NotNeeded,
}

#[allow(non_camel_case_types, unused_variables)]
/// Like std::fmt::Display but provides facilities for a 'context' that can provide
/// additional information to the formatter.
/// Implement xDisplay on the opcode bits to enable disassembly like printing.
/// The context feature is used to provide additional information such as a base
/// for relative addressing modes.
pub trait xDisplay{
    /// See xDisplay::required_context for what to pass to ctx.
    fn fmt(&self, f: &mut String, ctx: DisassemblyContext) -> anyhow::Result<()> {
        anyhow::bail!("Unimplemented")
    }
    fn required_context(&self) -> DisassemblyContext {
        DisassemblyContext::NotNeeded
    }
}

pub mod disassembly {
    use anyhow::bail;
    use crate::decode::thumb::*;

    pub fn disassmble_thumb(op: u16, address: u32) -> anyhow::Result<String> {
        let instrution = ThumbInst::decode(op);
        if instrution == crate::decode::thumb::ThumbInst::Undefined {
            bail!("Failed to decode opcde: {op:x}");
        }
        let bits = instrution.bits_for_display(op);
        let ctx = match bits.required_context() {
            super::DisassemblyContext::PC(_) => super::DisassemblyContext::PC(address),
            super::DisassemblyContext::BaseRegister(_) => super::DisassemblyContext::BaseRegister(match instrution {
                // These instructions want a base register as context
                ThumbInst::StrImmAlt |
                ThumbInst::LdrImmAlt => 13,
                ThumbInst::LdrLit => 15,
                _ => unreachable!(),
            }),
            super::DisassemblyContext::BlxDiscriminantAndPC(_) => unreachable!(), // not for thumb
            super::DisassemblyContext::NotNeeded => super::DisassemblyContext::NotNeeded,
        };
        let mut res = format!("{instrution:#}");
        bits.fmt(&mut res, ctx)?;
        Ok(res)
    }
    pub fn disassmble_arm(_op: u32, _address: u32) {
        todo!();
    }
}