use crate::compiler::{
    ast::utils::abs_to_f64,
    codegen::{
        utils::{duration_to_micros, transform_exps},
        *,
    },
};

use super::ast::*;

use std::ops::{Index, IndexMut};

#[derive(Debug, Default)]
pub struct Composer<'a> {
    contexts: Vec<Ctx>,
    tps: Tps,
    parents: Vec<Ctx>,
    scope_types: Vec<ScopeType>,
    lengths: Vec<MicroSeconds>,
    pcs: Vec<Vec<Pc>>,
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
        let mut f: F<Exp, Exp> = ID();
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
            Exp::None => ID(),
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
        let f = ID();
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
            Compound::Ratio(abss) => {
                todo!()
            }
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
                let micros = duration_to_micros(minutes, seconds);
                let child_ctx = self.append_child(ctx);
                self.length(child_ctx, micros);
                Box::new(move |exp| self.compose_exp(exp, ID(), child_ctx)(exp))
            }
            Duration::Fractional(fr) => {
                let tempo: Upb = self.tempos[ctx.0];
                let fr = abs_to_f64(fr);
                let multiplier = fr / 4 as f64;
                let length = MicroSeconds(f64::round(multiplier * tempo.0.0 as f64) as u64);

                let f: F<Exp, Exp> = Box::new(move |exp| match exp {
                    Exp::Simple(s) => {
                        let ctx = self.append_child(ctx);
                        self.length(ctx, length);
                        let mut f = self.compose_simple(s, ctx);

                        f(exp)
                    }
                    Exp::Compound(Compound::Parens(exps)) => {
                        let exps = transform_exps(
                            exps,
                            Box::new(move |exp| match exp {
                                Exp::Simple(Simple::Scalar(Scalar::Pure(Pure::Absolute(abs)))) => {
                                    let abs = abs_to_f64(abs);
                                    Exp::Simple(Simple::Scalar(Scalar::Pure(Pure::Absolute(
                                        Absolute::Integer(f64::round(multiplier * abs) as u64),
                                    ))))
                                }
                                _ => exp,
                            }),
                        );
                        let f: F<'static, Exp, Exp> = Box::new(move |exp| match exp {
                            Exp::Simple(Simple::Primitive(Primitive::Suffix(Suffix::Bpm))) => {
                                let h = move |exp| match exp {
                                    Exp::Simple(Simple::Scalar(Scalar::Pure(Pure::Absolute(
                                        abs,
                                    )))) => {
                                        let abs = abs_to_f64(abs);
                                    }
                                    _ => todo!(),
                                };

                                Exp::Compound(Compound::Parens(exps.zip(gs)))
                            }
                            _ => todo!(),
                        });
                        compound(transform_exps(exps, f))
                    }
                    Exp::None => todo!(),
                    _ => todo!(),
                });
                f
            }
        }
    }

    fn compose_primitive(&mut self, node: Primitive, ctx: Ctx) -> F<'static, Exp, Exp> {
        match node {
            Primitive::Prefix(p) => self.compose_prefix(p, ctx),

            Primitive::Suffix(suf) => Box::new(move |exp| self.compose_suffix(suf, ctx)(exp)),
        }
    }

    fn compose_prefix(&mut self, node: Prefix, ctx: Ctx) -> F<'static, Exp, Exp> {
        match node {
            Prefix::Dur => todo!(),
            Prefix::Pc => {
                let f: F<'static, Exp, Exp> = Box::new(move |exp| match exp {
                    Exp::Compound(c) => match c {
                        compound @ Compound::Braces(exps)
                        | compound @ Compound::Parens(exps)
                        | compound @ Compound::Brackets(exps) => {
                            let g: F<'static, Exp, Exp> = Box::new(move |exp| match exp {
                                Exp::Simple(Simple::Scalar(Scalar::Pure(Pure::Absolute(
                                    Absolute::Integer(int),
                                )))) => {
                                    self.pc(ctx, Pc(int as u8));
                                }
                                _ => unreachable!(),
                            });
                            compound(transform_exps(exps, g))
                        }
                        _ => todo!(),
                    },
                    _ => todo!(),
                });
                f
            }
            Prefix::Reg => todo!(),
            Prefix::Rest => todo!(),
        }
    }

    fn compose_suffix(&mut self, node: Suffix, ctx: Ctx) -> F<'static, Exp, Exp> {
        match node {
            Suffix::Amp => todo!(),
            Suffix::Bpm => Box::new(move |exp| match exp {
                Exp::Simple(Simple::Scalar(Scalar::Pure(Pure::Absolute(abs)))) => {
                    let bpm = abs_to_f64(abs);
                    self.tempo(ctx, Upb(MicroSeconds(bpm as u64)));
                    Exp::None
                }
                Exp::Compound(comp) => match comp {
                    compound @ Compound::Parens(exps)
                    | compound @ Compound::Braces(exps)
                    | compound @ Compound::Brackets(exps) => {
                        dbg!(compound);
                        let ctx = self.append_child(ctx);
                        compound(transform_exps(exps, g(exp)))
                    }
                    _ => todo!(),
                },
                _ => Exp::None,
            }),
            Suffix::Freq => todo!(),
        }
    }

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
            self.pcs[id].push(pc);
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
        let id = self.contexts.len();
        let idx = Ctx::Idx(id);
        self.contexts.push(idx);
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
