pub fn show(ui: &mut egui::Ui, state: &mut crate::State) {
    egui::CollapsingHeader::new("Controller 1 input").show_unindented(ui, |ui| {
        show_controller_input(ui, state, 0);
    });

    egui::CollapsingHeader::new("Controller 2 input").show_unindented(ui, |ui| {
        show_controller_input(ui, state, 1);
    });
}

fn show_controller_input(ui: &mut egui::Ui, state: &mut crate::State, number: usize) {
    let button_map = &mut state.button_maps[number];
    egui::Grid::new(number).show(ui, |ui| {
        for (button, key) in &button_map.list {
            ui.label(format!("Button {}:", button.name()));

            let text = if button_map.button_waiting_for_press == Some(*button) {
                "Press any key"
            } else {
                key.name()
            };

            if ui.button(text).clicked() {
                button_map.button_waiting_for_press = Some(*button);
            }
            ui.end_row();
        }
    });
}
