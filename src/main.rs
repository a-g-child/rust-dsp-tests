mod compressor;
mod delay;
mod distortion;
mod gain;
mod modulator;
mod parameter;
mod peak_detector;
mod processor;
mod sample_range;
mod utility;
mod wav;
mod reverb;
mod wav_renderer;

use crate::wav_renderer::WavRenderer;
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

    let wav = 
        WavAudioContext::new(
            "audio/input.wav")?;

    let aux = 
        WavAudioContext::new(
            "audio/input.wav")?;

    let (delay, 
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

    let chain: Vec<Box<dyn Processor>> = vec![
        // Box::new(gain_processor),
        // Box::new(sclipper),
        Box::new(delay),
        Box::new(reverb),
        // Box::new(comp),
    ];

    let renderer = WavRenderer::new(wav, "audio/wav_render.wav", chain, BLOCK_SIZE)?;
    renderer.render()?;

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
            0.1,
            0.5
        )?;
        let soft_clip = SoftClipper::new(
            smpl_rng, 
            SoftClipperMode::Logarithmic, 
            0.3, 
            20.0, 
            1.0
        )?;
        let delay = Delay::new(
            vec![600.0, 500.0], 
            vec![0.85, 0.5], 
            vec![1.0, 1.0],
            spec.sample_rate as f64,
            smpl_rng,
            spec.channels as usize,

        );
        let reverb: Reverb = Reverb::new(
            200.0, 
            50.0,
            [0.6, 0.6], 
            [0.5, 0.5],
            spec.sample_rate as f64,
        );
        Ok((delay, reverb, comp, gain, soft_clip, hard_clipper, fader))
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn processor_chain_runs_in_order() -> Result<(), Box<dyn std::error::Error>> {
        let range = SampleRange::new(16);
        let gain = Gain::new(2.0, range);
        let clip = HardClipper::new(range, 0.5, 0.5)?;
        let comp = Compressor::new(range, -6.0, 20.0, 2.0, 2.0, 1.0, 44100.0);

        let mut chain: Vec<Box<dyn Processor>> = Vec::new();
        chain.push(Box::new(gain));
        chain.push(Box::new(clip));
        chain.push(Box::new(comp));

        let mut first_processed_sample = 25000;

        for processor in &mut chain {
            first_processed_sample = processor.process(first_processed_sample);
        }

        let clip = HardClipper::new(range, 0.5, 0.5)?;
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
