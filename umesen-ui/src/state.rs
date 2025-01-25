use egui::ahash::HashMap;

pub struct Texture {
    pub handle: egui::TextureHandle,
    pub image_buffer: egui::ColorImage,
}

impl Texture {
    pub fn new(size: [usize; 2], ctx: &egui::Context) -> Self {
        let image_buffer = egui::ColorImage::new(size, egui::Color32::BLACK);
        Self {
            handle: ctx.load_texture("", image_buffer.clone(), egui::TextureOptions::NEAREST),
            image_buffer,
        }
    }

    pub fn update(&mut self) {
        self.handle
            .set(self.image_buffer.clone(), egui::TextureOptions::NEAREST);
    }
}

#[derive(Default)]
pub struct Stats {
    pub ui_render_time: f32,
    pub emulation_render_time: f32,
    pub frame_rate: f64,
}

#[derive(Default)]
pub struct State {
    pub emulator: umesen_core::Emulator,
    pub texture_map: HashMap<String, Texture>,
    pub running: bool,
    pub last_frame_time: f64,
    pub stats: Stats,
}

impl State {
    pub fn run_emulator(&mut self) {
        self.emulator.cpu.reset();
        // self.running = true;
    }

    pub fn next_frame(&mut self) {
        let start_time = std::time::Instant::now();
        if let Err(err) = self.emulator.next_frame_debug() {
            log::warn!("Emulation stopped: {err}");
            self.running = false;
        }
        // self.emulator.next_frame();
        self.update_ppu_texture();
        self.stats.emulation_render_time = start_time.elapsed().as_secs_f32();
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
    let bytes = color.to_be_bytes();
    egui::Color32::from_rgb(bytes[0], bytes[1], bytes[2])
}
