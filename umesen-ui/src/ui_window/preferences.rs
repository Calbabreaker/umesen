use bitflags::Flags;
use umesen_core::controller::Button;

use crate::{ActionKind, Preferences};

#[derive(Clone, Copy, PartialEq, Eq, Default, serde::Deserialize, serde::Serialize, Debug)]
enum Tab {
    #[default]
    KeyBinds,
    Emulation,
}

impl crate::egui_util::UiList for Tab {
    fn pretty_name(&self) -> &'static str {
        match self {
            Self::KeyBinds => "KeyBinds",
            Self::Emulation => "Emulation",
        }
    }

    const LIST: &[Self] = &[Self::Emulation, Self::KeyBinds];
}

pub fn show(ui: &mut egui::Ui, prefs: &mut Preferences) {
    let tab_open = crate::egui_util::ui_list_tab_group(ui);

    match tab_open {
        Tab::Emulation => {
            egui::Grid::new("pref list").striped(true).show(ui, |ui| {
                ui.label("Allow illegal press ").on_hover_text(
                    "Allow left and right or up and down to be pressed at the same time",
                );
                ui.checkbox(&mut prefs.allow_illegal_press, "");
                ui.end_row();
                ui.label("Allow unlimited sprites").on_hover_text(
                    "Allow unlimited sprites to be rendered on the same scanline at a time instead of the usual 8",
                );
                ui.checkbox(&mut prefs.allow_unlimted_sprites, "");
                ui.end_row();
                ui.label("Audio volume");
                ui.add(egui::Slider::new(&mut prefs.volume, (0.)..=1.));
                ui.end_row();
            });
        }
        Tab::KeyBinds => {
            ui.horizontal_top(|ui| {
                use ActionKind::*;
                show_key_map(
                    ui,
                    prefs,
                    "mainkeys",
                    [PauseResume, Reset, Step, NextFrame, QuickSave, QuickLoad].into_iter(),
                );
                for i in 0..=1 {
                    show_key_map(
                        ui,
                        prefs,
                        "controllerkeys",
                        Button::FLAGS
                            .iter()
                            .map(|flag| ControllerInput(i, *flag.value())),
                    );
                }
            });
        }
    }

    ui.separator();

    if ui.button("Reset").clicked() {
        *prefs = Preferences::default();
    }
}

fn show_key_map(
    ui: &mut egui::Ui,
    prefs: &mut Preferences,
    name: impl egui::AsIdSalt,
    action_iter: impl Iterator<Item = ActionKind>,
) {
    egui::Grid::new(name).striped(true).show(ui, |ui| {
        for action in action_iter {
            let map = &prefs.key_action_map.map;
            let shortcut = map
                .get(&action)
                .copied()
                .unwrap_or(action.default_shortcut());
            ui.label(action.name());

            let text = if prefs.key_action_map.action_to_rebind == Some(action) {
                "...".to_owned()
            } else {
                crate::egui_util::get_shortcut_text(&shortcut)
            };

            if ui.button(text).clicked() {
                prefs.key_action_map.action_to_rebind = Some(action);
            }
            ui.end_row();
        }
    });
}
