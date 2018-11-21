use Midgar;

pub trait App {
    fn new(midgar: &Midgar) -> Self;

    #[allow(unused_variables)]
    fn step(&mut self, midgar: &mut Midgar) {}

    #[allow(unused_variables)]
    fn resize(&mut self, size: (u32, u32), midgar: &Midgar) {}

    #[allow(unused_variables)]
    fn pause(&mut self, midgar: &Midgar) {}

    #[allow(unused_variables)]
    fn resume(&mut self, midgar: &Midgar) {}

    #[allow(unused_variables)]
    fn destroy(&mut self, midgar: &Midgar) {}
}
