pub fn show(ui: &mut egui::Ui, state: &mut crate::State) {
    ui.style_mut().override_text_style = Some(egui::TextStyle::Monospace);
    ui.label(format!("PC: ${0:02x}", state.emulator.cpu.pc));
    ui.label(format!("SP: ${0:02x}", state.emulator.cpu.sp));
    ui.label(format!("A:  ${0:02x}", state.emulator.cpu.a));
    ui.label(format!("X:  ${0:02x}", state.emulator.cpu.x));
    ui.label(format!("Y:  ${0:02x}", state.emulator.cpu.y));
    ui.label(format!("FLAGS: {}", state.emulator.cpu.flags));

    ui.horizontal(|ui| {
        if ui.button(if state.running { "⏸" } else { "⏵" }).clicked() {
            state.running = !state.running;
        }

        if ui.button("⟳").clicked() {
            state.emulator.cpu.reset();
        }

        if ui.button("Step").clicked() {
            state.running = false;
            state.emulator.step();
            state.update_ppu_texture();
        }

        if ui.button("Step Over").clicked() {
            state.running = false;
            let start_pc = state.emulator.cpu.pc;
            while state.emulator.cpu.pc <= start_pc {
                state.emulator.step();
            }
            state.update_ppu_texture();
        }

        if ui.button("Next Frame").clicked() {
            state.running = false;
            state.next_frame();
        }
    });

    ui.separator();

    let mut disassembler = umesen_core::cpu::Disassembler::new(&state.emulator.cpu);

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
