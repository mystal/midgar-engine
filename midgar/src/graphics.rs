use std::path::Path;

use glium::{self, DisplayBuild};
use glutin;
use image;

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

    // FIXME: Return a Result.
    pub fn load_texture<P: AsRef<Path>>(&self, path: P) -> glium::Texture2d {
        let image = image::open(path).unwrap().to_rgba();
        let image_dimensions = image.dimensions();
        let image = glium::texture::RawImage2d::from_raw_rgba_reversed(image.into_raw(), image_dimensions);
        glium::Texture2d::new(&self.display, image).unwrap()
    }

    fn draw(&self) {
    }
}
