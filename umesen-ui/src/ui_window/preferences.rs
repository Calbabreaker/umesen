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
            Self::KeyBinds => "Key binds",
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
                ui.label("Allow illegal press ").on_hover_text("Allow left and right or up and down to be pressed at the same time");
                ui.checkbox(&mut prefs.allow_illegal_press, "");
                ui.end_row();
                ui.label("Allow unlimited sprites").on_hover_text("Allow unlimited sprites to be rendered on the same scanline at a time instead of the usual 8");
                ui.checkbox(&mut prefs.ppu.unlimited_sprites, "");
                ui.end_row();
                ui.label("Audio volume");
                ui.add(egui::Slider::new(&mut prefs.apu.volume, (0.)..=1.));
                ui.end_row();
                ui.label("Extra audio filters").on_hover_text("Add extra low and high pass filters to make it sound more like on the NES, sounds kinda bad though");
                ui.checkbox(&mut prefs.apu.extra_filters, "");
                ui.end_row();
            });
        }
        Tab::KeyBinds => {
            ui.horizontal_top(|ui| {
                use ActionKind::*;
                show_key_map(ui, prefs, "mainkeys", |action| {
                    !matches!(action, ControllerInput(..))
                });
                for i in 0..=1 {
                    show_key_map(
                        ui,
                        prefs,
                        format!("controllerkeys{i}"),
                        |action| matches!(action, ControllerInput(num, _) if num == i),
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
    filter_func: impl Fn(ActionKind) -> bool,
) {
    egui::Grid::new(name).striped(true).show(ui, |ui| {
        let mut action_to_rebind = prefs.key_action_map.action_to_rebind;
        for (action, shortcut) in prefs.key_action_map.iter_map() {
            if filter_func(action) {
                ui.label(action.name());

                let text = if action_to_rebind == Some(action) {
                    "...".to_owned()
                } else {
                    crate::egui_util::get_shortcut_text(&shortcut)
                };
                if ui.button(text).clicked() {
                    action_to_rebind = Some(action);
                }
                ui.end_row();
            }
        }
        prefs.key_action_map.action_to_rebind = action_to_rebind;
    });
}
