//! Thumb instruction decoder.

use crate::bits::xDisplay;



#[derive(Clone, Debug, PartialEq)]
pub enum ThumbInst {
    SbcReg, CmpReg, OrrReg, BicReg, TstReg, EorReg, MvnReg, CmnReg, AdcReg,
    AndReg, MovReg, SubReg, AddReg, CmpRegAlt, AddRegAlt, MovRegAlt,
    MovRegShiftReg,

    Neg, AddImm, MovImm, SubImm, CmpImm, AddSpImm, SubSpImm,
    AddSpImmAlt, AddImmAlt, SubImmAlt, 

    StrbReg, LdrhReg, LdrbReg, StrReg, StrhReg, LdrReg, LdrsbReg, LdrshReg,

    StrhImm, StrImm, StrbImm, StrImmAlt, LdrhImm, LdrbImm, LdrImm, LdrImmAlt, 
    LdrLit, Stm, Ldm,

    Pop, Push, Mul,
    B, Bx, BlxReg, Svc, Bkpt, BAlt,

    Undefined,

    // These are exceptional (added by hand) until I decide sort how these
    // are decoded
    BlPrefix, BlImmSuffix, BlxImmSuffix,
}

impl std::fmt::Display for ThumbInst {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if !f.alternate() {
            return core::fmt::Debug::fmt(&self, f);
        }
        match self {
            ThumbInst::SbcReg         => write!(f, "sbc "),
            ThumbInst::CmpReg         => write!(f, "cmp "),
            ThumbInst::OrrReg         => write!(f, "orr "),
            ThumbInst::BicReg         => write!(f, "bic"),
            ThumbInst::TstReg         => write!(f, "tst "),
            ThumbInst::EorReg         => write!(f, "eor "),
            ThumbInst::MvnReg         => write!(f, "mvn "),
            ThumbInst::CmnReg         => write!(f, "cmn "),
            ThumbInst::AdcReg         => write!(f, "adc "),
            ThumbInst::AndReg         => write!(f, "and "),
            ThumbInst::MovReg         => write!(f, "mov "),
            ThumbInst::SubReg         => write!(f, "sub "),
            ThumbInst::AddReg         => write!(f, "add "),
            ThumbInst::CmpRegAlt      => write!(f, "cmp "),
            ThumbInst::AddRegAlt      => write!(f, "add "),
            ThumbInst::MovRegAlt      => write!(f, "mov "),
            ThumbInst::MovRegShiftReg => write!(f, "mov "),
            ThumbInst::Neg            => write!(f, "neg "),
            ThumbInst::AddImm         => write!(f, "add "),
            ThumbInst::MovImm         => write!(f, "mov "),
            ThumbInst::SubImm         => write!(f, "sub "),
            ThumbInst::CmpImm         => write!(f, "cmp "),
            ThumbInst::AddSpImm       => write!(f, "add sp"),
            ThumbInst::SubSpImm       => write!(f, "sub sp"),
            ThumbInst::AddSpImmAlt    => write!(f, "add sp"),
            ThumbInst::AddImmAlt      => write!(f, "add "),
            ThumbInst::SubImmAlt      => write!(f, "sub "),
            ThumbInst::StrbReg        => write!(f, "strb "),
            ThumbInst::LdrhReg        => write!(f, "ldrh "),
            ThumbInst::LdrbReg        => write!(f, "ldrb "),
            ThumbInst::StrReg         => write!(f, "str "),
            ThumbInst::StrhReg        => write!(f, "strh "),
            ThumbInst::LdrReg         => write!(f, "ldr "),
            ThumbInst::LdrsbReg       => write!(f, "ldrsb "),
            ThumbInst::LdrshReg       => write!(f, "ldrsh "),
            ThumbInst::StrhImm        => write!(f, "strh "),
            ThumbInst::StrImm         => write!(f, "str "),
            ThumbInst::StrbImm        => write!(f, "strb "),
            ThumbInst::StrImmAlt      => write!(f, "str "),
            ThumbInst::LdrhImm        => write!(f, "ldrh "),
            ThumbInst::LdrbImm        => write!(f, "ldrb "),
            ThumbInst::LdrImm         => write!(f, "ldr "),
            ThumbInst::LdrImmAlt      => write!(f, "ldr "),
            ThumbInst::LdrLit         => write!(f, "ldr "),
            ThumbInst::Stm            => write!(f, "stmia "),
            ThumbInst::Ldm            => write!(f, "ldmia "),
            ThumbInst::Pop            => write!(f, "pop "),
            ThumbInst::Push           => write!(f, "push "),
            ThumbInst::Mul            => write!(f, "mul "),
            ThumbInst::B              => write!(f, "b "),
            ThumbInst::Bx             => write!(f, "bx "),
            ThumbInst::BlxReg         => write!(f, "blx "),
            ThumbInst::Svc            => write!(f, "svc "),
            ThumbInst::Bkpt           => write!(f, "bkpt "),
            ThumbInst::BAlt           => write!(f, "b"),
            ThumbInst::BlPrefix       => write!(f, ""),
            ThumbInst::BlImmSuffix    => write!(f, "bl "),
            ThumbInst::BlxImmSuffix   => write!(f, "blx "),

            ThumbInst::Undefined      => write!(f, "Undefined"),
        }
    }
}


