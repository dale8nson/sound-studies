use super::ast::*;
use midly::{
    Header, MetaMessage, MidiMessage, Smf, Track, TrackEvent, TrackEventKind,
    num::{u24, u28},
};
use std::ops::{Index, IndexMut};
const FPS: u32 = 30;

#[derive(Debug, Clone, Copy)]
struct MicroSeconds(u64);
type Term = u64;
struct Velocity(u8);
type F<'a, Args, Ret> = Box<dyn FnMut(Args) -> Ret + 'a>;
const ID: fn(Exp) -> Exp = move |exp| exp;

#[derive(Clone, Debug)]
enum ScopeType {
    Sequence,
    Stack,
    Set,
    None,
}

#[derive(Debug, Clone, Copy)]
enum Ctx {
    Idx(usize),
    None,
}

#[derive(Debug, Clone, Copy)]
struct Pc(pub i8);

#[derive(Debug, Clone, Copy)]
struct Upb(MicroSeconds);

#[derive(Debug, Clone)]
struct Instrument(pub Vec<u8>);

#[derive(Debug, Clone)]
enum Instruction<'a> {
    MidiMessage(MidiMessage),
    MetaMessage(MetaMessage<'a>),
}

#[derive(Debug, Default)]
pub struct Composer<'a> {
    idxs: Vec<Ctx>,
    parents: Vec<Ctx>,
    scope_types: Vec<ScopeType>,
    lengths: Vec<MicroSeconds>,
    pcs: Vec<Pc>,
    tempos: Vec<Upb>,
    bpms: Vec<Bpm>,
    registers: Vec<Integer>,
    velocities: Vec<Velocity>,
    instruments: Vec<Instrument>,
    children: Vec<Vec<Ctx>>,
    instructions: Vec<Vec<Instruction<'a>>>,
    events: Vec<Vec<TrackEvent<'a>>>,
}

impl<'a> Composer<'a> {
    pub fn compose(&mut self, mut ast: Program) -> Smf {
        let mut es = ast.exps.into_iter();
        let ctx = self.append_child(Ctx::None);
        self.tempo(
            ctx,
            Upb(MicroSeconds(
                f64::round(1_000_000 as f64 / 120 as f64) as u64
            )),
        );
        let mut f: F<Exp, Exp> = Box::new(ID);
        let f = compose_exps(es, f, ctx);
        f();
        todo!()
    }

    fn compose_exps(
        &mut self,
        exps: Vec<Exp>,
        f: F<'static, Exp, Exp>,
        ctx: Ctx,
    ) -> F<'static, Exp, Exp> {
        let mut es = exps.into_iter();

