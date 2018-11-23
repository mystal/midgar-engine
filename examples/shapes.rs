extern crate midgar;
extern crate nalgebra_glm as glm;

use midgar::{
    App, Midgar, MidgarApp, MidgarAppConfig, Surface, KeyCode,
    graphics::shape::{DrawMode, ShapeRenderer},
};

pub struct GameApp {
    shape_renderer: ShapeRenderer,
    projection: glm::Mat4,

    rotation: f32,
    play: bool,

    time_to_fps: f32,
}

impl App for GameApp {
    fn new(midgar: &Midgar) -> Self {
        let (screen_width, screen_height) = midgar.graphics().screen_size();
        let projection = glm::ortho(0.0, screen_width as f32, 0.0, screen_height as f32, -1.0, 1.0);

        GameApp {
            shape_renderer: ShapeRenderer::new(midgar.graphics().display(), projection),
            projection,

            rotation: 0.0,
            play: false,

            time_to_fps: 1.0,
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

        self.shape_renderer.queue_rect(DrawMode::Fill, 100.0, 100.0, 50.0, 50.0, self.rotation, color);
        self.shape_renderer.queue_rect(DrawMode::Line(1.0), 200.0, 100.0, 50.0, 50.0, self.rotation / 2.0, color);
        self.shape_renderer.queue_circle(DrawMode::Fill, 300.0, 100.0, 25.0, color);
        self.shape_renderer.queue_circle(DrawMode::Line(1.0), 400.0, 100.0, 25.0, color);
        self.shape_renderer.draw_queued(midgar.graphics().display(), &mut target);

        target.finish()
            .unwrap();

        self.time_to_fps -= dt;
        if self.time_to_fps <= 0.0 {
            println!("FPS: {:.2}, Frame time: {:.2} ms", midgar.fps(), midgar.frame_time() * 1000.0);
            self.time_to_fps = 1.0;
        }
    }

    fn resize(&mut self, size: (u32, u32), _midgar: &Midgar) {
        println!("Resize: {:?}", size);
        self.projection = glm::ortho(0.0, size.0 as f32, 0.0, size.1 as f32, -1.0, 1.0);
        self.shape_renderer.set_projection_matrix(self.projection);
    }
}


fn main() {
    let config = MidgarAppConfig::new()
        .with_fps(240)
        .with_vsync(false);
    let app: MidgarApp<GameApp> = MidgarApp::new(config);
    app.run();
}
