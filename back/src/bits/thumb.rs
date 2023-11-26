use std::any::{Any, TypeId};
use super::xDisplay;
use anyhow::bail;

/// ["Bl", "Blx"]
#[repr(transparent)]
pub struct BlBits(pub u16);
impl BlBits {
    #[inline(always)]
    pub fn imm11(&self) -> u16 { self.0 & 0x07ff }
    #[inline(always)]
    pub fn h(&self) -> u16 { (self.0 >> 11) & 0x3 }
}
impl xDisplay for BlBits {} // 2 parter


/// ['Neg']
#[repr(transparent)]
pub struct NegBits(pub u16);
impl NegBits {
    #[inline(always)]
    pub fn rn(&self) -> u16 { (self.0 & 0x0038) >> 3 }
    #[inline(always)]
    pub fn rd(&self) -> u16 { self.0 & 0x0007 }
}
impl xDisplay for NegBits {
    fn fmt(&self, f: &mut String, _: Option<Box<dyn Any>>) -> anyhow::Result<()> {
        f.push_str(&format!("{}, {}", self.rd(), self.rn()));
        return Ok(());
    }
}

/// ['SbcReg', 'OrrReg', 'BicReg', 'EorReg', 'AdcReg', 'AndReg']
#[repr(transparent)]
pub struct BitwiseRegBits(pub u16);
impl BitwiseRegBits {
    #[inline(always)]
    pub fn rm(&self) -> u16 { (self.0 & 0x0038) >> 3 }
    #[inline(always)]
    pub fn rdn(&self) -> u16 { self.0 & 0x0007 }
}
impl xDisplay for BitwiseRegBits {
    fn fmt(&self, f: &mut String, _: Option<Box<dyn Any>>) -> anyhow::Result<()> {
        f.push_str(&format!("{}, {}", self.rdn(), self.rm()));
        return Ok(());
    }
}

/// ['CmpReg', 'TstReg', 'CmnReg']
#[repr(transparent)]
pub struct CmpRegBits(pub u16);
impl CmpRegBits {
    #[inline(always)]
    pub fn rm(&self) -> u16 { (self.0 & 0x0038) >> 3 }
    #[inline(always)]
    pub fn rn(&self) -> u16 { self.0 & 0x0007 }
}
impl xDisplay for CmpRegBits {
    fn fmt(&self, f: &mut String, _: Option<Box<dyn Any>>) -> anyhow::Result<()> {
        f.push_str(&format!("{}, {}", self.rn(), self.rm()));
        return Ok(());
    }
}

/// ['MvnReg']
#[repr(transparent)]
pub struct MvnRegBits(pub u16);
impl MvnRegBits {
    #[inline(always)]
    pub fn rm(&self) -> u16 { (self.0 & 0x0038) >> 3 }
    #[inline(always)]
    pub fn rd(&self) -> u16 { self.0 & 0x0007 }
}
impl xDisplay for MvnRegBits {
    fn fmt(&self, f: &mut String, _: Option<Box<dyn Any>>) -> anyhow::Result<()> {
        f.push_str(&format!("{}, {}", self.rd(), self.rm()));

        return Ok(())
    }
}

/// ['Mul']
#[repr(transparent)]
pub struct MulBits(pub u16);
impl MulBits {
    #[inline(always)]
    pub fn rn(&self) -> u16 { (self.0 & 0x0038) >> 3 }
    #[inline(always)]
    pub fn rdm(&self) -> u16 { self.0 & 0x0007 }
}
impl xDisplay for MulBits {
    fn fmt(&self, f: &mut String, _: Option<Box<dyn Any>>) -> anyhow::Result<()> {
        f.push_str(&format!("{}, {}", self.rdm(), self.rn()));

        return Ok(())
    }
}

