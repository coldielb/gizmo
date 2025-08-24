//! Lexical Analyzer for the Gizmo Scripting Language
//!
//! This module implements the tokenization phase of the Gizmo compiler pipeline.
//! It converts raw source code text into a stream of tokens that can be parsed
//! into an Abstract Syntax Tree.
//!
//! ## Tokenization Process
//!
//! The lexer performs several key functions:
//! 1. **Character Processing**: Iterates through source code character by character
//! 2. **Token Recognition**: Identifies keywords, operators, literals, and identifiers
//! 3. **Error Handling**: Reports malformed tokens with line/column information
//! 4. **Comment Filtering**: Strips single-line comments (`//`) from the token stream
//! 5. **Position Tracking**: Maintains accurate line and column numbers for debugging
//!
//! ## Supported Tokens
//!
//! - **Literals**: Numbers (`42`, `3.14`), Strings (`"hello"`)
//! - **Identifiers**: Variable names (`my_var`, `frame_data`)
//! - **Keywords**: Language constructs (`frame`, `if`, `pattern`, etc.)
//! - **Operators**: Arithmetic (`+`, `-`, `*`), Comparison (`>`, `==`), Logical (`and`, `or`)
//! - **Delimiters**: Parentheses, brackets, braces, punctuation
//! - **Special**: Newlines (significant for statement separation), EOF marker
//!
//! ## Design Notes
//!
//! The lexer uses a simple character-by-character scanning approach with lookahead
//! for multi-character tokens like `==`, `>=`, and `//`. This provides good error
//! reporting and is easy to understand and maintain.

use crate::error::GizmoError;
use std::fmt;

/// Represents all possible tokens in the Gizmo scripting language.
///
/// Each token represents a meaningful unit of source code that can be
/// recognized and processed by the parser. Tokens carry both type information
/// and any associated data (e.g., numeric values, identifier names).
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // === LITERAL TOKENS ===
    // These tokens carry actual data values from the source code
    
    /// Numeric literal: `42`, `3.14`, `0.5`
    ///
    /// All numbers are parsed as 64-bit floating point values for simplicity.
    /// Supports both integer and decimal notation.
    Number(f64),
    
    /// String literal: `"hello world"` (currently unused but reserved)
    ///
    /// Supports basic string literals for future language extensions.
    String(String),
    
    /// Identifier: `my_var`, `frame_data`, `calculate_distance`
    ///
    /// Variable names and user-defined identifiers. Must start with a letter
    /// or underscore, followed by letters, digits, or underscores.
    Identifier(String),
    
    // === KEYWORD TOKENS ===
    // Reserved words that have special meaning in the language
    
    /// Variable type keyword: `frame`
    Frame,
    /// Array type keyword: `frames`
    Frames,
    /// Function definition keyword: `function` (reserved)
    Function,
    /// Return statement keyword: `return`
    Return,
    /// Conditional keyword: `if`
    If,
    /// Conditional clause keyword: `then`
    Then,
    /// Conditional clause keyword: `else`
    Else,
    /// Loop keyword: `for` (reserved)
    For,
    /// Range keyword: `in` (reserved)
    In,
    /// Range constructor: `range` (reserved)
    Range,
    /// Pattern generator keyword: `pattern`
    Pattern,
    /// Loop keyword: `repeat`
    Repeat,
    /// Loop count keyword: `times`
    Times,
    /// Block start keyword: `do`
    Do,
    /// Block end keyword: `end`
    End,
    /// Logical operator: `and`
    And,
    /// Logical operator: `or`
    Or,
    
    // === OPERATOR TOKENS ===
    // Mathematical, comparison, and logical operators
    
    /// Addition operator: `+`
    Plus,
    /// Subtraction operator: `-`
    Minus,
    /// Multiplication operator: `*`
    Star,
    /// Division operator: `/`
    Slash,
    /// Modulo operator: `%`
    Percent,
    /// Assignment operator: `=`
    Equal,
    /// Equality operator: `==`
    EqualEqual,
    /// Greater than operator: `>`
    Greater,
    /// Less than operator: `<`
    Less,
    /// Greater than or equal operator: `>=`
    GreaterEqual,
    /// Less than or equal operator: `<=`
    LessEqual,
    
    // === DELIMITER TOKENS ===
    // Punctuation that structures the language syntax
    
    /// Left parenthesis: `(`
    LeftParen,
    /// Right parenthesis: `)`
    RightParen,
    /// Left square bracket: `[`
    LeftBracket,
    /// Right square bracket: `]`
    RightBracket,
    /// Left curly brace: `{`
    LeftBrace,
    /// Right curly brace: `}`
    RightBrace,
    /// Comma separator: `,`
    Comma,
    /// Semicolon terminator: `;`
    Semicolon,
    /// Ternary question mark: `?`
    Question,
    /// Ternary/label colon: `:`
    Colon,
    
    // === SPECIAL TOKENS ===
    // Structural tokens for parsing control
    
    /// Newline character: `\n`
    ///
    /// Significant in Gizmo for statement separation and formatting.
    /// The parser uses newlines to determine statement boundaries.
    Newline,
    
    /// End of file marker
    ///
    /// Indicates the end of the token stream. Always the last token
    /// produced by the lexer.
    Eof,
}

