//! The parser is based on the following grammar
//!
//! Equation ==> Term { "+" Term }
//!           |  Term { "-" Term }
//!
//! Term     ==> Function Product
//!           |  Product
//!
//! Product  ==> Factor { "*" Factor }
//!           |  Factor { "/" Factor }
//!
//! Factor   ==> "-" Factor
//!           |  Exponent { "^" Factor }
//!
//! Exponent ==> Number { "!" }
//!
//! Number   ==> "(" Equation ")"
//!           |  "|" Equation "|"
//!           |  Constant
//!           |  Digit { Digit } | Digit { Digit } "." { Digit }
//!
//! Function ==> "sin" | "cos" | "tan" | "asin" | "acos" | "atan" | "sqrt" | "abs" | "exp"
//!           |  "ln" | "log"
//!
//! Constant ==> "pi" | "e" | "phi"
//!
//! Digit    ==> "0".."9"

use std::str::Chars;
use std::iter::Peekable;
use errors::{CalcrResult, CalcrError};
use ast::{Ast, AstFunc};
use ast::AstBranch::*;
use ast::AstVal::*;
use ast::AstFunc::*;
use ast::AstOp::*;
use ast::AstConst::*;

pub fn parse_equation(eq: &String) -> CalcrResult<Ast> {
	let mut parser = Parser {
		pos: 0,
		iter: eq.chars().peekable(),
		paren_level: 0,
		abs_level: 0,
	};
	parser.parse_equation()
}

pub struct Parser<'a> {
    pos: usize,
    iter: Peekable<Chars<'a>>,
    paren_level: u32,
    abs_level: u32,
}

impl<'a> Parser<'a> {
    fn parse_equation(&mut self) -> CalcrResult<Ast> {
        let mut lhs = try!(self.parse_term());
        self.consume_whitespace();
        while self.peek_char() == Some('+') || self.peek_char() == Some('-') {
            let op_pos = self.pos;
            let op = if self.consume_char() == '+' {
                Plus
            } else {
                Minus
            };
            self.consume_whitespace();
            let rhs = try!(self.parse_term());
            lhs = Ast {
                val: Op(op),
                span: (op_pos, op_pos),
                branches: Binary(Box::new(lhs), Box::new(rhs)),
            };
            self.consume_whitespace();
        }
        if self.peek_char() == Some(')') && self.paren_level < 1 {
            Err(CalcrError {
                desc: format!("Missing opening parentheses"),
                span: Some((self.pos, self.pos)),
            })
        } else if self.peek_char() == Some('|') && self.abs_level < 1 {
            Err(CalcrError {
                desc: format!("Missing opening abs delimiter"),
                span: Some((self.pos, self.pos)),
            })
        } else {
            Ok(lhs)
        }
    }

    fn parse_term(&mut self) -> CalcrResult<Ast> {
        let begin_pos = self.pos;
        if let Some(func) = self.consume_function() {
            let end_pos = self.pos;
            self.consume_whitespace();
            let arg = try!(self.parse_product());
            Ok(Ast {
                val: Func(func),
                span: (begin_pos, end_pos),
                branches: Unary(Box::new(arg)),
            })
        } else {
            self.parse_product()
        }
    }

    fn parse_product(&mut self) -> CalcrResult<Ast> {
        let mut lhs = try!(self.parse_factor());
        self.consume_whitespace();
        while self.peek_char() == Some('*') || self.peek_char() == Some('/') {
            let op_pos = self.pos;
            let op = if self.consume_char() == '*' {
                Mult
            } else {
                Div
            };
            self.consume_whitespace();
            let rhs = try!(self.parse_factor());
            lhs = Ast {
                val: Op(op),
                span: (op_pos, op_pos),
                branches: Binary(Box::new(lhs), Box::new(rhs)),
            };
            self.consume_whitespace();
        }
        Ok(lhs)
    }

    fn parse_factor(&mut self) -> CalcrResult<Ast> {
        if self.peek_char() == Some('-') {
            let op_pos = self.pos;
            self.consume_char();
            let rhs = try!(self.parse_factor());
            Ok(Ast {
                val: Op(Neg),
                span: (op_pos, op_pos),
                branches: Unary(Box::new(rhs)),
            })
        } else {
            let lhs = try!(self.parse_exponent());
            if self.peek_char() == Some('^') {
                let op_pos = self.pos;
                self.consume_char();
                let rhs = try!(self.parse_factor());
                Ok(Ast {
                    val: Op(Pow),
                    span: (op_pos, op_pos),
                    branches: Binary(Box::new(lhs), Box::new(rhs)),
                })
            } else {
                Ok(lhs)
            }
        }
    }

