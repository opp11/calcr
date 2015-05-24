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

use std::vec::IntoIter;
use std::iter::Peekable;
use errors::{CalcrResult, CalcrError};
use ast::Ast;
use ast::AstVal;
use ast::AstBranch::*;
use ast::FuncKind::*;
use ast::ConstKind::*;
use ast::OpKind::*;
use token::Token;
use token::TokVal;

pub fn parse_equation(tokens: Vec<Token>) -> CalcrResult<Ast> {
    let end_pos = tokens.last().and_then(|tok| Some(tok.span.1)).unwrap_or(0);
    let mut parser = Parser {
        iter: tokens.into_iter().peekable(),
        paren_level: 0,
        abs_level: 0,
        end_pos: end_pos,
    };
    parser.parse_equation()
}

pub struct Parser {
    iter: Peekable<IntoIter<Token>>,
    paren_level: u32,
    abs_level: u32,
    end_pos: usize,
}

impl Parser {
    fn parse_equation(&mut self) -> CalcrResult<Ast> {
        let mut lhs = try!(self.parse_term());
        while self.peek_tok_val() == Some(TokVal::Op(Plus)) ||
              self.peek_tok_val() == Some(TokVal::Op(Minus)) {
            let Token { val: tok_val, span: tok_span } = self.consume_tok();
            let rhs = try!(self.parse_term());
            lhs = Ast {
                val: AstVal::Op(tok_val.op().unwrap()),
                span: tok_span,
                branches: Binary(Box::new(lhs), Box::new(rhs)),
            }
        }
        if self.peek_tok_val() == Some(TokVal::ParenClose) && self.paren_level < 1 {
            let Token { val: _, span: tok_span } = self.consume_tok();
            Err(CalcrError {
                desc: format!("Missing opening parentheses"),
                span: Some(tok_span),
            })
        } else if self.peek_tok_val() == Some(TokVal::AbsDelim) && self.abs_level < 1 {
            let Token { val: _, span: tok_span } = self.consume_tok();
            Err(CalcrError {
                desc: format!("Missing opening abs delimiter"),
                span: Some(tok_span),
            })
        } else {
            Ok(lhs)
        }
    }

    fn parse_term(&mut self) -> CalcrResult<Ast> {
        // check if we have a function
        let func_opt = match self.peek_tok_val() {
            // TODO: Make this not horrible, since the compiler kept bugging me
            Some(TokVal::Name(ref name)) if *name == "cos".to_string() => Some(Cos),
            Some(TokVal::Name(ref name)) if *name == "sin".to_string() => Some(Sin),
            Some(TokVal::Name(ref name)) if *name == "tan".to_string() => Some(Tan),
            Some(TokVal::Name(ref name)) if *name == "asin".to_string() => Some(Asin),
            Some(TokVal::Name(ref name)) if *name == "acos".to_string() => Some(Acos),
            Some(TokVal::Name(ref name)) if *name == "atan".to_string() => Some(Atan),
            Some(TokVal::Name(ref name)) if *name == "sqrt".to_string() => Some(Sqrt),
            Some(TokVal::Name(ref name)) if *name == "abs".to_string() => Some(Abs),
            Some(TokVal::Name(ref name)) if *name == "exp".to_string() => Some(Exp),
            Some(TokVal::Name(ref name)) if *name == "ln".to_string() => Some(Ln),
            Some(TokVal::Name(ref name)) if *name == "log".to_string() => Some(Log),
            _ => None,
        };

        if let Some(func) = func_opt {
            let Token { val: _, span: tok_span } = self.consume_tok();
            let arg = try!(self.parse_product());
            Ok(Ast {
                val: AstVal::Func(func),
                span: tok_span,
                branches: Unary(Box::new(arg)),
            })
        } else {
            self.parse_product()
        }
    }

    fn parse_product(&mut self) -> CalcrResult<Ast> {
        let mut lhs = try!(self.parse_factor());
        while self.peek_tok_val() == Some(TokVal::Op(Mult)) ||
              self.peek_tok_val() == Some(TokVal::Op(Div)) {
            let Token { val: tok_val, span: tok_span } = self.consume_tok();
            let rhs = try!(self.parse_factor());
            lhs = Ast {
                val: AstVal::Op(tok_val.op().unwrap()),
                span: tok_span,
                branches: Binary(Box::new(lhs), Box::new(rhs)),
            };
        }
        Ok(lhs)
    }

