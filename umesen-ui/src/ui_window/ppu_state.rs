pub fn show(ui: &mut egui::Ui, state: &mut crate::State) {
    let ppu = &state.emu.ppu();
    ui.style_mut().override_text_style = Some(egui::TextStyle::Monospace);
    ui.label(format!(
        "T: ${:04x} ({})",
        ppu.registers.t.0, ppu.registers.t
    ));
    ui.label(format!(
        "V: ${:04x} ({})",
        ppu.registers.v.0, ppu.registers.v,
    ));
    ui.label(format!("Latch: {}", ppu.registers.latch));
    ui.label(format!("Scanline: {}", ppu.scanline));
    ui.label(format!("Dot: {}", ppu.dot));
    ui.separator();
    ui.label("Control flags");
    crate::egui_util::show_flags_marked(ui, ppu.registers.control);
    ui.separator();
    ui.label("Status flags");
    crate::egui_util::show_flags_marked(ui, ppu.registers.status);
    ui.separator();
    ui.label("Mask flags");
    crate::egui_util::show_flags_marked(ui, ppu.registers.mask);
}
