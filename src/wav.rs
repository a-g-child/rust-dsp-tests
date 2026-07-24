
pub struct Wav {
    audio: hound::WavReader<std::io::BufReader<std::fs::File>>,
    spec: hound::WavSpec,
    channels: u16,
    frames: u32,
}

impl Wav {
    pub fn new(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let file = hound::WavReader::open(path)?;
        let frames = file.len() as u32;
        let spec = file.spec();

        Ok(Self {
            audio: file,
            spec,
            channels: spec.channels as u16,
            frames: frames,
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
    pub fn samples_i32(&mut self) -> impl Iterator<Item = Result<i32, hound::Error>> + '_{
        self.audio.samples()
    }
    pub fn bits_per_sample(&self) -> u16 {
            self.spec.bits_per_sample
        }
}