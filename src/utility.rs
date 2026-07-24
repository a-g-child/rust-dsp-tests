
use crate::sample_range::SampleRange;



    pub fn sample_to_amplitude(
        sample: i32,
        range: SampleRange,
    ) -> f64 {
            let magnitude = i64::from(sample).abs() as f64;
            let full_scale = -range.min_sample;
            magnitude / full_scale
    }

    pub fn amplitude_to_dbfs(
        amplitude: f64
    ) -> f64 {
            //Convert amplitude to dBFS (with a tiny floor to prevent log(0))k
            if amplitude < 1e-5 {
                return -100.0 // Hard noise floor in dBFS for near-silence
            } else {
                return 20.0 * amplitude.log10()
            };
    }

