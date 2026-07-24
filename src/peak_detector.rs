use crate::wav::Wav;
use crate::sample_range::SampleRange;

pub struct PeakDetect{
    peak: i64,
}

impl PeakDetect{
    
    pub fn new(wav: &mut Wav) -> Result<Self, Box<dyn std::error::Error>>  {
        let mut p = 0;
        wav.audio().samples().try_for_each(|sample| -> Result<(), Box<dyn std::error::Error>> {
                let sample: i32 = sample?;
                let magnitude = i64::from(sample).abs();   
                p = p.max(magnitude); 
                Ok(())
            })?;
            Ok(Self{peak: p})
    }
    pub fn peak_detect(&mut self, sample: i32) {
        let magnitude = i64::from(sample).abs();
        self.peak = self.peak.max(magnitude);
    }
    pub fn peak(&self) -> i64{
        self.peak
    }
    pub fn normalised_gain(&self, sample_range: SampleRange) -> Result<f64, Box<dyn std::error::Error>> {

        if self.peak == 0 {
            return Err("Peak is zero, cannot compute normalized gain".into());
        }
        Ok(sample_range.max_sample / self.peak as f64)
    }
}
