use std::num::Float;
use std::f64;
use ast::{Ast, ConstKind, FuncKind, OpKind};
use ast::AstVal::*;
use ast::FuncKind::*;
use ast::OpKind::*;
use ast::ConstKind::*;
use ast::AstBranch::*;
use errors::{CalcrResult, CalcrError};

pub fn eval_eq(ast: &Ast) -> CalcrResult<f64> {
    match ast.val {
        Func(ref f) => eval_func(f, ast),
        Op(ref o) => eval_op(o, ast),
        Const(ref c) => eval_const(c),
        Num(ref n) => Ok(*n),
        Paren => {
            if let Unary(ref child) = ast.branches {
                eval_eq(child)
            } else {
                Err(CalcrError {
                    desc: "Internal error - expected Paren to have unary branch".to_string(),
                    span: None,
                })
            }
        },
    }
}

fn eval_func(f: &FuncKind, ast: &Ast) -> CalcrResult<f64> {
    if let Unary(ref child) = ast.branches {
        let arg = try!(eval_eq(&*child));
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
    } else {
        Err(CalcrError {
            desc: "Interal error - expected AstFunc to have unary branch".to_string(),
            span: None,
        })
    }
}

fn eval_op(op: &OpKind, ast: &Ast) -> CalcrResult<f64> {
    match ast.branches {
        Binary(ref lhs, ref rhs) => {
            let (lhs, rhs) = (try!(eval_eq(&*lhs)), try!(eval_eq(&*rhs)));
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
        Unary(ref child) => {
            let val = try!(eval_eq(&*child));
            match *op {
                Neg => Ok(-val),
                Fact => evalf_fact(val, child),
                _ => Err(CalcrError {
                    desc: "Internal error - expected AstOp to have unary branch".to_string(),
                    span: None,
                })
            }
        },
        Leaf => Err(CalcrError {
            desc: "Internal error - AstOp nodes may not be leaf nodes".to_string(),
            span: None,
        })
    }
}

fn eval_const(c: &ConstKind) -> CalcrResult<f64> {
    Ok(match *c {
        Pi => f64::consts::PI,
        E => (1.0).exp(),
        Phi => 1.6180339887498948482,
    })
}

fn evalf_fact(mut num: f64, child: &Ast) -> CalcrResult<f64> {
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