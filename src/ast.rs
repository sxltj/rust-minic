use koopa::ir::builder_traits::{BasicBlockBuilder, LocalInstBuilder, ValueBuilder};
use koopa::ir::{BasicBlock, FunctionData, Program, Type};

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
    Return(i32),
}

impl CompUnit {
    pub fn dump(&self) -> Program {
        let mut program = Program::new();
        self.func_def.dump(&mut program);
        program
    }
}

fn user_identifier(id: &str) -> String {
    format!("@{id}")
}

fn temp_identifier(id: &str) -> String {
    format!("%{id}")
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
            Stmt::Return(n) => {
                let value = func.dfg_mut().new_value().integer(*n);
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
