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

#[derive(Clone, Default)]
pub struct Controller {
    strobe_active: bool,
    shift_register: u8,
    /// Bit flag of all button held down states
    state: Button,
}

impl Controller {
    pub fn write_u8(&mut self, value: u8) {
        self.strobe_active = value & 0b1 != 0;
        if !self.strobe_active {
            self.shift_register = self.state.bits();
        }
    }

    pub fn read_u8(&mut self) -> u8 {
        if self.strobe_active {
            // Always return A when strobe active
            self.state.contains(Button::A) as u8
        } else {
            let bit = self.shift_register & 0b1;
            // Shift right and add 1 into high bit so it will read one when all bits have been read
            self.shift_register >>= 1;
            self.shift_register |= 0b1000_0000;
            bit
        }
    }

    /// Set the state of a button with an option to allow left and right or up and down to be pressed at the same time
    pub fn set(&mut self, button: Button, held: bool, allow_left_right: bool) {
        if !allow_left_right && held {
            let illegal_combo = [(Button::LEFT, Button::RIGHT), (Button::UP, Button::DOWN)];
            for (a, b) in illegal_combo {
                if (button == a && self.state.contains(b))
                    || (button == b && self.state.contains(a))
                {
                    return;
                }
            }
        }

        self.state.set(button, held);
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn read_correct() {
        let mut con = Controller::default();
        con.write_u8(1);
        assert_eq!(con.read_u8(), 0);
        con.state.set(Button::A, true);
        assert_eq!(con.read_u8(), 1);

        con.state.set(Button::SELECT, true);
        con.write_u8(0);
        con.state.set(Button::B, true);
        let mut out = 0;
        for i in 0..10 {
            out |= (con.read_u8() as u16) << i;
        }
        assert_eq!(out, 0b11_0000_0101);
    }
}
