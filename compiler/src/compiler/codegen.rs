pub use midly::{
    Header, MetaMessage, MidiMessage, Smf, Track, TrackEvent, TrackEventKind,
    num::{u24, u28},
};

use crate::compiler::ast::*;

const FPS: u32 = 30;

#[derive(Debug, Clone, Copy)]
pub struct MicroSeconds(pub u64);
pub type Term = u64;
pub struct Velocity(u8);
pub type F<'a, Args, Ret> = Box<dyn FnMut(Args) -> Ret + 'a>;
pub const ID: fn() -> Box<fn(Exp) -> Exp> = || Box::new(move |exp| exp);

#[derive(Clone, Debug)]
pub enum ScopeType {
    Sequence,
    Stack,
    Set,
    None,
}

#[derive(Debug, Clone, Copy)]
pub enum Ctx {
    Idx(usize),
    None,
}

#[derive(Debug, Clone, Copy)]
pub struct Pc(pub u8);

#[derive(Debug, Clone, Copy)]
pub struct Upb(pub MicroSeconds);

#[derive(Debug, Clone)]
pub struct Instrument(pub Vec<u8>);

#[derive(Debug, Clone, Copy, Default)]
pub struct Tps(u64);

#[derive(Debug, Clone)]
pub enum Instruction<'a> {
    MidiMessage(MidiMessage),
    MetaMessage(MetaMessage<'a>),
}

pub mod utils {
    use crate::compiler::{
        ast::{utils::abs_to_f64, *},
        codegen::*,
    };

    pub fn to_length(frac: Absolute, tempo: Upb) -> MicroSeconds {
        let fr = abs_to_f64(frac);
        MicroSeconds(f64::round(fr / 4 as f64 * tempo.0.0 as f64) as u64)
    }

    pub fn duration_to_micros(minutes: Absolute, seconds: Absolute) -> MicroSeconds {
        MicroSeconds(f64::round(
            match minutes {
                Absolute::Integer(int) => (int * 60 * 1_000_000) as f64,
                Absolute::Float(float) => float * 1_000_000 as f64,
            } + match seconds {
                Absolute::Integer(int) => (int * 1_000_000) as f64,
                Absolute::Float(float) => float * 1_000_000 as f64,
            },
        ) as u64)
    }

    pub fn transform_exps(exps: Vec<Exp>, mut f: F<'static, Exp, Exp>) -> Vec<Exp> {
        exps.into_iter()
            .map(|exp| match exp {
                simple @ Exp::Simple(_) => f(simple),
                Exp::Compound(c) => match c {
                    compound @ Compound::Parens(exps)
                    | compound @ Compound::Braces(exps)
                    | compound @ Compound::Brackets(exps) => {
                        Exp::Compound(compound(transform_exps(exps, f)))
                    }
                    Compound::Ratio(abss) => {
                        Exp::Compound(Compound::Braces(transform_exps(exps, f)))
                    }
                },
                other => f(other),
            })
            .collect()
    }

    pub fn apply<T, U, V>(v1: Vec<T>, v2: Vec<U>, f: F<'static, (T, U), V>) -> Vec<V> {
        let mut iter_1 = v1.iter();
        let mut iter_2 = v2.iter().cycle();
        iter_1.zip(iter_2).cloned().map(|(item)| f(item)).collect()
    }
}
