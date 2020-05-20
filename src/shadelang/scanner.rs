use crate::shadelang::ast::{Position, Spanned};

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Out,
    In,
    
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
    Token(Spanned<Token>),
}

#[derive(Clone, Debug)]
pub enum ScanningError {
    UnexpectedCharacter(Spanned<char>),
    InvalidLiteral(Spanned<()>),
    UnexpectedEndOfFile,
}

type ScanningResult = Result<ScanningProduct, ScanningError>;

pub struct Scanner<I: Iterator<Item = char>> {
    input: I,
    line: u32,
    offset: u32,
    peeked: Option<char>,
}

impl<I: Iterator<Item = char>> Scanner<I> {
    pub fn new(input: I) -> Self {
        Scanner {
            input,
            line: 1,
            offset: 0,
            peeked: None,
        }
    }

    pub fn scan_all(mut self) -> Result<Vec<Spanned<Token>>, ScanningError> {
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
        self.offset += 1;
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
            "in" => Some(Token::In),
            "float" => Some(Token::Float),
            "void" => Some(Token::Void),
            "return" => Some(Token::Return),
            _ => None,
        }
    }

    pub fn position(&self) -> Position {
        Position {
            line: self.line,
            offset: Some(self.offset),
        }
    }

    pub fn scan_token(&mut self) -> ScanningResult {
        let from = self.position();
        let c = match self.advance() {
            Some(c) => c,
            None => {
                return Ok(ScanningProduct::Finished);
            }
        };

        let tok = |t| {
            let to = self.position();
            Ok(ScanningProduct::Token(Spanned::new(t, from, to)))
        };

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
                return Err(ScanningError::UnexpectedCharacter(Spanned::new(
                    c,
                    from,
                    self.position(),
                )))
            }
        }
    }

    pub fn scan_identifier(&mut self, begin: char) -> ScanningResult {
        let mut from = self.position();
        from.offset = from.offset.map(|v| v - 1);

        let mut ident = String::new();
        ident.push(begin);

        loop {
            match self.peek() {
                Some(c) if c.is_alphanumeric() || c == '_'  => ident.push(self.advance().unwrap()),
                None => {
                    return Err(ScanningError::UnexpectedEndOfFile);
                }
                _ => {
                    break;
                }
            }
        }

        let to = self.position();

        Ok(match self.keyword(&ident) {
            Some(k) => ScanningProduct::Token(Spanned::new(k, from, to)),
            None => ScanningProduct::Token(Spanned::new(Token::Identifier(ident), from, to)),
        })
    }

    pub fn scan_numerics(&mut self, begin: char) -> ScanningResult {
        let mut from = self.position();
        from.offset = from.offset.map(|v| v - 1);

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
            let to = self.position();

            match text.parse::<f64>() {
                Ok(f) => Ok(ScanningProduct::Token(Spanned::new(
                    Token::FloatLiteral(f),
                    from,
                    to,
                ))),
                Err(_) => Err(ScanningError::InvalidLiteral(Spanned::new((), from, to))),
            }
        } else {
            let to = self.position();
            match text.parse::<i64>() {
                Ok(i) => Ok(ScanningProduct::Token(Spanned::new(
                    Token::IntegerLiteral(i),
                    from,
                    to,
                ))),
                Err(_) => Err(ScanningError::InvalidLiteral(Spanned::new((), from, to))),
            }
        }
    }
}
