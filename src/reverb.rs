use crate::parameter::{ParameterError, ParameterId, ParameterInfo};
use crate::processor::Processor;
use crate::utility::mixer;
use rand::RngExt;

const REVERB_PARAMETERS: [ParameterInfo; 3] = [
    ParameterInfo {
        id: ParameterId::Size,
        name: "size",
        min: 0.0,
        max: 4.0,
        default: 1.0,
    },
    ParameterInfo {
        id: ParameterId::Spread,
        name: "spread",
        min: 0.0,
        max: 4.0,
        default: 1.0,
    },
    ParameterInfo {
        id: ParameterId::Feedback,
        name: "feedback",
        min: 0.0,
        max: 4.0,
        default: 1.0,
    },
];

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
    fn size(&self) -> usize {
        self.buffer.len()
    }
    fn write_and_advance(&mut self, sample: f64) {
        self.buffer[self.position] = sample;

        self.position += 1;

        if self.position == self.buffer.len() {
            self.position = 0;
        }
    }
}

#[derive(Clone)]
struct AllPassLine {
    buffer: Vec<f64>,
    position: usize,
}

impl AllPassLine {
    fn new(size: usize) -> Self {
        Self {
            buffer: vec![0.0; size],
            position: 0,
        }
    }
    fn read(&self) -> f64 {
        self.buffer[self.position]
    }
    fn size(&self) -> usize {
        self.buffer.len()
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
    all_pass_lines: [[AllPassLine; 3]; 2],
    decay: [f64; 2],
    mix: [f64; 2],
}

impl Reverb {
    pub fn new(
        delay_ms: f64,
        spread_ms: f64,
        decay: [f64; 2],
        mix: [f64; 2],
        sample_rate: f64,
    ) -> Self {
        let size = (delay_ms * sample_rate / 1000.0).max(1.0) as usize;
        let spread = (spread_ms * sample_rate / 1000.0).max(0.0) as usize;
        let all_pass_line1 = [
            AllPassLine::new(220),
            AllPassLine::new(70),
            AllPassLine::new(30),
        ];
        let all_pass_line2 = [
            AllPassLine::new(220),
            AllPassLine::new(70),
            AllPassLine::new(30),
        ];
        let lines = std::array::from_fn(|_| std::array::from_fn(|_| ReverbLine::new(size, spread)));

        Self {
            lines,
            all_pass_lines: [all_pass_line1, all_pass_line2],
            decay: [decay[0].clamp(0.0, 0.99), decay[1].clamp(0.0, 0.99)],
            mix: [mix[0].clamp(0.0, 1.0), mix[1].clamp(0.0, 1.0)],
        }
    }
    pub fn process_channel(&mut self, channel: usize, sample: i32) -> i32 {
        let input = sample as f64;
        let mut wet_sum = 0.0;

        for line in &mut self.lines[channel] {
            let delayed = line.read();

            wet_sum += delayed;

            let feedback_sample = input + delayed * self.decay[channel];

            line.write_and_advance(feedback_sample);
        }

        let wet = wet_sum / self.lines[channel].len() as f64;

        for line in &mut self.all_pass_lines[channel] {
            let delayed = line.read();

            // wet_sum += delayed;
            // wet_sum /= 2.0;

            let feedback_sample = input + delayed * self.decay[channel];

            line.write_and_advance(feedback_sample);
        }

        mixer(wet.round() as i32, sample, self.mix[channel])
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
    fn parameters(&self) -> &[ParameterInfo]{
        &REVERB_PARAMETERS
    }
    fn get_parameter(&self, id: ParameterId) -> Option<f64>{
        Some(0.0)
    }

    fn set_parameter(&mut self, id: ParameterId, value: f64,) -> Result<(), ParameterError> {
        Ok(())
    }

    
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn reverb_line() {
        let mut line = ReverbLine {
            buffer: vec![0.0; 3],
            position: 0,
        };

        let impulse = [1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];
        let mut output = Vec::with_capacity(impulse.len());

        for segment in impulse {
            let delayed = line.read();
            output.push(delayed);

            let feedback_sample = segment + delayed * 0.5;
            line.write_and_advance(feedback_sample);
        }

        assert_eq!(output, [0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.5]);
    }

    #[test]
    fn silence_stays_silent() {
        let mut reverb = Reverb::new(1.0, 0.0, [0.5, 0.5], [1.0, 1.0], 1000.0);
        let mut buffer = [0, 0, 0, 0, 0, 0, 0, 0];

        reverb.process_buffer(&mut buffer);

        assert_eq!(buffer, [0, 0, 0, 0, 0, 0, 0, 0]);
    }

    #[test]
    fn impulse_produces_late_output() {
        let mut reverb = Reverb::new(1.0, 0.0, [0.5, 0.5], [1.0, 1.0], 1000.0);
        let mut buffer = [1, 0, 0, 0, 0, 0, 0, 0];

        reverb.process_buffer(&mut buffer);

        assert!(buffer[2..].iter().any(|&sample| sample != 0));
    }
}