/// Lexical analyzer that converts source code into tokens.
///
/// The lexer maintains state about the current position in the source code
/// and provides methods to scan through characters and produce tokens.
/// It handles position tracking for error reporting and supports lookahead
/// for multi-character tokens.
pub struct Lexer {
    /// Source code as a vector of characters for efficient indexing
    input: Vec<char>,
    /// Current position in the character stream
    position: usize,
    /// Current line number (1-based for human-readable error messages)
    line: usize,
    /// Current column number (1-based for human-readable error messages)
    column: usize,
}

impl Lexer {
    /// Creates a new lexer for the given source code.
    ///
    /// Initializes the lexer state with the source code converted to a character
    /// vector for efficient random access during tokenization.
    ///
    /// # Arguments
    /// * `input` - Source code string to tokenize
    ///
    /// # Returns
    /// A new Lexer ready to tokenize the input
    pub fn new(input: &str) -> Self {
        Self {
            input: input.chars().collect(),
            position: 0,
            line: 1,
            column: 1,
        }
    }
    
    /// Tokenizes the entire input source code into a vector of tokens.
    ///
    /// This is the main entry point for lexical analysis. It repeatedly
    /// calls `next_token()` until the end of file is reached, collecting
    /// all tokens into a vector.
    ///
    /// # Returns
    /// * `Ok(Vec<Token>)` - Complete token stream ending with `Token::Eof`
    /// * `Err(GizmoError)` - Lexical error with position information
    ///
    /// # Error Handling
    /// If tokenization fails at any point, the entire process stops and
    /// returns the error with precise location information.
    pub fn tokenize(&mut self) -> Result<Vec<Token>, GizmoError> {
        let mut tokens = Vec::new();
        
        loop {
            let token = self.next_token()?;
            if token == Token::Eof {
                tokens.push(token);
                break;
            }
            tokens.push(token);
        }
        
        Ok(tokens)
    }
    
