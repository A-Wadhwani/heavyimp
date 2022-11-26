pub type Ident = String;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Constant {
    Nat(i64),
    Bool(bool),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Expr {
    StoreRead(Ident),
    HeapRead(Ident),
    Constant(Constant),
    NatAdd(Box<Expr>, Box<Expr>),
    NatLeq(Box<Expr>, Box<Expr>),
    BoolAnd(Box<Expr>, Box<Expr>),
    BoolNot(Box<Expr>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Statement {
    StoreAssign(Ident, Expr),
    HeapNew(Ident, Expr),
    HeapUpdate(Ident, Expr),
    HeapAlias(Ident, Ident),
    Sequence(Box<Statement>, Box<Statement>),
    Conditional(Expr, Box<Statement>, Box<Statement>),
    While(Expr, Box<Statement>),
    Skip,
}
