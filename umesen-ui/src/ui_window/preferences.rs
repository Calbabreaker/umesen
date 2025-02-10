use crate::Preferences;

pub fn show(ui: &mut egui::Ui, prefs: &mut Preferences) {
    egui::CollapsingHeader::new("Key binds").show_unindented(ui, |ui| {
        egui::Grid::new("key grid").striped(true).show(ui, |ui| {
            show_action_maps(ui, prefs);
        });
    });

    egui::CollapsingHeader::new("Emulation").show_unindented(ui, |ui| {
        egui::Grid::new("key grid").striped(true).show(ui, |ui| {
            ui.label("Allow left right: ");
            ui.checkbox(&mut prefs.allow_left_right, "");
        });
    });

    ui.separator();

    if ui.button("Reset").clicked() {
        *prefs = Preferences::default();
    }
}

fn show_action_maps(ui: &mut egui::Ui, prefs: &mut Preferences) {
    for (action, key) in &prefs.key_action_map.map {
        ui.label(action.name());

        let action_waiting_for_press = &mut prefs.key_action_map.action_waiting_for_press;
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
