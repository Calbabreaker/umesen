use crate::{
    cpu::{AddrMode, Opcode},
    Cpu, CpuError,
};

pub struct Disassembler<'a> {
    cpu: &'a Cpu,
    pub current_address: u16,
}

impl<'a> Disassembler<'a> {
    pub fn new(cpu: &'a Cpu) -> Self {
        Self {
            current_address: cpu.pc,
            cpu,
        }
    }

    pub fn disassemble_lines(&mut self, mut amount: usize) -> String {
        let mut output = String::new();
        loop {
            self.disassemble_next(&mut output).unwrap();
            if amount <= 1 {
                break;
            }
            output += "\n";
            amount -= 1;
        }
        output
    }

    pub fn disassemble_next(&mut self, mut f: impl std::fmt::Write) -> std::fmt::Result {
        write!(f, "${:04x}: ", self.current_address)?;

        let opcode_byte = self.cpu.bus.unclocked_read_byte(self.current_address);
        let opcode = match Opcode::from_byte(opcode_byte) {
            Ok(x) => x,
            Err(CpuError::UnknownOpcode(byte)) => {
                write!(f, "??? (${byte:02x})")?;
                return Ok(());
            }
        };

        self.current_address = self.current_address.wrapping_add(1);
        write!(f, "{} ", opcode.name)?;

        match opcode.addr_mode {
            AddrMode::Accumulator => write!(f, "A")?,
            AddrMode::Implied => (),
            AddrMode::Immediate => write!(f, "#{}", self.next_byte())?,
            AddrMode::ZeroPage => write!(f, "{}", self.next_byte())?,
            AddrMode::ZeroPageX => write!(f, "{},X", self.next_byte())?,
            AddrMode::ZeroPageY => write!(f, "{},Y", self.next_byte())?,
            AddrMode::Absolute => write!(f, "{}", self.next_word())?,
            AddrMode::AbsoluteX | AddrMode::AbsoluteXForceClock => {
                write!(f, "{},X", self.next_word())?
            }
            AddrMode::AbsoluteY | AddrMode::AbsoluteYForceClock => {
                write!(f, "{},Y", self.next_word())?
            }
            AddrMode::Indirect => write!(f, "[{}]", self.next_word())?,
            AddrMode::IndirectX => write!(f, "[{},X]", self.next_byte())?,
            AddrMode::IndirectY | AddrMode::IndirectYForceClock => {
                write!(f, "[{}],Y", self.next_byte())?
            }
            AddrMode::Relative => {
                let offset = self.next_byte().0 as i16;
                let address = U16HexDisplay((self.current_address as i16 + offset) as u16);
                let sign = if offset >= 0 { '+' } else { '-' };
                write!(f, "*{sign}{} ({address})", offset.abs())?
            }
        };

        Ok(())
    }

    fn next_byte(&mut self) -> U8HexDisplay {
        let address = self.current_address;
        self.current_address = self.current_address.wrapping_add(1);
        U8HexDisplay(self.cpu.bus.unclocked_read_byte(address))
    }

    fn next_word(&mut self) -> U16HexDisplay {
        U16HexDisplay(self.next_byte().0 as u16 | ((self.next_byte().0 as u16) << 8))
    }
}

struct U16HexDisplay(u16);
struct U8HexDisplay(u8);

impl std::fmt::Display for U8HexDisplay {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "${0:02x}", self.0)
    }
}

impl std::fmt::Display for U16HexDisplay {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "${0:04x}", self.0)
    }
}
