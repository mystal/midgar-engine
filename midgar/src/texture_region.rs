use std::borrow::Borrow;
use std::rc::Rc;

use cgmath::{self, Vector2};
use glium::{self, Surface};


pub struct TextureRegion {
    texture: Rc<glium::Texture2d>,
    texture_size: Vector2<u32>,
    offset: Vector2<u32>,
    size: Vector2<u32>,
    flip_x: bool,
    flip_y: bool,

    normalized_offset: Vector2<f32>,
    normalized_size: Vector2<f32>,
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
            flip_x: false,
            flip_y: false,

            normalized_offset: cgmath::vec2(0.0, 0.0),
            normalized_size: cgmath::vec2(1.0, 1.0),
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
            flip_x: false,
            flip_y: false,

            normalized_offset: normalized_offset,
            normalized_size: normalized_size,
        }
    }

    pub fn set_flip_x(&mut self, flip: bool) {
        self.flip_x = flip;
    }

    pub fn set_flip_y(&mut self, flip: bool) {
        self.flip_y = flip;
    }

    pub fn texture(&self) -> &glium::Texture2d {
        self.texture.borrow()
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

        match (self.flip_x, self.flip_y) {
            (false, false) => [top_left, top_right, bot_left, bot_right],
            (true, false) => [top_right, top_left, bot_right, bot_left],
            (false, true) => [bot_left, bot_right, top_left, top_right],
            (true, true) => [bot_right, bot_left, top_right, top_left],
        }
    }
}
