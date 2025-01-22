use umesen_core::Emulator;

pub fn show(ui: &mut egui::Ui, emulator: &mut Emulator) {
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

    let frame = egui::Frame::canvas(ui.style())
        .inner_margin(6.0)
        .outer_margin(6.0);
    egui::CollapsingHeader::new("Disassemble")
        .default_open(true)
        .show_unindented(ui, |ui| {
            frame.show(ui, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    let mut dissassembler = umesen_core::cpu::Disassembler::new(&emulator.cpu);
                    ui.label(egui::RichText::new(dissassembler.disassemble_next()).strong());
                    for _ in 0..32 {
                        ui.label(dissassembler.disassemble_next());
                    }
                    ui.allocate_space(egui::Vec2::new(ui.available_width(), 0.));
                });
            });
        });
}
