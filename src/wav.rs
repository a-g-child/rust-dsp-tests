use crate::sample_range::SampleRange;

pub struct WavAudioContext {
    audio: hound::WavReader<std::io::BufReader<std::fs::File>>,
    spec: hound::WavSpec,
    channels: u16,
    bit_depth: u16,
    range: SampleRange,
    frames: u32,
    total_samples: u32,
}

impl WavAudioContext {
    pub fn new(input_path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let input_file = hound::WavReader::open(input_path)?;
        let spec = input_file.spec();
        let bit_depth = spec.bits_per_sample;
        let channels = spec.channels;
        let total_samples = input_file.duration();
        let frames = (total_samples as f64 / channels as f64) as u32;

        Ok(Self {
            audio: input_file,
            spec: spec,
            channels,
            bit_depth, 
            range: SampleRange::new(bit_depth),
            frames,
            total_samples,
        })
    }

    pub fn read_block(
        &mut self,
        buffer: &mut Vec<i32>,
        block_samples: usize,
    ) -> Result<usize, Box<dyn std::error::Error>>{
        buffer.clear();
        for sample in self.audio.samples(){
            buffer.push(sample?);
            if buffer.len() == block_samples{
                break;
            }
        }
        Ok(buffer.len())
    }
    pub fn audio(&mut self) -> &mut hound::WavReader<std::io::BufReader<std::fs::File>> {
        &mut self.audio
    }
    pub fn spec(&self) -> hound::WavSpec {
        self.spec
    }
    pub fn frames(&self) -> u32 {
        self.frames
    }
    pub fn total_samples(&self) -> u32 {
        self.total_samples
    }
    pub fn channels(&self) -> u16 {
        self.channels
    }
    pub fn range(&self) -> SampleRange {
        self.range
    }
    pub fn sample_rate(&self) -> u32 {
        self.spec.sample_rate
    }
    pub fn samples(&mut self) -> impl Iterator<Item = Result<i32, hound::Error>> + '_ {
        self.audio.samples()
    }
    pub fn bits_per_sample(&self) -> u16 {
        self.spec.bits_per_sample
    }
}

#[cfg(test)]
mod test{
    use super::*;

    #[test]
    fn read_block() -> Result<(), Box<dyn std::error::Error>>{
        let block_size = 256;
        let mut buffer: Vec<i32> = Vec::new();
        let mut wav = WavAudioContext::new("audio/input.wav")?;
        let len = wav.read_block(&mut buffer, block_size)?;
        assert_eq!(len, block_size);

        Ok(())
    }
}
