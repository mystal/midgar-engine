use std::borrow::Cow;
use std::ops::Deref;

use glium::{Surface, Texture2d, uniform};
use glium::vertex::EmptyVertexAttributes;
pub use glyph_brush::{
    rusttype::Scale,
    FontId, Section, VariedSection,
};
use glyph_brush::{BrushAction, BrushError, GlyphBrush, GlyphBrushBuilder};

const VERTEX_SHADER_SRC: &str = include_str!("shaders/text.vs.glsl");
const FRAGMENT_SHADER_SRC: &str = include_str!("shaders/text.fs.glsl");

const DEFAULT_FONT_BYTES: &[u8] = include_bytes!("../../assets/VeraMono.ttf");

#[derive(Clone, Copy, Debug)]
struct GlyphVertex {
    // Screen coordinates.
    left_top: [f32; 3],
    right_bottom: [f32; 2],
    // Texture coordinates.
    tex_left_top: [f32; 2],
    tex_right_bottom: [f32; 2],
    // Text color.
    color: [f32; 4],
}

glium::implement_vertex!(GlyphVertex, left_top, right_bottom, tex_left_top, tex_right_bottom, color);

pub struct TextRenderer<'font> {
    shader: glium::Program,
    vertex_buffer: Option<glium::VertexBuffer<GlyphVertex>>,
    glyph_brush: GlyphBrush<'font, GlyphVertex>,
    glyph_cache_tex: Texture2d,
}

impl<'font> TextRenderer<'font> {
    pub fn new<F: glium::backend::Facade>(display: &F) -> Self {
        // TODO: Store the DPI from SDL2?
        let _dpi_factor = 1;

        // TODO: Make glyph brush configurable.
        let glyph_brush = GlyphBrushBuilder::using_font_bytes(DEFAULT_FONT_BYTES)
            .build();
        let (cache_width, cache_height) = glyph_brush.texture_dimensions();

        let glyph_cache_tex = glium::texture::Texture2d::with_format(
            display,
            glium::texture::RawImage2d {
                data: Cow::Owned(vec![128u8; cache_width as usize * cache_height as usize]),
                width: cache_width,
                height: cache_height,
                format: glium::texture::ClientFormat::U8,
            },
            glium::texture::UncompressedFloatFormat::U8,
            glium::texture::MipmapsOption::NoMipmap,
        ).expect("Could not create glyph cache texture.");

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
            .expect("Could not create TextRenderer shader program.");

        TextRenderer {
            shader,
            vertex_buffer: None,
            glyph_brush,
            glyph_cache_tex,
        }
    }

    pub fn queue<'a, S>(&mut self, section: S)
    where
        S: Into<Cow<'a, VariedSection<'a>>>
    {
        self.glyph_brush.queue(section);
    }

    pub fn draw_queued<F, S>(&mut self, display: &F, target: &mut S)
    where
        F: glium::backend::Facade + Deref<Target = glium::backend::Context>,
        S: Surface,
    {
        let (screen_width, screen_height) = display.get_framebuffer_dimensions();
        let transform = glm::ortho(0.0, screen_width as f32, screen_height as f32, 0.0, -1.0, 1.0);
        self.draw_queued_with_transform(&transform, display, target);
    }

