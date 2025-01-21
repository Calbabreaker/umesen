use umesen_core::Catridge;

use crate::view_window::{self, ViewWindowKind};

#[derive(serde::Deserialize, serde::Serialize, Default)]
#[serde(default)]
pub struct App {
    view_windows: Vec<ViewWindowKind>,

    #[serde(skip)]
    emulator: umesen_core::Emulator,
}

impl App {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        if let Some(storage) = cc.storage {
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }

        Default::default()
    }

    pub fn open_nes_rom(&mut self, path: &std::path::Path) -> anyhow::Result<()> {
        let file = std::fs::File::open(path)?;
        let catridge = Catridge::from_nes(file)?;
        self.emulator.attach_catridge(catridge);
        self.run_emulator();
        Ok(())
    }

    pub fn run_emulator(&mut self) {
        self.emulator.cpu.reset();
    }

    pub fn pick_nes_rom(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("NES Rom", &["nes"])
            .pick_file()
        {
            if let Err(err) = self.open_nes_rom(&path) {
                self.view_windows.push(ViewWindowKind::ErrorPopup {
                    title: "Failed to load NES rom".to_string(),
                    message: format!("{:?}", err),
                })
            }
        }
    }
}

impl eframe::App for App {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:

            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Open ROM...").clicked() {
                        self.pick_nes_rom();
                        ui.close_menu();
                    }
                });

                ui.menu_button("View", |ui| {
                    if ui.button("Cpu state...").clicked() {
                        self.view_windows.push(ViewWindowKind::CpuState);
                        ui.close_menu();
                    }
                });
            });
        });

        self.view_windows
            .retain(|window_kind| view_window::show(ctx, &mut self.emulator, window_kind));

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                egui::warn_if_debug_build(ui);
            });
        });

        ctx.input(|i| {
            if i.key_pressed(egui::Key::CloseBracket) {
                if let Err(err) = self.emulator.cpu.execute_next() {
                    log::error!("{err}");
                }
            }
        })
    }
}