/// ['AddSpImmAlt', 'SubSpImm']
#[repr(transparent)]
pub struct AddSubSpImmAltBits(pub u16);
impl AddSubSpImmAltBits {
    #[inline(always)]
    pub fn imm7(&self) -> u16 { self.0 & 0x007f }
}
impl xDisplay for AddSubSpImmAltBits {
    fn fmt(&self, f: &mut String, _: Option<Box<dyn Any>>) -> anyhow::Result<()> {
        f.push_str(&format!("{}", self.imm7() << 2));
        Ok(())
    }
}

/// ['Bx', 'BlxReg']
#[repr(transparent)]
pub struct BxBits(pub u16);
impl BxBits {
    #[inline(always)]
    pub fn rm(&self) -> u16 { (self.0 & 0x0078) >> 3 }
}
impl xDisplay for BxBits {} //FIXME

/// ['Svc', 'Bkpt']
#[repr(transparent)]
pub struct MiscBits(pub u16);
impl MiscBits {
    #[inline(always)]
    pub fn imm8(&self) -> u16 { self.0 & 0x00ff }
}
impl xDisplay for MiscBits {
    fn fmt(&self, f: &mut String, _: Option<Box<dyn Any>>) -> anyhow::Result<()> {
        f.push_str(&format!("{}", self.imm8()));
        Ok(())
    }
}

/// ['CmpRegAlt']
#[repr(transparent)]
pub struct CmpRegAltBits(pub u16);
impl CmpRegAltBits {
    #[inline(always)]
    pub fn n(&self) -> bool { (self.0 & 0x0080) != 0 }
    #[inline(always)]
    pub fn rm(&self) -> u16 { (self.0 & 0x0078) >> 3 }
    #[inline(always)]
    pub fn rn(&self) -> u16 { self.0 & 0x0007 }
}
impl xDisplay for CmpRegAltBits {
    fn fmt(&self, f: &mut String, _: Option<Box<dyn Any>>) -> anyhow::Result<()> {
        let real_rn = if self.n() { self.rn() | 0x8 } else { self.rn() };
        f.push_str(&format!("{real_rn}, {}", self.rm()));
        return Ok(());
    }
}

/// ['AddRegAlt']
#[repr(transparent)]
pub struct AddRegAltBits(pub u16);
impl AddRegAltBits {
    #[inline(always)]
    pub fn dn(&self) -> bool { (self.0 & 0x0080) != 0 }
    #[inline(always)]
    pub fn rm(&self) -> u16 { (self.0 & 0x0078) >> 3 }
    #[inline(always)]
    pub fn rdn(&self) -> u16 { self.0 & 0x0007 }
}
impl xDisplay for AddRegAltBits {
    fn fmt(&self, f: &mut String, _: Option<Box<dyn Any>>) -> anyhow::Result<()> {
        let real_rd = if self.dn() { self.rdn() | 0x8 } else { self.rdn() };
        f.push_str(&format!("{real_rd}, {}", self.rm()));
        return Ok(())
    }
}

/// ['MovReg']
#[repr(transparent)]
pub struct MovRegBits(pub u16);
impl MovRegBits {
    #[inline(always)]
    pub fn d(&self) -> bool { (self.0 & 0x0080) != 0 }
    #[inline(always)]
    pub fn rm(&self) -> u16 { (self.0 & 0x0078) >> 3 }
    #[inline(always)]
    pub fn rd(&self) -> u16 { self.0 & 0x0007 }
}
impl xDisplay for MovRegBits {
    fn fmt(&self, f: &mut String, _: Option<Box<dyn Any>>) -> anyhow::Result<()> {
        let real_rd = if self.d() { self.rd() | 0x8 } else { self.rd() };
        f.push_str(&format!("{real_rd}, {}", self.rm()));
        return Ok(());
    }
}

/// ['AddImm', 'SubImm']
#[repr(transparent)]
pub struct AddSubImmBits(pub u16);
impl AddSubImmBits {
    #[inline(always)]
    pub fn imm3(&self) -> u16 { (self.0 & 0x01c0) >> 6 }
    #[inline(always)]
    pub fn rn(&self) -> u16 { (self.0 & 0x0038) >> 3 }
    #[inline(always)]
    pub fn rd(&self) -> u16 { self.0 & 0x0007 }
}
impl xDisplay for AddSubImmBits {
    fn fmt(&self, f: &mut String, _: Option<Box<dyn Any>>) -> anyhow::Result<()> {
        f.push_str(&format!("{}, {}, #{}", self.rd(), self.rn(), self.imm3()));
        Ok(())
    }
}

