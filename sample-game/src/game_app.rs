use cgmath;
use glium;
use image;
use midgar::{App, Midgar, Surface, VirtualKeyCode};
use midgar::sprite::{Sprite, SpriteRenderer};

pub struct GameApp {
    sprite_renderer: SpriteRenderer,
    sprite: Sprite,
    projection: cgmath::Matrix4<f32>,
    //camera: OrthographicCamera,
    //sprite_manager: SpriteManager,
    //world: World,
    //renderer: Renderer,
}

impl App for GameApp {
    fn create(midgar: &Midgar) -> Self {
        // TODO: Have midgar provide image loading
        //let texture = midgar.graphics.load_texture("assets/awesomeface.png").unwrap();
        let image = image::open("assets/awesomeface.png").unwrap().to_rgba();
        let image_dimensions = image.dimensions();
        println!("Image dimensions: {:?}", image_dimensions);
        let image = glium::texture::RawImage2d::from_raw_rgba_reversed(image.into_raw(), image_dimensions);
        let texture = glium::Texture2d::new(&midgar.graphics().display, image).unwrap();
        let mut sprite = Sprite::new(texture);
        sprite.set_position(cgmath::vec2(200.0, 200.0));
        sprite.set_color(cgmath::vec3(0.0, 1.0, 0.0));

        let (screen_width, screen_height) = midgar.graphics().display.get_framebuffer_dimensions();
        let projection = cgmath::ortho(0.0, screen_width as f32, 0.0, screen_height as f32, -1.0, 1.0);

        GameApp {
            sprite_renderer: SpriteRenderer::new(&midgar.graphics().display),
            sprite: sprite,
            projection: projection,
            //sprite_manager: SpriteManager::load(),
            //world: World::new(),
            //renderer: Renderer::new(),
        }
    }

    fn step(&mut self, midgar: &mut Midgar) {
        if midgar.input().was_key_pressed(&VirtualKeyCode::Escape) {
            midgar.set_should_exit();
            return;
        }

        // NOTE: Framework takes care of tracking inputs for user.
        //world.update();

        // TODO: Have draw be called on graphics
        let mut target = midgar.graphics().display.draw();
        target.clear_color(0.1, 0.3, 0.4, 1.0);
        self.sprite_renderer.draw_sprite(&self.sprite, &self.projection, &mut target);
        //renderer.render(&world.renderables);
        target.finish().unwrap();
    }

    fn resize(&mut self, width: u32, height: u32, midgar: &Midgar) {
        //renderer.resize(width, height);
        println!("Resize: {}, {}", width, height);
        self.projection = cgmath::ortho(0.0, width as f32, 0.0, height as f32, -1.0, 1.0);
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
