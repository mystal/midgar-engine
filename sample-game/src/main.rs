extern crate midgar;

use midgar::{MidgarApp, MidgarAppConfig};

mod game_app;

fn main() {
    println!("Hello, world!");

    // TODO: Consider using a builder.
    let config = MidgarAppConfig::new();
    // TODO: Any need to actually return an app? Just run the config? Maybe run and return a
    // handle?
    let app: MidgarApp<game_app::GameApp> = MidgarApp::new(config);
    app.run();
}
