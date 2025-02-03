use egui::ahash::HashMap;
use serde::{Deserialize, Serialize};

use crate::{button_map::ButtonMap, texture::Texture};

#[derive(Default, Deserialize, Serialize)]
pub struct State {
    #[serde(skip)]
    pub emu: Box<umesen_core::Emulator>,
    #[serde(skip)]
    pub texture_map: HashMap<String, Texture>,
    pub running: bool,
    pub last_egui_update_time: f64,
    pub ui_render_time: f32,
    pub speed: f64,
    pub button_maps: [ButtonMap; 2],
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

    pub fn run_emulator(&mut self) {
        self.emu.cpu.reset();
        self.running = true;
    }

    pub fn update_emulation(&mut self, ctx: &egui::Context) {
        let now = ctx.input(|i| i.time);
        let elapsed_secs = (now - self.last_egui_update_time) * self.speed;
        self.last_egui_update_time = now;

        if !self.running {
            return;
        }

        if let Err(err) = self.emu.clock(elapsed_secs.min(0.03)) {
            log::error!("Emulation stopped: {err}");
            self.running = false;
        }

        let frame_complete = self.emu.frame_complete();
        // Don't sync to screen if speed is less than 1 for debugging
        if self.speed == 1. {
            if frame_complete {
                self.update_ppu_texture();
            }
        } else {
            self.update_ppu_texture();
        }

        ctx.request_repaint();
    }

    pub fn update_ppu_texture(&mut self) {
        let pixels = &self.emu.ppu().screen_pixels;
        let texture = self.texture_map.get_mut("ppu_output").unwrap();
        for (i, color) in pixels.iter().enumerate() {
            texture.image_buffer.pixels[i] = to_egui_color(*color);
        }
        texture.update();
    }
}

#[inline]
pub fn to_egui_color(color: u32) -> egui::Color32 {
    let bytes = color.to_be_bytes();
    egui::Color32::from_rgb(bytes[0], bytes[1], bytes[2])
}
