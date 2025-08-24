//! Error Handling for the Gizmo Scripting Language
//!
//! This module defines the comprehensive error system used throughout the Gizmo
//! compiler and interpreter. It provides detailed error reporting with context
//! to help users debug their scripts and understand what went wrong.
//!
//! ## Error Categories
//!
//! The error system is organized into logical categories based on the phase
//! of processing where errors can occur:
//!
//! ### Lexical Analysis Errors (`LexError`)
//! - Invalid characters in source code
//! - Malformed numeric literals
//! - Position information for debugging
//!
//! ### Parse Errors (`ParseError`)
//! - Syntax errors and malformed expressions
//! - Missing tokens (e.g., expected `)` but found `;`)
//! - Invalid statement structure
//!
//! ### Runtime Errors
//! - **`RuntimeError`**: General execution problems
//! - **`TypeError`**: Type mismatches and invalid operations
//! - **`DivisionByZero`**: Mathematical errors
//! - **`UndefinedVariable`**: Variable access errors
//! - **`UndefinedFunction`**: Function call errors
//!
//! ### Specialized Errors
//! - **`InvalidFrameSize`**: Frame dimension validation
//! - **`IndexError`**: Array bounds violations
//! - **`ArgumentError`**: Function argument validation
//! - **`IOError`**: File system operations
//!
//! ## Error Flow
//!
//! Errors propagate through the system using Rust's `Result` type:
//! ```text
//! Source Code → Lexer → Parser → Interpreter → Output
//!      |           |        |           |
//!   LexError   ParseError RuntimeError  Success
//! ```
//!
//! Each phase can fail independently and provides specific error context
//! to help users understand and fix issues in their scripts.

use std::fmt;
use std::error::Error;

/// Comprehensive error type for all Gizmo language operations.
///
/// This enum covers all possible error conditions that can occur during
/// script compilation and execution, providing detailed context for debugging.
#[derive(Debug, Clone)]
pub enum GizmoError {
    /// Lexical analysis error during tokenization.
    ///
    /// Occurs when the lexer encounters invalid characters, malformed numbers,
    /// or other tokenization problems. Includes position information when available.
    ///
    /// # Examples
    /// - Invalid character: `Unexpected character '@' at line 5, column 12`
    /// - Bad number format: `Invalid number '42.3.14' at line 2, column 8`
    LexError(String),
    
    /// Syntax error during parsing.
    ///
    /// Occurs when the parser encounters invalid syntax, missing tokens,
    /// or malformed language constructs.
    ///
    /// # Examples
    /// - Missing token: `Expected ')' after expression, found ';'`
    /// - Invalid syntax: `Unexpected token 'if' in expression context`
    ParseError(String),
    
    /// General runtime execution error.
    ///
    /// Covers miscellaneous runtime problems that don't fit other categories.
    /// Less common than the more specific error types.
    RuntimeError(String),
    
    /// Type mismatch or invalid type operation.
    ///
    /// Occurs when operations are attempted on incompatible types or when
    /// expressions don't evaluate to expected types.
    ///
    /// # Examples
    /// - `Binary operations only supported for numbers`
    /// - `if condition must be a number`
    /// - `Cannot create array from mixed types`
    TypeError(String),
    
    /// Array or collection index out of bounds.
    ///
    /// Used for array access violations and similar bounds checking errors.
    IndexError(String),
    
    /// Mathematical division by zero.
    ///
    /// Special case for the common mathematical error of dividing by zero.
    /// Provides consistent handling across all division operations.
    DivisionByZero,
    
    /// Invalid frame dimensions or structure.
    ///
    /// Occurs when creating frames with invalid dimensions, mismatched row lengths,
    /// or other frame construction problems.
    ///
    /// # Examples
    /// - `Frame cannot be empty`
    /// - `Row 2 has length 5 but expected 8`
    InvalidFrameSize(String),
    
    /// Reference to undefined variable.
    ///
    /// Occurs when trying to access a variable that hasn't been declared
    /// or is out of scope.
    UndefinedVariable(String),
    
    /// Call to undefined function.
    ///
    /// Occurs when trying to call a function that doesn't exist in the
    /// built-in function registry.
    UndefinedFunction(String),
    
    /// Invalid function arguments.
    ///
    /// Occurs when calling functions with wrong number of arguments or
    /// arguments of invalid types.
    ///
    /// # Examples
    /// - `sin expects 1 argument, got 3`
    /// - `sqrt of negative number`
    ArgumentError(String),
    
    /// File system or I/O operation error.
    ///
    /// Wraps standard I/O errors that occur during file operations.
    /// Automatically converted from `std::io::Error`.
    IOError(String),
}

impl fmt::Display for GizmoError {
    /// Formats the error for user display.
    ///
    /// Provides clear, human-readable error messages that help users understand
    /// what went wrong and how to fix it. Each error type has a descriptive prefix
    /// to categorize the problem.
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

/// Automatic conversion from `std::io::Error` to `GizmoError::IOError`.
///
/// This allows using the `?` operator with I/O operations throughout the codebase,
/// automatically wrapping I/O errors in the appropriate Gizmo error type.

impl From<std::io::Error> for GizmoError {
    /// Converts a standard I/O error into a GizmoError.
    ///
    /// This conversion preserves the original error message while wrapping it
    /// in the Gizmo error system for consistent error handling.
    ///
    /// # Arguments
    /// * `err` - The I/O error to convert
    ///
    /// # Returns
    /// A `GizmoError::IOError` containing the original error message
    fn from(err: std::io::Error) -> Self {
        GizmoError::IOError(err.to_string())
    }
}

/// Convenience type alias for Results that can contain GizmoErrors.
///
/// This alias simplifies function signatures throughout the codebase by providing
/// a default error type. Most Gizmo functions return `Result<T>` instead of
/// the more verbose `std::result::Result<T, GizmoError>`.
///
/// # Usage
/// ```rust
/// fn parse_expression() -> Result<Expression> {
///     // ... parsing logic ...
///     Ok(expression)
/// }
/// ```
pub type Result<T> = std::result::Result<T, GizmoError>;