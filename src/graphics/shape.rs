use cgmath::{self, Matrix4};
use cgmath::prelude::*;
use glium::{self, Surface};


const VERTEX_SHADER_SRC: &'static str = include_str!("shaders/shape.vs.glsl");
const FRAGMENT_SHADER_SRC: &'static str = include_str!("shaders/shape.fs.glsl");

const QUAD_SIZE: usize = 6;

#[derive(Clone, Copy)]
struct VertexData {
    pos: [f32; 2],
    color: [f32; 4],
}

implement_vertex!(VertexData, pos, color);

pub struct ShapeRenderer {
    projection_matrix: Matrix4<f32>,
    shader: glium::Program,
    vertex_buffer: glium::VertexBuffer<VertexData>,
}

impl ShapeRenderer {
    // TODO: Create a builder for ShapeRenderer.
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

        ShapeRenderer {
            projection_matrix: projection,
            shader: shader,
            vertex_buffer: vertex_buffer,
        }
    }

    // TODO: Add a begin_batch method that creates the batched renderer for a certain shape?

    // TODO: Pull out common drawing logic.
    pub fn draw_filled_rect<S: Surface>(&self, x: f32, y: f32, width: f32, height: f32,
                                        rotation: f32, color: [f32; 4], target: &mut S) {
        // TODO: Cache model in sprite?
        let size = cgmath::vec2(width, height);
        let scale = 1.0f32;
        let position = cgmath::vec2(x, y);

        let model = {
            let scaled_size = size.mul_element_wise(scale);
            let origin = cgmath::vec2(0.5, 0.5);
            let pixel_origin = scaled_size.mul_element_wise(origin);
            // Position the shape at the origin.
            let position = position - pixel_origin;
            let translate = Matrix4::from_translation(position.extend(0.0));
            // Also rotate around the origin.
            let rotate = if rotation != 0.0 {
                let rotate_angle = cgmath::Deg(rotation);
                let rotate_rotation = Matrix4::from_angle_z(rotate_angle);
                Matrix4::from_translation(pixel_origin.extend(0.0)) *
                    rotate_rotation *
                    Matrix4::from_translation(-pixel_origin.extend(0.0))
            } else {
                Matrix4::identity()
            };
            let scale = Matrix4::from_nonuniform_scale(scaled_size.x, scaled_size.y, 1.0);
            translate * rotate * scale
        };

        // TODO: Have a const 4x4 matrix with the coordinates and simply multiply it with the
        // model. Then take slices to put into vertex data.
        let pos_top_left = model * cgmath::vec4(0.0, 1.0, 0.0, 1.0);
        let pos_top_right = model * cgmath::vec4(1.0, 1.0, 0.0, 1.0);
        let pos_bottom_left = model * cgmath::vec4(1.0, 0.0, 0.0, 1.0);
        let pos_bottom_right = model * cgmath::vec4(0.0, 0.0, 0.0, 1.0);

        let pos_top_left = [pos_top_left.x, pos_top_left.y];
        let pos_top_right = [pos_top_right.x, pos_top_right.y];
        let pos_bottom_left = [pos_bottom_left.x, pos_bottom_left.y];
        let pos_bottom_right = [pos_bottom_right.x, pos_bottom_right.y];

        let vertices = &[
            VertexData { pos: pos_top_left, color },
            VertexData { pos: pos_bottom_left, color },
            VertexData { pos: pos_bottom_right, color },
            VertexData { pos: pos_top_left, color },
            VertexData { pos: pos_top_right, color },
            VertexData { pos: pos_bottom_left, color },
        ];
        // NOTE: For batched rendering, you can allocate a big vertex buffer at the start and copy
        // vertex data each time you go to draw. So copy the data to the vertex buffer here!
        self.vertex_buffer.write(vertices);

        // FIXME: We want to use indexed vertices to pass in 4 vertices instead of 6.
        //let indices = [0, 1, 2, 0, 2, 3];
        let index_buffer = glium::index::NoIndices(glium::index::PrimitiveType::TrianglesList);

        let uniforms = uniform! {
            projectionView: cgmath::conv::array4x4(self.projection_matrix),
        };

        // TODO: Let user specify alpha blending.
        let blend =  Default::default();
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
