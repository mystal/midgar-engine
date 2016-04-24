pub struct MidgarAppConfig {
    fps: u8,
}

impl MidgarAppConfig {
    pub fn new() -> Self {
        MidgarAppConfig {
            fps: 60,
        }
    }

    pub fn fps(&self) -> u8 {
        self.fps
    }
}
