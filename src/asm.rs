use koopa::ir::{FunctionData, Program, ValueKind};
use std::fmt::Write;

pub trait GenerateAsm {
    fn generate(&self) -> String;
}

impl GenerateAsm for Program {
    fn generate(&self) -> String {
        format!(
            "\t.text\n{}",
            self.func_layout()
                .iter()
                .map(|&func| self.func(func).generate())
                .collect::<String>()
        )
    }
}

impl GenerateAsm for FunctionData {
    fn generate(&self) -> String {
        let mut asm = String::new();
        let name = self.name().strip_prefix(['%', '@']).unwrap();

        writeln!(asm, "\t.globl {name}").unwrap();
        writeln!(asm, "{name}:").unwrap();

        for (_, node) in self.layout().bbs() {
            for &inst in node.insts().keys() {
                let value_data = self.dfg().value(inst);

                match value_data.kind() {
                    ValueKind::Return(ret) => {
                        if let Some(operand) = ret.value() {
                            match self.dfg().value(operand).kind() {
                                ValueKind::Integer(int) => {
                                    writeln!(asm, "\tli a0, {}", int.value()).unwrap();
                                }
                                _ => todo!(),
                            }
                        }
                        writeln!(asm, "\tret").unwrap();
                    }
                    _ => todo!(),
                }
            }
        }

        asm.push('\n');
        asm
    }
}
