mod wav;
mod gain;
mod modulator;
mod sample_range;
mod distortion;
mod compressor;
mod peak_detector;
mod utility;

use crate::gain::{Gain, GainProcessor};
use crate::peak_detector::PeakDetect;
use crate::sample_range::SampleRange;
use crate::distortion::HardClipper;
use crate::wav::Wav;
use crate::modulator::Fader;
use crate::compressor::Compressor;

fn main() -> Result<(), Box<dyn std::error::Error>> {

    const INPUT_PATH: &str = "audio/input.wav";
    const OUTPUT_PATH: &str = "audio/output1.wav";

    // println!("Enter Gain value (e.g., 0.5 for half volume, 2.0 for double volume):");
    
    // let mut input = String::new();
    // std::io::stdin().read_line(&mut input)?;
    // let gain: f64 = input.trim().parse()?;  

    let mut wav = Wav::new(INPUT_PATH)?;
    // Peak analysis is a full read pass; use a separate reader for it.
    let mut wav_for_peak = Wav::new(INPUT_PATH)?;
    let spec = wav.spec();
    let channels = wav.channels();

    let bits = wav.bits_per_sample();
    let output = hound::WavWriter::create(OUTPUT_PATH, spec)?;
    let smpl_rng = SampleRange::new(bits);


    let gain_processor = GainProcessor::new(smpl_rng);

    let mut fader = Fader::new(1.0, 0.0, wav.audio().len()/2, channels);

    let comp = Compressor::new(
        smpl_rng, 
        -20.1, 
        60.0, 
        10.0, 
        5.0, 
        spec.sample_rate as f64 
    );
    
    println!("Computing peak...");

    let peak = PeakDetect::new(&mut wav_for_peak)?;
    println!("peak: {}\nnormalized_gain: {}", peak.peak(), peak.normalised_gain(smpl_rng)?);
    let normalised_gain = Gain::new(peak.normalised_gain(smpl_rng)?);

    let hard_clipper = HardClipper::new(smpl_rng, 0.2)?;

    process_audio(wav, output, &normalised_gain, gain_processor, &mut fader, hard_clipper, comp)

}
    
fn process_audio(
    mut wav: Wav, 
    mut output: hound::WavWriter<std::io::BufWriter<std::fs::File>>,
    gain: &Gain, 
    gain_processor: GainProcessor,
    fader: &mut Fader,
    clipper: HardClipper,
    mut comp: Compressor,
) -> Result<(), Box<dyn std::error::Error>> {
        wav.audio()
            .samples::<i32>()
            .try_for_each(
                |sample
                | -> Result<(), Box<dyn std::error::Error>> {

                    let sample: i32 = sample?;
                    let f_gain: &Gain = &Gain::new(fader.next_gain());
                    let processed_sample: i32 = gain_processor.apply_gain(sample, gain);
                    let processed_sample: i32 = comp.apply(processed_sample);
                    let processed_sample: i32 = gain_processor.apply_gain(processed_sample, gain);
                    let processed_sample: i32 = gain_processor.apply_gain(processed_sample, f_gain);
                    let processed_sample: i32 = clipper.apply(processed_sample);


                    output.write_sample(processed_sample)?;
                    Ok(())
        })?;
        output.finalize()?;
        Ok(())
}
