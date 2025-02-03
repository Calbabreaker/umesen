use egui::ahash::HashSet;

mod cpu_memory;
mod cpu_state;
mod ppu_memory;
mod ppu_state;
mod preferences;
mod stats;

#[derive(Default, serde::Serialize, serde::Deserialize)]
pub struct UiWindowSet {
    pub set: HashSet<UiWindowKind>,
}

impl UiWindowSet {
    pub fn toggle_open(&mut self, view_window: UiWindowKind) {
        match self.set.contains(&view_window) {
            true => self.set.remove(&view_window),
            false => self.set.insert(view_window),
        };
    }

    pub fn show(&mut self, ctx: &egui::Context, state: &mut crate::State) {
        self.set.retain(|kind| show(ctx, state, kind));
    }

    pub fn remove_popups(&mut self) {
        self.set
            .retain(|kind| !matches!(kind, UiWindowKind::Popup { .. }));
    }
}

#[derive(serde::Deserialize, serde::Serialize, Clone, Debug, Hash, PartialEq, Eq)]
pub enum UiWindowKind {
    CpuState,
    CpuMemory,
    PpuMemory,
    PpuState,
    Stats,
    Preferences,
    Popup { heading: String, message: String },
}

impl UiWindowKind {
    pub fn title(&self) -> &'static str {
        match self {
            UiWindowKind::CpuState => "Debugger",
            UiWindowKind::CpuMemory => "Cpu Memory",
            UiWindowKind::Popup { .. } => "Error",
            UiWindowKind::Stats => "Stats",
            UiWindowKind::PpuMemory { .. } => "Ppu Memory",
            UiWindowKind::PpuState { .. } => "Ppu State",
            UiWindowKind::Preferences { .. } => "Preferences",
        }
    }
}

// Returns whether the window is still open
fn show(ctx: &egui::Context, state: &mut crate::State, kind: &UiWindowKind) -> bool {
    let mut open = true;

    if let UiWindowKind::Popup { heading, message } = kind {
        egui::Window::new(kind.title())
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
        let window = egui::Window::new(kind.title())
            .pivot(egui::Align2::CENTER_CENTER)
            .min_width(200.)
            .default_pos(ctx.screen_rect().size().to_pos2() / 2.)
            .open(&mut open);

        window.show(ctx, |ui| match kind {
            UiWindowKind::CpuState => cpu_state::show(ui, state),
            UiWindowKind::CpuMemory => cpu_memory::show(ui, state),
            UiWindowKind::PpuMemory => ppu_memory::show(ui, state),
            UiWindowKind::Stats => stats::show(ui, state),
            UiWindowKind::PpuState => ppu_state::show(ui, state),
            UiWindowKind::Preferences => preferences::show(ui, state),
            UiWindowKind::Popup { .. } => unreachable!(),
        });
    }

    open
}
