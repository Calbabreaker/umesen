use serde::{Deserialize, Serialize};
use umesen_core::controller::Button;

#[derive(Deserialize, Serialize)]
pub struct ButtonMap {
    pub list: [(Button, egui::Key); 8],
    /// The button to bind the next pressed key to
    pub button_waiting_for_press: Option<Button>,
}

impl Default for ButtonMap {
    fn default() -> Self {
        ButtonMap {
            list: [
                (Button::RIGHT, egui::Key::ArrowRight),
                (Button::LEFT, egui::Key::ArrowLeft),
                (Button::UP, egui::Key::ArrowUp),
                (Button::DOWN, egui::Key::ArrowDown),
                (Button::A, egui::Key::C),
                (Button::B, egui::Key::X),
                (Button::SELECT, egui::Key::D),
                (Button::START, egui::Key::S),
            ],
            button_waiting_for_press: None,
        }
    }
}
