use crate::shadelang::ast::*;

#[derive(Debug, Clone)]
pub struct Position {
    line: u32,
    character: Option<u32>,
}

#[derive(Debug, Clone)]
pub struct Spanned<T> {
    item: T,
    from: Position,
    to: Position,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Out,
    Float,
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Equals,
    EqualsEquals,

    Plus,
    Minus,
    Star,
    Slash,

    Void,
    Return,

    Identifier(String),
    FloatLiteral(f64),
    IntegerLiteral(i64),
}

#[derive(Debug, Clone)]
pub enum ScanningProduct {
    Skip,
    Finished,
    Token(Token),
}

type ScanningError = ();

type ScanningResult = Result<ScanningProduct, ScanningError>;

pub struct Scanner<I: Iterator<Item = char>> {
    input: I,
    line: u32,
    peeked: Option<char>,
}

impl<I: Iterator<Item = char>> Scanner<I> {
    pub fn new(input: I) -> Self {
        Scanner {
            input,
            line: 1,
            peeked: None,
        }
    }

    pub fn scan_all(mut self) -> Result<Vec<Token>, ScanningError> {
        let mut output = Vec::new();

        loop {
            match self.scan_token()? {
                ScanningProduct::Skip => (),
                ScanningProduct::Finished => return Ok(output),
                ScanningProduct::Token(token) => {
                    output.push(token);
                }
            }
        }
    }

    pub fn advance(&mut self) -> Option<char> {
        match self.peeked {
            None => self.input.next(),
            Some(c) => {
                self.peeked = None;
                Some(c)
            }
        }
    }

    pub fn peek(&mut self) -> Option<char> {
        match self.peeked {
            Some(c) => Some(c),
            None => {
                self.peeked = self.input.next();
                self.peeked
            }
        }
    }

    pub fn keyword(&self, what: &str) -> Option<Token> {
        match what.to_owned().to_lowercase().as_str() {
            "out" => Some(Token::Out),
            "float" => Some(Token::Float),
            "void" => Some(Token::Void),
            "return" => Some(Token::Return),
            _ => None,
        }
    }

    pub fn scan_token(&mut self) -> ScanningResult {
        let c = match self.advance() {
            Some(c) => c,
            None => {
                return Ok(ScanningProduct::Finished);
            }
        };

        let tok = |t| Ok(ScanningProduct::Token(t));

        match c {
            '(' => tok(Token::LeftParen),
            ')' => tok(Token::RightParen),
            '{' => tok(Token::LeftBrace),
            '}' => tok(Token::RightBrace),
            '=' => tok(Token::Equals),
            '-' => tok(Token::Minus),
            '+' => tok(Token::Plus),
            '/' => tok(Token::Slash),
            '*' => tok(Token::Star),

            '\n' => {
                self.line += 1;
                Ok(ScanningProduct::Skip)
            }
            c if c.is_whitespace() => Ok(ScanningProduct::Skip),
            c if c.is_alphabetic() => self.scan_identifier(c),
            c if c.is_numeric() => self.scan_numerics(c),
            c => {
                panic!("bad input {:?}", c);
            }
        }
    }

    pub fn scan_identifier(&mut self, begin: char) -> ScanningResult {
        let mut ident = String::new();
        ident.push(begin);
        while self.peek().unwrap().is_alphanumeric() {
            ident.push(self.advance().unwrap());
        }

        loop {
            match self.peek() {
                Some(c) if c.is_alphanumeric() => ident.push(self.advance().unwrap()),
                None => {
                    return Err(());
                }
                _ => {
                    break;
                }
            }
        }

        Ok(match self.keyword(&ident) {
            Some(k) => ScanningProduct::Token(k),
            None => ScanningProduct::Token(Token::Identifier(ident)),
        })
    }

    pub fn scan_numerics(&mut self, begin: char) -> ScanningResult {
        let mut text = String::new();
        text.push(begin);

        while self.peek().unwrap().is_numeric() {
            text.push(self.advance().unwrap());
        }

        if self.peek().unwrap() == '.' {
            text.push(self.advance().unwrap());
            while self.peek().unwrap().is_numeric() {
                text.push(self.advance().unwrap());
            }

            match text.parse::<f64>() {
                Ok(f) => Ok(ScanningProduct::Token(Token::FloatLiteral(f))),
                Err(_) => Err(()),
            }
        } else {
            match text.parse::<i64>() {
                Ok(i) => Ok(ScanningProduct::Token(Token::IntegerLiteral(i))),
                Err(_) => Err(()),
            }
        }
    }
}
