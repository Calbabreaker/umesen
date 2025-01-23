pub fn show(ui: &mut egui::Ui, state: &mut crate::State) {
    ui.style_mut().override_text_style = Some(egui::TextStyle::Monospace);

    let frame = egui::Frame::canvas(ui.style())
        .inner_margin(6.0)
        .outer_margin(6.0);
    frame.show(ui, |ui| {
        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.label(format!("{}", state.emulator.cpu.bus));
        });
    });
}
