use crate::shadelang::ast::*;

pub static mut COUNTER: i32 = 0;

///
/// returns (
///    a series of statements to insert before the current one,
///    the replaced expression   
/// )
///
pub fn inline_expr(program: &Program, expr: &Expr) -> (Vec<Statement>, Expr) {
    match expr {
        Expr::FuncCall(ident, _params) => {
            let tmp_name = unsafe {
                COUNTER += 1;
                format!("tmp_{}", COUNTER)
            };
            let statements = &program.get_function(ident.to_string()).unwrap().statements;
            let statements = statements
                .iter()
                .map(|s| match &*s {
                    Statement::Return(expr) => {
                        Statement::Assignment(tmp_name.clone(), expr.clone())
                    }
                    _ => s.clone(),
                })
                .collect();

            (statements, Expr::Symbol(Symbol { ident: tmp_name }))
        }
        _ => (vec![], expr.clone()),
    }
}

pub fn inline_pass(program: Program) -> Program {
    let declarations = program
        .declarations
        .iter()
        .flat_map(|d| match &d {
            TopLevelDeclaration::FunctionDeclaration(func) => {
                // TODO: Maybe implement option to have other functions in the final output?
                if func.ident != "main" {
                    return None;
                }

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

                Some(TopLevelDeclaration::FunctionDeclaration(
                    FunctionDeclaration {
                        ident: func.ident.clone(),
                        param_types: func.param_types.clone(),
                        statements,
                    },
                ))
            }
            _ => Some(d.clone()),
        })
        .collect();

    Program { declarations }
}

pub fn compile(ast: Program) -> Program {
    let inlined = inline_pass(ast);

    inlined
}
