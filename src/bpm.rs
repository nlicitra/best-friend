#![allow(unused, dead_code)]
use super::utils::{mean, median};
use std::fmt;

const BUFFER_SIZE: usize = 512;
const BUFFERS_PER_FRAME: usize = 4;
const FRAME_SIZE: usize = BUFFER_SIZE * BUFFERS_PER_FRAME;

#[derive(Clone, Copy)]
struct FrameSlice([f32; BUFFER_SIZE]);

impl FrameSlice {
    fn new() -> Self {
        Self([0.; BUFFER_SIZE])
    }
    fn from_array(data: [f32; BUFFER_SIZE]) -> Self {
        Self(data)
    }
    fn buffer(&self) -> &[f32; BUFFER_SIZE] {
        &self.0
    }
    fn release_buffer(&self) -> [f32; BUFFER_SIZE] {
        self.0
    }
}

#[derive(Clone)]
struct Frame {
    samples: Vec<FrameSlice>,
}

impl Frame {
    fn new() -> Self {
        let mut samples = vec![];
        for i in 0..BUFFERS_PER_FRAME {
            samples.push((FrameSlice::new()));
        }
        Self { samples }
    }

    fn write(&mut self, sample: [f32; BUFFER_SIZE]) -> [f32; BUFFER_SIZE] {
        self.samples.push(FrameSlice::from_array(sample));
        let old_slice = self.samples.remove(0);
        old_slice.release_buffer()
    }

    fn buffer(&self) -> [f32; FRAME_SIZE] {
        let mut buffer = [0.; FRAME_SIZE];
        for (i, sample) in self.samples.iter().enumerate() {
            let start = i * BUFFER_SIZE;
            let mut slice = &mut buffer[start..(start + BUFFER_SIZE)];
            slice.copy_from_slice(sample.buffer());
        }
        buffer
    }

    fn energy(&self) -> f32 {
        self.buffer().iter().fold(0., |acc, x| acc + x * x)
    }
}

impl fmt::Display for Frame {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let items = self.buffer()[0..4].to_vec();
        write!(
            f,
            "[{},{},{},{}...]",
            items[0], items[1], items[2], items[3]
        )
    }
}

#[derive(PartialEq, Debug)]
enum OnsetDetectionMode {
    Energy,
    SpectralDifference,
}

pub struct FrameProcessor {
    mode: OnsetDetectionMode,
    frames: (Frame, Frame),
    history: Vec<f32>,
    threshold: f32,
    highest_peak: f32,
}

struct ThresholdParams {
    lambda: f32,
    alpha: f32,
    m: usize,
    hp_weight: f32,
}

impl FrameProcessor {
    pub fn new() -> Self {
        Self {
            mode: OnsetDetectionMode::Energy,
            frames: (Frame::new(), Frame::new()),
            history: vec![],
            threshold: 0f32,
            highest_peak: 0f32,
        }
    }

    fn write(&mut self, buffer: [f32; BUFFER_SIZE]) {
        let carry_over = self.frames.1.write(buffer);
        self.frames.0.write(carry_over);
    }

    fn update_history(&mut self, value: f32) {
        self.history.insert(0, value);
    }

    fn calculate_threshold(&mut self) -> f32 {
        // σn = λ × median(O[nm]) + α × mean(O[nm]) + N
        let params = ThresholdParams {
            lambda: 1.0,
            alpha: 0.7,
            m: 10,
            hp_weight: 0.05,
        };
        let ThresholdParams {
            lambda,
            alpha,
            m,
            hp_weight,
        } = params;
        let weighted_highest_peak = self.highest_peak * hp_weight;
        let prev_values = &self.history[0..m];
        self.threshold =
            lambda * median(prev_values) + alpha * mean(prev_values) + weighted_highest_peak;
        self.threshold
    }

    fn check_for_previous_onset(&mut self) -> bool {
        let (curr, prev, prev_prev) = match self.history[0..3] {
            [a, b, c] => (a, b, c),
            _ => (0., 0., 0.),
        };
        if prev > curr && prev > prev_prev {
            if prev > self.threshold {
                self.highest_peak = match (prev > self.highest_peak) {
                    true => prev,
                    false => self.highest_peak,
                };
                return true;
            }
        }
        false
    }

    pub fn process(&mut self, buffer: [f32; BUFFER_SIZE]) -> bool {
        self.write(buffer);

        let (prev, curr) = &self.frames;
        let odf = (curr.energy() - prev.energy()).abs();

        self.update_history(odf);
        self.calculate_threshold();
        self.check_for_previous_onset()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_frame_write() {
        let mut frame = Frame::new();
        let buffer = frame.buffer();
        assert_eq!(buffer.len(), FRAME_SIZE);
        assert_eq!(buffer[0], 0.);
        frame.write([1.; BUFFER_SIZE]);
        let buffer = frame.buffer();
        assert_eq!(buffer[0], 0.);
        assert_eq!(buffer[FRAME_SIZE - 1], 1.);
    }

    #[test]
    fn test_frame_processor_write() {
        let mut processor = FrameProcessor::new();
        for i in 0..10 {
            processor.write([i as f32; BUFFER_SIZE]);
        }
        assert_eq!(processor.mode, OnsetDetectionMode::Energy);
        assert_eq!(processor.frames.0.buffer()[0], 2.);
        assert_eq!(processor.frames.1.buffer()[FRAME_SIZE - 1], 9.);
    }
}
