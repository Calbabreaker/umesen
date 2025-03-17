use crate::Preferences;

#[derive(Clone, Copy, PartialEq, Eq, Default, serde::Deserialize, serde::Serialize)]
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
    let tab_open = crate::egui_util::show_tab_group(ui);

    egui::Grid::new("key grid")
        .striped(true)
        .show(ui, |ui| match tab_open {
            Tab::Emulation => {
                ui.label("Allow left right: ").on_hover_text(
                    "Allow left and right or up and down to be pressed at the same time",
                );
                ui.checkbox(&mut prefs.allow_illegal_press, "");
            }
            Tab::KeyBinds => {
                show_key_map(ui, prefs);
            }
        });

    ui.separator();

    if ui.button("Reset").clicked() {
        *prefs = Preferences::default();
    }
}

fn show_key_map(ui: &mut egui::Ui, prefs: &mut Preferences) {
    for (action, shortcut) in &prefs.key_action_map.map {
        ui.label(action.name());

        let action_waiting_for_press = &mut prefs.key_action_map.action_waiting_for_press;
        let text = if action_waiting_for_press.as_ref() == Some(action) {
            "...".to_owned()
        } else {
            crate::egui_util::get_shortcut_text(shortcut)
        };

        if ui.button(text).clicked() {
            *action_waiting_for_press = Some(action.clone());
        }
        ui.end_row();
    }
}
