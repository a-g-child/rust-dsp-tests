use crate::processor::Processor;
use crate::wav::WavAudioContext;

pub struct TailSettings {
    min_tail_frames: usize,
    max_tail_frames: usize,
    tail_frame_count: usize,
    rendered_frames: usize,
    silent_frames_count: usize,
    silent_blocks_threshold: usize,
    silence_level_threshold: i32,
}

impl TailSettings {
    pub fn new(
        silence_level_threshold: i32,
        silent_blocks_threshold: usize,
        max_timeout_period_s: f64,
        min_timeout_period_s: f64,
        sample_rate: f64,
    ) -> Self {

            let min_tail_frames =
            (sample_rate * min_timeout_period_s).round() as usize;

            let max_tail_frames = 
            (sample_rate * max_timeout_period_s).round() as usize;

            Self{
                min_tail_frames,
                max_tail_frames,
                tail_frame_count: 0,
                rendered_frames: 0,
                silent_frames_count: 0,
                silent_blocks_threshold,
                silence_level_threshold,
            }
    }
    

}

pub struct WavRenderer {
    audio: WavAudioContext,
    output: hound::WavWriter<std::io::BufWriter<std::fs::File>>,
    chain: Vec<Box<dyn Processor>>,
    block_frames: usize,
    tail: TailSettings,
}

impl WavRenderer {

    pub fn new(
        audio: WavAudioContext,
        output_path: &str,
        chain: Vec<Box<dyn Processor>>,
        block_frames: usize,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let output = hound::WavWriter::create(output_path, audio.spec())?;
        let sample_rate = audio.sample_rate();

        Ok(Self {
            audio,
            output,
            chain,
            block_frames,
            tail: TailSettings::new(
                0, 
                20, 
                10.0, 
                1.0, 
                sample_rate as f64)
        })
    }

    fn process_block(&mut self, buffer: &mut Vec<i32>) {
        for processor in &mut self.chain {
            processor.process_buffer(buffer);
        }
    }

    fn write_block(
        &mut self,
        buffer: &[i32],
    ) -> Result<(), Box<dyn std::error::Error>> {
        for sample in buffer {
            self.output.write_sample(*sample)?;
        }
        Ok(())
    }

    pub fn render(mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("rendering source");
        
        self.render_source()?;
        self.render_tail()?;
         println!("finalizing output");
        self.output.finalize()?;
        Ok(())
    }

    fn render_source(&mut self) -> Result<(), Box<dyn std::error::Error>>{
        let mut buffer: Vec<i32> = Vec::with_capacity(self.block_frames);
        loop {
            let read_len = 
                self.audio
                .read_block(
                    &mut buffer, 
                    self.block_frames * 
                    self.audio.channels() as usize)?;
            if read_len == 0 {
                break; // Stop if no data is read
            }
            self.process_block(&mut buffer);
            self.write_block(&buffer)?;
        }
        Ok(())
    }

    fn render_tail(&mut self) -> Result<(), Box<dyn std::error::Error>> {

            while self.tail.rendered_frames < self.tail.max_tail_frames{

                let remaining = self.tail.max_tail_frames - self.tail.rendered_frames;
                let frames_this_block = remaining.min(self.block_frames);
                let samples_this_block = frames_this_block * self.audio.channels() as usize;

                // creates full block to accomodate samples
                let mut tail_buffer =
                    vec![0_i32; samples_this_block];
        
                self.process_block(&mut tail_buffer);
                let peak = block_peak(&tail_buffer);
                self.write_block(&mut tail_buffer)?;

                self.tail.rendered_frames += frames_this_block;
                // println!("peak: {}", peak);
                if peak <= self.tail.silence_level_threshold  {
                    self.tail.silent_frames_count += 1;
                } else {
                    self.tail.silent_frames_count = 0;
                }

                if self.tail.rendered_frames >= self.tail.max_tail_frames
                    && self.tail.silent_frames_count>= self.tail.silent_blocks_threshold
                {
                    break;
                }
            }
        
        Ok(())

    }

    
}

pub fn block_peak(buffer: &[i32]) -> i32 {
        buffer
            .iter()
            .map(|sample| sample.saturating_abs())
            .max()
            .unwrap_or(0)
    }



