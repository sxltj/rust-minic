use koopa::back::KoopaGenerator;
use rust_minic::asm::GenerateAsm;
use crate::sysy;

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

#[test]
fn asm_gen_test() {
    let ast = sysy::CompUnitParser::new().parse(r"
        int main() {
            // 注释也应该被删掉哦
            return 0;
        }
    ").unwrap();
    let program = ast.dump();
    println!("{}", program.generate())
}