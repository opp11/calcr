//! The parser is based on the following grammar
//!
//! Expression ==> "let" Name "=" Equation
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
//! Number     ==> Function "(" Equation ")"
//!             |  Constant
//!             |  Name
//!             |  "ans"
//!             |  "(" Equation ")"
//!             |  "|" Equation "|"
//!             |  NumLiteral
//!
//! Function   ==> "sin" | "cos" | "tan" | "asin" | "acos" | "atan" | "sqrt" | "abs" | "exp"
//!             |  "ln" | "log"
//!
//! Constant   ==> "pi" | "e" | "phi"

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
use token::TokVal::*;
use token::KeywordKind::*;

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
        if self.next_tok_is(Keyword(Let)) {
            self.consume_tok();
            if !self.toks_empty() {
                let Token { val: tok_val, span: tok_span } = self.consume_tok();
                match tok_val {
                    Name(ref name) if get_builtin_name(name).is_none() => {
                        if self.next_tok_is(Op(Assign)) {
                            let assign_tok = self.consume_tok();
                            let eq = try!(self.parse_equation());
                            let name_ast = Ast {
                                val: AstVal::Name(name.clone()),
                                span: tok_span,
                                branches: Leaf,
                            };
                            Ok(Ast {
                                val: AstVal::Op(Assign),
                                span: assign_tok.span,
                                branches: Binary(Box::new(name_ast), Box::new(eq)),
                            })
                        } else {
                            Err(CalcrError {
                                desc: "Expected assignment operator '=' after name".to_string(),
                                span: Some(tok_span),
                            })
                        }
                    },
                    Name(ref name) => Err(CalcrError {
                        desc: format!("Cannot assign to builtin name '{}'", name),
                        span: Some(tok_span),
                    }),
                    _ => Err(CalcrError {
                        desc: "Expected name to assign to".to_string(),
                        span: Some(tok_span),
                    }),
                }
            } else {
                Err(CalcrError {
                    desc: "Expected name to assign to".to_string(),
                    span: Some((self.end_pos, self.end_pos)),
                })
            }

        } else {
            self.parse_equation()
        }
    }

    fn parse_equation(&mut self) -> CalcrResult<Ast> {
        let mut lhs = try!(self.parse_product());
        while self.next_tok_matches(|val| *val == Op(Plus) || *val == Op(Minus)) {
            let Token { val: tok_val, span: tok_span } = self.consume_tok();
            let rhs = try!(self.parse_product());
            lhs = Ast {
                val: AstVal::Op(tok_val.op().unwrap()),
                span: tok_span,
                branches: Binary(Box::new(lhs), Box::new(rhs)),
            }
        }
        if self.next_tok_is(ParenClose) && self.paren_level < 1 {
            let Token { val: _, span: tok_span } = self.consume_tok();
            Err(CalcrError {
                desc: format!("Missing opening parentheses"),
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
        while self.next_tok_matches(|val| *val == Op(Mult) || *val == Op(Div)) {
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
        if self.next_tok_is(Op(Minus)) {
            let tok_span = self.consume_tok().span;
            let rhs = try!(self.parse_factor());
            Ok(Ast {
                val: AstVal::Op(Neg),
                span: tok_span,
                branches: Unary(Box::new(rhs)),
            })
        } else {
            let lhs = try!(self.parse_exponent());
            if self.next_tok_is(Op(Pow)) {
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

        while self.next_tok_is(Op(Fact)) {
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
                        if self.next_tok_is(ParenOpen) {
                            // since we know the next token is an open paren, we use
                            // this function to get its AST
                            let arg = try!(self.parse_equation());
                            Ok(Ast {
                                val: val,
                                span: tok_span,
                                branches: Unary(Box::new(arg)),
                            })
                        } else {
                            Err(CalcrError {
                                desc: "Missing opening parentheses after function".to_string(),
                                span: Some(tok_span),
                            })
                        }
                    } else {
                        Ok(Ast {
                            val: val,
                            span: tok_span,
                            branches: Leaf,
                        })
                    }
                },
                ParenOpen => {
                    self.paren_level += 1;
                    let eq = try!(self.parse_equation());
                    if !self.next_tok_is(ParenClose) {
                        Err(CalcrError {
                            desc: "Missing closing parentheses".to_string(),
                            span: Some(tok_span),
                        })
                    } else {
                        self.paren_level -= 1;
                        let close_paren_span = self.consume_tok().span;
                        Ok(Ast {
                            val: AstVal::Paren,
                            span: (tok_span.0, close_paren_span.1),
                            branches: Unary(Box::new(eq)),
                        })
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
                            branches: Unary(Box::new(eq)),
                        })
                    }
                },
                Num(num) => {
                    Ok(Ast {
                        val: AstVal::Num(num),
                        span: tok_span,
                        branches: Leaf,
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
        let ast = parse_equation(toks);
        assert_eq!(ast, Ok(Ast { val: AstVal::Num(2.0), span: (0, 1), branches: AstBranch::Leaf }));
    }

    #[test]
    fn constants() {
        assert_eq!(parse_equation(vec!(Token { val: TokVal::Name("pi".to_string()), span: (0, 2)})),
                   Ok(Ast { val: AstVal::Const(Pi), span: (0, 2), branches: AstBranch::Leaf }));

        assert_eq!(parse_equation(vec!(Token { val: TokVal::Name("π".to_string()), span: (0, 1)})),
                   Ok(Ast { val: AstVal::Const(Pi), span: (0, 1), branches: AstBranch::Leaf }));

        assert_eq!(parse_equation(vec!(Token { val: TokVal::Name("e".to_string()), span: (0, 1)})),
                   Ok(Ast { val: AstVal::Const(E), span: (0, 1), branches: AstBranch::Leaf }));

        assert_eq!(parse_equation(vec!(Token { val: TokVal::Name("phi".to_string()), span: (0, 3)})),
                   Ok(Ast { val: AstVal::Const(Phi), span: (0, 3), branches: AstBranch::Leaf }));

        assert_eq!(parse_equation(vec!(Token { val: TokVal::Name("ϕ".to_string()), span: (0, 1)})),
                   Ok(Ast { val: AstVal::Const(Phi), span: (0, 1), branches: AstBranch::Leaf }));
    }

    #[test]
    fn empty() {
        let toks = vec!();
        let err = parse_equation(toks);
        assert!(err.is_err());
    }
}