    fn parse_factor(&mut self) -> CalcrResult<Ast> {
        // when we lex we only store `Minus`s since we do not have any context there,
        // however we know if we see a `Minus` now, then it is a `Neg`.
        if self.peek_tok_val() == Some(TokVal::Op(Minus)) {
            let tok_span = self.consume_tok().span;
            let rhs = try!(self.parse_factor());
            Ok(Ast {
                val: AstVal::Op(Neg),
                span: tok_span,
                branches: Unary(Box::new(rhs)),
            })
        } else {
            let lhs = try!(self.parse_exponent());
            if self.peek_tok_val() == Some(TokVal::Op(Pow)) {
                let tok_span = self.consume_tok().span;
                let rhs = try!(self.parse_factor());
                Ok(Ast {
                    val: AstVal::Op(Pow),
                    span: tok_span,
                    branches: Binary(Box::new(lhs), Box::new(rhs)),
                })
            } else {
                Ok(lhs)
            }
        }
    }

    fn parse_exponent(&mut self) -> CalcrResult<Ast> {
        let mut out = try!(self.parse_number());

        while self.peek_tok_val() == Some(TokVal::Op(Fact)) {
            let tok_span = self.consume_tok().span;
            out = Ast {
                val: AstVal::Op(Fact),
                span: tok_span,
                branches: Unary(Box::new(out)),
            };
        }
        Ok(out)
    }

    fn parse_number(&mut self) -> CalcrResult<Ast> {
        match self.peek_tok_val() {
            Some(TokVal::ParenOpen) => {
                // store the current pos in case we need to report a paren error
                let open_paren_span = self.consume_tok().span;
                self.paren_level += 1;
                let eq = try!(self.parse_equation());

                if self.peek_tok_val() != Some(TokVal::ParenClose) {
                    Err(CalcrError {
                        desc: "Missing closing parentheses".to_string(),
                        span: Some(open_paren_span),
                    })
                } else {
                    self.paren_level -= 1;
                    let close_paren_span = self.consume_tok().span;
                    Ok(Ast {
                        val: AstVal::Paren,
                        span: (open_paren_span.0, close_paren_span.1),
                        branches: Unary(Box::new(eq)),
                    })
                }
            },
            Some(TokVal::AbsDelim) => {
                // store the current pos in case we need to report a paren error
                let open_delim_span = self.consume_tok().span;
                self.abs_level += 1;
                let eq = try!(self.parse_equation());

                if self.peek_tok_val() != Some(TokVal::AbsDelim) {
                    Err(CalcrError {
                        desc: "Missing closing abs delimiter".to_string(),
                        span: Some(open_delim_span),
                    })
                } else {
                    self.abs_level -= 1;
                    let close_delim_span = self.consume_tok().span;
                    Ok(Ast {
                        val: AstVal::Func(Abs),
                        span: (open_delim_span.0, close_delim_span.1),
                        branches: Unary(Box::new(eq)),
                    })
                }
            },
            Some(TokVal::Name(ref name)) => {
                // at this point any Name, HAS to be a known constant
                let cnst = match name.as_ref() {
                    "pi" => Pi,
                    "e" => E,
                    "phi" => Phi,
                    _ => return Err(CalcrError {
                        desc: format!("Invalid function or constant: {}", name),
                        span: Some(self.consume_tok().span),
                    }),
                };
                let span = self.consume_tok().span;
                Ok(Ast {
                    val: AstVal::Const(cnst),
                    span: span,
                    branches: Leaf,
                })
            },
            Some(TokVal::Num(num)) => {
                let span = self.consume_tok().span;
                Ok(Ast {
                    val: AstVal::Num(num),
                    span: span,
                    branches: Leaf,
                })
            },
            Some(_) => Err(CalcrError {
                desc: format!("Expected number or constant"),
                span: Some(self.consume_tok().span),
            }),
            None => Err(CalcrError {
                desc: format!("Expected number or constant"),
                span: Some((self.end_pos, self.end_pos)),
            }),
        }
    }

    /// Peeks at the next `Token` and returns `Some` if one was found, or `None` if none are left
    fn peek_tok_val(&mut self) -> Option<TokVal> {
        self.iter.peek().and_then(|ref tok| Some(tok.val.clone()))
    }

    /// Consumes a `Token` - thereby advanding `pos` - and returns it
    ///
    /// # Panics
    /// This function panics if there are no more chars to consume
    fn consume_tok(&mut self) -> Token {
        let tok = self.iter.next();
        tok.unwrap()
    }
}