use std::rc::Rc;

use cgmath::{self, Matrix3, Matrix4, Vector2, Vector3};
use cgmath::prelude::*;
use glium::{self, Surface};
use glium::uniforms::MagnifySamplerFilter;

use super::texture_region::TextureRegion;


const VERTEX_SHADER_SRC: &'static str = include_str!("shaders/sprite.vs");
const FRAGMENT_SHADER_SRC: &'static str = include_str!("shaders/sprite.fs");

const QUAD_SIZE: usize = 6;


#[derive(Clone, Copy)]
struct Vertex {
    vertex: [f32; 4],
}

implement_vertex!(Vertex, vertex);


pub struct SpriteRenderer {
    shader: glium::Program,
    vertex_buffer: glium::VertexBuffer<Vertex>,
}

impl SpriteRenderer {
    pub fn new<F: glium::backend::Facade>(display: &F) -> Self {
        // NOTE: By default, assume shaders output sRGB colors.
        let program_creation_input = glium::program::ProgramCreationInput::SourceCode {
            vertex_shader: VERTEX_SHADER_SRC,
            fragment_shader: FRAGMENT_SHADER_SRC,
            geometry_shader: None,
            tessellation_control_shader: None,
            tessellation_evaluation_shader: None,
            transform_feedback_varyings: None,
            outputs_srgb: true,
            uses_point_size: false,
        };
        let shader = glium::Program::new(display, program_creation_input).unwrap();

        Self::with_shader(display, shader)
    }

    pub fn with_shader<F: glium::backend::Facade>(display: &F, shader: glium::Program) -> Self {
        // TODO: Evaluate other types of buffers.
        let vertex_buffer = glium::VertexBuffer::empty_dynamic(display, QUAD_SIZE).unwrap();

        SpriteRenderer {
            shader: shader,
            vertex_buffer: vertex_buffer,
        }
    }

    pub fn draw_region<S: Surface>(&self, region: &TextureRegion, x: f32, y: f32,
                                   width: f32, height: f32, projection: &Matrix4<f32>,
                                   target: &mut S) {
        self.draw_region_with_rotation(region, x, y, 0.0, width, height, projection, target);
    }

    // TODO: Pull out common drawing logic.
    pub fn draw_region_with_rotation<S: Surface>(&self, region: &TextureRegion, x: f32, y: f32,
                                                 rotation: f32, width: f32, height: f32,
                                                 projection: &Matrix4<f32>, target: &mut S) {
        // TODO: Cache model in sprite?
        let scale = 1.0f32;
        let position = cgmath::vec2(x, y);
        let model = {
            let scaled_size = region.size().cast::<f32>().mul_element_wise(scale);
            let translate = Matrix4::from_translation(position.extend(0.0));
            let rotate_axis = cgmath::vec3(0.0f32, 0.0, 1.0);
            let rotate_angle = cgmath::deg(rotation);
            let rotate_rotation: Matrix4<f32> = Matrix3::from_axis_angle(rotate_axis, rotate_angle.into()).into();
            // FIXME: Rotate around Sprite's origin
            let rotate =
                Matrix4::from_translation(cgmath::vec3(0.5 * scaled_size.x, 0.5 * scaled_size.y, 0.0)) *
                rotate_rotation *
                Matrix4::from_translation(cgmath::vec3(-0.5 * scaled_size.x, -0.5 * scaled_size.y, 0.0));
            let scale = Matrix4::from_nonuniform_scale(scaled_size.x, scaled_size.y, 1.0);
            translate * rotate * scale
        };

        let tex_coords = region.texture_coordinates();

        let top_left = tex_coords[0];
        let top_right = tex_coords[1];
        let bottom_left = tex_coords[2];
        let bottom_right = tex_coords[3];

        let normalized_width = width / region.size().x as f32;
        let normalized_height = height / region.size().y as f32;

        let vertices = &[
            Vertex { vertex: [0.0, normalized_height, top_left[0], top_left[1]] },
            Vertex { vertex: [normalized_width, 0.0, bottom_right[0], bottom_right[1]] },
            Vertex { vertex: [0.0, 0.0, bottom_left[0], bottom_left[1]] },
            Vertex { vertex: [0.0, normalized_height, top_left[0], top_left[1]] },
            Vertex { vertex: [normalized_width, normalized_height, top_right[0], top_right[1]] },
            Vertex { vertex: [normalized_width, 0.0, bottom_right[0], bottom_right[1]] },
        ];
        // NOTE: For batched rendering, you can allocate a big vertex buffer at the start and copy
        // vertex data each time you go to draw. So copy the data to the vertex buffer here!
        self.vertex_buffer.write(vertices);

        // FIXME: We want to use indexed vertices to pass in 4 vertices instead of 6.
        //let indices = [0, 1, 2, 0, 2, 3];
        let index_buffer = glium::index::NoIndices(glium::index::PrimitiveType::TrianglesList);

        let texture = if let Some(magnify_filter) = region.magnify_filter() {
            region.texture().sampled().magnify_filter(magnify_filter)
        } else {
            region.texture().sampled()
        };

        // TODO: Let user specify color.
        let color = [1.0f32, 1.0, 1.0];
        let uniforms = uniform! {
            image: texture,
            spriteColor: color,
            model: cgmath::conv::array4x4(model),
            view: cgmath::conv::array4x4(Matrix4::<f32>::identity()),
            projection: cgmath::conv::array4x4(*projection),
        };

        // Set alpha blending from sprite.
        let blend = if region.alpha() {
            glium::Blend::alpha_blending()
        } else {
            Default::default()
        };
        let params = glium::DrawParameters {
            blend: blend,
            .. Default::default()
        };

        target.draw(&self.vertex_buffer, &index_buffer, &self.shader, &uniforms, &params).unwrap();
    }

