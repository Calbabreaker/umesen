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
