use cgmath::Zero;
use std::{collections::HashSet, hash::Hash};
use winit::{
    event::{ElementState, KeyboardInput, MouseButton, VirtualKeyCode, WindowEvent},
    window::{CursorGrabMode, Window},
};

// TODO: Input states game/menu that are specified when pulling buttons and changed in systems instead of the window.
struct ButtonSet<T: Copy + Hash + Eq> {
    pressed_buttons: HashSet<T>,
    released_buttons: HashSet<T>,
    held_buttons: HashSet<T>,
}

impl<T: Copy + Hash + Eq> ButtonSet<T> {
    pub fn new() -> Self {
        Self {
            pressed_buttons: HashSet::new(),
            released_buttons: HashSet::new(),
            held_buttons: HashSet::new(),
        }
    }

    pub fn was_button_pressed(&self, button: T, is_focused: bool) -> bool {
        is_focused && self.pressed_buttons.contains(&button)
    }

    pub fn was_button_released(&self, button: T, is_focused: bool) -> bool {
        is_focused && self.released_buttons.contains(&button)
    }

    pub fn is_button_held(&self, button: T, is_focused: bool) -> bool {
        is_focused && self.held_buttons.contains(&button)
    }

    pub fn button_state_changed(&mut self, button: T, state: ElementState) {
        match state {
            ElementState::Pressed => {
                if self.held_buttons.insert(button) {
                    self.pressed_buttons.insert(button);
                }
            }
            ElementState::Released => {
                self.released_buttons.insert(button);
                self.held_buttons.remove(&button);
            }
        }
    }

    pub fn update(&mut self) {
        self.pressed_buttons.clear();
        self.released_buttons.clear();
    }
}

pub struct Input {
    keys: ButtonSet<VirtualKeyCode>,
    mouse_buttons: ButtonSet<MouseButton>,
    mouse_delta_x: f32,
    mouse_delta_y: f32,
    mouse_position: cgmath::Vector2<f32>,
    gui_mouse_position: cgmath::Vector2<f32>,
    is_focused: bool,
}

impl Input {
    pub fn new() -> Self {
        Self {
            keys: ButtonSet::new(),
            mouse_buttons: ButtonSet::new(),
            mouse_delta_x: 0.0,
            mouse_delta_y: 0.0,
            mouse_position: cgmath::Vector2::zero(),
            gui_mouse_position: cgmath::Vector2::zero(),
            is_focused: false,
        }
    }

    pub fn process_button(&mut self, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::KeyboardInput {
                input:
                    KeyboardInput {
                        state,
                        virtual_keycode: Some(keycode),
                        ..
                    },
                ..
            } => {
                self.key_state_changed(*keycode, *state);
            }
            WindowEvent::MouseInput { state, button, .. } => {
                self.mouse_button_state_changed(*button, *state);
            }
            WindowEvent::CursorMoved { position, .. } => {
                self.mouse_position.x = position.x as f32;
                self.mouse_position.y = position.y as f32;
            }
            _ => return false,
        }

        true
    }

    pub fn process_mouse_motion(&mut self, delta_x: f32, delta_y: f32) {
        self.mouse_moved(delta_x, delta_y);
    }

    pub fn was_key_pressed(&self, keycode: VirtualKeyCode) -> bool {
        self.keys.was_button_pressed(keycode, self.is_focused)
    }

    pub fn was_key_released(&self, keycode: VirtualKeyCode) -> bool {
        self.keys.was_button_released(keycode, self.is_focused)
    }

    pub fn is_key_held(&self, keycode: VirtualKeyCode) -> bool {
        self.keys.is_button_held(keycode, self.is_focused)
    }

    pub fn was_mouse_button_pressed(&self, button: MouseButton) -> bool {
        self.mouse_buttons
            .was_button_pressed(button, self.is_focused)
    }

    pub fn was_mouse_button_pressed_ignore_focus(&self, button: MouseButton) -> bool {
        self.mouse_buttons.was_button_pressed(button, true)
    }

    pub fn was_mouse_button_released(&self, button: MouseButton) -> bool {
        self.mouse_buttons
            .was_button_released(button, self.is_focused)
    }

    pub fn is_mouse_button_held(&self, button: MouseButton) -> bool {
        self.mouse_buttons.is_button_held(button, self.is_focused)
    }

    pub fn key_state_changed(&mut self, keycode: VirtualKeyCode, state: ElementState) {
        self.keys.button_state_changed(keycode, state);
    }

    pub fn mouse_button_state_changed(&mut self, button: MouseButton, state: ElementState) {
        self.mouse_buttons.button_state_changed(button, state);
    }

    pub fn mouse_moved(&mut self, delta_x: f32, delta_y: f32) {
        self.mouse_delta_x += delta_x;
        self.mouse_delta_y += delta_y;
    }

    pub fn update_gui_mouse_position(&mut self, scale: f32, window_height: u32) {
        self.gui_mouse_position.x = self.mouse_position.x;
        self.gui_mouse_position.y = window_height as f32 - self.mouse_position.y;
        self.gui_mouse_position /= scale;
    }

    pub fn mouse_position(&self) -> cgmath::Vector2<f32> {
        self.mouse_position
    }

    pub fn gui_mouse_position(&self) -> cgmath::Vector2<f32> {
        self.gui_mouse_position
    }

    pub fn is_mouse_in(&self, position: cgmath::Vector2<f32>, size: cgmath::Vector2<f32>) -> bool {
        self.mouse_position.x >= position.x
            && self.mouse_position.x <= position.x + size.x
            && self.mouse_position.y >= position.y
            && self.mouse_position.y <= position.y + size.y
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
            _ = window
                .set_cursor_grab(CursorGrabMode::Confined)
                .or_else(|_e| window.set_cursor_grab(CursorGrabMode::Locked));
        } else {
            _ = window.set_cursor_grab(CursorGrabMode::None);
        }

        window.set_cursor_visible(!is_locked);

        is_locked
    }

    pub fn update(&mut self) {
        self.keys.update();
        self.mouse_buttons.update();
        self.mouse_delta_x = 0.0;
        self.mouse_delta_y = 0.0;
    }
}
