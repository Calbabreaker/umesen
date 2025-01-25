pub fn show(ui: &mut egui::Ui, state: &mut crate::State) {
    let registers = &state.emulator.ppu().registers;
    ui.style_mut().override_text_style = Some(egui::TextStyle::Monospace);
    ui.label(format!("T: ${0:04x}", registers.t_register.0));
    ui.label(format!("V: ${0:04x}", registers.v_register.0));
    ui.label(format!("Latch: {}", registers.latch));
    ui.label(format!("{:?}", registers.control));
    ui.label(format!("{:?}", registers.status));
}
