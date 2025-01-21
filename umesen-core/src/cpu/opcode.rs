use crate::{Cpu, CpuError};

/// Addressing modes (most of them) for instructions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AddrMode {
    Implied,
    /// Operand contains the value
    Immediate,
    /// Operand contains the address to the value in the first page (256 bytes)
    ZeroPage,
    /// Same as ZeroPage + x register
    ZeroPageX,
    /// Same as ZeroPage + y register
    ZeroPageY,
    /// Same as ZeroPage but now the whole address range (16 bits)
    Absolute,
    /// Same as Absolute + x register
    AbsoluteX,
    /// Same as AbsoluteX but always clock when it would otherwise depend on page cross
    AbsoluteXForceClock,
    /// Same as Absolute + y register
    AbsoluteY,
    AbsoluteYForceClock,
    /// Operand contains the address to the address to the value
    Indirect,
    /// Operand contains the address (with x added) to the address to the value
    IndirectX,
    /// Operand contains the address to the address (with y added) to the value
    IndirectY,
    IndirectYForceClock,
    Relative,
}

impl AddrMode {
    pub fn short_name(&self) -> &'static str {
        match *self {
            Self::Immediate => "imm",
            Self::Implied => "imp",
            Self::ZeroPage => "zrp",
            Self::ZeroPageX => "zrx",
            Self::ZeroPageY => "zry",
            Self::Absolute => "abs",
            Self::AbsoluteX | Self::AbsoluteXForceClock => "abx",
            Self::AbsoluteY | Self::AbsoluteYForceClock => "aby",
            Self::Indirect => "ind",
            Self::IndirectX => "inx",
            Self::IndirectY | Self::IndirectYForceClock => "iny",
            Self::Relative => "rel",
        }
    }
}

pub struct Opcode {
    pub name: &'static str,
    pub addr_mode: AddrMode,
    pub byte: u8,
    pub expected_cycles: u8,
}

impl Opcode {
    pub fn new(name: &'static str, addr_mode: AddrMode) -> Self {
        Self {
            name,
            addr_mode,
            byte: 0,
            expected_cycles: 0,
        }
    }

