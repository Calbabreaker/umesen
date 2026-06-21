pub struct Texture {
    handle: Option<egui::TextureHandle>,
    pub image_buffer: egui::ColorImage,
}

impl Texture {
    pub fn new(size: [usize; 2]) -> Self {
        Self {
            handle: None,
            image_buffer: egui::ColorImage::filled(size, egui::Color32::BLACK),
        }
    }

    pub fn image(&mut self, ui: &egui::Ui) -> egui::Image<'_> {
        if let Some(handle) = self.handle.as_mut() {
            handle.set(self.image_buffer.clone(), egui::TextureOptions::NEAREST);
        } else {
            self.handle = Some(ui.ctx().load_texture(
                "",
                self.image_buffer.clone(),
                egui::TextureOptions::NEAREST,
            ));
        }
        egui::Image::new(self.handle.as_ref().unwrap())
    }
}

#[derive(Default)]
pub struct TextureMap(pub egui::ahash::HashMap<String, Texture>);

impl TextureMap {
    pub fn update_ppu_texture(&mut self, pixels: &umesen_core::ppu::ScreenPixels) {
        let texture = self.0.entry("ppu_output".into()).or_insert_with(|| {
            Texture::new([umesen_core::ppu::WIDTH, umesen_core::ppu::HEIGHT]) //
        });
        for (i, color) in pixels.iter().enumerate() {
            texture.image_buffer.pixels[i] = egui::Color32::from_rgb(color[0], color[1], color[2]);
        }
    }

    pub fn get(&mut self, name: impl ToString, size: [usize; 2]) -> &mut Texture {
        self.0
            .entry(name.to_string())
            .or_insert_with(|| Texture::new(size))
    }
}
