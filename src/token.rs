use ast;

#[derive(Debug, PartialEq)]
pub struct Token {
    pub val: TokVal,
    pub span: (usize, usize),
}

#[derive(Debug, PartialEq, Clone)]
pub enum TokVal {
    Name(String),
    Num(f64),
    Op(OpKind),
    OpenDelim(DelimKind),
    CloseDelim(DelimKind),
    AbsDelim
}

#[derive(Debug, PartialEq, Clone)]
pub enum OpKind {
    Plus,
    Minus,
    Mult,
    Div,
    Pow,
    Fact,
    Assign,
}

impl Into<ast::OpKind> for OpKind {
    fn into(self) -> ast::OpKind {
        match self {
            OpKind::Plus => ast::OpKind::Plus,
            OpKind::Minus => ast::OpKind::Minus,
            OpKind::Mult => ast::OpKind::Mult,
            OpKind::Div => ast::OpKind::Div,
            OpKind::Pow => ast::OpKind::Pow,
            OpKind::Fact => ast::OpKind::Fact,
            OpKind::Assign => ast::OpKind::Assign,
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum DelimKind {
    Paren,
    Bracket,
    Brace,
}

impl TokVal {
    pub fn op(self) -> Option<OpKind> {
        if let TokVal::Op(op) = self {
            Some(op)
        } else {
            None
        }
    }

    pub fn is_open_delim(&self) -> bool {
        if let TokVal::OpenDelim(_) = *self {
            true
        } else {
            false
        }
    }

    pub fn is_close_delim(&self) -> bool {
        if let TokVal::CloseDelim(_) = *self {
            true
        } else {
            false
        }
    }
}