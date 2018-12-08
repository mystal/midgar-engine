use std::rc::Rc;

use midgar::{App, Midgar, MidgarApp, MidgarAppConfig, Surface, KeyCode};
use midgar::graphics::sprite::{Sprite, SpriteDrawParams, SpriteRenderer};
use midgar::graphics::text::{Scale, Section, TextRenderer};

pub struct GameApp<'a> {
    sprite_renderer: SpriteRenderer,
    text_renderer: TextRenderer<'a>,
    sprite: Sprite<'a>,
    projection: glm::Mat4,
    text_projection: glm::Mat4,

    play: bool,
}

impl<'a> App for GameApp<'a> {
    fn new(midgar: &Midgar) -> Self {
        let texture = midgar.graphics().load_texture("assets/awesomeface.png", true);
        let texture = Rc::new(texture);
        let mut sprite = Sprite::new(texture);
        sprite.set_position(200.0, 200.0);
        sprite.set_color([0.0, 1.0, 0.0, 0.2]);
        sprite.set_origin(0.0, 0.0);
        //sprite.set_flip_x(true);
        sprite.set_flip_y(true);

        let (screen_width, screen_height) = midgar.graphics().screen_size();
        let projection = glm::ortho(0.0, screen_width as f32, 0.0, screen_height as f32, -1.0, 1.0);
        let text_projection = glm::ortho(0.0, screen_width as f32, screen_height as f32, 0.0, -1.0, 1.0);

        GameApp {
            sprite_renderer: SpriteRenderer::new(midgar.graphics().display(), projection),
            text_renderer: TextRenderer::new(midgar.graphics().display()),
            sprite: sprite,
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
        self.text_renderer.queue(Section {
            text: "Testing!\n1, 2, 3, testing!",
            color: [1.0, 1.0, 1.0, 1.0],
            screen_position: (100.0, 100.0),
            scale: Scale::uniform(20.0),
            .. Section::default()
        });
        self.text_renderer.draw_queued(midgar.graphics().display(), &mut target);
        target.finish()
            .unwrap();
    }

    fn resize(&mut self, size: (u32, u32), _midgar: &Midgar) {
        println!("Resize: {:?}", size);
        self.projection = glm::ortho(0.0, size.0 as f32, 0.0, size.1 as f32, -1.0, 1.0);
        self.text_projection = glm::ortho(0.0, size.0 as f32, 0.0, size.1 as f32, -1.0, 1.0);
        self.sprite_renderer.set_projection_matrix(self.projection);
    }

    fn pause(&mut self, _midgar: &Midgar) {
        println!("Pause");
    }

    fn resume(&mut self, _midgar: &Midgar) {
        println!("Resume");
    }

    fn destroy(&mut self, _midgar: &Midgar) {
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
