use midgar::{App, Midgar};

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
        //renderer.render(&world.renderables);
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
