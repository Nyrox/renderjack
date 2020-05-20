pub type Ident = String;

use std::fmt;

pub type VResult = Result<(), Box<dyn std::error::Error>>;

pub trait Visitor {
    fn post_expr(&mut self, _t: &mut Expr) -> VResult {
        Ok(())
    }
}

pub trait Visitable {
    fn visit(&mut self, v: &mut dyn Visitor) -> VResult;
}

impl<T: Visitable> Visitable for Vec<T> {
    fn visit(&mut self, v: &mut dyn Visitor) -> VResult {
        for t in self {
            t.visit(v)?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Position {
    pub line: u32,
    pub offset: Option<u32>,
}

#[derive(Debug, Clone)]
pub struct Spanned<T> {
    pub item: T,
    pub from: Position,
    pub to: Position,
}

impl<T: Copy> Copy for Spanned<T> {}

impl<T> Spanned<T> {
    pub fn new(item: T, from: Position, to: Position) -> Spanned<T> {
        Spanned { item, from, to }
    }

    pub fn encompass<A, B>(item: T, s1: Spanned<A>, s2: Spanned<B>) -> Spanned<T> {
        Spanned {
            item,
            from: s1.from,
            to: s2.to,
        }
    }

    pub fn map<U, F>(&self, f: F) -> Spanned<U>
    where
        F: FnOnce(&T) -> U,
    {
        Spanned {
            from: self.from,
            to: self.to,
            item: f(&self.item),
        }
    }
}

use std::ops::Deref;

impl<T> Deref for Spanned<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.item
    }
}

pub struct Reference<T, R> {
    raw: T,
    resolved: Option<R>,
}

impl<T: fmt::Debug, R: fmt::Debug> fmt::Debug for Reference<T, R> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(r) = &self.resolved {
            write!(f, "{:?} => {:?}", self.raw, r)
        } else {
            write!(f, "{:?}", self.raw)
        }
    }
}

#[derive(Clone, Debug, Copy)]
pub enum BinaryOperator {
    Add,
    Sub,
    Mul,
    Div,
}

#[derive(Clone, Debug, Copy)]
pub enum UnaryOperator {
    Not,
    Sub,
}

#[derive(Clone, Debug, Copy)]
pub enum Literal {
    IntegerLiteral(i64),
    DecimalLiteral(f64),
}

#[derive(Debug, Clone, Copy)]
pub enum TypeKind {
    Void,
    I32,
    F32,
}

#[derive(Clone, Debug)]
pub struct Symbol {
    pub ident: Ident,
}

#[derive(Clone, Debug)]
pub enum Expr {
    FuncCall(Spanned<Ident>, Vec<Box<Expr>>),
    BinaryOp(BinaryOperator, Box<Expr>, Box<Expr>),
    UnaryOp(UnaryOperator, Box<Expr>),
    Literal(Spanned<Literal>),
    Symbol(Symbol),
}

impl Visitable for Expr {
    fn visit(&mut self, v: &mut dyn Visitor) -> VResult {
        match self {
            Expr::FuncCall(_, params) => {
                for e in params {
                    (*e).visit(v)?
                }
            }
            Expr::BinaryOp(_, e1, e2) => (|| {
                e1.visit(v)?;
                e2.visit(v)
            })()?,
            Expr::UnaryOp(_, e) => e.visit(v)?,
            _ => (),
        }

        v.post_expr(self)
    }
}

#[derive(Clone, Debug)]
pub enum Statement {
    Assignment(Spanned<Ident>, Expr),
    Return(Expr),
}

impl Visitable for Statement {
    fn visit(&mut self, v: &mut dyn Visitor) -> VResult {
        match self {
            Statement::Assignment(_, expr) => expr.visit(v),
            Statement::Return(expr) => expr.visit(v),
        }
    }
}

#[derive(Clone, Debug)]
pub struct FunctionDeclaration {
    pub ident: Spanned<Ident>,
    pub param_types: Vec<Spanned<Ident>>,
    pub statements: Vec<Statement>,
}

impl Visitable for FunctionDeclaration {
    fn visit(&mut self, v: &mut dyn Visitor) -> VResult {
        self.statements.visit(v)
    }
}

#[derive(Clone, Debug)]
pub struct InParameterDeclaration {
    pub type_kind: Spanned<TypeKind>,
    pub ident: Spanned<Ident>,
}

impl Visitable for InParameterDeclaration {
    fn visit(&mut self, v: &mut dyn Visitor) -> VResult {
        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct OutParameterDeclaration {
    pub type_kind: Spanned<TypeKind>,
    pub ident: Spanned<Ident>,
}

impl Visitable for OutParameterDeclaration {
    fn visit(&mut self, v: &mut dyn Visitor) -> VResult {
        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct Program {
    pub functions: Vec<FunctionDeclaration>,
    pub out_parameters: Vec<OutParameterDeclaration>,
    pub in_parameters: Vec<InParameterDeclaration>,
}

impl Program {
    pub fn new() -> Self {
        Program {
            functions: Vec::new(),
            out_parameters: Vec::new(),
            in_parameters: Vec::new(),
        }
    }

    pub fn get_function(&self, ident: Ident) -> Option<&FunctionDeclaration> {
        self.functions.iter().find(|f| *f.ident == ident)
    }
}

impl Visitable for Program {
    fn visit(&mut self, v: &mut dyn Visitor) -> VResult {
        (|| {
            self.in_parameters.visit(v)?;
            self.out_parameters.visit(v)?;
            self.functions.visit(v)
        })()?;

        Ok(())
    }
}
