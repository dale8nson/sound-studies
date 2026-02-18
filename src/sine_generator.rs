use std::{
    f32,
    ops::Add,
    sync::{Arc, LazyLock, Mutex, MutexGuard},
};

use bit_set::BitSet;
use cpal::{
    Host, SupportedStreamConfig,
    traits::{DeviceTrait, HostTrait},
};
use ndarray::ShapeBuilder;
use std::f32::consts::PI;

static HOST: LazyLock<Host> = std::sync::LazyLock::new(|| cpal::default_host());
pub static OUTPUT_DEVICE: LazyLock<cpal::Device> =
    std::sync::LazyLock::new(|| HOST.default_output_device().unwrap());
pub static STREAM_CONFIG: LazyLock<SupportedStreamConfig> =
    std::sync::LazyLock::new(|| OUTPUT_DEVICE.default_output_config().unwrap());
pub static MIDI: LazyLock<Vec<Vec<f32>>> =
    std::sync::LazyLock::new(|| notes(STREAM_CONFIG.sample_rate()));

#[derive(Debug, Clone)]
pub struct SineGenerator {
    note_mask: BitSet,
    frequencies: Vec<Vec<f32>>,
    phases: Vec<Vec<f32>>,
    sample_rate: u32,
    channels: u16,
    delta_angles: Vec<Vec<f32>>,
    volume: f32,
}

impl SineGenerator {
    pub fn new(frequencies: Vec<Vec<f32>>, config: SupportedStreamConfig) -> Self {
        let mut phases = Vec::<Vec<f32>>::new();
        phases.resize(frequencies.len(), Vec::<f32>::new());
        let sample_rate = config.sample_rate();
        let channels = config.channels();
        let delta_angles: Vec<Vec<f32>> = MIDI.clone();

        // frequencies
        //   .iter()
        //   .map(|freqs| {
        //       freqs
        //           .into_iter()
        //           .map(|freq| 2. * PI * *freq / sample_rate as f32)
        //           .collect::<Vec<f32>>()
        //   })
        //   .collect();

        let volume = 0.5;
        let note_mask = BitSet::new();

        Self {
            note_mask,
            frequencies,
            phases,
            sample_rate,
            channels,
            delta_angles,
            volume,
        }
    }

    // pub fn freq(&mut self, freq: f32) {
    //     self.phases.push(0.0);
    //     self.delta_angles
    //         .push(2. * PI * freq / self.sample_rate as f32);
    //     self.frequencies.push(freq);
    // }

    pub fn note(&mut self, n: u8, velocity: u8) {
        let idx = n as usize;

        // println!("freq: {freq}");
        if velocity > 0 {
            let _ = self.note_mask.insert(idx);
        } else {
            let _ = self.note_mask.remove(idx);
        }
    }

    pub fn update_volume(&mut self, volume: u8) {
        self.volume = volume as f32 / 127.;
    }

    pub fn volume(&self) -> f32 {
        self.volume
    }

    pub fn partial(&mut self, n: u8, freq: f32, partial: usize, amplitude: f32) {
        let idx = n as usize;
        let mut ks = Vec::<f32>::new();
        ks.resize(partial, 0.0);
        let t = 2. * PI * freq / self.sample_rate as f32;
        let f: f32 = ks
            .iter()
            .enumerate()
            .map(|(k, _)| amplitude * f32::cos(2. * PI * k as f32 * freq * t + amplitude.abs()))
            .sum();

        // println!("partial {partial} for frequency {freq}: {f}");
        self.frequencies[idx].push(freq + f);
    }

    pub fn build(self) -> SineGeneratorBuilder {
        SineGeneratorBuilder(self)
    }

    pub fn default(config: SupportedStreamConfig) -> Self {
        let mut frequencies = Vec::<Vec<f32>>::new();
        frequencies.resize(153, Vec::<f32>::new());
        let mut phases = Vec::<Vec<f32>>::new();

        phases.resize(153, Vec::<f32>::new());
        phases.iter_mut().enumerate().for_each(|(idx, phase)| {
            phase.resize(MIDI[idx].len(), 0.0);
        });
        let sample_rate = config.sample_rate();
        let channels = config.channels();
        let delta_angles = MIDI.clone();
        // delta_angles.resize(153, Vec::<f32>::new());
        // delta_angles
        //     .iter_mut()
        //     .enumerate()
        //     .for_each(|(idx, angle)| {
        //         angle.resize(MIDI[idx].len(), 0.0);
        //     });
        // println!("delta_angles.len(): {}", delta_angles.len());
        let volume = 0.5;
        let note_mask = BitSet::new();

        Self {
            note_mask,
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
            .note_mask
            .iter()
            .map(|idx| {
                let mut phase = self.phases[idx].iter_mut();
                let delta_angles = self.delta_angles[idx].iter_mut();

                let next_phase = phase.zip(delta_angles).scan(0.0, |_state, (p, a)| {
                    *p += *a;
                    if *p > 2. * PI {
                        *p -= 2. * PI;
                    }
                    Some(p)
                });

                next_phase.fold(0.0, |acc, p| acc + f32::sin(*p))
            })
            .sum();
        if sin > 0. {
            // println!("sin: {sin}");
        }
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
    // pub fn freq(mut self, freq: f32) -> Self {
    //     self.0.freq(freq);
    //     self
    // }

    pub fn partial(mut self, n: u8, freq: f32, partial: usize, amplitude: f32) -> Self {
        self.0.partial(n, freq, partial, amplitude);
        self
    }

    pub fn finish(self) -> SineGenerator {
        self.0
    }
}

pub fn note(n: f32) -> f32 {
    440. * 2.0_f32.powf((n - 69.) / 12.)
}

pub fn partial(freq: f32, partial: usize, sample_rate: u32, amplitude: f32) -> f32 {
    let mut ks = Vec::<f32>::new();
    ks.resize(partial, 0.0);
    let t = 2. * PI * freq / sample_rate as f32;
    let f: f32 = ks
        .iter()
        .enumerate()
        .map(|(k, _)| amplitude * f32::cos(2. * PI * k as f32 * freq * t + amplitude.abs()))
        .sum();

    f
}

pub fn delta(freq: f32, sample_rate: u32) -> f32 {
    2. * PI * freq / sample_rate as f32
}

pub fn partial_delta(freq: f32, part: usize, sample_rate: u32, amplitude: f32) -> f32 {
    delta(partial(freq, part, sample_rate, amplitude), sample_rate)
}

pub fn notes(sample_rate: u32) -> Vec<Vec<f32>> {
    // let mut frequencies = Vec::<Vec<f32>>::new();
    // frequencies.resize(154, Vec::<f32>::new());
    let mut delta_angles = Vec::<Vec<f32>>::new();
    delta_angles.resize(154, Vec::<f32>::new());
    for n in 1..=153 {
        let idx = n as usize;
        let freq = note(n as f32);
        // let part_1 = delta(freq + 0.002, sample_rate);
        // let part_2 = delta(freq - 0.002, sample_rate);
        // let part_1 = partial_delta(freq, 2, sample_rate, 0.25);
        // let part_2 = partial_delta(freq, 3, sample_rate, 0.25);
        // let part_3 = partial_delta(freq, 4, sample_rate, 0.1);
        // let part_4 = partial_delta(freq, 5, sample_rate, 0.1);
        let freq_delta_angle = delta(freq, sample_rate);
        // let part_delta_angle = 2. * PI * part_1 / sample_rate as f32;
        delta_angles[idx].extend([freq_delta_angle]);
        // delta_angles[idx].extend([freq_delta_angle]);
    }

    delta_angles
}
