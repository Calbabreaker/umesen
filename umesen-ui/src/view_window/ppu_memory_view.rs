use umesen_core::ppu::Control;

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

    let cart = state.emulator.cartridge();
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

fn show_palette_row(ui: &mut egui::Ui, state: &mut crate::State, row: usize) {
    let ppu = &state.emulator.cpu.bus.ppu;
    let pixel_size = egui::Vec2::splat(10.);
    ui.horizontal(|ui| {
        for col in 0..4 {
            let (mut response, painter) =
                ui.allocate_painter(pixel_size * egui::vec2(4., 1.), egui::Sense::hover());
            let offset = row * 16 + col * 4;
            for x in 0..4 {
                if offset + x >= 64 {
                    dbg!(x, offset);
                }
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

fn show_pattern_table(ui: &mut egui::Ui, state: &mut crate::State, table_number: u16) {
    let name = format!("pattern{table_number}");
    let get_tile_number_fn = |tile_x, tile_y, _| (table_number << 8) | (tile_y * 16 + tile_x);
    let get_color_fn = |color_index| egui::Color32::from_gray(color_index * (255 / 3));

    // 16 by 16 tiles with 8 pixels each
    show_ppu_mem_tiles(ui, name, state, [16, 16], get_tile_number_fn, get_color_fn);
}

fn show_nametable(ui: &mut egui::Ui, state: &mut crate::State, table_number: u16) {
    let name = format!("nametable{table_number}");
    let get_tile_number_fn = |tile_x, tile_y, ppu: &umesen_core::Ppu| {
        let nametable_tile_index = tile_y * 32 + tile_x;
        let offset = 0x2000 + table_number * 0x400;
        let mut tile_number = ppu.registers.bus.read_byte(offset + nametable_tile_index) as u16;
        if ppu
            .registers
            .control
            .contains(Control::BACKGROUND_TABLE_OFFSET)
        {
            tile_number |= 1 << 8
        }
        tile_number
    };

    let get_color_fn = |color_index| egui::Color32::from_gray(color_index * (255 / 3));

    // 32 by 30 tiles with 8 pixels each
    show_ppu_mem_tiles(ui, name, state, [32, 30], get_tile_number_fn, get_color_fn);
}

fn show_ppu_mem_tiles<'a>(
    ui: &mut egui::Ui,
    name: String,
    state: &'a mut crate::State,
    tile_size: [usize; 2],
    get_tile_number_fn: impl Fn(u16, u16, &'a umesen_core::Ppu) -> u16,
    color_index_to_color32_fn: impl Fn(u8) -> egui::Color32,
) {
    let image_size = [tile_size[0] * 8, tile_size[1] * 8];
    let default_fn = || crate::Texture::new(image_size, ui.ctx());
    let texture = state.texture_map.entry(name).or_insert_with(default_fn);
    let ppu = state.emulator.ppu();

    for tile_y in 0..tile_size[1] {
        for tile_x in 0..tile_size[0] {
            let tile_number = get_tile_number_fn(tile_x as u16, tile_y as u16, ppu);
            for y in 0..8 {
                // From nes wiki: https://www.nesdev.org/wiki/PPU_pattern_tables#Addressing
                // DCBA98 76543210
                // ---------------
                // 0HNNNN NNNNPyyy
                // |||||| |||||+++- T: Fine Y offset, the row number within a tile
                // |||||| ||||+---- P: Bit plane (0: less significant bit; 1: more significant bit)
                // ||++++-++++----- N: Tile number from name table
                // |+-------------- H: Half of pattern table (0: "left"; 1: "right")
                // +--------------- 0: Pattern table is at $0000-$1FFF
                let byte_index = (tile_number << 4) + y as u16;
                let lsb_plane = ppu.registers.bus.read_byte(byte_index);
                let msb_plane = ppu.registers.bus.read_byte(byte_index + 8);
                // Get a value between 0 and 3
                for x in 0..8 {
                    let shift = 7 - x;
                    let lsb = (lsb_plane >> shift) & 1;
                    let msb = (msb_plane >> shift) & 1;
                    let color_index = lsb + (msb << 1);
                    let i = (tile_y * 8 + y) * image_size[0] + (tile_x * 8 + x);
                    texture.image_buffer.pixels[i] = color_index_to_color32_fn(color_index);
                }
            }
        }
    }

    texture.update();
    ui.add(egui::Image::new(&texture.handle).fit_to_original_size(2.));
}
