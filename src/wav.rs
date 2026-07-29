use crate::sample_range::SampleRange;

pub struct WavAudioContext {
    audio: hound::WavReader<std::io::BufReader<std::fs::File>>,
    spec: hound::WavSpec,
    channels: u16,
    bit_depth: u16,
    range: SampleRange,
    frames: u32,
}

impl WavAudioContext {
    pub fn new(
        input_path: &str
) -> Result<Self, Box<dyn std::error::Error>> {
        let input_file = 
            hound::WavReader::open(input_path)?;
        let audio_aux = 
            hound::WavReader::open(input_path)?;
        let spec = 
            hound::WavReader::open(input_path)?.spec();
        let frames = 
            hound::WavReader::open(input_path)?.duration();
        let bit_depth = input_file.spec().bits_per_sample;

        Ok(Self {
            audio: input_file,
            spec: spec,
            channels: spec.channels as u16,
            bit_depth: bit_depth,
            range: SampleRange::new(bit_depth),
            frames,
        })
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
    pub fn channels(&self) -> u16 {
        self.channels
    }
    pub fn range(&self) -> SampleRange{
        self.range
    }
    pub fn sample_rate(&self) -> u32{
        self.spec.sample_rate
    }
    pub fn samples(&mut self) -> impl Iterator<Item = Result<i32, hound::Error>> + '_ {
        self.audio.samples()
    }
    pub fn bits_per_sample(&self) -> u16 {
        self.spec.bits_per_sample
    }
}
