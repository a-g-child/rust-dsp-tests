use crate::{processor::Processor, sample_range::SampleRange, utility::mixer, utility::clamp_to_bit_depth};

struct DelayLine {
    buffer: Vec<f64>,
    position: usize,
}

impl DelayLine {
    fn new(delay_samples: usize) -> Self {
        Self {
            buffer: vec![0.0; delay_samples.max(1)],
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

pub struct Delay {
    lines: [DelayLine; 2],
    // decay: [f64; 2],
    feedback: [f64; 2],
    mix: [f64; 2],
    bit_depth: SampleRange,
}

impl Delay {
    pub fn new(time_ms: [f64; 2], feedback: [f64; 2], mix: [f64; 2], sample_rate: f64, bit_depth: SampleRange) -> Self{
        Self { 
            lines: [
                DelayLine::new((sample_rate * time_ms[0] / 1000.0) as usize), 
                DelayLine::new((sample_rate * time_ms[1] / 1000.0)as usize)], 
            feedback, 
            // decay,
            mix,
            bit_depth, }
    }

    fn process_channel(&mut self, channel: usize, input: i32,) -> i32 {

        let inp: f64 = input as f64;

        let delayed = self.lines[channel].read();

        let output = inp + delayed * self.mix[channel];

        let feedback_sample = 
            clamp_to_bit_depth(inp + delayed * self.feedback[channel], self.bit_depth);

        self.lines[channel].write_and_advance(feedback_sample);

        clamp_to_bit_depth(output, self.bit_depth) as i32
    }
}

impl Processor for Delay {
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

