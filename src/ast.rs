#[derive(Debug)]
pub struct Ast {
    pub val: AstVal,
    pub span: (usize, usize),
    pub branches: AstBranch,
}

#[derive(Debug)]
pub enum AstBranch {
    Binary(Box<Ast>, Box<Ast>),
    Unary(Box<Ast>),
    Leaf,
}

#[derive(Debug)]
pub enum AstVal {
    Func(AstFunc),
    Op(AstOp),
    Const(AstConst),
    Num(f64),
}

#[derive(Debug)]
pub enum AstFunc {
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

#[derive(Debug)]
pub enum AstOp {
    Plus,
    Minus,
    Mult,
    Div,
    Pow,
    Fact,
    Neg,
}

#[derive(Debug)]
pub enum AstConst {
    Pi,
    E,
    Phi,
}