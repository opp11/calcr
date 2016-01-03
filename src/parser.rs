//! The parser is based on the following grammar
//!
//! Expression ==> Name "=" Equation
//!             |  Equation
//!
//! Equation   ==> Product { "+" Product }
//!             |  Product { "-" Product }
//!
//! Product    ==> Factor { "*" Factor }
//!             |  Factor { "/" Factor }
//!
//! Factor     ==> "-" Factor
//!             |  Exponent { "^" Factor }
//!
//! Exponent   ==> Number { "!" }
//!
//! Number     ==> Function OpenDelim Equation CloseDelim
//!             |  Constant
//!             |  Name
//!             |  "ans"
//!             |  OpenDelim Equation CloseDelim
//!             |  "|" Equation "|"
//!             |  NumLiteral
//!
//! Function   ==> "sin" | "cos" | "tan" | "asin" | "acos" | "atan" | "sqrt" | "abs" | "exp"
//!             |  "ln" | "log"
//!
//! Constant   ==> "pi" | "π" | "e" | "phi" | "ϕ" | "ans"
//!
//! OpenDelim  ==> "(" | "[" | "{"
//!
//! CloseDelim ==> ")" | "]" | "}"

use std::vec::IntoIter;
use std::iter::Peekable;
use errors::{CalcrResult, CalcrError};
use ast::Ast;
use ast::AstVal;
use ast::OpKind as AstOp;
//use ast::AstBranch::*;
use ast::FuncKind::*;
use ast::ConstKind::*;
use token::Token;
use token::OpKind as TokOp;
use token::TokVal;
use token::TokVal::*;

pub fn parse_tokens(tokens: Vec<Token>) -> CalcrResult<Ast> {
    let end_pos = tokens.last().and_then(|tok| Some(tok.span.1)).unwrap_or(0);
    let mut parser = Parser {
        iter: tokens.into_iter().peekable(),
        paren_level: 0,
        abs_level: 0,
        end_pos: end_pos,
    };
    parser.parse_expression()
}

fn get_builtin_name(name: &String) -> Option<AstVal> {
    match name.as_ref() {
        "ans" => Some(AstVal::LastResult),
        "pi" | "π" => Some(AstVal::Const(Pi)),
        "e" => Some(AstVal::Const(E)),
        "phi" | "ϕ" => Some(AstVal::Const(Phi)),
        "cos" => Some(AstVal::Func(Cos)),
        "sin" => Some(AstVal::Func(Sin)),
        "tan" => Some(AstVal::Func(Tan)),
        "asin" => Some(AstVal::Func(Asin)),
        "acos" => Some(AstVal::Func(Acos)),
        "atan" => Some(AstVal::Func(Atan)),
        "sqrt" | "√" => Some(AstVal::Func(Sqrt)),
        "abs" => Some(AstVal::Func(Abs)),
        "exp" => Some(AstVal::Func(Exp)),
        "ln" => Some(AstVal::Func(Ln)),
        "log" => Some(AstVal::Func(Log)),
        _ => None
    }
}

pub struct Parser {
    iter: Peekable<IntoIter<Token>>,
    paren_level: u32,
    abs_level: u32,
    end_pos: usize,
}

impl Parser {
    fn parse_expression(&mut self) -> CalcrResult<Ast> {
        let eq = try!(self.parse_equation());
        if self.toks_empty() {
            Ok(eq)
        } else if self.next_tok_is(Op(TokOp::Assign)) {
            self.consume_tok();
            if let AstVal::Name(_) = eq.val {
                let rhs = try!(self.parse_equation());
                Ok(Ast {
                    val: AstVal::Op(AstOp::Assign),
                    span: (eq.span.0, rhs.span.1),
                    branches: vec!(eq, rhs)
                })
            } else {
                let assign_target = match eq {
                    Ast { val: AstVal::Func(_), span: _, branches: _ } => "function",
                    Ast { val: AstVal::Const(_), span: _, branches: _ } => "constant",
                    Ast { val: AstVal::Num(_), span: _, branches: _ } => "number",
                    Ast { val: AstVal::LastResult, span: _, branches: _ } => "constant",
                    _ => "equtation", // TODO: Make this case more nuanced
                };
                Err(CalcrError {
                    desc: format!("Cannot assign to {}", assign_target),
                    span: Some(eq.get_total_span()),
                })
            }
        } else {
            let tok = self.consume_tok();
            Err(CalcrError {
                desc: "Expected operator".to_string(),
                span: Some(tok.span),
            })
        }
    }

