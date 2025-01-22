pub fn show(ui: &mut egui::Ui, emulator: &mut umesen_core::Emulator) {
    ui.style_mut().override_text_style = Some(egui::TextStyle::Monospace);
    ui.label(format!("PC: ${0:02x}", emulator.cpu.pc));
    ui.label(format!("SP: ${0:02x}", emulator.cpu.sp));
    ui.label(format!("A:  ${0:02x}", emulator.cpu.a));
    ui.label(format!("X:  ${0:02x}", emulator.cpu.x));
    ui.label(format!("Y:  ${0:02x}", emulator.cpu.y));
    ui.label(format!("FLAGS: {}", emulator.cpu.flags));

    if ui.button("Step (])").clicked() {
        emulator.step();
    }
    ui.separator();

    let mut disassembler = umesen_core::cpu::Disassembler::new(&emulator.cpu);

    let frame = egui::Frame::canvas(ui.style())
        .inner_margin(6.0)
        .outer_margin(6.0);
    egui::CollapsingHeader::new("Disassemble")
        .default_open(true)
        .show_unindented(ui, |ui| {
            frame.show(ui, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.label(egui::RichText::new(disassembler.disassemble_lines(1)).strong());
                    ui.label(disassembler.disassemble_lines(31));
                    ui.allocate_space(egui::Vec2::new(ui.available_width(), 0.));
                });
            });
        });
}
