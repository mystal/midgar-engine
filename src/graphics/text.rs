use std::borrow::Cow;
use std::fs::File;
use std::io::Read;

use cgmath::{self, Matrix4};
use cgmath::prelude::*;
use glium::{self, Surface, Texture2d, uniform};
pub use rusttype::Font;
use rusttype::{FontCollection, PositionedGlyph, Scale, ScaledGlyph, point};
use rusttype::gpu_cache::{Cache, CacheBuilder};

//use super::texture_region::{TextureRegion, TextureRegionHolder};


const VERTEX_SHADER_SRC: &'static str = include_str!("shaders/text.vs.glsl");
const FRAGMENT_SHADER_SRC: &'static str = include_str!("shaders/text.fs.glsl");

// TODO: Change this to 4 to use indexed drawing.
const QUAD_SIZE: usize = 6;
const NUM_VERTS: usize = QUAD_SIZE * 512;


#[derive(Clone, Copy)]
struct Vertex {
    vertex: [f32; 4],
}

glium::implement_vertex!(Vertex, vertex);


pub struct TextRenderer<'cache> {
    // TODO: Store projection matrix here?
    shader: glium::Program,
    vertex_buffer: glium::VertexBuffer<Vertex>,
    //index_buffer: glium::VertexBuffer<Vertex>,
    glyph_cache: Cache<'cache>,
    glyph_cache_tex: Texture2d,
}

impl<'cache> TextRenderer<'cache> {
    pub fn new<F: glium::backend::Facade>(display: &F) -> Self {
        // TODO: Store the DPI from SDL2?

        let dpi_factor = 1;
        // TODO: Make these tunable.
        let (cache_width, cache_height) = (512 * dpi_factor, 512 * dpi_factor);
        let glyph_cache = CacheBuilder {
            width: cache_width,
            height: cache_height,
            .. CacheBuilder::default()
        }.build();

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
        ).unwrap();

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

        let vertex_buffer = glium::VertexBuffer::empty_dynamic(display, QUAD_SIZE).unwrap();
        //let index_buffer = glium::IndexBuffer::empty_dynamic(display, NUM_VERTS).unwrap();

        TextRenderer {
            shader: shader,
            vertex_buffer: vertex_buffer,
            //index_buffer: index_buffer,
            glyph_cache: glyph_cache,
            glyph_cache_tex: glyph_cache_tex,
        }
    }

    // TODO: I want to pass the font by ref, but tend to have borrow errors in calling code...
    // TODO: Maybe it's worth just using gfx_glyph.
    pub fn draw_text<S>(&mut self, text: &str, font: Font<'cache>, color: [f32; 3],
                        size: u8, x: f32, y: f32, width: u32,
                        projection: &Matrix4<f32>, target: &mut S)
        where S: Surface {
        // TODO: Correctly get the dpi_factor for High DPI displays.
        //let dpi_factor = {
        //    let window = display.get_window().unwrap();
        //    (window.hidpi_factor())
        //};
        let dpi_factor = 1;

        // Get glyph positions.
        let glyphs = layout_paragraph(font, Scale::uniform((size * dpi_factor) as f32), width, text);

        // Queue up glyphs for caching on the GPU.
        for glyph in &glyphs {
            // TODO: Replace the 0 with a unique font ID for the given font.
            self.glyph_cache.queue_glyph(0, glyph.clone());
        }

        // TODO: Move cache_queued call to a flush method for text batching.
        // Upload glyph cache to the GPU.
        {
            let glyph_cache = &mut self.glyph_cache;
            let glyph_cache_tex = &mut self.glyph_cache_tex;
            glyph_cache.cache_queued(|rect, data| {
                glyph_cache_tex.main_level().write(glium::Rect {
                    left: rect.min.x,
                    bottom: rect.min.y,
                    width: rect.width(),
                    height: rect.height(),
                }, glium::texture::RawImage2d {
                    data: Cow::Borrowed(data),
                    width: rect.width(),
                    height: rect.height(),
                    format: glium::texture::ClientFormat::U8,
                });
            }).expect("Failed to upload glyph cache to GPU texture.");
        }

        // Batch up the glyph quads for drawing.
        for glyph in &glyphs {
            // TODO: Replace the 0 with a unique font ID for the given font.
            let (vertices, model) = if let Ok(Some((uv_rect, screen_rect))) = self.glyph_cache.rect_for(0, glyph) {
                let model = {
                    let position = cgmath::vec2(x + screen_rect.min.x as f32,
                                                y + screen_rect.min.y as f32);
                    let translate = Matrix4::from_translation(position.extend(0.0));

                    // TODO: Support rotations.
                    //let rotate_angle = cgmath::Deg(rotation);
                    //let rotate_rotation = Matrix4::from_angle_z(rotate_angle);
                    // FIXME: Rotate around the center of the positioned text.
                    //let rotate =
                    //    Matrix4::from_translation(cgmath::vec3(0.5 * scaled_size.x, 0.5 * scaled_size.y, 0.0)) *
                    //    rotate_rotation *
                    //    Matrix4::from_translation(cgmath::vec3(-0.5 * scaled_size.x, -0.5 * scaled_size.y, 0.0));
                    let rotate = Matrix4::identity();

                    let scale = Matrix4::from_nonuniform_scale(screen_rect.width() as f32, screen_rect.height() as f32, 1.0);

                    translate * rotate * scale
                };

                // TODO: Verify that these match how the GPU cache returns coordinates.
                let top_left = [uv_rect.min.x, uv_rect.max.y];
                let top_right = [uv_rect.max.x, uv_rect.max.y];
                let bottom_left = [uv_rect.min.x, uv_rect.min.y];
                let bottom_right = [uv_rect.max.x, uv_rect.min.y];

                //let normalized_width = width / region.size().x as f32;
                //let normalized_height = height / region.size().y as f32;
                let normalized_width = 1.0;
                let normalized_height = 1.0;

                let vertices = [
                    Vertex { vertex: [0.0, normalized_height, top_left[0], top_left[1]] },
                    Vertex { vertex: [normalized_width, 0.0, bottom_right[0], bottom_right[1]] },
                    Vertex { vertex: [0.0, 0.0, bottom_left[0], bottom_left[1]] },
                    Vertex { vertex: [0.0, normalized_height, top_left[0], top_left[1]] },
                    Vertex { vertex: [normalized_width, normalized_height, top_right[0], top_right[1]] },
                    Vertex { vertex: [normalized_width, 0.0, bottom_right[0], bottom_right[1]] },
                ];

                (vertices, model)
            } else {
                continue;
            };

            // NOTE: For batched rendering, you can allocate a big vertex buffer at the start and copy
            // vertex data each time you go to draw. So copy the data to the vertex buffer here!
            self.vertex_buffer.write(&vertices);

            let index_buffer = glium::index::NoIndices(glium::index::PrimitiveType::TrianglesList);

            // TODO: Set any options on the sampler?
            let texture = self.glyph_cache_tex.sampled()
                .magnify_filter(glium::uniforms::MagnifySamplerFilter::Nearest);

            let uniforms = uniform! {
                tex: texture,
                textColor: color,
                model: cgmath::conv::array4x4(model),
                view: cgmath::conv::array4x4(Matrix4::<f32>::identity()),
                projection: cgmath::conv::array4x4(*projection),
            };

            // Set alpha blending from sprite.
            //let blend = if region.alpha() {
            //    glium::Blend::alpha_blending()
            //} else {
            //    Default::default()
            //};
            let blend = glium::Blend::alpha_blending();
            let params = glium::DrawParameters {
                blend: blend,
                .. Default::default()
            };

            target.draw(&self.vertex_buffer, &index_buffer, &self.shader, &uniforms, &params).unwrap();
        }

        // TODO: Batch up drawing text!

        // FIXME: We want to use indexed vertices to pass in 4 vertices instead of 6.
        //let indices = [0, 1, 2, 0, 2, 3];
    }
}

