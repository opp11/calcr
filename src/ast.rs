use std::cmp::{min, max};

#[derive(Debug, PartialEq)]
pub struct Ast {
    pub val: AstVal,
    pub span: (usize, usize),
    pub branches: AstBranch,
}

impl Ast {
    pub fn get_total_span(&self) -> (usize, usize) {
        if self.val == AstVal::Paren {
            // since parens always encapsulates their child, we can just stop here
            self.span
        } else {
            match self.branches {
                AstBranch::Binary(ref lhs, ref rhs) => {
                    let lhs_span = lhs.get_total_span();
                    let rhs_span = rhs.get_total_span();
                    let begin = min(self.span.0, lhs_span.0);
                    let end = max(self.span.1, rhs_span.1);
                    (begin, end)
                },
                AstBranch::Unary(ref child) => {
                    let child_span = child.get_total_span();
                    let begin = min(self.span.0, child_span.0);
                    let end = max(self.span.1, child_span.1);
                    (begin, end)
                },
                AstBranch::Leaf => self.span,
            }
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum AstBranch {
    Binary(Box<Ast>, Box<Ast>),
    Unary(Box<Ast>),
    Leaf,
}

#[derive(Debug, PartialEq)]
pub enum AstVal {
    Func(FuncKind),
    Op(OpKind),
    Const(ConstKind),
    Num(f64),
    LastResult,
    Paren,
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
}

#[derive(Debug, PartialEq)]
pub enum ConstKind {
    Pi,
    E,
    Phi,
}