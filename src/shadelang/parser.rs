use crate::shadelang::ast::*;

use crate::shadelang::scanner::*;
use std::iter::Iterator;
use std::iter::Peekable;

pub fn parse(input: impl AsRef<str>) -> Program {
    let tokens = Scanner::new(input.as_ref().chars()).scan_all().unwrap();

    println!("{:#?}", tokens);

    let mut tokens = tokens.into_iter().peekable();

    let declarations = parse_declarations(&mut tokens).unwrap();

    Program {
        declarations: declarations,
    }
}

#[derive(Debug, Clone)]
pub enum ParsingError {
    UnexpectedToken(Token),
    UnexpectedEndOfInput,
}

type ParsingResult<T> = Result<T, ParsingError>;

pub trait TokenSource {
    fn next(&mut self) -> Option<Token>;
    fn peek(&mut self) -> Option<&Token>;

    fn expect_next(&mut self) -> ParsingResult<Token> {
        match TokenSource::next(self) {
            None => Err(ParsingError::UnexpectedEndOfInput),
            Some(t) => Ok(t),
        }
    }

    fn expect_token(&mut self, token: Token) -> ParsingResult<Token> {
        match TokenSource::expect_next(self)? {
            t if t == token => Ok(token),
            t => Err(ParsingError::UnexpectedToken(t)),
        }
    }

    fn expect_identifier(&mut self) -> ParsingResult<String> {
        match TokenSource::expect_next(self)? {
            Token::Identifier(s) => Ok(s),
            t => Err(ParsingError::UnexpectedToken(t)),
        }
    }
}

impl<T> TokenSource for Peekable<T>
where
    T: Iterator<Item = Token>,
{
    fn next(&mut self) -> Option<Token> {
        std::iter::Iterator::next(self)
    }

    fn peek(&mut self) -> Option<&Token> {
        self.peek()
    }
}

pub fn get_typekind(t: &Token) -> Option<TypeKind> {
    Some(match t {
        Token::Float => TypeKind::F32,
        Token::Void => TypeKind::Void,
        _ => {
            return None;
        }
    })
}

pub fn parse_declarations(
    tokens: &mut impl TokenSource,
) -> ParsingResult<Vec<TopLevelDeclaration>> {
    let mut declarations = vec![];

    'parsing: loop {
        match tokens.next() {
            // Out Specifiers
            Some(Token::Out) => {
                let type_kind = match tokens.expect_next()? {
                    Token::Float => TypeKind::F32,
                    t => {
                        return Err(ParsingError::UnexpectedToken(t));
                    }
                };

                let ident = tokens.expect_identifier()?;
                declarations.push(TopLevelDeclaration::OutParameterDeclaration(
                    type_kind, ident,
                ));
                continue;
            }
            // func declarations
            Some(t) if get_typekind(&t).is_some() => {
                let _tk = get_typekind(&t).unwrap();
                let ident = tokens.expect_identifier()?;

                // arg list
                tokens.expect_token(Token::LeftParen)?;
                tokens.expect_token(Token::RightParen)?;

                // body
                tokens.expect_token(Token::LeftBrace)?;

                let statements = parse_statements(tokens)?;

                declarations.push(TopLevelDeclaration::FunctionDeclaration(
                    FunctionDeclaration {
                        ident,
                        param_types: vec![],
                        statements,
                    },
                ))
            }

            None => {
                break 'parsing;
            }
            Some(t) => {
                return Err(ParsingError::UnexpectedToken(t));
            }
        }
    }

    Ok(declarations)
}

pub fn parse_statements(tokens: &mut impl TokenSource) -> ParsingResult<Vec<Statement>> {
    let mut output = Vec::new();

    'parsing: loop {
        match tokens.next() {
            Some(Token::Return) => {
                output.push(Statement::Return(parse_expr_bp(tokens, 0)?));
            }
            Some(Token::Identifier(s)) => {
                tokens.expect_token(Token::Equals)?;

                output.push(Statement::Assignment(s, parse_expr_bp(tokens, 0)?));
            }
            Some(Token::RightBrace) => break 'parsing,
            None => {
                break 'parsing;
            }
            Some(t) => {
                return Err(ParsingError::UnexpectedToken(t));
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

pub fn parse_expr_bp(lexer: &mut impl TokenSource, min_bp: u8) -> ParsingResult<Expr> {
    let mut lhs = match lexer.expect_next()? {
        Token::FloatLiteral(f) => Expr::Literal(Literal::DecimalLiteral(f)),
        Token::IntegerLiteral(i) => Expr::Literal(Literal::IntegerLiteral(i)),
        Token::Identifier(i) => match lexer.peek() {
            Some(Token::LeftParen) => {
                lexer.next();
                lexer.expect_token(Token::RightParen)?;

                Expr::FuncCall(i, vec![])
            }
            _ => Expr::Symbol(Symbol { ident: i }),
        },
        t => return Err(ParsingError::UnexpectedToken(t)),
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
