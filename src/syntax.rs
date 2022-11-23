
type Ident = String;

enum Constant {
    Nat(i64),
    Bool(bool),
}

enum Expr {
    StoreRead(Ident),
    HeapRead(Ident),
    Constant(Constant),
    NatAdd(Box<Expr>, Box<Expr>),
    BoolAnd(Box<Expr>, Box<Expr>),
    BoolNot(Box<Expr>),
    BoolLeq(Box<Expr>, Box<Expr>),
}

enum Statement {
    StoreAssign(Ident, Expr),
    HeapNew(Ident, Expr),
    HeapUpdate(Ident, Expr),
    HeapAlias(Ident, Ident),
    Sequence(Box<Statement>, Box<Statement>),
    Conditional(Expr, Box<Statement>, Box<Statement>),
    While(Expr, Box<Statement>),
    Skip,
}