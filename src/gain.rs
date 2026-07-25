use crate::sample_range::SampleRange;
use crate::processor::Processor;

#[derive (Copy, Clone)]
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
    pub fn gain(&self) ->f64{
        self.gain
    }
    pub fn set_gain(&mut self, new_value: f64){
        self.gain = new_value;
    }
    pub fn apply_gain(&self, sample: i32, gain: f64) -> i32 {

        let scaled: f64 = (sample as f64) * gain;
        scaled.clamp(self.sample_range.min_sample, self.sample_range.max_sample).round() as i32
    }
}

impl Processor for Gain{
    fn process(&mut self, sample: i32) -> i32{
        // println!("Gain::process({})", sample);
        self.apply_gain(sample, self.gain)
    }
}