impl ThumbInst {
    pub const fn decode(opcd: u16) -> ThumbInst {
        use ThumbInst::*;
        match opcd & 0xffc0 {
            0x4240 => return Neg,
            0x4180 => return SbcReg,
            0x4280 => return CmpReg,
            0x4300 => return OrrReg,
            0x4380 => return BicReg,
            0x4200 => return TstReg,
            0x4040 => return EorReg,
            0x43c0 => return MvnReg,
            0x42c0 => return CmnReg,
            0x4140 => return AdcReg,
            0x4340 => return Mul,
            0x4000 => return AndReg,
            _ => {},
        }
        match opcd & 0xff80 {
            0xb000 => return AddSpImmAlt,
            0xb080 => return SubSpImm,
            0x4700 => return Bx,
            0x4780 => return BlxReg,
            _ => {},
        }
        match opcd & 0xff00 {
            0xdf00 => return Svc,
            0x4500 => return CmpRegAlt,
            0x4400 => return AddRegAlt,
            0x4600 => return MovReg,
            0xbe00 => return Bkpt,
            _ => {},
        }
        match opcd & 0xfe00 {
            0x1c00 => return AddImm,
            0x5800 => return LdrReg,
            0x5600 => return LdrsbReg,
            0x4000 => return MovRegShiftReg,
            0x5e00 => return LdrshReg,
            0x1e00 => return SubImm,
            0xbc00 => return Pop,
            0x1800 => return AddReg,
            0x5400 => return StrbReg,
            0x1a00 => return SubReg,
            0xb400 => return Push,
            0x5a00 => return LdrhReg,
            0x5c00 => return LdrbReg,
            0x5000 => return StrReg,
            0x5200 => return StrhReg,
            _ => {},
        }
        match opcd & 0xf800 {
            // Exceptional (added by hand)
            0xf000 => return BlPrefix,
            0xf800 => return BlImmSuffix,
            0xe800 => return BlxImmSuffix,

            0xe000 => return BAlt,
            0x2000 => return MovImm,
            0x3000 => return AddImmAlt,
            0xa800 => return AddSpImm,
            0x8000 => return StrhImm,
            0xc000 => return Stm,
            0x3800 => return SubImmAlt,
            0x2800 => return CmpImm,
            0x6000 => return StrImm,
            0x9000 => return StrImmAlt,
            0x7000 => return StrbImm,
            0x8800 => return LdrhImm,
            0x7800 => return LdrbImm,
            0xc800 => return Ldm,
            0x6800 => return LdrImm,
            0x9800 => return LdrImmAlt,
            0x4800 => return LdrLit,
            _ => {},
        }
        if opcd & 0xf000 == 0xd000 {
            return B;
        }
        else if opcd & 0xe000 == 0x0 {
            return MovRegAlt;
        }
        Undefined
    }

