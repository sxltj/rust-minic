mod test;

use koopa::back::KoopaGenerator;
use rust_minic::asm::GenerateAsm;
use rust_minic::ast;
use std::env::args;
use std::fs;
use std::fs::read_to_string;
use std::io::{Error, ErrorKind, Result};

#[allow(clippy::ptr_arg)]
#[rustfmt::skip]
pub mod sysy {
    include!(concat!(env!("OUT_DIR"), "/sysy.rs"));
}

fn main() -> Result<()> {
    // 解析命令行参数
    let mut args = args();
    args.next();
    let mode = args.next().unwrap();
    let input = args.next().unwrap();
    args.next();
    let output = args.next().unwrap();

    // 读取输入文件
    let input = read_to_string(input)?;

    // 调用 lalrpop 生成的 parser 解析输入文件
    let ast = sysy::CompUnitParser::new().parse(&input).unwrap();
    let program = ast.dump();
    match mode.as_str() {
        "-koopa" =>
            // 输出解析得到的 AST
            KoopaGenerator::from_path(output)?.generate_on(&program),
        "-riscv" =>
            // 输出 RISC-V 汇编
            fs::write(output, program.generate()),
        _ => Err(Error::new(ErrorKind::InvalidInput, "Invalid mode"))
    }
}