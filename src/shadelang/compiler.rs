use crate::shadelang::ast::*;

pub static mut COUNTER: i32 = 0;

pub fn compile(ast: Program) -> VMProgram {
    codegen(ast)
}

pub fn codegen(ast: Program) -> VMProgram {
    let mut program = VMProgram::new();
    let mut stack_offset = 0;

    for d in ast.declarations.iter() {
        match d {
            TopLevelDeclaration::FunctionDeclaration(f) => {
                let mut funcMeta = FuncMeta::new(program.code.len());
                let mut hasReturn = false;

                for s in f.statements.iter() {
                    match s {
                        Statement::Assignment(i, expr) => {
                            funcMeta.symbols.insert(
                                i.clone(),
                                SymbolMeta {
                                    offset: stack_offset,
                                    type_kind: TypeKind::F32,
                                },
                            );

                            generate_expr(&mut program, &ast, expr);
                            program
                                .code
                                .push(MemoryCell::with_data(OpCode::Mov4, stack_offset as u16));

                            stack_offset += 4;
                        }
                        Statement::Return(expr) => {
                            generate_expr(&mut program, &ast, expr);
                            program.code.push(MemoryCell::plain_inst(OpCode::Ret));
                            hasReturn = true;
                        }
                    }
                }

                if !hasReturn {
                    program.code.push(MemoryCell::plain_inst(OpCode::Ret));
                }

                program.data.functions.insert(f.ident.clone(), funcMeta);
            }
            TopLevelDeclaration::OutParameterDeclaration(tk, id) => {
                program.data.global_symbols.insert(
                    id.clone(),
                    SymbolMeta {
                        offset: stack_offset,
                        type_kind: *tk,
                    },
                );

                stack_offset += 4;
            }
            _ => unimplemented!(),
        }
    }

    program.data.min_stack_size = stack_offset;
    program
}

pub fn generate_expr(program: &mut VMProgram, ast: &Program, expr: &Expr) {
    match expr {
        Expr::BinaryOp(op, e1, e2) => {
            generate_expr(program, ast, e1);
            generate_expr(program, ast, e2);

            let inst = match op {
                BinaryOperator::Add => OpCode::AddF32,
                BinaryOperator::Sub => OpCode::SubF32,
                BinaryOperator::Mul => OpCode::MulF32,
                BinaryOperator::Div => OpCode::DivF32,
            };

            program.code.push(MemoryCell::plain_inst(inst));
        }
        Expr::FuncCall(id, args) => {
            let offset = program.data.functions.get(id).unwrap().address;

            program
                .code
                .push(MemoryCell::with_data(OpCode::Call, offset as u16));
        }
        Expr::Literal(l) => match l {
            Literal::DecimalLiteral(f) => {
                program.code.push(MemoryCell::plain_inst(OpCode::ConstF32));
                program
                    .code
                    .push(MemoryCell::raw(unsafe { std::mem::transmute(*f as f32) }));
            }
            _ => unimplemented!(),
        },
        _ => {
            dbg!(expr);
            unimplemented!();
        }
    }
}

#[repr(u16)]
#[derive(Clone, Copy, Debug)]
pub enum OpCode {
    AddI32,
    SubI32,
    MulI32,
    DivI32,

    AddF32,
    SubF32,
    MulF32,
    DivF32,

    ConstF32,

    Mov4,

    Ret,
    Call,
    Jmp,
    JmpIf,
}

use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct SymbolMeta {
    offset: usize,
    type_kind: TypeKind,
}

#[derive(Debug, Clone)]
pub struct FuncMeta {
    pub address: usize,
    pub symbols: HashMap<String, SymbolMeta>,
}

impl FuncMeta {
    pub fn new(address: usize) -> Self {
        FuncMeta {
            address,
            symbols: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ProgramData {
    pub functions: HashMap<String, FuncMeta>,
    pub global_symbols: HashMap<String, SymbolMeta>,
    pub min_stack_size: usize,
}

impl ProgramData {
    pub fn new() -> Self {
        ProgramData {
            functions: HashMap::new(),
            global_symbols: HashMap::new(),
            min_stack_size: 0,
        }
    }
}

#[derive(Clone)]
pub struct MemoryCell {
    pub data: u32,
}

impl std::fmt::Debug for MemoryCell {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (op, ax) = self.get_inst();

        fmt.debug_struct("MemoryCell")
            .field("op", &op)
            .field("ax", &ax)
            .finish()
    }
}

impl MemoryCell {
    pub fn raw(data: u32) -> Self {
        MemoryCell { data }
    }

    pub fn plain_inst(inst: OpCode) -> Self {
        MemoryCell {
            data: inst as u16 as u32,
        }
    }

    pub fn with_data(inst: OpCode, data: u16) -> Self {
        MemoryCell {
            data: (inst as u16 as u32) | ((data as u32) << 16),
        }
    }

    pub fn get_inst(&self) -> (OpCode, u16) {
        (
            unsafe { std::mem::transmute(self.data as u16) },
            (self.data >> 16) as u16,
        )
    }
}

#[derive(Debug, Clone)]
pub struct VMProgram {
    pub code: Vec<MemoryCell>,
    pub data: ProgramData,
}

impl VMProgram {
    pub fn new() -> Self {
        VMProgram {
            code: Vec::new(),
            data: ProgramData::new(),
        }
    }
}
