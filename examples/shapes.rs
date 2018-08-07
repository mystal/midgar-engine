extern crate cgmath;
extern crate midgar;

use std::rc::Rc;

use midgar::{App, Midgar, MidgarApp, MidgarAppConfig, Surface, KeyCode};
use midgar::graphics::shape::ShapeRenderer;

pub struct GameApp {
    shape_renderer: ShapeRenderer,
    projection: cgmath::Matrix4<f32>,

    rotation: f32,
    play: bool,
}

impl App for GameApp {
    fn create(midgar: &Midgar) -> Self {
        let (screen_width, screen_height) = midgar.graphics().screen_size();
        let projection = cgmath::ortho(0.0, screen_width as f32, 0.0, screen_height as f32, -1.0, 1.0);

        GameApp {
            shape_renderer: ShapeRenderer::new(midgar.graphics().display(), projection),
            projection,

            rotation: 0.0,
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
            self.rotation += rotate_speed * dt;
        }

        let mut target = midgar.graphics().display().draw();
        target.clear_color(0.1, 0.3, 0.4, 1.0);
        let color = [1.0, 0.0, 0.0, 1.0];
        self.shape_renderer.draw_filled_rect(100.0, 100.0, 50.0, 50.0, self.rotation, color, &mut target);
        target.finish()
            .unwrap();
    }

    fn resize(&mut self, size: (u32, u32), midgar: &Midgar) {
        println!("Resize: {:?}", size);
        self.projection = cgmath::ortho(0.0, size.0 as f32, 0.0, size.1 as f32, -1.0, 1.0);
        self.shape_renderer.set_projection_matrix(self.projection);
    }
}


fn main() {
    let config = MidgarAppConfig::new();
    let app: MidgarApp<GameApp> = MidgarApp::new(config);
    app.run();
}
