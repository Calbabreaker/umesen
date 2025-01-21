use crate::{
    cpu::{AddrMode, Opcode},
    Cpu,
};

pub struct Dissassembler<'a> {
    cpu: &'a Cpu,
    pub current_address: u16,
}

impl<'a> Dissassembler<'a> {
    pub fn new(cpu: &'a Cpu) -> Self {
        Self {
            current_address: cpu.pc,
            cpu,
        }
    }

    pub fn dissassemble_next(&mut self) -> String {
        let start_address = self.current_address;
        let opcode_byte = self.cpu.bus.unclocked_read_byte(start_address);
        let opcode = match Opcode::from_byte(opcode_byte) {
            Ok(x) => x,
            Err(_) => return format!("??? 0x{0:02x}", opcode_byte),
        };

        self.current_address = self.current_address.wrapping_add(1);

        let operand = match opcode.addr_mode {
            AddrMode::Accumulator => "A".to_owned(),
            AddrMode::Implied => "".to_owned(),
            AddrMode::Immediate => format!("#{}", self.next_byte_hex()),
            AddrMode::ZeroPage => self.next_byte_hex().to_string(),
            AddrMode::ZeroPageX => format!("{},X", self.next_byte_hex()),
            AddrMode::ZeroPageY => format!("{},y", self.next_byte_hex()),
            AddrMode::Absolute => self.next_word_hex(),
            AddrMode::AbsoluteX | AddrMode::AbsoluteXForceClock => {
                format!("{},X", self.next_word_hex())
            }
            AddrMode::AbsoluteY | AddrMode::AbsoluteYForceClock => {
                format!("{},Y", self.next_word_hex())
            }
            AddrMode::Indirect => format!("({})", self.next_word_hex()),
            AddrMode::IndirectX => format!("({},X)", self.next_byte_hex()),
            AddrMode::IndirectY | AddrMode::IndirectYForceClock => {
                format!("({}),Y", self.next_byte_hex())
            }
            AddrMode::Relative => {
                let offset = self.next_byte() as i8;
                let address = (start_address + 2) as i16 + offset as i16;
                let sign = if offset >= 0 { '+' } else { '-' };
                format!("*{0}{1} (${2:04x})", sign, offset.abs(), address)
            }
        };

        format!("${0:04x}: {1} {2}", start_address, opcode.name, operand)
    }

    fn next_byte(&mut self) -> u8 {
        let address = self.current_address;
        self.current_address = self.current_address.wrapping_add(1);
        self.cpu.bus.unclocked_read_byte(address)
    }

    fn next_word(&mut self) -> u16 {
        let address = self.current_address;
        self.current_address = self.current_address.wrapping_add(2);
        self.cpu.bus.unclocked_read_word(address)
    }

    fn next_byte_hex(&mut self) -> String {
        format!("${0:02x}", self.next_byte())
    }

    fn next_word_hex(&mut self) -> String {
        format!("${0:04x}", self.next_word())
    }
}
