use crate::typechecker::Type;

pub type EvalResult<T> = std::result::Result<T, EvalError>;

#[derive(Debug)]
pub enum EvalError {
    UnboundVariable,
    TypeMismatch { expected: Type, got: Type },
    BoundTypeMismatch,
    InvalidDereference,
}

#[derive(Debug)]
pub enum TypeError {
    Mismatch { expected: Type, got: Type },
    UnboundVariable,
    Other,
}

#[derive(Debug)]
pub enum ImpParseError {
    Other,
}
