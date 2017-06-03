use std::borrow::Borrow;
use std::rc::Rc;
use std::thread;

use cgmath::{self, Matrix4, Vector2, Vector3};
use cgmath::prelude::*;
use glium::{self, DrawError, GlObject, Surface};
use glium::uniforms::{Sampler, SamplerBehavior};
pub use glium::uniforms::{MagnifySamplerFilter, MinifySamplerFilter, SamplerWrapFunction};

use super::texture_region::{TextureRegion, TextureRegionHolder};


const VERTEX_SHADER_SRC: &'static str = include_str!("shaders/sprite.vs.glsl");
const FRAGMENT_SHADER_SRC: &'static str = include_str!("shaders/sprite.fs.glsl");

const QUAD_VERTEX_SIZE: usize = 4;
const QUAD_INDEX_SIZE: usize = 6;
const BATCH_SIZE: usize = 1024;
const BATCH_VERTEX_SIZE: usize = QUAD_VERTEX_SIZE * BATCH_SIZE;
const BATCH_INDEX_SIZE: usize = QUAD_INDEX_SIZE * BATCH_SIZE;


#[derive(Clone, Copy)]
struct VertexData {
    pos: [f32; 2],
    tex_coords: [f32; 2],
    color: [f32; 3],
}
implement_vertex!(VertexData, pos, tex_coords, color);


#[derive(Clone, Copy, Debug, Default)]
pub struct SpriteDrawParams {
    pub sampler_behavior: SamplerBehavior,
    pub alpha_blending: bool,
}

impl SpriteDrawParams {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn alpha(mut self, alpha: bool) -> Self {
        self.alpha_blending = alpha;
        self
    }

    pub fn wrap_function(mut self, function: SamplerWrapFunction) -> Self {
        self.sampler_behavior.wrap_function = (function, function, function);
        self
    }

    pub fn minify_filter(mut self, filter: MinifySamplerFilter) -> Self {
        self.sampler_behavior.minify_filter = filter;
        self
    }

    pub fn magnify_filter(mut self, filter: MagnifySamplerFilter) -> Self {
        self.sampler_behavior.magnify_filter = filter;
        self
    }
}

pub struct SpriteBatch<'a, 'b, S>
    where S: 'b + Surface
{
    renderer: &'a mut SpriteRenderer,
    target: &'b mut S,
    draw_params: SpriteDrawParams,
    draw_calls: u32,
    finished: bool,
}

impl<'a, 'b, S> SpriteBatch<'a, 'b, S>
    where S: Surface
{
    // TODO: Take a builder that sets the draw options. Blend mode, texture options, etc. Draw
    // state that will be used for the entire batch.
    fn new(renderer: &'a mut SpriteRenderer, draw_params: SpriteDrawParams, target: &'b mut S) -> Self {
        renderer.sprite_queue.clear();

        SpriteBatch {
            renderer: renderer,
            target: target,
            draw_params: draw_params,
            draw_calls: 0,
            finished: false,
        }
    }

    pub fn draw(&mut self, sprite: &Sprite) -> Result<(), DrawError> {
        if self.renderer.sprite_queue.len() == BATCH_SIZE {
            self.flush()?;
        }

        // Queue the sprite data.
        let vertices = sprite.get_vertex_data();
        self.renderer.sprite_queue.push(vertices, sprite.rc_texture().clone());

        Ok(())
    }

    pub fn finish(mut self) -> Result<u32, DrawError> {
        self.flush()?;
        self.finished = true;
        Ok(self.draw_calls)
    }

    fn flush(&mut self) -> Result<(), DrawError> {
        if self.renderer.sprite_queue.vertices.is_empty() {
            return Ok(());
        }

        // Build draw parameters for the entire batch.
        let params = {
            let blend = if self.draw_params.alpha_blending {
                glium::Blend::alpha_blending()
            } else {
                Default::default()
            };
            glium::DrawParameters {
                blend: blend,
                .. Default::default()
            }
        };

        // TODO: If sorting sprites, do so before writing vertex data.
        // Write vertex data to the vertex buffer.
        {
            let vertex_buffer = self.renderer.vertex_buffer.slice(0..self.renderer.sprite_queue.vertices.len())
                .expect("Vertex buffer does not contain enough elements!");
            vertex_buffer.write(&self.renderer.sprite_queue.vertices);
        }

        // TODO: Flush sprites based on their textures.
        // TODO: Do a fold, using accumulator as previous texture ID and range? Use it to queue up flushes?
        // TODO: Return ranges of sprites that all have same texture, use that to flush out each
        // range.

        let mut render_texture = self.renderer.sprite_queue.textures[0].clone();
        let mut offset = 0;
        for (i, texture) in self.renderer.sprite_queue.textures.iter().enumerate().skip(1) {
            if texture.get_id() != render_texture.get_id() {
                {
                    let sampler: Sampler<glium::Texture2d> = glium::uniforms::Sampler(
                        render_texture.borrow(),
                        self.draw_params.sampler_behavior,
                    );
                    let uniforms = uniform! {
                        image: sampler,
                        projectionView: cgmath::conv::array4x4(self.renderer.projection_matrix),
                    };

                    // Draw the batch.
                    let (vertex_start, vertex_end) = (offset * QUAD_VERTEX_SIZE, i * QUAD_VERTEX_SIZE);
                    let vertex_buffer = self.renderer.vertex_buffer.slice(vertex_start..vertex_end)
                        .expect("Vertex buffer does not contain enough elements!");
                    let (index_start, index_end) = (offset * QUAD_INDEX_SIZE, i * QUAD_INDEX_SIZE);
                    let index_buffer = self.renderer.index_buffer.slice(index_start..index_end)
                        .expect("Index buffer does not contain enough elements!");

                    self.target.draw(vertex_buffer, index_buffer, &self.renderer.shader, &uniforms, &params)?;
                }

                self.draw_calls += 1;

                offset = i;
                render_texture = texture.clone();
            }
        }

        // Draw any final sprites.
        {
            let i = self.renderer.sprite_queue.len();

            let sampler: Sampler<glium::Texture2d> = glium::uniforms::Sampler(
                render_texture.borrow(),
                self.draw_params.sampler_behavior,
            );
            let uniforms = uniform! {
                image: sampler,
                projectionView: cgmath::conv::array4x4(self.renderer.projection_matrix),
            };

            // Draw the batch.
            let (vertex_start, vertex_end) = (offset * QUAD_VERTEX_SIZE, i * QUAD_VERTEX_SIZE);
            let vertex_buffer = self.renderer.vertex_buffer.slice(vertex_start..vertex_end)
                .expect("Vertex buffer does not contain enough elements!");
            let (index_start, index_end) = (offset * QUAD_INDEX_SIZE, i * QUAD_INDEX_SIZE);
            let index_buffer = self.renderer.index_buffer.slice(index_start..index_end)
                .expect("Index buffer does not contain enough elements!");

            self.target.draw(vertex_buffer, index_buffer, &self.renderer.shader, &uniforms, &params)?;

            self.draw_calls += 1;
        }

        // All sprites have been flushed, so clear out the queue.
        self.renderer.sprite_queue.clear();

        Ok(())
    }
}

