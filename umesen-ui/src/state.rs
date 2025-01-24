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
pub struct State {
    pub emulator: umesen_core::Emulator,
    pub texture_map: HashMap<&'static str, Texture>,
    pub running: bool,
    pub last_frame_time: f64,
}

impl State {
    pub fn add_texture(&mut self, name: &'static str, size: [usize; 2], ctx: &egui::Context) {
        let image_buffer = egui::ColorImage::new(size, egui::Color32::BLACK);
        self.texture_map.insert(
            name,
            Texture {
                handle: ctx.load_texture(name, image_buffer.clone(), egui::TextureOptions::NEAREST),
                image_buffer,
            },
        );
    }

    pub fn run_emulator(&mut self) {
        self.emulator.cpu.reset();
        // self.running = true;
    }

    pub fn step_emulator(&mut self) {
        if let Err(err) = self.emulator.cpu.execute_next() {
            log::error!("{err}")
        }
        self.update_ppu_texture();
    }

    pub fn next_frame(&mut self) {
        self.emulator.next_frame();
        self.update_ppu_texture();
    }

    pub fn update_ppu_texture(&mut self) {
        let pixels = &self.emulator.ppu().screen_pixels;
        let texture = self.texture_map.get_mut("ppu_output").unwrap();
        for (i, color) in pixels.iter().enumerate() {
            texture.image_buffer.pixels[i] = to_egui_color(*color);
        }
        texture.update();
    }
}

pub fn to_egui_color(color: u32) -> egui::Color32 {
    let bytes = color.to_le_bytes();
    egui::Color32::from_rgb(bytes[0], bytes[1], bytes[2])
}