/// ['LdrReg', 'LdrsbReg', 'LdrshReg', 'StrbReg', 'LdrhReg', 'LdrbReg', 'StrReg', 'StrhReg']
#[repr(transparent)]
pub struct LoadStoreRegBits(pub u16);
impl LoadStoreRegBits {
    #[inline(always)]
    pub fn rm(&self) -> u16 { (self.0 & 0x01c0) >> 6 }
    #[inline(always)]
    pub fn rn(&self) -> u16 { (self.0 & 0x0038) >> 3 }
    #[inline(always)]
    pub fn rt(&self) -> u16 { self.0 & 0x0007 }
}
impl xDisplay for LoadStoreRegBits {
    fn fmt(&self, f: &mut String, _: Option<Box<dyn Any>>) -> anyhow::Result<()> {
        f.push_str(&format!("{}, [{}, {}]", self.rt(), self.rn(), self.rm()));

        return Ok(())
    }
}

/// ['MovRegShiftReg']
#[repr(transparent)]
pub struct MovRsrBits(pub u16);
impl MovRsrBits {
    #[inline(always)]
    pub fn op(&self) -> u16 { (self.0 & 0x03c0) >> 6 }
    #[inline(always)]
    pub fn rs(&self) -> u16 { (self.0 & 0x0038) >> 3 }
    #[inline(always)]
    pub fn rdm(&self) -> u16 { self.0 & 0x0007 }
}
impl xDisplay for MovRsrBits {} //FIXME

/// ['Pop']
#[repr(transparent)]
pub struct PopBits(pub u16);
impl PopBits {
    #[inline(always)]
    pub fn p(&self) -> bool { (self.0 & 0x0100) != 0 }
    #[inline(always)]
    pub fn register_list(&self) -> u16 { self.0 & 0x00ff }
}
impl xDisplay for PopBits {
    fn fmt(&self, f: &mut String, _: Option<Box<dyn Any>>) -> anyhow::Result<()> {
        let mut reglist = String::with_capacity(30);
        for i in 0..=7 {
            if (self.register_list() >> i) & 0x1 == 1 {
                reglist += format!(" r{i}").as_str();
            }
        }
        if self.p() { reglist += " pc" }
        f.push_str(&format!("{{{reglist} }}"));

        Ok(())
    }
}

/// ['AddReg', 'SubReg']
#[repr(transparent)]
pub struct AddSubRegBits(pub u16);
impl AddSubRegBits {
    #[inline(always)]
    pub fn rm(&self) -> u16 { (self.0 & 0x01c0) >> 6 }
    #[inline(always)]
    pub fn rn(&self) -> u16 { (self.0 & 0x0038) >> 3 }
    #[inline(always)]
    pub fn rd(&self) -> u16 { self.0 & 0x0007 }
}
impl xDisplay for AddSubRegBits {
    fn fmt(&self, f: &mut String, _: Option<Box<dyn Any>>) -> anyhow::Result<()> {
        f.push_str(&format!("{}, {}, {}", self.rd(), self.rn(), self.rm()));

        return Ok(())
    }
}

/// ['Push']
#[repr(transparent)]
pub struct PushBits(pub u16);
impl PushBits {
    #[inline(always)]
    pub fn m(&self) -> bool { (self.0 & 0x0100) != 0 }
    #[inline(always)]
    pub fn register_list(&self) -> u16 { self.0 & 0x00ff }
}
impl xDisplay for PushBits {
    fn fmt(&self, f: &mut String, _: Option<Box<dyn Any>>) -> anyhow::Result<()> {
        let mut reglist = String::with_capacity(30);
        for i in 0..=7 {
            if (self.register_list() >> i) & 0x1 == 1 {
                reglist += format!(" r{i}").as_str();
            }
        }
        if self.m() {reglist += " lr"}
        f.push_str(&format!("{{{reglist} }}"));

        return Ok(())
    }
}

