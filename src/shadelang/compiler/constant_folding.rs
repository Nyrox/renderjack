use crate::shadelang::ast::*;

pub struct ConstantFolding;

impl Visitor for ConstantFolding {
    fn post_expr(&mut self, expr: &mut Expr) -> VResult {
        match expr {
            _ => Ok(()),
        }
    }
}

pub fn fold(ast: &mut Program) -> VResult {
    ast.visit(&mut ConstantFolding {})
}
