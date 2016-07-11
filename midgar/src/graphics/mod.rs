use std::path::Path;

use glium;
use glium_sdl2::{DisplayBuild, SDL2Facade};
use image;
use sdl2;

use config::MidgarAppConfig;

pub mod animation;
pub mod shape;
pub mod sprite;
pub mod texture_region;


pub struct Graphics {
    display: SDL2Facade,
}

impl Graphics {
    // FIXME: This shouldn't be accessible outside the crate.
    pub fn new(config: &MidgarAppConfig, sdl_context: &sdl2::Sdl) -> Self {
        let video_subsystem = sdl_context.video().unwrap();

        // Set OpenGL version
        // TODO: Allow App to request OpenGL versions
        video_subsystem.gl_attr().set_context_version(3, 3);
        video_subsystem.gl_attr().set_context_profile(sdl2::video::GLProfile::Core);

        // Configure vsync
        let swap_interval = if config.vsync() { 1 } else { 0 };
        video_subsystem.gl_set_swap_interval(swap_interval);

        let screen_size = config.screen_size();
        let display = video_subsystem.window(config.title(), screen_size.0, screen_size.1)
            .resizable()
            .build_glium()
            .unwrap();

        Graphics {
            display: display,
        }
    }

    pub fn display(&self) -> &SDL2Facade {
        &self.display
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
