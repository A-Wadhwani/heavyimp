pub type EvalResult<T> = std::result::Result<T, EvalError>;

pub enum EvalError {
    UnboundVariable,
    TypeMismatch,
    InvalidDereference,
}
