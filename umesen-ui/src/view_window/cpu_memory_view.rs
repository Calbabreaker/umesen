use umesen_core::Emulator;

pub fn show(ui: &mut egui::Ui, emulator: &mut Emulator) {
    ui.style_mut().override_text_style = Some(egui::TextStyle::Monospace);

    let frame = egui::Frame::canvas(ui.style())
        .inner_margin(6.0)
        .outer_margin(6.0);
    frame.show(ui, |ui| {
        egui::ScrollArea::vertical().show(ui, |ui| {
            for i in 0..0x1000 {
                let line_address_start = i * 0x10;
                let line_text = (0..0x10)
                    .map(|i| {
                        let byte = emulator.cpu.bus.unclocked_read_byte(line_address_start + i);
                        format!("{byte:02x}")
                    })
                    .collect::<Vec<_>>()
                    .join(" ");

                ui.label(format!("${line_address_start:04x}: {line_text}"));
            }
        });
    });
}
