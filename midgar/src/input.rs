pub use glutin::VirtualKeyCode;

use std::collections::HashSet;

use glutin;


// Implement a useful structure that holds current input state.
// TODO: Track mouse buttons and mouse position
pub struct Input {
    held_keys: HashSet<glutin::VirtualKeyCode>,
    pressed_keys: HashSet<glutin::VirtualKeyCode>,
    released_keys: HashSet<glutin::VirtualKeyCode>,
}

impl Input {
    // FIXME: This shouldn't be accessible outside the crate.
    pub fn new() -> Self {
        Input {
            held_keys: HashSet::new(),
            pressed_keys: HashSet::new(),
            released_keys: HashSet::new(),
        }
    }

    pub fn is_key_held(&self, keycode: &glutin::VirtualKeyCode) -> bool {
        self.held_keys.contains(keycode)
    }

    pub fn was_key_pressed(&self, keycode: &glutin::VirtualKeyCode) -> bool {
        self.pressed_keys.contains(keycode)
    }

    pub fn was_key_released(&self, keycode: &glutin::VirtualKeyCode) -> bool {
        self.released_keys.contains(&keycode)
    }

    // FIXME: This shouldn't be accessible outside the crate.
    pub fn begin_frame(&mut self) {
        self.pressed_keys.clear();
        self.released_keys.clear();
    }

    // FIXME: This shouldn't be accessible outside the crate.
    pub fn handle_keyboard_input(&mut self, state: glutin::ElementState, scancode: glutin::ScanCode,
                             keycode: Option<glutin::VirtualKeyCode>) {
        if let Some(keycode) = keycode {
            match state {
                glutin::ElementState::Pressed => self.press_key(keycode),
                glutin::ElementState::Released => self.release_key(keycode),
            }
        }
    }

    fn press_key(&mut self, keycode: glutin::VirtualKeyCode) {
        self.held_keys.insert(keycode);
        self.pressed_keys.insert(keycode);
    }

    fn release_key(&mut self, keycode: glutin::VirtualKeyCode) {
        self.held_keys.remove(&keycode);
        self.released_keys.insert(keycode);
    }
}
