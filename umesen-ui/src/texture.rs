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
