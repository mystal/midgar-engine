use glium::{Surface, uniform};
use lyon_tessellation as tess;

const VERTEX_SHADER_SRC: &str = include_str!("shaders/shape.vs.glsl");
const FRAGMENT_SHADER_SRC: &str = include_str!("shaders/shape.fs.glsl");

#[derive(Clone, Copy, Debug)]
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
    transformation_matrix: Option<glm::Mat3>,
}

impl VertexConstructor {
    fn new(color: [f32; 4]) -> Self {
        Self {
            color,
            transformation_matrix: None,
        }
    }

    fn with_rotation(color: [f32; 4], pivot: glm::Vec2, rotation: f32) -> Self {
        let transformation_matrix = if rotation != 0.0 {
            let rotation_matrix = glm::rotation2d(rotation.to_radians());
            Some(glm::translation2d(&pivot) * rotation_matrix * glm::translation2d(&-pivot))
        } else {
            None
        };
        Self {
            color,
            transformation_matrix,
        }
    }

    fn vertex_data(&self, position: tess::math::Point) -> VertexData {
        let (x, y) = if let Some(trans) = self.transformation_matrix {
            let rotated_vert = trans * glm::vec3(position.x, position.y, 1.0);
            (rotated_vert.x, rotated_vert.y)
        } else {
            (position.x, position.y)
        };
        VertexData {
            pos: [x, y],
            color: self.color,
        }
    }
}

impl tess::FillVertexConstructor<VertexData> for VertexConstructor {
    fn new_vertex(&mut self, position: tess::math::Point, _attributes: tess::FillAttributes) -> VertexData {
        self.vertex_data(position)
    }
}

impl tess::StrokeVertexConstructor<VertexData> for VertexConstructor {
    fn new_vertex(&mut self, position: tess::math::Point, _attributes: tess::StrokeAttributes) -> VertexData {
        self.vertex_data(position)
    }
}

impl tess::BasicVertexConstructor<VertexData> for VertexConstructor {
    fn new_vertex(&mut self, position: tess::math::Point) -> VertexData {
        self.vertex_data(position)
    }
}

pub struct ShapeRenderer {
    projection_matrix: glm::Mat4,
    shader: glium::Program,
    vertices: tess::VertexBuffers<VertexData, u16>,
}

impl ShapeRenderer {
    // TODO: Create a builder for ShapeRenderer.
    pub fn new<F: glium::backend::Facade>(display: &F, projection: glm::Mat4) -> Self {
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

    pub fn with_shader<F: glium::backend::Facade>(_display: &F, shader: glium::Program,
                                                  projection: glm::Mat4) -> Self {
        ShapeRenderer {
            projection_matrix: projection,
            shader,
            vertices: tess::VertexBuffers::new(),
        }
    }

    pub fn queue_rect(&mut self, draw_mode: DrawMode, x: f32, y: f32, width: f32, height: f32, rotation: f32, color: [f32; 4]) {
        // Center on x and y.
        let pivot = glm::vec2(x, y);
        let (x, y) = (x - width / 2.0, y - height / 2.0);
        let vertex_ctor = VertexConstructor::with_rotation(color, pivot, rotation);
        match draw_mode {
            DrawMode::Fill => {
                let options = tess::FillOptions::default();
                tess::basic_shapes::fill_rectangle(
                    &tess::math::rect(x, y, width, height),
                    &options,
                    &mut tess::BuffersBuilder::new(&mut self.vertices, vertex_ctor),
                ).expect("Could not create a filled rectangle.");
            }
            DrawMode::Line(line_width) => {
                let options = tess::StrokeOptions::default()
                    .with_line_width(line_width);
                tess::basic_shapes::stroke_rectangle(
                    &tess::math::rect(x, y, width, height),
                    &options,
                    &mut tess::BuffersBuilder::new(&mut self.vertices, vertex_ctor),
                ).expect("Could not create a stroked rectangle.");
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
                ).expect("Could not create a filled circle.");
            }
            DrawMode::Line(line_width) => {
                let options = tess::StrokeOptions::default()
                    .with_line_width(line_width);
                tess::basic_shapes::stroke_circle(
                    tess::math::point(x, y),
                    radius,
                    &options,
                    &mut tess::BuffersBuilder::new(&mut self.vertices, vertex_ctor),
                ).expect("Could not create a stroked circle.");
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
            projectionView: *self.projection_matrix.as_ref(),
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

    pub fn set_projection_matrix(&mut self, projection: glm::Mat4) {
        self.projection_matrix = projection;
    }

    pub fn get_projection_matrix(&self) -> glm::Mat4 {
        self.projection_matrix
    }
}
