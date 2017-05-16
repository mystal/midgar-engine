use std::borrow::Borrow;
use std::rc::Rc;

use cgmath::{self, Vector2};
use glium::{self, Surface};


#[derive(Clone)]
pub struct TextureRegion {
    texture: Rc<glium::Texture2d>,
    texture_size: Vector2<u32>,
    offset: Vector2<u32>,
    size: Vector2<u32>,

    normalized_offset: Vector2<f32>,
    normalized_size: Vector2<f32>,

    //normalized_half_pixel: Vector2<f32>,
}

impl TextureRegion {
    pub fn new(texture: Rc<glium::Texture2d>) -> Self {
        let texture_size = texture.as_surface().get_dimensions();
        let texture_size = cgmath::vec2(texture_size.0, texture_size.1);

        TextureRegion {
            texture: texture,
            texture_size: texture_size,
            offset: cgmath::vec2(0, 0),
            size: texture_size,

            normalized_offset: cgmath::vec2(0.0, 0.0),
            normalized_size: cgmath::vec2(1.0, 1.0),

            //normalized_half_pixel: cgmath::vec2(0.5 / texture_size.x as f32, 0.5 / texture_size.y as f32),
        }
    }

    pub fn with_sub_field(texture: Rc<glium::Texture2d>, offset: (u32, u32), size: (u32, u32)) -> Self {
        let texture_size = texture.as_surface().get_dimensions();
        let texture_size = cgmath::vec2(texture_size.0, texture_size.1);

        let offset = cgmath::vec2(offset.0, offset.1);
        let size = cgmath::vec2(size.0, size.1);

        let normalized_offset = cgmath::vec2(offset.x as f32 / texture_size.x as f32,
                                             offset.y as f32 / texture_size.y as f32);
        let normalized_size = cgmath::vec2(size.x as f32 / texture_size.x as f32,
                                           size.y as f32 / texture_size.y as f32);

        TextureRegion {
            texture: texture,
            texture_size: texture_size,
            offset: offset,
            size: size,

            normalized_offset: normalized_offset,
            normalized_size: normalized_size,

            //normalized_half_pixel: cgmath::vec2(0.5 / texture_size.x as f32, 0.5 / texture_size.y as f32),
        }
    }

    pub fn texture(&self) -> &glium::Texture2d {
        self.texture.borrow()
    }

    pub fn rc_texture(&self) -> &Rc<glium::Texture2d> {
        &self.texture
    }

    pub fn texture_size(&self) -> Vector2<u32> {
        self.texture_size
    }

    pub fn offset(&self) -> Vector2<u32> {
        self.offset
    }

    pub fn size(&self) -> Vector2<u32> {
        self.size
    }

    pub fn normalized_offset(&self) -> Vector2<f32> {
        self.normalized_offset
    }

    pub fn normalized_size(&self) -> Vector2<f32> {
        self.normalized_size
    }

    // Return top left, top right, bottom left, bottom right
    pub fn texture_coordinates(&self) -> [[f32; 2]; 4] {
        let top_left = [self.normalized_offset.x, self.normalized_offset.y + self.normalized_size.y];
        let top_right = [self.normalized_offset.x + self.normalized_size.x, self.normalized_offset.y + self.normalized_size.y];
        let bot_left = [self.normalized_offset.x, self.normalized_offset.y];
        let bot_right = [self.normalized_offset.x + self.normalized_size.x, self.normalized_offset.y];

        // NOTE: Code to try to set UV corners to be in the middle of texels. See:
        // http://stackoverflow.com/a/6051557
        //let top_left = [self.normalized_offset.x + self.normalized_half_pixel.x,
        //                self.normalized_offset.y + self.normalized_size.y - self.normalized_half_pixel.y];
        //let top_right = [self.normalized_offset.x + self.normalized_size.x - self.normalized_half_pixel.x,
        //                 self.normalized_offset.y + self.normalized_size.y - self.normalized_half_pixel.y];
        //let bot_left = [self.normalized_offset.x + self.normalized_half_pixel.x,
        //                self.normalized_offset.y + self.normalized_half_pixel.y];
        //let bot_right = [self.normalized_offset.x + self.normalized_size.x - self.normalized_half_pixel.x,
        //                 self.normalized_offset.y + self.normalized_half_pixel.y];

        [top_left, top_right, bot_left, bot_right]
    }
}

pub trait TextureRegionHolder {
    fn texture_region(&self) -> &TextureRegion;
    fn mut_texture_region(&mut self) -> &mut TextureRegion;

    fn texture(&self) -> &glium::Texture2d {
        self.texture_region().texture()
    }

    fn rc_texture(&self) -> &Rc<glium::Texture2d> {
        self.texture_region().rc_texture()
    }

    fn texture_size(&self) -> Vector2<u32> {
        self.texture_region().texture_size()
    }

    fn offset(&self) -> Vector2<u32> {
        self.texture_region().offset()
    }

    fn size(&self) -> Vector2<u32> {
        self.texture_region().size()
    }

    fn normalized_offset(&self) -> Vector2<f32> {
        self.texture_region().normalized_offset()
    }

    fn normalized_size(&self) -> Vector2<f32> {
        self.texture_region().normalized_size()
    }

    fn texture_coordinates(&self) -> [[f32; 2]; 4] {
        self.texture_region().texture_coordinates()
    }
}

impl TextureRegionHolder for TextureRegion {
    fn texture_region(&self) -> &TextureRegion {
        self
    }

    fn mut_texture_region(&mut self) -> &mut TextureRegion {
        self
    }
}
