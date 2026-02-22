use core::f32;

const PI: f32 = f32::consts::PI;

pub fn note(n: f32) -> f32 {
    440. * 2.0_f32.powf((n - 69.) / 12.)
}

pub const fn delta(freq: f32, sample_rate: u32) -> f32 {
    2. * PI * freq / sample_rate as f32
}

pub fn notes(sample_rate: u32) -> Vec<Vec<f32>> {
    println!("synth::notes({sample_rate})");
    let mut delta_angles = Vec::<Vec<f32>>::new();
    delta_angles.resize(154, Vec::<f32>::new());
    for n in 1..=153 {
        let idx = n as usize;
        let freq = note(n as f32);
        let freq_delta_angle = delta(freq, sample_rate);
        delta_angles[idx].extend([freq_delta_angle]);
    }

    delta_angles
}

pub fn f32_to_u32(n: f32) -> u32 {
    n.to_bits()
}
pub fn u32_to_f32(n: u32) -> f32 {
    f32::from_bits(n)
}
