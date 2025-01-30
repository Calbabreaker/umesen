pub fn show(ui: &mut egui::Ui, state: &mut crate::State) {
    ui.style_mut().override_text_style = Some(egui::TextStyle::Monospace);
    ui.label(format!("PC: ${0:02x}", state.emu.cpu.pc));
    ui.label(format!("SP: ${0:02x}", state.emu.cpu.sp));
    ui.label(format!("A:  ${0:02x}", state.emu.cpu.a));
    ui.label(format!("X:  ${0:02x}", state.emu.cpu.x));
    ui.label(format!("Y:  ${0:02x}", state.emu.cpu.y));
    ui.label(format!("FLAGS: {}", state.emu.cpu.flags));
    ui.label(format!("CYCLES: {}", state.emu.cpu.cpu_cycles_total));

    ui.horizontal(|ui| {
        ui.label("Speed:");
        ui.style_mut().spacing.slider_width = 150.0;
        ui.add(egui::Slider::new(&mut state.speed, 0.001..=1.000).step_by(0.001));
    });

    ui.horizontal(|ui| {
        if ui.button(if state.running { "⏸" } else { "⏵" }).clicked() {
            state.running = !state.running;
        }

        if ui.button("⟳").clicked() {
            state.emu.cpu.reset();
        }

        if ui.button("Step").clicked() {
            state.running = false;
            state.emu.step();
            state.update_ppu_texture();
        }

        if ui.button("Step Over").clicked() {
            state.running = false;
            let start_pc = state.emu.cpu.pc;
            while state.emu.cpu.pc <= start_pc {
                state.emu.step();
            }
            state.update_ppu_texture();
        }

        if ui.button("Next Frame").clicked() {
            state.running = false;
            state.emu.next_frame();
            state.update_ppu_texture();
        }
    });

    ui.separator();

    let mut disassembler = umesen_core::cpu::Disassembler::new(&state.emu.cpu);

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
