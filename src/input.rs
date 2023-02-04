use std::collections::HashSet;
use winit::event::{ElementState, VirtualKeyCode};

pub struct Input {
    pressed_keys: HashSet<VirtualKeyCode>,
    released_keys: HashSet<VirtualKeyCode>,
    held_keys: HashSet<VirtualKeyCode>,
}

impl Input {
    pub fn new() -> Self {
        Self {
            pressed_keys: HashSet::new(),
            released_keys: HashSet::new(),
            held_keys: HashSet::new(),
        }
    }

    pub fn was_key_pressed(&self, keycode: VirtualKeyCode) -> bool {
        return self.pressed_keys.contains(&keycode);
    }

    pub fn was_key_released(&self, keycode: VirtualKeyCode) -> bool {
        return self.released_keys.contains(&keycode);
    }

    pub fn is_key_held(&self, keycode: VirtualKeyCode) -> bool {
        return self.held_keys.contains(&keycode);
    }

    pub fn key_state_changed(&mut self, keycode: VirtualKeyCode, state: ElementState) {
        match state {
            ElementState::Pressed => {
                if self.held_keys.insert(keycode) {
                    self.pressed_keys.insert(keycode);
                }
            }
            ElementState::Released => {
                self.released_keys.insert(keycode);
                self.held_keys.remove(&keycode);
            }
        }
    }

    pub fn update(&mut self) {
        self.pressed_keys.clear();
        self.released_keys.clear();
    }
}