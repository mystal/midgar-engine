extern crate cgmath;
#[macro_use]
extern crate glium;
extern crate glium_sdl2;
extern crate image;
extern crate sdl2;

pub use glium::{Surface, Texture2d};
pub use glium::uniforms::MagnifySamplerFilter;

pub use app::App;
pub use config::MidgarAppConfig;
pub use input::KeyCode;

use std::time::{
    Duration,
    Instant,
};
use std::thread;

use graphics::Graphics;
use input::{ElementState, Input};
use time::Time;

mod app;
mod config;
pub mod graphics;
mod input;
mod time;


pub struct MidgarApp<T: App> {
    frame_time: Duration,
    midgar: Midgar,
    app: T,
}

impl<T: App> MidgarApp<T> {
    pub fn new(config: MidgarAppConfig) -> Self {
        // Compute the frame_time Duration from FPS.
        // TODO: Consider using nanosecond accuracy instead of milliseconds.
        let frame_time_ms = ((1.0 / config.fps() as f64) * 1000.0) as u64;
        let frame_time = Duration::from_millis(frame_time_ms);

        let midgar = Midgar::new(&config);
        let app = T::create(&midgar);

        MidgarApp {
            frame_time: frame_time,
            midgar: midgar,
            app: app,
        }
    }

    pub fn run(mut self) {
        use sdl2::event::{Event, WindowEventId};

        let mut window_closed = false;
        let mut win_size = self.midgar.graphics.screen_size();
        let mut resized: Option<(u32, u32)> = None;

        // Game loop
        while !window_closed && !self.midgar.should_exit() {
            let start_time = Instant::now();
            self.midgar.time.update();

            self.midgar.input.begin_frame();

            // Respond to event updates
            for event in self.midgar.event_pump().poll_iter() {
                match event {
                    Event::Quit { .. } => window_closed = true,
                    Event::Window { win_event_id, data1, data2, .. } => {
                        if win_event_id == WindowEventId::SizeChanged {
                            resized = Some((data1 as u32, data2 as u32));
                        }
                    },
                    Event::KeyDown { keycode, .. } =>
                        self.midgar.input.handle_keyboard_input(ElementState::Pressed, keycode),
                    Event::KeyUp { keycode, .. } =>
                        self.midgar.input.handle_keyboard_input(ElementState::Released, keycode),
                    _ => {},
                }
            }

            // TODO: Implement resizing via glutin's resize callback. Simply track the last call to
            // the callback.

            // Detect resize on platforms where Resized event does not work.
            let cur_win_size = self.midgar.graphics.screen_size();
            if cur_win_size != win_size {
                resized = Some(cur_win_size);
                win_size = cur_win_size;
            }
            if let Some(size) = resized {
                self.app.resize(size, &self.midgar);
                resized = None;
            }

            // TODO: Process input events

            // Call app step func
            self.app.step(&mut self.midgar);

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
    sdl_context: sdl2::Sdl,
    time: Time,
    graphics: Graphics,
    input: Input,
    should_exit: bool,
}

impl Midgar {
    fn new(config: &MidgarAppConfig) -> Self {
        let sdl_context = sdl2::init().unwrap();
        let graphics = Graphics::new(config, &sdl_context);

        Midgar {
            sdl_context: sdl_context,
            time: Time::new(),
            graphics: graphics,
            input: Input::new(),
            should_exit: false,
        }
    }

    pub fn time(&self) -> &Time {
        &self.time
    }

    pub fn graphics(&self) -> &Graphics {
        &self.graphics
    }

    pub fn input(&self) -> &Input {
        &self.input
    }

    pub fn set_should_exit(&mut self) {
        self.should_exit = true
    }

    pub fn should_exit(&self) -> bool {
        self.should_exit
    }

    fn event_pump(&self) -> sdl2::EventPump {
        self.sdl_context.event_pump().unwrap()
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn it_works() {
    }
}