/// ['MovImm', 'AddSpImm']
#[repr(transparent)]
pub struct MovImmBits(pub u16);
impl MovImmBits {
    #[inline(always)]
    pub fn rd(&self) -> u16 { (self.0 & 0x0700) >> 8 }
    #[inline(always)]
    pub fn imm8(&self) -> u16 { self.0 & 0x00ff }
}
impl xDisplay for MovImmBits {
    fn fmt(&self, f: &mut String, _: Option<Box<dyn Any>>) -> anyhow::Result<()> {
        f.push_str(&format!("{}, #{}", self.rd(), self.imm8()));

        return Ok(())
    }
}

/// ['AddImmAlt', 'SubImmAlt']
#[repr(transparent)]
pub struct AddSubImmAltBits(pub u16);
impl AddSubImmAltBits {
    #[inline(always)]
    pub fn rdn(&self) -> u16 { (self.0 & 0x0700) >> 8 }
    #[inline(always)]
    pub fn imm8(&self) -> u16 { self.0 & 0x00ff }
}
impl xDisplay for AddSubImmAltBits {
    fn fmt(&self, f: &mut String, _: Option<Box<dyn Any>>) -> anyhow::Result<()> {
        f.push_str(&format!("{}, #{}", self.rdn(), self.imm8()));

        return Ok(())
    }
}

/// ['StrhImm', 'StrImm', 'StrbImm', 'LdrhImm', 'LdrbImm', 'LdrImm']
#[repr(transparent)]
pub struct LoadStoreImmBits(pub u16);
impl LoadStoreImmBits {
    #[inline(always)]
    pub fn imm5(&self) -> u16 { (self.0 & 0x07c0) >> 6 }
    #[inline(always)]
    pub fn rn(&self) -> u16 { (self.0 & 0x0038) >> 3 }
    #[inline(always)]
    pub fn rt(&self) -> u16 { self.0 & 0x0007 }
}
impl xDisplay for LoadStoreImmBits {
    fn fmt(&self, f: &mut String, _: Option<Box<dyn Any>>) -> anyhow::Result<()> {
        f.push_str(&format!("{}, [{} #{}]", self.rt(), self.rn(), self.imm5()));

        return Ok(())
    }
}

/// ['Stm', 'Ldm']
#[repr(transparent)]
pub struct LoadStoreMultiBits(pub u16);
impl LoadStoreMultiBits {
    #[inline(always)]
    pub fn rn(&self) -> u16 { (self.0 & 0x0700) >> 8 }
    #[inline(always)]
    pub fn register_list(&self) -> u16 { self.0 & 0x00ff }
}
impl xDisplay for LoadStoreMultiBits {
    fn fmt(&self, f: &mut String, _: Option<Box<dyn Any>>) -> anyhow::Result<()> {
        let mut reglist = String::with_capacity(30);
        for i in 0..=7 {
            if (self.register_list() >> i) & 0x1 == 1 {
                reglist += format!(" r{i}").as_str();
            }
        }
        f.push_str(&format!("{}!, {{{reglist}}}", self.rn()));

        return Ok(())
    }
}

/// ['CmpImm']
#[repr(transparent)]
pub struct CmpImmBits(pub u16);
impl CmpImmBits {
    #[inline(always)]
    pub fn rn(&self) -> u16 { (self.0 & 0x0700) >> 8 }
    #[inline(always)]
    pub fn imm8(&self) -> u16 { self.0 & 0x00ff }
}
impl xDisplay for CmpImmBits {
    fn fmt(&self, f: &mut String, _: Option<Box<dyn Any>>) -> anyhow::Result<()> {
        f.push_str(&format!("{}, #{}", self.rn(), self.imm8()));

        return Ok(())
    }
}

