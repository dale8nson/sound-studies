mod msg;
mod player;
mod synth;
mod track;
mod utils;

use msg::Msg::*;
use ndarray::{Array1, Ix1, array};
use std::{
    sync::{Arc, Mutex},
    thread,
    time::{Duration, Instant},
};
use synth::Synth;

use crate::player::Player;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let synth = Synth::default();
    let player = Player::new();
    let handle = synth.connect(player);
    let durations: Array1<u64> = array![7, 5, 3, 1];
    let pitches: Array1<u8> = array![60, 67, 68, 67];
    handle.send(SetVolume(0.0025))?;
    handle.send(Play)?;
    let duration_sum = durations.sum();

    durations.iter().enumerate().for_each(|(idx, duration_1)| {
        durations.iter().enumerate().for_each(|(idx2, duration_2)| {
            let p1 = pitches[idx] + (idx % 2) as u8 * (12 / 1 + idx % 3) as u8;
            let d1 = *duration_2 as f32 / duration_sum as f32 * *duration_1 as f32;
            let p2 = pitches[idx] - (idx % 3) as u8 * (12 / 1 + idx % 3) as u8;
            handle.send(NoteOn(p1)).unwrap();
            handle.send(NoteOn(p2)).unwrap();
            let now = Instant::now();
            while now.elapsed() < Duration::from_secs_f32(d1) {}
            handle.send(NoteOff(p1)).unwrap();
            handle.send(NoteOff(p2)).unwrap();
        });
    });

    handle.send(Stop)?;
    handle.send(Disconnect).unwrap();
    handle.join().unwrap();

    Ok(())
}
