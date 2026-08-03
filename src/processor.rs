use crate::parameter::{ParameterId, ParameterInfo, ParameterError};

pub trait Processor {
    fn process(&mut self, sample: i32) -> i32;

    fn process_buffer(&mut self, buffer: &mut [i32]) {
        for sample in buffer {
            *sample = self.process(*sample);
            // println!("processing...")
        }
    }

    fn parameters(&self) -> &[ParameterInfo];

    fn get_parameter(&self, id: ParameterId) -> Option<f64>;

    fn set_parameter(
        &mut self,
        id: ParameterId,
        value: f64,
    ) -> Result<(), ParameterError>;
}
