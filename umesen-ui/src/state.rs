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

    pub fn set_pixel(&mut self, x: usize, y: usize, color: egui::Color32) {
        debug_assert!(x < self.image_buffer.size[0] && y < self.image_buffer.size[1]);
        let index = y * self.image_buffer.size[1] + x;
        self.image_buffer.pixels[index] = color;
    }

    pub fn set_pixels(&mut self, get_pixel: impl Fn(usize, usize) -> egui::Color32) {
        for x in 0..self.image_buffer.size[0] {
            for y in 0..self.image_buffer.size[1] {
                self.set_pixel(x, y, get_pixel(x, y));
            }
        }
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
    pub running: bool,
    pub last_frame_time: f64,
}

impl State {
    pub fn run_emulator(&mut self) {
        self.emulator.cpu.reset();
        self.running = true;
    }

    pub fn step_emulator(&mut self) {
        if let Err(err) = self.emulator.cpu.execute_next() {
            log::error!("{err}")
        }
    }
}
