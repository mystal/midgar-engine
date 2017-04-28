use std::rc::Rc;

use cgmath::{self, Matrix4, Vector2, Vector3};
use cgmath::prelude::*;
use glium::{self, Surface};

use super::texture_region::{TextureRegion, TextureRegionHolder};


const VERTEX_SHADER_SRC: &'static str = include_str!("shaders/sprite.vs.glsl");
const FRAGMENT_SHADER_SRC: &'static str = include_str!("shaders/sprite.fs.glsl");

const QUAD_SIZE: usize = 6;


#[derive(Clone, Copy)]
struct Vertex {
    vertex: [f32; 4],
}

implement_vertex!(Vertex, vertex);


pub struct SpriteRenderer {
    projection_matrix: Matrix4<f32>,
    shader: glium::Program,
    vertex_buffer: glium::VertexBuffer<Vertex>,
}

impl SpriteRenderer {
    // TODO: Create a builder for SpriteRenderer.
    pub fn new<F: glium::backend::Facade>(display: &F, projection: Matrix4<f32>) -> Self {
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

        Self::with_shader(display, shader, projection)
    }

    pub fn with_shader<F: glium::backend::Facade>(display: &F, shader: glium::Program,
                                                  projection: Matrix4<f32>) -> Self {
        // TODO: Evaluate other types of buffers.
        let vertex_buffer = glium::VertexBuffer::empty_dynamic(display, QUAD_SIZE).unwrap();

        SpriteRenderer {
            projection_matrix: projection,
            shader: shader,
            vertex_buffer: vertex_buffer,
        }
    }

    // TODO: Add a begin_batch method that creates the batched renderer.

    pub fn draw_region<S: Surface>(&self, region: &TextureRegion, x: f32, y: f32,
                                   width: f32, height: f32,
                                   target: &mut S) {
        self.draw_region_with_rotation(region, x, y, 0.0, width, height, target);
    }

    // TODO: Pull out common drawing logic.
    // TODO: Instead of having "with_rotation", pass a Transform structure that can scale and
    // rotate (and position?) the region. And it implements Default, or you can pass None or
    // something.
    pub fn draw_region_with_rotation<S: Surface>(&self, region: &TextureRegion, x: f32, y: f32,
                                                 rotation: f32, width: f32, height: f32,
                                                 target: &mut S) {
        // TODO: Cache model in sprite?
        let position = cgmath::vec2(x, y);
        let model = {
            let scaled_size = region.scaled_size();
            let translate = Matrix4::from_translation(position.extend(0.0));
            let rotate = if rotation != 0.0 {
                let rotate_angle = cgmath::Deg(rotation);
                let rotate_rotation = Matrix4::from_angle_z(rotate_angle);
                let origin = cgmath::vec2(0.5, 0.5);
                Matrix4::from_translation(cgmath::vec3(origin.x * scaled_size.x, origin.y * scaled_size.y, 0.0)) *
                    rotate_rotation *
                    Matrix4::from_translation(cgmath::vec3(-origin.x * scaled_size.x, -origin.y * scaled_size.y, 0.0))
            } else {
                Matrix4::identity()
            };
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
            projection: cgmath::conv::array4x4(self.projection_matrix),
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

    pub fn draw_sprite<S: Surface>(&self, sprite: &Sprite, target: &mut S) {
        // TODO: Cache model in sprite?
        let model = {
            let scaled_size = sprite.size().cast::<f32>().mul_element_wise(sprite.scale);
            let translate = Matrix4::from_translation(sprite.position.cast::<f32>().extend(0.0));
            let rotate = if sprite.rotation != 0.0 {
                let rotate_angle = cgmath::Deg(sprite.rotation);
                let rotate_rotation = Matrix4::from_angle_z(rotate_angle);
                let origin = sprite.origin();
                Matrix4::from_translation(cgmath::vec3(origin.x * scaled_size.x, origin.y * scaled_size.y, 0.0)) *
                    rotate_rotation *
                    Matrix4::from_translation(cgmath::vec3(-origin.x * scaled_size.x, -origin.y * scaled_size.y, 0.0))
            } else {
                Matrix4::identity()
            };
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
            projection: cgmath::conv::array4x4(self.projection_matrix),
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

    pub fn set_projection_matrix(&mut self, projection: Matrix4<f32>) {
        self.projection_matrix = projection;
    }

    pub fn get_projection_matrix(&self) -> Matrix4<f32> {
        self.projection_matrix
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

        Sprite {
            texture_region: texture_region,

            position: cgmath::vec2(0.0, 0.0),
            origin: cgmath::vec2(0.5, 0.5),
            rotation: 0.0,
            scale: cgmath::vec2(1.0, 1.0),
            color: cgmath::vec3(1.0, 1.0, 1.0),
        }
    }

    pub fn with_sub_field(texture: Rc<glium::Texture2d>, offset: (u32, u32), size: (u32, u32)) -> Self {
        let texture_region = TextureRegion::with_sub_field(texture, offset, size);

        Sprite {
            texture_region: texture_region,

            position: cgmath::vec2(0.0, 0.0),
            origin: cgmath::vec2(0.5, 0.5),
            rotation: 0.0,
            scale: cgmath::vec2(1.0, 1.0),
            color: cgmath::vec3(1.0, 1.0, 1.0),
        }
    }

    pub fn set_position(&mut self, position: Vector2<f32>) {
        self.position = position;
    }

    pub fn position(&self) -> Vector2<f32> {
        self.position
    }

    pub fn set_origin(&mut self, origin: Vector2<f32>) {
        // FIXME: Clamp each dimension between 0 and 1.
        self.origin = origin;
    }

    pub fn origin(&self) -> Vector2<f32> {
        self.origin
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
        // FIXME: Either clamp between 0 and 1, or use u8. Probably use our own color struct.
        self.color = color;
    }

    pub fn color(&self) -> Vector3<f32> {
        self.color
    }
}

impl TextureRegionHolder for Sprite {
    fn texture_region(&self) -> &TextureRegion {
        &self.texture_region
    }

    fn mut_texture_region(&mut self) -> &mut TextureRegion {
        &mut self.texture_region
    }
}
