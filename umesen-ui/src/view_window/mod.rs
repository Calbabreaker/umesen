use egui::ahash::HashSet;
use umesen_core::Emulator;

mod cpu_memory_view;
mod cpu_state_view;

#[derive(Default, serde::Serialize, serde::Deserialize)]
pub struct ViewWindowSet {
    pub set: HashSet<ViewWindowKind>,
}

impl ViewWindowSet {
    pub fn toggle_open(&mut self, view_window: ViewWindowKind) {
        match self.set.contains(&view_window) {
            true => self.set.remove(&view_window),
            false => self.set.insert(view_window),
        };
    }

    pub fn show(&mut self, ctx: &egui::Context, emulator: &mut Emulator) {
        self.set
            .retain(|window_kind| show(ctx, emulator, window_kind));
    }

    pub fn remove_popups(&mut self) {
        self.set
            .retain(|window_kind| !matches!(window_kind, ViewWindowKind::Popup { .. }));
    }
}

#[derive(serde::Deserialize, serde::Serialize, Clone, Debug, Hash, PartialEq, Eq)]
pub enum ViewWindowKind {
    CpuState,
    CpuMemory,
    Popup { heading: String, message: String },
}

// Returns whether the window is still open
fn show(ctx: &egui::Context, emulator: &mut Emulator, kind: &ViewWindowKind) -> bool {
    let mut open = true;

    let title = match kind {
        ViewWindowKind::CpuState => "Cpu state",
        ViewWindowKind::CpuMemory => "Cpu memory",
        ViewWindowKind::Popup { .. } => "Error",
    };

    if let ViewWindowKind::Popup { heading, message } = kind {
        egui::Window::new(title)
            .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
            .movable(false)
            .open(&mut open)
            .collapsible(false)
            .order(egui::Order::TOP)
            .resizable(false)
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.heading(heading);
                    ui.spacing();
                    ui.label(egui::RichText::new(message).color(egui::Color32::LIGHT_RED));
                })
            });
    } else {
        let window = egui::Window::new(title)
            .pivot(egui::Align2::CENTER_CENTER)
            .min_width(200.)
            .default_pos(ctx.screen_rect().size().to_pos2() / 2.)
            .open(&mut open);

        window.show(ctx, |ui| match kind {
            ViewWindowKind::CpuState => cpu_state_view::show(ui, emulator),
            ViewWindowKind::CpuMemory => cpu_memory_view::show(ui, emulator),
            ViewWindowKind::Popup { .. } => unreachable!(),
        });
    }

    open
}