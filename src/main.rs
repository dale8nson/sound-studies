#![allow(unsafe_code)]
mod sine_generator;
use std::{
    cell::RefCell,
    ffi::c_str,
    fs::File,
    io::{Read, Seek, Write},
    os::fd::{AsRawFd, FromRawFd},
    rc::Rc,
    sync::{Arc, Mutex, RwLock},
    time::Duration,
};

use cpal::{
    SupportedBufferSize, SupportedStreamConfigRange,
    traits::{DeviceTrait, HostTrait, StreamTrait},
};
use macros::midi;
use rusb::EndpointDescriptor;
use sine_generator::{SineGenerator, note};

use libc::{ECHO, ICANON, STDERR_FILENO, TCSANOW, c_char, getchar, poll, pollfd};

// midi!();

fn update_midi(synth: Arc<RwLock<SineGenerator>>, n: u8, velocity: u8) {
    let mut guard = synth.write().unwrap();
    guard.note(n, velocity);
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let host = cpal::default_host();
    let output_device = host.default_output_device().unwrap();

    //   host
    //     .devices()?
    //     .find(|device| device.supports_output())
    //     .unwrap();
    // println!(
    //     "{:#?}",
    //     output_device
    //         .supported_output_configs()?
    //         .collect::<Vec<SupportedStreamConfigRange>>()
    // );

    let stream_config = output_device.default_output_config()?;

    let buf_sz = stream_config.buffer_size();
    if let SupportedBufferSize::Range { min, max } = buf_sz {
        println!("buffer size range: min: {min} max: {max}");
    }

    let mut midi = Vec::<f32>::new();
    midi.resize(89, 0.0);
    for n in 1..=88 {
        midi[n] = note(n as f32);
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

    let keyboard = keyboard_device.open().unwrap();

    // keyboard.claim_interface(0)?;
    keyboard.claim_interface(1)?;
    let mut buf = Vec::<u8>::new();
    buf.resize(64, 0);

    //   for_each(|device| {
    //     if let Ok(dd) = device.device_descriptor() {
    //         let file_handler = device.open().unwrap();
    //         let languages = file_handler.read_languages(timeout).unwrap();
    //         let configuration = device.active_config_descriptor().unwrap();
    //         println!(
    //             "product id: {:#?}",
    //             file_handler.read_configuration_string(languages[0], &configuration, timeout)
    //         );

    //         println!(
    //             "product: {}",
    //             file_handler
    //                 .read_product_string(languages[0], &dd, std::time::Duration::from_secs(10))
    //                 .unwrap()
    //         );
    //     }

    //     // println!("{}, address: {}", device.device_descriptor(), device.address())}}
    // });

    println!("\n\n\n{:#?}", output_device.default_output_config());
    let A4 = note(49.);
    let E3 = note(44.);
    let C4 = note(52.);
    println!("A4: {A4}");

    let sound = Arc::new(RwLock::new(SineGenerator::default(stream_config)));
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
    let os = output_device.build_output_stream::<f32, _, _>(
        &output_device.default_output_config()?.config(),
        move |data, _cb_info| {
            data.into_iter().for_each(|s| {
                *s = sound_iter.write().unwrap().next().unwrap() * 0.0125;
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

    os.play()?;

    loop {
        keyboard.read_interrupt(129_u8, &mut buf.as_mut_slice(), Duration::from_millis(0))?;
        println!("note: {}  velocity: {}", buf[2], buf[3]);
        let n = buf[2];
        let velocity = buf[3];

        update_midi(sound.clone(), n, velocity);
        // std::io::stdin().read_exact(&mut input)?;
        // let res = unsafe { poll(&mut kbd, 1, 0) };
        // println!("errno: {res}");
        // if res < 0 {
        //     println!("errno: {res}");
        //     continue;
        // }

        // stdin.read_exact(&mut buf)?;
        // let value: u8 = input[0];

        // let key_value = value as u32
        //     | (input[1] as u32) << 8
        //     | (input[2] as u32) << 16
        //     | (input[3] as u32) << 24;

        // println!("{key_value:0x}");

        // let Some(key) = char::from_u32(key_value) else {
        //     continue;
        // };

        // let key = unsafe { getchar() };
        // let key = char::from_u32(key as u32).unwrap();

        // println!("{key:?}");

        // match key {
        //     'p' => {
        //         println!("play");
        //         os.play().unwrap()
        //     }
        //     's' => {
        //         println!("stop");
        //         os.pause().unwrap()
        //     }
        //     'q' => break,
        //     _ => {
        //         // input.clear();
        //         stdin.flush()?;
        //         continue;
        //     }
        // }
        // input.clear();
        // break;
    }
    #[allow(unreachable_code)]
    stdin.flush()?;
    unsafe {
        let _ = libc::tcsetattr(STDERR_FILENO, TCSANOW, &mut original_termios);
    }
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