    fn parse_equation(&mut self) -> CalcrResult<Ast> {
        let mut lhs = try!(self.parse_product());
        while self.next_tok_matches(|val| *val == Op(TokOp::Plus) || *val == Op(TokOp::Minus)) {
            let Token { val: tok_val, span: tok_span } = self.consume_tok();
            let rhs = try!(self.parse_product());
            lhs = Ast {
                val: AstVal::Op(tok_val.op().unwrap().into()),
                span: tok_span,
                branches: vec!(lhs, rhs),
            }
        }
        if self.next_tok_matches(|val| val.is_close_delim()) && self.paren_level < 1 {
            let Token { val: _, span: tok_span } = self.consume_tok();
            Err(CalcrError {
                desc: format!("Missing matching opening delimiter"),
                span: Some(tok_span),
            })
        } else if self.next_tok_is(AbsDelim) && self.abs_level < 1 {
            let Token { val: _, span: tok_span } = self.consume_tok();
            Err(CalcrError {
                desc: format!("Missing opening abs delimiter"),
                span: Some(tok_span),
            })
        } else {
            Ok(lhs)
        }
    }

    fn parse_product(&mut self) -> CalcrResult<Ast> {
        let mut lhs = try!(self.parse_factor());
        while self.next_tok_matches(|val| *val == Op(TokOp::Mult) || *val == Op(TokOp::Div)) {
            let Token { val: tok_val, span: tok_span } = self.consume_tok();
            let rhs = try!(self.parse_factor());
            lhs = Ast {
                val: AstVal::Op(tok_val.op().unwrap().into()),
                span: tok_span,
                branches: vec!(lhs, rhs),
            };
        }
        Ok(lhs)
    }

    fn parse_factor(&mut self) -> CalcrResult<Ast> {
        // when we lex we only store `Minus`s since we do not have any context there,
        // however we know if we see a `Minus` now, then it is a `Neg`.
        if self.next_tok_is(Op(TokOp::Minus)) {
            let tok_span = self.consume_tok().span;
            let rhs = try!(self.parse_factor());
            Ok(Ast {
                val: AstVal::Op(AstOp::Neg),
                span: tok_span,
                branches: vec!(rhs),
            })
        } else {
            let lhs = try!(self.parse_exponent());
            if self.next_tok_is(Op(TokOp::Pow)) {
                let tok_span = self.consume_tok().span;
                let rhs = try!(self.parse_factor());
                Ok(Ast {
                    val: AstVal::Op(AstOp::Pow),
                    span: tok_span,
                    branches: vec!(lhs, rhs),
                })
            } else {
                Ok(lhs)
            }
        }
    }

    fn parse_exponent(&mut self) -> CalcrResult<Ast> {
        let mut out = try!(self.parse_number());

        while self.next_tok_is(Op(TokOp::Fact)) {
            let tok_span = self.consume_tok().span;
            out = Ast {
                val: AstVal::Op(AstOp::Fact),
                span: tok_span,
                branches: vec!(out),
            };
        }
        Ok(out)
    }