impl<'a, 'b, S> Drop for SpriteBatch<'a, 'b, S>
    where S: Surface
{
    #[inline]
    fn drop(&mut self) {
        if !thread::panicking() {
            assert!(self.finished, "The `SpriteBatch` object must be explicitly destroyed \
                                    by calling `.finish()`");
        }
    }
}

pub struct SpriteQueue {
    vertices: Vec<VertexData>,
    textures: Vec<Rc<glium::Texture2d>>,
}

impl SpriteQueue {
    fn new() -> Self {
        SpriteQueue {
            vertices: Vec::with_capacity(BATCH_VERTEX_SIZE),
            textures: Vec::with_capacity(BATCH_SIZE),
        }
    }

    fn push(&mut self, vertices: [VertexData; 4], texture: Rc<glium::Texture2d>) {
        assert!(self.textures.len() < BATCH_SIZE, "Sprite queue is full!");

        self.vertices.extend_from_slice(&vertices);
        self.textures.push(texture);
    }

    fn clear(&mut self) {
        self.vertices.clear();
        self.textures.clear();
    }

    fn len(&self) -> usize {
        self.textures.len()
    }
}

pub struct SpriteRenderer {
    projection_matrix: Matrix4<f32>,
    shader: glium::Program,
    vertex_buffer: glium::VertexBuffer<VertexData>,
    index_buffer: glium::IndexBuffer<u16>,
    sprite_queue: SpriteQueue,
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
        let vertex_buffer = glium::VertexBuffer::empty_dynamic(
            display,
            BATCH_VERTEX_SIZE,
        ).unwrap();

        let mut indices = Vec::with_capacity(BATCH_INDEX_SIZE);
        for quad_index in 0..BATCH_SIZE {
            let offset = quad_index as u16 * QUAD_VERTEX_SIZE as u16;
            let new_indices = [
                0 + offset, 1 + offset, 2 + offset,
                0 + offset, 2 + offset, 3 + offset,
            ];
            indices.extend_from_slice(&new_indices);
        }
        let index_buffer = glium::IndexBuffer::immutable(
            display,
            glium::index::PrimitiveType::TrianglesList,
            &indices,
        ).unwrap();

