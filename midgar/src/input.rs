use std::collections::HashSet;

pub use sdl2::keyboard::Keycode as KeyCode;
pub use sdl2::mouse::Mouse;


pub enum ElementState {
    Pressed,
    Released,
}

// Implement a useful structure that holds current input state.
// TODO: Track mouse buttons and mouse position
pub struct Input {
    held_keys: HashSet<KeyCode>,
    pressed_keys: HashSet<KeyCode>,
    released_keys: HashSet<KeyCode>,

    held_buttons: HashSet<Mouse>,
    pressed_buttons: HashSet<Mouse>,
    released_buttons: HashSet<Mouse>,
    mouse_pos: (i32, i32),
    mouse_moved: bool,
}

impl Input {
    // FIXME: This shouldn't be accessible outside the crate.
    pub fn new() -> Self {
        Input {
            held_keys: HashSet::new(),
            pressed_keys: HashSet::new(),
            released_keys: HashSet::new(),

            held_buttons: HashSet::new(),
            pressed_buttons: HashSet::new(),
            released_buttons: HashSet::new(),
            mouse_pos: (0, 0),
            mouse_moved: false,
        }
    }

    pub fn is_key_held(&self, keycode: &KeyCode) -> bool {
        self.held_keys.contains(keycode)
    }

    pub fn was_key_pressed(&self, keycode: &KeyCode) -> bool {
        self.pressed_keys.contains(keycode)
    }

    pub fn was_key_released(&self, keycode: &KeyCode) -> bool {
        self.released_keys.contains(&keycode)
    }

    pub fn is_button_held(&self, button: &Mouse) -> bool {
        self.held_buttons.contains(button)
    }

    pub fn was_button_pressed(&self, button: &Mouse) -> bool {
        self.pressed_buttons.contains(button)
    }

    pub fn was_button_released(&self, button: &Mouse) -> bool {
        self.released_buttons.contains(&button)
    }

    pub fn mouse_pos(&self) -> (i32, i32) {
        self.mouse_pos
    }

    // FIXME: This shouldn't be accessible outside the crate.
    pub fn begin_frame(&mut self) {
        self.pressed_keys.clear();
        self.released_keys.clear();

        self.pressed_buttons.clear();
        self.released_buttons.clear();
        self.mouse_moved = false;
    }

    // FIXME: This shouldn't be accessible outside the crate.
    pub fn handle_keyboard_input(&mut self, state: ElementState, keycode: Option<KeyCode>) {
        if let Some(keycode) = keycode {
            match state {
                ElementState::Pressed => self.press_key(keycode),
                ElementState::Released => self.release_key(keycode),
            }
        }
    }

    // FIXME: This shouldn't be accessible outside the crate.
    pub fn handle_mouse_input(&mut self, state: ElementState, button: Mouse) {
        match state {
            ElementState::Pressed => self.press_button(button),
            ElementState::Released => self.release_button(button),
        }
    }

    // FIXME: This shouldn't be accessible outside the crate.
    pub fn handle_mouse_motion(&mut self, x: i32, y: i32) {
        self.mouse_pos = (x, y);
    }

    fn press_key(&mut self, keycode: KeyCode) {
        self.held_keys.insert(keycode);
        self.pressed_keys.insert(keycode);
    }

    fn release_key(&mut self, keycode: KeyCode) {
        self.held_keys.remove(&keycode);
        self.released_keys.insert(keycode);
    }

    fn press_button(&mut self, button: Mouse) {
        self.held_buttons.insert(button);
        self.pressed_buttons.insert(button);
    }

    fn release_button(&mut self, button: Mouse) {
        self.held_buttons.remove(&button);
        self.released_buttons.insert(button);
    }
}
