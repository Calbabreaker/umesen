use umesen_core::Emulator;

pub fn show(ui: &mut egui::Ui, emulator: &mut Emulator) {
    ui.style_mut().override_text_style = Some(egui::TextStyle::Monospace);

    let frame = egui::Frame::canvas(ui.style())
        .inner_margin(6.0)
        .outer_margin(6.0);
    frame.show(ui, |ui| {
        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.label(format!("{}", emulator.cpu.bus));
        });
    });
}
