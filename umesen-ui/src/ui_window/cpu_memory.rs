pub fn show(ui: &mut egui::Ui, state: &mut crate::State) {
    ui.style_mut().override_text_style = Some(egui::TextStyle::Monospace);

    let frame = egui::Frame::canvas(ui.style())
        .inner_margin(6.0)
        .outer_margin(6.0);
    frame.show(ui, |ui| {
        let total_rows = 0x1000;
        let row_height =
            ui.text_style_height(&egui::TextStyle::Monospace) - ui.spacing().item_spacing.y;
        egui::ScrollArea::vertical().show_rows(ui, row_height, total_rows, |ui, row_range| {
            let mut output = String::new();
            let bus = &state.emu.cpu.bus;
            bus.dump_memory(row_range, &mut output).unwrap();
            for line in output.lines() {
                ui.label(line);
            }
        });
    });
}
