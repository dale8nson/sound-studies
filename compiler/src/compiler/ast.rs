use std::{
    default,
    ops::{Add, Div, Mul, Rem, Sub},
};

// #[macro_export]
// macro_rules! unpack {
//   ($enum: ident , $($pat: ident )->* $(->)? $last:ident $( -> ($ty: ident))?) => {
//       match $enum {
//         $($pat::)*$last(param : $ty) => param,
//         $($pat::)*$last => $last
//         $last(param: $ty) => param
//         other => other
//       }
//     }
// }

#[derive(Debug, Clone)]
pub struct Program {
    pub exps: Vec<Exp>,
}

#[derive(Debug, Clone, Copy)]
pub enum Exp {
    Simple(Simple),
    Compound(Compound),
    None,
}

impl Exp {
    pub fn to_simple(self) -> Simple {
        if let Exp::Simple(simple) = self {
            simple
        } else {
            todo!()
        }
    }

    pub fn to_compound(self) -> Compound {
        if let Exp::Compound(compound) = self {
            compound
        } else {
            todo!()
        }
    }
}

impl Add for Exp {
    type Output = Exp;
    fn add(self, rhs: Exp) -> Self::Output {
        match self {
            Exp::Simple(s1) => match rhs {
                Exp::Simple(s2) => match (s1, s2) {
                    (
                        Simple::Scalar(Scalar::Pure(Pure::Absolute(Absolute::Integer(int1)))),
                        Simple::Scalar(Scalar::Pure(Pure::Absolute(Absolute::Integer(int2)))),
                    ) => Exp::Simple(Simple::Scalar(Scalar::Pure(Pure::Absolute(
                        Absolute::Integer(int1 + int2),
                    )))),
                    _ => self,
                },
                Exp::Compound(c) => s1 + c,
                Exp::None => self,
            },
            Exp::Compound(c1) => match rhs {
                Exp::Simple(s) => c1 + s,
                Exp::Compound(c2) => c1 + c2,
                Exp::None => self,
            },
            Exp::None => rhs,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct Fractional(pub Absolute);

#[derive(Debug, Clone, Default)]
pub enum Bpm {
    Compound(Compound),
    Pure(Pure),
    #[default]
    None,
}

#[derive(Debug, Clone)]
pub enum Compound {
    Parens(Vec<Exp>),
    Braces(Vec<Exp>),
    Brackets(Vec<Exp>),
    Ratio(Vec<Absolute>),
}

#[derive(Debug, Clone)]
pub enum Simple {
    Scalar(Scalar),
    Op(Op),
    Ident(Ident),
    Primitive(Primitive),
}

#[derive(Debug, Clone)]
pub enum Scalar {
    Duration(Duration),
    Frequency(Absolute),
    Pure(Pure),
}

#[derive(Debug, Clone)]
pub struct Frequency(pub Pure);

#[derive(Debug, Clone, Copy)]
pub enum Op {
    Colon,
    Intercalate,
}

#[derive(Debug, Clone)]
pub struct Ident(pub String);

#[derive(Debug, Clone, Copy)]
pub enum Primitive {
    Prefix(Prefix),
    Suffix(Suffix),
}

#[derive(Debug, Clone, Copy)]
pub enum Prefix {
    Dur,
    Rest,
    Pc,
    Reg,
}

#[derive(Debug, Clone, Copy)]
pub enum Suffix {
    Amp,
    Bpm,
    Freq,
}

#[derive(Debug, Clone)]
pub enum Duration {
    Fixed(Fixed),
    Fractional(Absolute),
}

#[derive(Debug, Clone)]
pub struct Fixed {
    pub minutes: Absolute,
    pub seconds: Absolute,
}

#[derive(Debug, Clone, Copy)]
pub struct Integer(pub u64);

#[derive(Debug, Clone, Copy)]
pub struct Float(pub f64);

#[derive(Debug, Clone)]
pub struct Minutes(pub Pure);

#[derive(Debug, Clone)]
pub struct Seconds(pub Pure);

#[derive(Debug, Clone)]
pub enum Pure {
    Relative(Relative),
    Absolute(Absolute),
}

#[derive(Debug, Clone)]
pub enum Relative {
    Integer { sign: Sign, val: u64 },
    Float { sign: Sign, val: f64 },
}

#[derive(Debug, Clone)]
pub enum Absolute {
    Integer(u64),
    Float(f64),
}

#[derive(Debug, Clone)]
pub enum Sign {
    Plus,
    Minus,
}

pub mod utils {
    use crate::compiler::ast::Absolute;

    pub fn abs_to_f64(abs: Absolute) -> f64 {
        match abs {
            Absolute::Integer(int) => int as f64,
            Absolute::Float(float) => float,
        }
    }
}