    fn parse_number(&mut self) -> CalcrResult<Ast> {
        if self.toks_empty() {
            Err(CalcrError {
                desc: format!("Expected number or constant"),
                span: Some((self.end_pos, self.end_pos)),
            })
        } else {
            let Token { val: tok_val, span: tok_span } = self.consume_tok();
            match tok_val {
                Name(ref name) => {
                    let val = match get_builtin_name(name) {
                        Some(val) => val,
                        None => AstVal::Name(name.clone()),
                    };
                    if let AstVal::Func(_) = val {
                        // it's a function so we need to grab its argument
                        if self.next_tok_matches(|val| val.is_open_delim()) {
                            // since we know the next token is an open paren, we use
                            // this function to get its AST
                            let arg = try!(self.parse_number());
                            Ok(Ast {
                                val: val,
                                span: tok_span,
                                branches: vec!(arg) ,
                            })
                        } else {
                            Err(CalcrError {
                                desc: "Missing opening delimiter after function".to_string(),
                                span: Some(tok_span),
                            })
                        }
                    } else {
                        Ok(Ast {
                            val: val,
                            span: tok_span,
                            branches: vec!(),
                        })
                    }
                },
                OpenDelim(kind) => {
                    self.paren_level += 1;
                    let eq = try!(self.parse_equation());
                    if !self.next_tok_is(CloseDelim(kind)) {
                        Err(CalcrError {
                            desc: "Missing matching closing delimiter".to_string(),
                            span: Some(tok_span),
                        })
                    } else {
                        self.consume_tok();
                        self.paren_level -= 1;
                        Ok(eq)
                    }
                },
                AbsDelim => {
                    self.abs_level += 1;
                    let eq = try!(self.parse_equation());
                    if !self.next_tok_is(AbsDelim) {
                        Err(CalcrError {
                            desc: "Missing closing abs delimiter".to_string(),
                            span: Some(tok_span),
                        })
                    } else {
                        self.abs_level -= 1;
                        let close_delim_span = self.consume_tok().span;
                        Ok(Ast {
                            val: AstVal::Func(Abs),
                            span: (tok_span.0, close_delim_span.1),
                            branches: vec!(eq),
                        })
                    }
                },
                Num(num) => {
                    Ok(Ast {
                        val: AstVal::Num(num),
                        span: tok_span,
                        branches: vec!(),
                    })
                },
                _ => Err(CalcrError {
                    desc: format!("Expected number or constant"),
                    span: Some(tok_span),
                }),
            }
        }
    }

    /// Peeks at the next token and check whether its values is equal to `val`
    fn next_tok_is(&mut self, val: TokVal) -> bool {
        self.next_tok_matches(|v| *v == val)
    }

    /// Peeks at the next token and checks whether its value makes `pred` true
    fn next_tok_matches<F>(&mut self, pred: F) -> bool where F: FnOnce(&TokVal) -> bool {
        self.iter.peek().map_or(false, |ref tok| pred(&tok.val))
    }

    /// Checks if we have run out of `Token`s to parse
    fn toks_empty(&mut self) -> bool {
        self.iter.peek().is_none()
    }

    /// Consumes a `Token` - thereby advanding `pos` - and returns it
    ///
    /// # Panics
    /// This function panics if there are no more `Token`s to consume
    fn consume_tok(&mut self) -> Token {
        let tok = self.iter.next();
        tok.unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use token::Token;
    use token::TokVal;
    use ast::Ast;
    use ast::AstVal;
    use ast::AstBranch;
    use ast::ConstKind::*;

    #[test]
    fn single_num() {
        let toks = vec!(Token { val: TokVal::Num(2.0), span: (0, 1) });
        let ast = parse_tokens(toks);
        assert_eq!(ast, Ok(Ast { val: AstVal::Num(2.0), span: (0, 1), branches: AstBranch::Leaf }));
    }

    #[test]
    fn constants() {
        assert_eq!(parse_tokens(vec!(Token { val: TokVal::Name("pi".to_string()), span: (0, 2)})),
                   Ok(Ast { val: AstVal::Const(Pi), span: (0, 2), branches: AstBranch::Leaf }));

        assert_eq!(parse_tokens(vec!(Token { val: TokVal::Name("π".to_string()), span: (0, 1)})),
                   Ok(Ast { val: AstVal::Const(Pi), span: (0, 1), branches: AstBranch::Leaf }));

        assert_eq!(parse_tokens(vec!(Token { val: TokVal::Name("e".to_string()), span: (0, 1)})),
                   Ok(Ast { val: AstVal::Const(E), span: (0, 1), branches: AstBranch::Leaf }));

        assert_eq!(parse_tokens(vec!(Token { val: TokVal::Name("phi".to_string()), span: (0, 3)})),
                   Ok(Ast { val: AstVal::Const(Phi), span: (0, 3), branches: AstBranch::Leaf }));

        assert_eq!(parse_tokens(vec!(Token { val: TokVal::Name("ϕ".to_string()), span: (0, 1)})),
                   Ok(Ast { val: AstVal::Const(Phi), span: (0, 1), branches: AstBranch::Leaf }));
    }

    #[test]
    fn empty() {
        let toks = vec!();
        let err = parse_tokens(toks);
        assert!(err.is_err());
    }
}