use egui::ahash::HashSet;
use umesen_core::Emulator;

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
    Popup { heading: String, message: String },
}

// Returns whether the window is still open
fn show(ctx: &egui::Context, emulator: &mut Emulator, kind: &ViewWindowKind) -> bool {
    let mut open = true;

    let title = match kind {
        ViewWindowKind::CpuState => "Cpu state",
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
            ViewWindowKind::CpuState => cpu_state_view(ui, emulator),
            ViewWindowKind::Popup { .. } => unreachable!(),
        });
    }

    open
}

fn cpu_state_view(ui: &mut egui::Ui, emulator: &mut Emulator) {
    ui.style_mut().override_text_style = Some(egui::TextStyle::Monospace);
    ui.label(format!("PC: ${0:02x}", emulator.cpu.pc));
    ui.label(format!("A:  ${0:02x}", emulator.cpu.a));
    ui.label(format!("X:  ${0:02x}", emulator.cpu.x));
    ui.label(format!("Y:  ${0:02x}", emulator.cpu.y));
    ui.label(format!("SP: ${0:02x}", emulator.cpu.sp));
    ui.label(format!("FLAGS: {}", emulator.cpu.flags));
    ui.separator();

    let frame = egui::Frame::canvas(ui.style())
        .inner_margin(6.0)
        .outer_margin(6.0);
    egui::CollapsingHeader::new("Disassemble")
        .default_open(true)
        .show_unindented(ui, |ui| {
            frame.show(ui, |ui| {
                let mut dissassembler = umesen_core::Disassembler::new(&emulator.cpu);
                ui.label(egui::RichText::new(dissassembler.disassemble_next()).strong());
                for _ in 0..8 {
                    ui.label(dissassembler.disassemble_next());
                }
                ui.allocate_space(egui::Vec2::new(ui.available_width(), 0.));
            });
        });

    if ui.button("Step").clicked() {
        emulator.step();
    }

    ui.input(|i| {
        if i.key_pressed(egui::Key::CloseBracket) {
            emulator.step();
        }
    })
}
