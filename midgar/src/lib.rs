extern crate glium;
extern crate glutin;

pub use glium::Surface;

use std::collections::HashSet;
use std::time::{
    Duration,
    Instant,
};
use std::thread;

use glium::DisplayBuild;

pub trait App {
    fn create(midgar: &Midgar) -> Self;
    fn step(&mut self, midgar: &Midgar) {}
    fn resize(&mut self, width: u32, height: u32, midgar: &Midgar) {}
    fn pause(&mut self, midgar: &Midgar) {}
    fn resume(&mut self, midgar: &Midgar) {}
    fn destroy(&mut self, midgar: &Midgar) {}
}

pub struct MidgarAppConfig {
    fps: u8,
}

impl MidgarAppConfig {
    pub fn new() -> Self {
        MidgarAppConfig {
            fps: 60,
        }
    }
}

pub struct MidgarApp<T: App> {
    frame_time: Duration,
    midgar: Midgar,
    app: T,
}

impl<T: App> MidgarApp<T> {
    pub fn new(config: MidgarAppConfig) -> Self {
        // Compute the frame_time Duration from FPS.
        // TODO: Consider using nanosecond accuracy instead of milliseconds.
        let frame_time_ms = ((1.0 / config.fps as f64) * 1000.0) as u64;
        let frame_time = Duration::from_millis(frame_time_ms);

        // TODO: Set window options from app config
        let display = glutin::WindowBuilder::new()
            .build_glium()
            .unwrap();

        let midgar = Midgar::new(display);
        let app = T::create(&midgar);

        MidgarApp {
            frame_time: frame_time,
            midgar: midgar,
            app: app,
        }
    }

    pub fn run(mut self) {
        let mut running = true;

        // Game loop
        while running {
            let start_time = Instant::now();

            self.midgar.input.begin_frame();

            // TODO: Gather events
            for event in self.midgar.display.poll_events() {
                match event {
                    glutin::Event::Closed => running = false,
                    //glutin::Event::Resized(width, height) => self.app.resize(width, height, &self.midgar),
                    glutin::Event::KeyboardInput(state, scancode, keycode) =>
                        self.midgar.input.handle_keyboard_input(state, scancode, keycode),
                    //glutin::Event::ReceivedCharacter(c) => println!("Char: {}", c),
                    _ => {},
                }
            }

            // TODO: Implement resizing via glutin's resize callback. Simply track the last call to
            // the callback.
            // TODO: Process input events

            // Call app step func
            self.app.step(&self.midgar);

            // Sleep
            let time_elapsed = start_time.elapsed();
            if time_elapsed < self.frame_time {
                thread::sleep(self.frame_time - time_elapsed);
            }
        }

        self.app.destroy(&self.midgar);
    }
}

pub struct Midgar {
    //pub time: Time,
    pub display: glium::Display,
    pub graphics: Graphics,
    pub input: Input,
}

impl Midgar {
    fn new(display: glium::Display) -> Self {
        Midgar {
            //time: Time::new(),
            display: display,
            graphics: Graphics,
            input: Input::new(),
        }
    }
}

pub struct Time {
    // The number of nanoseconds since the last call to step (current_time - last_frame_time)
    pub delta_time: u64,
    last_frame_time: u64,
}

// TODO: Implement time's methods

// TODO: What the fuck do we do here?
pub struct Graphics; /*{
}*/

// Implement a useful structure that holds current input state.
// TODO: Track mouse buttons and mouse position
pub struct Input {
    held_keys: HashSet<glutin::VirtualKeyCode>,
    pressed_keys: HashSet<glutin::VirtualKeyCode>,
    released_keys: HashSet<glutin::VirtualKeyCode>,
}

impl Input {
    fn new() -> Self {
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

    fn begin_frame(&mut self) {
        self.pressed_keys.clear();
        self.released_keys.clear();
    }

    fn handle_keyboard_input(&mut self, state: glutin::ElementState, scancode: glutin::ScanCode,
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

#[cfg(test)]
mod test {
    #[test]
    fn it_works() {
    }
}
