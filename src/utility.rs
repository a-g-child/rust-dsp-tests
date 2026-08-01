use crate::sample_range::SampleRange;

/// doesn't check if mix value is valid and this is to be used in hot loop,
/// this needs to be assured outside of this function
pub fn mixer(wet: i32, dry: i32, mix: f64) -> i32 {
    let wet = wet as f64;
    let dry = dry as f64;

    (dry + (wet - dry) * mix).round() as i32
}
pub fn clamp_to_bit_depth(sample: f64, range: SampleRange) -> f64 {
    sample.clamp(range.min_sample, range.max_sample)
}
#[allow(dead_code)]
pub fn milliseconds_from_coefficient(sample_rate: f64, coefficient: f64) -> f64 {
    if coefficient > 0.0 {
        let ln_coeff = coefficient.ln();
        -1000.0 / (sample_rate * ln_coeff)
    } else {
        0.0 // Handle the case where attack_ms is non-positive
    }
}

pub fn coefficient_from_milleseconds(sample_rate: f64, ms: f64) -> f64 {
    (-1.0 / (sample_rate * (ms / 1000.0))).exp()
}

pub fn sample_to_amplitude(sample: i32, range: SampleRange) -> f64 {
    let magnitude = i64::from(sample).abs() as f64;
    let full_scale = -range.min_sample;
    magnitude / full_scale
}

pub fn amplitude_to_dbfs(amplitude: f64) -> f64 {
    //Convert amplitude to dBFS (with a tiny floor to prevent log(0))k
    if amplitude < 1e-5 {
        return -100.0; // Hard noise floor in dBFS for near-silence
    } else {
        return 20.0 * amplitude.log10();
    };
}