    fn parse_exponent(&mut self) -> CalcrResult<Ast> {
        let mut out = try!(self.parse_number());
        self.consume_whitespace();

        // we don't parse the factorial signs (`!`) using recursion, since we need to put the
        // current `out` at the bottum of the tree at each step, so it is easier if we have access
        // to each node as we create them.
        while self.peek_char() == Some('!') {
            let op_pos = self.pos;
            self.consume_char();
            self.consume_whitespace();
            out = Ast {
                val: Op(Fact),
                span: (op_pos, op_pos),
                branches: Unary(Box::new(out)),
            };
        }
        Ok(out)
    }

    fn parse_number(&mut self) -> CalcrResult<Ast> {
        match self.peek_char() {
            Some('(') => {
                // store the current pos in case we need to report a paren error
                let pre_pos = self.pos;
                self.consume_char();
                self.consume_whitespace();
                self.paren_level += 1;
                let eq = try!(self.parse_equation());

                if self.eof() || self.consume_char() != ')' {
                    Err(CalcrError {
                        desc: "Missing closing parentheses".to_string(),
                        span: Some((pre_pos, pre_pos)),
                    })
                } else {
                    self.paren_level -= 1;
                    Ok(Ast {
                        val: Paren,
                        span: (pre_pos, self.pos),
                        branches: Unary(Box::new(eq)),
                    })
                }
            },
            Some('|') => {
                // store the current pos in case we need to report an abs delim error
                let pre_pos = self.pos;
                self.consume_char();
                self.consume_whitespace();
                self.abs_level += 1;
                let eq = try!(self.parse_equation());

                if self.eof() || self.consume_char() != '|' {
                    Err(CalcrError {
                        desc: "Missing closing abs delimiter".to_string(),
                        span: Some((pre_pos, pre_pos)),
                    })
                } else {
                    self.abs_level -= 1;
                    Ok(Ast {
                        val: Func(Abs),
                        span: (pre_pos, pre_pos),
                        branches: Unary(Box::new(eq)),
                    })
                }
            },
            Some(ch) if ch.is_alphabetic() => {
                let cnst_str = self.consume_while(|ch| ch.is_alphabetic());
                let cnst = match cnst_str.as_ref() {
                    "pi" => Pi,
                    "e" => E,
                    "phi" => Phi,
                    _ => return Err(CalcrError {
                        desc: format!("Invalid function or constant: {}", cnst_str),
                        span: Some((self.pos - cnst_str.len(), self.pos)),
                    }),
                };
                Ok(Ast {
                    val: Const(cnst),
                    span: (self.pos - cnst_str.len(), self.pos),
                    branches: Leaf,
                })
            },
            Some(ch) if ch.is_numeric() => {
                let num_str = self.consume_while(|ch| ch.is_numeric() || ch == '.');
                if let Ok(num) = num_str.parse::<f64>() {
                    Ok(Ast {
                        val: Num(num),
                        span: (self.pos - num_str.len(), self.pos),
                        branches: Leaf,
                    })
                } else {
                    Err(CalcrError {
                        desc: format!("Invalid number: {}", num_str),
                        span: Some((self.pos - num_str.len(), self.pos)),
                    })
                }
            },
            _ => Err(CalcrError {
                desc: format!("Expected number or constant"),
                span: Some((self.pos, self.pos)),
            }),
        }
    }

    fn consume_function(&mut self) -> Option<AstFunc> {
        let pre_pos = self.pos;
        let pre_iter = self.iter.clone();
        match self.consume_while(|ch| ch.is_alphabetic()).as_ref() {
            "sin" => Some(Sin),
            "cos" => Some(Cos),
            "tan" => Some(Tan),
            "asin" => Some(Asin),
            "acos" => Some(Acos),
            "atan" => Some(Atan),
            "sqrt" => Some(Sqrt),
            "abs" => Some(Abs),
            "exp" => Some(Exp),
            "ln" => Some(Ln),
            "log" => Some(Log),
            _ => {
                // no function found, so restore the previous position
                self.iter = pre_iter;
                self.pos = pre_pos;
                None
            },
        }
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