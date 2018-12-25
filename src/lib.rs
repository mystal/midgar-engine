pub use glium::{Surface, Texture2d};
use moving_average::MovingAverage;
pub use sdl2::event::{Event, WindowEvent};

pub use crate::app::App;
pub use crate::config::MidgarAppConfig;
pub use crate::input::{Axis, Button, Input, KeyCode, MouseButton, MouseWheelDirection};

use std::time::{
    Duration,
    Instant,
};
use std::thread;

use crate::graphics::Graphics;
use crate::input::ElementState;
use crate::time::Time;

mod app;
mod config;
pub mod graphics;
mod input;
mod time;

pub struct MidgarApp<T: App> {
    frame_duration: Duration,
    midgar: Midgar,
    app: T,
}

impl<T: App> MidgarApp<T> {
    pub fn new(config: MidgarAppConfig) -> Self {
        // Compute the frame duration from FPS.
        let frame_time_ns = (1_000_000_000.0 / config.fps() as f64) as u64;
        let frame_duration = Duration::from_nanos(frame_time_ns);

        let midgar = Midgar::new(&config);
        let app = T::new(&midgar);

        MidgarApp {
            frame_duration,
            midgar,
            app,
        }
    }

    pub fn run(mut self) {
        let mut window_closed = false;
        let mut win_size = self.midgar.graphics.screen_size();
        let mut resized: Option<(u32, u32)> = None;

        // Game loop
        while !window_closed && !self.midgar.should_exit() {
            let start_time = Instant::now();
            self.midgar.time.update();
            self.midgar.delta_times.add(self.midgar.time.delta_time());

            self.midgar.input.begin_frame();

            // Respond to event updates
            for event in self.midgar.event_pump().poll_iter() {
                use sdl2::event::Event::*;

                // Allow the app to process the raw event.
                self.app.event(&event, &mut self.midgar);

                match event {
                    // TODO: Allow apps to customize this behavior.
                    Quit { .. } => window_closed = true,

                    // Window events.
                    Window { win_event, .. } => {
                        if let WindowEvent::Resized(x, y) = win_event {
                            resized = Some((x as u32, y as u32));
                        }
                    }

                    // Keyboard events.
                    KeyDown { keycode, repeat, .. } => {
                        if !repeat {
                            self.midgar.input.handle_keyboard_input(ElementState::Pressed, keycode);
                        }
                    }
                    KeyUp { keycode, .. } =>
                        self.midgar.input.handle_keyboard_input(ElementState::Released, keycode),

                    // Mouse events.
                    MouseButtonDown { mouse_btn, .. } =>
                        self.midgar.input.handle_mouse_input(ElementState::Pressed, mouse_btn),
                    MouseButtonUp { mouse_btn, .. } =>
                        self.midgar.input.handle_mouse_input(ElementState::Released, mouse_btn),
                    MouseMotion { x, y, .. } =>
                        self.midgar.input.handle_mouse_motion(x, y),

                    // Controller events.
                    ControllerDeviceAdded { which, .. } =>
                        self.midgar.input.handle_controller_added(which),
                    ControllerDeviceRemoved { which, .. } =>
                        self.midgar.input.handle_controller_removed(which),
                    ControllerDeviceRemapped { which, .. } =>
                        self.midgar.input.handle_controller_remapped(which),
                    ControllerAxisMotion { which, axis, value, .. } =>
                        self.midgar.input.handle_controller_axis(which, axis, value),
                    ControllerButtonDown { which, button, .. } =>
                        self.midgar.input.handle_controller_button(which, ElementState::Pressed, button),
                    ControllerButtonUp { which, button, .. } =>
                        self.midgar.input.handle_controller_button(which, ElementState::Released, button),

                    _ => {}
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

            // Call app step func
            self.app.step(&mut self.midgar);

            // Get how long this frame took.
            let time_elapsed = start_time.elapsed();
            // Add it to frame times.
            self.midgar.frame_times.add(Time::duration_as_f64(time_elapsed));
            // Sleep for the rest of the remaining frame duration.
            // TODO: Add a way to have uncapped frame rate.
            if time_elapsed < self.frame_duration {
                thread::sleep(self.frame_duration - time_elapsed);
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

    frame_times: MovingAverage<f64>,
    delta_times: MovingAverage<f64>,
    should_exit: bool,
}

impl Midgar {
    fn new(config: &MidgarAppConfig) -> Self {
        let sdl_context = sdl2::init()
            .expect("Could not initialize SDL2");
        let graphics = Graphics::new(config, &sdl_context);
        let input = Input::new(&sdl_context);

        Self {
            sdl_context,
            time: Time::new(),
            graphics,
            input,

            frame_times: MovingAverage::new(200),
            delta_times: MovingAverage::new(200),
            should_exit: false,
        }
    }

    pub fn time(&self) -> &Time {
        &self.time
    }

    pub fn graphics(&self) -> &Graphics {
        &self.graphics
    }

    pub fn graphics_mut(&mut self) -> &mut Graphics {
        &mut self.graphics
    }

    pub fn input(&self) -> &Input {
        &self.input
    }

    pub fn frame_time(&self) -> f64 {
        self.frame_times.average()
    }

    pub fn fps(&self) -> f64 {
        1.0 / self.delta_times.average()
    }

    pub fn set_should_exit(&mut self) {
        self.should_exit = true
    }

    pub fn should_exit(&self) -> bool {
        self.should_exit
    }

    // TODO: This should not be public.
    pub fn event_pump(&self) -> sdl2::EventPump {
        self.sdl_context.event_pump()
            .expect("Could ont get SDL2 event pump")
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn it_works() {
    }
}
