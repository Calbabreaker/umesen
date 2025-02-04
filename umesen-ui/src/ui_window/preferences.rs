use crate::Preferences;

pub fn show(ui: &mut egui::Ui, saved_state: &mut Preferences) {
    egui::CollapsingHeader::new("Key binds").show_unindented(ui, |ui| {
        egui::Grid::new("key grid").striped(true).show(ui, |ui| {
            show_action_maps(ui, saved_state);
        });
    });

    if ui.button("Reset Everything").clicked() {
        *saved_state = Preferences::default();
    }
}

fn show_action_maps(ui: &mut egui::Ui, saved_state: &mut Preferences) {
    for (action, key) in &saved_state.key_action_map.map {
        ui.label(action.name());

        let action_waiting_for_press = &mut saved_state.key_action_map.action_waiting_for_press;
        let text = if action_waiting_for_press.as_ref() == Some(action) {
            "..."
        } else {
            key.name()
        };

        if ui.button(text).clicked() {
            *action_waiting_for_press = Some(action.clone());
        }
        ui.end_row();
    }
}
