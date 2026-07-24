use crate::sample_range::SampleRange;
use crate::processor::Processor;

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
        let upper = self.sample_range.max_sample * self.ceiling;
        let lower = self.sample_range.min_sample * self.ceiling;

        (sample as f64).clamp(lower, upper).round() as i32
    }
}

#[cfg(test)]
mod tests{
    use super::*;

    #[test]
    fn hard_clipper_limits_positive_and_negative_samples() -> Result<(), Box<dyn std::error::Error>>{
        let clip = HardClipper::new(SampleRange::new(16), 0.5)?;

        assert_eq!(clip.apply(30000), 16384);// 32767 / 2 = 16383.5 rounded to 16384
        assert_eq!(clip.apply(-30000), -16384); // -32768 / 2 = -16384
        assert!(HardClipper::new(SampleRange::new(16), 2.0).is_err());
        assert!(HardClipper::new(SampleRange::new(16), -2.0).is_err());
        Ok(())
        // ceiling 0.5
    }
}

impl Processor for HardClipper{
    fn process(&self, sample: i32) -> i32{
        self.apply(sample)
    }
}