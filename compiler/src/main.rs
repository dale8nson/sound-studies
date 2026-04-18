mod compiler;
mod pest_parser;

use crate::compiler::{ast::*, composer::Composer};
use crate::pest_parser::{Parser, Rule, parse_program};
use pest::set_error_detail;

use ndarray::{Array1, Ix1, array};

use num_integer::gcd;

use std::{
    iter::IntoIterator,
    ops::{Index, IndexMut},
    str::FromStr,
};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn main() -> Result<()> {
    set_error_detail(true);

    let src = std::fs::read_to_string("prototype.rsk")?;

    let pairs = Parser::parse(Rule::program, &src)?.next().unwrap();
    let ast = parse_program(pairs)?;

    println!("{ast:#?}");

    Ok(())
}
