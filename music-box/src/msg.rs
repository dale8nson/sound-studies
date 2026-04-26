#[repr(u8)]
pub enum Msg {
    NoteOn(u8) = 0,
    NoteOff(u8),
    Play,
    Stop,
    SetVolume(f32),
    Sample(f32),
    Volume(f32),
    GetVolume,
    Disconnect,
}