    pub fn draw_queued_with_transform<F, S>(&mut self, transform: &glm::Mat4, display: &F, target: &mut S)
    where
        F: glium::backend::Facade,
        S: Surface,
    {
        let brush_action = loop {
            let brush_result = {
                let glyph_brush = &mut self.glyph_brush;
                let glyph_cache_tex = &mut self.glyph_cache_tex;

                glyph_brush.process_queued(
                    |rect, tex_data| {
                        glyph_cache_tex.write(glium::Rect {
                            left: rect.min.x,
                            bottom: rect.min.y,
                            width: rect.width(),
                            height: rect.height(),
                        }, glium::texture::RawImage2d {
                            data: Cow::Borrowed(tex_data),
                            width: rect.width(),
                            height: rect.height(),
                            format: glium::texture::ClientFormat::U8,
                        });
                    },
                    to_vertex,
                )
            };

            match brush_result {
                Ok(action) => break action,
                Err(BrushError::TextureTooSmall { suggested }) => {
                    let (cache_width, cache_height) = suggested;
                    let glyph_cache_tex = glium::texture::Texture2d::with_format(
                        display,
                        glium::texture::RawImage2d {
                            data: Cow::Owned(vec![128u8; cache_width as usize * cache_height as usize]),
                            width: cache_width,
                            height: cache_height,
                            format: glium::texture::ClientFormat::U8,
                        },
                        glium::texture::UncompressedFloatFormat::U8,
                        glium::texture::MipmapsOption::NoMipmap,
                    ).expect("Could not resize glyph cache texture.");
                    self.glyph_brush.resize_texture(cache_width, cache_height);
                    self.glyph_cache_tex = glyph_cache_tex;
                },
            }
        };

        match brush_action {
            BrushAction::Draw(vertices) => {
                self.vertex_buffer = Some(glium::VertexBuffer::immutable(display, &vertices)
                    .expect("Could not create TextRenderer vertex buffer."));
            }
            BrushAction::ReDraw => {}
        }

        if let Some(ref vertex_buffer) = self.vertex_buffer {
            let indices = glium::index::NoIndices(glium::index::PrimitiveType::TriangleStrip);

            let sampler = self.glyph_cache_tex.sampled()
                .magnify_filter(glium::uniforms::MagnifySamplerFilter::Nearest);

            let uniforms = uniform! {
                font_tex: sampler,
                transform: *transform.as_ref(),
            };
            let params = glium::DrawParameters {
                blend: glium::Blend::alpha_blending(),
                .. Default::default()
            };
            target.draw((EmptyVertexAttributes { len: 4 }, vertex_buffer.per_instance().unwrap()), indices, &self.shader, &uniforms, &params)
                .expect("Failed to draw text.");
        }
    }
}

#[inline]
fn to_vertex(
    glyph_brush::GlyphVertex {
        mut tex_coords,
        pixel_coords,
        bounds,
        color,
        z,
    }: glyph_brush::GlyphVertex,
) -> GlyphVertex {
    use glyph_brush::rusttype::{Rect, point};

    let gl_bounds = bounds;

    let mut gl_rect = Rect {
        min: point(pixel_coords.min.x as f32, pixel_coords.min.y as f32),
        max: point(pixel_coords.max.x as f32, pixel_coords.max.y as f32),
    };

    // handle overlapping bounds, modify uv_rect to preserve texture aspect
    if gl_rect.max.x > gl_bounds.max.x {
        let old_width = gl_rect.width();
        gl_rect.max.x = gl_bounds.max.x;
        tex_coords.max.x = tex_coords.min.x + tex_coords.width() * gl_rect.width() / old_width;
    }
    if gl_rect.min.x < gl_bounds.min.x {
        let old_width = gl_rect.width();
        gl_rect.min.x = gl_bounds.min.x;
        tex_coords.min.x = tex_coords.max.x - tex_coords.width() * gl_rect.width() / old_width;
    }
    if gl_rect.max.y > gl_bounds.max.y {
        let old_height = gl_rect.height();
        gl_rect.max.y = gl_bounds.max.y;
        tex_coords.max.y = tex_coords.min.y + tex_coords.height() * gl_rect.height() / old_height;
    }
    if gl_rect.min.y < gl_bounds.min.y {
        let old_height = gl_rect.height();
        gl_rect.min.y = gl_bounds.min.y;
        tex_coords.min.y = tex_coords.max.y - tex_coords.height() * gl_rect.height() / old_height;
    }

    GlyphVertex {
        left_top: [gl_rect.min.x, gl_rect.max.y, z],
        right_bottom: [gl_rect.max.x, gl_rect.min.y],
        tex_left_top: [tex_coords.min.x, tex_coords.max.y],
        tex_right_bottom: [tex_coords.max.x, tex_coords.min.y],
        color,
    }
}
