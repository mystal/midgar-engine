use Midgar;


pub trait App {
    fn create(midgar: &Midgar) -> Self;
    fn step(&mut self, midgar: &mut Midgar) {}
    fn resize(&mut self, width: u32, height: u32, midgar: &Midgar) {}
    fn pause(&mut self, midgar: &Midgar) {}
    fn resume(&mut self, midgar: &Midgar) {}
    fn destroy(&mut self, midgar: &Midgar) {}
}
