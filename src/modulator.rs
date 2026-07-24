pub struct Fader {
    start: f64,
    end: f64,
    total_frames: u32,
    channels: u16,
    current_frame: u32,
    current_channel: u16,
}

impl Fader {

    pub fn new(start: f64, end: f64, total_frames: u32, channels: u16) -> Self {
        
        Fader { 
            start, 
            end, 
            total_frames, 
            channels,
            current_frame: 0, 
            current_channel: 0, 
            }
    }

    pub fn next_gain(&mut self) -> f64 {
        let gain = self.calculate_gain();
        self.advance_frame_and_channel();
        gain
    }

    fn calculate_gain(&self) -> f64 {
        let progress = (self.current_frame as f64 / self.total_frames as f64).min(1.0);
        self.start + (self.end - self.start) * (1.0 + 9.0 * progress).log10()
    }

    fn advance_frame_and_channel(&mut self) {
        self.current_channel += 1;

        if self.current_channel == self.channels {
            self.current_channel = 0;
            self.current_frame += 1;
        }
    }

}

