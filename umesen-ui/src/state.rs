use egui::ahash::HashMap;

pub struct Texture {
    pub handle: egui::TextureHandle,
    pub image_buffer: egui::ColorImage,
}

impl Texture {
    pub fn update(&mut self) {
        self.handle
            .set(self.image_buffer.clone(), egui::TextureOptions::NEAREST);
    }
}

#[derive(Default)]
pub struct TextureMap {
    map: HashMap<String, Texture>,
}

impl TextureMap {
    pub fn get(
        &mut self,
        name: impl ToString,
        size: [usize; 2],
        ctx: &egui::Context,
    ) -> &mut Texture {
        self.map.entry(name.to_string()).or_insert_with(|| {
            let image_buffer = egui::ColorImage::new(size, egui::Color32::PLACEHOLDER);
            Texture {
                handle: ctx.load_texture("", image_buffer.clone(), egui::TextureOptions::NEAREST),
                image_buffer,
            }
        })
    }
}

#[derive(Default)]
pub struct State {
    pub emulator: umesen_core::Emulator,
    pub texture_map: TextureMap,
}

impl State {
    pub fn run_emulator(&mut self) {
        self.emulator.cpu.reset();
    }

    pub fn step_emulator(&mut self) {
        if let Err(err) = self.emulator.cpu.execute_next() {
            log::error!("{err}")
        }
    }
}