    /// Scans and returns the next token from the input stream.
    ///
    /// This is the core tokenization method that:
    /// 1. Skips whitespace (except newlines)
    /// 2. Identifies the token type based on the current character
    /// 3. Consumes the appropriate number of characters
    /// 4. Returns the resulting token
    ///
    /// # Returns
    /// * `Ok(Token)` - The next token in the stream
    /// * `Err(GizmoError)` - Lexical error for invalid characters or malformed tokens
    ///
    /// # Character Processing
    /// Uses a character-by-character state machine with lookahead for:
    /// - Multi-character operators (`==`, `>=`, `<=`)
    /// - Comments (`//` to end of line)
    /// - Numeric literals with decimal points
    /// - Identifiers vs keywords
    fn next_token(&mut self) -> Result<Token, GizmoError> {
        self.skip_whitespace();
        
        if self.is_at_end() {
            return Ok(Token::Eof);
        }
        
        let c = self.advance();
        
        match c {
            '\n' => {
                self.line += 1;
                self.column = 1;
                Ok(Token::Newline)
            }
            '(' => Ok(Token::LeftParen),
            ')' => Ok(Token::RightParen),
            '[' => Ok(Token::LeftBracket),
            ']' => Ok(Token::RightBracket),
            '{' => Ok(Token::LeftBrace),
            '}' => Ok(Token::RightBrace),
            ',' => Ok(Token::Comma),
            ';' => Ok(Token::Semicolon),
            '?' => Ok(Token::Question),
            ':' => Ok(Token::Colon),
            '+' => Ok(Token::Plus),
            '-' => Ok(Token::Minus),
            '*' => Ok(Token::Star),
            '/' => {
                if self.peek() == '/' {
                    // Single-line comment: consume until end of line
                    // Comments are stripped from the token stream entirely
                    while self.peek() != '\n' && !self.is_at_end() {
                        self.advance();
                    }
                    // Recursively get the next token after the comment
                    self.next_token()
                } else {
                    // Division operator
                    Ok(Token::Slash)
                }
            }
            '%' => Ok(Token::Percent),
            '=' => {
                if self.peek() == '=' {
                    self.advance();
                    Ok(Token::EqualEqual)
                } else {
                    Ok(Token::Equal)
                }
            }
            '>' => {
                if self.peek() == '=' {
                    self.advance();
                    Ok(Token::GreaterEqual)
                } else {
                    Ok(Token::Greater)
                }
            }
            '<' => {
                if self.peek() == '=' {
                    self.advance();
                    Ok(Token::LessEqual)
                } else {
                    Ok(Token::Less)
                }
            }
            c if c.is_ascii_digit() => self.number_literal(c),
            c if c.is_ascii_alphabetic() || c == '_' => self.identifier_or_keyword(c),
            _ => Err(GizmoError::LexError(format!(
                "Unexpected character '{}' at line {}, column {}",
                c, self.line, self.column
            ))),
        }
    }
    
    /// Scans a numeric literal token starting with the given digit.
    ///
    /// Supports both integer and floating-point numbers:
    /// - Integers: `42`, `0`, `123`
    /// - Decimals: `3.14`, `0.5`, `42.0`
    ///
    /// Uses lookahead to distinguish decimal points from other uses of `.`
    /// (e.g., method calls in future language versions).
    ///
    /// # Arguments
    /// * `first_digit` - The first digit character already consumed
    ///
    /// # Returns
    /// * `Ok(Token::Number)` - Valid numeric literal
    /// * `Err(GizmoError)` - Invalid number format
    ///
    /// # Error Cases
    /// - Malformed decimal numbers
    /// - Numbers too large for f64 representation
    /// - Invalid numeric syntax
    fn number_literal(&mut self, first_digit: char) -> Result<Token, GizmoError> {
        let mut value = String::from(first_digit);
        
        // Consume integer part
        while self.peek().is_ascii_digit() {
            value.push(self.advance());
        }
        
        // Check for decimal point (with lookahead to ensure digit follows)
        if self.peek() == '.' && self.peek_next().is_ascii_digit() {
            value.push(self.advance()); // consume '.'
            // Consume fractional part
            while self.peek().is_ascii_digit() {
                value.push(self.advance());
            }
        }
        
        // Parse the collected string as a floating-point number
        match value.parse::<f64>() {
            Ok(num) => Ok(Token::Number(num)),
            Err(_) => Err(GizmoError::LexError(format!(
                "Invalid number '{}' at line {}, column {}",
                value, self.line, self.column
            ))),
        }
    }
    
