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

#[cfg(test)]
mod tests{
    use super::*;

    #[test]
fn sample_range_for_16_bit_pcm() {
    let range = SampleRange::new(16);
    assert_eq!(range.max_sample, 32767.0);// i32:max = 32767 
    assert_eq!(range.min_sample, -32768.0);// i32:min = -32768 
    
    
}
}