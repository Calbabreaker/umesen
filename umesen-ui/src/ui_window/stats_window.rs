pub fn show(ui: &mut egui::Ui, state: &mut crate::State) {
    ui.label(format!(
        "UI render time: {:.3}ms",
        state.stats.ui_render_time * 1000.
    ));
    ui.label(format!(
        "Emulation render time: {:.3}ms",
        state.stats.emulation_render_time * 1000.
    ));
    ui.label(format!(
        "Emulation frame rate: {:.3}fps",
        state.emu.frame_rate()
    ));
}
