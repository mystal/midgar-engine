use midgar::{App, Midgar, Surface};

pub struct GameApp; /*{
    //sprite_manager: SpriteManager,
    //world: World,
    //renderer: Renderer,
}*/

impl App for GameApp {
    fn create(midgar: &Midgar) -> Self {
        // NOTE: Framework takes care of setting up display for user.
        GameApp /*{
            //sprite_manager: SpriteManager::load(),
            //world: World::new(),
            //renderer: Renderer::new(),
        }*/
    }

    fn step(&mut self, midgar: &Midgar) {
        // NOTE: Framework takes care of tracking inputs for user.
        //world.update();

        let mut target = midgar.display.draw();
        target.clear_color(0.1, 0.3, 0.4, 1.0);
        //renderer.render(&world.renderables);
        target.finish().unwrap();
    }

    fn resize(&mut self, width: u32, height: u32, midgar: &Midgar) {
        //renderer.resize(width, height);
        println!("Resize: {}, {}", width, height);
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
