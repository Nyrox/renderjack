use crate::shadelang::ast::*;

use crate::shadelang::scanner::*;
use std::iter::Iterator;
use std::iter::Peekable;

pub fn parse(input: impl AsRef<str>) -> Program {
    let tokens = Scanner::new(input.as_ref().chars()).scan_all().unwrap();

    let mut tokens = tokens.into_iter().peekable();

    parse_program(&mut tokens).unwrap()
}

type ItemType = Spanned<Token>;

#[derive(Debug, Clone)]
pub enum ParsingError {
    UnexpectedToken(ItemType),
    UnexpectedEndOfInput,
}

type ParsingResult<T> = Result<T, ParsingError>;

pub trait TokenSource {
    fn next(&mut self) -> Option<ItemType>;
    fn peek(&mut self) -> Option<&ItemType>;

    fn expect_next(&mut self) -> ParsingResult<ItemType> {
        match TokenSource::next(self) {
            None => Err(ParsingError::UnexpectedEndOfInput),
            Some(t) => Ok(t),
        }
    }

    fn expect_token(&mut self, token: Token) -> ParsingResult<ItemType> {
        match TokenSource::expect_next(self)? {
            t if *t == token => Ok(t),
            t => Err(ParsingError::UnexpectedToken(t)),
        }
    }

    fn expect_identifier(&mut self) -> ParsingResult<Spanned<String>> {
        let token = TokenSource::expect_next(self)?;
        match token.item {
            Token::Identifier(s) => Ok(Spanned::new(s, token.from, token.to)),
            _ => Err(ParsingError::UnexpectedToken(token)),
        }
    }
}

impl<T> TokenSource for Peekable<T>
where
    T: Iterator<Item = ItemType>,
{
    fn next(&mut self) -> Option<ItemType> {
        std::iter::Iterator::next(self)
    }

    fn peek(&mut self) -> Option<&ItemType> {
        self.peek()
    }
}

pub fn get_typekind(t: &Token) -> Option<TypeKind> {
    Some(match t {
        Token::Float => TypeKind::F32,
        Token::Void => TypeKind::Void,
        Token::Vec3 => TypeKind::Vec3,
        _ => {
            return None;
        }
    })
}

pub fn expect_typekind(tokens: &mut impl TokenSource) -> ParsingResult<Spanned<TypeKind>> {
    let token = tokens.expect_next()?;

    let res = token.map(|t| match t {
        Token::Float => Ok(TypeKind::F32),
        Token::Void => Ok(TypeKind::Void),
        Token::Vec3 => Ok(TypeKind::Vec3),
        Token::Identifier(i) => Ok(TypeKind::TypeRef(i.clone())),
        t => Err(ParsingError::UnexpectedToken(token.map(|_| t.clone()))),
    });

    match res.item {
        Ok(t) => Ok(token.map(|_| t)),
        Err(e) => Err(e),
    }
}

pub fn parse_program(tokens: &mut impl TokenSource) -> ParsingResult<Program> {
    let mut program = Program::new();

    'parsing: loop {
        let token = tokens.next();
        if token.is_none() {
            break 'parsing;
        }
        let token = token.unwrap();

        match &token.item {
            // Out Specifiers
            Token::Out => {
                let type_kind = expect_typekind(tokens)?;

                let ident = tokens.expect_identifier()?;
                program
                    .out_parameters
                    .push(OutParameterDeclaration { type_kind, ident });
                continue;
            }
            Token::In => {
                let type_kind = expect_typekind(tokens)?;

                let ident = tokens.expect_identifier()?;
                program
                    .in_parameters
                    .push(InParameterDeclaration { type_kind, ident });
                continue;
            }
            // func declarations
            t => {
                let tk = match t {
                    Token::Identifier(i) => Ok(TypeKind::TypeRef(i.clone())),
                    t => match get_typekind(&t) {
                        Some(v) => Ok(v),
                        None => Err(ParsingError::UnexpectedToken(token)),
                    },
                }?;
                let ident = tokens.expect_identifier()?;

                // arg list
                tokens.expect_token(Token::LeftParen)?;
                tokens.expect_token(Token::RightParen)?;

                // body
                tokens.expect_token(Token::LeftBrace)?;

                let statements = parse_statements(tokens)?;

                program.functions.push(FunctionDeclaration {
                    ident,
                    param_types: vec![],
                    statements,
                });
            }

            _ => {
                return Err(ParsingError::UnexpectedToken(token));
            }
        }
    }

    Ok(program)
}

