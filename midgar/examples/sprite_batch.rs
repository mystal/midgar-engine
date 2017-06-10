extern crate cgmath;
extern crate midgar;

use std::rc::Rc;

use midgar::{App, Midgar, MidgarApp, MidgarAppConfig, Surface, KeyCode};
use midgar::graphics::sprite::{Sprite, SpriteDrawParams, SpriteRenderer};

// 10000 evenly spaced sprites, 100x100 grid.
const GRID: (usize, usize) = (100, 100);

pub struct GameApp<'a> {
    renderer: SpriteRenderer,
    sprite: Sprite<'a>,
    sprite_positions: Vec<(f32, f32)>,
    projection: cgmath::Matrix4<f32>,

    batch: bool,
    time_to_fps: f32,
}

impl<'a> App for GameApp<'a> {
    fn create(midgar: &Midgar) -> Self {
        let texture = midgar.graphics().load_texture("../sample/assets/awesomeface.png", true);
        let texture = Rc::new(texture);

        let mut sprite = Sprite::new(texture);
        sprite.set_uniform_scale(0.05);

        let (screen_width, screen_height) = midgar.graphics().screen_size();
        let projection = cgmath::ortho(0.0, screen_width as f32, 0.0, screen_height as f32, -1.0, 1.0);

        // Compute evenly spaced sprite positions.
        let horizontal_spacing = screen_width as f32 / GRID.0 as f32;
        let vertical_spacing = screen_height as f32 / GRID.1 as f32;

        let mut sprite_positions = Vec::new();
        for row in 0..GRID.1 {
            for col in 0..GRID.0 {
                sprite_positions.push((col as f32 * horizontal_spacing, row as f32 * vertical_spacing));
            }
        }

        GameApp {
            renderer: SpriteRenderer::new(midgar.graphics().display(), projection),
            sprite: sprite,
            sprite_positions: sprite_positions,
            projection: projection,
            batch: false,
            time_to_fps: 1.0,
        }
    }

    fn step(&mut self, midgar: &mut Midgar) {
        if midgar.input().was_key_pressed(KeyCode::Escape) {
            midgar.set_should_exit();
            return;
        }

        if midgar.input().was_key_pressed(KeyCode::Space) {
            self.batch = !self.batch;
            println!("Toggling batch rendering. New value: {}", self.batch);
        }

        let dt = midgar.time().delta_time() as f32;

        // TODO: Rotate the sprites.
        let rotate_speed = 45.0;
        //let old_rotation = self.sprite.rotation();
        //self.sprite.set_rotation(old_rotation + rotate_speed * dt);

        let mut target = midgar.graphics().display().draw();
        target.clear_color(0.1, 0.3, 0.4, 1.0);

        let draw_params = SpriteDrawParams {
            alpha_blending: true,
            .. Default::default()
        };

        if self.batch {
            let mut batch = self.renderer.begin_batch(draw_params, &mut target);
            for pos in &self.sprite_positions {
                self.sprite.set_position(cgmath::vec2(pos.0, pos.1));
                batch.draw(&self.sprite);
            }
            batch.finish().unwrap();
        } else {
            for pos in &self.sprite_positions {
                self.sprite.set_position(cgmath::vec2(pos.0, pos.1));
                self.renderer.draw(&self.sprite, draw_params, &mut target);
            }
        }

        target.finish()
            .expect("Swapping buffers failed");

        self.time_to_fps -= dt;
        if self.time_to_fps <= 0.0 {
            println!("FPS: {:.2}, Frame time: {:.2} ms", midgar.fps(), midgar.frame_time() * 1000.0);
            self.time_to_fps = 1.0;
        }
    }
}

fn main() {
    let config = MidgarAppConfig::new()
        .with_screen_size((1280, 720))
        .with_fps(240)
        .with_vsync(false);
    let app: MidgarApp<GameApp> = MidgarApp::new(config);
    app.run();
}
