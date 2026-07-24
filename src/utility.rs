
use crate::sample_range::SampleRange;
pub struct Utility{
}

impl Utility{

    pub fn sample_to_amplitude(sample: i32, range: SampleRange) -> f64 {
        // return sample as f64 / range.max_sample;
        return (sample.abs() as f64) / range.max_sample
    }

    pub fn amplitude_to_dBFS(amplitude: f64) -> f64 {
        //Convert amplitude to dBFS (with a tiny floor to prevent log(0))k
        if amplitude < 1e-5 {
            return -100.0 // Hard noise floor in dBFS for near-silence
        } else {
            return 20.0 * amplitude.log10()
        };
    }
}
