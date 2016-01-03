use std::cmp::{min, max};
use errors::{CalcrResult, CalcrError};

#[derive(Debug, PartialEq)]
pub struct Ast {
    pub val: AstVal,
    pub span: (usize, usize),
    pub branches: Vec<Ast>,
}

impl Ast {
    pub fn is_leaf(&self) -> bool {
        self.branches.is_empty()
    }

    pub fn get_unary_branch(&self) -> CalcrResult<&Ast> {
        if self.branches.len() == 1 {
            Ok(&self.branches[0])
        } else {
            Err(CalcrError {
                desc: "Internal error - expected AST to have 1 branch".to_string(),
                span: Some(self.span),
            })
        }
    }

    pub fn get_binary_branches(&self) -> CalcrResult<(&Ast, &Ast)> {
        if self.branches.len() == 2 {
            Ok((&self.branches[0], &self.branches[1]))
        } else {
            Err(CalcrError {
                desc: "Internal error - expected AST to have 2 branches".to_string(),
                span: Some(self.span),
            })
        }
    }

    pub fn get_total_span(&self) -> (usize, usize) {
        if self.is_leaf() {
            self.span
        } else {
            self.branches.iter()
                         .map(|br| br.get_total_span())
                         .fold(self.span, |out, span| (min(out.0, span.0), max(out.1, span.1)))
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum AstVal {
    Func(FuncKind),
    Op(OpKind),
    Const(ConstKind),
    Num(f64),
    LastResult,
    Name(String),
}

#[derive(Debug, PartialEq)]
pub enum FuncKind {
    Sin,
    Cos,
    Tan,
    Asin,
    Acos,
    Atan,
    Sqrt,
    Abs,
    Exp,
    Ln,
    Log,
}

#[derive(Debug, PartialEq, Clone)]
pub enum OpKind {
    Plus,
    Minus,
    Mult,
    Div,
    Pow,
    Fact,
    Neg,
    Assign,
}

#[derive(Debug, PartialEq)]
pub enum ConstKind {
    Pi,
    E,
    Phi,
}