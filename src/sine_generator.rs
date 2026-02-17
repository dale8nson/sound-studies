use std::{
    ops::Add,
    sync::{Mutex, MutexGuard},
};

use cpal::SupportedStreamConfig;
use std::f32::consts::PI;

#[derive(Debug, Clone)]
pub struct SineGenerator {
    frequencies: Vec<f32>,
    phases: Vec<f32>,
    sample_rate: u32,
    channels: u16,
    delta_angles: Vec<f32>,
    volume: f32,
}

impl SineGenerator {
    pub fn new(frequencies: Vec<f32>, config: SupportedStreamConfig) -> Self {
        let mut phases = Vec::<f32>::new();
        phases.resize(frequencies.len(), 0.0);
        let sample_rate = config.sample_rate();
        let channels = config.channels();
        let delta_angles: Vec<f32> = frequencies
            .iter()
            .map(|freq| 2. * PI * *freq / sample_rate as f32)
            .collect();

        let volume = 0.0;

        Self {
            frequencies,
            phases,
            sample_rate,
            channels,
            delta_angles,
            volume,
        }
    }

    pub fn freq(&mut self, freq: f32) {
        self.phases.push(0.0);
        self.delta_angles
            .push(2. * PI * freq / self.sample_rate as f32);
        self.frequencies.push(freq);
    }

    pub fn note(&mut self, n: u8, velocity: u8) {
        let idx = n as usize;

        let freq = note(n as f32);
        // println!("freq: {freq}");
        if velocity > 0 {
            self.frequencies[idx] = note(n as f32);
            self.delta_angles[idx] = 2. * PI * freq / self.sample_rate as f32;
            self.phases[idx] = 0.0;
        } else {
            self.frequencies[idx] = 0.0;
            self.delta_angles[idx] = 0.0;
            self.phases[idx] = 0.0;
        }
    }

    pub fn update_volume(&mut self, volume: u8) {
        self.volume = volume as f32 / 127.;
    }

    pub fn volume(&self) -> f32 {
        self.volume
    }

    pub fn partial(&mut self, freq: f32, partial: usize, amplitude: f32) {
        let mut ks = Vec::<f32>::new();
        ks.resize(partial, 0.0);
        let t = 2. * PI * freq / self.sample_rate as f32;
        let f: f32 = ks
            .iter()
            .enumerate()
            .map(|(k, _)| amplitude * f32::cos(2. * PI * k as f32 * freq * t + amplitude.abs()))
            .sum();

        // println!("partial {partial} for frequency {freq}: {f}");
        self.freq(freq + f);
    }

    pub fn build(self) -> SineGeneratorBuilder {
        SineGeneratorBuilder(self)
    }

    pub fn default(config: SupportedStreamConfig) -> Self {
        let mut frequencies = Vec::<f32>::new();
        frequencies.resize(89, 0.0);
        let mut phases = Vec::<f32>::new();
        phases.resize(89, 0.0);
        let sample_rate = config.sample_rate();
        let channels = config.channels();
        let mut delta_angles = Vec::<f32>::new();
        delta_angles.resize(89, 0.0);
        let volume = 0.0;

        Self {
            frequencies,
            phases,
            sample_rate,
            channels,
            delta_angles,
            volume,
        }
    }
}

impl Iterator for SineGenerator {
    type Item = f32;
    fn next(&mut self) -> Option<Self::Item> {
        let sin = self
            .phases
            .iter_mut()
            .enumerate()
            .fold(0.0, |acc, (idx, phase)| {
                let sin = f32::sin(*phase);
                *phase += self.delta_angles[idx];
                if *phase > 2. * PI {
                    *phase -= 2. * PI;
                }
                acc + sin
            });

        Some(sin)
    }
}

impl Add for SineGenerator {
    type Output = Self;
    fn add(mut self, rhs: Self) -> Self::Output {
        self.frequencies.extend(rhs.frequencies);
        self.phases.extend(rhs.phases);
        self.delta_angles.extend(rhs.delta_angles);
        self
    }
}

pub struct SineGeneratorBuilder(SineGenerator);

impl SineGeneratorBuilder {
    pub fn freq(mut self, freq: f32) -> Self {
        self.0.freq(freq);
        self
    }

    pub fn partial(mut self, freq: f32, partial: usize, amplitude: f32) -> Self {
        self.0.partial(freq, partial, amplitude);
        self
    }

    pub fn finish(self) -> SineGenerator {
        self.0
    }
}

pub fn note(n: f32) -> f32 {
    440. * 2.0_f32.powf((n - 69.) / 12.)
}

pub fn notes() -> Vec<f32> {
    let mut frequencies = Vec::<f32>::new();
    for n in 1..=88 {
        frequencies.push(note(n as f32));
    }
    frequencies
}
