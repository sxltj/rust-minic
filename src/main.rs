use koopa::back::KoopaGenerator;
use rust_minic::ast;
use std::env::args;
use std::fs::{File, read_to_string};
use std::io::{Error, Result};

#[allow(clippy::ptr_arg)]
#[rustfmt::skip]
pub mod sysy {
    include!(concat!(env!("OUT_DIR"), "/sysy.rs"));
}

#[test]
fn parse_test() {
    let ast = sysy::CompUnitParser::new().parse(r"
        int main() {
            // 注释也应该被删掉哦
            return 0;
        }
    ").unwrap();

    // 输出解析得到的 AST
    println!("{:#?}", ast);
}

#[test]
fn ir_gen_test() {
    let ast = sysy::CompUnitParser::new().parse(r"
        int main() {
            // 注释也应该被删掉哦
            return 0;
        }
    ").unwrap();
    let program = ast.dump();
    KoopaGenerator::new(std::io::stdout()).generate_on(&program).unwrap();
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

    // 输出解析得到的 AST
    
    let program = ast.dump();
    KoopaGenerator::from_path(output)?.generate_on(&program)
}