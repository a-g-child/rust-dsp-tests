use crate::{delay, processor::Processor, utility::mixer};
use std::{collections::VecDeque, sync::mpsc::channel};

struct CircularBuffer<T> {
    buffer: VecDeque<T>,
    capacity: usize,
}

impl<T> CircularBuffer<T> {
    fn new(capacity: usize) -> Self {
        Self {
            buffer: VecDeque::with_capacity(capacity),
            capacity,
        }
    }
    fn push(&mut self, item: T) {
        if self.capacity == 0 {
            return;
        }

        if self.buffer.len() == self.capacity {
            self.buffer.pop_front(); // Evict oldest item
        }
        self.buffer.push_back(item);
    }
    fn pop(&mut self) -> Option<T> {
        self.buffer.pop_front()
    }
    fn get(&self, index: usize) -> Option<&T>{
        self.buffer.get(index)
    }
    fn resize(&mut self, capacity: usize) {
        self.capacity = capacity;

        if self.buffer.len() > capacity {
            self.buffer.drain(..self.buffer.len() - capacity);
        }

        self.buffer
            .reserve(capacity.saturating_sub(self.buffer.len()));
    }
}

pub struct Delay {
    buffer: [CircularBuffer<f64>; 2],
    position: [usize; 2],
    frame_delay: [usize; 2],
    feedback: [f64; 2],
    decay: [f64; 2],
    mix: [f64; 2],
    sample_rate: f64,
    current_channel: usize,
}

impl Delay {
    pub fn new(
        time_ms: [f64; 2], 
        feedback: [f64; 2], 
        decay: [f64; 2],
        mix: [f64; 2], 
        sample_rate: f64) -> Self 
        {
        let t: [f64; 2]  = [time_ms[0].abs(), time_ms[1].abs()];
        let fb: [f64; 2] = [feedback[0].abs(), feedback[1].abs()];
        let  buf = [
            CircularBuffer::<f64>::new(((t[0] / 1000.0) * sample_rate * fb[0]).ceil() as usize),
            CircularBuffer::<f64>::new(((t[1] / 1000.0) * sample_rate * fb[1]).ceil() as usize),
        ];
        println!("frame_delay[L]: {}, frame_delay[R]: {}", (t[0] / 1000.0) * sample_rate , (t[1] / 1000.0) * sample_rate );
        Self {
            buffer: buf,
            position: [0,0],
            frame_delay: [ ((t[0] / 1000.0) * sample_rate) as usize , ((t[1] / 1000.0) * sample_rate ) as usize],
            feedback: fb,
            decay: [decay[0].clamp(0.0, 1.0),decay[1].clamp(0.0, 1.0)],
            mix: [mix[0].clamp(0.0, 1.0),mix[1].clamp(0.0, 1.0)],
            sample_rate,
            current_channel: 0,
        }
    }
    fn calculate_delayed_sample(&mut self, sample: i32) -> i32 {
        let channel = self.current_channel;
        let position = self.position[channel];
        let frame_delay = self.frame_delay[channel];
        let buffer = &mut self.buffer[channel];
        let delayed: i32;

        if position <= frame_delay { // buffer still filling
            return sample
        } else { // frame_delay is full
            if position <= buffer.capacity {
                // read sample from current position minus frame_delay
                let index = position - frame_delay;
                delayed = *buffer.get(index).expect("failed to pull sample during unfilled buffer") as i32;
            } else { // buffer completely full, feedback lookahead complete.
                let index = frame_delay;
                delayed = *buffer.get(index).expect("failed to pull sample during filled buffer") as i32;
                buffer.pop();
            }
        }
        return delayed;
    }

    pub fn apply(&mut self, sample: i32) -> i32 {
        let channel = self.current_channel;
        let mut delayed = self.calculate_delayed_sample(sample);
        
        self.position[channel] += 1; // advance position
        
        delayed = (delayed as f64 * self.decay[channel]) as i32;
        let mixed = mixer(delayed, sample, self.mix[channel]);
        self.buffer[channel].push(mixed as f64);
        mixed // mix delayed sample with current
    }
}
    // pub fn apply(&mut self, sample: i32) -> i32 {

    //     let channel: usize = self.current_channel;
    //     let mut delayed;    
        
    //     if self.position[channel] <= self.frame_delay[channel] { // buffer still filling

    //         delayed = sample;
    //     } else { // frame_delay is full // so can start using delayed samples

    //         if self.position[channel] <= self.buffer[channel].capacity as usize {
    //             // read sample from current position minus frame_delay
    //             // continue to add to buffer without consuming
    //             // println!("channel[{}], buffer cap: {}, index: {}, frame_delay: {}", channel, self.buffer[channel].capacity, self.position[channel] - self.frame_delay[channel], self.frame_delay[channel]);
    //             delayed = *self
    //             .buffer[channel]
    //             .get(
    //                 self.position[channel] - 
    //                 self.frame_delay[channel])
    //                     .expect("failed to pull sample during unfilled buffer") as i32;
                
    //         } else { // buffer completely full, feedback lookahead complete.

    //             delayed = *self
    //             .buffer[channel]
    //             .get(
    //                  self.frame_delay[channel])
    //                     .expect("failed to pull sample during filled buffer") as i32;

    //             self.buffer[channel].pop();
    //         }
    //     }
        
    //     self.position[channel] += 1; // advance position
    //     delayed = (delayed as f64 * self.decay[channel]) as i32;
    //     let mixed = mixer(delayed, sample, self.mix[channel]);
    //     self.buffer[channel].push(mixed as f64);
    //     mixed // mix delayed sample with current
    // }
// }

impl Processor for Delay {
    fn process(&mut self, sample: i32) -> i32 {
        self.apply(sample)
    }
    fn process_buffer(&mut self, buffer: &mut [i32]) {
        for frame in buffer.chunks_exact_mut(2) {

            let left = frame[0];
            let right = frame[1];
            frame[0] = self.process(left);
            self.current_channel = 1;
            frame[1] = self.process(right);
            self.current_channel = 0;
        }
    }
}

