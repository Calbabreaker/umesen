use umesen_core::ppu::{add_bit_planes, Control, TvRegister};

use crate::state::to_egui_color;

pub fn show(ui: &mut egui::Ui, state: &mut crate::State) {
    ui.style_mut().spacing.item_spacing = egui::vec2(5., 5.);
    ui.style_mut().spacing.interact_size.y = 0.;

    egui::CollapsingHeader::new("Palettes")
        .default_open(true)
        .show_unindented(ui, |ui| {
            for i in 0..2 {
                show_palette_row(ui, state, i);
            }
        });

    egui::CollapsingHeader::new("Pattern tables")
        .default_open(true)
        .show_unindented(ui, |ui| {
            ui.horizontal(|ui| {
                for i in 0..2 {
                    show_pattern_table(ui, state, i);
                }
            });
        });

    let cart = state.emu.cartridge();
    let mirroring = cart.map(|c| c.mirroring()).unwrap_or_default();
    egui::CollapsingHeader::new(format!("Nametables ({:?} mirroring)", mirroring))
        .default_open(true)
        .show_unindented(ui, |ui| {
            for i in 0..2 {
                ui.horizontal(|ui| {
                    for j in 0..2 {
                        show_nametable(ui, state, i * 2 + j);
                    }
                });
            }
        });
}

fn show_palette_row(ui: &mut egui::Ui, state: &mut crate::State, row: u8) {
    let ppu = &state.emu.cpu.bus.ppu;
    let pixel_size = egui::Vec2::splat(10.);
    ui.horizontal(|ui| {
        for col in 0..4 {
            let (mut response, painter) =
                ui.allocate_painter(pixel_size * egui::vec2(4., 1.), egui::Sense::hover());
            let palette_id = row * 4 + col;
            for i in 0..4 {
                painter.rect_filled(
                    egui::Rect::from_min_size(response.rect.min, pixel_size),
                    0.,
                    to_egui_color(ppu.get_palette_color(palette_id, i)),
                );
                response.rect.min.x += pixel_size.x;
            }
        }
    });
}

fn show_pattern_table(ui: &mut egui::Ui, state: &mut crate::State, table_number: u8) {
    let get_tile_info_fn = |tile_x, tile_y, _| {
        let tile_number = tile_y * 16 + tile_x;
        let palette = [0x000000ff, 0x555555ff, 0xaaaaaaff, 0xffffffff];
        (tile_number, palette)
    };

    // 16 by 16 tiles with 8 pixels each
    show_ppu_mem_tiles(
        ui,
        format!("pattern{table_number}"),
        state,
        [16, 16],
        table_number,
        get_tile_info_fn,
        2.,
    );
}

fn show_nametable(ui: &mut egui::Ui, state: &mut crate::State, table_number: u16) {
    let get_tile_info_fn = |tile_x, tile_y, ppu: &umesen_core::Ppu| {
        let mut register = TvRegister::default();
        register.set(TvRegister::NAMETABLE, table_number);
        register.set(TvRegister::COARSE_X, tile_x);
        register.set(TvRegister::COARSE_Y, tile_y);

        let tile_number = ppu.registers.bus.read_u8(register.nametable_address());
        let tile_attribute = ppu.registers.bus.read_u8(register.attribute_address());
        let palette_id = register.shift_attribute(tile_attribute);

        let mut palette = [0; 4];
        for (i, color) in palette.iter_mut().enumerate() {
            *color = ppu.get_palette_color(palette_id, i as u8);
        }

        (tile_number, palette)
    };

    let control = &state.emu.ppu().registers.control;
    let pattern_table_number = control.contains(Control::BACKGROUND_TABLE_OFFSET) as u8;

    // 32 by 30 tiles with 8 pixels each
    show_ppu_mem_tiles(
        ui,
        format!("nametable{table_number}"),
        state,
        [32, 30],
        pattern_table_number,
        get_tile_info_fn,
        1.,
    );
}

fn show_ppu_mem_tiles<'a>(
    ui: &mut egui::Ui,
    name: String,
    state: &'a mut crate::State,
    tile_size: [usize; 2],
    pattern_table_number: u8,
    get_tile_info_fn: impl Fn(u8, u8, &'a umesen_core::Ppu) -> (u8, [u32; 4]),
    pixel_size: f32,
) {
    let image_size = [tile_size[0] * 8, tile_size[1] * 8];
    let default_fn = || crate::Texture::new(image_size, ui.ctx());
    let texture = state.texture_map.entry(name).or_insert_with(default_fn);
    let ppu = state.emu.ppu();

    for tile_y in 0..tile_size[1] {
        for tile_x in 0..tile_size[0] {
            let (tile_number, palette) = get_tile_info_fn(tile_x as u8, tile_y as u8, ppu);
            for y in 0..8 {
                let (lsb_plane, msb_plane) = ppu.registers.bus.read_pattern_tile_planes(
                    tile_number,
                    pattern_table_number,
                    y as u8,
                );

                // Get a value between 0 and 3
                for x in 0..8 {
                    let pixel_index = add_bit_planes(lsb_plane << x, msb_plane << x, 0b1000_0000);
                    let i = (tile_y * 8 + y) * image_size[0] + (tile_x * 8 + x);
                    texture.image_buffer.pixels[i] = to_egui_color(palette[pixel_index as usize]);
                }
            }
        }
    }

    texture.update();
    ui.add(egui::Image::new(&texture.handle).fit_to_original_size(pixel_size));
}
