use umesen_core::ppu::{
    PATTERN_TILE_COUNT, VramRegister, add_bit_planes, get_pattern_tile_addresses,
};

#[derive(Clone, Copy, PartialEq, Eq, Default, serde::Deserialize, serde::Serialize, Debug)]
enum Tab {
    #[default]
    Palettes,
    PatternTables,
    Nametables,
    Sprites,
}

impl crate::egui_util::UiList for Tab {
    fn pretty_name(&self) -> &'static str {
        match self {
            Self::Palettes => "Palettes",
            Self::PatternTables => "Pattern Tables",
            Self::Nametables => "Nametables",
            Self::Sprites => "Sprites",
        }
    }

    const LIST: &[Self] = &[
        Self::Sprites,
        Self::Palettes,
        Self::Nametables,
        Self::PatternTables,
    ];
}

pub fn show(ui: &mut egui::Ui, state: &mut crate::State) {
    let tab_open = crate::egui_util::ui_list_tab_group::<Tab>(ui);

    ui.style_mut().spacing.item_spacing = egui::vec2(5., 5.);
    ui.style_mut().spacing.interact_size.y = 0.;
    match tab_open {
        Tab::Palettes => {
            for row in 0..2 {
                ui.label(if row == 0 { "Background:" } else { "Sprite:" });
                ui.horizontal(|ui| {
                    for col in 0..4 {
                        show_pallete(ui, state.emu.ppu(), row * 4 + col);
                    }
                });
            }
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
        Tab::Sprites => {
            ui.horizontal(|ui| {
                show_oam_grid(ui, state);
                show_oam_info(ui, state.emu.ppu());
            });
        }
    }
}

fn show_pallete(ui: &mut egui::Ui, ppu: &umesen_core::Ppu, palette_id: u8) {
    let pixel_size = egui::Vec2::splat(15.);
    let (mut response, painter) =
        ui.allocate_painter(pixel_size * egui::vec2(4., 1.), egui::Sense::hover());
    for color in ppu.get_palette_colors(palette_id) {
        painter.rect_filled(
            egui::Rect::from_min_size(response.rect.min, pixel_size),
            0.,
            egui::Color32::from_rgb(color[0], color[1], color[2]),
        );
        response.rect.min.x += pixel_size.x;
    }
}

fn show_pattern_table(ui: &mut egui::Ui, state: &mut crate::State, table_number: u16) {
    let get_tile_info_fn = |tile_index, _| {
        let tile_index = tile_index as u16;
        let palette = [[0, 0, 0], [85, 85, 85], [170, 170, 170], [255, 255, 255]];
        (tile_index + table_number * PATTERN_TILE_COUNT, palette)
    };

    let config = UiPatternTilesConfig {
        name: format!("pattern{table_number}"),
        tile_count: [16, 16],
        image_scale: 3.,
    };
    ui.vertical(|ui| {
        show_pattern_tiles(ui, state, &config, get_tile_info_fn);
        let id = egui::Id::new(config.name.as_str());
        if let Some(i) = ui.memory_mut(|m| m.data.get_persisted::<usize>(id)) {
            ui.label(format!(
                "Address: ${:03x}0",
                (i as u16 + table_number * PATTERN_TILE_COUNT)
            ));
        }
    });
}

fn show_nametable(ui: &mut egui::Ui, state: &mut crate::State, table_number: u16) {
    let get_tile_info_fn = |tile_index, ppu: &umesen_core::Ppu| {
        let mut register = VramRegister::default();
        register.set(VramRegister::NAMETABLE, table_number);
        register.set(VramRegister::COARSE_X, tile_index as u16 % 32);
        register.set(VramRegister::COARSE_Y, tile_index as u16 / 32);

        let tile_number = ppu.registers.bus.peek_read(register.nametable_address()) as u16;
        let tile_attribute = ppu.registers.bus.peek_read(register.attribute_address());
        let palette_id = register.palette_id(tile_attribute);
        (
            tile_number + ppu.registers.control.background_table_offset(),
            ppu.get_palette_colors(palette_id),
        )
    };

    let config = UiPatternTilesConfig {
        name: format!("nametable{table_number}"),
        tile_count: [32, 30],
        image_scale: 1.,
    };
    show_pattern_tiles(ui, state, &config, get_tile_info_fn);
}

fn show_oam_info(ui: &mut egui::Ui, ppu: &umesen_core::Ppu) {
    ui.vertical(|ui| {
        ui.style_mut().override_text_style = Some(egui::TextStyle::Monospace);
        if let Some(i) = ui.memory_mut(|m| m.data.get_persisted("oam_grid".into())) {
            let mut sprite = ppu.registers.get_oam_sprite(i, 0).unwrap();
            ui.label(format!("Index: {i}"));
            ui.label(format!("Position (x,y): {}, {}", sprite.x, sprite.y));
            ui.label(format!(
                "Tile address: ${:03x}0",
                sprite.tile_number(&ppu.registers)
            ));

            ui.horizontal(|ui| {
                ui.label("Pallete:");
                show_pallete(ui, ppu, sprite.attributes.palette());
            });
            crate::egui_util::show_flags(ui, &mut sprite.attributes);
        }
    });
}

fn show_oam_grid(ui: &mut egui::Ui, state: &mut crate::State) {
    let get_tile_info_fn = |tile_index, ppu: &umesen_core::Ppu| {
        let sprite = ppu.registers.get_oam_sprite(tile_index, 0).unwrap();
        let palette = sprite.attributes.palette() + 4;
        (
            sprite.tile_number(&ppu.registers),
            ppu.get_palette_colors(palette),
        )
    };

    let config = UiPatternTilesConfig {
        name: "oam_grid".to_string(),
        tile_count: [8, 8],
        image_scale: 4.,
    };
    show_pattern_tiles(ui, state, &config, get_tile_info_fn);
}

struct UiPatternTilesConfig {
    name: String,
    tile_count: [usize; 2],
    image_scale: f32,
}

/// Show a grid of ppu pattern table tiles
/// get_tile_info_fn is ran for each tile in the grid and should return a tuple of
/// (patern tile number, pallete id) based on the input tile index
/// If a user clicked on a tile that tile will be highligted and the tile index will be stored in
/// egui persistent memory with the id being the name
fn show_pattern_tiles<'a>(
    ui: &mut egui::Ui,
    state: &'a mut crate::State,
    config: &UiPatternTilesConfig,
    get_tile_info_fn: impl Fn(usize, &'a umesen_core::Ppu) -> (u16, [[u8; 3]; 4]),
) {
    let [tile_count_x, tile_count_y] = config.tile_count;
    let id = egui::Id::new(&config.name);
    let image_size = [tile_count_x * 8, tile_count_y * 8];
    let texture = state.texture_map.get(config.name.clone(), image_size);
    let ppu = state.emu.ppu();

    let mut pixels = vec![egui::Color32::BLACK; image_size[0] * image_size[1]];
    for tile_y in 0..tile_count_y {
        for tile_x in 0..tile_count_x {
            let tile_index = (tile_y * tile_count_x) + tile_x;
            let (tile_number, palette) = get_tile_info_fn(tile_index, ppu);
            for y in 0..8 {
                let (lsb_address, msb_address) = get_pattern_tile_addresses(tile_number, y);

                for x in 0..8 {
                    let pixel_index = add_bit_planes(
                        ppu.registers.bus.peek_read(lsb_address),
                        ppu.registers.bus.peek_read(msb_address),
                        0b1000_0000 >> x,
                    );
                    let pixel_x = tile_x * 8 + x as usize;
                    let pixel_y = tile_y * 8 + y as usize;
                    let c = palette[pixel_index as usize];
                    pixels[pixel_y * image_size[0] + pixel_x] =
                        egui::Color32::from_rgb(c[0], c[1], c[2]);
                }
            }
        }
    }
    texture.update_pixels(pixels);

    let image_pos = ui.cursor().left_top();
    let response = ui.add(
        texture
            .image(ui)
            .sense(egui::Sense::CLICK)
            .fit_to_original_size(config.image_scale),
    );
    let size_scale = 8. * config.image_scale;
    if response
        .on_hover_cursor(egui::CursorIcon::PointingHand)
        .clicked()
    {
        let pos = ui.input(|i| i.pointer.interact_pos()).unwrap_or_default() - image_pos;
        let tile_pos = pos / size_scale;
        let tile_index = (tile_pos.y as usize * tile_count_x) + tile_pos.x as usize;
        if tile_index < tile_count_x * tile_count_y {
            ui.memory_mut(|m| m.data.insert_persisted(id, tile_index))
        }
    }

    if let Some(i) = ui.memory_mut(|m| m.data.get_persisted::<usize>(id)) {
        ui.painter().rect_stroke(
            egui::Rect::from_min_size(
                egui::Pos2::new((i % tile_count_x) as f32, (i / tile_count_x) as f32) * size_scale
                    + image_pos.to_vec2(),
                egui::Vec2::splat(size_scale),
            ),
            0.,
            (2., egui::Color32::RED),
            egui::StrokeKind::Outside,
        );
    }
}
