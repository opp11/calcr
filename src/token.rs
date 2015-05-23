use ast::OpKind;
use ast::OpKind::*;

pub struct Token {
    pub val: TokenVal,
    pub span: (usize, usize),
}

pub enum TokenVal {
    Name(String),
    Num(f64),
    Op(OpKind),
    ParenOpen,
    ParenClose,
    AbsDelim
}