use crate::shadelang::ast::*;

pub static mut counter: i32 = 0;

///
/// returns (
///    a series of statements to insert before the current one,
///    the replaced expression   
/// )
///
pub fn inline_expr(program: &Program, expr: &Expr) -> (Vec<Statement>, Expr) {
    match expr {
        Expr::FuncCall(ident, params) => {
            let tmpName = unsafe {
                counter += 1;
                format!("tmp_{}", counter)
            };
            let statements = &program.get_function(ident.to_string()).unwrap().statements;
            let statements = statements
                .iter()
                .map(|s| match &*s {
                    Statement::Return(expr) => Statement::Assignment(tmpName.clone(), expr.clone()),
                    _ => s.clone(),
                })
                .collect();

            (statements, Expr::Symbol(Symbol { ident: tmpName }))
        }
        _ => (vec![], expr.clone()),
    }
}

pub fn inline_pass(mut program: Program) -> Program {
    let declarations = program
        .declarations
        .iter()
        .map(|d| match &d {
            TopLevelDeclaration::FunctionDeclaration(func) => {
                let statements = func
                    .statements
                    .iter()
                    .map(|s| match s {
                        Statement::Assignment(i, expr) => {
                            let (mut insert, e) = inline_expr(&program, expr);

                            insert.push(Statement::Assignment(i.to_string(), e));

                            insert
                        }
                        _ => vec![s.clone()],
                    })
                    .flatten()
                    .collect();

                TopLevelDeclaration::FunctionDeclaration(FunctionDeclaration {
                    ident: func.ident.clone(),
                    param_types: func.param_types.clone(),
                    statements,
                })
            }
            _ => d.clone(),
        })
        .collect();

    Program { declarations }
}

pub fn compile(ast: Program) -> Program {
    let inlined = inline_pass(ast);

    inlined
}