        SpriteRenderer {
            projection_matrix: projection,
            shader: shader,
            vertex_buffer: vertex_buffer,
            index_buffer: index_buffer,
            sprite_queue: SpriteQueue::new(),
        }
    }

    pub fn begin_batch<'a, 'b, S: Surface>(&'a mut self, draw_params: SpriteDrawParams, target: &'b mut S) -> SpriteBatch<'a, 'b, S> {
        SpriteBatch::new(self, draw_params, target)
    }

    pub fn draw<S: Surface>(&self, sprite: &Sprite, draw_params: SpriteDrawParams, target: &mut S) {
        let vertices = sprite.get_vertex_data();

        // Copy the data to the vertex buffer here for immediate rendering.
        let vertex_buffer = self.vertex_buffer.slice(0..QUAD_VERTEX_SIZE)
            .expect("Vertex buffer does not contain enough elements!");
        vertex_buffer.write(&vertices);

        let sampler: Sampler<glium::Texture2d> = glium::uniforms::Sampler(
            sprite.texture(),
            draw_params.sampler_behavior,
        );

        let uniforms = uniform! {
            image: sampler,
            projectionView: cgmath::conv::array4x4(self.projection_matrix),
        };

        let blend = if draw_params.alpha_blending {
            glium::Blend::alpha_blending()
        } else {
            Default::default()
        };
        let params = glium::DrawParameters {
            blend: blend,
            .. Default::default()
        };

        let index_buffer = self.index_buffer.slice(0..QUAD_INDEX_SIZE)
            .expect("Index buffer does not contain enough elements!");

        target.draw(vertex_buffer, index_buffer, &self.shader, &uniforms, &params).unwrap();
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
    flip_x: bool,
    flip_y: bool,
}

impl Sprite {
    pub fn new(texture: Rc<glium::Texture2d>) -> Self {
        let texture_region = TextureRegion::new(texture);
        Sprite::from_texture_region(texture_region)
    }

    pub fn with_sub_field(texture: Rc<glium::Texture2d>, offset: (u32, u32), size: (u32, u32)) -> Self {
        let texture_region = TextureRegion::with_sub_field(texture, offset, size);
        Sprite::from_texture_region(texture_region)
    }

    pub fn from_texture_region(texture_region: TextureRegion) -> Self {
        Sprite {
            texture_region: texture_region,

            position: cgmath::vec2(0.0, 0.0),
            origin: cgmath::vec2(0.5, 0.5),
            rotation: 0.0,
            scale: cgmath::vec2(1.0, 1.0),
            color: cgmath::vec3(1.0, 1.0, 1.0),
            flip_x: false,
            flip_y: false,
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

    //pub fn scaled_size(&self) -> Vector2<f32> {
    //    self.size.cast::<f32>().mul_element_wise(self.scale)
    //}

    pub fn set_flip_x(&mut self, flip_x: bool) {
        self.flip_x = flip_x;
    }

    pub fn flip_x(&mut self) -> bool {
        self.flip_x
    }

    pub fn set_flip_y(&mut self, flip_y: bool) {
        self.flip_y = flip_y;
    }

    pub fn flip_y(&mut self) -> bool {
        self.flip_y
    }

    pub fn set_color(&mut self, color: Vector3<f32>) {
        // FIXME: Either clamp between 0 and 1, or use u8. Probably use our own color struct.
        self.color = color;
    }

    pub fn color(&self) -> Vector3<f32> {
        self.color
    }

    fn get_vertex_data(&self) -> [VertexData; 4] {
        // TODO: Cache model matrix in the sprite?
        // Compute model matrix.
        let model = {
            let scaled_size = self.size().cast::<f32>().mul_element_wise(self.scale);
            let translate = Matrix4::from_translation(self.position.cast::<f32>().extend(0.0));
            let rotate = if self.rotation != 0.0 {
                let rotate_angle = cgmath::Deg(self.rotation);
                let rotate_rotation = Matrix4::from_angle_z(rotate_angle);
                let origin = self.origin();
                Matrix4::from_translation(cgmath::vec3(origin.x * scaled_size.x, origin.y * scaled_size.y, 0.0)) *
                    rotate_rotation *
                    Matrix4::from_translation(cgmath::vec3(-origin.x * scaled_size.x, -origin.y * scaled_size.y, 0.0))
            } else {
                Matrix4::identity()
            };
            let scale = Matrix4::from_nonuniform_scale(scaled_size.x, scaled_size.y, 1.0);
            translate * rotate * scale
        };

        // Get texture coordinates.
        let tex_coords = self.texture_coordinates();

        let tex_top_left = tex_coords[0];
        let tex_top_right = tex_coords[1];
        let tex_bottom_left = tex_coords[2];
        let tex_bottom_right = tex_coords[3];

        // Flip texture coordinates if necessary.
        let (tex_top_left, tex_top_right, tex_bottom_left, tex_bottom_right) = match (self.flip_x, self.flip_y) {
            (false, false) => (tex_top_left, tex_top_right, tex_bottom_left, tex_bottom_right),
            (true, false) => (tex_top_right, tex_top_left, tex_bottom_right, tex_bottom_left),
            (false, true) => (tex_bottom_left, tex_bottom_right, tex_top_left, tex_top_right),
            (true, true) => (tex_bottom_right, tex_bottom_left, tex_top_right, tex_top_left),
        };

        // Transform vertices with the model matrix.
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

        let color = cgmath::conv::array3(self.color);

        [
            VertexData { pos: pos_top_left, tex_coords: tex_top_left, color: color },
            VertexData { pos: pos_top_right, tex_coords: tex_top_right, color: color },
            VertexData { pos: pos_bottom_left, tex_coords: tex_bottom_right, color: color },
            VertexData { pos: pos_bottom_right, tex_coords: tex_bottom_left, color: color },
        ]
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
