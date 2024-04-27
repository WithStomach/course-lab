#![allow(unused_variables)]

use lalrpop_util::lalrpop_mod;
use std::env::args;
use std::fs::{read_to_string, File};
use std::io::{Result, Write};

lalrpop_mod!(sysy);
mod ir_gen;
mod koopa2asm;
use koopa2asm::koopa2asm;

fn main() -> Result<()> {
    let mut args = args();
    args.next();
    let mode = args.next().unwrap();
    let input = args.next().unwrap();
    args.next();
    let output = args.next().unwrap();

    let input = read_to_string(input)?;
    let ast = sysy::CompUnitParser::new().parse(&input).unwrap();
    let mut file = File::create(output)?;
    let koopa_str = ast.generate_koopa();
    if mode == "-koopa" {
        let _err = write!(file, "{}", koopa_str);
    } else if mode == "-riscv" {
        let driver = koopa::front::Driver::from(koopa_str);
        let program = driver.generate_program().unwrap();
        let asm_str = koopa2asm(&program);
        let _err = write!(file, "{}", asm_str);
    }
    Ok(())
}
