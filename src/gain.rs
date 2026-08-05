use crate::parameter::{ParameterInfo, ParameterId, ParameterError, ParameterAddress, ParameterScope};
use crate::processor::Processor;
use crate::sample_range::SampleRange;

const GAIN_PARAMETERS: [ParameterInfo; 1] = [
    ParameterInfo {
        id: ParameterId::Gain,
        name: "Gain",
        min: 0.0,
        max: 4.0,
        default: 1.0,
        scope: ParameterScope::Global,
    },
];

#[derive(Copy, Clone)]
pub struct Gain {
    gain: f64,
    sample_range: SampleRange,
}

impl Gain {
    pub fn new(gain: f64, sample_range: SampleRange) -> Self {
        Self { gain, sample_range }
    }
    pub fn apply_gain(&self, sample: i32, gain: f64) -> i32 {
        let scaled: f64 = (sample as f64) * gain;
        scaled
            .clamp(self.sample_range.min_sample, self.sample_range.max_sample)
            .round() as i32
    }
}

impl Processor for Gain {
    fn process(&mut self, sample: i32) -> i32 {
        // println!("Gain::process({})", sample);
        self.apply_gain(sample, self.gain)
    }
    fn parameters(&self) -> &[ParameterInfo] {
        &GAIN_PARAMETERS
    }

    fn get_parameter(
        &self,
        address: ParameterAddress,
    ) -> Result<Option<f64>, ParameterError> {
        match address.id {
            ParameterId::Gain => Ok(Some(self.gain)),
            _ => Ok(None),
        }
    }

    fn set_parameter(
        &mut self,
        address: ParameterAddress,
        value: f64,
    ) -> Result<(), ParameterError> {
        match address.id {
            ParameterId::Gain => {
                let info = &GAIN_PARAMETERS[0];

                if !(info.min..=info.max).contains(&value) {
                    return Err(
                        ParameterError::OutOfRange {
                            id: address.id,
                            value,
                            min: info.min,
                            max: info.max,
                        },
                    );
                }

                self.gain = value;
                Ok(())
            }
            _ => Err(
                ParameterError::UnknownParameter(address.id),
            ),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::parameter::ParameterTarget;

use super::*;

    #[test]
    fn gain_processes_an_entire_buffer() {
        let mut processor = Gain::new(2.0, SampleRange::new(16));

        let i = processor.process(10);
        assert_eq!(i, 20);

        let mut buffer = [1_000, -2_000, 3_000, -4_000];

        processor.process_buffer(&mut buffer);

        assert_eq!(buffer, [2_000, -4_000, 6_000, -8_000,]);
    }

    #[test]
    fn gain_exposes_parameter_metadata() {

        let gain = Gain::new(2.0, SampleRange::new(16));

        let parameters = gain.parameters();

        assert_eq!(parameters.len(), 1);
        assert_eq!(
            parameters[0].id,
            ParameterId::Gain,
        );
    }
    #[test]
    fn gain_parameter_can_be_updated_through_trait() {
        let mut gain = Gain::new(2.0, SampleRange::new(16));

        let processor: &mut dyn Processor =
            &mut gain;

        processor
            .set_parameter(
                ParameterAddress { id: ParameterId::Gain, target: ParameterTarget::Global },
                2.0,
            )
            .unwrap();

        // assert_eq!(
        //     processor.get_parameter(
        //         ParameterAddress { id: ParameterId::Gain, target: ParameterTarget::Global }
        //     ),
        //     (Some(2.0))?,
        // );
    }
    #[test]
    fn gain_rejects_out_of_range_parameter() {
        let mut gain = Gain::new(2.0, SampleRange::new(16));

        let result =
            gain.set_parameter(
                ParameterAddress { id: ParameterId::Gain, target: ParameterTarget::Global },
                10.0,
            );

        assert!(matches!(
            result,
            Err(ParameterError::OutOfRange { .. })
        ));
    }
    #[test]
    fn gain_rejects_unknown_parameter() {
        let mut gain = Gain::new(2.0, SampleRange::new(16));

        let result =
            gain.set_parameter(
                ParameterAddress { id: ParameterId::Feedback, target: ParameterTarget::Global },
                0.5,
            );

        assert_eq!(
            result,
            Err(
                ParameterError::UnknownParameter(
                    ParameterId::Feedback,
                ),
            ),
        );
    }
}
