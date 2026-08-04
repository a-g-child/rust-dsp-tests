use crate::parameter::{ParameterAddress, ParameterError, ParameterId, ParameterInfo, ParameterScope, validate_channel_exists, validate_has_channels, validate_parameter};
use crate::processor::Processor;
use crate::sample_range::SampleRange;
use crate::utility::mixer;

const HARDCLIP_PARAMETERS: [ParameterInfo; 2] = [
    ParameterInfo {
        id: ParameterId::Ceiling,
        name: "ceiling",
        min: 0.0,
        max: 1.0,
        default: 1.0,
        scope: ParameterScope::Global,
    },
    ParameterInfo {
        id: ParameterId::Mix,
        name: "mix",
        min: 0.0,
        max: 1.0,
        default: 0.5,
        scope: ParameterScope::Global,
    },
];

const SOFTCLIP_PARAMETERS: [ParameterInfo; 3] = [
    // mode require option list
    // ParameterInfo {
    //     id: ParameterId::Mode,
    //     name: "Mode",
    //     min: 0.0,
    //     max: 1.0,
    //     default: 1.0,
    // },
    ParameterInfo {
        id: ParameterId::Threshold,
        name: "threshold",
        min: 0.0,
        max: 1.0,
        default: 0.5,
        scope: ParameterScope::Global,
    },
    ParameterInfo {
        id: ParameterId::Mix,
        name: "mix",
        min: 0.0,
        max: 1.0,
        default: 0.5,
        scope: ParameterScope::Global,
    },
    ParameterInfo {
        id: ParameterId::Mix,
        name: "mix",
        min: 0.0,
        max: 1.0,
        default: 0.5,
        scope: ParameterScope::Global,
    },
];

#[derive(Clone)]
pub struct HardClipper {
    sample_range: SampleRange,
    ceiling: f64,
    mix: f64,
}
impl HardClipper {
    pub fn new(
        sample_range: SampleRange,
        ceiling: f64,
        mix: f64,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        if !(0.0..=1.0).contains(&ceiling) {
            return Err("Clipping ceiling must be between 0.0 and 1.0".into());
        } else {
            Ok(Self {
                sample_range,
                ceiling,
                mix: mix.clamp(0.0, 1.0),
            })
        }
    }
    #[allow(dead_code)]
    pub fn ceiling(&self) -> f64 {
        self.ceiling
    }
    #[allow(dead_code)]
    pub fn set_ceiling(&mut self, new_value: f64) {
        self.ceiling = new_value.clamp(0.0, 1.0);
    }
    pub fn apply(&self, sample: i32) -> i32 {
        let upper = self.sample_range.max_sample * self.ceiling;
        let lower = self.sample_range.min_sample * self.ceiling;
        let wet =(sample as f64).clamp(lower, upper).round() as i32;
        mixer(wet, sample, self.mix)
    }
}

pub enum SoftClipperMode {
    Logarithmic,
    #[allow(dead_code)]
    CubicBand,
}

pub struct SoftClipper {
    sample_range: SampleRange,
    mode: SoftClipperMode,
    threshold: f64,
    drive: f64,
    mix: f64,
}

impl SoftClipper {
    pub fn new(
        sample_range: SampleRange,
        mode: SoftClipperMode,
        threshold: f64,
        drive: f64,
        mix: f64,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        if !(0.0..=1.0).contains(&threshold) {
            return Err("Clipping ceiling must be between 0.0 and 1.0".into());
        } else {
            Ok(Self {
                sample_range,
                mode,
                threshold: threshold.clamp(0.0, 1.0),
                drive: drive.clamp(1.0, 50.0),
                mix: mix.clamp(0.0, 1.0),
            })
        }
    }
    #[allow(dead_code)]
    pub fn threshold(&self) -> f64 {
        self.threshold
    }
    #[allow(dead_code)]
    pub fn set_threshold(&mut self, new_value: f64) {
        self.mix = new_value.clamp(0.0, 1.0);
    }
    #[allow(dead_code)]
    pub fn mix(&self) -> f64 {
        self.mix
    }
    #[allow(dead_code)]
    pub fn set_mix(&mut self, new_value: f64) {
        self.mix = new_value.clamp(0.0, 1.0);
    }
    #[allow(dead_code)]
    pub fn drive(&self) -> f64 {
        self.drive
    }
    #[allow(dead_code)]
    pub fn set_drive(&mut self, new_value: f64) {
        self.drive = new_value.clamp(1.0, 50.0);
    }
    /// Continuous log-based soft clipper
    /// `input`: Audio sample normalized between -1.0 and 1.0
    /// `drive`: Saturation factor (>= 0.0). 0.0 is completely linear.
    pub fn apply_logarithmic(&self, s: i32) -> i32 {
        let sample = s as f64 / self.sample_range.min_sample.abs() as f64;
        if self.drive <= 1.0 {
            return s;
        }
        let sign = sample.signum();
        let abs_input = sample.abs();
        // Formula: sign(x) * ln(1 + drive * |x|) / ln(1 + drive)
        let wet = sign * ((1.0 + self.drive * abs_input).ln()) / (1.0 + self.drive).ln()
            * self.sample_range.min_sample.abs();
        mixer(wet.round() as i32, s, self.mix)
    }
    /// Piecewise band soft clipper
    /// `input`: Audio sample normalized between -1.0 and 1.0
    /// `threshold`: Where dampening begins (e.g., 0.5 or 0.7)
    pub fn apply_band(&self, input: i32) -> i32 {
        if self.threshold >= 1.0 {
            return input;
        }
        if self.drive <= 1.0 {
            return input;
        }
        let full_scale = self.sample_range.min_sample.abs();
        let sample = input as f64 / full_scale;
        let driven_sample = sample * self.drive;
        let abs_input = driven_sample.abs();
        let sign = driven_sample.signum();
        // 1. Linear region (No dampening)
        if abs_input <= self.threshold {
            return input;
        }
        // 2. Hard limit boundary
        let clipped_input = abs_input.min(1.0);
        // 3. Dampening/Compression band
        // Normalize position within the wet dampening band [threshold, 1.0]
        let num = clipped_input - self.threshold;
        let den = 1.0 - self.threshold;
        let normalized_pos = num / den;
        // Smooth soft-clipping curve (Cubic interpolation)
        let dampened = self.threshold + den * (normalized_pos - (normalized_pos.powi(3) / 3.0));
        let wet = sign * dampened * self.sample_range.max_sample;

        mixer(wet.round() as i32, input, self.mix)
    }
}

