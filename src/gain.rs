use crate::sample_range::SampleRange;
use crate::processor::Processor;

pub struct Gain {
    gain: f64,
    sample_range: SampleRange,
}

impl Gain {
    pub fn new(gain: f64, sample_range: SampleRange) -> Self {
        
        Self {
            gain,
            sample_range,
        }
    }
    pub fn apply_gain(&self, sample: i32, gain: f64) -> i32 {

        let scaled = (sample as f64) * gain;
        scaled.clamp(self.sample_range.min_sample, self.sample_range.max_sample).round() as i32
    }
}

impl Processor for Gain{
    fn process(&self, sample: i32) -> i32{
        self.apply_gain(sample, self.gain)
    }
}


