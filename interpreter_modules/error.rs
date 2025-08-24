use std::fmt;
use std::error::Error;

#[derive(Debug, Clone)]
pub enum GizmoError {
    LexError(String),
    ParseError(String),
    RuntimeError(String),
    TypeError(String),
    IndexError(String),
    DivisionByZero,
    InvalidFrameSize(String),
    UndefinedVariable(String),
    UndefinedFunction(String),
    ArgumentError(String),
    IOError(String),
}

impl fmt::Display for GizmoError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            GizmoError::LexError(msg) => write!(f, "Lexical error: {}", msg),
            GizmoError::ParseError(msg) => write!(f, "Parse error: {}", msg),
            GizmoError::RuntimeError(msg) => write!(f, "Runtime error: {}", msg),
            GizmoError::TypeError(msg) => write!(f, "Type error: {}", msg),
            GizmoError::IndexError(msg) => write!(f, "Index error: {}", msg),
            GizmoError::DivisionByZero => write!(f, "Division by zero"),
            GizmoError::InvalidFrameSize(msg) => write!(f, "Invalid frame size: {}", msg),
            GizmoError::UndefinedVariable(name) => write!(f, "Undefined variable: {}", name),
            GizmoError::UndefinedFunction(name) => write!(f, "Undefined function: {}", name),
            GizmoError::ArgumentError(msg) => write!(f, "Argument error: {}", msg),
            GizmoError::IOError(msg) => write!(f, "IO error: {}", msg),
        }
    }
}

impl Error for GizmoError {}

impl From<std::io::Error> for GizmoError {
    fn from(err: std::io::Error) -> Self {
        GizmoError::IOError(err.to_string())
    }
}

pub type Result<T> = std::result::Result<T, GizmoError>;