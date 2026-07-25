use crate::sample_range::SampleRange;
use crate::processor::Processor;
use crate::utility::mixer;

#[derive (Clone)]
pub struct HardClipper{
    sample_range: SampleRange,
    ceiling: f64,
}

impl HardClipper {  
    pub fn new(
        sample_range: SampleRange, 
        ceiling: f64
    ) -> Result<Self, Box<dyn std::error::Error>>{
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
    pub fn ceiling(&self) ->f64{
        self.ceiling
    }
    pub fn set_ceiling(&mut self, new_value: f64){
        self.ceiling = new_value;
    }
    pub fn apply(&self, sample: i32) -> i32 {
        let upper = self.sample_range.max_sample * self.ceiling;
        let lower = self.sample_range.min_sample * self.ceiling;
        (sample as f64).clamp(lower, upper).round() as i32
    }
}

pub struct SoftClipper{
    sample_range: SampleRange,
    threshold: f64,
    drive: f64,
    mix: f64,
    sample_rate: f64,
}

impl SoftClipper{
    pub fn new(
        sample_range: SampleRange, 
        threshold: f64, 
        drive: f64,
        mix: f64, 
        sample_rate:f64
    ) -> Result<Self, Box<dyn std::error::Error>>{

        if !(0.0..=1.0)
            .contains(&threshold) {
                return Err("Clipping ceiling must be between 0.0 and 1.0".into());
        }
        else{
            Ok(Self{
                sample_range,
                threshold,
                drive,
                mix: mix.clamp(0.0, 1.0),
                sample_rate,
            })
        }
    }
    pub fn threshold(&self) -> f64{
        self.threshold
    }
    pub fn set_threshold(&mut self, new_value: f64){
        self.threshold = new_value
    }
    pub fn mix(&self) -> f64{
        self.mix
    }
    pub fn set_mix(&mut self, new_value: f64){
        self.mix = new_value;
    }

    /// Continuous log-based soft clipper
    /// `input`: Audio sample normalized between -1.0 and 1.0
    /// `drive`: Saturation factor (>= 0.0). 0.0 is completely linear.
    pub fn apply(&self, s: i32, drive: f64) -> i32 {
        let sample = s as f64 / self.sample_range.min_sample.abs() as f64; 
        if drive <= 0.0 {
            return s;
        }
        let sign = sample.signum();
        let abs_input = sample.abs();
        // Formula: sign(x) * ln(1 + drive * |x|) / ln(1 + drive)
        ((sign * ((1.0 + drive * abs_input).ln()) / (1.0 + drive).ln()) * self.sample_range.min_sample.abs()) as i32
    }

    /// Piecewise band soft clipper
    /// `input`: Audio sample normalized between -1.0 and 1.0
    /// `threshold`: Where dampening begins (e.g., 0.5 or 0.7)
    pub fn band_clip(&self, input: i32, threshold: f64) -> i32 {
        let sample = (input as f64 / self.sample_range.min_sample.abs() as f64).abs();
        let abs_input = sample.abs();
        
        // 1. Linear region (No dampening)
        if abs_input <= threshold {
            return input;
        }
        
        // 2. Hard limit boundary
        if abs_input >= 1.0 {
            return (input.signum() as f64 * 1.0) as i32;
        }
        
        // 3. Dampening/Compression band
        let sign = input.signum();
        
        // Normalize position within the wet dampening band [threshold, 1.0]
        let num = abs_input - threshold;
        let den = 1.0 - threshold;
        let normalized_pos = num / den;
        
        // Smooth soft-clipping curve (Cubic interpolation)
        let dampened = threshold + den * (normalized_pos - (normalized_pos.powi(3) / 3.0));
        mixer((sign as f64 * dampened) as i32, input, self.mix)
        // self.mixer(input, (sign as f64 * dampened) as i32)
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
    }
}

impl Processor for HardClipper{
    fn process(&mut self, sample: i32) -> i32{
        self.apply(sample)
    }
}

impl Processor for SoftClipper{
    fn process(&mut self, sample: i32) -> i32{
        // self.band(sample, self.drive)
        self.band_clip(sample, self.threshold)
    }
}