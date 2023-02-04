use std::collections::HashSet;
use winit::{
    event::{ElementState, MouseButton, VirtualKeyCode},
    window::{CursorGrabMode, Window},
};

pub struct Input {
    pressed_keys: HashSet<VirtualKeyCode>,
    released_keys: HashSet<VirtualKeyCode>,
    held_keys: HashSet<VirtualKeyCode>,
    pressed_mouse_buttons: HashSet<MouseButton>,
    mouse_delta_x: f32,
    mouse_delta_y: f32,
    is_focused: bool,
}

impl Input {
    pub fn new() -> Self {
        Self {
            pressed_keys: HashSet::new(),
            released_keys: HashSet::new(),
            held_keys: HashSet::new(),
            pressed_mouse_buttons: HashSet::new(),
            mouse_delta_x: 0.0,
            mouse_delta_y: 0.0,
            is_focused: false,
        }
    }

    pub fn was_key_pressed(&self, keycode: VirtualKeyCode) -> bool {
        if self.is_focused {
            self.pressed_keys.contains(&keycode)
        } else {
            false
        }
    }

    pub fn was_key_released(&self, keycode: VirtualKeyCode) -> bool {
        if self.is_focused {
            self.released_keys.contains(&keycode)
        } else {
            false
        }
    }

    pub fn is_key_held(&self, keycode: VirtualKeyCode) -> bool {
        if self.is_focused {
            self.held_keys.contains(&keycode)
        } else {
            false
        }
    }

    pub fn was_mouse_button_pressed(&self, button: MouseButton, ignore_focus: bool) -> bool {
        if ignore_focus || self.is_focused {
            self.pressed_mouse_buttons.contains(&button)
        } else {
            false
        }
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

    pub fn mouse_state_changed(&mut self, button: MouseButton) {
        self.pressed_mouse_buttons.insert(button);
    }

    pub fn mouse_moved(&mut self, delta_x: f32, delta_y: f32) {
        self.mouse_delta_x += delta_x;
        self.mouse_delta_y += delta_y;
    }

    pub fn mouse_delta_x(&mut self) -> f32 {
        if self.is_focused {
            self.mouse_delta_x
        } else {
            0.0
        }
    }

    pub fn mouse_delta_y(&mut self) -> f32 {
        if self.is_focused {
            self.mouse_delta_y
        } else {
            0.0
        }
    }

    pub fn set_focused(&mut self, window: &Window, is_focused: bool) {
        self.is_focused = is_focused;
        Self::set_locked_cursor(window, is_focused);
    }

    fn set_locked_cursor(window: &Window, is_locked: bool) -> bool {
        if is_locked {
            if let Err(_) = window.set_cursor_grab(CursorGrabMode::Locked) {
                _ = window.set_cursor_grab(CursorGrabMode::Confined);
            }
        } else {
            _ = window.set_cursor_grab(CursorGrabMode::None);
        }

        _ = window.set_cursor_visible(!is_locked);

        is_locked
    }

    pub fn update(&mut self) {
        self.pressed_keys.clear();
        self.released_keys.clear();
        self.pressed_mouse_buttons.clear();
        self.mouse_delta_x = 0.0;
        self.mouse_delta_y = 0.0;
    }
}
