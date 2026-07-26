mod wav;
mod gain;
mod modulator;
mod sample_range;
mod distortion;
mod compressor;
mod peak_detector;
mod utility;
mod processor;

use crate::gain::Gain;
use crate::peak_detector::PeakDetect;
use crate::sample_range::SampleRange;
use crate::distortion::{HardClipper, SoftClipper, SoftClipperMode};
use crate::wav::Wav;
use crate::modulator::Fader;
use crate::compressor::Compressor;
use crate::processor::Processor;

fn main() -> Result<(), Box<dyn std::error::Error>> {

    const INPUT_PATH: &str = "audio/input.wav";
    const OUTPUT_PATH: &str = "audio/output1.wav";
    const BLOCK_SIZE: usize = 256;

    let mut wav = Wav::new(INPUT_PATH)?;
    // Peak analysis is a full read pass; use a separate reader for it.
    let mut wav_for_peak = Wav::new(
        INPUT_PATH)?;
    let spec = wav.spec();
    let channels = wav.channels();

    let bits = wav.bits_per_sample();
    let mut output = hound::WavWriter::create(
        OUTPUT_PATH, 
        spec)?;

    let smpl_rng = SampleRange::new(
        bits);

    let total_frames: u32 = wav
        .audio()
        .duration();

    
    let mut fader = Fader::new(
        1.0, 
        0.0, 
        total_frames, channels);

    let comp = Compressor::new(
        smpl_rng, 
        -30.0, 
        15.0, 
        10.0, 
        1.0, 
        spec.sample_rate as f64 
    );

    let peak = PeakDetect::new(
        &mut wav_for_peak)?;

    let normalised_gain = peak.normalised_gain(
        smpl_rng
    )?;

    let gain_processor = Gain::new(
        normalised_gain, 
        smpl_rng);

    let hard_clipper = HardClipper::new(
        smpl_rng, 
        0.1
    )?;

    let sclipper = SoftClipper::new(
        smpl_rng,
        SoftClipperMode::Logarithmic,
        0.3,
        20.0,
        1.0,
    )?;

    let mut chain: Vec<Box<dyn Processor>> = vec![
    Box::new(gain_processor),
    // Box::new(sclipper),
    Box::new(comp),
    ];

    let mut buffer: Vec<i32> = Vec::with_capacity(BLOCK_SIZE);
    let mut blocks = 0;
    
    for sample in wav.audio().samples() {

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
        buffer.clear();
    }
    output.finalize()?;

    let mut slice = [0,1, 2, 3, 4, 5, 6, 7, 8, 9];

    let iter = slice.chunks_exact_mut(2);
    for e in iter{
        println!("{:?}", e);
    }
    

    Ok(())

}

fn process_chain(
    chain: &mut [Box<dyn Processor>],
    buffer: &mut [i32],
) {
    for processor in chain {
        processor.process_buffer(buffer);
    }
}


#[cfg(test)]
mod test{
    use super::*;


    #[test]
    fn processor_chain_runs_in_order() -> Result<(), Box<dyn std::error::Error>> {
        let range = SampleRange::new(16);
        let gain = Gain::new(2.0,range);
        let clip = HardClipper::new(range, 0.5)?;
        let comp = Compressor::new(range,-6.0, 20.0, 2.0, 2.0, 44100.0);

        let mut chain: Vec<Box<dyn Processor>> = Vec::new();
        chain.push(Box::new(gain));
        chain.push(Box::new(clip));
        chain.push(Box::new(comp));

        let mut first_processed_sample = 25000;

                    for processor in &mut chain {
                        first_processed_sample =
                            processor.process(first_processed_sample);
                    }
        
        let clip = HardClipper::new(range, 0.5)?;
        let comp = Compressor::new(range,-6.0, 20.0, 2.0, 2.0, 44100.0);

        chain.clear();
        chain.push(Box::new(clip));
        chain.push(Box::new(comp));
        chain.push(Box::new(gain));           

        let mut second_processed_sample = 25000;

                    for processor in &mut chain {
                        second_processed_sample =
                            processor.process(second_processed_sample);
                    }
        assert_ne!(first_processed_sample, second_processed_sample);

        Ok(())
        // gain then clip should differ from clip then gain
    }

}
