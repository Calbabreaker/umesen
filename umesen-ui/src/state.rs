use egui::ahash::HashMap;

use crate::{ActionKind, texture::Texture};

#[derive(serde::Deserialize, serde::Serialize, Default)]
#[serde(default)]
pub struct Preferences {
    pub key_action_map: crate::KeyActionMap,
    pub allow_illegal_press: bool,
    pub allow_unlimted_sprites: bool,
}

#[derive(Default)]
pub struct State {
    pub emu: Box<umesen_core::Emulator>,
    pub texture_map: HashMap<String, Texture>,
    pub ui_render_time: f32,
    pub save_states: std::collections::HashMap<u8, umesen_core::Cpu>,
    pub selected_quick_save: u8,
    pub counter: u32,
}

impl State {
    pub fn update_emulation(&mut self, ctx: &egui::Context) {
        match self.emu.update() {
            Ok(frame_complete) => {
                // Don't sync to screen if speed is less than 1 for debugging
                if frame_complete || self.emu.speed < 1. {
                    self.update_ppu_texture();
                    ctx.request_repaint();
                }
                if frame_complete {
                    self.update_ppu_texture();
                }
            }
            Err(err) => {
                log::warn!("CPU halted: {err}");
                self.emu.running = false;
            }
        }
    }

    pub fn update_ppu_texture(&mut self) {
        let pixels = &self.emu.ppu().screen_pixels;
        let texture = self
            .texture_map
            .entry("ppu_output".into())
            .or_insert_with(|| {
                crate::Texture::new([umesen_core::ppu::WIDTH, umesen_core::ppu::HEIGHT])
            });
        for (i, color) in pixels.iter().enumerate() {
            texture.image_buffer.pixels[i] = egui::Color32::from_rgb(color[0], color[1], color[2]);
        }
    }

    pub fn do_action(&mut self, action: ActionKind) {
        match action {
            ActionKind::Reset => {
                self.emu.cpu.reset();
                self.emu.running = true;
            }
            ActionKind::PauseResume => self.emu.running = !self.emu.running,
            ActionKind::Step => {
                self.emu.running = false;
                self.emu.cpu.execute_next().ok();
                self.update_ppu_texture();
            }
            ActionKind::QuickSave => {
                self.save_states
                    .insert(self.selected_quick_save, self.emu.cpu.clone());
            }
            ActionKind::QuickLoad => {
                if let Some(cpu) = self.save_states.get(&self.selected_quick_save) {
                    self.emu.cpu = cpu.clone();
                }
            }
            ActionKind::ControllerInput(..) => unreachable!(),
        }
    }
}
