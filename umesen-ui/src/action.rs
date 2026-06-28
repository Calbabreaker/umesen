use std::sync::LazyLock;

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
}

type ActionMapType = indexmap::IndexMap<ActionKind, egui::KeyboardShortcut>;

#[derive(Default, serde::Deserialize, serde::Serialize)]
pub struct KeyActionMap {
    pub bindings_map: ActionMapType,
    /// The button to bind the next pressed key to
    #[serde(skip)]
    pub action_to_rebind: Option<ActionKind>,
}

pub static DEFAULT_ACTION_MAP: LazyLock<ActionMapType> = LazyLock::new(|| {
    use ActionKind::*;
    use egui::Key::*;
    let mapping = [
        (PauseResume, F5),
        (Reset, F4),
        (Step, OpenBracket),
        (QuickSave, W),
        (QuickLoad, O),
        (NextFrame, CloseBracket),
        (ControllerInput(0, Button::UP), I),
        (ControllerInput(0, Button::DOWN), K),
        (ControllerInput(0, Button::LEFT), J),
        (ControllerInput(0, Button::RIGHT), L),
        (ControllerInput(0, Button::A), C),
        (ControllerInput(0, Button::B), X),
        (ControllerInput(0, Button::START), S),
        (ControllerInput(0, Button::SELECT), D),
        (ControllerInput(1, Button::UP), ArrowUp),
        (ControllerInput(1, Button::DOWN), ArrowDown),
        (ControllerInput(1, Button::LEFT), ArrowLeft),
        (ControllerInput(1, Button::RIGHT), ArrowRight),
        (ControllerInput(1, Button::A), Slash),
        (ControllerInput(1, Button::B), Period),
        (ControllerInput(1, Button::SELECT), Quote),
        (ControllerInput(1, Button::START), Semicolon),
    ];
    ActionMapType::from(mapping.map(|(action, key)| {
        (
            action,
            egui::KeyboardShortcut::new(egui::Modifiers::NONE, key),
        )
    }))
});

impl KeyActionMap {
    pub fn check_key_down(&mut self, input: &egui::InputState) {
        if let Some(key) = input.keys_down.iter().next()
            && let Some(action) = self.action_to_rebind.take()
        {
            self.add_with_mod(action, input.modifiers, *key);
        }
    }

    pub fn add_with_mod(&mut self, action: ActionKind, modifiers: egui::Modifiers, key: egui::Key) {
        self.bindings_map
            .insert(action, egui::KeyboardShortcut::new(modifiers, key));
    }

    pub fn iter_map(&self) -> impl Iterator<Item = (ActionKind, egui::KeyboardShortcut)> {
        DEFAULT_ACTION_MAP
            .iter()
            .map(|(action, shortcut)| (*action, *self.bindings_map.get(action).unwrap_or(shortcut)))
    }
}
