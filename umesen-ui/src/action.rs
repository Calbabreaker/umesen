use umesen_core::controller::Button;

#[derive(PartialEq, Eq, serde::Serialize, serde::Deserialize, Clone, Hash, Debug, Copy)]
pub enum ActionKind {
    ControllerInput(u8, Button),
    PauseResume,
    Reset,
    Step,
    NextFrame,
    QuickSave,
    QuickLoad,
}

impl ActionKind {
    pub fn name(&self) -> String {
        match self {
            Self::ControllerInput(number, button) => {
                format!("Controller {number} {}", button.name())
            }
            Self::NextFrame => "Step next frame".to_owned(),
            Self::PauseResume => "Pause/Resume".to_owned(),
            Self::Reset => "Reset".to_owned(),
            Self::Step => "Step Instruction".to_owned(),
            Self::QuickSave => "Quick Save".to_owned(),
            Self::QuickLoad => "Quick Load".to_owned(),
        }
    }

    pub fn default_shortcut(&self) -> egui::KeyboardShortcut {
        use egui::Key::*;
        let key = match self {
            Self::PauseResume => F5,
            Self::Reset => F4,
            Self::Step => OpenBracket,
            Self::QuickSave => W,
            Self::QuickLoad => O,
            Self::NextFrame => CloseBracket,
            Self::ControllerInput(0, Button::RIGHT) => L,
            Self::ControllerInput(0, Button::LEFT) => J,
            Self::ControllerInput(0, Button::UP) => I,
            Self::ControllerInput(0, Button::DOWN) => K,
            Self::ControllerInput(0, Button::A) => C,
            Self::ControllerInput(0, Button::B) => X,
            Self::ControllerInput(0, Button::START) => S,
            Self::ControllerInput(0, Button::SELECT) => D,
            Self::ControllerInput(1, Button::RIGHT) => ArrowRight,
            Self::ControllerInput(1, Button::LEFT) => ArrowLeft,
            Self::ControllerInput(1, Button::UP) => ArrowUp,
            Self::ControllerInput(1, Button::DOWN) => ArrowDown,
            Self::ControllerInput(1, Button::A) => Slash,
            Self::ControllerInput(1, Button::B) => Period,
            Self::ControllerInput(1, Button::SELECT) => Quote,
            Self::ControllerInput(1, Button::START) => Semicolon,
            _ => unreachable!("got action {:?}", self),
        };
        egui::KeyboardShortcut::new(egui::Modifiers::NONE, key)
    }
}

#[derive(Default, serde::Deserialize, serde::Serialize)]
pub struct KeyActionMap {
    pub map: std::collections::HashMap<ActionKind, egui::KeyboardShortcut>,
    /// The button to bind the next pressed key to
    #[serde(skip)]
    pub action_to_rebind: Option<ActionKind>,
}

impl KeyActionMap {
    pub fn check_key_down(&mut self, input: &egui::InputState) {
        if let Some(key) = input.keys_down.iter().next()
            && let Some(action) = self.action_to_rebind.take()
        {
            self.add_with_mod(action, input.modifiers, *key);
        }
    }

    pub fn add_with_mod(&mut self, action: ActionKind, modifiers: egui::Modifiers, key: egui::Key) {
        self.map
            .insert(action, egui::KeyboardShortcut::new(modifiers, key));
    }
}
