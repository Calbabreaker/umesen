use crate::view_window::{ViewWindowKind, ViewWindowSet};

#[derive(serde::Deserialize, serde::Serialize, Default)]
#[serde(default)]
pub struct App {
    view_windows: ViewWindowSet,
    recent_file_paths: Vec<std::path::PathBuf>,

    #[serde(skip)]
    state: crate::State,
}

impl App {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        cc.storage
            .and_then(|storage| eframe::get_value(storage, eframe::APP_KEY))
            .unwrap_or_default()
    }

    pub fn init(&mut self, ctx: &egui::Context) {
        let screen_size = [umesen_core::ppu::WIDTH, umesen_core::ppu::HEIGHT];
        self.state.texture_map.insert(
            "ppu_output".to_string(),
            crate::Texture::new(screen_size, ctx),
        );

        if let Some(path) = self.recent_file_paths.last().cloned() {
            self.load_nes_rom(&path);
        }
    }

    fn load_nes_rom(&mut self, path: &std::path::Path) {
        log::trace!("Loading {path:?}");
        if let Err(err) = self.state.emulator.load_nes_rom(path) {
            self.view_windows.set.insert(ViewWindowKind::Popup {
                heading: "Failed to load NES ROM!".to_string(),
                message: format!("{err}"),
            });
            log::error!("{err}");
        } else {
            self.recent_file_paths.retain(|x| x != path);
            self.recent_file_paths.push(path.to_path_buf());
            self.recent_file_paths.truncate(10);

            self.state.run_emulator();
        }
    }

    fn show_top_panel(&mut self, ui: &mut egui::Ui) {
        // Doing ui.style_mut doesn't actually set the style so you have to do this for some stupid reason
        ui.ctx()
            .style_mut(|style| style.spacing.menu_width = 10000.);

        egui::menu::bar(ui, |ui| {
            ui.menu_button("File", |ui| {
                if ui.button("Open ROM...").clicked() {
                    if let Some(path) = rfd::FileDialog::new()
                        .add_filter("NES ROM", &["nes"])
                        .pick_file()
                    {
                        self.load_nes_rom(&path);
                    }
                    ui.close_menu();
                }

                ui.menu_button("Recent ROMS", |ui| {
                    let path = self.recent_file_paths.iter().rev().find(|path| {
                        //
                        ui.button(path.to_string_lossy()).clicked()
                    });
                    if let Some(path) = path.cloned() {
                        self.load_nes_rom(&path);
                        ui.close_menu();
                    }
                });
            });

            ui.menu_button("View", |ui| {
                use ViewWindowKind::*;
                for kind in [CpuState, CpuMemory, PpuMemory, Stats] {
                    let mut open = self.view_windows.set.contains(&kind);
                    let text = format!("{}...", kind.title());
                    if ui.toggle_value(&mut open, text).clicked() {
                        self.view_windows.toggle_open(kind);
                    }
                }
            });
        });
    }
}

impl eframe::App for App {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        // Remove all popups from being saved
        self.view_windows.remove_popups();
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        let default_bg = ctx.style().visuals.noninteractive().bg_fill;
        egui::TopBottomPanel::top("top_panel")
            .frame(egui::Frame::default().fill(default_bg).inner_margin(6.0))
            .show(ctx, |ui| {
                self.show_top_panel(ui);
            });

        self.view_windows.show(ctx, &mut self.state);

        if self.state.running {
            use umesen_core::ppu::FRAME_INTERVAL;

            let now = ctx.input(|i| i.time);
            let delta = now - self.state.last_frame_time;

            if delta > FRAME_INTERVAL {
                self.state.stats.frame_rate = 1. / delta;
                self.state.last_frame_time = now;
                self.state.next_frame();
            }

            ctx.request_repaint();
        }

        egui::CentralPanel::default()
            .frame(egui::Frame::none())
            .show(ctx, |ui| {
                ui.centered_and_justified(|ui| {
                    let texture = &self.state.texture_map["ppu_output"];
                    ui.add(egui::Image::new(&texture.handle).fit_to_fraction(egui::vec2(1., 1.)));
                });
            });

        ctx.input_mut(|i| {
            if i.key_pressed(egui::Key::CloseBracket) {
                self.state.step_emulator();
            }

            let file_path = i.raw.dropped_files.pop().and_then(|f| f.path);
            if let Some(path) = file_path {
                self.load_nes_rom(&path);
            }
            i.raw.dropped_files.clear();
        });

        self.state.stats.ui_render_time = frame.info().cpu_usage.unwrap_or(0.);
    }
}
