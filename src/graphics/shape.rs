use cgmath::{self, Matrix4, Vector2};
use cgmath::prelude::*;
use glium::{self, Surface, uniform};
use lyon::tessellation as tess;

const VERTEX_SHADER_SRC: &'static str = include_str!("shaders/shape.vs.glsl");
const FRAGMENT_SHADER_SRC: &'static str = include_str!("shaders/shape.fs.glsl");

pub enum DrawMode {
    Fill,
    Line(f32),
}

#[derive(Clone, Copy, Debug)]
struct VertexData {
    pos: [f32; 2],
    color: [f32; 4],
}

glium::implement_vertex!(VertexData, pos, color);

// The vertex constructor. This is the object that will be used to create custom
// vertexices from the information provided by lyon tessellators.
struct VertexConstructor {
    color: [f32; 4],
    rotation_matrix: Option<Matrix4<f32>>,
}

impl VertexConstructor {
    fn new(color: [f32; 4]) -> Self {
        Self {
            color,
            rotation_matrix: None,
        }
    }

    fn with_rotation(color: [f32; 4], pivot: Vector2<f32>, rotation: f32) -> Self {
        let rotation_matrix = if rotation != 0.0 {
            let rotate_angle = cgmath::Deg(rotation);
            let rotate_rotation = Matrix4::from_angle_z(rotate_angle);
            Some(Matrix4::from_translation(pivot.extend(0.0)) *
                 rotate_rotation *
                 Matrix4::from_translation(-pivot.extend(0.0)))
        } else {
            None
        };
        Self {
            color,
            rotation_matrix,
        }
    }
}

impl tess::VertexConstructor<tess::FillVertex, VertexData> for VertexConstructor {
    fn new_vertex(&mut self, vertex: tess::FillVertex) -> VertexData {
        let (x, y) = if let Some(rotation_matrix) = self.rotation_matrix {
            let rotated_vert = rotation_matrix * cgmath::vec4(vertex.position.x, vertex.position.y, 0.0, 1.0);
            (rotated_vert.x, rotated_vert.y)
        } else {
            (vertex.position.x, vertex.position.y)
        };
        // FillVertex also provides normals but we don't need it here.
        VertexData {
            pos: [x, y],
            color: self.color,
        }
    }
}

impl tess::VertexConstructor<tess::StrokeVertex, VertexData> for VertexConstructor {
    fn new_vertex(&mut self, vertex: tess::StrokeVertex) -> VertexData {
        let (x, y) = if let Some(rotation_matrix) = self.rotation_matrix {
            let rotated_vert = rotation_matrix * cgmath::vec4(vertex.position.x, vertex.position.y, 0.0, 1.0);
            (rotated_vert.x, rotated_vert.y)
        } else {
            (vertex.position.x, vertex.position.y)
        };
        // FillVertex also provides normals but we don't need it here.
        VertexData {
            pos: [x, y],
            color: self.color,
        }
    }
}

pub struct ShapeRenderer {
    projection_matrix: Matrix4<f32>,
    shader: glium::Program,
    vertices: tess::VertexBuffers<VertexData, u16>,
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
        let shader = glium::Program::new(display, program_creation_input)
            .expect("Could not create ShapeRenderer shader program.");

        Self::with_shader(display, shader, projection)
    }

    pub fn with_shader<F: glium::backend::Facade>(display: &F, shader: glium::Program,
                                                  projection: Matrix4<f32>) -> Self {
        ShapeRenderer {
            projection_matrix: projection,
            shader,
            vertices: tess::VertexBuffers::new(),
        }
    }

    pub fn queue_rect(&mut self, draw_mode: DrawMode, x: f32, y: f32, width: f32, height: f32, rotation: f32, color: [f32; 4]) {
        // Center on x and y.
        let pivot = Vector2::new(x, y);
        let (x, y) = (x - width / 2.0, y - height / 2.0);
        let vertex_ctor = VertexConstructor::with_rotation(color, pivot, rotation);
        match draw_mode {
            DrawMode::Fill => {
                let options = tess::FillOptions::default();
                tess::basic_shapes::fill_rectangle(
                    &tess::math::rect(x, y, width, height),
                    &options,
                    &mut tess::BuffersBuilder::new(&mut self.vertices, vertex_ctor),
                );
            }
            DrawMode::Line(line_width) => {
                let options = tess::StrokeOptions::default()
                    .with_line_width(line_width);
                tess::basic_shapes::stroke_rectangle(
                    &tess::math::rect(x, y, width, height),
                    &options,
                    &mut tess::BuffersBuilder::new(&mut self.vertices, vertex_ctor),
                );
            }
        }
    }

    pub fn queue_circle(&mut self, draw_mode: DrawMode, x: f32, y: f32, radius: f32, color: [f32; 4]) {
        // TODO: Allow setting tolerance.
        let vertex_ctor = VertexConstructor::new(color);
        match draw_mode {
            DrawMode::Fill => {
                let options = tess::FillOptions::default();
                tess::basic_shapes::fill_circle(
                    tess::math::point(x, y),
                    radius,
                    &options,
                    &mut tess::BuffersBuilder::new(&mut self.vertices, vertex_ctor),
                );
            }
            DrawMode::Line(line_width) => {
                let options = tess::StrokeOptions::default()
                    .with_line_width(line_width);
                tess::basic_shapes::stroke_circle(
                    tess::math::point(x, y),
                    radius,
                    &options,
                    &mut tess::BuffersBuilder::new(&mut self.vertices, vertex_ctor),
                );
            }
        }
    }

    pub fn draw_queued<F, S: Surface>(&mut self, display: &F, target: &mut S)
    where
        F: glium::backend::Facade,
        S: Surface,
    {
        let vertex_buffer = glium::VertexBuffer::immutable(display, &self.vertices.vertices)
            .expect("Could not create ShapeRenderer vertex buffer.");
        let index_buffer = glium::IndexBuffer::immutable(
            display,
            glium::index::PrimitiveType::TrianglesList,
            &self.vertices.indices,
        ).expect("Could not create SpriteRenderer index buffer.");

        let uniforms = uniform! {
            projectionView: cgmath::conv::array4x4(self.projection_matrix),
        };
        let params = glium::DrawParameters {
            // TODO: Only set alpha blending if necessary.
            blend: glium::Blend::alpha_blending(),
            .. Default::default()
        };

        target.draw(&vertex_buffer, &index_buffer, &self.shader, &uniforms, &params)
            .expect("Failed to draw text.");

        self.vertices.vertices.clear();
        self.vertices.indices.clear();
    }

    pub fn set_projection_matrix(&mut self, projection: Matrix4<f32>) {
        self.projection_matrix = projection;
    }

    pub fn get_projection_matrix(&self) -> Matrix4<f32> {
        self.projection_matrix
    }
}
