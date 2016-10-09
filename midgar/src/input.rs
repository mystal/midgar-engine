use std::collections::{HashMap, HashSet};

use sdl2;
pub use sdl2::controller::{Axis, Button, GameController};
pub use sdl2::keyboard::Keycode as KeyCode;
pub use sdl2::mouse::Mouse;


#[derive(Clone, Copy, Debug)]
pub enum ElementState {
    Pressed,
    Released,
}

//#[derive(Debug)]
pub struct Controller {
    id: u32,
    sdl_controller: GameController,
    axis_positions: HashMap<Axis, i16>,
    held_buttons: HashSet<Button>,
    pressed_buttons: HashSet<Button>,
    released_buttons: HashSet<Button>,
}

impl Controller {
    fn new(id: u32, sdl_controller: GameController) -> Self {
        Controller {
            id: id,
            sdl_controller: sdl_controller,
            axis_positions: HashMap::new(),
            held_buttons: HashSet::new(),
            pressed_buttons: HashSet::new(),
            released_buttons: HashSet::new(),
        }
    }

    pub fn get_axis_position(&self, axis: Axis) -> i16 {
        self.axis_positions.get(&axis).cloned().unwrap_or(0)
    }

    pub fn is_button_held(&self, button: &Button) -> bool {
        self.held_buttons.contains(button)
    }

    pub fn was_button_pressed(&self, button: &Button) -> bool {
        self.pressed_buttons.contains(button)
    }

    pub fn was_button_released(&self, button: &Button) -> bool {
        self.released_buttons.contains(&button)
    }

    fn begin_frame(&mut self) {
        self.pressed_buttons.clear();
        self.released_buttons.clear();
    }

    fn press_button(&mut self, button: Button) {
        self.held_buttons.insert(button);
        self.pressed_buttons.insert(button);
    }

    fn release_button(&mut self, button: Button) {
        self.held_buttons.remove(&button);
        self.released_buttons.insert(button);
    }
}

// Implement a useful structure that holds current input state.
pub struct Input {
    held_keys: HashSet<KeyCode>,
    pressed_keys: HashSet<KeyCode>,
    released_keys: HashSet<KeyCode>,

    held_buttons: HashSet<Mouse>,
    pressed_buttons: HashSet<Mouse>,
    released_buttons: HashSet<Mouse>,
    mouse_pos: (i32, i32),
    mouse_moved: bool,

    controllers: Vec<Controller>,
    controller_subsystem: sdl2::GameControllerSubsystem,
}

impl Input {
    // FIXME: This shouldn't be accessible outside the crate.
    pub fn new(sdl_context: &sdl2::Sdl) -> Self {
        // Initialize controller subsystem.
        let controller_subsystem = sdl_context.game_controller().unwrap();

        // Iterate over any currently connected devices.
        let num_joysticks = controller_subsystem.num_joysticks().unwrap();
        let controllers: Vec<_> = (0..num_joysticks)
            .filter(|&id| controller_subsystem.is_game_controller(id))
            .map(|id| {
                let sdl_controller = controller_subsystem.open(id).unwrap();
                Controller::new(id, sdl_controller)
            })
            .collect();

        Input {
            held_keys: HashSet::new(),
            pressed_keys: HashSet::new(),
            released_keys: HashSet::new(),

            held_buttons: HashSet::new(),
            pressed_buttons: HashSet::new(),
            released_buttons: HashSet::new(),
            mouse_pos: (0, 0),
            mouse_moved: false,

            controllers: controllers,
            controller_subsystem: controller_subsystem,
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

    pub fn controllers(&self) -> &[Controller] {
        self.controllers.as_slice()
    }

    // FIXME: This shouldn't be accessible outside the crate.
    pub fn begin_frame(&mut self) {
        self.pressed_keys.clear();
        self.released_keys.clear();

        self.pressed_buttons.clear();
        self.released_buttons.clear();
        self.mouse_moved = false;

        for controller in &mut self.controllers {
            controller.begin_frame();
        }
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

    pub fn handle_controller_added(&mut self, id: i32) {
        if id >= 0 {
            // TODO: Check for duplicate entry?
            let id = id as u32;
            let sdl_controller = self.controller_subsystem.open(id).unwrap();
            self.controllers.push(Controller::new(id, sdl_controller));
        } else {
            // TODO: Log error?
        }
    }

    pub fn handle_controller_removed(&mut self, id: i32) {
        let index = self.controllers.iter().enumerate()
            .find(|&(_, controller)| controller.id == id as u32)
            .map(|(i, _)| i);
        if let Some(index) = index {
            self.controllers.remove(index);
        } else {
            // TODO: Log error?
        }
    }

    pub fn handle_controller_remapped(&mut self, id: i32) {
        // TODO: Implement
    }

    pub fn handle_controller_axis(&mut self, id: i32, axis: Axis, value: i16) {
        let controller = self.controllers.iter_mut()
            .find(|controller| controller.id == id as u32);
        if let Some(controller) = controller {
            controller.axis_positions.insert(axis, value);
        } else {
            // TODO: Log error?
        }
    }

    pub fn handle_controller_button(&mut self, id: i32, state: ElementState, button: Button) {
        let controller = self.controllers.iter_mut()
            .find(|controller| controller.id == id as u32);
        if let Some(controller) = controller {
            match state {
                ElementState::Pressed => controller.press_button(button),
                ElementState::Released => controller.release_button(button),
            }
        } else {
            // TODO: Log error?
        }
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
