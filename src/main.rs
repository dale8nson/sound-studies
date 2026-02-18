#![allow(unsafe_code)]
mod midi_event_handler;
mod sine_generator;

use std::{
    cell::RefCell,
    ffi::c_str,
    fs::File,
    io::{Read, Seek, Write},
    ops::Deref,
    os::fd::{AsRawFd, FromRawFd},
    rc::Rc,
    sync::{Arc, LazyLock, Mutex, RwLock, mpsc},
    thread,
    time::Duration,
};

use cpal::{
    Host, SupportedBufferSize, SupportedStreamConfig, SupportedStreamConfigRange,
    traits::{DeviceTrait, HostTrait, StreamTrait},
};
use macros::midi;
use rusb::{
    Context, Device, EndpointDescriptor, UsbContext,
    ffi::{
        libusb_context, libusb_device, libusb_device_descriptor, libusb_device_handle,
        libusb_get_descriptor, libusb_get_device_descriptor, libusb_get_pollfds, libusb_pollfd,
    },
};
use sine_generator::{OUTPUT_DEVICE, STREAM_CONFIG, SineGenerator, note};

use libc::{ECHO, ICANON, STDERR_FILENO, TCSANOW, c_char, getchar, poll, pollfd};

// midi!();
//

#[inline(always)]
fn note_on(synth: Arc<RwLock<SineGenerator>>, n: u8, velocity: u8) {
    let mut guard = synth.write().unwrap();
    guard.note(n, velocity);
}