        loop {
            let exp = es.next();
            match exp {
                Some(exp) => f = compose_exp(exp, f, ctx),
                None => break,
            }
        }
        f
    }

    fn compose_exp(
        &mut self,
        node: Exp,
        mut f: F<'static, Exp, Exp>,
        ctx: Ctx,
    ) -> F<'static, Exp, Exp> {
        match f(node) {
            Exp::Simple(s) => {
                let mut f: F<'static, Exp, Exp> = self.compose_simple(s, ctx);
                f
            }
            Exp::Compound(c) => {
                let child_ctx = self.append_child(ctx);
                let f: F<Exp, Exp> = self.compose_compound(c, child_ctx);
                f
            }
            Exp::None => Box::new(ID),
        }
    }

    fn compose_simple(&mut self, node: Simple, ctx: Ctx) -> F<'static, Exp, Exp> {
        match node {
            Simple::Scalar(s) => {
                let f: F<Exp, Exp> = self.compose_scalar(s, ctx);
                f
            }
            Simple::Primitive(p) => match p {
                Primitive::Prefix(pre) => {
                    let f: F<Exp, Exp> = self.compose_primitve(pre, ctx);
                    f
                }
                Primitive::Suffix(suf) => {
                    let f: F<Exp, Exp> = self.compose_primitve(suf, ctx);
                    f
                }
            },

            Simple::Op(o) => self.compose_op(o, ctx),
            Simple::Ident(i) => self.compose_ident(i, ctx),
        }
    }

    fn compose_compound(&mut self, node: Compound, ctx: Ctx) -> F<'static, Exp, Exp> {
        let f = Box::new(ID);
        match node {
            Compound::Parens(exps) => {
                self.scope_type(ctx, ScopeType::Sequence);
                self.compose_exps(exps, f, ctx)
            }
            Compound::Brackets(exps) => {
                self.scope_type(ctx, ScopeType::Set);
                self.compose_exps(exps, f, ctx)
            }
            Compound::Braces(exps) => {
                self.scope_type(ctx, ScopeType::Stack);
                self.compose_exps(exps, f, ctx)
            }
            Compound::Ratio(abs) => {}
        }
    }

    fn compose_scalar(&mut self, node: Scalar, ctx: Ctx) -> F<'static, Exp, Exp> {
        match node {
            Scalar::Duration(d) => {
                let f: F<'static, Exp, Exp> = compose_duration(d, ctx);
                f
            }
            Scalar::Frequency(freq) => todo!(),
            Scalar::Pure(p) => todo!(),
        }
    }

    fn compose_duration(&mut self, node: Duration, ctx: Ctx) -> F<'static, Exp, Exp> {
        match node {
            Duration::Fixed(Fixed { minutes, seconds }) => {
                let micros = f64::round(
                    match minutes {
                        Absolute::Integer(int) => (int * 60 * 1_000_000) as f64,
                        Absolute::Float(float) => float * 1_000_000 as f64,
                    } + match seconds {
                        Absolute::Integer(int) => (int * 1_000_000) as f64,
                        Absolute::Float(float) => float * 1_000_000 as f64,
                    },
                ) as u64;

                let child_ctx = self.append_child(ctx);
                self.length(child_ctx, MicroSeconds(micros));

                let f: F<'static, Exp, Exp> = Box::new(move |exp| match exp {
                    Exp::Simple(s) => {
                        let mut f = self.compose_simple(s, child_ctx);
                        f(exp)
                    }
                    Exp::Compound(c) => {
                        let mut f = self.compose_compound(c, child_ctx);
                        f(exp)
                    }
                    Exp::None => todo!(),
                });
                f
            }
            Duration::Fractional(fr) => {
                let tempo: Upb = self.tempos[ctx.0];
                let length = MicroSeconds(match fr {
                    Absolute::Integer(int) => {
                        f64::round(int as f64 / 4 as f64 * tempo.0.0 as f64) as u64
                    }
                    Absolute::Float(float) => {
                        f64::round(float / 4 as f64 * tempo.0.0 as f64) as u64
                    }
                });

                let f: F<Exp, Exp> = Box::new(move |exp| match exp {
                    Exp::Simple(s) => {
                        let ctx = self.append_child(ctx);
                        self.length(ctx, length);
                        let mut f = self.compose_simple(s, ctx);

                        f(exp)
                    }
                    Exp::Compound(Compound::Parens(exps)) => {
                        let mut fs: Vec<(F<'static, Exp, Exp>, Ctx)>::new();
                        for ex in exps {
                            let child = self.append_child(ctx);
                            self.length(child, length);
                            fs.push((self.compose_exp(ex, f, child), child));
                        }
                        let mut g: F<Exp, Exp> = Box::new(move |exp| {
                            let mut g: F<Exp, Exp> = Box::new(ID);
                            for (f, ctx) in fs {
                                g = f(self.compose_exp(exp, g, ctx));
                            }
                            g(exp)
                        });
                        g(exp)
                    }
                    Exp::None => todo!(),
                    _ => todo!(),
                });
                f
            }
        }
    }

    fn compose_primitive(&mut self, node: Primitive, ctx: Ctx) -> F<'static, Exp, Primitive> {
        match node {
            Primitive::Prefix(p) => {
                let f: F<'static, Exp, Exp> = Box::new(move |exp| match p {
                    Prefix::Dur => match exp {
                        Exp::Simple(s) => match s {
                            Simple::Scalar(Scalar::Pure(Absolute(abs))) => match abs {
                                Absolute::Integer(int) => {
                                    let multiplier = int / 4;
                                    match exp {
                                        Exp::Simple(Simple::Scalar(Scalar::Pure(
                                            Pure::Absolute(abs),
                                        ))) => match abs {
                                            Absolute::Integer(int2) => {}
                                        },
                                    }
                                }
                                Absolute::Float(float) => {}
                            },
                        },
                        Exp::Compound(c) => todo!(),
                        Exp::None => todo!(),
                    },
                    Prefix::Pc => match exp {
                        Exp::Simple(s) => todo!(),
                        Exp::Compound(c) => todo!(),
                        Exp::None => todo!(),
                    },
                    Prefix::Reg => match exp {
                        Exp::Simple(s) => todo!(),
                        Exp::Compound(c) => todo!(),
                        Exp::None => todo!(),
                    },
                    Prefix::Rest => match exp {
                        Exp::Simple(s) => todo!(),
                        Exp::Compound(c) => todo!(),
                        Exp::None => todo!(),
                    },
                });
                f
            }
            Primitve::Suffix(suf) => {
                let f: F<'static, Exp, Exp> = Box::new(move |exp| match suf {
                    Suffix::Amp => match exp {
                        Exp::Simple(s) => todo!(),
                        Exp::Compound(c) => todo!(),
                        Exp::None => todo!(),
                    },
                    Suffix::Bpm => match exp {
                        Exp::Simple(s) => match s {
                            Simple::Scalar(Scalar::Pure(Pure::Absolute(abs))) => match abs {
                                Absolute::Integer(int) => {
                                    self.tempo(ctx, Upb(MicroSeconds(1_000_000 / int)));
                                    Exp::None
                                }
                                Absolute::Float(float) => {
                                    self.tempo(
                                        ctx,
                                        Upb(MicroSeconds(
                                            f64::round(1_000_000 as f64 / float) as u64
                                        )),
                                    );
                                    Exp::None
                                }
                            },
                            _ => todo!(),
                        },
                        Exp::Compound(c) => todo!(),
                        Exp::None => todo!(),
                    },
                    Suffix::Freq => match exp {
                        Exp::Simple(s) => todo!(),
                        Exp::Compound(c) => todo!(),
                        Exp::None => todo!(),
                    },
                });
                f
            }
        }
    }

    // fn compose_primitive(&mut self, node: Primitive, ctx: Ctx) -> F<'static, Exp, Exp> {
    //     match node {
    //         Primitive::Amp => todo!(),
    //         Primitive::Bpm => {
    //           let f: F<'static, Exp, Exp> = Box::new(move |exp| )
    //         },
    //         Primitive::Freq => todo!(),
    //         Primitive::Dur => todo!(),
    //         Primitive::Pc => todo!(),
    //         Primitive::Reg => todo!(),
    //         Primitive::Rest => todo!(),
    //     }
    // }

    fn compose_op(&mut self, node: Op, ctx: Ctx) -> F<'static, Exp, Exp> {
        match node {
            Op::Colon => {
                todo!()
            }
            Op::Intercalate => {
                todo!()
            }
        }
    }

    fn compose_ident(&mut self, node: Ident, ctx: Ctx) -> F<'static, Exp, Exp> {
        let f: F<'static, Exp, Exp> = Box::new(|exp| {
            let ident = node.0;
        });
    }

    fn compose_pure(&mut self, node: Pure, ctx: Ctx) -> F<'static, Exp, Exp> {
        match node {
            Pure::Absolute(a) => self.compose_absolute(a, ctx),
            Pure::Relative(r) => self.compose_relative(r, ctx),
        }
    }

    fn compose_absolute(&mut self, node: Absolute, ctx: Ctx) -> F<'static, Exp, Exp> {
        match node {
            Absolute::Integer(mut i) => Box::new(move |exp| match exp {
                Exp::Simple(s) => todo!(),
                Exp::Compound(c) => todo!(),
                Exp::None => todo!(),
            }),
            Absolute::Float(mut f) => Box::new(move |exp| todo!()),
        }
    }

    fn compose_relative(&mut self, node: Relative, ctx: Ctx) -> F<'static, Exp, Exp> {
        match node {
            Relative::Integer { sign, val } => match sign {
                Sign::Plus => self.compose_absolute(Absolute::Integer(n + val), ctx),
                Sign::Minus => Box::new(move |n| n - val),
            },
            Relative::Float { sign, val } => match sign {
                Sign::Plus => Box::new(move |n| n + val),
                Sign::Minus => Box::new(move |n| n - val),
            },
        }
    }

    fn scope_type(&mut self, idx: Ctx, scope_type: ScopeType) {
        if let Ctx::Idx(id) = idx {
            self.scope_types[id] = scope_type;
        }
    }

    fn length(&mut self, idx: Ctx, length: MicroSeconds) {
        if let Ctx::Idx(id) = idx {
            self.lengths[id] = length;
        }
    }

    fn pc(&mut self, idx: Ctx, pc: Pc) {
        if let Ctx::Idx(id) = idx {
            self.pcs[id] = pc;
        }
    }

    fn bpm(&mut self, idx: Ctx, bpm: Bpm) {
        if let Ctx::Idx(id) = idx {
            self.bpms[id] = bpm;
        }
    }

    fn register(&mut self, register: Integer) {
        if let Ctx::Idx(id) = idx {
            self.registers[id] = register;
        }
    }

    fn velocity(&mut self, velocity: Velocity) {
        if let Ctx::Idx(id) = idx {
            self.velocities[id] = velocity;
        }
    }

    fn instrument(&mut self, idx: Ctx, instrument: String) {
        if let Ctx::Idx(id) = idx {
            self.instruments[id] = Instrument(Vec::from_iter(*instrument.as_bytes().iter()));
        }
    }

    fn tempo(&mut self, idx: Ctx, tempo: Upb) {
        if let Ctx::Idx(id) = idx {
            self.tempos[id] = tempo;
        }
    }

    fn add_instruction(&mut self, idx: Ctx, instruction: Instruction) {
        if let Ctx::Idx(id) = idx {
            self.instructions[id].push(instruction);
        }
    }

    fn add_event(&mut self, idx: Ctx, event: TrackEvent) {
        if let Ctx::Idx(id) = idx {
            self.events[id].push(event);
        }
    }

    fn append_child(&mut self, parent: Ctx) -> Ctx {
        let id = self.idxs.len();
        let idx = Ctx::Idx(id);
        self.idxs.push(idx);
        self.parents.push(parent);
        self.children[parent].push(idx);
        self.children.push(Vec::<Ctx>::new());
        self.scope_types.push(ScopeType::None);
        self.lengths.push(MicroSeconds(0));
        self.pcs.push(Pc(-1));
        self.bpms
            .push(Bpm::Pure(Pure::Absolute(Absolute::Integer(0))));
        self.registers.push(Integer(4));
        self.velocities.push(Velocity(0));
        self.instruments
            .push(Instrument("Piano".as_bytes().iter().collect()));
        self.instructions.push(Vec::<Instruction>::new());
        self.events.push(Vec::<TrackEvent>::new());
        idx
    }
}
