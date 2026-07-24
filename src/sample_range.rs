#[derive(Copy, Clone)]
pub struct SampleRange{
    pub min_sample: f64,
    pub max_sample: f64,
}
impl SampleRange{
    pub fn new(bit_depth: u16) -> Self{
        let bits = bit_depth.clamp(1, 32) as u32;
        Self {
            min_sample: -(1_i64 << (bits - 1)) as f64,
            max_sample: ((1_i64 << (bits - 1)) - 1) as f64,
        }
    }
}