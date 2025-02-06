use std::fmt::Write;

#[allow(clippy::upper_case_acronyms)]
#[derive(Default, Clone, Copy, PartialEq, Eq, Debug)]
pub enum HexViewKind {
    #[default]
    CPU,
    PPU,
}

pub fn show(ui: &mut egui::Ui, state: &mut crate::State) {
    ui.style_mut().override_text_style = Some(egui::TextStyle::Monospace);
    egui::Frame::none().outer_margin(6.0).show(ui, |ui| {
        egui::ComboBox::from_label("")
            .selected_text(format!("{:?}", state.hex_view_kind))
            .show_ui(ui, |ui| {
                for kind in [HexViewKind::CPU, HexViewKind::PPU] {
                    ui.selectable_value(&mut state.hex_view_kind, kind, format!("{:?}", kind));
                }
            });
        ui.add_space(6.);

        let cpu_bus = &state.emu.cpu.bus;
        let ppu_bus = &state.emu.ppu().registers.bus;
        match state.hex_view_kind {
            HexViewKind::CPU => show_hex_view(ui, |address| cpu_bus.immut_read_u8(address), 0x1000),
            HexViewKind::PPU => show_hex_view(ui, |address| ppu_bus.read_u8(address), 0x400),
        }
    });
}

/// Display the memory dump of a range of rows
/// Each row shows 16 bytes so 0..0x1000 shows the entire range
fn show_hex_view(ui: &mut egui::Ui, read_u8_fn: impl Fn(u16) -> u8, total_rows: usize) {
    let frame = egui::Frame::canvas(ui.style()).inner_margin(6.0);
    frame.show(ui, |ui| {
        let row_height =
            ui.text_style_height(&egui::TextStyle::Monospace) - ui.spacing().item_spacing.y;
        egui::ScrollArea::vertical().show_rows(ui, row_height, total_rows, |ui, row_range| {
            for i in row_range {
                let line_address_start = i * 0x10;
                let mut output = format!("${line_address_start:04x}:");
                for i in 0..0x10 {
                    let byte = read_u8_fn(line_address_start as u16 + i);
                    write!(output, " {byte:02x}").unwrap();
                }

                ui.label(output);
            }
        });
    });
}
