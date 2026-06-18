pub trait UiList: egui::util::id_type_map::SerializableAny + Default + Eq + Copy {
    fn pretty_name(&self) -> &'static str;

    const LIST: &[Self];
}

pub fn ui_list_tab_group<T: UiList + std::fmt::Debug>(ui: &mut egui::Ui) -> T {
    let tab_open: T = ui.memory_mut(|w| w.data.get_persisted(egui::Id::NULL).unwrap_or_default());
    ui.horizontal(|ui| {
        for tab in T::LIST {
            if ui
                .selectable_label(*tab == tab_open, tab.pretty_name())
                .clicked()
            {
                ui.memory_mut(|w| w.data.insert_persisted(egui::Id::NULL, *tab));
            }
        }
    });

    ui.separator();
    tab_open
}

pub fn ui_list_combo_select<T: UiList + std::fmt::Debug>(ui: &mut egui::Ui) -> T {
    let mut selected: T =
        ui.memory_mut(|w| w.data.get_persisted(egui::Id::NULL).unwrap_or_default());
    egui::ComboBox::from_id_salt(std::any::TypeId::of::<T>())
        .selected_text(selected.pretty_name())
        .show_ui(ui, |ui| {
            for kind in T::LIST {
                if ui
                    .selectable_value(&mut selected, *kind, kind.pretty_name())
                    .clicked()
                {
                    ui.memory_mut(|w| w.data.insert_persisted(egui::Id::NULL, selected));
                };
            }
        });

    selected
}

pub fn get_shortcut_text(shortcut: &egui::KeyboardShortcut) -> String {
    shortcut.format(&egui::ModifierNames::NAMES, cfg!(target_os = "macos"))
}

pub fn show_flags<T: bitflags::Flags + Copy>(ui: &mut egui::Ui, value: &mut T, columns: usize) {
    egui::Grid::new(std::any::TypeId::of::<T>()).show(ui, |ui| {
        for (i, flag) in T::FLAGS.iter().filter(|f| f.is_named()).enumerate() {
            ui.label(flag.name());
            let mut checked = value.contains(*flag.value());
            ui.checkbox(&mut checked, "");
            value.set(*flag.value(), checked);
            if i % columns == columns - 1 {
                ui.end_row()
            };
        }
    });
}
