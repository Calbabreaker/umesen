use std::collections::HashSet;

use crate::{ActionKind, Preferences, audio::setup_audio_stream, ui_window::UiWindowKind};

#[derive(serde::Deserialize, serde::Serialize, Default)]
#[serde(default)]
pub struct App {
    ui_windows: HashSet<UiWindowKind>,
    preferences: Preferences,
    recent_file_paths: Vec<std::path::PathBuf>,

    #[serde(skip)]
    state: crate::State,
    #[serde(skip)]
    audio_stream: Option<cpal::Stream>,
}

impl App {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let mut app: App = cc
            .storage
            .and_then(|storage| eframe::get_value(storage, eframe::APP_KEY))
            .unwrap_or_default();

        if let Some(path) = app.recent_file_paths.first().cloned() {
            app.load_nes_rom(&path);
        }

        match setup_audio_stream(&mut app.state.emu) {
            Ok(stream) => app.audio_stream = Some(stream),
            Err(err) => {
                app.ui_windows.insert(UiWindowKind::Popup {
                    heading: "Failed to initialize audio".to_string(),
                    message: format!("{err}"),
                });
            }
        }

        app
    }

    fn load_nes_rom(&mut self, path: &std::path::Path) {
        log::trace!("Loading {path:?}");
        if let Err(err) = self.state.emu.load_nes_rom(path) {
            self.ui_windows.insert(UiWindowKind::Popup {
                heading: "Failed to load NES ROM!".to_string(),
                message: format!("{err}"),
            });
            log::error!("{err}");
        } else {
            log::trace!(
                "Loaded cartridge with header: {:?}",
                self.state.emu.cartridge().unwrap().header()
            );
            // Make sure added path is on top
            self.recent_file_paths.retain(|x| x != path);
            self.recent_file_paths.insert(0, path.to_path_buf());
            self.recent_file_paths.truncate(20);
            self.state.emu.running = true;
        }
    }

    fn show_top_bar(&mut self, ui: &mut egui::Ui) {
        ui.menu_button("File", |ui| {
            if ui.button("Open ROM...").clicked() {
                if let Some(path) = rfd::FileDialog::new()
                    .add_filter("NES ROM", &["nes"])
                    .pick_file()
                {
                    self.load_nes_rom(&path);
                }
                ui.close();
            }

            ui.menu_button("Recent ROMS", |ui| {
                let mut paths = self.recent_file_paths.iter();
                let path = paths.find(|path| ui.button(path.to_string_lossy()).clicked());

                if let Some(path) = path.cloned() {
                    self.load_nes_rom(&path);
                    ui.close();
                }
            });

            if ui.button("Preferences...").clicked() {
                self.ui_windows.insert(UiWindowKind::Preferences);
                ui.close();
            }
        });

        ui.menu_button("View", |ui| {
            use UiWindowKind::*;
            for kind in [
                Debugger,
                HexViewer,
                PpuMemory,
                PpuState,
                Stats,
                CatridgeInfo,
            ] {
                let mut open = self.ui_windows.contains(&kind);
                let text = format!("{}...", kind.title());
                if ui.toggle_value(&mut open, text).clicked() {
                    if open {
                        self.ui_windows.insert(kind);
                    } else {
                        self.ui_windows.remove(&kind);
                    }
                }
            }
        });

        ui.menu_button("Emulation", |ui| {
            use ActionKind::*;
            self.show_action_list(ui, [PauseResume, Reset, QuickSave, QuickLoad].into_iter());

            ui.menu_button("Quick Save Slot", |ui| {
                for i in 0..9 {
                    let mut text = egui::RichText::new(format!("Slot {i}"));
                    if i == self.state.selected_quick_save {
                        text = text.underline();
                    }
                    if ui.button(text).clicked() {
                        self.state.selected_quick_save = i;
                    }
                }
            });
        });
    }

    fn show_action_list(&mut self, ui: &mut egui::Ui, iter: impl Iterator<Item = ActionKind>) {
        for action in iter {
            let shortcut = self.preferences.key_action_map.map[&action];
            let button = egui::Button::new(action.name())
                .shortcut_text(crate::egui_util::get_shortcut_text(&shortcut));
            if ui.add(button).clicked() {
                self.state.do_action(action);
                ui.close();
            }
        }
    }

    fn check_input(&mut self, i: &mut egui::InputState) {
        // Do controller input seperate
        for (action, shortcut) in &self.preferences.key_action_map.map {
            if let ActionKind::ControllerInput(number, button) = action {
                let controller = self.state.emu.controller(*number);
                let key_down = i.key_down(shortcut.logical_key);
                let is_illegal = key_down && controller.check_illegal_press(*button);

                if !is_illegal || self.preferences.allow_illegal_press {
                    controller.state.set(*button, key_down);
                }
            } else {
                if i.consume_shortcut(shortcut) {
                    self.state.do_action(*action);
                }
            }
        }

        self.preferences.key_action_map.check_key_down(i);

        if let Some(path) = i.raw.dropped_files.pop().and_then(|f| f.path) {
            self.load_nes_rom(&path);
        }
        i.raw.dropped_files.clear();
    }
}

impl eframe::App for App {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        // Remove all popups from being saved
        self.ui_windows
            .retain(|kind| !matches!(kind, UiWindowKind::Popup { .. }));
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    fn logic(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
        ctx.input_mut(|i| self.check_input(i));

        self.state.emu.cpu.bus.ppu.unlimited_sprites = self.preferences.allow_unlimted_sprites;

        self.state.update_emulation(ctx);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        let default_bg = ui.style().visuals.noninteractive().bg_fill;
        egui::Panel::top("top_panel")
            .frame(egui::Frame::default().fill(default_bg).inner_margin(6.0))
            .show(ui, |ui| {
                egui::MenuBar::new().ui(ui, |ui| self.show_top_bar(ui))
            });

        self.ui_windows
            .retain(|kind| kind.show(ui, &mut self.state, &mut self.preferences));

        egui::CentralPanel::default()
            .frame(egui::Frame::NONE)
            .show(ui, |ui| {
                ui.centered_and_justified(|ui| {
                    if let Some(texture) = self.state.texture_map.0.get_mut("ppu_output") {
                        ui.add(texture.image(ui).fit_to_fraction(egui::vec2(1., 1.)));
                    }
                });
            });

        self.state.ui_render_time = frame.info().cpu_usage.unwrap_or(0.);
    }
}
