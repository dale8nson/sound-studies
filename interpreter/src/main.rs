mod msg;
mod player;
mod synth;
mod track;
mod utils;

use msg::Msg::*;
use ndarray::{Array1, Ix1, array};
use std::{
    sync::{Arc, Mutex, mpsc::channel},
    thread,
    time::{Duration, Instant},
};
use synth::Synth;

use crate::player::Player;

static DURATIONS: [f32; 1] = [0.75];
static PITCHES: [u8; 11] = [57, 60, 64, 69, 72, 76, 69, 64, 60, 57, 74];

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let synth = Synth::default();
    let player = Player::new();
    let handle = synth.connect(player);

    handle.send(SetVolume(0.0025))?;
    handle.send(Play)?;

    let duration_sum: f32 = DURATIONS.iter().sum();
    let pitch_count = PITCHES.iter().len();

    let (tx, rx) = channel();
    let tx1 = tx.clone();
    let tx2 = tx.clone();
    let tx3 = tx.clone();

    let t1 = thread::spawn(move || {
        DURATIONS
                .iter()
                .rev()
                .cycle()
                .enumerate()
                .for_each(|(idx, duration_1)| {
                    let mut p_iter = PITCHES.iter().rev().cycle();
                    let mut p_iter_2 = PITCHES.iter().cycle();
                    let tx1 = tx1.clone();
                    DURATIONS.iter().cycle().take(idx).enumerate().for_each(
                        |(idx2, duration_2)| {
                            // let p1 = p_iter.next().unwrap() + (idx2 % 2) as u8 * (12 / 1 + idx % 3) as u8;
                            let p1 = *p_iter.next().unwrap() as u8 + ((idx % 2) * 7) as u8;
                            let p2 = *p_iter_2.next().unwrap() as u8 + ((idx % 2) * 7) as u8;
                            let d1 = *duration_2 as f32 / duration_sum as f32 * *duration_1 / (1. + (idx % 3) as f32 )
                            // + 2.
                            // + (f32::sin(idx2 as f32)) * 2.
                            ;
                            // + f32::abs(f32::sin(idx as f32 * 1.15));
                            tx1.send(NoteOn(p1)).unwrap();
                            tx1.send(NoteOn(p2)).unwrap();
                            let now = Instant::now();
                            while now.elapsed() < Duration::from_secs_f32(d1) {}
                            tx1.send(NoteOff(p1));
                            tx1.send(NoteOff(p2));
                        },
                    );
                });
    });
    let t2 = thread::spawn(move || {
        DURATIONS
            .iter()
            .cycle()
            .enumerate()
            .for_each(|(idx, duration_1)| {
                let mut p_iter = PITCHES.iter().rev().cycle();
                let mut p_iter_2 = PITCHES.iter().cycle();
                let tx2 = tx2.clone();
                DURATIONS
                    .iter()
                    .cycle()
                    .enumerate()
                    .for_each(|(idx2, duration_2)| {
                        // let p1 = p_iter.next().unwrap() + (idx2 % 2) as u8 * (12 / 1 + idx % 3) as u8;
                        let p1 = *p_iter.next().unwrap() as u8 - 12 + ((idx % 2) * 7) as u8;
                        let p2 = *p_iter_2.next().unwrap() as u8 - 24 + ((idx % 2) * 7) as u8;
                        let d1 = (
                            *duration_2 as f32 / duration_sum as f32 * *duration_1 / (1. + (idx % 3) as f32 )
                            // + (f32::cos(idx2 as f32)) * 2.
                        ) / 2.
                        // * ((idx2 % 3) as f32 / 4.)
                        ;
                        // + f32::abs(f32::sin(idx as f32 * 1.15));
                        tx2.send(NoteOn(p1)).unwrap();
                        tx2.send(NoteOn(p2)).unwrap();
                        let now = Instant::now();
                        while now.elapsed() < Duration::from_secs_f32(d1) {}
                        tx2.send(NoteOff(p1));
                        tx2.send(NoteOff(p2));
                    });
            });
    });

    let t3 = thread::spawn(move || {
        tx3.send(NoteOn(37)).unwrap();
        tx3.send(NoteOn(32)).unwrap();
        tx3.send(NoteOn(25)).unwrap();
        loop {}
    });

    loop {
        if let Ok(msg) = rx.try_recv() {
            handle.send(msg).unwrap();
        }

        if t1.is_finished() && t2.is_finished() {
            break;
        }
    }

    //     durations
    //         .iter()
    //         .rev()
    //         .enumerate()
    //         .for_each(|(idx2, duration_2)| {
    //             // let p1 = pitches[idx2];

    //             let p2 =
    //                 p_iter_2.next().unwrap() - (1 + (idx) % 3) as u8 * (12 / 1 + idx2 % 3) as u8;
    //             let d1 = *duration_2 as f32 / duration_sum as f32 * *duration_1 as f32;
    //             let t1 = thread::spawn(move || );

    //             let now = Instant::now();
    //             while now.elapsed() < Duration::from_secs_f32(d1) {}
    //             // handle.send(NoteOff(p1)).unwrap();
    //             // handle.send(NoteOff(p2)).unwrap();
    //         });
    // });

    // durations
    //     .iter()
    //     .rev()
    //     .enumerate()
    //     .for_each(|(idx, duration_1)| {
    //         durations.iter().enumerate().for_each(|(idx2, duration_2)| {
    //             // let p1 = pitches[idx2];

    //             let p1 = p_iter_1.next().unwrap() + (idx % 2) as u8 * (12 / 1 + idx2 % 3) as u8;
    //             let d1 = *duration_2 as f32 / duration_sum as f32 * *duration_1 as f32;
    //             let p2 =
    //                 p_iter_2.next().unwrap() - (1 + (idx) % 3) as u8 * (12 / 1 + idx2 % 3) as u8;
    //             // handle.send(NoteOn(p1)).unwrap();
    //             // handle.send(NoteOn(p2)).unwrap();
    //             handle.send(NoteOff(p1)).unwrap();
    //             handle.send(NoteOff(p2)).unwrap();
    //             let now = Instant::now();
    //             while now.elapsed() < Duration::from_secs_f32(d1) {}
    //         });
    //     });
    let now = Instant::now();
    while now.elapsed() < Duration::from_secs(2) {}
    handle.send(Stop)?;
    handle.send(Disconnect).unwrap();
    handle.join().unwrap();

    Ok(())
}