/// ['StrImmAlt', 'LdrImmAlt', 'LdrLit']
#[repr(transparent)]
pub struct LoadStoreAltBits(pub u16);
impl LoadStoreAltBits {
    #[inline(always)]
    pub fn rt(&self) -> u16 { (self.0 & 0x0700) >> 8 }
    #[inline(always)]
    pub fn imm8(&self) -> u16 { self.0 & 0x00ff }
}
impl xDisplay for LoadStoreAltBits {
    fn fmt(&self, f: &mut String, ctx: Option<Box<dyn Any>>) -> anyhow::Result<()> {
        let reg = if let Some(maybe_basereg) = ctx {
            if let Ok(basereg) = maybe_basereg.downcast::<u32>() {
                match *basereg {
                    13 => "sp",
                    15 => "pc",
                    _ => bail!("Inappropriate base register"),
                }
            } else { bail!("downcast failed");}
        } else {bail!("Context required");};
        f.push_str(&format!("{}, [{reg}, #{}]", self.rt(), self.imm8()*4));

        return Ok(())
    }
    fn required_context(&self) -> Option<std::any::TypeId> {
        Some(TypeId::of::<u32>())
    }
}

/// ['BAlt']
#[repr(transparent)]
pub struct BranchAltBits(pub u16);
impl BranchAltBits {
    #[inline(always)]
    pub fn imm11(&self) -> u16 { self.0 & 0x07ff }
}
impl xDisplay for BranchAltBits {
    fn fmt(&self, f: &mut String, ctx: Option<Box<dyn Any>>) -> anyhow::Result<()> {
        let pc = if let Some(maybe_pc) = ctx{
            if let Ok(maybe_pc) = maybe_pc.downcast::<u32>() {
                *maybe_pc as i64
            } else { bail!("downcast failed"); }
        } else {bail!("context required");};
        let offset = 
            crate::interp::thumb::branch::sign_extend(self.imm11() as u32, 11) << 1;
        let target = pc.wrapping_add(offset as i64);
        f.push_str(&format!("0x{target:x}"));
        Ok(())
    }
    fn required_context(&self) -> Option<std::any::TypeId> {
        Some(TypeId::of::<u32>())
    }
}

/// ['B']
#[repr(transparent)]
pub struct BranchBits(pub u16);
impl BranchBits {
    #[inline(always)]
    pub fn cond(&self) -> u16 { (self.0 & 0x0f00) >> 8 }
    #[inline(always)]
    pub fn imm8(&self) -> u16 { self.0 & 0x00ff }
}
impl xDisplay for BranchBits {
    fn fmt(&self, f: &mut String, ctx: Option<Box<dyn Any>>) -> anyhow::Result<()> {
        let pc = if let Some(maybe_pc) = ctx{
            if let Ok(maybe_pc) = maybe_pc.downcast::<u32>() {
                *maybe_pc as i64
            } else { bail!("downcast failed"); }
        } else {bail!("context required");};
        let offset = 
            crate::interp::thumb::branch::sign_extend(self.imm8() as u32, 11) << 1;
        let target = pc.wrapping_add(offset as i64);
        let cond = ironic_core::cpu::reg::Cond::try_from(self.cond() as u32)?;
        f.push_str(&format!("{cond:?} 0x{target:x}"));
        Ok(())
    }
    fn required_context(&self) -> Option<std::any::TypeId> {
        Some(TypeId::of::<u32>())
    }
}

/// ['MovRegAlt']
#[repr(transparent)]
pub struct MovRegAltBits(pub u16);
impl MovRegAltBits {
    #[inline(always)]
    pub fn op(&self) -> u16 { (self.0 & 0x1800) >> 11 }
    #[inline(always)]
    pub fn imm5(&self) -> u16 { (self.0 & 0x07c0) >> 6 }
    #[inline(always)]
    pub fn rm(&self) -> u16 { (self.0 & 0x0038) >> 3 }
    #[inline(always)]
    pub fn rd(&self) -> u16 { self.0 & 0x0007 }
}
impl xDisplay for MovRegAltBits {} //FIXME
