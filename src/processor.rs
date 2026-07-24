pub trait Processor {
    fn process(&self, sample: i32) -> i32;

}   