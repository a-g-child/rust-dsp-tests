use rand::RngExt;
use crate::processor::Processor;
use crate::utility::mixer;

#[derive(Clone)]
struct ReverbLine {
    buffer: Vec<f64>,
    position: usize,
}

impl ReverbLine {
    fn new(size: usize, spread: usize) -> Self {
        let mut rng = rand::rng();
        let min_size = size.saturating_sub(spread).max(1);
        let max_size = size.saturating_add(spread).max(min_size);
        let random_size = rng.random_range(min_size..=max_size);
        println!("comb filter size: {}", random_size);

        Self {
            buffer: vec![0.0; random_size],
            position: 0,
        }
    }
    fn read(&self) -> f64 {
        self.buffer[self.position]
    }
    fn write_and_advance(&mut self, sample: f64) {
        self.buffer[self.position] = sample;

        self.position += 1;

        if self.position == self.buffer.len() {
            self.position = 0;
        }
    }
}

pub struct Reverb {
    lines: [[ReverbLine; 10]; 2],
    decay: [f64; 2],
    mix: [f64; 2],
    bit_depth: f64,
}

impl Reverb {
    pub fn new(
        size: f64,
        spread: f64, 
        decay: [f64; 2], 
        mix: [f64; 2],
        bit_depth: f64,) -> Self {

            let size = size.max(1.0) as usize;
            let spread = spread.max(0.0) as usize;
            let lines = std::array::from_fn(|_| {
                std::array::from_fn(|_| ReverbLine::new(size, spread))
            });

            Self {
                lines,
                decay,
                mix,
                bit_depth,
            }
    }
    pub fn process_channel(&mut self, channel: usize, sample: i32) -> i32{

        let mut output: f64 = sample as f64;

        for line in self.lines[channel].iter_mut() {  
            let reverb = line.read(); 
            output = reverb + output * 0.5;
                // mixer(reverb as i32, output, self.mix[channel]);

            line.write_and_advance(sample as f64 * self.decay[channel]);

        }
        
        let mix = mixer(output as i32 , sample, self.mix[channel]);
        // println!("output: {}, input: {}, mix: {}", output, sample, mix);
        mix
    }

}

impl Processor for Reverb {
    fn process(&mut self, sample: i32) -> i32 {
        self.process_channel(0, sample)
    }
    fn process_buffer(&mut self, buffer: &mut [i32]) {
        for frame in buffer.chunks_exact_mut(2) {

            let left = frame[0];
            let right = frame[1];
            let l = self.process_channel(0, left);
            let r = self.process_channel(1, right);
            frame[0] = l;
            frame[1] = r;
        }
    }
}

