//! The parser works based on the following grammar
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
//!           |  { Digit }
//!
//! Function ==> "sin" | "cos" | "tan" | "asin" | "acos" | "atan" | "sqrt" | "qbrt" | "abs" | "exp"
//!           |  "ln" | "log"
//!
//! Constant ==> "pi" | "e" | "phi"
//!
//! Digit    ==> "0".."9"

use std::str::CharRange;
use errors::{CResult, CError};
use ast::{Ast, AstFunc};
use ast::AstBranch::*;
use ast::AstVal::*;
use ast::AstFunc::*;
use ast::AstOp::*;
use ast::AstConst::*;

#[derive(Debug)]
pub struct Parser {
    pos: usize,
    input: String,
    paren_level: u32,
    abs_level: u32,
}

impl Parser {
    pub fn new(input: String) -> Self {
        Parser {
            pos: 0,
            input: input,
            paren_level: 0,
            abs_level: 0,
        }
    }

    pub fn parse_equation(&mut self) -> CResult<Ast> {
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
            Err(CError {
                desc: format!("Missing opening parentheses"),
                span: (self.pos, self.pos),
            })
        } else if self.peek_char() == Some('|') && self.abs_level < 1 {
            Err(CError {
                desc: format!("Missing opening abs delimiter"),
                span: (self.pos, self.pos),
            })
        } else {
            Ok(lhs)
        }
    }

    fn parse_term(&mut self) -> CResult<Ast> {
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

    fn parse_product(&mut self) -> CResult<Ast> {
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

    fn parse_factor(&mut self) -> CResult<Ast> {
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

    fn parse_exponent(&mut self) -> CResult<Ast> {
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

    fn parse_number(&mut self) -> CResult<Ast> {
        match self.peek_char() {
            Some('(') => {
                // store the current pos in case we need to report a paren error
                let pre_pos = self.pos;
                self.consume_char();
                self.consume_whitespace();
                self.paren_level += 1;
                let eq = try!(self.parse_equation());

                if self.eof() || self.consume_char() != ')' {
                    Err(CError {
                        desc: "Missing closing parentheses".to_string(),
                        span: (pre_pos, pre_pos),
                    })
                } else {
                    self.paren_level -= 1;
                    Ok(eq)
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
                    Err(CError {
                        desc: "Missing closing abs delimiter".to_string(),
                        span: (pre_pos, pre_pos),
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
                let cnst = match cnst_str.as_slice() {
                    "pi" => Pi,
                    "e" => E,
                    "phi" => Phi,
                    _ => return Err(CError {
                        desc: format!("Invalid function or constant: {}", cnst_str),
                        span: (self.pos - cnst_str.len(), self.pos),
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
                    Err(CError {
                        desc: format!("Invalid number: {}", num_str),
                        span: (self.pos - num_str.len(), self.pos),
                    })
                }
            },
            _ => Err(CError {
                desc: format!("Expected number or constant"),
                span: (self.pos, self.pos),
            }),
        }
    }

    fn consume_function(&mut self) -> Option<AstFunc> {
        let pre_pos = self.pos;
        match self.consume_while(|ch| ch.is_alphabetic()).as_slice() {
            "sin" => Some(Sin),
            "cos" => Some(Cos),
            "tan" => Some(Tan),
            "asin" => Some(Asin),
            "acos" => Some(Acos),
            "atan" => Some(Atan),
            "sqrt" => Some(Sqrt),
            "qbrt" => Some(Qbrt),
            "abs" => Some(Abs),
            "exp" => Some(Exp),
            "ln" => Some(Ln),
            "log" => Some(Log),
            _ => {
                // no function found, so restore the previous position
                self.pos = pre_pos;
                None
            },
        }
    }

    /// Peeks at the next `char` and returns `Some` if one was found, or `None` if none are left
    fn peek_char(&self) -> Option<char> {
        if self.eof() {
            None
        } else {
            Some(self.input.char_at(self.pos).to_lowercase())
        }
    }

    /// Consumes a `char` - thereby advanding `pos` - and returns it
    ///
    /// # Panics
    /// This function panics if there are no more chars to consume
    fn consume_char(&mut self) -> char {
        let CharRange { ch, next } = self.input.char_range_at(self.pos);
        self.pos = next;
        ch.to_lowercase()
    }

    /// Consumes `char`s long as `pred` returns true and we are not eof
    ///
    /// The `char`s are returned as a `String`. Note that unlike `consume_char` this function will
    /// panic.
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
    fn eof(&self) -> bool {
        self.pos >= self.input.len()
    }
}