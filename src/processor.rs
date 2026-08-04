use crate::parameter::{ParameterId, ParameterInfo, ParameterError, ParameterAddress};

pub trait Processor {
    fn process(&mut self, sample: i32) -> i32;

    fn process_buffer(&mut self, buffer: &mut [i32]) {
        for sample in buffer {
            *sample = self.process(*sample);
            // println!("processing...")
        }
    }

    fn parameters(&self) -> &[ParameterInfo];

    fn get_parameter(&self, address: ParameterAddress) -> Result<Option<f64>, ParameterError>;

    fn set_parameter(
        &mut self,
        address: ParameterAddress,
        value: f64,
    ) -> Result<(), ParameterError>;
}
