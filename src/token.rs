use ast::OpKind;

#[derive(Debug, PartialEq)]
pub struct Token {
    pub val: TokVal,
    pub span: (usize, usize),
}

#[derive(Debug, PartialEq, Clone)]
pub enum TokVal {
    Name(String),
    Keyword(KeywordKind),
    Num(f64),
    Op(OpKind),
    ParenOpen,
    ParenClose,
    AbsDelim
}

#[derive(Clone, PartialEq, Debug)]
pub enum KeywordKind {
    Let,
}

impl TokVal {
    pub fn op(self) -> Option<OpKind> {
        if let TokVal::Op(op) = self {
            Some(op)
        } else {
            None
        }
    }
}