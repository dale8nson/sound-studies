use std::{
    default,
    ops::{Add, Div, Mul, Rem, Sub},
};

#[derive(Debug, Clone)]
pub struct Program {
    pub exps: Vec<Exp>,
}

#[derive(Debug, Clone)]
pub enum Exp {
    Simple(Simple),
    Compound(Compound),
    None,
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

// impl Sub for Exp {
//     type Output = Exp;
//     fn sub(self, rhs: Self) -> Self::Output {
//         match self {
//             Exp::Simple(s1) => match rhs {
//                 Exp::Simple(s2) => s1 - s2,
//                 Exp::Compound(c) => s1 - c,
//                 Exp::None => self,
//             },
//             Exp::Compound(c1) => match rhs {
//                 Exp::Simple(s) => c1 - s,
//                 Exp::Compound(c2) => c1 - c2,
//                 Exp::None => self,
//             },
//             Exp::None => rhs,
//         }
//     }
// }

// impl Mul for Exp {
//     type Output = Exp;
//     fn mul(self, rhs: Self) -> Self::Output {
//         match self {
//             Exp::Simple(s1) => match rhs {
//                 Exp::Simple(s2) => s1 * s2,
//                 Exp::Compound(c) => s1 * c,
//                 Exp::None => self,
//             },
//             Exp::Compound(c1) => match rhs {
//                 Exp::Simple(s) => c1 * s,
//                 Exp::Compound(c2) => c1 * c2,
//                 Exp::None => self,
//             },
//             Exp::None => rhs,
//         }
//     }
// }

// impl Div for Exp {
//     type Output = Exp;
//     fn div(self, rhs: Self) -> Self::Output {
//         match self {
//             Exp::Simple(s1) => match rhs {
//                 Exp::Simple(s2) => s1 / s2,
//                 Exp::Compound(c) => s1 / c,
//                 Exp::None => self,
//             },
//             Exp::Compound(c1) => match rhs {
//                 Exp::Simple(s) => c1 / s,
//                 Exp::Compound(c2) => c1 / c2,
//                 Exp::None => self,
//             },
//             Exp::None => rhs,
//         }
//     }
// }

// impl Rem for Exp {
//     type Output = Exp;
//     fn rem(self, rhs: Self) -> Self::Output {
//         match self {
//             Exp::Simple(s1) => match rhs {
//                 Exp::Simple(s2) => s1 % s2,
//                 Exp::Compound(c) => s1 % c,
//                 Exp::None => self,
//             },
//             Exp::Compound(c1) => match rhs {
//                 Exp::Simple(s) => c1 % s,
//                 Exp::Compound(c2) => c1 % c2,
//                 Exp::None => self,
//             },
//             Exp::None => rhs,
//         }
//     }
// }

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

// impl Add for Compound<Rhs = Simple> {
//     type Output = Exp;
//     fn add(self, rhs: Simple) -> Self::Output {
//         match self {
//             Compound::Parens(p) => match rhs {},
//             Compound::Braces(b) => match rhs {},
//             Compound::Brackets(b) => match rhs {},
//             Compound::Ratio(r) => match rhs {},
//         }
//     }
// }

// impl Sub<Rhs = Simple> for Compound {
//     type Output = Exp;
//     fn sub(self, rhs: Simple) -> Self::Output {
//         match self {
//             Compound::Parens(p) => match rhs {},
//             Compound::Braces(b) => match rhs {},
//             Compound::Brackets(b) => match rhs {},
//             Compound::Ratio(r) => {
//                 let sum = r.iter().map(|Absolute::Integer(int)| int).sum();

//             }
//         }
//     }
// }

// impl Mul for Exp {
//     type Output = Exp;
//     fn mul(self, rhs: Self) -> Self::Output {
//         match self {
//             Exp::Simple(s1) => match rhs {
//                 Exp::Simple(s2) => s1 * s2,
//                 Exp::Compound(c) => s1 * c,
//                 Exp::None => self,
//             },
//             Exp::Compound(c1) => match rhs {
//                 Exp::Simple(s) => c1 * s,
//                 Exp::Compound(c2) => c1 * c2,
//                 Exp::None => self,
//             },
//             Exp::None => rhs,
//         }
//     }
// }

// impl Div for Exp {
//     type Output = Exp;
//     fn div(self, rhs: Self) -> Self::Output {
//         match self {
//             Exp::Simple(s1) => match rhs {
//                 Exp::Simple(s2) => s1 / s2,
//                 Exp::Compound(c) => s1 / c,
//                 Exp::None => self,
//             },
//             Exp::Compound(c1) => match rhs {
//                 Exp::Simple(s) => c1 / s,
//                 Exp::Compound(c2) => c1 / c2,
//                 Exp::None => self,
//             },
//             Exp::None => rhs,
//         }
//     }
// }

// impl Rem for Exp {
//     type Output = Exp;
//     fn rem(self, rhs: Self) -> Self::Output {
//         match self {
//             Exp::Simple(s1) => match rhs {
//                 Exp::Simple(s2) => s1 % s2,
//                 Exp::Compound(c) => s1 % c,
//                 Exp::None => self,
//             },
//             Exp::Compound(c1) => match rhs {
//                 Exp::Simple(s) => c1 % s,
//                 Exp::Compound(c2) => c1 % c2,
//                 Exp::None => self,
//             },
//             Exp::None => rhs,
//         }
//     }
// }

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