    pub fn draw_sprite<S: Surface>(&self, sprite: &Sprite, projection: &Matrix4<f32>,
                                   target: &mut S) {
        // TODO: Cache model in sprite?
        let model = {
            let scaled_size = sprite.size().cast::<f32>().mul_element_wise(sprite.scale);
            let translate = Matrix4::from_translation(sprite.position.cast::<f32>().extend(0.0));
            let rotate_axis = cgmath::vec3(0.0f32, 0.0, 1.0);
            let rotate_angle = cgmath::deg(sprite.rotation);
            let rotate_rotation: Matrix4<f32> = Matrix3::from_axis_angle(rotate_axis, rotate_angle.into()).into();
            // FIXME: Rotate around Sprite's origin
            let rotate =
                Matrix4::from_translation(cgmath::vec3(0.5 * scaled_size.x, 0.5 * scaled_size.y, 0.0)) *
                rotate_rotation *
                Matrix4::from_translation(cgmath::vec3(-0.5 * scaled_size.x, -0.5 * scaled_size.y, 0.0));
            let scale = Matrix4::from_nonuniform_scale(scaled_size.x, scaled_size.y, 1.0);
            translate * rotate * scale
        };

        let tex_coords = sprite.texture_coordinates();

        let top_left = tex_coords[0];
        let top_right = tex_coords[1];
        let bottom_left = tex_coords[2];
        let bottom_right = tex_coords[3];

        let vertices = &[
            Vertex { vertex: [0.0, 1.0, top_left[0], top_left[1]] },
            Vertex { vertex: [1.0, 0.0, bottom_right[0], bottom_right[1]] },
            Vertex { vertex: [0.0, 0.0, bottom_left[0], bottom_left[1]] },
            Vertex { vertex: [0.0, 1.0, top_left[0], top_left[1]] },
            Vertex { vertex: [1.0, 1.0, top_right[0], top_right[1]] },
            Vertex { vertex: [1.0, 0.0, bottom_right[0], bottom_right[1]] },
        ];
        // NOTE: For batched rendering, you can allocate a big vertex buffer at the start and copy
        // vertex data each time you go to draw. So copy the data to the vertex buffer here!
        self.vertex_buffer.write(vertices);

        // FIXME: We want to use indexed vertices to pass in 4 vertices instead of 6.
        //let indices = [0, 1, 2, 0, 2, 3];
        let index_buffer = glium::index::NoIndices(glium::index::PrimitiveType::TrianglesList);

        let texture = if let Some(magnify_filter) = sprite.magnify_filter() {
            sprite.texture().sampled().magnify_filter(magnify_filter)
        } else {
            sprite.texture().sampled()
        };

        let uniforms = uniform! {
            image: texture,
            spriteColor: cgmath::conv::array3(sprite.color),
            model: cgmath::conv::array4x4(model),
            view: cgmath::conv::array4x4(Matrix4::<f32>::identity()),
            projection: cgmath::conv::array4x4(*projection),
        };

        // Set alpha blending from sprite.
        let blend = if sprite.alpha() {
            glium::Blend::alpha_blending()
        } else {
            Default::default()
        };
        let params = glium::DrawParameters {
            blend: blend,
            .. Default::default()
        };

        target.draw(&self.vertex_buffer, &index_buffer, &self.shader, &uniforms, &params).unwrap();
    }
}

