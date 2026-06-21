use crate::{ActionKind, texture::TextureMap};

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
    pub texture_map: TextureMap,
    pub ui_render_time: f32,
    pub save_states: std::collections::HashMap<u8, umesen_core::Cpu>,
    pub selected_quick_save: u8,
    pub counter: u32,
}

impl State {
    pub fn update_emulation(&mut self, ctx: &egui::Context) {
        if let Err(err) = self
            .emu
            .update(|pixels| self.texture_map.update_ppu_texture(pixels))
        {
            log::warn!("CPU halted: {err}");
            self.emu.running = false;
        }

        if self.emu.speed < 1. {
            self.texture_map
                .update_ppu_texture(&self.emu.ppu().screen_pixels);
        }
        if self.emu.running {
            ctx.request_repaint();
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
            ActionKind::NextFrame => {
                self.emu.running = false;
                self.emu.next_frame().ok();
            }
            ActionKind::ControllerInput(..) => unreachable!(),
        }
        self.texture_map
            .update_ppu_texture(&self.emu.ppu().screen_pixels);
    }
}
