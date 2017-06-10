use std::rc::Rc;

use cgmath;
use midgar::{App, Midgar, Surface, KeyCode};
use midgar::graphics::sprite::{Sprite, SpriteDrawParams, SpriteRenderer};


pub struct GameApp<'a> {
    sprite_renderer: SpriteRenderer,
    sprite: Sprite<'a>,
    projection: cgmath::Matrix4<f32>,

    play: bool,
}

impl<'a> App for GameApp<'a> {
    fn create(midgar: &Midgar) -> Self {
        let texture = midgar.graphics().load_texture("assets/awesomeface.png", true);
        let texture = Rc::new(texture);
        let mut sprite = Sprite::new(texture);
        sprite.set_position(cgmath::vec2(200.0, 200.0));
        sprite.set_color(cgmath::vec3(0.0, 1.0, 0.0));
        sprite.set_origin(cgmath::vec2(0.0, 0.0));
        //sprite.set_flip_x(true);
        sprite.set_flip_y(true);

        let (screen_width, screen_height) = midgar.graphics().screen_size();
        let projection = cgmath::ortho(0.0, screen_width as f32, 0.0, screen_height as f32, -1.0, 1.0);

        GameApp {
            sprite_renderer: SpriteRenderer::new(midgar.graphics().display(), projection),
            sprite: sprite,
            projection: projection,
            play: false,
        }
    }

    fn step(&mut self, midgar: &mut Midgar) {
        if midgar.input().was_key_pressed(KeyCode::Escape) {
            midgar.set_should_exit();
            return;
        }

        if midgar.input().was_key_pressed(KeyCode::Space) {
            self.play = !self.play;
        }

        let dt = midgar.time().delta_time() as f32;

        if self.play {
            let rotate_speed = 45.0;
            let old_rotation = self.sprite.rotation();
            self.sprite.set_rotation(old_rotation + rotate_speed * dt);
        }

        let draw_params = SpriteDrawParams {
            alpha_blending: true,
            .. Default::default()
        };

        // TODO: Have draw be called on graphics
        let mut target = midgar.graphics().display().draw();
        target.clear_color(0.1, 0.3, 0.4, 1.0);
        self.sprite_renderer.draw(&self.sprite, draw_params, &mut target);
        target.finish().unwrap();
    }

    fn resize(&mut self, size: (u32, u32), midgar: &Midgar) {
        println!("Resize: {:?}", size);
        self.projection = cgmath::ortho(0.0, size.0 as f32, 0.0, size.1 as f32, -1.0, 1.0);
        self.sprite_renderer.set_projection_matrix(self.projection);
    }

    fn pause(&mut self, midgar: &Midgar) {
        println!("Pause");
    }

    fn resume(&mut self, midgar: &Midgar) {
        println!("Resume");
    }

    fn destroy(&mut self, midgar: &Midgar) {
    }
}