pub fn parse_statements(tokens: &mut impl TokenSource) -> ParsingResult<Vec<Statement>> {
    let mut output = Vec::new();

    'parsing: loop {
        let token = tokens.next();
        if token.is_none() {
            break 'parsing;
        }
        let token = token.unwrap();

        match &token.item {
            Token::Return => {
                output.push(Statement::Return(parse_expr_bp(tokens, 0)?));
            }
            Token::Identifier(s) => {
                tokens.expect_token(Token::Equals)?;

                output.push(Statement::Assignment(
                    token.map(|_| s.clone()),
                    parse_expr_bp(tokens, 0)?,
                ));
            }
            Token::RightBrace => break 'parsing,
            _ => {
                return Err(ParsingError::UnexpectedToken(token));
            }
        }
    }

    Ok(output)
}

pub fn get_infix_operator(t: &Token) -> Option<BinaryOperator> {
    match t {
        Token::Plus => Some(BinaryOperator::Add),
        Token::Minus => Some(BinaryOperator::Sub),
        Token::Star => Some(BinaryOperator::Mul),
        Token::Slash => Some(BinaryOperator::Div),
        _ => None,
    }
}

pub fn infix_binding_power(op: BinaryOperator) -> (u8, u8) {
    match op {
        BinaryOperator::Add => (1, 2),
        BinaryOperator::Sub => (1, 2),
        BinaryOperator::Mul => (3, 4),
        BinaryOperator::Div => (3, 4),
    }
}

pub fn get_prefix_operator(t: &Token) -> Option<UnaryOperator> {
    match t {
        Token::Minus => Some(UnaryOperator::Sub),
        _ => None,
    }
}

pub fn prefix_binding_power(op: UnaryOperator) -> ((), u8) {
    match op {
        UnaryOperator::Sub => ((), 5),
        _ => unimplemented!(),
    }
}

pub fn parse_expr_bp(lexer: &mut impl TokenSource, min_bp: u8) -> ParsingResult<Expr> {
    let token = lexer.expect_next()?;
    // atoms
    let mut lhs = match &token.item {
        Token::FloatLiteral(f) => Expr::Literal(token.map(|_| Literal::DecimalLiteral(*f))),
        Token::IntegerLiteral(i) => Expr::Literal(token.map(|_| Literal::IntegerLiteral(*i))),
        Token::Identifier(i) => match lexer.peek() {
            Some(t) if t.item == Token::LeftParen => {
                lexer.next();

                let mut exprs = Vec::new();
                loop {
                    let e = parse_expr_bp(lexer, 0)?;
                    exprs.push(Box::new(e));
                    match lexer.expect_next()? {
                        t if t.item == Token::RightParen => {
                            break;
                        }
                        t if t.item == Token::Comma => {
                            continue;
                        }
                        t => Err(ParsingError::UnexpectedToken(t))?,
                    }
                }

                Expr::FuncCall((Reference::unresolved(token.map(|_| i.clone())), exprs))
            }
            _ => Expr::Symbol(Reference::unresolved(token.map(|_| i.clone()))),
        },
        Token::LeftParen => {
            let e = parse_expr_bp(lexer, 0)?;
            lexer.expect_token(Token::RightParen)?;
            e
        }
        t if get_prefix_operator(t).is_some() => {
            let op = get_prefix_operator(t).unwrap();
            let ((), r_bp) = prefix_binding_power(op);

            let rhs = parse_expr_bp(lexer, r_bp)?;
            Expr::UnaryOp(op, Box::new(rhs))
        }
        _ => return Err(ParsingError::UnexpectedToken(token)),
    };

    loop {
        let op = match lexer.peek() {
            Some(t) if get_infix_operator(t).is_some() => get_infix_operator(t).unwrap(),
            _ => break,
        };

        let (l_bp, r_bp) = infix_binding_power(op);
        {
            if l_bp < min_bp {
                break;
            }

            lexer.next().unwrap();
            let rhs = parse_expr_bp(lexer, r_bp)?;

            lhs = Expr::BinaryOp(op, Box::new(lhs), Box::new(rhs));
            continue;
        }
    }

    Ok(lhs)
}
