pub fn show(ui: &mut egui::Ui, state: &mut crate::State) {
    let ppu = &state.emu.ppu();
    ui.style_mut().override_text_style = Some(egui::TextStyle::Monospace);
    ui.label(format!("T: ${0:04x}", ppu.registers.t.0));
    ui.label(format!("V: ${0:04x}", ppu.registers.v.0));
    ui.label(format!("Latch: {}", ppu.registers.latch));
    ui.label(format!("Scanline: {}", ppu.scanline));
    ui.label(format!("Dot: {}", ppu.dot));
    ui.label(format!("{:?}", ppu.registers.control));
    ui.label(format!("{:?}", ppu.registers.status));
    ui.label(format!("{:?}", ppu.registers.mask));
}
