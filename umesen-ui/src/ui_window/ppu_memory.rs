use umesen_core::ppu::{add_bit_planes, TvRegister, PATTERN_TILE_COUNT};

#[derive(Clone, Copy, PartialEq, Eq, Default, serde::Deserialize, serde::Serialize)]
enum Tab {
    #[default]
    Palettes,
    PatternTables,
    Nametables,
    OamData,
}

impl crate::egui_util::UiList for Tab {
    fn pretty_name(&self) -> &'static str {
        match self {
            Self::Palettes => "Palettes",
            Self::PatternTables => "Pattern Tables",
            Self::Nametables => "Nametables",
            Self::OamData => "OAM Data",
        }
    }

    const LIST: &[Self] = &[
        Self::OamData,
        Self::Palettes,
        Self::Nametables,
        Self::PatternTables,
    ];
}

pub fn show(ui: &mut egui::Ui, state: &mut crate::State) {
    let tab_open = crate::egui_util::show_tab_group::<Tab>(ui);

    ui.style_mut().spacing.item_spacing = egui::vec2(5., 5.);
    ui.style_mut().spacing.interact_size.y = 0.;
    match tab_open {
        Tab::Palettes => {
            ui.label("Background:");
            show_palette_row(ui, state, 0);
            ui.label("Sprite:");
            show_palette_row(ui, state, 1);
        }
        Tab::PatternTables => {
            ui.horizontal(|ui| {
                show_pattern_table(ui, state, 0);
                show_pattern_table(ui, state, 1);
            });
        }
        Tab::Nametables => {
            let cart = state.emu.cartridge();
            let mirroring = cart.map(|c| c.mirroring()).unwrap_or_default();
            ui.label(format!("Mirroring: {mirroring:?}"));

            ui.style_mut().spacing.item_spacing = egui::vec2(0., 0.);
            for i in 0..2 {
                ui.horizontal(|ui| {
                    for j in 0..2 {
                        show_nametable(ui, state, i * 2 + j);
                    }
                });
            }
        }
        Tab::OamData => {
            ui.horizontal(|ui| {
                show_oam_grid(ui, state);

                egui::Frame::canvas(ui.style())
                    .inner_margin(6.0)
                    .outer_margin(egui::Margin {
                        left: 4,
                        ..Default::default()
                    })
                    .show(ui, |ui| {
                        ui.vertical(|ui| {
                            show_oam_list(ui, state.emu.ppu());
                        });
                    });
            });
        }
    }
}

fn show_palette_row(ui: &mut egui::Ui, state: &mut crate::State, row: u8) {
    let ppu = &state.emu.cpu.bus.ppu;
    let pixel_size = egui::Vec2::splat(15.);
    ui.horizontal(|ui| {
        for col in 0..4 {
            let (mut response, painter) =
                ui.allocate_painter(pixel_size * egui::vec2(4., 1.), egui::Sense::hover());
            for color in ppu.get_palette_colors(row * 4 + col) {
                painter.rect_filled(
                    egui::Rect::from_min_size(response.rect.min, pixel_size),
                    0.,
                    crate::egui_util::to_egui_color(color),
                );
                response.rect.min.x += pixel_size.x;
            }
        }
    });
}

fn show_pattern_table(ui: &mut egui::Ui, state: &mut crate::State, table_number: u16) {
    let get_tile_info_fn = |tile_x, tile_y, _| {
        let tile_number = tile_y * 16 + tile_x;
        let palette = [0x000000ff, 0x555555ff, 0xaaaaaaff, 0xffffffff];
        (tile_number + table_number * PATTERN_TILE_COUNT, palette)
    };

    // 16 by 16 tiles with 8 pixels each
    let image = show_pattern_tiles(
        ui,
        format!("pattern{table_number}"),
        state,
        [16, 16],
        get_tile_info_fn,
    );

    ui.add(image.fit_to_original_size(2.));
}

