use serde::{Deserialize, Serialize};

bitflags::bitflags! {
    #[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize, Hash)]
    pub struct Button: u8 {
        const A = 1 << 0;
        const B = 1 << 1;
        const SELECT = 1 << 2;
        const START = 1 << 3;
        const UP = 1 << 4;
        const DOWN = 1 << 5;
        const LEFT = 1 << 6;
        const RIGHT = 1 << 7;
    }
}

impl Button {
    pub fn name(self) -> &'static str {
        match self {
            Button::A => "A",
            Button::B => "B",
            Button::SELECT => "Select",
            Button::START => "Start",
            Button::DOWN => "Down",
            Button::UP => "Up",
            Button::LEFT => "Left",
            Button::RIGHT => "Right",
            _ => unreachable!(),
        }
    }
}

#[derive(Default)]
pub struct Controller {
    strobe_active: bool,
    shift_register: u8,
    pub state: Button,
}

impl Controller {
    pub fn write_u8(&mut self, value: u8) {
        self.strobe_active = value & 0b1 != 0;
        if !self.strobe_active {
            self.shift_register = self.state.bits();
        }
    }

    pub fn read_u8(&mut self) -> u8 {
        let bit = self.shift_register & 0b1;
        self.shift_register >>= 1;
        bit
    }
}
