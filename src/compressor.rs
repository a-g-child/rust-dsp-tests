use crate::sample_range::SampleRange;
use crate::processor::Processor;
use crate::utility::{
    sample_to_amplitude,
    amplitude_to_dbfs,
    coefficient_from_milleseconds,
};

#[derive(Clone)]
pub struct Compressor {
    sample_range: SampleRange,
    threshold: f64,
    ratio: f64,
    attack_coeff: f64,  // Pre-calculated smoothing coefficient
    release_coeff: f64, // Pre-calculated smoothing coefficient
    current_gain: f64,  // Tracks the smoothed envelope state
}

impl Compressor {
    pub fn new(
        range: SampleRange, 
        threshold: f64, 
        ratio: f64, 
        attack_ms: f64, 
        release_ms: f64, 
        sample_rate: f64
    ) -> Self {
            // Standard digital filter coefficient calculation from milliseconds
            let attack_coeff = coefficient_from_milleseconds(sample_rate, attack_ms);
            let release_coeff = coefficient_from_milleseconds(sample_rate, release_ms);

            Self {
                sample_range: range,
                threshold,
                ratio,
                attack_coeff,
                release_coeff,
                current_gain: 1.0, // Start with completely clean path
            }
    }
    pub fn ratio(&self) -> f64{
        self.ratio
    }
    pub fn set_ratio(&mut self, new_value: f64){
        self.ratio = new_value;
    }
    pub fn threshold(&self) -> f64{
        self.threshold
    }
    pub fn set_threshold(&mut self, new_value: f64){
        self.threshold = new_value;
    }
    pub fn apply(&mut self, sample: i32, gain: f64) -> i32 {
        
        // Calculate the target reduction for this single sample
        let target_reduction: f64 = self.calculate_gain_reduction(sample);
        // Determine if we are attacking (compressing more) or releasing (recovering)
        // Note: target_reduction is smaller when compression increases (e.g., 0.5 vs 1.0)
        let coeff: f64 = if target_reduction < self.current_gain {
            self.attack_coeff
        } else {
            self.release_coeff
        };
        // Smooth the envelope state (Exponential Moving Average filter)
        self.current_gain = coeff * self.current_gain + (1.0 - coeff) * target_reduction;
        // Multiply the original sample by the smoothed envelope state
        let i = (sample as f64 * self.current_gain) as i32;
        if target_reduction != 1.0 {println!("sample: {}, compressed: {}, target_reduction: {}", sample, i, target_reduction)};
        i
    }
    // This remains a read-only mathematical helper
    pub fn calculate_gain_reduction(&self, sample: i32) -> f64 {
        let amplitude: f64 = sample_to_amplitude(sample, self.sample_range);
        let input_db: f64 = amplitude_to_dbfs(amplitude);
        if input_db > self.threshold {
            let excess_db: f64 = input_db - self.threshold;
            let compressed_db: f64 = self.threshold + (excess_db / self.ratio);
            let reduction_db: f64 = compressed_db - input_db; 
            10.0f64.powf(reduction_db / 20.0)
        } else {
            1.0
        }
    }
}

impl Processor for Compressor{
    fn process(&mut self, sample: i32) -> i32{
        self.apply(sample, self.current_gain)
    }
    // stereo compression which technically sidechains the average of left and right as the target reduction.
    fn process_buffer(&mut self, buffer: &mut [i32]) {
        for frame in buffer.chunks_exact_mut(2) {
            let left = frame[0];
            let right = frame[1];

            let average: i32 = (left as f64 + right as f64 / 2.0) as i32;
            let target_reduction = self.calculate_gain_reduction(average);
            self.apply(frame[0], target_reduction);
            self.apply(frame[1], target_reduction);
            
        }
    }
}