pub fn show(ui: &mut egui::Ui, state: &mut crate::State) {
    egui::CollapsingHeader::new("Palettes")
        .default_open(true)
        .show_unindented(ui, |ui| {
            ui.style_mut().spacing.item_spacing = egui::vec2(5., 5.);
            ui.style_mut().spacing.interact_size.y = 0.;
            show_palette_row(ui, state, 0);
            show_palette_row(ui, state, 1);
        });
}

pub fn show_palette_row(ui: &mut egui::Ui, state: &mut crate::State, row: usize) {
    let ppu = &state.emulator.cpu.bus.ppu;
    let pixel_size = egui::Vec2::splat(10.);
    ui.horizontal(|ui| {
        for col in 0..4 {
            let (mut response, painter) =
                ui.allocate_painter(pixel_size * egui::vec2(4., 1.), egui::Sense::hover());
            let offset = row * 16 + col * 4;
            for i in 0..4 {
                let palette_index = ppu.bus.read_byte(0x3f00 + (offset + i) as u16);
                painter.rect_filled(
                    egui::Rect::from_min_size(response.rect.min, pixel_size),
                    0.,
                    to_egui_color(ppu.palette.get(palette_index)),
                );
                response.rect.min.x += pixel_size.x;
            }
        }
    });
}

fn to_egui_color(color: u32) -> egui::Color32 {
    let bytes = color.to_le_bytes();
    egui::Color32::from_rgb(bytes[0], bytes[1], bytes[2])
}
