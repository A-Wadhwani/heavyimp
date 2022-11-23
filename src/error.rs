pub type EvalResult<T> = std::result::Result<T, EvalError>;

#[derive(Debug)]
pub enum EvalError {
    UnboundVariable,
    TypeMismatch,
    InvalidDereference,
}
