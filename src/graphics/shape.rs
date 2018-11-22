use cgmath::{self, Matrix4};
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

// The vertex constructor. This is the object that will be used to create the custom
// verticex from the information provided by the tessellators.
struct WithColor([f32; 4]);

impl tess::VertexConstructor<tess::FillVertex, VertexData> for WithColor {
    fn new_vertex(&mut self, vertex: tess::FillVertex) -> VertexData {
        // FillVertex also provides normals but we don't need it here.
        VertexData {
            pos: [
                vertex.position.x,
                vertex.position.y,
            ],
            color: self.0,
        }
    }
}

impl tess::VertexConstructor<tess::StrokeVertex, VertexData> for WithColor {
    fn new_vertex(&mut self, vertex: tess::StrokeVertex) -> VertexData {
        // FillVertex also provides normals but we don't need it here.
        VertexData {
            pos: [
                vertex.position.x,
                vertex.position.y,
            ],
            color: self.0,
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

    // TODO: Support rotations!(?)
    pub fn queue_rect(&mut self, draw_mode: DrawMode, x: f32, y: f32, width: f32, height: f32, color: [f32; 4]) {
        // Center on x and y.
        let (x, y) = (x - width / 2.0, y - height / 2.0);
        match draw_mode {
            DrawMode::Fill => {
                let options = tess::FillOptions::default();
                tess::basic_shapes::fill_rectangle(
                    &tess::math::rect(x, y, width, height),
                    &options,
                    &mut tess::BuffersBuilder::new(&mut self.vertices, WithColor(color)),
                );
            }
            DrawMode::Line(line_width) => {
                let options = tess::StrokeOptions::default()
                    .with_line_width(line_width);
                tess::basic_shapes::stroke_rectangle(
                    &tess::math::rect(x, y, width, height),
                    &options,
                    &mut tess::BuffersBuilder::new(&mut self.vertices, WithColor(color)),
                );
            }
        }
    }

    pub fn queue_circle(&mut self, draw_mode: DrawMode, x: f32, y: f32, radius: f32, color: [f32; 4]) {
        // TODO: Allow setting tolerance.
        match draw_mode {
            DrawMode::Fill => {
                let options = tess::FillOptions::default();
                tess::basic_shapes::fill_circle(
                    tess::math::point(x, y),
                    radius,
                    &options,
                    &mut tess::BuffersBuilder::new(&mut self.vertices, WithColor(color)),
                );
            }
            DrawMode::Line(line_width) => {
                let options = tess::StrokeOptions::default()
                    .with_line_width(line_width);
                tess::basic_shapes::stroke_circle(
                    tess::math::point(x, y),
                    radius,
                    &options,
                    &mut tess::BuffersBuilder::new(&mut self.vertices, WithColor(color)),
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
