use crate::Preferences;

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
                ui.label("Allow illegal press: ").on_hover_text(
                    "Allow left and right or up and down to be pressed at the same time",
                );
                ui.checkbox(&mut prefs.allow_illegal_press, "");
                ui.end_row();
            });
        }
        Tab::KeyBinds => {
            ui.horizontal_top(|ui| {
                show_key_map(ui, prefs, None);
                show_key_map(ui, prefs, Some(0));
                show_key_map(ui, prefs, Some(1));
            });
        }
    }

    ui.separator();

    if ui.button("Reset").clicked() {
        *prefs = Preferences::default();
    }
}

fn show_key_map(ui: &mut egui::Ui, prefs: &mut Preferences, controller_num: Option<u8>) {
    egui::Grid::new(format!("Key map {controller_num:?}"))
        .striped(true)
        .show(ui, |ui| {
            let mut action_to_rebind = prefs.key_action_map.action_to_rebind;
            for (action, shortcut) in prefs.key_action_map.map_iter(controller_num) {
                ui.label(action.name());

                let text = if action_to_rebind.as_ref() == Some(action) {
                    "...".to_owned()
                } else {
                    crate::egui_util::get_shortcut_text(shortcut)
                };

                if ui.button(text).clicked() {
                    action_to_rebind = Some(*action);
                }
                ui.end_row();
            }
            prefs.key_action_map.action_to_rebind = action_to_rebind;
        });
}
