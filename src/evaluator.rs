use std::num::Float;
use std::f64;
use ast::{Ast, AstConst, AstFunc, AstOp};
use ast::AstVal::*;
use ast::AstFunc::*;
use ast::AstOp::*;
use ast::AstConst::*;
use ast::AstBranch::*;
use errors::{CalcrResult, CalcrError};

pub fn eval_eq(ast: &Ast) -> CalcrResult<f64> {
    match ast.val {
        Func(ref f) => eval_func(f, ast),
        Op(ref o) => eval_op(o, ast),
        Const(ref c) => eval_const(c),
        Num(ref n) => Ok(*n),
    }
}

fn eval_func(f: &AstFunc, ast: &Ast) -> CalcrResult<f64> {
    if let Unary(ref child) = ast.branches {
        let arg = try!(eval_eq(&*child));
        Ok(match *f {
            Sin => arg.sin(),
            Cos => arg.cos(),
            Tan => arg.tan(),
            Asin => arg.asin(),
            Acos => arg.acos(),
            Atan => arg.atan(),
            Sqrt => arg.sqrt(),
            Abs => arg.abs(),
            Exp => arg.exp(),
            Ln => arg.ln(),
            Log => arg.log10(),
        })
    } else {
        Err(CalcrError {
            desc: "Interal error - expected AstFunc to have unary branch".to_string(),
            span: ast.span,
        })
    }
}

fn eval_op(op: &AstOp, ast: &Ast) -> CalcrResult<f64> {
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
                    span: ast.span,
                })
            }
        },
        Unary(ref val) => {
            let val = try!(eval_eq(&*val));
            match *op {
                Neg => Ok(-val),
                Fact => evalf_fact(val, ast),
                _ => Err(CalcrError {
                    desc: "Internal error - expected AstOp to have unary branch".to_string(),
                    span: ast.span,
                })
            }
        },
        Leaf => Err(CalcrError {
            desc: "Internal error - AstOp nodes may not be leaf nodes".to_string(),
            span: ast.span,
        })
    }
}

fn eval_const(c: &AstConst) -> CalcrResult<f64> {
    Ok(match *c {
        Pi => f64::consts::PI,
        E => (1.0).exp(),
        Phi => 1.6180339887498948482,
    })
}

fn evalf_fact(mut num: f64, ast: &Ast) -> CalcrResult<f64> {
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
            span: ast.span,
        })
    }
}