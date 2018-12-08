use std::borrow::Borrow;
use std::rc::Rc;

#[derive(Clone)]
pub struct TextureRegion {
    texture: Rc<glium::Texture2d>,
    texture_size: glm::TVec2<u32>,
    offset: glm::TVec2<u32>,
    size: glm::TVec2<u32>,

    normalized_offset: glm::TVec2<f32>,
    normalized_size: glm::TVec2<f32>,

    //normalized_half_pixel: glm::TVec2<f32>,
}

impl TextureRegion {
    pub fn new(texture: Rc<glium::Texture2d>) -> Self {
        let texture_size = texture.dimensions();
        let texture_size = glm::vec2(texture_size.0, texture_size.1);

        TextureRegion {
            texture,
            texture_size,
            offset: glm::vec2(0, 0),
            size: texture_size,

            normalized_offset: glm::vec2(0.0, 0.0),
            normalized_size: glm::vec2(1.0, 1.0),

            //normalized_half_pixel: glm::vec2(0.5 / texture_size.x as f32, 0.5 / texture_size.y as f32),
        }
    }

    pub fn with_sub_field(texture: Rc<glium::Texture2d>, offset: (u32, u32), size: (u32, u32)) -> Self {
        let texture_size = texture.dimensions();
        let texture_size = glm::vec2(texture_size.0, texture_size.1);

        let offset = glm::vec2(offset.0, offset.1);
        let size = glm::vec2(size.0, size.1);

        let normalized_offset = glm::vec2(offset.x as f32 / texture_size.x as f32,
                                          offset.y as f32 / texture_size.y as f32);
        let normalized_size = glm::vec2(size.x as f32 / texture_size.x as f32,
                                        size.y as f32 / texture_size.y as f32);

        TextureRegion {
            texture,
            texture_size,
            offset,
            size,

            normalized_offset,
            normalized_size,

            //normalized_half_pixel: glm::vec2(0.5 / texture_size.x as f32, 0.5 / texture_size.y as f32),
        }
    }

    // TODO: Return a 2D vector or a structure that lets you index in 2D.
    pub fn split(texture: Rc<glium::Texture2d>, size: (u32, u32)) -> Vec<Self> {
        let texture_size = texture.dimensions();

        let mut regions = Vec::new();

        for j in 0..(texture_size.1 / size.1) {
            for i in 0..(texture_size.0 / size.0) {
                let offset = (i * size.0, j * size.1);
                regions.push(Self::with_sub_field(texture.clone(), offset, size))
            }
        }

        regions
    }

    pub fn texture(&self) -> &glium::Texture2d {
        self.texture.borrow()
    }

    pub fn rc_texture(&self) -> &Rc<glium::Texture2d> {
        &self.texture
    }

    pub fn texture_size(&self) -> glm::TVec2<u32> {
        self.texture_size
    }

    pub fn offset(&self) -> glm::TVec2<u32> {
        self.offset
    }

    pub fn size(&self) -> glm::TVec2<u32> {
        self.size
    }

    pub fn normalized_offset(&self) -> glm::TVec2<f32> {
        self.normalized_offset
    }

    pub fn normalized_size(&self) -> glm::TVec2<f32> {
        self.normalized_size
    }

    // Return top left, top right, bottom left, bottom right.
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

    fn texture(&self) -> &glium::Texture2d {
        self.texture_region().texture()
    }

    fn rc_texture(&self) -> &Rc<glium::Texture2d> {
        self.texture_region().rc_texture()
    }

    fn texture_size(&self) -> glm::TVec2<u32> {
        self.texture_region().texture_size()
    }

    fn offset(&self) -> glm::TVec2<u32> {
        self.texture_region().offset()
    }

    fn size(&self) -> glm::TVec2<u32> {
        self.texture_region().size()
    }

    fn normalized_offset(&self) -> glm::TVec2<f32> {
        self.texture_region().normalized_offset()
    }

    fn normalized_size(&self) -> glm::TVec2<f32> {
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
}
