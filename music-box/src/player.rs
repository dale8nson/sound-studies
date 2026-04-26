#![allow(dead_code)]

use crate::{
    msg::Msg::{self, *},
    utils::{f32_to_u32, u32_to_f32},
};
use cpal::{
    Host, Stream, SupportedStreamConfig,
    traits::{DeviceTrait, HostTrait, StreamTrait},
};
use ringbuf::{
    SharedRb, StaticRb,
    storage::Owning,
    traits::{Consumer, Producer, Split},
};
use std::{
    mem::MaybeUninit,
    sync::{
        Arc, LazyLock, Mutex, RwLock,
        atomic::AtomicU32,
        mpsc::{Receiver, Sender},
    },
    thread::{self, JoinHandle},
};

static HOST: LazyLock<Host> = std::sync::LazyLock::new(|| cpal::default_host());
pub static OUTPUT_DEVICE: LazyLock<cpal::Device> =
    std::sync::LazyLock::new(|| HOST.default_output_device().unwrap());
pub static STREAM_CONFIG: LazyLock<SupportedStreamConfig> =
    std::sync::LazyLock::new(|| OUTPUT_DEVICE.default_output_config().unwrap());

pub struct Player {
    ostream: Option<Stream>,
}

impl Player {
    pub fn new() -> Self {
        println!("Player::new()");
        Self { ostream: None }
    }

    pub fn connect(
        mut self,
        mut cons: <SharedRb<Owning<[MaybeUninit<f32>; 8192]>> as ringbuf::traits::Split>::Cons,
        rx: Receiver<Msg>,
    ) -> JoinHandle<()> {
        let volume = Arc::new(AtomicU32::new(f32_to_u32(0.2)));
        let volume = volume.clone();
        let new_volume = volume.clone();
        self.ostream = Some(
            OUTPUT_DEVICE
                .build_output_stream::<f32, _, _>(
                    &OUTPUT_DEVICE.default_output_config().unwrap().config(),
                    move |data, _cb_info| {
                        let vol = u32_to_f32(volume.load(std::sync::atomic::Ordering::Relaxed));
                        let mut next = 0.0;
                        data.into_iter().enumerate().for_each(|(idx, s)| {
                            if idx % 2 == 0 {
                                next = cons.try_pop().unwrap_or(0.0);
                            }
                            *s = next * vol;
                        });
                    },
                    |e| {
                        println!("{e}");
                    },
                    None,
                )
                .unwrap(),
        );

        thread::spawn(move || {
            for msg in rx.into_iter() {
                match msg {
                    Play => self.play(),
                    Stop => self.stop(),
                    SetVolume(val) => {
                        new_volume.store(f32_to_u32(val), std::sync::atomic::Ordering::Relaxed)
                    }
                    Disconnect => break,
                    _ => continue,
                }
            }
        })
    }

    pub fn play(&mut self) {
        println!("Player::play()");
        self.ostream.as_ref().unwrap().play().unwrap();
    }

    pub fn stop(&mut self) {
        self.ostream.as_ref().unwrap().pause().unwrap();
    }
}
