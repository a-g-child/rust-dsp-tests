mod compressor;
mod delay;
mod distortion;
mod gain;
mod modulator;
mod peak_detector;
mod processor;
mod sample_range;
mod utility;
mod wav;
mod reverb;

use crate::compressor::Compressor;
use crate::delay::Delay;
use crate::distortion::{HardClipper, SoftClipper, SoftClipperMode};
use crate::gain::Gain;
use crate::modulator::Fader;
use crate::peak_detector::PeakDetect;
use crate::processor::Processor;
use crate::sample_range::SampleRange;
use crate::wav::WavAudioContext;
use crate::reverb::Reverb;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    
    const BLOCK_SIZE: usize = 256;

    let mut wav = 
        WavAudioContext::new(
            "audio/input.wav")?;

    let mut aux = 
        WavAudioContext::new(
            "audio/input.wav")?;
    
    let mut output = 
        hound::WavWriter::create(
            "audio/out.wav", 
            wav.spec()
        )?;

    let (_delay, 
        reverb, 
        _comp, 
        gain_processor, 
        _soft_clipper,
        _clipper,
        _fader
    ) = initialise_processors(
            aux, 
            wav.spec(), 
            wav.channels(), 
            wav.range(), 
            wav.frames())?;

    let mut chain: Vec<Box<dyn Processor>> = vec![
        Box::new(gain_processor),
        // Box::new(sclipper),
        // Box::new(delay),
        Box::new(reverb),
        // Box::new(comp),
    ];

    let mut buffer: Vec<i32> = Vec::with_capacity(BLOCK_SIZE);

    for sample in wav.samples() {
        buffer.push(sample?);

        if buffer.len() == BLOCK_SIZE {
            process_chain(&mut chain, &mut buffer);

            for processed_sample in &buffer {
                output.write_sample(*processed_sample)?;
            }
            buffer.clear();
        }
    }
    if !buffer.is_empty() {
        process_chain(&mut chain, &mut buffer);

        for processed_sample in &buffer {
            output.write_sample(*processed_sample)?;
        }
        // println!("{:#?}", buffer.len() % 2);
        buffer.clear();
    }

    const BLOCK_FRAMES: usize = 128;
    const SILENCE_THRESHOLD: i32 = 10;
    const SILENT_BLOCKS_REQUIRED: usize = 20;
    const MAX_TAIL_SECONDS: f64 = 20.0;
    const MIN_TAIL_SECONDS: f64 = 1.0;
    
    let max_tail_frames =
        (wav.sample_rate() as f64
            * MAX_TAIL_SECONDS)
            .round() as usize;

    let mut rendered_frames = 0;
    let mut silent_blocks = 0;

    let minimum_tail_frames =
            (wav.sample_rate() as f64 * MIN_TAIL_SECONDS).round() as usize;

    while rendered_frames < max_tail_frames {
        let remaining =
            max_tail_frames - rendered_frames;
        // tracks number of frames and returns minimum of 128 if the actual value is less     
        let frames_this_block =
            remaining.min(BLOCK_FRAMES);
        // 2 channels
        let samples_this_block =
            frames_this_block * wav.channels() as usize;
        
        // creates full block to accomodate samples
        let mut tail_buffer =
            vec![0_i32; samples_this_block];
        
        process_chain(
            &mut chain,
            &mut tail_buffer,
        );

        let peak = block_peak(&tail_buffer);

        for sample in &tail_buffer {
            output.write_sample(*sample)?;
        }

        rendered_frames += frames_this_block;

        if peak <= SILENCE_THRESHOLD  {
            silent_blocks += 1;
        } else {
            silent_blocks = 0;
        }

        if rendered_frames >= minimum_tail_frames
            && silent_blocks >= SILENT_BLOCKS_REQUIRED
        {
            break;
        }
    }

    output.finalize()?;

    

    Ok(())
}

fn initialise_processors(
    mut wav_for_peak: WavAudioContext, 
    spec: hound::WavSpec, 
    channels: u16, 
    smpl_rng: SampleRange, 
    total_frames: u32,
) -> Result<(
        Delay, 
        Reverb, 
        Compressor, 
        Gain, 
        SoftClipper, 
        HardClipper, 
        Fader
    ), Box<dyn std::error::Error>> {
        let mut fader = Fader::new(
            1.0, 
            0.0, 
            total_frames, 
            channels);
        let comp = Compressor::new(
            smpl_rng,
            -40.0,
            20.0,
            10.0,
            10.0,
            1.0,
            spec.sample_rate as f64,
        );
        let peak = PeakDetect::new(&mut wav_for_peak)?;
        let normalised_gain = peak.normalised_gain(smpl_rng)?;
        let gain = Gain::new(
            normalised_gain, 
            smpl_rng);
        let hard_clipper = HardClipper::new(
            smpl_rng, 
            0.1
        )?;
        let soft_clip = SoftClipper::new(
            smpl_rng, 
            SoftClipperMode::Logarithmic, 
            0.3, 
            20.0, 
            1.0
        )?;
        let delay = Delay::new(
            [100.0, 300.0], 
            [0.85, 0.5], 
            [1.0, 1.0],
            spec.sample_rate as f64,
            smpl_rng,

        );
        let reverb: Reverb = Reverb::new(
            100.0, 
            50.0,
            [0.6, 0.6], 
            [0.5, 0.5],
            spec.sample_rate as f64,
        );
        Ok((delay, reverb, comp, gain, soft_clip, hard_clipper, fader))
}

fn process_chain(chain: &mut [Box<dyn Processor>], buffer: &mut [i32]) {
    for processor in chain {
        processor.process_buffer(buffer);
    }
}

fn block_peak(buffer: &[i32]) -> i32 {
    buffer
        .iter()
        .map(|sample| sample.saturating_abs())
        .max()
        .unwrap_or(0)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn processor_chain_runs_in_order() -> Result<(), Box<dyn std::error::Error>> {
        let range = SampleRange::new(16);
        let gain = Gain::new(2.0, range);
        let clip = HardClipper::new(range, 0.5)?;
        let comp = Compressor::new(range, -6.0, 20.0, 2.0, 2.0, 1.0, 44100.0);

        let mut chain: Vec<Box<dyn Processor>> = Vec::new();
        chain.push(Box::new(gain));
        chain.push(Box::new(clip));
        chain.push(Box::new(comp));

        let mut first_processed_sample = 25000;

        for processor in &mut chain {
            first_processed_sample = processor.process(first_processed_sample);
        }

        let clip = HardClipper::new(range, 0.5)?;
        let comp = Compressor::new(range, -6.0, 20.0, 2.0, 2.0, 1.0, 44100.0);

        chain.clear();
        chain.push(Box::new(clip));
        chain.push(Box::new(comp));
        chain.push(Box::new(gain));

        let mut second_processed_sample = 25000;

        for processor in &mut chain {
            second_processed_sample = processor.process(second_processed_sample);
        }
        assert_ne!(first_processed_sample, second_processed_sample);

        Ok(())
        // gain then clip should differ from clip then gain
    }

    #[test]
    fn stereo_compressor_applies_equal_gain_to_both_channels() {
        let range = SampleRange::new(16);

        let mut compressor = Compressor::new(range, -12.0, 4.0, 0.0, 0.0, 1.0, 44_100.0);

        let mut buffer = [20_000, 10_000];

        compressor.process_buffer(&mut buffer);

        let left_gain = buffer[0] as f64 / 20_000.0;

        let right_gain = buffer[1] as f64 / 10_000.0;

        assert!((left_gain - right_gain).abs() < 0.001);
    }
}
