use glium::{self, DisplayBuild};
use glutin;

use config::MidgarAppConfig;


pub struct Graphics {
    pub display: glium::Display,
}

impl Graphics {
    // FIXME: This shouldn't be accessible outside the crate.
    pub fn new(config: &MidgarAppConfig) -> Graphics {
        // TODO: Set window options from app config
        let display = glutin::WindowBuilder::new()
            .build_glium()
            .unwrap();

        Graphics {
            display: display,
        }
    }

    fn draw(&self) {
    }
}
