use std::f64;
use std::collections::HashMap;
use ast::{Ast, ConstKind, FuncKind, OpKind};
use ast::AstVal::*;
use ast::FuncKind::*;
use ast::OpKind::*;
use ast::ConstKind::*;
use lexer::lex_equation;
use parser::parse_tokens;
use errors::{CalcrResult, CalcrError};

pub struct Interpreter {
    vars: HashMap<String, f64>,
    last_result: f64,
}

impl Interpreter {
    pub fn new() -> Interpreter {
        Interpreter {
            vars: HashMap::new(),
            last_result: 0.0,
        }
    }

    pub fn eval_expression(&mut self, expr: &String) -> CalcrResult<Option<f64>> {
        let toks = try!(lex_equation(expr));
        let ast = try!(parse_tokens(toks));
        let result = self.eval_expr(&ast);
        // if we got an actual number as the result, then store it for later use
        if let Ok(Some(ref res)) = result {
            self.last_result = *res;
        }
        result
    }

    fn eval_expr(&mut self, ast: &Ast) -> CalcrResult<Option<f64>> {
        if ast.val == Op(Assign) {
            let (lhs, rhs) = try!(ast.get_binary_branches());
            if let Name(ref name) = lhs.val {
                let val = try!(self.eval_eq(rhs));
                self.vars.insert(name.clone(), val);
                Ok(None)
            } else {
                Err(CalcrError {
                    desc: "Interal error - expected Assign to have Name in left branch"
                          .to_string(),
                    span: None,
                })
            }
        } else {
            self.eval_eq(ast).map(|val| Some(val))
        }
    }

    fn eval_eq(&mut self, ast: &Ast) -> CalcrResult<f64> {
        match ast.val {
            Func(ref f) => self.eval_func(f, ast),
            Op(ref o) => self.eval_op(o, ast),
            Const(ref c) => self.eval_const(c),
            Num(ref n) => Ok(*n),
            LastResult => Ok(self.last_result),
            Name(ref name) => {
                if let Some(val) = self.vars.get(name) {
                    Ok(*val)
                } else {
                    Err(CalcrError {
                        desc: format!("Invalid function or constant: {}", name),
                        span: Some(ast.get_total_span()),
                    })
                }
            }
        }
    }

    fn eval_func(&mut self, f: &FuncKind, ast: &Ast) -> CalcrResult<f64> {
        let child = try!(ast.get_unary_branch());
        let arg = try!(self.eval_eq(child));
        match *f {
            Sin => Ok(arg.sin()),
            Cos => Ok(arg.cos()),
            Tan => Ok(arg.tan()),
            Asin => Ok(arg.asin()),
            Acos => Ok(arg.acos()),
            Atan => Ok(arg.atan()),
            Abs => Ok(arg.abs()),
            Exp => Ok(arg.exp()),
            Sqrt => {
                if arg < 0.0 {
                    Err(CalcrError {
                        desc: "Cannot take the square root of a negative number".to_string(),
                        span: Some(child.get_total_span()),
                    })
                } else {
                    Ok(arg.sqrt())
                }
            },
            Ln => {
                if arg <= 0.0 {
                    Err(CalcrError {
                        desc: "Cannot take the logarithm of a non-positive number".to_string(),
                        span: Some(child.get_total_span()),
                    })
                } else {
                    Ok(arg.ln())
                }
            },
            Log =>  {
                if arg <= 0.0 {
                    Err(CalcrError {
                        desc: "Cannot take the logarithm of a non-positive number".to_string(),
                        span: Some(child.get_total_span()),
                    })
                } else {
                    Ok(arg.log10())
                }
            },
        }
    }

    fn eval_op(&mut self, op: &OpKind, ast: &Ast) -> CalcrResult<f64> {
        match ast.branches.len() {
            2 => {
                let (lhs, rhs) = ast.get_binary_branches().unwrap();
                let (lhs, rhs) = (try!(self.eval_eq(lhs)), try!(self.eval_eq(rhs)));
                match *op {
                    Plus => Ok(lhs + rhs),
                    Minus => Ok(lhs - rhs),
                    Mult => Ok(lhs * rhs),
                    Div => Ok(lhs / rhs),
                    Pow => Ok(lhs.powf(rhs)),
                    _ => Err(CalcrError {
                        desc: "Internal error - expected AstOp to have binary branch".to_string(),
                        span: None,
                    })
                }
            },
            1 => {
                let child = ast.get_unary_branch().unwrap();
                let val = try!(self.eval_eq(child));
                match *op {
                    Neg => Ok(-val),
                    Fact => self.evalf_fact(val, child),
                    _ => Err(CalcrError {
                        desc: "Internal error - expected AstOp to have unary branch".to_string(),
                        span: None,
                    })
                }
            },
            _ => Err(CalcrError {
                desc: "Internal error - AstOp nodes must have 1 or 2 branches".to_string(),
                span: None,
            })
        }
    }

    fn eval_const(&mut self, c: &ConstKind) -> CalcrResult<f64> {
        Ok(match *c {
            Pi => f64::consts::PI,
            E => (1.0f64).exp(),
            Phi => 1.6180339887498948482,
        })
    }

    fn evalf_fact(&mut self, mut num: f64, child: &Ast) -> CalcrResult<f64> {
        if num.fract() == 0.0 && num >= 0.0 {
            let mut out = 1.0;
            while num > 0.0 {
                out *= num;
                num -= 1.0;
            }
            Ok(out)
        } else {
            Err(CalcrError {
                desc: "The factorial function only accepts positive whole numbers".to_string(),
                span: Some(child.get_total_span()),
            })
        }
    }
}