pub struct Sprite {
    texture_region: TextureRegion,

    position: Vector2<f32>,
    origin: Vector2<f32>,
    rotation: f32,
    scale: Vector2<f32>,
    color: Vector3<f32>,
}

impl Sprite {
    pub fn new(texture: Rc<glium::Texture2d>) -> Self {
        let texture_region = TextureRegion::new(texture);
        let origin = cgmath::vec2(texture_region.texture_size().x as f32 / 2.0,
                                  texture_region.texture_size().y as f32 / 2.0);

        Sprite {
            texture_region: texture_region,

            position: cgmath::vec2(0.0, 0.0),
            origin: origin,
            rotation: 0.0,
            scale: cgmath::vec2(1.0, 1.0),
            color: cgmath::vec3(1.0, 1.0, 1.0),
        }
    }

    pub fn with_sub_field(texture: Rc<glium::Texture2d>, offset: (u32, u32), size: (u32, u32)) -> Self {
        let texture_region = TextureRegion::with_sub_field(texture, offset, size);
        let origin = texture_region.offset().cast::<f32>() + (texture_region.size().cast::<f32>() / 2.0);

        Sprite {
            texture_region: texture_region,

            position: cgmath::vec2(0.0, 0.0),
            origin: origin,
            rotation: 0.0,
            scale: cgmath::vec2(1.0, 1.0),
            color: cgmath::vec3(1.0, 1.0, 1.0),
        }
    }

    pub fn set_flip_x(&mut self, flip: bool) {
        self.texture_region.set_flip_x(flip);
    }

    pub fn set_flip_y(&mut self, flip: bool) {
        self.texture_region.set_flip_y(flip);
    }

    pub fn set_position(&mut self, position: Vector2<f32>) {
        self.position = position;
    }

    pub fn position(&self) -> Vector2<f32> {
        self.position
    }

    pub fn set_rotation(&mut self, rotation: f32) {
        self.rotation = rotation;
    }

    pub fn rotation(&self) -> f32 {
        self.rotation
    }

    pub fn set_scale(&mut self, scale: Vector2<f32>) {
        self.scale = scale;
    }

    pub fn set_uniform_scale(&mut self, scale: f32) {
        self.scale = cgmath::vec2(scale, scale);
    }

    pub fn scale(&self) -> Vector2<f32> {
        self.scale
    }

    pub fn set_color(&mut self, color: Vector3<f32>) {
        self.color = color;
    }

    pub fn color(&self) -> Vector3<f32> {
        self.color
    }

    pub fn magnify_filter(&self) -> Option<glium::uniforms::MagnifySamplerFilter> {
        self.texture_region.magnify_filter()
    }

    pub fn set_magnify_filter(&mut self, magnify_filter: Option<glium::uniforms::MagnifySamplerFilter>) {
        self.texture_region.set_magnify_filter(magnify_filter);
    }

    pub fn alpha(&self) -> bool {
        self.texture_region.alpha()
    }

    pub fn set_alpha(&mut self, alpha: bool) {
        self.texture_region.set_alpha(alpha);
    }

    pub fn texture(&self) -> &glium::Texture2d {
        self.texture_region.texture()
    }

    pub fn texture_size(&self) -> Vector2<u32> {
        self.texture_region.texture_size()
    }

    pub fn offset(&self) -> Vector2<u32> {
        self.texture_region.offset()
    }

    pub fn size(&self) -> Vector2<u32> {
        self.texture_region.size()
    }

    pub fn normalized_offset(&self) -> Vector2<f32> {
        self.texture_region.normalized_offset()
    }

    pub fn normalized_size(&self) -> Vector2<f32> {
        self.texture_region.normalized_size()
    }

    pub fn texture_coordinates(&self) -> [[f32; 2]; 4] {
        self.texture_region.texture_coordinates()
    }
}