fn show_nametable(ui: &mut egui::Ui, state: &mut crate::State, table_number: u16) {
    let get_tile_info_fn = |tile_x, tile_y, ppu: &umesen_core::Ppu| {
        let mut register = TvRegister::default();
        register.set(TvRegister::NAMETABLE, table_number);
        register.set(TvRegister::COARSE_X, tile_x);
        register.set(TvRegister::COARSE_Y, tile_y);

        let tile_number = ppu.registers.bus.read_u8(register.nametable_address()) as u16;
        let tile_attribute = ppu.registers.bus.read_u8(register.attribute_address());
        let palette_id = register.shift_attribute(tile_attribute);

        (
            tile_number + ppu.registers.control.background_table_offset(),
            ppu.get_palette_colors(palette_id),
        )
    };

    let image = show_pattern_tiles(
        ui,
        format!("nametable{table_number}"),
        state,
        // 32 by 30 tiles with 8 pixels each
        [32, 30],
        get_tile_info_fn,
    );
    ui.add(image.fit_to_original_size(1.));
}

fn show_oam_list(ui: &mut egui::Ui, ppu: &umesen_core::Ppu) {
    ui.style_mut().override_text_style = Some(egui::TextStyle::Monospace);
    egui::ScrollArea::vertical().show(ui, |ui| {
        for (i, chunk) in ppu.registers.oam_data.chunks(4).enumerate() {
            if i != 0 {
                ui.separator();
            }
            let sprite = umesen_core::ppu::Sprite::new(chunk);
            ui.label(format!("INDEX: {i} ({}x{})", i % 8, i / 8));
            ui.label(format!("X: {}", sprite.x));
            ui.label(format!("Y: {}", sprite.y));
            ui.label(format!(
                "TILE: ${0:02x}",
                sprite.tile_number(&ppu.registers)
            ));
            ui.label(format!("ATTR: {}", sprite.attributes));
        }
    });
}

fn show_oam_grid(ui: &mut egui::Ui, state: &mut crate::State) {
    let get_tile_info_fn = |tile_x, tile_y, ppu: &umesen_core::Ppu| {
        let index = ((tile_y * 8 + tile_x) * 4) as usize;
        let oam = &ppu.registers.oam_data[index..index + 4];
        let sprite = umesen_core::ppu::Sprite::new(oam);
        let palette = sprite.attributes.palette() + 4;
        (
            sprite.tile_number(&ppu.registers),
            ppu.get_palette_colors(palette),
        )
    };

    let image = show_pattern_tiles(ui, "oam_grid".to_string(), state, [8, 8], get_tile_info_fn);
    ui.add(image.fit_to_original_size(4.));
}

fn show_pattern_tiles<'a>(
    ui: &mut egui::Ui,
    name: String,
    state: &'a mut crate::State,
    tile_size: [usize; 2],
    get_tile_info_fn: impl Fn(u16, u16, &'a umesen_core::Ppu) -> (u16, [u32; 4]),
) -> egui::Image<'a> {
    let image_size = [tile_size[0] * 8, tile_size[1] * 8];
    let default_fn = || crate::Texture::new(image_size, ui.ctx());
    let texture = state.texture_map.entry(name).or_insert_with(default_fn);
    let ppu = state.emu.ppu();

    for tile_y in 0..tile_size[1] {
        for tile_x in 0..tile_size[0] {
            let (tile_number, palette) = get_tile_info_fn(tile_x as u16, tile_y as u16, ppu);
            for y in 0..8 {
                let bus = &ppu.registers.bus;
                let (lsb_plane, msb_plane) = bus.read_pattern_tile_planes(tile_number, y);

                // Get a value between 0 and 3
                for x in 0..8 {
                    let pixel_index = add_bit_planes(lsb_plane << x, msb_plane << x, 0b1000_0000);
                    let pixel_x = tile_x * 8 + x as usize;
                    let pixel_y = tile_y * 8 + y as usize;
                    let i = pixel_y * image_size[0] + pixel_x;
                    texture.image_buffer.pixels[i] =
                        crate::egui_util::to_egui_color(palette[pixel_index as usize]);
                }
            }
        }
    }

    texture.update();
    egui::Image::new(&texture.handle)
}
