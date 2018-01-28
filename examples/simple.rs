extern crate cgmath;
extern crate midgar;

use std::rc::Rc;

use midgar::{App, Midgar, MidgarApp, MidgarAppConfig, Surface, KeyCode};
use midgar::graphics::sprite::{Sprite, SpriteDrawParams, SpriteRenderer};
use midgar::graphics::text::{self, Font, TextRenderer};

pub struct GameApp<'a> {
    sprite_renderer: SpriteRenderer,
    text_renderer: TextRenderer,
    sprite: Sprite<'a>,
    font: Font<'a>,
    projection: cgmath::Matrix4<f32>,
    text_projection: cgmath::Matrix4<f32>,

    play: bool,
}

impl<'a> App for GameApp<'a> {
    fn create(midgar: &Midgar) -> Self {
        let texture = midgar.graphics().load_texture("assets/awesomeface.png", true);
        let texture = Rc::new(texture);
        let mut sprite = Sprite::new(texture);
        sprite.set_position(cgmath::vec2(200.0, 200.0));
        sprite.set_color(cgmath::vec4(0.0, 1.0, 0.0, 0.2));
        sprite.set_origin(cgmath::vec2(0.0, 0.0));
        //sprite.set_flip_x(true);
        sprite.set_flip_y(true);

        let (screen_width, screen_height) = midgar.graphics().screen_size();
        let projection = cgmath::ortho(0.0, screen_width as f32, 0.0, screen_height as f32, -1.0, 1.0);
        let text_projection = cgmath::ortho(0.0, screen_width as f32, screen_height as f32, 0.0, -1.0, 1.0);

        GameApp {
            sprite_renderer: SpriteRenderer::new(midgar.graphics().display(), projection),
            text_renderer: TextRenderer::new(midgar.graphics().display()),
            sprite: sprite,
            font: text::load_font_from_path("assets/VeraMono.ttf"),
            projection,
            text_projection,
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
        // TODO: Fix text rendering so it doesn't require a separate projection matrix?
        self.text_renderer.draw_text("Testing!\n1, 2, 3, testing!", &self.font, [1.0, 1.0, 1.0],
                                     20, 100.0, 100.0, 300, &self.text_projection, &mut target);
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


fn main() {
    // TODO: Consider using a builder.
    let config = MidgarAppConfig::new();
    // TODO: Any need to actually return an app? Just run the config? Maybe run and return a
    // handle?
    let app: MidgarApp<GameApp> = MidgarApp::new(config);
    app.run();
}
