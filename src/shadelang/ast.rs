pub type Ident = String;

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

#[derive(Clone, Debug)]
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
    FuncCall(Ident, Vec<Box<Expr>>),
    BinaryOp(BinaryOperator, Box<Expr>, Box<Expr>),
    UnaryOp(UnaryOperator, Box<Expr>),
    Literal(Literal),
    Symbol(Symbol),
}

#[derive(Clone, Debug)]
pub enum Statement {
    Assignment(Ident, Expr),
    Return(Expr),
}

#[derive(Clone, Debug)]
pub struct FunctionDeclaration {
    pub ident: Ident,
    pub param_types: Vec<Ident>,
    pub statements: Vec<Statement>,
}

#[derive(Clone, Debug)]
pub enum TopLevelDeclaration {
    FunctionDeclaration(FunctionDeclaration),
    OutParameterDeclaration(TypeKind, Ident),
}

#[derive(Clone, Debug)]
pub struct Program {
    pub declarations: Vec<TopLevelDeclaration>,
}

impl Program {
    pub fn from_declarations(declarations: Vec<TopLevelDeclaration>) -> Self {
        Program { declarations }
    }

    pub fn get_function(&self, ident: Ident) -> Option<&FunctionDeclaration> {
        for d in self.declarations.iter() {
            match d {
                TopLevelDeclaration::FunctionDeclaration(decl) => {
                    if decl.ident == ident {
                        return Some(decl);
                    }
                }
                _ => (),
            }
        }

        None
    }
}
