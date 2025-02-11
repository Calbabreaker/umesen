pub fn to_egui_color(color: u32) -> egui::Color32 {
    let bytes = color.to_be_bytes();
    egui::Color32::from_rgb(bytes[0], bytes[1], bytes[2])
}

pub trait UiList: egui::util::id_type_map::SerializableAny + Default + Eq {
    fn pretty_name(&self) -> &'static str;

    const LIST: &[Self];
}

pub fn show_tab_group<T: UiList>(ui: &mut egui::Ui) -> T {
    let tab_open: T = mem_get(ui);
    ui.horizontal(|ui| {
        for tab in T::LIST {
            if ui
                .selectable_label(*tab == tab_open, tab.pretty_name())
                .clicked()
            {
                mem_set(ui, tab.clone());
            }
        }
    });

    ui.separator();
    tab_open
}

pub fn show_combo_select<T: UiList>(ui: &mut egui::Ui) -> T {
    let mut selected: T = mem_get(ui);
    egui::ComboBox::from_id_salt(std::any::type_name::<T>())
        .selected_text(selected.pretty_name())
        .show_ui(ui, |ui| {
            for kind in T::LIST {
                if ui
                    .selectable_value(&mut selected, kind.clone(), kind.pretty_name())
                    .clicked()
                {
                    mem_set(ui, selected.clone());
                };
            }
        });

    selected
}

pub fn mem_get<T: egui::util::id_type_map::SerializableAny + Default>(ui: &mut egui::Ui) -> T {
    let id = egui::Id::from(std::any::type_name::<T>());
    ui.memory_mut(|w| w.data.get_persisted(id).unwrap_or_default())
}

pub fn mem_set<T: egui::util::id_type_map::SerializableAny + Default>(ui: &mut egui::Ui, val: T) {
    let id = egui::Id::from(std::any::type_name::<T>());
    ui.memory_mut(|w| *w.data.get_persisted_mut_or_default(id) = val);
}
