use crate::shadelang::ast::*;
use crate::shadelang::vm::*;

pub mod program_data;
pub mod resolve_types;

use program_data::{ProgramData, FuncMeta, SymbolMeta};

pub static mut COUNTER: i32 = 0;

pub fn compile(mut ast: Program) -> VMProgram {
    let mut program_data = ProgramData::new();

    resolve_types::resolve(&mut ast, &mut program_data).unwrap();

    codegen(ast, program_data)
}

pub fn codegen(ast: Program, mut data: ProgramData) -> VMProgram {
    let mut program = VMProgram::new();
    program.data = data.clone();

    let mut static_section = 0;

    for i in ast.in_parameters.iter() {
        data.global_symbols.insert(
            i.ident.item.clone(),
            SymbolMeta {
                stack_offset: Some(static_section),
                type_kind: i.type_kind.clone().item,
                is_static: true,
            },
        );

        static_section += 12;
    }

    program.data = data.clone();

    for o in ast.out_parameters.iter() {
        data.global_symbols.insert(
            o.ident.item.clone(),
            SymbolMeta {
                stack_offset: Some(static_section),
                type_kind: o.type_kind.clone().item,
                is_static: true,
            },
        );

        static_section += 4;
    }

    for f in ast.functions.iter() {
        let func_meta = data.functions.get_mut(f.ident.item.as_str()).unwrap();
        func_meta.address = Some(program.code.len());

        let mut has_return = false;
        let mut stack_offset = 0;
        
        for s in f.statements.iter() {
            match s {
                Statement::Assignment(i, expr) => {
                    generate_expr(&mut program, &ast, &func_meta, &expr);
                    
                    if let Some(o) = data.global_symbols.get(&i.item) {
                        program
                        .code
                        .push(MemoryCell::with_data(OpCode::Mov4Global, o.stack_offset.unwrap() as u16));
                    } else {
                        func_meta.symbols.get_mut(i.item.as_str()).unwrap().stack_offset = Some(stack_offset);
                        stack_offset += expr.get_type().unwrap().size();
                    }
                }
                Statement::Return(expr) => {
                    generate_expr(&mut program, &ast, &func_meta, &expr);
                    program
                        .code
                        .push(MemoryCell::with_data(OpCode::Ret, stack_offset as u16));
                    has_return = true;
                }
            }
        }
        if !has_return {
            program.code.push(MemoryCell::plain_inst(OpCode::Void));
            program
                .code
                .push(MemoryCell::with_data(OpCode::Ret, stack_offset as u16));
        }

    }

    program.data = data;
    program.data.static_section_size = static_section;
    program.data.min_stack_size = static_section + 1024;
    program
}

pub fn generate_expr(program: &mut VMProgram, ast: &Program, fnc: &FuncMeta, expr: &Expr) {
    match expr {
        Expr::BinaryOp(op, e1, e2) => {
            generate_expr(program, ast, fnc, e1);
            generate_expr(program, ast, fnc, e2);

            let inst = match (op, e1.get_type(), e2.get_type()) {
                (BinaryOperator::Mul, Some(TypeKind::F32), Some(TypeKind::Vec3)) => OpCode::MulF32_Vec3,
                (BinaryOperator::Add, _, _) => OpCode::AddF32,
                (BinaryOperator::Sub, _, _) => OpCode::SubF32,
                (BinaryOperator::Mul, _, _) => OpCode::MulF32,
                (BinaryOperator::Div, _, _) => OpCode::DivF32,
            };

            program.code.push(MemoryCell::plain_inst(inst));
        }
        Expr::FuncCall((id, args)) => {
            for arg in args {
                generate_expr(program, ast, fnc, arg);
            }

            if let Some((func, _)) = crate::shadelang::builtins::get_builtin_fn(id.raw.as_ref()) {
                program
                    .code
                    .push(MemoryCell::with_data(OpCode::CallBuiltIn, func as u16));
            } else if let Some(func) = program.data.functions.get(id.raw.as_str()) {
                program
                    .code
                    .push(MemoryCell::with_data(OpCode::Call, func.address.unwrap() as u16));
            } else {
                panic!("Unrecognized function: {:?}", id);
            }
        }
        Expr::Literal(l) => match l.item {
            Literal::DecimalLiteral(f) => {
                program.code.push(MemoryCell::plain_inst(OpCode::ConstF32));
                program
                    .code
                    .push(MemoryCell::raw(unsafe { std::mem::transmute(f as f32) }));
            }
            _ => unimplemented!(),
        },
        Expr::Symbol(s) => {
            let symbol = {
                if let Some(symbol) = fnc.symbols.get(s.raw.as_str()) {
                    symbol
                } else if let Some(symbol) = program.data.global_symbols.get(s.raw.as_str()) {
                    symbol
                }
                else {
                    panic!("Unknown symbol: {:?}", s);
                }
            };

            let offset = symbol.stack_offset.unwrap();
            let size = symbol.type_kind.size();

            let instruction = match symbol.is_static {
                true => OpCode::Load4Global,
                false => OpCode::Load4,
            };

            for i in 0..(size / 4) {
                program.code.push(MemoryCell::with_data(
                    instruction,
                    (offset + (i * 4)) as u16,
                ))
            }
        }
        Expr::UnaryOp(op, rhs) => match op {
            UnaryOperator::Sub => {
                generate_expr(program, ast, fnc, rhs);
                program.code.push(MemoryCell::plain_inst(OpCode::ConstF32));
                program
                    .code
                    .push(MemoryCell::raw(unsafe { std::mem::transmute(-1.0 as f32) }));

                program.code.push(MemoryCell::plain_inst(OpCode::MulF32));
            }
            _ => unimplemented!(),
        },
        _ => {
            dbg!(expr);
            unimplemented!();
        }
    }
}