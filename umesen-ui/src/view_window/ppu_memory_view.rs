use crate::state::to_egui_color;

pub fn show(ui: &mut egui::Ui, state: &mut crate::State) {
    ui.style_mut().spacing.item_spacing = egui::vec2(5., 5.);
    ui.style_mut().spacing.interact_size.y = 0.;

    egui::CollapsingHeader::new("Palettes")
        .default_open(true)
        .show_unindented(ui, |ui| {
            show_palette_row(ui, state, 0);
            show_palette_row(ui, state, 1);
        });

    egui::CollapsingHeader::new("Pattern tables")
        .default_open(true)
        .show_unindented(ui, |ui| {
            ui.horizontal(|ui| {
                show_pattern_table(ui, state, 0, "pattern_0");
                show_pattern_table(ui, state, 0x1000, "pattern_1");
            });
        });
}

fn show_palette_row(ui: &mut egui::Ui, state: &mut crate::State, row: usize) {
    let ppu = &state.emulator.cpu.bus.ppu;
    let pixel_size = egui::Vec2::splat(10.);
    ui.horizontal(|ui| {
        for col in 0..4 {
            let (mut response, painter) =
                ui.allocate_painter(pixel_size * egui::vec2(4., 1.), egui::Sense::hover());
            let offset = row * 16 + col * 4;
            for x in 0..4 {
                painter.rect_filled(
                    egui::Rect::from_min_size(response.rect.min, pixel_size),
                    0.,
                    to_egui_color(ppu.get_palette_color((offset + x) as u16)),
                );
                response.rect.min.x += pixel_size.x;
            }
        }
    });
}

fn show_pattern_table(
    ui: &mut egui::Ui,
    state: &mut crate::State,
    offset: u16,
    name: &'static str,
) {
    // 16 by 16 tiles with 8 pixels each
    let texture = state.texture_map.get_mut(name).unwrap();
    let ppu_bus = &state.emulator.ppu().registers.bus;

    texture.set_pixels(|x, y| {
        // Every 8 pixels (a tile) go to the next tile index
        let x_skip = x / 8 * (8 * 2);
        let y_skip = y / 8 * (8 * 2 * 16);

        let tile_byte_index = (x_skip + y_skip + y % 8) as u16;
        let shift = 7 - (x % 8);
        let lsb_plane = ppu_bus.read_byte(offset + tile_byte_index) >> shift;
        let msb_plane = ppu_bus.read_byte(offset + tile_byte_index + 8) >> shift;
        // Get a value between 0 and 3
        let pixel = (lsb_plane & 1) + ((msb_plane & 1) << 1);
        egui::Color32::from_gray(pixel * (255 / 3))
    });

    texture.update();
    ui.add(egui::Image::new(&texture.handle).fit_to_original_size(3.));
}
