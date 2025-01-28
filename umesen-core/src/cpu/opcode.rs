/// Addressing modes for instructions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AddrMode {
    Implied,
    Accumulator,
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

pub struct Opcode {
    pub name: &'static str,
    pub addr_mode: AddrMode,
    pub byte: u8,
}

impl Opcode {
    pub fn new(name: &'static str, addr_mode: AddrMode) -> Self {
        Self {
            name,
            addr_mode,
            byte: 0,
        }
    }

    /// Convert opcode byte into opcode name with addressing mode including unofficial ones
    pub fn from_byte(byte: u8) -> Option<Self> {
        use AddrMode::*;
        let mut opcode = match byte {
            // -- Stack --
            0x48 => Opcode::new("pha", Implied),
            0x08 => Opcode::new("php", Implied),
            0x68 => Opcode::new("pla", Implied),
            0x28 => Opcode::new("plp", Implied),

            // -- Shift and rotate --
            0x0a => Opcode::new("asl", Accumulator),
            0x06 => Opcode::new("asl", ZeroPage),
            0x16 => Opcode::new("asl", ZeroPageX),
            0x0e => Opcode::new("asl", Absolute),
            0x1e => Opcode::new("asl", AbsoluteXForceClock),

            0x4a => Opcode::new("lsr", Accumulator),
            0x46 => Opcode::new("lsr", ZeroPage),
            0x56 => Opcode::new("lsr", ZeroPageX),
            0x4e => Opcode::new("lsr", Absolute),
            0x5e => Opcode::new("lsr", AbsoluteXForceClock),

            0x2a => Opcode::new("rol", Accumulator),
            0x26 => Opcode::new("rol", ZeroPage),
            0x36 => Opcode::new("rol", ZeroPageX),
            0x2e => Opcode::new("rol", Absolute),
            0x3e => Opcode::new("rol", AbsoluteXForceClock),

            0x6a => Opcode::new("ror", Accumulator),
            0x66 => Opcode::new("ror", ZeroPage),
            0x76 => Opcode::new("ror", ZeroPageX),
            0x6e => Opcode::new("ror", Absolute),
            0x7e => Opcode::new("ror", AbsoluteXForceClock),

            0x07 => Opcode::new("slo", ZeroPage),
            0x17 => Opcode::new("slo", ZeroPageX),
            0x0f => Opcode::new("slo", Absolute),
            0x1f => Opcode::new("slo", AbsoluteX),
            0x1b => Opcode::new("slo", AbsoluteY),
            0x03 => Opcode::new("slo", IndirectX),
            0x13 => Opcode::new("slo", IndirectY),

            0x27 => Opcode::new("rla", ZeroPage),
            0x37 => Opcode::new("rla", ZeroPageX),
            0x2f => Opcode::new("rla", Absolute),
            0x3f => Opcode::new("rla", AbsoluteX),
            0x3b => Opcode::new("rla", AbsoluteY),
            0x23 => Opcode::new("rla", IndirectX),
            0x33 => Opcode::new("rla", IndirectY),

            0x47 => Opcode::new("sre", ZeroPage),
            0x57 => Opcode::new("sre", ZeroPageX),
            0x4f => Opcode::new("sre", Absolute),
            0x5f => Opcode::new("sre", AbsoluteX),
            0x5b => Opcode::new("sre", AbsoluteY),
            0x43 => Opcode::new("sre", IndirectX),
            0x53 => Opcode::new("sre", IndirectY),

            0x67 => Opcode::new("rra", ZeroPage),
            0x77 => Opcode::new("rra", ZeroPageX),
            0x6f => Opcode::new("rra", Absolute),
            0x7f => Opcode::new("rra", AbsoluteX),
            0x7b => Opcode::new("rra", AbsoluteY),
            0x63 => Opcode::new("rra", IndirectX),
            0x73 => Opcode::new("rra", IndirectY),

            // -- Arithmetic --
            0x69 => Opcode::new("adc", Immediate),
            0x65 => Opcode::new("adc", ZeroPage),
            0x75 => Opcode::new("adc", ZeroPageX),
            0x6d => Opcode::new("adc", Absolute),
            0x7d => Opcode::new("adc", AbsoluteX),
            0x79 => Opcode::new("adc", AbsoluteY),
            0x61 => Opcode::new("adc", IndirectX),
            0x71 => Opcode::new("adc", IndirectY),

            0xe9 | 0xeb => Opcode::new("sbc", Immediate),
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

            0xe7 => Opcode::new("isc", ZeroPage),
            0xf7 => Opcode::new("isc", ZeroPageX),
            0xef => Opcode::new("isc", Absolute),
            0xff => Opcode::new("isc", AbsoluteX),
            0xfb => Opcode::new("isc", AbsoluteY),
            0xe3 => Opcode::new("isc", IndirectX),
            0xf3 => Opcode::new("isc", IndirectY),

            0xc7 => Opcode::new("dcp", ZeroPage),
            0xd7 => Opcode::new("dcp", ZeroPageX),
            0xcf => Opcode::new("dcp", Absolute),
            0xdf => Opcode::new("dcp", AbsoluteX),
            0xdb => Opcode::new("dcp", AbsoluteY),
            0xc3 => Opcode::new("dcp", IndirectX),
            0xd3 => Opcode::new("dcp", IndirectY),

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

            0xa7 => Opcode::new("lax", ZeroPage),
            0xb7 => Opcode::new("lax", ZeroPageY),
            0xaf => Opcode::new("lax", Absolute),
            0xbf => Opcode::new("lax", AbsoluteY),
            0xa3 => Opcode::new("lax", IndirectX),
            0xb3 => Opcode::new("lax", IndirectY),

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

            0x87 => Opcode::new("sax", ZeroPage),
            0x97 => Opcode::new("sax", ZeroPageY),
            0x8f => Opcode::new("sax", Absolute),
            0x83 => Opcode::new("sax", IndirectX),

            // -- Register transfers --
            0xaa => Opcode::new("tax", Implied),
            0xa8 => Opcode::new("tay", Implied),
            0xba => Opcode::new("tsx", Implied),
            0x8a => Opcode::new("txa", Implied),
            0x9a => Opcode::new("txs", Implied),
            0x98 => Opcode::new("tya", Implied),

            // -- Flag clear and set --
            0x18 => Opcode::new("clc", Implied),
            0xd8 => Opcode::new("cld", Implied),
            0x58 => Opcode::new("cli", Implied),
            0xb8 => Opcode::new("clv", Implied),
            0x38 => Opcode::new("sec", Implied),
            0xf8 => Opcode::new("sed", Implied),
            0x78 => Opcode::new("sei", Implied),

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

            0x90 => Opcode::new("bcc", Relative),
            0xb0 => Opcode::new("bcs", Relative),
            0xf0 => Opcode::new("beq", Relative),
            0x30 => Opcode::new("bmi", Relative),
            0xd0 => Opcode::new("bne", Relative),
            0x10 => Opcode::new("bpl", Relative),
            0x50 => Opcode::new("bvc", Relative),
            0x70 => Opcode::new("bvs", Relative),

            // Nop madness but like why are there so many nop opcodes
            0xea | 0x1a | 0x3a | 0x5a | 0x7a | 0xda | 0xfa => Opcode::new("nop", Implied),
            0x80 => Opcode::new("nop", Immediate),
            0x04 | 0x44 | 0x64 => Opcode::new("nop", ZeroPage),
            0x14 | 0x34 | 0x54 | 0x74 | 0xd4 | 0xf4 => Opcode::new("nop", ZeroPageX),
            0x0c => Opcode::new("nop", Absolute),
            0x1c | 0x3c | 0x5c | 0x7c | 0xdc | 0xfc => Opcode::new("nop", AbsoluteX),
            _ => return None,
        };

        opcode.byte = byte;
        Some(opcode)
    }
}