pub fn load_font_from_path<'font>(font_path: &str) -> Font<'font> {
    // TODO: Handle errors! Return a Result!
    let font_bytes = {
        let mut font_bytes = Vec::new();
        let mut font_file = File::open(font_path)
            .expect("Could not load font file.");
        font_file.read_to_end(&mut font_bytes)
            .expect("Could not read font file.");

        font_bytes
    };
    FontCollection::from_bytes(font_bytes)
        .expect("Could not load font file")
        .into_font()
        .expect("Could not turn FontCollection into a Font")
}

struct GlyphPos<'a> {
    pub scaled_glyph: ScaledGlyph<'a>,
    pub advance_width: i16,
}

// Based on the rusttype gpu_cache example and Pathfinder's shaper::shape_text method:
// * https://github.com/dylanede/rusttype/blob/master/examples/gpu_cache.rs
// * https://github.com/pcwalton/pathfinder/blob/master/src/shaper.rs
fn layout_paragraph<'font>(font: Font<'font>,
                           scale: Scale,
                           width: u32,
                           text: &str) -> Vec<PositionedGlyph<'font>> {
    // TODO: We should probably support use this.
    //use unicode_normalization::UnicodeNormalization;

    let space_advance = font.glyph(' ').scaled(scale).h_metrics().advance_width;

    let mut glyphs = Vec::new();
    let v_metrics = font.v_metrics(scale);
    let advance_height = v_metrics.ascent - v_metrics.descent + v_metrics.line_gap;
    let mut caret = point(0.0, v_metrics.ascent);

    // Iterate over each word in each line and wrap words to fit in width.
    for line in text.lines() {
        for word in line.split_whitespace() {
            // Compute the scaled glyph positions.
            let scaled_glyph_positions = {
                // NOTE: If using unicode normalization, replace .chars() with .nfc()
                let mut chars = word.chars().peekable();
                let mut next_glyph = None;
                let mut glyph_positions = Vec::new();

                while let Some(ch) = chars.next() {
                    let glyph = match next_glyph.take() {
                        None => font.glyph(ch).scaled(scale),
                        Some(next_glyph) => next_glyph,
                    };

                    let mut advance = glyph.h_metrics().advance_width;

                    if let Some(&next_char) = chars.peek() {
                        let next = font.glyph(next_char).scaled(scale);
                        let next_id = next.id();
                        next_glyph = Some(next);
                        advance += font.pair_kerning(scale, glyph.id(), next_id)
                    }

                    glyph_positions.push(GlyphPos {
                        scaled_glyph: glyph,
                        advance_width: advance as i16,
                    })
                }

                glyph_positions
            };

            // Check if the glyphs fit in this line's remaining space.
            let total_advance: u32 = scaled_glyph_positions.iter()
                .map(|p: &GlyphPos| p.advance_width as u32)
                .sum();
            // Only push the word down if it's not the first word in this line.
            if caret.x != 0.0 && caret.x as u32 + total_advance > width {
                caret = point(0.0, caret.y + advance_height);
            }

            // Get the final positioned glyphs.
            for glyph_pos in scaled_glyph_positions {
                let advance_width = glyph_pos.scaled_glyph.h_metrics().advance_width;
                glyphs.push(glyph_pos.scaled_glyph.positioned(caret));
                caret.x += advance_width;
            }

            caret.x += space_advance;
        }

        caret = point(0.0, caret.y + advance_height);
    }

    glyphs
}