#[inline(always)]
fn vol(synth: Arc<RwLock<SineGenerator>>, volume: u8) {
    let mut guard = synth.write().unwrap();
    guard.update_volume(volume);
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let buf_sz = STREAM_CONFIG.buffer_size();
    if let SupportedBufferSize::Range { min, max } = buf_sz {
        println!("buffer size range: min: {min} max: {max}");
    }

    let timeout = std::time::Duration::from_secs(10);

    let keyboard_device = rusb::devices()?
        .iter()
        .find(|device| {
            let dev = device.open().unwrap();
            let dd = device.device_descriptor().unwrap();
            let language = dev.read_languages(timeout).unwrap()[0];
            let product = dev.read_product_string(language, &dd, timeout).unwrap();
            product.as_str() == "USB Keystation 49e"
        })
        .unwrap();
    let interfaces = keyboard_device
        .active_config_descriptor()?
        .interfaces()
        .for_each(|interface| {
            interface.descriptors().for_each(|desc| {
                println!(
                    "iface: {}, endpoints: {}, endpoint descriptors: {:#?}",
                    desc.interface_number(),
                    desc.num_endpoints(),
                    desc.endpoint_descriptors()
                        .map(|ed| ed)
                        .collect::<Vec<EndpointDescriptor>>(),
                );
            })
        });

    println!("interfaces: {:#?}", interfaces);

    println!("\n\n\n{:#?}", OUTPUT_DEVICE.default_output_config());
    let A4 = note(49.);
    let E3 = note(44.);
    let C4 = note(52.);
    println!("A4: {A4}");

    let sound = Arc::new(RwLock::new(SineGenerator::default(STREAM_CONFIG.clone())));
    // .build()
    // .freq(A4)
    // .partial(A4, 2, 10.)
    // .freq(E3)
    // .freq(C4)
    // .partial(E3, 2, 5.)
    // .partial(A4, 0, 10.)
    // .partial(220., 2, 10.)
    // .partial(220., 3, 10.)
    // .partial(220., 4, 10.)
    // .partial(220., 5, 10.)
    // .partial(220., 6, 10.)
    // .partial(220., 8, 10.)
    // .partial(15., 1, 50.)
    // .partial(220., 11, 10.)
    // .freq(220.)
    // .partial(220., 2, 20.)
    // .partial(220., 3, 10.)
    // .partial(220., 4, 10.)
    // .partial(220., 5, 10.)
    // .finish();
    // let sound_clone = sound;

    let sound_iter = sound.clone();
    let os = OUTPUT_DEVICE.build_output_stream::<f32, _, _>(
        &OUTPUT_DEVICE.default_output_config()?.config(),
        move |data, _cb_info| {
            data.into_iter().for_each(|s| {
                let mut guard = sound_iter.write().unwrap();
                let vol = guard.volume();
                *s = guard.next().unwrap() * vol;
            });
        },
        |e| {
            println!("{e}");
        },
        None,
    )?;

    os.pause()?;

    let mut stdin = unsafe { File::from_raw_fd(libc::STDIN_FILENO) };

    let mut termios: libc::termios = unsafe { std::mem::zeroed() };

    unsafe {
        libc::tcgetattr(libc::STDIN_FILENO, &mut termios);
    }

    let mut original_termios = termios.clone();

    termios.c_lflag &= !(ICANON | ECHO);

    unsafe {
        let _ = libc::tcsetattr(STDERR_FILENO, TCSANOW, &mut termios);
    }

    let (tx, rx) = mpsc::channel::<Vec<u8>>();

    let thread_handle = thread::spawn(move || {
        let keyboard = keyboard_device.open().unwrap();
        keyboard.claim_interface(1).unwrap();
        loop {
            let mut buf = [0_u8; 64];
            if let Ok(size) =
                keyboard.read_interrupt(129_u8, &mut buf.as_mut_slice(), Duration::from_millis(100))
            {
                println!(
                    "{}\t",
                    buf.iter()
                        .map(|b| format!("{b:02x}"))
                        .collect::<Vec<String>>()
                        .join(" ")
                );

                tx.send(buf[..size].to_vec()).unwrap();
            }
        }
    });

    let (ktx, krx) = mpsc::channel::<char>();

    let key_event_handle = thread::spawn(move || {
        loop {
            let key = unsafe { getchar() };
            let key = char::from_u32(key as u32).unwrap();

            println!("{key:?}");

            ktx.send(key).unwrap();
        }
    });

    os.play()?;

    let end_time = std::time::Instant::now() + Duration::from_secs(10);

    loop {
        if let Ok(buf) = rx.try_recv() {
            println!("buf: {buf:?}");

            match buf[0] {
                0xb => {
                    println!("vol: {}", buf[3]);
                    let v = buf[3];
                    vol(sound.clone(), v);
                }
                0x9 => {
                    let n = buf[2];
                    let velocity = buf[3];
                    println!("note: {}  velocity: {}", n, velocity);
                    note_on(sound.clone(), n, velocity);
                }
                _ => (),
            }
        }

        if let Ok(key) = krx.try_recv() {
            match key {
                'p' => {
                    println!("play");
                    os.play().unwrap()
                }
                's' => {
                    println!("stop");
                    os.pause().unwrap()
                }
                'q' => break,
                _ => {
                    // input.clear();
                    stdin.flush()?;
                    continue;
                }
            }
        }

        thread::sleep(Duration::from_millis(1));
    }

    stdin.flush()?;
    unsafe {
        let _ = libc::tcsetattr(STDERR_FILENO, TCSANOW, &mut original_termios);
    }

    thread_handle.join().unwrap();
    key_event_handle.join().unwrap();
    Ok(())
}

// inputs:
// σ = sample rate
// c = channels
// f = frequency
// β = buffer size
// j = buffer index
// i = frame count
// Φ = frame size (samples per frame)
//
// λ = samples per cycle
// ν = frames per cycle
// n = sample index
//
// output: θ = angle to compute sine of for given frame count and frame index
//
// Φ = β / c
// ν = σ / Φ
// λ = σ / f
// ν = λ / Φ
// n = i * Φ + j
// rads per sample = 2π / λ
// θ  = n * 2π / λ
//    = (i * Φ + j) * (2π / λ)
//    = ((i * (β / c)) + j) * (2π / (σ / f))
//
// samples per cycle = 44100 / 440
