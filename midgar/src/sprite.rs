use cgmath::{self, Matrix3, Matrix4, Vector2, Vector3};
use cgmath::prelude::*;
use glium::{self, Surface};


const VERTEX_SHADER_SRC: &'static str = include_str!("shaders/sprite.vs");
const FRAGMENT_SHADER_SRC: &'static str = include_str!("shaders/sprite.fs");

const QUAD_VERTICES: &'static [Vertex] = &[
    Vertex { vertex: [0.0, 1.0, 0.0, 1.0] },
    Vertex { vertex: [1.0, 0.0, 1.0, 0.0] },
    Vertex { vertex: [0.0, 0.0, 0.0, 0.0] },
    Vertex { vertex: [0.0, 1.0, 0.0, 1.0] },
    Vertex { vertex: [1.0, 1.0, 1.0, 1.0] },
    Vertex { vertex: [1.0, 0.0, 1.0, 0.0] },
];


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
    pub fn new(display: &glium::Display) -> Self {
        let shader = glium::Program::from_source(display, VERTEX_SHADER_SRC, FRAGMENT_SHADER_SRC, None).unwrap();
        let vertex_buffer = glium::VertexBuffer::new(display, QUAD_VERTICES).unwrap();

        SpriteRenderer {
            shader: shader,
            vertex_buffer: vertex_buffer,
        }
    }

    pub fn with_shader(display: &glium::Display, shader: glium::Program) -> Self {
        let vertex_buffer = glium::VertexBuffer::new(display, QUAD_VERTICES).unwrap();

        SpriteRenderer {
            shader: shader,
            vertex_buffer: vertex_buffer,
        }
    }

    pub fn draw_sprite(&self, sprite: &Sprite, projection: &Matrix4<f32>, target: &mut glium::Frame) {
        // TODO: Cache model in sprite?
        let model = {
            let scaled_size = sprite.size.mul_element_wise(sprite.scale);
            let translate = Matrix4::from_translation(sprite.position.extend(0.0));
            let rotate_axis = cgmath::vec3(0.0f32, 0.0, 1.0);
            let rotate_angle = cgmath::deg(sprite.rotation);
            let rotate_rotation: Matrix4<f32> = Matrix3::from_axis_angle(rotate_axis, rotate_angle.into()).into();
            let rotate =
                Matrix4::from_translation(cgmath::vec3(0.5 * scaled_size.x, 0.5 * scaled_size.y, 0.0)) *
                rotate_rotation *
                Matrix4::from_translation(cgmath::vec3(-0.5 * scaled_size.x, -0.5 * scaled_size.y, 0.0));
            let scale = Matrix4::from_nonuniform_scale(scaled_size.x, scaled_size.y, 1.0);
            translate * rotate * scale
        };

        let index_buffer = glium::index::NoIndices(glium::index::PrimitiveType::TrianglesList);

        let uniforms = uniform! {
            image: &sprite.texture,
            spriteColor: cgmath::conv::array3(sprite.color),
            model: cgmath::conv::array4x4(model),
            view: cgmath::conv::array4x4(Matrix4::<f32>::identity()),
            projection: cgmath::conv::array4x4(*projection),
        };

        target.draw(&self.vertex_buffer, &index_buffer, &self.shader, &uniforms, &Default::default()).unwrap();
    }
}

//pub struct Sprite<'a> {
pub struct Sprite {
    texture: glium::Texture2d,
    position: Vector2<f32>,
    size: Vector2<f32>,
    origin: Vector2<f32>,
    rotation: f32,
    scale: Vector2<f32>,
    color: Vector3<f32>,
    //bounds: Rectangle,
}

//impl<'a> Sprite<'a> {
impl Sprite {
    pub fn new(texture: glium::Texture2d) -> Self {
        let size = texture.as_surface().get_dimensions();
        let size = cgmath::vec2(size.0 as f32, size.1 as f32);
        let origin = cgmath::vec2(size.x / 2.0, size.y / 2.0);

        Sprite {
            texture: texture,
            position: cgmath::vec2(0.0, 0.0),
            size: size,
            origin: origin,
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
}
