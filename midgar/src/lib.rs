extern crate glium;
extern crate glutin;

use std::time::Instant;

use glium::DisplayBuild;

pub trait App {
    fn create(midgar: &Midgar) -> Self;
    fn step(&mut self, midgar: &Midgar) {}
    fn resize(&mut self, width: u32, height: u32, midgar: &Midgar) {}
    fn pause(&mut self, midgar: &Midgar) {}
    fn resume(&mut self, midgar: &Midgar) {}
    fn destroy(&mut self, midgar: &Midgar) {}
}

pub struct MidgarAppConfig; /*{
}*/

impl MidgarAppConfig {
    pub fn new() -> Self {
        MidgarAppConfig
    }
}

pub struct MidgarApp<T: App> {
    midgar: Midgar,
    app: T,
}

impl<T: App> MidgarApp<T> {
    pub fn new(config: MidgarAppConfig) -> Self {
        // TODO: Set window options from app config
        let display = glutin::WindowBuilder::new()
            .build_glium()
            .unwrap();

        let midgar = Midgar::new(display);
        let app = T::create(&midgar);

        MidgarApp {
            midgar: midgar,
            app: app,
        }
    }

    pub fn run(mut self) {
        let mut running = true;

        // Game loop
        while running {
            // TODO: Gather events
            for event in self.midgar.display.poll_events() {
                match event {
                    glutin::Event::Closed => running = false,
                    //glutin::Event::Resized(width, height) => self.app.resize(width, height, &self.midgar),
                    //glutin::Event::ReceivedCharacter(c) => println!("Char: {}", c),
                    _ => {},
                }
            }

            // TODO: Implement resizing via glutin's resize callback. Simply track the last call to
            // the callback.
            // TODO: Process input events

            // Call app step func
            self.app.step(&self.midgar);

            // TODO: Sleep zzzzz
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
            input: Input,
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

// TODO: Implement a useful structure that holds current input state.
pub struct Input; /*{
}*/

#[cfg(test)]
mod test {
    #[test]
    fn it_works() {
    }
}
