use egui::ahash::HashMap;

use crate::{texture::Texture, ActionKind};

#[derive(serde::Deserialize, serde::Serialize, Default)]
#[serde(default)]
pub struct Preferences {
    pub key_action_map: crate::KeyActionMap,
    pub allow_left_right: bool,
}

#[derive(Default)]
pub struct State {
    pub emu: Box<umesen_core::Emulator>,
    pub texture_map: HashMap<String, Texture>,
    pub running: bool,
    pub last_egui_update_time: f64,
    pub ui_render_time: f32,
    pub speed: f64,
    pub save_states: Box<[Option<umesen_core::Cpu>; 4]>,
    pub clocks_remaining: i32,
}

impl State {
    pub fn init(&mut self, ctx: &egui::Context) {
        let screen_size = [umesen_core::ppu::WIDTH, umesen_core::ppu::HEIGHT];
        self.speed = 1.;
        self.texture_map.insert(
            "ppu_output".to_string(),
            crate::Texture::new(screen_size, ctx),
        );
    }

    pub fn update_emulation(&mut self, ctx: &egui::Context) {
        let now = ctx.input(|i| i.time);
        let elapsed_secs = (now - self.last_egui_update_time).min(0.05) * self.speed;
        self.last_egui_update_time = now;

        if !self.running {
            return;
        }

        self.clocks_remaining +=
            (elapsed_secs * umesen_core::cpu::CLOCK_SPEED_HZ as f64).round() as i32;

        let mut a = false;
        while self.clocks_remaining > 0 {
            match self.emu.clock_until_frame(&mut self.clocks_remaining) {
                Ok(frame_complete) => {
                    // Don't sync to screen if speed is less than 1 for debugging
                    if frame_complete || self.speed < 1. {
                        a = true;
                        self.update_ppu_texture();
                    }
                }
                Err(err) => {
                    log::error!("Emulation stopped: {err}");
                    self.running = false;
                }
            }
        }

        if !a {
            println!("{} {}", self.emu.ppu().scanline, self.clocks_remaining);
        }

        ctx.request_repaint();
    }

    pub fn update_ppu_texture(&mut self) {
        let pixels = &self.emu.ppu().screen_pixels;
        let texture = self.texture_map.get_mut("ppu_output").unwrap();
        for (i, color) in pixels.iter().enumerate() {
            texture.image_buffer.pixels[i] = crate::egui_util::to_egui_color(*color);
        }
        texture.update();
    }

    pub fn do_action(&mut self, action: &ActionKind) {
        match action {
            ActionKind::Reset => {
                self.emu.cpu.reset();
                self.running = true;
            }
            ActionKind::Run(running) => self.running = *running,
            ActionKind::Step => {
                self.running = false;
                self.emu.step().ok();
                self.update_ppu_texture();
            }
            ActionKind::SaveState(number) => {
                self.save_states[*number as usize] = Some(self.emu.cpu.clone());
            }
            ActionKind::LoadState(number) => {
                if let Some(cpu) = self.save_states[*number as usize].clone() {
                    self.emu.cpu = cpu;
                }
            }
            ActionKind::ToggleSpeedIncrease => {
                if self.speed == 1. {
                    self.speed = 2.;
                } else {
                    self.speed = 1.;
                }
            }
            ActionKind::ControllerInput(..) => unreachable!(),
        }
    }
}