    pub fn from_byte(byte: u8) -> Result<Self, CpuError> {
        use AddrMode::*;
        let mut opcode = match byte {
            // -- Stack --
            0x48 => Opcode::new("pha", Implied),
            0x08 => Opcode::new("php", Implied),
            0x68 => Opcode::new("pla", Implied),
            0x28 => Opcode::new("plp", Implied),

            // -- Shift and rotate --
            0x0a => Opcode::new("asl", Implied),
            0x06 => Opcode::new("asl", ZeroPage),
            0x16 => Opcode::new("asl", ZeroPageX),
            0x0e => Opcode::new("asl", Absolute),
            0x1e => Opcode::new("asl", AbsoluteXForceClock),

            0x4a => Opcode::new("lsr", Implied),
            0x46 => Opcode::new("lsr", ZeroPage),
            0x56 => Opcode::new("lsr", ZeroPageX),
            0x4e => Opcode::new("lsr", Absolute),
            0x5e => Opcode::new("lsr", AbsoluteXForceClock),

            0x2a => Opcode::new("rol", Implied),
            0x26 => Opcode::new("rol", ZeroPage),
            0x36 => Opcode::new("rol", ZeroPageX),
            0x2e => Opcode::new("rol", Absolute),
            0x3e => Opcode::new("rol", AbsoluteXForceClock),

            0x6a => Opcode::new("ror", Implied),
            0x66 => Opcode::new("ror", ZeroPage),
            0x76 => Opcode::new("ror", ZeroPageX),
            0x6e => Opcode::new("ror", Absolute),
            0x7e => Opcode::new("ror", AbsoluteXForceClock),

            // -- Arithmetic --
            0x69 => Opcode::new("adc", Immediate),
            0x65 => Opcode::new("adc", ZeroPage),
            0x75 => Opcode::new("adc", ZeroPageX),
            0x6d => Opcode::new("adc", Absolute),
            0x7d => Opcode::new("adc", AbsoluteX),
            0x79 => Opcode::new("adc", AbsoluteY),
            0x61 => Opcode::new("adc", IndirectX),
            0x71 => Opcode::new("adc", IndirectY),

            0xe9 => Opcode::new("sbc", Immediate),
            0xe5 => Opcode::new("sbc", ZeroPage),
            0xf5 => Opcode::new("sbc", ZeroPageX),
            0xed => Opcode::new("sbc", Absolute),
            0xfd => Opcode::new("sbc", AbsoluteX),
            0xf9 => Opcode::new("sbc", AbsoluteY),
            0xe1 => Opcode::new("sbc", IndirectX),
            0xf1 => Opcode::new("sbc", IndirectY),

            // -- Increment and decrement --
            0xe6 => Opcode::new("inc", ZeroPage),
            0xf6 => Opcode::new("inc", ZeroPageX),
            0xee => Opcode::new("inc", Absolute),
            0xfe => Opcode::new("inc", AbsoluteXForceClock),

            0xc6 => Opcode::new("dec", ZeroPage),
            0xd6 => Opcode::new("dec", ZeroPageX),
            0xce => Opcode::new("dec", Absolute),
            0xde => Opcode::new("dec", AbsoluteXForceClock),

            0xe8 => Opcode::new("inx", Implied),
            0xc8 => Opcode::new("iny", Implied),
            0xca => Opcode::new("dex", Implied),
            0x88 => Opcode::new("dey", Implied),

            // -- Register loads --
            0xa9 => Opcode::new("lda", Immediate),
            0xa5 => Opcode::new("lda", ZeroPage),
            0xb5 => Opcode::new("lda", ZeroPageX),
            0xad => Opcode::new("lda", Absolute),
            0xbd => Opcode::new("lda", AbsoluteX),
            0xb9 => Opcode::new("lda", AbsoluteY),
            0xa1 => Opcode::new("lda", IndirectX),
            0xb1 => Opcode::new("lda", IndirectY),

            0xa2 => Opcode::new("ldx", Immediate),
            0xa6 => Opcode::new("ldx", ZeroPage),
            0xb6 => Opcode::new("ldx", ZeroPageY),
            0xae => Opcode::new("ldx", Absolute),
            0xbe => Opcode::new("ldx", AbsoluteY),

            0xa0 => Opcode::new("ldy", Immediate),
            0xa4 => Opcode::new("ldy", ZeroPage),
            0xb4 => Opcode::new("ldy", ZeroPageX),
            0xac => Opcode::new("ldy", Absolute),
            0xbc => Opcode::new("ldy", AbsoluteX),

            // -- Register stores --
            0x85 => Opcode::new("sta", ZeroPage),
            0x95 => Opcode::new("sta", ZeroPageX),
            0x8d => Opcode::new("sta", Absolute),
            0x9d => Opcode::new("sta", AbsoluteXForceClock),
            0x99 => Opcode::new("sta", AbsoluteYForceClock),
            0x81 => Opcode::new("sta", IndirectX),
            0x91 => Opcode::new("sta", IndirectYForceClock),

            0x8e => Opcode::new("stx", Absolute),
            0x86 => Opcode::new("stx", ZeroPage),
            0x96 => Opcode::new("stx", ZeroPageY),

            0x8c => Opcode::new("sty", Absolute),
            0x84 => Opcode::new("sty", ZeroPage),
            0x94 => Opcode::new("sty", ZeroPageX),

            // -- Register transfers --
            0xaa => Opcode::new("tax", Implied),
            0xa8 => Opcode::new("tay", Implied),
            0xba => Opcode::new("tsx", Implied),
            0x8a => Opcode::new("txa", Implied),
            0x9a => Opcode::new("txs", Implied),
            0x98 => Opcode::new("tya", Implied),

            // -- Flag clear and set --
            0x18 => Opcode::new("clc", AddrMode::Implied),
            0xd8 => Opcode::new("cld", AddrMode::Implied),
            0x58 => Opcode::new("cli", AddrMode::Implied),
            0xb8 => Opcode::new("clv", AddrMode::Implied),
            0x38 => Opcode::new("sec", AddrMode::Implied),
            0xf8 => Opcode::new("sed", AddrMode::Implied),
            0x78 => Opcode::new("sei", AddrMode::Implied),

            // -- Logic --
            0x29 => Opcode::new("and", Immediate),
            0x25 => Opcode::new("and", ZeroPage),
            0x35 => Opcode::new("and", ZeroPageX),
            0x2d => Opcode::new("and", Absolute),
            0x3d => Opcode::new("and", AbsoluteX),
            0x39 => Opcode::new("and", AbsoluteY),
            0x21 => Opcode::new("and", IndirectX),
            0x31 => Opcode::new("and", IndirectY),

            0x2c => Opcode::new("bit", Absolute),
            0x24 => Opcode::new("bit", ZeroPage),

            0x49 => Opcode::new("eor", Immediate),
            0x45 => Opcode::new("eor", ZeroPage),
            0x55 => Opcode::new("eor", ZeroPageX),
            0x4d => Opcode::new("eor", Absolute),
            0x5d => Opcode::new("eor", AbsoluteX),
            0x59 => Opcode::new("eor", AbsoluteY),
            0x41 => Opcode::new("eor", IndirectX),
            0x51 => Opcode::new("eor", IndirectY),

            0x09 => Opcode::new("ora", Immediate),
            0x05 => Opcode::new("ora", ZeroPage),
            0x15 => Opcode::new("ora", ZeroPageX),
            0x0d => Opcode::new("ora", Absolute),
            0x1d => Opcode::new("ora", AbsoluteX),
            0x19 => Opcode::new("ora", AbsoluteY),
            0x01 => Opcode::new("ora", IndirectX),
            0x11 => Opcode::new("ora", IndirectY),

            0xc9 => Opcode::new("cmp", Immediate),
            0xc5 => Opcode::new("cmp", ZeroPage),
            0xd5 => Opcode::new("cmp", ZeroPageX),
            0xcd => Opcode::new("cmp", Absolute),
            0xdd => Opcode::new("cmp", AbsoluteX),
            0xd9 => Opcode::new("cmp", AbsoluteY),
            0xc1 => Opcode::new("cmp", IndirectX),
            0xd1 => Opcode::new("cmp", IndirectY),

            0xe0 => Opcode::new("cpx", Immediate),
            0xe4 => Opcode::new("cpx", ZeroPage),
            0xec => Opcode::new("cpx", Absolute),

            0xc0 => Opcode::new("cpy", Immediate),
            0xc4 => Opcode::new("cpy", ZeroPage),
            0xcc => Opcode::new("cpy", Absolute),

            // -- Control flow --
            0x4c => Opcode::new("jmp", Absolute),
            0x6c => Opcode::new("jmp", Indirect),
            0x20 => Opcode::new("jsr", Absolute),
            0x60 => Opcode::new("rts", Implied),
            0x00 => Opcode::new("brk", Implied),
            0x40 => Opcode::new("rti", Implied),

            0x90 => Opcode::new("bcc", AddrMode::Relative),
            0xb0 => Opcode::new("bcs", AddrMode::Relative),
            0xf0 => Opcode::new("beq", AddrMode::Relative),
            0x30 => Opcode::new("bmi", AddrMode::Relative),
            0xd0 => Opcode::new("bne", AddrMode::Relative),
            0x10 => Opcode::new("bpl", AddrMode::Relative),
            0x50 => Opcode::new("bvc", AddrMode::Relative),
            0x70 => Opcode::new("bvs", AddrMode::Relative),

            // Does nothing
            0xea => Opcode::new("nop", AddrMode::Implied),
            _ => return Err(CpuError::UnknownOpcode(byte)),
        };

        opcode.byte = byte;
        Ok(opcode)
    }
}
