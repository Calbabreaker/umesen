use crate::Cpu;

/// High level struct for controlling the cpu
#[derive(Default)]
pub struct Emulator {
    pub cpu: Cpu,
}

impl Emulator {
    pub fn next_frame(&mut self) {
        todo!()
    }
}
