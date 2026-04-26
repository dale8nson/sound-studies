use crate::Player;
use crate::msg::{Msg, Msg::*};
use crate::utils::*;
use std::any::Any;
use std::cell::LazyCell;
use std::sync::Arc;
use std::sync::mpsc::{Receiver, SendError, Sender};
use std::thread;
use std::{
    sync::{LazyLock, atomic::AtomicU32, mpsc::channel},
    thread::JoinHandle,
};

use crate::player::STREAM_CONFIG;
use bit_set::BitSet;
use macros::keys;

static DELTA: LazyLock<Vec<Vec<f32>>> =
    std::sync::LazyLock::new(|| notes(STREAM_CONFIG.sample_rate()));

keys!();
use Key::*;
use ringbuf::StaticRb;
use ringbuf::traits::Split;
use ringbuf::traits::{Observer, Producer};

pub struct EngineHandle(JoinHandle<()>, Sender<Msg>);

impl EngineHandle {
    pub fn send(&self, msg: Msg) -> Result<(), SendError<Msg>> {
        self.1.send(msg)
    }

    pub fn join(self) -> Result<(), Box<dyn Any + Send + 'static>> {
        self.0.join()
    }
}

#[derive(Debug)]
pub struct Synth {
    note_mask: BitSet,
    volume: f32,
    phases: Vec<Vec<f32>>,
    key: Key,
}

impl Synth {
    pub fn volume(&self) -> f32 {
        println!("volume: {}", self.volume);
        self.volume
    }

    pub fn set_volume(&mut self, vol: f32) {
        self.volume = vol;
    }

    pub fn note_on(&mut self, n: u8) {
        println!("note {n} on");
        self.note_mask.insert(n as usize);
    }

    pub fn note_off(&mut self, n: u8) {
        self.note_mask.remove(n as usize);
    }

    pub fn connect(mut self, player: Player) -> EngineHandle {
        let (player_tx, player_rx) = channel();
        let (synth_tx, synth_rx) = channel();
        let buf = StaticRb::<f32, 8192>::default();
        let (mut prod, cons) = buf.split();

        player.connect(cons, player_rx);

        let handle = thread::spawn(move || {
            loop {
                if let Ok(msg) = synth_rx.try_recv() {
                    match msg {
                        NoteOff(n) => self.note_off(n),
                        NoteOn(n) => self.note_on(n),
                        Play => player_tx.send(Play).unwrap(),
                        Stop => player_tx.send(Stop).unwrap(),
                        Disconnect => {
                            player_tx.send(Disconnect).unwrap();
                            break;
                        }
                        _ => (),
                    }
                }
                // if prod.vacant_len() < 1024 {
                //     thread::yield_now();
                // } else {
                prod.push_iter(self.by_ref().take(1024));
                // }
            }
        });

        EngineHandle(handle, synth_tx)
    }
}

impl Default for Synth {
    fn default() -> Self {
        let note_mask = BitSet::new();
        let volume: f32 = 0.0;
        let mut phases = Vec::<Vec<f32>>::new();
        phases.resize_with(154, || {
            let mut v = Vec::<f32>::new();
            v.resize(1, 0.0);
            v
        });
        let key = CMaj;
        Self {
            note_mask,
            volume,
            phases,
            key,
        }
    }
}

impl Iterator for Synth {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        // println!("Synth::next()");
        // let mut phases = std::mem::take(&mut self.phases);

        let next = self.note_mask.iter().fold(0.0, |acc, n| {
            // println!("n: {n}");
            let next = self.phases[n]
                .iter_mut()
                .enumerate()
                .fold(0.0, |acc, (idx, m)| {
                    // println!("DELTA[{n}][{idx}]: {}", DELTA[n][idx]);
                    *m += DELTA[n][idx];
                    acc + f32::sin(*m)
                });

            acc + next
        });

        // println!("next: {next}");
        // self.phases = phases;
        Some(next)
    }
}