    /// Scans an identifier or keyword starting with the given character.
    ///
    /// Identifiers follow standard rules:
    /// - Start with letter (a-z, A-Z) or underscore (_)
    /// - Contain letters, digits (0-9), or underscores
    /// - Case-sensitive
    ///
    /// After scanning the complete identifier, checks against the keyword
    /// table to determine if it's a reserved word or user identifier.
    ///
    /// # Arguments
    /// * `first_char` - The first character already consumed
    ///
    /// # Returns
    /// * `Ok(Token::Keyword)` - If identifier matches a reserved word
    /// * `Ok(Token::Identifier)` - If identifier is user-defined
    ///
    /// # Keyword Recognition
    /// The lexer recognizes these reserved words:
    /// - Types: `frame`, `frames`
    /// - Control: `if`, `then`, `else`, `repeat`, `times`, `do`, `end`
    /// - Functions: `function`, `return`, `pattern`
    /// - Logic: `and`, `or`
    /// - Reserved: `for`, `in`, `range` (for future use)
    fn identifier_or_keyword(&mut self, first_char: char) -> Result<Token, GizmoError> {
        let mut value = String::from(first_char);
        
        // Collect all alphanumeric characters and underscores
        while self.peek().is_ascii_alphanumeric() || self.peek() == '_' {
            value.push(self.advance());
        }
        
        // Check against keyword table
        let token = match value.as_str() {
            // Type keywords
            "frame" => Token::Frame,
            "frames" => Token::Frames,
            
            // Function keywords
            "function" => Token::Function,
            "return" => Token::Return,
            "pattern" => Token::Pattern,
            
            // Control flow keywords
            "if" => Token::If,
            "then" => Token::Then,
            "else" => Token::Else,
            "repeat" => Token::Repeat,
            "times" => Token::Times,
            "do" => Token::Do,
            "end" => Token::End,
            
            // Logical operators
            "and" => Token::And,
            "or" => Token::Or,
            
            // Reserved for future use
            "for" => Token::For,
            "in" => Token::In,
            "range" => Token::Range,
            
            // Default: user identifier
            _ => Token::Identifier(value),
        };
        
        Ok(token)
    }
    
    /// Skips whitespace characters but preserves newlines.
    ///
    /// Whitespace characters (space, carriage return, tab) are ignored
    /// for tokenization purposes, but newlines are significant in Gizmo
    /// and are preserved as tokens for statement separation.
    ///
    /// # Whitespace Handling
    /// - Spaces, tabs, and carriage returns are skipped
    /// - Newlines (\n) are preserved as `Token::Newline`
    /// - This allows flexible formatting while maintaining statement structure
    fn skip_whitespace(&mut self) {
        while !self.is_at_end() {
            match self.peek() {
                ' ' | '\r' | '\t' => {
                    self.advance();
                }
                _ => break,
            }
        }
    }
    
    /// Checks if the lexer has reached the end of the input.
    ///
    /// Used throughout the tokenization process to prevent reading
    /// beyond the input bounds.
    ///
    /// # Returns
    /// `true` if at or past the end of input, `false` otherwise
    fn is_at_end(&self) -> bool {
        self.position >= self.input.len()
    }
    
    /// Consumes and returns the current character, advancing the position.
    ///
    /// Updates the position tracking (line and column numbers) for accurate
    /// error reporting. Line numbers are updated when newlines are processed
    /// in the main tokenization loop.
    ///
    /// # Returns
    /// The character at the current position, or '\0' if at end of input
    fn advance(&mut self) -> char {
        if !self.is_at_end() {
            self.column += 1;
            let c = self.input[self.position];
            self.position += 1;
            c
        } else {
            '\0'
        }
    }
    
    /// Returns the current character without consuming it.
    ///
    /// Provides one-character lookahead for tokenization decisions
    /// (e.g., distinguishing `=` from `==`, `/` from `//`).
    ///
    /// # Returns
    /// The character at the current position, or '\0' if at end of input
    fn peek(&self) -> char {
        if self.is_at_end() {
            '\0'
        } else {
            self.input[self.position]
        }
    }
    
    /// Returns the character after the current position without consuming it.
    ///
    /// Provides two-character lookahead, primarily used for decimal number
    /// recognition (ensuring a digit follows a decimal point).
    ///
    /// # Returns
    /// The character at position + 1, or '\0' if past end of input
    fn peek_next(&self) -> char {
        if self.position + 1 >= self.input.len() {
            '\0'
        } else {
            self.input[self.position + 1]
        }
    }
}