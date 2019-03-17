use std::collections::{HashMap, HashSet};
use std::fmt;

pub use sdl2::controller::{Axis, Button, GameController};
pub use sdl2::keyboard::{KeyboardState, Keycode as KeyCode};
pub use sdl2::mouse::{MouseButton, MouseState, MouseWheelDirection};

#[derive(Clone, Copy, Debug)]
pub enum ElementState {
    Pressed,
    Released,
}

pub struct Controller {
    instance_id: i32,
    sdl_controller: GameController,
    axis_positions: HashMap<Axis, i16>,
    held_buttons: HashSet<Button>,
    pressed_buttons: HashSet<Button>,
    released_buttons: HashSet<Button>,
}

impl fmt::Debug for Controller {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Controller")
            .field("instance_id", &self.instance_id)
            .field("axis_positions", &self.axis_positions)
            .field("held_buttons", &self.held_buttons)
            .field("pressed_buttons", &self.pressed_buttons)
            .field("released_buttons", &self.released_buttons)
            .finish()
    }
}

impl Controller {
    fn new(instance_id: i32, sdl_controller: GameController) -> Self {
        Controller {
            instance_id,
            sdl_controller,
            axis_positions: HashMap::new(),
            held_buttons: HashSet::new(),
            pressed_buttons: HashSet::new(),
            released_buttons: HashSet::new(),
        }
    }

    pub fn get_axis_position(&self, axis: Axis) -> i16 {
        self.axis_positions.get(&axis).cloned().unwrap_or(0)
    }

    pub fn is_button_held(&self, button: Button) -> bool {
        self.held_buttons.contains(&button)
    }

    pub fn was_button_pressed(&self, button: Button) -> bool {
        self.pressed_buttons.contains(&button)
    }

    pub fn was_button_released(&self, button: Button) -> bool {
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
    pub(crate) event_pump: sdl2::EventPump,

    held_keys: HashSet<KeyCode>,
    pressed_keys: HashSet<KeyCode>,
    released_keys: HashSet<KeyCode>,

    held_buttons: HashSet<MouseButton>,
    pressed_buttons: HashSet<MouseButton>,
    released_buttons: HashSet<MouseButton>,
    mouse_pos: (i32, i32),
    mouse_moved: bool,

    controllers: Vec<Controller>,
    controller_subsystem: sdl2::GameControllerSubsystem,
}

impl Input {
    pub(crate) fn new(sdl_context: &sdl2::Sdl) -> Self {
        let event_pump = sdl_context.event_pump()
            .expect("Could ont get SDL2 event pump");
        // Initialize controller subsystem.
        let controller_subsystem = sdl_context.game_controller()
            .expect("Could not initialize SDL2 controller subsystem");

        Input {
            event_pump,

            held_keys: HashSet::new(),
            pressed_keys: HashSet::new(),
            released_keys: HashSet::new(),

            held_buttons: HashSet::new(),
            pressed_buttons: HashSet::new(),
            released_buttons: HashSet::new(),
            // TODO: Different initial mouse position? Option?
            mouse_pos: (0, 0),
            mouse_moved: false,

            controllers: Vec::new(),
            controller_subsystem,
        }
    }

    pub fn mouse_state(&self) -> MouseState {
        self.event_pump.mouse_state()
    }

    pub fn keyboard_state(&self) -> KeyboardState {
        self.event_pump.keyboard_state()
    }

    pub fn is_key_held(&self, keycode: KeyCode) -> bool {
        self.held_keys.contains(&keycode)
    }

    pub fn was_key_pressed(&self, keycode: KeyCode) -> bool {
        self.pressed_keys.contains(&keycode)
    }

    pub fn was_key_released(&self, keycode: KeyCode) -> bool {
        self.released_keys.contains(&keycode)
    }

    pub fn is_button_held(&self, button: MouseButton) -> bool {
        self.held_buttons.contains(&button)
    }

    pub fn was_button_pressed(&self, button: MouseButton) -> bool {
        self.pressed_buttons.contains(&button)
    }

    pub fn was_button_released(&self, button: MouseButton) -> bool {
        self.released_buttons.contains(&button)
    }

    pub fn mouse_pos(&self) -> (i32, i32) {
        self.mouse_pos
    }

    pub fn controllers(&self) -> &[Controller] {
        self.controllers.as_slice()
    }

    pub(crate) fn begin_frame(&mut self) {
        self.pressed_keys.clear();
        self.released_keys.clear();

        self.pressed_buttons.clear();
        self.released_buttons.clear();
        self.mouse_moved = false;

        for controller in &mut self.controllers {
            controller.begin_frame();
        }
    }

    pub(crate) fn handle_keyboard_input(&mut self, state: ElementState, keycode: Option<KeyCode>) {
        if let Some(keycode) = keycode {
            match state {
                ElementState::Pressed => self.press_key(keycode),
                ElementState::Released => self.release_key(keycode),
            }
        }
    }

    pub(crate) fn handle_mouse_input(&mut self, state: ElementState, button: MouseButton) {
        match state {
            ElementState::Pressed => self.press_button(button),
            ElementState::Released => self.release_button(button),
        }
    }

    pub(crate) fn handle_mouse_motion(&mut self, x: i32, y: i32) {
        self.mouse_pos = (x, y);
    }

    pub(crate) fn handle_controller_added(&mut self, joystick_id: u32) {
        // TODO: Check for duplicate entry?
        let sdl_controller = self.controller_subsystem.open(joystick_id)
            .expect(&format!("Could not open joystick {} as a controller", joystick_id));
        self.controllers.push(Controller::new(sdl_controller.instance_id(), sdl_controller));
    }

    pub(crate) fn handle_controller_removed(&mut self, instance_id: i32) {
        let index = self.controllers.iter().enumerate()
            .find(|&(_, controller)| controller.instance_id == instance_id)
            .map(|(i, _)| i);
        if let Some(index) = index {
            self.controllers.remove(index);
        } else {
            // TODO: Log error?
        }
    }

    pub(crate) fn handle_controller_remapped(&mut self, _instance_id: i32) {
        // TODO: Implement
    }

    pub(crate) fn handle_controller_axis(&mut self, instance_id: i32, axis: Axis, value: i16) {
        let controller = self.controllers.iter_mut()
            .find(|controller| controller.instance_id == instance_id);
        if let Some(controller) = controller {
            controller.axis_positions.insert(axis, value);
        } else {
            // TODO: Log error?
        }
    }

    pub(crate) fn handle_controller_button(&mut self, instance_id: i32, state: ElementState, button: Button) {
        let controller = self.controllers.iter_mut()
            .find(|controller| controller.instance_id == instance_id);
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

    fn press_button(&mut self, button: MouseButton) {
        self.held_buttons.insert(button);
        self.pressed_buttons.insert(button);
    }

    fn release_button(&mut self, button: MouseButton) {
        self.held_buttons.remove(&button);
        self.released_buttons.insert(button);
    }
}