    pub fn bits_for_display(&self, bits: u16) -> Box<dyn xDisplay> {
        use crate::bits::thumb::*;
        let res: Box<dyn xDisplay> = match self {
            ThumbInst::SbcReg         => Box::new(BitwiseRegBits(bits)) as Box<dyn xDisplay>,
            ThumbInst::CmpReg         => Box::new(CmpRegBits(bits)) as Box<dyn xDisplay>,
            ThumbInst::OrrReg         => Box::new(BitwiseRegBits(bits)) as Box<dyn xDisplay>,
            ThumbInst::BicReg         => Box::new(BitwiseRegBits(bits)) as Box<dyn xDisplay>,
            ThumbInst::TstReg         => Box::new(CmpRegBits(bits)) as Box<dyn xDisplay>,
            ThumbInst::EorReg         => Box::new(BitwiseRegBits(bits)) as Box<dyn xDisplay>,
            ThumbInst::MvnReg         => Box::new(MvnRegBits(bits)) as Box<dyn xDisplay>,
            ThumbInst::CmnReg         => Box::new(CmpRegBits(bits)) as Box<dyn xDisplay>,
            ThumbInst::AdcReg         => Box::new(BitwiseRegBits(bits)) as Box<dyn xDisplay>,
            ThumbInst::AndReg         => Box::new(BitwiseRegBits(bits)) as Box<dyn xDisplay>,
            ThumbInst::MovReg         => Box::new(MovRegBits(bits)) as Box<dyn xDisplay>,
            ThumbInst::SubReg         => Box::new(AddSubRegBits(bits)) as Box<dyn xDisplay>,
            ThumbInst::AddReg         => Box::new(AddSubRegBits(bits)) as Box<dyn xDisplay>,
            ThumbInst::CmpRegAlt      => Box::new(CmpRegAltBits(bits)) as Box<dyn xDisplay>,
            ThumbInst::AddRegAlt      => Box::new(AddRegAltBits(bits)) as Box<dyn xDisplay>,
            ThumbInst::MovRegAlt      => Box::new(MovRegAltBits(bits)) as Box<dyn xDisplay>,
            ThumbInst::MovRegShiftReg => Box::new(MovRsrBits(bits)) as Box<dyn xDisplay>,
            ThumbInst::Neg            => Box::new(NegBits(bits)) as Box<dyn xDisplay>,
            ThumbInst::AddImm         => Box::new(AddSubImmBits(bits)) as Box<dyn xDisplay>,
            ThumbInst::MovImm         => Box::new(MovImmBits(bits)) as Box<dyn xDisplay>,
            ThumbInst::SubImm         => Box::new(AddSubImmBits(bits)) as Box<dyn xDisplay>,
            ThumbInst::CmpImm         => Box::new(CmpImmBits(bits)) as Box<dyn xDisplay>,
            ThumbInst::AddSpImm       => Box::new(MovImmBits(bits)) as Box<dyn xDisplay>,
            ThumbInst::SubSpImm       => Box::new(AddSubSpImmAltBits(bits)) as Box<dyn xDisplay>,
            ThumbInst::AddSpImmAlt    => Box::new(AddSubSpImmAltBits(bits)) as Box<dyn xDisplay>,
            ThumbInst::AddImmAlt      => Box::new(AddSubImmAltBits(bits)) as Box<dyn xDisplay>,
            ThumbInst::SubImmAlt      => Box::new(AddSubImmAltBits(bits)) as Box<dyn xDisplay>,
            ThumbInst::StrbReg        => Box::new(LoadStoreRegBits(bits)) as Box<dyn xDisplay>,
            ThumbInst::LdrhReg        => Box::new(LoadStoreRegBits(bits)) as Box<dyn xDisplay>,
            ThumbInst::LdrbReg        => Box::new(LoadStoreRegBits(bits)) as Box<dyn xDisplay>,
            ThumbInst::StrReg         => Box::new(LoadStoreRegBits(bits)) as Box<dyn xDisplay>,
            ThumbInst::StrhReg        => Box::new(LoadStoreRegBits(bits)) as Box<dyn xDisplay>,
            ThumbInst::LdrReg         => Box::new(LoadStoreRegBits(bits)) as Box<dyn xDisplay>,
            ThumbInst::LdrsbReg       => Box::new(LoadStoreRegBits(bits)) as Box<dyn xDisplay>,
            ThumbInst::LdrshReg       => Box::new(LoadStoreRegBits(bits)) as Box<dyn xDisplay>,
            ThumbInst::StrhImm        => Box::new(LoadStoreImmBits(bits)) as Box<dyn xDisplay>,
            ThumbInst::StrImm         => Box::new(LoadStoreImmBits(bits)) as Box<dyn xDisplay>,
            ThumbInst::StrbImm        => Box::new(LoadStoreImmBits(bits)) as Box<dyn xDisplay>,
            ThumbInst::StrImmAlt      => Box::new(LoadStoreAltBits(bits)) as Box<dyn xDisplay>,
            ThumbInst::LdrhImm        => Box::new(LoadStoreImmBits(bits)) as Box<dyn xDisplay>,
            ThumbInst::LdrbImm        => Box::new(LoadStoreImmBits(bits)) as Box<dyn xDisplay>,
            ThumbInst::LdrImm         => Box::new(LoadStoreImmBits(bits)) as Box<dyn xDisplay>,
            ThumbInst::LdrImmAlt      => Box::new(LoadStoreAltBits(bits)) as Box<dyn xDisplay>,
            ThumbInst::LdrLit         => Box::new(LoadStoreAltBits(bits)) as Box<dyn xDisplay>,
            ThumbInst::Stm            => Box::new(LoadStoreMultiBits(bits)) as Box<dyn xDisplay>,
            ThumbInst::Ldm            => Box::new(LoadStoreMultiBits(bits)) as Box<dyn xDisplay>,
            ThumbInst::Pop            => Box::new(PopBits(bits)) as Box<dyn xDisplay>,
            ThumbInst::Push           => Box::new(PushBits(bits)) as Box<dyn xDisplay>,
            ThumbInst::Mul            => Box::new(MulBits(bits)) as Box<dyn xDisplay>,
            ThumbInst::B              => Box::new(BranchBits(bits)) as Box<dyn xDisplay>,
            ThumbInst::Bx             => Box::new(BxBits(bits)) as Box<dyn xDisplay>,
            ThumbInst::BlxReg         => Box::new(BxBits(bits)) as Box<dyn xDisplay>,
            ThumbInst::Svc            => Box::new(MiscBits(bits)) as Box<dyn xDisplay>,
            ThumbInst::Bkpt           => Box::new(MiscBits(bits)) as Box<dyn xDisplay>,
            ThumbInst::BAlt           => Box::new(BranchAltBits(bits)) as Box<dyn xDisplay>,
            ThumbInst::BlPrefix       => Box::new(BlBits(bits)) as Box<dyn xDisplay>,
            ThumbInst::BlImmSuffix    => Box::new(BlBits(bits)) as Box<dyn xDisplay>,
            ThumbInst::BlxImmSuffix   => Box::new(BlBits(bits)) as Box<dyn xDisplay>,

            ThumbInst::Undefined      => todo!(),
        };
        res
    }
}

