mod wav;
mod gain;
mod modulator;
mod sample_range;
mod distortion;
mod compressor;
mod peak_detector;
mod utility;
mod processor;


// use audio_lab_dsp::distortion::SoftClipper;

use crate::gain::Gain;
use crate::peak_detector::PeakDetect;
use crate::sample_range::SampleRange;
use crate::distortion::{HardClipper, SoftClipper};
use crate::wav::Wav;
use crate::modulator::Fader;
use crate::compressor::Compressor;
use crate::processor::Processor;

fn main() -> Result<(), Box<dyn std::error::Error>> {

    const INPUT_PATH: &str = "audio/input.wav";
    const OUTPUT_PATH: &str = "audio/output1.wav";

    // println!("Enter Gain value (e.g., 0.5 for half volume, 2.0 for double volume):");
    
    // let mut input = String::new();
    // std::io::stdin().read_line(&mut input)?;
    // let gain: f64 = input.trim().parse()?;  

    let mut wav = Wav::new(INPUT_PATH)?;
    // Peak analysis is a full read pass; use a separate reader for it.
    let mut wav_for_peak = Wav::new(
        INPUT_PATH)?;
    let spec = wav.spec();
    let channels = wav.channels();

    let bits = wav.bits_per_sample();
    let output = hound::WavWriter::create(
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
        0.2,
        30.0,
        0.4,
        spec.sample_rate as f64,
    )?;

    process_audio(
        wav, 
        output, 
        normalised_gain, 
        gain_processor, 
        &mut fader, 
        hard_clipper, 
        comp,
        sclipper,
    )

}
    
fn process_audio(
    mut wav: Wav, 
    mut output: hound::WavWriter<std::io::BufWriter<std::fs::File>>,
    _gain: f64, 
    gain_processor: Gain,
    _fader: &mut Fader,
    clipper: HardClipper,
    comp: Compressor,
    sclip: SoftClipper,
) -> Result<(), Box<dyn std::error::Error>> {
        let mut chain: Vec<Box<dyn Processor>> = Vec::new();
        chain.push(Box::new(gain_processor));
        chain.push(Box::new(sclip));
        chain.push(Box::new(comp));

        wav.audio()
            .samples::<i32>()
            .try_for_each(
                |sample
                | -> Result<(), Box<dyn std::error::Error>> {

                    let mut processed_sample = sample?;

                    for processor in &mut chain {
                        processed_sample =
                            processor.process(processed_sample);
                    }

                    output.write_sample(
                        processed_sample
                    )?;
                    Ok(())
        })?;
        output.finalize()?;
        Ok(())
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