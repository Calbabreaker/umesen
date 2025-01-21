use umesen_core::Emulator;

#[derive(serde::Deserialize, serde::Serialize, Clone, Debug)]
pub enum ViewWindowKind {
    CpuState,
    ErrorPopup { title: String, message: String },
}

// Returns whether the window is still open
pub fn show(ctx: &egui::Context, emulator: &mut Emulator, kind: &ViewWindowKind) -> bool {
    let title = match kind {
        ViewWindowKind::CpuState => "Cpu state",
        ViewWindowKind::ErrorPopup { title, .. } => title.as_str(),
    };

    let mut open = true;
    let popup = egui::Window::new(title)
        .default_pos(ctx.screen_rect().size().to_pos2() / 2.)
        .open(&mut open)
        .resizable(false);

    popup.show(ctx, |ui| match kind {
        ViewWindowKind::CpuState => cpu_state_view(ui, emulator),
        ViewWindowKind::ErrorPopup { title, message } => {
            ui.centered_and_justified(|ui| {
                ui.heading(title);
                ui.label(egui::RichText::new(message).color(egui::Color32::RED));
            });
        }
    });

    open
}

fn cpu_state_view(ui: &mut egui::Ui, emulator: &mut Emulator) {
    ui.style_mut().override_text_style = Some(egui::TextStyle::Monospace);
    ui.label(format!("PC: 0x{0:02x}", emulator.cpu.pc));
    ui.label(format!("A:  0x{0:02x}", emulator.cpu.a));
    ui.label(format!("X:  0x{0:02x}", emulator.cpu.x));
    ui.label(format!("Y:  0x{0:02x}", emulator.cpu.y));

    egui::CollapsingHeader::new("Disassemble")
        .default_open(true)
        .show(ui, |ui| {
            let mut dissassembler = umesen_core::Dissassembler::new(&emulator.cpu);
            ui.label(egui::RichText::new(dissassembler.dissassemble_next()).strong());
            for _ in 0..8 {
                ui.label(dissassembler.dissassemble_next());
            }
        });
}
