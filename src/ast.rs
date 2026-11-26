use koopa::ir::builder_traits::{BasicBlockBuilder, LocalInstBuilder, ValueBuilder};
use koopa::ir::{BasicBlock, BinaryOp, FunctionData, Program, Type, Value};

#[derive(Debug)]
pub struct CompUnit {
    pub func_def: FuncDef,
}

#[derive(Debug)]
pub struct FuncDef {
    pub func_type: FuncType,
    pub ident: String,
    pub block: Block,
}

#[derive(Debug)]
pub enum FuncType {
    Int,
}

#[derive(Debug)]
pub struct Block {
    pub stmt: Stmt,
}

#[derive(Debug)]
pub enum Stmt {
    Return(Exp),
}

#[derive(Debug)]
pub enum Exp {
    Number(i32),
    Unary {
        operator: UnaryOp,
        operand: Box<Exp>,
    },
    Binary {
        operator: BinOp,
        lhs: Box<Exp>,
        rhs: Box<Exp>,
    },
}

#[derive(Debug)]
pub enum BinOp {
    Plus,
    Minus,
    Mul,
    Div,
    Mod,
    Le,
    Lt,
    Gt,
    Ge,
    And,
    Or,
    Eq,
    Ne,
}

#[derive(Debug)]
pub enum UnaryOp {
    Plus,
    Minus,
    Not,
}

impl CompUnit {
    pub fn dump(&self) -> Program {
        let mut program = Program::new();
        self.func_def.dump(&mut program);
        program
    }
}

impl BinOp {
    fn to_koopa_ir_binary_op(&self) -> BinaryOp {
        match self {
            BinOp::Plus => BinaryOp::Add,
            BinOp::Minus => BinaryOp::Sub,
            BinOp::Div => BinaryOp::Div,
            BinOp::Mul => BinaryOp::Mul,
            BinOp::Mod => BinaryOp::Mod,
            BinOp::Lt => BinaryOp::Lt,
            BinOp::Le => BinaryOp::Le,
            BinOp::Gt => BinaryOp::Gt,
            BinOp::Ge => BinaryOp::Ge,
            BinOp::Eq => BinaryOp::Eq,
            BinOp::Ne => BinaryOp::NotEq,
            _ => panic!("Not a binary operation"),
        }
    }
}

fn user_identifier(id: &str) -> String {
    format!("@{id}")
}

fn temp_identifier(id: &str) -> String {
    format!("%{id}")
}

impl Exp {
    fn dump(&self, func: &mut FunctionData, block: BasicBlock) -> Value {
        match self {
            Exp::Number(n) => func.dfg_mut().new_value().integer(*n),
            Exp::Unary { operator, operand } => {
                let operand_val = operand.dump(func, block);
                match operator {
                    UnaryOp::Not => {
                        let zero = func.dfg_mut().new_value().integer(0);
                        let eq = func
                            .dfg_mut()
                            .new_value()
                            .binary(BinaryOp::Eq, operand_val, zero);
                        func.layout_mut()
                            .bb_mut(block)
                            .insts_mut()
                            .push_key_back(eq)
                            .unwrap();
                        eq
                    }
                    UnaryOp::Plus => operand_val,
                    UnaryOp::Minus => {
                        let zero = func.dfg_mut().new_value().integer(0);
                        let sub =
                            func.dfg_mut()
                                .new_value()
                                .binary(BinaryOp::Sub, zero, operand_val);
                        func.layout_mut()
                            .bb_mut(block)
                            .insts_mut()
                            .push_key_back(sub)
                            .unwrap();
                        sub
                    }
                }
            }
            Exp::Binary { operator, lhs, rhs } => {
                let lhs_val = lhs.dump(func, block);
                let rhs_val = rhs.dump(func, block);
                let zero = func.dfg_mut().new_value().integer(0);
                if let BinOp::And | BinOp::Or = operator {
                    let lhs_snez =
                        func.dfg_mut()
                            .new_value()
                            .binary(BinaryOp::NotEq, lhs_val, zero);
                    let rhs_snez =
                        func.dfg_mut()
                            .new_value()
                            .binary(BinaryOp::NotEq, rhs_val, zero);
                    let op = if let BinOp::And = operator {
                        BinaryOp::And
                    } else {
                        BinaryOp::Or
                    };
                    let binary = func.dfg_mut().new_value().binary(op, lhs_snez, rhs_snez);
                    func.layout_mut()
                        .bb_mut(block)
                        .insts_mut()
                        .extend([lhs_snez, rhs_snez, binary]);
                    binary
                } else {
                    let binary_op = operator.to_koopa_ir_binary_op();
                    let binary = func
                        .dfg_mut()
                        .new_value()
                        .binary(binary_op, lhs_val, rhs_val);
                    func.layout_mut()
                        .bb_mut(block)
                        .insts_mut()
                        .push_key_back(binary)
                        .unwrap();
                    binary
                }
            }
        }
    }
}

impl FuncDef {
    fn dump(&self, program: &mut Program) {
        let main = program.new_func_def(user_identifier(&self.ident), Vec::new(), Type::get_i32());
        self.block.dump(program.func_mut(main));
    }
}

impl Block {
    fn dump(&self, func: &mut FunctionData) {
        let entry = func
            .dfg_mut()
            .new_bb()
            .basic_block(Some(temp_identifier("entry")));
        func.layout_mut().bbs_mut().push_key_back(entry).unwrap();
        self.stmt.dump(func, entry);
    }
}

impl Stmt {
    fn dump(&self, func: &mut FunctionData, block: BasicBlock) {
        match self {
            Stmt::Return(exp) => {
                let value = exp.dump(func, block);
                let ret = func.dfg_mut().new_value().ret(Some(value));
                func.layout_mut()
                    .bb_mut(block)
                    .insts_mut()
                    .push_key_back(ret)
                    .unwrap();
            }
        }
    }
}
