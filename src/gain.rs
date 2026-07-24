use crate::sample_range::SampleRange;
pub struct Gain {
    value: f64,
}

impl Gain {
    pub fn new(value: f64) -> Self {
        Self { value }
    }
    pub fn set(&mut self, value: f64) {
        self.value = value;
    }
    pub fn value(&self) -> f64 {
        self.value
    }
    pub fn display(&self) -> String {
        format!("Gain: {:.2}", self.value)
    }
}

pub struct GainProcessor {
    sample_range: SampleRange,
}

impl GainProcessor {
    pub fn new(sample_range: SampleRange) -> Self {
        
        Self {
            sample_range,
        }
    }
    pub fn apply_gain(&self, sample: i32, gain: f64) -> i32 {

        let scaled = (sample as f64) * gain;
        scaled.clamp(self.sample_range.min_sample, self.sample_range.max_sample).round() as i32
    }
}


