use umesen_core::controller::Button;

#[derive(PartialEq, Eq, serde::Serialize, serde::Deserialize, Clone, Hash, Debug, Copy)]
pub enum ActionKind {
    ControllerInput(u8, Button),
    PauseResume,
    QuickSave,
    QuickLoad,
    Reset,
    Step,
    NextFrame,
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

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct KeyActionMap {
    pub map: indexmap::IndexMap<ActionKind, egui::KeyboardShortcut>,
    /// The button to bind the next pressed key to
    #[serde(skip)]
    pub action_to_rebind: Option<ActionKind>,
}

impl Default for KeyActionMap {
    fn default() -> Self {
        let mut map = KeyActionMap {
            map: indexmap::IndexMap::default(),
            action_to_rebind: None,
        };

        use ActionKind::*;
        use egui::Key::*;

        map.add(PauseResume, F5);
        map.add(Reset, F4);
        map.add(Step, CloseBracket);
        map.add(ControllerInput(0, Button::RIGHT), L);
        map.add(ControllerInput(0, Button::LEFT), J);
        map.add(ControllerInput(0, Button::UP), I);
        map.add(ControllerInput(0, Button::DOWN), K);
        map.add(ControllerInput(0, Button::A), C);
        map.add(ControllerInput(0, Button::B), X);
        map.add(ControllerInput(0, Button::START), S);
        map.add(ControllerInput(0, Button::SELECT), D);
        map.add(ControllerInput(1, Button::RIGHT), ArrowRight);
        map.add(ControllerInput(1, Button::LEFT), ArrowLeft);
        map.add(ControllerInput(1, Button::UP), ArrowUp);
        map.add(ControllerInput(1, Button::DOWN), ArrowDown);
        map.add(ControllerInput(1, Button::A), Slash);
        map.add(ControllerInput(1, Button::B), Period);
        map.add(ControllerInput(1, Button::SELECT), Quote);
        map.add(ControllerInput(1, Button::START), Semicolon);
        map.add(QuickSave, W);
        map.add(QuickLoad, Q);

        map
    }
}

impl KeyActionMap {
    pub fn check_key_down(&mut self, input: &egui::InputState) {
        if let Some(key) = input.keys_down.iter().next()
            && let Some(action) = self.action_to_rebind.take()
        {
            self.add_with_mod(action, input.modifiers, *key);
        }
    }

    pub fn add(&mut self, action: ActionKind, key: egui::Key) {
        self.add_with_mod(action, egui::Modifiers::NONE, key);
    }

    pub fn add_with_mod(&mut self, action: ActionKind, modifiers: egui::Modifiers, key: egui::Key) {
        self.map
            .insert(action, egui::KeyboardShortcut::new(modifiers, key));
    }

    pub fn map_iter(
        &self,
        controller_num: Option<u8>,
    ) -> impl Iterator<Item = (&ActionKind, &egui::KeyboardShortcut)> {
        self.map.iter().filter(move |(a, _)| {
            if let Some(num) = controller_num {
                matches!(a, ActionKind::ControllerInput(n, _) if *n == num)
            } else {
                !matches!(a, ActionKind::ControllerInput(..))
            }
        })
    }
}
