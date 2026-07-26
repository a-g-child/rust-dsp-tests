pub trait Processor {
    fn process(&mut self, sample: i32) -> i32;

    fn process_buffer(&mut self, buffer: &mut [i32]) {
        for sample in buffer {
            *sample = self.process(*sample);
            // println!("processing...")
        }
    }

}   