impl Processor for HardClipper {
    fn process(&mut self, sample: i32) -> i32 {
        self.apply(sample)
    }
        fn parameters(&self) -> &[ParameterInfo]{
        &HARDCLIP_PARAMETERS
    }
    fn get_parameter(&self, address: ParameterAddress) -> Result<Option<f64>, ParameterError>{
        match address.id {
            ParameterId::Ceiling => Ok(Some(self.ceiling)),
            ParameterId::Mix => Ok(Some(self.mix)),
            _ => Ok(None),
        }
    }

    fn set_parameter(
        &mut self,
        address: ParameterAddress,
        value: f64,
    ) -> Result<(), ParameterError> {
        match address.id {
            ParameterId::Ceiling => {
                let info: &ParameterInfo = &HARDCLIP_PARAMETERS[0];
                validate_parameter(address.id, value, info)?;
                self.ceiling = value;
                Ok(())
            }
            ParameterId::Mix => {
                let info: &ParameterInfo = &HARDCLIP_PARAMETERS[1];
                
                validate_parameter(address.id, value, info)?;

                self.mix = value;
                Ok(())
            }
            _ => Err(
                ParameterError::UnknownParameter(address.id),
            ),
        }
    }
}

impl Processor for SoftClipper {
    fn process(&mut self, sample: i32) -> i32 {
        match self.mode {
            SoftClipperMode::Logarithmic => self.apply_logarithmic(sample),
            SoftClipperMode::CubicBand => self.apply_band(sample),
        }
    }
    fn parameters(&self) -> &[ParameterInfo]{
        &SOFTCLIP_PARAMETERS
    }
    fn get_parameter(&self, address: ParameterAddress) -> Result<Option<f64>, ParameterError>{
        match address.id {
            ParameterId::Threshold => Ok(Some(self.threshold)),
            ParameterId::Drive => Ok(Some(self.drive)),
            ParameterId::Mix => Ok(Some(self.mix)),
            _ => Ok(None),
        }
    }

    fn set_parameter(
        &mut self,
        address: ParameterAddress,
        value: f64,
    ) -> Result<(), ParameterError> {
        match address.id {
            ParameterId::Threshold => {
                let info: &ParameterInfo = &SOFTCLIP_PARAMETERS[0];
                
                validate_parameter(address.id, value, info)?;

                self.threshold = value;
                Ok(())
            }
            ParameterId::Drive => {
                let info: &ParameterInfo = &SOFTCLIP_PARAMETERS[1];
                
                validate_parameter(address.id, value, info)?;

                self.drive = value;
                Ok(())
            }
            ParameterId::Mix => {
                let info: &ParameterInfo = &SOFTCLIP_PARAMETERS[2];
                
                validate_parameter(address.id, value, info)?;

                self.mix = value;
                Ok(())
            }
            _ => Err(
                ParameterError::UnknownParameter(address.id),
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hard_clipper_limits_positive_and_negative_samples() -> Result<(), Box<dyn std::error::Error>>
    {
        let clip = HardClipper::new(SampleRange::new(16), 0.5, 1.0)?;
        assert_eq!(clip.apply(30000), 16384); // 32767 / 2 = 16383.5 rounded to 16384
        assert_eq!(clip.apply(-30000), -16384); // -32768 / 2 = -16384
        assert!(HardClipper::new(SampleRange::new(16), 2.0, 0.5).is_err());
        assert!(HardClipper::new(SampleRange::new(16), -2.0, 0.5).is_err());
        Ok(())
    }

    #[test]
    fn band_clip_uses_drive_to_push_signal_into_clipping_band()
    -> Result<(), Box<dyn std::error::Error>> {
        let clip = SoftClipper::new(
            SampleRange::new(16),
            SoftClipperMode::Logarithmic,
            0.2,
            30.0,
            1.0,
        )?;

        assert_eq!(clip.apply_band(14846), 24029);
        assert_eq!(clip.apply_band(-14846), -24029);

        Ok(())
    }
}
