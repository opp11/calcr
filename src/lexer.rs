use std::str::Chars;
use std::iter::Peekable;
use errors::{CalcrResult, CalcrError};
use token::Token;
use token::TokenVal::*;
use ast::OpKind::*;

pub fn lex_equation(eq: &String) -> CalcrResult<Vec<Token>> {
    let mut lexer = Lexer {
        pos: 0,
        iter: eq.chars().peekable(),
    };
    lexer.lex_equation()
}

pub struct Lexer<'a> {
    pos: usize,
    iter: Peekable<Chars<'a>>,
}

impl<'a> Lexer<'a> {
    pub fn lex_equation(&mut self) -> CalcrResult<Vec<Token>> {
        let mut out = Vec::new();
        loop {
            self.consume_whitespace();
            let tok = match self.peek_char() {
                Some(ch) if ch.is_numeric() => try!(self.lex_number()),
                Some(ch) if ch.is_alphabetic() => try!(self.lex_name()),
                Some(ch) => try!(self.lex_single_char()),
                None => break,
            };
            out.push(tok);
        }
        Ok(out)
    }

    fn lex_number(&mut self) -> CalcrResult<Token> {
        let num_str = self.consume_while(|ch| ch.is_numeric() || ch == '.');
        if let Ok(num) = num_str.parse::<f64>() {
            Ok(Token {
                val: Num(num),
                span: (self.pos - num_str.len(), self.pos),
            })
        } else {
            Err(CalcrError {
                desc: format!("Invalid number: {}", num_str),
                span: Some((self.pos - num_str.len(), self.pos)),
            })
        }
    }

    fn lex_name(&mut self) -> CalcrResult<Token> {
        let name_str = self.consume_while(|ch| ch.is_alphabetic());
        let name_len = name_str.len();
        Ok(Token {
            val: Name(name_str),
            span: (self.pos - name_len, self.pos),
        })
    }

    fn lex_single_char(&mut self) -> CalcrResult<Token> {
        let val = match self.consume_char() {
            '+' => Op(Plus),
            '-' => Op(Minus),
            '*' => Op(Mult),
            '/' => Op(Div),
            '^' => Op(Pow),
            '!' => Op(Fact),
            '(' => ParenOpen,
            ')' => ParenClose,
            '|' => AbsDelim,
            ch => return Err(CalcrError {
                desc: format!("Invalid char: {}", ch),
                span: Some((self.pos, self.pos)),
            }),
        };
        Ok(Token {
            val: val,
            span: (self.pos, self.pos),
        })
    }

    /// Peeks at the next `char` and returns `Some` if one was found, or `None` if none are left
    fn peek_char(&mut self) -> Option<char> {
        self.iter.peek().map(|ch| *ch)
    }

    /// Consumes a `char` - thereby advanding `pos` - and returns it
    ///
    /// # Panics
    /// This function panics if there are no more chars to consume
    fn consume_char(&mut self) -> char {
        let ch = self.iter.next();
        self.pos += 1;
        ch.unwrap().to_lowercase().next().unwrap()
    }

    /// Consumes `char`s long as `pred` returns true and we are not eof
    ///
    /// The `char`s are returned as a `String`. Note that unlike `consume_char` this function will
    /// not panic.
    fn consume_while<F>(&mut self, pred: F) -> String where F: Fn(char) -> bool {
        let mut out = String::new();
        loop {
            match self.peek_char() {
                Some(ch) if pred(ch) => out.push(self.consume_char()),
                _ => break,
            }
        }
        out
    }

    /// Consumes any current whitespace
    fn consume_whitespace(&mut self) {
        self.consume_while(|ch| ch.is_whitespace());
    }

    /// Returns true if we are the end of input
    fn eof(&mut self) -> bool {
        self.iter.peek().is_none()
    }
}