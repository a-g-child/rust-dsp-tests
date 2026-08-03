use crate::{
    processor::Processor, sample_range::SampleRange, utility::clamp_to_bit_depth, utility::mixer,
};
use crate::parameter::{ParameterId, ParameterInfo, ParameterError, validate_parameter};

const DELAY_PARAMETERS: [ParameterInfo; 6] = [
    ParameterInfo {
        id: ParameterId::DelayTime,
        name: "time_left",
        min: 0.0,
        max: 10000.0,
        default: 300.0,
    },
    ParameterInfo {
        id: ParameterId::DelayTime,
        name: "time_right",
        min: 0.0,
        max: 10000.0,
        default: 300.0,
    },
    ParameterInfo {
        id: ParameterId::Feedback,
        name: "feedback_left",
        min: 0.0,
        max: 0.99,
        default: 0.5,
    },
    ParameterInfo {
        id: ParameterId::Feedback,
        name: "feedback_right",
        min: 0.0,
        max: 0.99,
        default: 0.5,
    },
    ParameterInfo {
        id: ParameterId::Mix,
        name: "mix_left",
        min: 0.0,
        max: 1.0,
        default: 0.5,
    },
    ParameterInfo {
        id: ParameterId::Mix,
        name: "mix_right",
        min: 0.0,
        max: 1.0,
        default: 0.5,
    },
];

struct DelayLine {
    buffer: Vec<f64>,
    position: usize,
}

impl DelayLine {
    fn new(delay_samples: usize) -> Self {
        Self {
            buffer: vec![0.0; delay_samples.max(1)],
            position: 0,
        }
    }
    fn read(&self) -> f64 {
        self.buffer[self.position]
    }
    fn write_and_advance(&mut self, sample: f64) {
        self.buffer[self.position] = sample;
        self.position += 1;
        if self.position == self.buffer.len() {
            self.position = 0;
        }
    }
    fn len(&self) -> f64 {
        self.buffer.len() as f64
    }
}

pub struct Delay {
    lines: [DelayLine; 2],
    feedback: [f64; 2],
    mix: [f64; 2],
    bit_depth: SampleRange,
    sample_rate: f64,
}

impl Delay {
    pub fn new(
        time_ms: [f64; 2],
        feedback: [f64; 2],
        mix: [f64; 2],
        sample_rate: f64,
        bit_depth: SampleRange,
    ) -> Self {
        Self {
            lines: [
                DelayLine::new((sample_rate * time_ms[0] / 1000.0) as usize),
                DelayLine::new((sample_rate * time_ms[1] / 1000.0) as usize),
            ],
            feedback,
            mix,
            bit_depth,
            sample_rate,
        }
    }

    fn process_channel(&mut self, channel: usize, input: i32) -> i32 {
        let inp: f64 = input as f64;
        let delayed = self.lines[channel].read();
        let output = inp + delayed * self.mix[channel];
        let feedback_sample =
            clamp_to_bit_depth(inp + delayed * self.feedback[channel], self.bit_depth);
        self.lines[channel].write_and_advance(feedback_sample);
        clamp_to_bit_depth(output, self.bit_depth) as i32
    }
}

impl Processor for Delay {
    fn process(&mut self, sample: i32) -> i32 {
        self.process_channel(0, sample)
    }
    fn process_buffer(&mut self, buffer: &mut [i32]) {
        for frame in buffer.chunks_exact_mut(2) {
            let left = frame[0];
            let right = frame[1];
            let l = self.process_channel(0, left);
            let r = self.process_channel(1, right);
            frame[0] = l;
            frame[1] = r;
        }
    }
    fn parameters(&self) -> &[ParameterInfo]{
        &DELAY_PARAMETERS
    }
    fn get_parameter(&self, id: ParameterId) -> Option<f64>{
        match id {
            ParameterId::DelayTimeLeft => Some(self.lines[0].len() / self.sample_rate * 1000.0),
            ParameterId::DelayTimeRight => Some(self.lines[1].len() / self.sample_rate * 1000.0),
            ParameterId::FeedbackLeft => Some(self.feedback[0]),
            ParameterId::FeedbackRight => Some(self.feedback[1]),
            ParameterId::MixLeft => Some(self.mix[0]),
            ParameterId::MixRight => Some(self.mix[1]),
            _ => None,
        }
    }

    fn set_parameter(
        &mut self,
        id: ParameterId,
        value: f64,
    ) -> Result<(), ParameterError> {
        match id {
            ParameterId::DelayTimeLeft => {
                let info: &ParameterInfo = &DELAY_PARAMETERS[0];
                validate_parameter(id, value, info)?;
                self.lines[0] = DelayLine::new((self.sample_rate * value / 1000.0) as usize);
                Ok(())
            }
            ParameterId::DelayTimeRight => {
                let info: &ParameterInfo = &DELAY_PARAMETERS[1];
                validate_parameter(id, value, info)?;
                self.lines[1] = DelayLine::new((self.sample_rate * value / 1000.0) as usize);
                Ok(())
            }
            ParameterId::FeedbackLeft => {
                let info: &ParameterInfo = &DELAY_PARAMETERS[2];
                validate_parameter(id, value, info)?;
                self.feedback[0] = value;
                Ok(())
            }
            ParameterId::FeedbackRight => {
                let info: &ParameterInfo = &DELAY_PARAMETERS[3];   
                validate_parameter(id, value, info)?;
                self.feedback[1] = value;
                Ok(())
            }
            ParameterId::MixLeft => {
                let info: &ParameterInfo = &DELAY_PARAMETERS[4];
                validate_parameter(id, value, info)?;
                self.mix[0] = value;
                Ok(())
            }
            ParameterId::MixRight => {
                let info: &ParameterInfo = &DELAY_PARAMETERS[5];
                validate_parameter(id, value, info)?;
                self.mix[1] = value;
                Ok(())
            }
            _ => Err(
                ParameterError::UnknownParameter(id),
            ),
        }
    }
}
