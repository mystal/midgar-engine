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
        let screen_size = config.screen_size();
        let window_builder = glutin::WindowBuilder::new()
            .with_dimensions(screen_size.0, screen_size.1);
        let window_builder = if config.vsync() { window_builder.with_vsync() } else { window_builder };
        let display = window_builder.build_glium().unwrap();

        Graphics {
            display: display,
        }
    }

    pub fn screen_size(&self) -> (u32, u32) {
        self.display.get_framebuffer_dimensions()
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
