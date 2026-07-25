pub trait Processor {
    fn process(&mut self, sample: i32) -> i32;

}   