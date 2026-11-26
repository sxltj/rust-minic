use koopa::ir::values::{Binary, Return};
use koopa::ir::{BinaryOp, FunctionData, Program, Value, ValueKind};
use std::collections::HashMap;
use std::fmt::{Display, Formatter, Write};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Reg(&'static str);

impl Reg {
    const TEMP_REGS: [Reg; 15] = [
        Reg("t0"),
        Reg("t1"),
        Reg("t2"),
        Reg("t3"),
        Reg("t4"),
        Reg("t5"),
        Reg("t6"),
        Reg("a0"),
        Reg("a1"),
        Reg("a2"),
        Reg("a3"),
        Reg("a4"),
        Reg("a5"),
        Reg("a6"),
        Reg("a7"),
    ];

    const A0: Reg = Reg("a0");
    const X0: Reg = Reg("x0");

    fn t(n: usize) -> Reg {
        Self::TEMP_REGS[n]
    }
}

impl Display for Reg {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

pub fn generate_asm(program: &Program) -> String {
    let mut asm = String::new();
    program.generate(&mut asm);
    asm
}

pub trait GenerateAsm {
    fn generate(&self, asm: &mut String);
}

impl GenerateAsm for Program {
    fn generate(&self, asm: &mut String) {
        writeln!(asm, "\t.text").unwrap();
        for &func in self.func_layout() {
            let func_data = self.func(func);
            if func_data.layout().entry_bb().is_some() {
                func_data.generate(asm);
            }
        }
    }
}

impl GenerateAsm for FunctionData {
    fn generate(&self, asm: &mut String) {
        let name = self.name().strip_prefix(['%', '@']).unwrap();

        writeln!(asm, "\t.globl {name}").unwrap();
        writeln!(asm, "{name}:").unwrap();

        let mut register_map: HashMap<Value, Reg> = HashMap::new();

        for (_, node) in self.layout().bbs() {
            for &inst in node.insts().keys() {
                let value_data = self.dfg().value(inst);

                match value_data.kind() {
                    ValueKind::Binary(bin) => {
                        generate_binary(self, asm, bin, inst, &mut register_map)
                    }
                    ValueKind::Return(ret) => generate_return(self, asm, ret, &mut register_map),
                    _ => todo!(),
                }
            }
        }
    }
}

fn generate_binary(
    func: &FunctionData,
    asm: &mut String,
    bin: &Binary,
    this: Value,
    register_map: &mut HashMap<Value, Reg>,
) {
    let result = alloc_tmp_reg(register_map, this);
    let lhs_reg = load_to_reg(asm, register_map, func, bin.lhs(), Some(result));
    let rhs_reg = load_to_reg(asm, register_map, func, bin.rhs(), None);

    match bin.op() {
        BinaryOp::Eq => match (lhs_reg, rhs_reg) {
            (Reg::X0, Reg::X0) => emit_li(asm, result, 1),
            (Reg::X0, reg) | (reg, Reg::X0) => emit_seqz(asm, result, reg),
            (lhs, rhs) => {
                emit_xor(asm, result, lhs, rhs);
                emit_seqz(asm, result, result);
            }
        },
        BinaryOp::Sub => match (lhs_reg, rhs_reg) {
            (Reg::X0, Reg::X0) => emit_li(asm, result, 0),
            (lhs, rhs) => emit_sub(asm, result, lhs, rhs),
        },
        BinaryOp::Add => match (lhs_reg, rhs_reg) {
            (Reg::X0, Reg::X0) => emit_li(asm, result, 0),
            (lhs, rhs) => emit_add(asm, result, lhs, rhs),
        },
        BinaryOp::Mul => match (lhs_reg, rhs_reg) {
            (Reg::X0, _) | (_, Reg::X0) => emit_li(asm, result, 0),
            (lhs, rhs) => emit_mul(asm, result, lhs, rhs),
        },
        BinaryOp::Div => match (lhs_reg, rhs_reg) {
            (Reg::X0, _) => emit_li(asm, result, 0),
            (lhs, rhs) => emit_div(asm, result, lhs, rhs),
        }, // What about division by zero?
        BinaryOp::Mod => match (lhs_reg, rhs_reg) {
            (Reg::X0, _) => emit_li(asm, result, 0),
            (lhs, rhs) => emit_rem(asm, result, lhs, rhs),
        },
        BinaryOp::Le => {
            emit_sgt(asm, result, lhs_reg, rhs_reg);
            emit_seqz(asm, result, result);
        }
        BinaryOp::Ge => {
            emit_slt(asm, result, lhs_reg, rhs_reg);
            emit_seqz(asm, result, result);
        }
        BinaryOp::Lt => emit_slt(asm, result, lhs_reg, rhs_reg),
        BinaryOp::Gt => emit_sgt(asm, result, lhs_reg, rhs_reg),
        BinaryOp::And => match (lhs_reg, rhs_reg) {
            (Reg::X0, _) | (_, Reg::X0) => emit_li(asm, result, 0),
            (lhs, rhs) => {
                emit_and(asm, result, lhs, rhs);
            }
        },
        BinaryOp::Or => emit_or(asm, result, lhs_reg, rhs_reg),
        BinaryOp::NotEq => {
            emit_xor(asm, result, lhs_reg, rhs_reg);
            emit_snez(asm, result, result);
        }
        _ => todo!(),
    }
}

fn generate_return(
    func: &FunctionData,
    asm: &mut String,
    ret: &Return,
    register_map: &mut HashMap<Value, Reg>,
) {
    if let Some(operand) = ret.value() {
        let reg = load_to_reg(asm, register_map, func, operand, None);
        emit_mv(asm, Reg::A0, reg)
    }
    emit_ret(asm);
}

fn alloc_tmp_reg(map: &mut HashMap<Value, Reg>, val: Value) -> Reg {
    let len = map.len();
    let reg = Reg::TEMP_REGS[len];
    map.insert(val, reg);
    reg
}

fn load_to_reg(
    asm: &mut String,
    map: &mut HashMap<Value, Reg>,
    func: &FunctionData,
    val: Value,
    fallback: Option<Reg>,
) -> Reg {
    if let Some(&reg) = map.get(&val) {
        return reg;
    }

    match func.dfg().value(val).kind() {
        ValueKind::Integer(i) if i.value() == 0 => Reg::X0,
        ValueKind::Integer(i) => {
            if let Some(reg) = fallback {
                emit_li(asm, reg, i.value());
                reg
            } else {
                let tmp = alloc_tmp_reg(map, val);
                emit_li(asm, tmp, i.value());
                tmp
            }
        }
        _ => panic!("load_to_reg: unknown value kind"),
    }
}

fn emit_mv(asm: &mut String, target: Reg, source: Reg) {
    writeln!(asm, "\tmv {target}, {source}").unwrap();
}

fn emit_li(asm: &mut String, target: Reg, im: i32) {
    writeln!(asm, "\tli {target}, {im}").unwrap();
}

fn emit_xor(asm: &mut String, target: Reg, lhs: Reg, rhs: Reg) {
    writeln!(asm, "\txor {target}, {lhs}, {rhs}").unwrap();
}

fn emit_seqz(asm: &mut String, target: Reg, source: Reg) {
    writeln!(asm, "\tseqz {target}, {source}").unwrap();
}

fn emit_snez(asm: &mut String, target: Reg, source: Reg) {
    writeln!(asm, "\tsnez {target}, {source}").unwrap();
}

fn emit_ret(asm: &mut String) {
    writeln!(asm, "\tret").unwrap();
}

fn emit_sub(asm: &mut String, target: Reg, r1: Reg, r2: Reg) {
    writeln!(asm, "\tsub {target}, {r1}, {r2}").unwrap();
}

fn emit_add(asm: &mut String, target: Reg, r1: Reg, r2: Reg) {
    writeln!(asm, "\tadd {target}, {r1}, {r2}").unwrap();
}

fn emit_mul(asm: &mut String, target: Reg, r1: Reg, r2: Reg) {
    writeln!(asm, "\tmul {target}, {r1}, {r2}").unwrap();
}

fn emit_div(asm: &mut String, target: Reg, r1: Reg, r2: Reg) {
    writeln!(asm, "\tdiv {target}, {r1}, {r2}").unwrap();
}

fn emit_rem(asm: &mut String, target: Reg, r1: Reg, r2: Reg) {
    writeln!(asm, "\trem {target}, {r1}, {r2}").unwrap();
}

fn emit_sgt(asm: &mut String, target: Reg, r1: Reg, r2: Reg) {
    writeln!(asm, "\tsgt {target}, {r1}, {r2}").unwrap();
}

fn emit_slt(asm: &mut String, target: Reg, r1: Reg, r2: Reg) {
    writeln!(asm, "\tslt {target}, {r1}, {r2}").unwrap();
}

fn emit_and(asm: &mut String, target: Reg, r1: Reg, r2: Reg) {
    writeln!(asm, "\tand {target}, {r1}, {r2}").unwrap();
}

fn emit_or(asm: &mut String, target: Reg, r1: Reg, r2: Reg) {
    writeln!(asm, "\tor {target}, {r1}, {r2}").unwrap();
}
