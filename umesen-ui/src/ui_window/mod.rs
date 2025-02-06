mod debugger;
pub mod hex_viewer;
pub mod ppu_memory;
mod ppu_state;
mod preferences;
mod stats;

#[derive(serde::Deserialize, serde::Serialize, Clone, Debug, Hash, PartialEq, Eq)]
pub enum UiWindowKind {
    Debugger,
    HexViewer,
    PpuMemory,
    PpuState,
    Stats,
    Preferences,
    Popup { heading: String, message: String },
}

impl UiWindowKind {
    pub fn title(&self) -> &'static str {
        match self {
            UiWindowKind::Debugger => "Debugger",
            UiWindowKind::HexViewer => "Hex Viewer",
            UiWindowKind::Popup { .. } => "Error",
            UiWindowKind::Stats => "Stats",
            UiWindowKind::PpuMemory => "Ppu Memory",
            UiWindowKind::PpuState => "Ppu State",
            UiWindowKind::Preferences => "Preferences",
        }
    }

    // Returns whether the window is still open
    pub fn show(
        &self,
        ctx: &egui::Context,
        state: &mut crate::State,
        preferences: &mut crate::Preferences,
    ) -> bool {
        if let UiWindowKind::Popup { heading, message } = self {
            let modal = egui::Modal::new(self.title().into()).show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.heading(heading);
                    ui.add_space(10.);
                    ui.label(egui::RichText::new(message).color(egui::Color32::LIGHT_RED));
                })
            });

            return !modal.should_close();
        }

        ctx.style_mut(|style| style.spacing.window_margin = egui::Margin::same(10.));
        let mut open = true;
        egui::Window::new(self.title())
            .pivot(egui::Align2::CENTER_CENTER)
            .min_width(200.)
            .default_pos(ctx.screen_rect().size().to_pos2() / 2.)
            .open(&mut open)
            .show(ctx, |ui| match self {
                UiWindowKind::Debugger => debugger::show(ui, state),
                UiWindowKind::HexViewer => hex_viewer::show(ui, state),
                UiWindowKind::PpuMemory => ppu_memory::show(ui, state),
                UiWindowKind::Stats => stats::show(ui, state),
                UiWindowKind::PpuState => ppu_state::show(ui, state),
                UiWindowKind::Preferences => preferences::show(ui, preferences),
                UiWindowKind::Popup { .. } => unreachable!(),
            });

        open
    }
}
