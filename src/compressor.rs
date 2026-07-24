use crate::sample_range::SampleRange;
use crate::utility::Utility;

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
            let attack_coeff = (-1.0 / (sample_rate * (attack_ms / 1000.0))).exp();
            let release_coeff = (-1.0 / (sample_rate * (release_ms / 1000.0))).exp();

            Self {
                sample_range: range,
                threshold,
                ratio,
                attack_coeff,
                release_coeff,
                current_gain: 1.0, // Start with completely clean path
            }
    }
    pub fn apply(&mut self, sample: i32) -> i32 {
        // Calculate the target reduction for this single sample
        let target_reduction = self.calculate_gain_reduction(sample);

        // Determine if we are attacking (compressing more) or releasing (recovering)
        // Note: target_reduction is smaller when compression increases (e.g., 0.5 vs 1.0)
        let coeff = if target_reduction < self.current_gain {
            self.attack_coeff
        } else {
            self.release_coeff
        };

        // Smooth the envelope state (Exponential Moving Average filter)
        self.current_gain = coeff * self.current_gain + (1.0 - coeff) * target_reduction;

        // Multiply the original sample by the smoothed envelope state
        (sample as f64 * self.current_gain) as i32
    }


    // This remains a read-only mathematical helper
    pub fn calculate_gain_reduction(&self, sample: i32) -> f64 {
        let amplitude: f64 = Utility::sample_to_amplitude(sample, self.sample_range);
        let input_db: f64 = Utility::amplitude_to_dBFS(amplitude);

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