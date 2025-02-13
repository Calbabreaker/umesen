use umesen_core::controller::Button;

#[derive(PartialEq, Eq, serde::Serialize, serde::Deserialize, Clone, Hash)]
pub enum ActionKind {
    ControllerInput(u8, Button),
    Run(bool),
    SaveState(u8),
    LoadState(u8),
    Reset,
    Step,
}

impl ActionKind {
    pub fn name(&self) -> String {
        match self {
            Self::ControllerInput(number, button) => {
                format!("Controller {number} {}", button.name())
            }
            Self::Run(true) => "Resume".to_owned(),
            Self::Run(false) => "Pause".to_owned(),
            Self::Reset => "Reset".to_owned(),
            Self::Step => "Step Instruction".to_owned(),
            Self::SaveState(number) => format!("Save state {number}"),
            Self::LoadState(number) => format!("Load state {number}"),
        }
    }
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct KeyActionMap {
    pub map: indexmap::IndexMap<ActionKind, egui::KeyboardShortcut>,
    /// The button to bind the next pressed key to
    #[serde(skip)]
    pub action_waiting_for_press: Option<ActionKind>,
}

impl Default for KeyActionMap {
    fn default() -> Self {
        let mut map = KeyActionMap {
            map: indexmap::IndexMap::default(),
            action_waiting_for_press: None,
        };

        use egui::Key::*;
        use ActionKind::*;

        map.add(Run(true), egui::Key::F5);
        map.add(Run(false), F6);
        map.add(Reset, F4);
        map.add(Step, CloseBracket);
        map.add(ControllerInput(0, Button::RIGHT), L);
        map.add(ControllerInput(0, Button::LEFT), J);
        map.add(ControllerInput(0, Button::UP), I);
        map.add(ControllerInput(0, Button::DOWN), K);
        map.add(ControllerInput(0, Button::A), C);
        map.add(ControllerInput(0, Button::B), X);
        map.add(ControllerInput(0, Button::SELECT), S);
        map.add(ControllerInput(0, Button::START), D);
        map.add(ControllerInput(1, Button::RIGHT), ArrowRight);
        map.add(ControllerInput(1, Button::LEFT), ArrowLeft);
        map.add(ControllerInput(1, Button::UP), ArrowUp);
        map.add(ControllerInput(1, Button::DOWN), ArrowDown);
        map.add(ControllerInput(1, Button::A), Slash);
        map.add(ControllerInput(1, Button::B), Period);
        map.add(ControllerInput(1, Button::SELECT), Quote);
        map.add(ControllerInput(1, Button::START), Semicolon);
        map.add_with_mod(SaveState(1), egui::Modifiers::CTRL, Num1);
        map.add_with_mod(SaveState(2), egui::Modifiers::CTRL, Num2);
        map.add_with_mod(SaveState(3), egui::Modifiers::CTRL, Num3);
        map.add_with_mod(SaveState(4), egui::Modifiers::CTRL, Num4);
        map.add_with_mod(LoadState(1), egui::Modifiers::ALT, Num1);
        map.add_with_mod(LoadState(2), egui::Modifiers::ALT, Num2);
        map.add_with_mod(LoadState(3), egui::Modifiers::ALT, Num3);
        map.add_with_mod(LoadState(4), egui::Modifiers::ALT, Num4);

        map
    }
}

impl KeyActionMap {
    pub fn check_key_down(&mut self, input: &egui::InputState) {
        if let Some(key) = input.keys_down.iter().next() {
            if let Some(action) = self.action_waiting_for_press.take() {
                self.add_with_mod(action, input.modifiers, *key);
            }
        }
    }

    pub fn add(&mut self, action: ActionKind, key: egui::Key) {
        self.add_with_mod(action, egui::Modifiers::NONE, key);
    }

    pub fn add_with_mod(&mut self, action: ActionKind, modifiers: egui::Modifiers, key: egui::Key) {
        self.map
            .insert(action, egui::KeyboardShortcut::new(modifiers, key));
    }
}
