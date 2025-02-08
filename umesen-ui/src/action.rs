use umesen_core::controller::Button;

#[derive(PartialEq, Eq, serde::Serialize, serde::Deserialize, Clone, Hash)]
pub enum ActionKind {
    ControllerInput(u8, Button),
    Run(bool),
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
            ActionKind::Step => "Step Instruction".to_owned(),
        }
    }
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct KeyActionMap {
    pub map: indexmap::IndexMap<ActionKind, egui::Key>,
    /// The button to bind the next pressed key to
    #[serde(skip)]
    pub action_waiting_for_press: Option<ActionKind>,
}

impl Default for KeyActionMap {
    fn default() -> Self {
        use ActionKind::*;
        KeyActionMap {
            map: indexmap::indexmap! {
                Run(true) => egui::Key::F5,
                Run(false) => egui::Key::F6,
                Reset => egui::Key::F4,
                Step => egui::Key::CloseBracket,
                ControllerInput(0, Button::RIGHT) => egui::Key::L,
                ControllerInput(0, Button::LEFT) => egui::Key::J,
                ControllerInput(0, Button::UP) => egui::Key::I,
                ControllerInput(0, Button::DOWN) => egui::Key::K,
                ControllerInput(0, Button::A) => egui::Key::C,
                ControllerInput(0, Button::B) => egui::Key::X,
                ControllerInput(0, Button::SELECT) => egui::Key::S,
                ControllerInput(0, Button::START) => egui::Key::D,
                ControllerInput(1, Button::RIGHT) => egui::Key::ArrowRight,
                ControllerInput(1, Button::LEFT) => egui::Key::ArrowLeft,
                ControllerInput(1, Button::UP) => egui::Key::ArrowUp,
                ControllerInput(1, Button::DOWN) => egui::Key::ArrowDown,
                ControllerInput(1, Button::A) => egui::Key::Slash,
                ControllerInput(1, Button::B) => egui::Key::Period,
                ControllerInput(1, Button::SELECT) => egui::Key::Quote,
                ControllerInput(1, Button::START) => egui::Key::Semicolon,
            },
            action_waiting_for_press: None,
        }
    }
}

impl KeyActionMap {
    pub fn check_key_down(&mut self, input: &egui::InputState) {
        if let Some(key) = input.keys_down.iter().next() {
            if let Some(action) = self.action_waiting_for_press.take() {
                self.map.insert(action, *key);
            }
        }
    }
}
