use crate::sample_range::SampleRange;

pub struct HardClipper{
    sample_range: SampleRange,
    ceiling: f64,
}

impl HardClipper {  
    pub fn new(sample_range: SampleRange, ceiling: f64) -> Result<Self, Box<dyn std::error::Error>>{
        if !(0.0..=1.0)
            .contains(&ceiling) {
                return Err("Clipping ceiling must be between 0.0 and 1.0".into());
        }
        else{
            Ok(Self{
                sample_range,
                ceiling,
            })
        }
    }

    pub fn apply(&self, sample: i32) -> i32 {

        let upper_bound = self.sample_range.max_sample * self.ceiling;
        let lower_bound = self.sample_range.min_sample * self.ceiling;
        let scaled = (sample as f64)
            .min(upper_bound)
            .max(lower_bound);

        let clipped = scaled.clamp(self
            .sample_range
            .min_sample, self
            .sample_range
            .max_sample);
        clipped.round() as i32
    }
}