//! Parser for the Gizmo Scripting Language
//!
//! This module implements a recursive descent parser with operator precedence climbing
//! to convert a stream of tokens into an Abstract Syntax Tree (AST). The parser handles
//! the complete Gizmo language grammar including expressions, statements, and control flow.
//!
//! ## Parser Architecture
//!
//! The parser uses a **recursive descent** approach with separate methods for different
//! grammar productions. The key design elements are:
//!
//! ### Statement Parsing
//! - **Variable Declarations**: `frame var = expr`, `frames arr = expr`
//! - **Assignments**: `var = expr`
//! - **Control Flow**: `if/then/else/end`, `repeat/times/do/end`
//! - **Expression Statements**: Function calls and standalone expressions
//!
//! ### Expression Parsing with Operator Precedence
//! The parser implements precedence climbing for mathematical expressions:
//!
//! ```text
//! Precedence Levels (lowest to highest):
//! 1. Ternary (?:)           - condition ? true_expr : false_expr
//! 2. Logical OR (or)        - left-associative
//! 3. Logical AND (and)      - left-associative
//! 4. Equality (==, !=)      - left-associative
//! 5. Comparison (<, >, <=, >=) - left-associative
//! 6. Addition (+, -)        - left-associative
//! 7. Multiplication (*, /, %) - left-associative
//! 8. Primary (literals, identifiers, function calls, parentheses)
//! ```
//!
//! ### Pattern Generation
//! Special handling for `pattern(width, height) { statements... return expr; }` blocks
//! that generate pixel art through per-pixel expression evaluation.
//!
//! ## Error Recovery
//! The parser provides detailed error messages with context about what was expected
//! and what was found. It fails fast on syntax errors to give clear feedback.
//!
//! ## Newline Handling
//! Newlines are significant in Gizmo for statement separation but are flexibly
//! handled - they can appear almost anywhere for formatting without affecting semantics.

use crate::lexer::Token;
use crate::ast::*;
use crate::error::{GizmoError, Result};

/// Recursive descent parser for the Gizmo scripting language.
///
/// The parser maintains state about the current position in the token stream
/// and provides methods to parse different grammar productions into AST nodes.
pub struct Parser {
    /// Vector of tokens to parse (produced by the lexer)
    tokens: Vec<Token>,
    /// Current position in the token stream
    current: usize,
}

impl Parser {
    /// Creates a new parser for the given token stream.
    ///
    /// # Arguments
    /// * `tokens` - Vector of tokens produced by the lexer
    ///
    /// # Returns
    /// A new Parser ready to parse the token stream into an AST
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, current: 0 }
    }
    
    /// Parses the complete token stream into a Program AST.
    ///
    /// This is the main entry point for parsing. It processes all tokens
    /// from the stream, parsing them into statements that make up the program.
    ///
    /// # Returns
    /// * `Ok(Program)` - Successfully parsed AST
    /// * `Err(GizmoError)` - Syntax error with details about what went wrong
    ///
    /// # Grammar
    /// ```text
    /// program → statement* EOF
    /// ```
    ///
    /// Newlines are skipped at the top level for flexible formatting.
    pub fn parse(&mut self) -> Result<Program> {
        let mut statements = Vec::new();
        
        while !self.is_at_end() {
            // Skip newlines at the top level for flexible formatting
            if self.peek() == &Token::Newline {
                self.advance();
                continue;
            }
            
            statements.push(self.statement()?);
        }
        
        Ok(Program { statements })
    }
    
    /// Parses a statement from the current token position.
    ///
    /// Statements are the top-level constructs in Gizmo programs. The parser
    /// uses lookahead to determine which type of statement to parse based on
    /// the current token.
    ///
    /// # Returns
    /// * `Ok(Statement)` - Successfully parsed statement
    /// * `Err(GizmoError)` - Syntax error in statement
    ///
    /// # Grammar
    /// ```text
    /// statement → variable_declaration
    ///           | assignment
    ///           | repeat_statement
    ///           | if_statement  
    ///           | expression_statement
    /// ```
    ///
    /// Uses intelligent lookahead to distinguish assignments from expression statements
    /// when encountering identifiers.
    fn statement(&mut self) -> Result<Statement> {
        match self.peek() {
            Token::Frame | Token::Frames => {
                self.variable_declaration()
            }
            Token::Repeat => {
                self.repeat_statement()
            }
            Token::If => {
                self.if_statement()
            }
            Token::Identifier(_) => {
                // Lookahead to distinguish assignment from expression statement
                if self.peek_ahead_is_assignment() {
                    self.assignment_statement()
                } else {
                    self.expression_statement()
                }
            }
            _ => self.expression_statement(),
        }
    }
    
    /// Parses a variable declaration statement.
    ///
    /// Variable declarations create new variables in the current scope with
    /// explicit type annotations to help the interpreter understand the data.
    ///
    /// # Grammar
    /// ```text
    /// variable_declaration → ("frame" | "frames") IDENTIFIER "=" expression (";")?  
    /// ```
    ///
    /// # Examples
    /// - `frame my_frame = pattern(8, 8) { return col > row; }`
    /// - `frames animation = [frame1, frame2, frame3];`
    ///
    /// # Error Handling
    /// Provides specific error messages for missing identifiers, assignment operators,
    /// and malformed expressions.
    fn variable_declaration(&mut self) -> Result<Statement> {
        let var_type = match self.advance() {
            Token::Frame => VariableType::Frame,
            Token::Frames => VariableType::Frames,
            token => {
                return Err(GizmoError::ParseError(format!(
                    "Expected variable type, found '{:?}'", token
                )));
            }
        };
        
        let name = match self.advance() {
            Token::Identifier(name) => name.clone(),
            token => {
                return Err(GizmoError::ParseError(format!(
                    "Expected identifier, found '{:?}'", token
                )));
            }
        };
        
        if self.peek() != &Token::Equal {
            return Err(GizmoError::ParseError(format!(
                "Expected '=', found '{:?}'", self.peek()
            )));
        }
        self.advance(); // consume '='
        
        let value = self.expression()?;
        
        // Optional semicolon terminator
        if self.peek() == &Token::Semicolon {
            self.advance();
        }
        
        self.skip_newlines();
        
        Ok(Statement::VariableDeclaration {
            var_type,
            name,
            value,
        })
    }
    
    fn assignment_statement(&mut self) -> Result<Statement> {
        let name = match self.advance() {
            Token::Identifier(name) => name.clone(),
            token => {
                return Err(GizmoError::ParseError(format!(
                    "Expected identifier, found '{:?}'", token
                )));
            }
        };
        
        if self.peek() != &Token::Equal {
            return Err(GizmoError::ParseError(format!(
                "Expected '=', found '{:?}'", self.peek()
            )));
        }
        self.advance(); // consume '='
        
        let value = self.expression()?;
        
        if self.peek() == &Token::Semicolon {
            self.advance();
        }
        self.skip_newlines();
        
        Ok(Statement::Assignment { name, value })
    }
    
    fn expression_statement(&mut self) -> Result<Statement> {
        let expr = self.expression()?;
        
        if self.peek() == &Token::Semicolon {
            self.advance();
        }
        self.skip_newlines();
        
        Ok(Statement::ExpressionStatement(expr))
    }
    
    /// Parses an if statement with optional else clause.
    ///
    /// If statements provide conditional execution based on boolean expressions.
    /// The condition is evaluated, and if true (non-zero), the then_body is executed.
    /// Otherwise, the optional else_body is executed.
    ///
    /// # Grammar
    /// ```text
    /// if_statement → "if" expression "then" statement* ("else" statement*)? "end"
    /// ```
    ///
    /// # Examples
    /// ```gzmo
    /// if x > 5 then
    ///     result = 1
    /// else
    ///     result = 0
    /// end
    /// ```
    ///
    /// # Newline Handling
    /// Newlines are flexibly handled within if blocks - they can appear after
    /// keywords and between statements without affecting semantics.
    fn if_statement(&mut self) -> Result<Statement> {
        self.advance(); // consume 'if'
        
        let condition = self.expression()?;
        
        // Expect 'then' keyword
        if self.peek() != &Token::Then {
            return Err(GizmoError::ParseError(format!(
                "Expected 'then', found '{:?}'", self.peek()
            )));
        }
        self.advance(); // consume 'then'
        
        self.skip_newlines();
        
        let mut then_body = Vec::new();
        
        // Parse statements until we hit 'else' or 'end'
        while self.peek() != &Token::Else && self.peek() != &Token::End && !self.is_at_end() {
            if self.peek() == &Token::Newline {
                self.advance();
                continue;
            }
            then_body.push(self.statement()?);
        }
        
        let mut else_body = None;
        
        // Check for optional else clause
        if self.peek() == &Token::Else {
            self.advance(); // consume 'else'
            self.skip_newlines();
            
            let mut else_statements = Vec::new();
            while self.peek() != &Token::End && !self.is_at_end() {
                if self.peek() == &Token::Newline {
                    self.advance();
                    continue;
                }
                else_statements.push(self.statement()?);
            }
            else_body = Some(else_statements);
        }
        
        // Expect 'end'
        if self.peek() != &Token::End {
            return Err(GizmoError::ParseError(format!(
                "Expected 'end', found '{:?}'", self.peek()
            )));
        }
        self.advance(); // consume 'end'
        
        if self.peek() == &Token::Semicolon {
            self.advance();
        }
        self.skip_newlines();
        
        Ok(Statement::IfStatement {
            condition,
            then_body,
            else_body,
        })
    }
    
    /// Parses a repeat loop statement.
    ///
    /// Repeat loops execute a block of statements a specified number of times.
    /// The count expression is evaluated once at the start of the loop, and
    /// the loop body is executed that many times.
    ///
    /// # Grammar
    /// ```text
    /// repeat_statement → "repeat" expression "times" "do" statement* "end"
    /// ```
    ///
    /// # Examples
    /// ```gzmo
    /// repeat 10 times do
    ///     add_frame(frames, current_frame)
    ///     current_frame = transform(current_frame)
    /// end
    /// ```
    ///
    /// # Loop Variables
    /// The interpreter automatically provides a `time` variable inside the loop
    /// containing the current iteration index (0-based).
    fn repeat_statement(&mut self) -> Result<Statement> {
        self.advance(); // consume 'repeat'
        
        let count = self.expression()?;
        
        // Expect 'times' keyword
        if self.peek() != &Token::Times {
            return Err(GizmoError::ParseError(format!(
                "Expected 'times', found '{:?}'", self.peek()
            )));
        }
        self.advance(); // consume 'times'
        
        // Expect 'do' keyword
        if self.peek() != &Token::Do {
            return Err(GizmoError::ParseError(format!(
                "Expected 'do', found '{:?}'", self.peek()
            )));
        }
        self.advance(); // consume 'do'
        
        self.skip_newlines();
        
        let mut body = Vec::new();
        
        // Parse statements until we hit 'end'
        while self.peek() != &Token::End && !self.is_at_end() {
            if self.peek() == &Token::Newline {
                self.advance();
                continue;
            }
            body.push(self.statement()?);
        }
        
        // Expect 'end'
        if self.peek() != &Token::End {
            return Err(GizmoError::ParseError(format!(
                "Expected 'end', found '{:?}'", self.peek()
            )));
        }
        self.advance(); // consume 'end'
        
        if self.peek() == &Token::Semicolon {
            self.advance();
        }
        self.skip_newlines();
        
        Ok(Statement::RepeatLoop {
            count: Box::new(count),
            body,
        })
    }
    
    /// Parses an expression using operator precedence climbing.
    ///
    /// This is the entry point for expression parsing. It delegates to the
    /// lowest precedence level (ternary operations) and lets precedence
    /// climbing handle the operator hierarchy.
    ///
    /// # Returns
    /// * `Ok(Expression)` - Successfully parsed expression
    /// * `Err(GizmoError)` - Syntax error in expression
    fn expression(&mut self) -> Result<Expression> {
        self.ternary()
    }
    
    /// Parses ternary conditional expressions (lowest precedence).
    ///
    /// Ternary operations provide conditional expressions that select between
    /// two values based on a boolean condition.
    ///
    /// # Precedence Level: 1 (lowest)
    /// 
    /// # Grammar
    /// ```text
    /// ternary → logical_or ("?" expression ":" expression)?
    /// ```
    ///
    /// # Examples
    /// - `x > 0 ? 1 : -1`
    /// - `condition ? true_value : false_value`
    ///
    /// # Associativity
    /// Right-associative: `a ? b : c ? d : e` parses as `a ? b : (c ? d : e)`
    fn ternary(&mut self) -> Result<Expression> {
        let mut expr = self.logical_or()?;
        
        if self.peek() == &Token::Question {
            self.advance(); // consume '?'
            let true_expr = self.expression()?;
            
            if self.peek() != &Token::Colon {
                return Err(GizmoError::ParseError(format!(
                    "Expected ':' in ternary operation, found '{:?}'", self.peek()
                )));
            }
            self.advance(); // consume ':'
            
            let false_expr = self.expression()?;
            
            expr = Expression::TernaryOperation {
                condition: Box::new(expr),
                true_expr: Box::new(true_expr),
                false_expr: Box::new(false_expr),
            };
        }
        
        Ok(expr)
    }
    
    /// Parses logical OR expressions.
    ///
    /// Logical OR has short-circuit evaluation: if the left operand is true (non-zero),
    /// the right operand is not evaluated.
    ///
    /// # Precedence Level: 2
    /// 
    /// # Grammar
    /// ```text
    /// logical_or → logical_and ("or" logical_and)*
    /// ```
    ///
    /// # Examples
    /// - `x > 0 or y > 0`
    /// - `condition1 or condition2 or condition3`
    ///
    /// # Associativity
    /// Left-associative: `a or b or c` parses as `(a or b) or c`
    fn logical_or(&mut self) -> Result<Expression> {
        let mut expr = self.logical_and()?;
        
        while matches!(self.peek(), Token::Or) {
            let operator = match self.advance() {
                Token::Or => BinaryOperator::Or,
                _ => unreachable!(),
            };
            let right = self.logical_and()?;
            expr = Expression::BinaryOperation {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }
        
        Ok(expr)
    }
    
    /// Parses logical AND expressions.
    ///
    /// Logical AND has short-circuit evaluation: if the left operand is false (zero),
    /// the right operand is not evaluated.
    ///
    /// # Precedence Level: 3
    /// 
    /// # Grammar
    /// ```text
    /// logical_and → equality ("and" equality)*
    /// ```
    ///
    /// # Examples
    /// - `x > 0 and x < 10`
    /// - `condition1 and condition2 and condition3`
    ///
    /// # Associativity
    /// Left-associative: `a and b and c` parses as `(a and b) and c`
    fn logical_and(&mut self) -> Result<Expression> {
        let mut expr = self.equality()?;
        
        while matches!(self.peek(), Token::And) {
            let operator = match self.advance() {
                Token::And => BinaryOperator::And,
                _ => unreachable!(),
            };
            let right = self.equality()?;
            expr = Expression::BinaryOperation {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }
        
        Ok(expr)
    }
    
    /// Parses equality comparison expressions.
    ///
    /// Equality operations compare two values for exact equality or inequality.
    /// Uses floating-point epsilon comparison for numeric values.
    ///
    /// # Precedence Level: 4
    /// 
    /// # Grammar
    /// ```text
    /// equality → comparison ("==" comparison)*
    /// ```
    ///
    /// # Examples
    /// - `x == 42`
    /// - `result != expected`
    ///
    /// # Associativity
    /// Left-associative: `a == b == c` parses as `(a == b) == c`
    ///
    /// # Note
    /// Currently only implements `==` operator. The `!=` operator would be
    /// handled similarly but is not yet in the token set.
    fn equality(&mut self) -> Result<Expression> {
        let mut expr = self.comparison()?;
        
        while matches!(self.peek(), Token::EqualEqual) {
            let operator = match self.advance() {
                Token::EqualEqual => BinaryOperator::Equal,
                _ => unreachable!(),
            };
            let right = self.comparison()?;
            expr = Expression::BinaryOperation {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }
        
        Ok(expr)
    }
    
    /// Parses relational comparison expressions.
    ///
    /// Comparison operations test ordering relationships between values.
    /// All comparisons return 1.0 for true, 0.0 for false.
    ///
    /// # Precedence Level: 5
    /// 
    /// # Grammar
    /// ```text
    /// comparison → term ((">" | ">=" | "<" | "<=") term)*
    /// ```
    ///
    /// # Examples
    /// - `x > 10`
    /// - `y <= max_value`
    /// - `min_val < value < max_val` (parses as `(min_val < value) < max_val`)
    ///
    /// # Associativity
    /// Left-associative: `a < b < c` parses as `(a < b) < c`
    ///
    /// # Operators
    /// - `>`: Greater than
    /// - `>=`: Greater than or equal
    /// - `<`: Less than  
    /// - `<=`: Less than or equal
    fn comparison(&mut self) -> Result<Expression> {
        let mut expr = self.term()?;
        
        while matches!(self.peek(), Token::Greater | Token::GreaterEqual | Token::Less | Token::LessEqual) {
            let operator = match self.advance() {
                Token::Greater => BinaryOperator::Greater,
                Token::GreaterEqual => BinaryOperator::GreaterEqual,
                Token::Less => BinaryOperator::Less,
                Token::LessEqual => BinaryOperator::LessEqual,
                _ => unreachable!(),
            };
            let right = self.term()?;
            expr = Expression::BinaryOperation {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }
        
        Ok(expr)
    }
    
    /// Parses addition and subtraction expressions.
    ///
    /// Term-level operations handle addition and subtraction with equal precedence.
    /// Both operations work on numeric values.
    ///
    /// # Precedence Level: 6
    /// 
    /// # Grammar
    /// ```text
    /// term → factor (("+" | "-") factor)*
    /// ```
    ///
    /// # Examples
    /// - `x + y`
    /// - `a - b + c`
    /// - `position + offset - adjustment`
    ///
    /// # Associativity
    /// Left-associative: `a - b + c` parses as `(a - b) + c`
    ///
    /// # Operators
    /// - `+`: Addition
    /// - `-`: Subtraction
    fn term(&mut self) -> Result<Expression> {
        let mut expr = self.factor()?;
        
        while matches!(self.peek(), Token::Plus | Token::Minus) {
            let operator = match self.advance() {
                Token::Plus => BinaryOperator::Add,
                Token::Minus => BinaryOperator::Subtract,
                _ => unreachable!(),
            };
            let right = self.factor()?;
            expr = Expression::BinaryOperation {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }
        
        Ok(expr)
    }
    
    /// Parses multiplication, division, and modulo expressions.
    ///
    /// Factor-level operations have the highest precedence among binary operators.
    /// All operations work on numeric values.
    ///
    /// # Precedence Level: 7 (highest binary precedence)
    /// 
    /// # Grammar
    /// ```text
    /// factor → unary (("*" | "/" | "%") unary)*
    /// ```
    ///
    /// # Examples
    /// - `x * y`
    /// - `area / 2`
    /// - `index % array_size`
    /// - `a * b / c` (parses as `(a * b) / c`)
    ///
    /// # Associativity
    /// Left-associative: `a / b * c` parses as `(a / b) * c`
    ///
    /// # Operators
    /// - `*`: Multiplication
    /// - `/`: Division (error on division by zero)
    /// - `%`: Modulo (remainder after division)
    fn factor(&mut self) -> Result<Expression> {
        let mut expr = self.unary()?;
        
        while matches!(self.peek(), Token::Star | Token::Slash | Token::Percent) {
            let operator = match self.advance() {
                Token::Star => BinaryOperator::Multiply,
                Token::Slash => BinaryOperator::Divide,
                Token::Percent => BinaryOperator::Modulo,
                _ => unreachable!(),
            };
            let right = self.unary()?;
            expr = Expression::BinaryOperation {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }
        
        Ok(expr)
    }
    
    /// Parses unary expressions.
    ///
    /// Currently, this is a placeholder that delegates to primary expressions.
    /// In the future, this could handle unary operators like `-`, `+`, or `!`.
    ///
    /// # Precedence Level: 8 (would be highest if implemented)
    /// 
    /// # Grammar
    /// ```text
    /// unary → ("-" | "+" | "!")? primary
    /// ```
    ///
    /// # Future Extensions
    /// Potential unary operators to implement:
    /// - `-x`: Negation
    /// - `+x`: Unary plus (no-op)
    /// - `!x`: Logical not
    fn unary(&mut self) -> Result<Expression> {
        // For now, just delegate to primary - can add unary operators later
        self.primary()
    }
    
    fn primary(&mut self) -> Result<Expression> {
        match self.advance().clone() {
            Token::Number(n) => Ok(Expression::Number(n)),
            Token::String(s) => Ok(Expression::String(s)),
            Token::Identifier(name) => {
                // Check if this is a function call
                if self.peek() == &Token::LeftParen {
                    self.advance(); // consume '('
                    let args = self.argument_list()?;
                    
                    if self.peek() != &Token::RightParen {
                        return Err(GizmoError::ParseError(format!(
                            "Expected ')', found '{:?}'", self.peek()
                        )));
                    }
                    self.advance();
                    
                    Ok(Expression::FunctionCall { name, args })
                } else {
                    Ok(Expression::Identifier(name))
                }
            }
            Token::Pattern => {
                self.pattern_expression()
            }
            Token::LeftParen => {
                let expr = self.expression()?;
                if self.peek() != &Token::RightParen {
                    return Err(GizmoError::ParseError(format!(
                        "Expected ')', found '{:?}'", self.peek()
                    )));
                }
                self.advance();
                Ok(expr)
            }
            Token::LeftBracket => self.array_literal(),
            Token::Newline => {
                // Skip newlines and try again
                self.skip_newlines();
                if !self.is_at_end() {
                    self.primary()
                } else {
                    Err(GizmoError::ParseError("Unexpected end of input".to_string()))
                }
            }
            token => Err(GizmoError::ParseError(format!(
                "Unexpected token '{:?}'", token
            ))),
        }
    }
    
    /// Parses a pattern generator expression.
    ///
    /// Pattern generators are the core feature of Gizmo, creating pixel art by
    /// evaluating expressions for each pixel coordinate. The pattern body can
    /// contain setup statements, and must end with a return expression that
    /// determines whether each pixel is on or off.
    ///
    /// # Grammar
    /// ```text
    /// pattern_expression → "pattern" "(" expression "," expression ")"
    ///                       "{" statement* "return" expression "}"
    /// ```
    ///
    /// # Examples
    /// ```gzmo
    /// pattern(8, 8) {
    ///     distance = sqrt((col - 4)^2 + (row - 4)^2)
    ///     return distance < 3
    /// }
    /// ```
    ///
    /// # Execution Model
    /// During interpretation, the pattern is evaluated for each pixel (col, row):
    /// 1. Set `col` and `row` variables to current pixel coordinates
    /// 2. Execute all statements in the body
    /// 3. Evaluate the return expression to determine pixel state (true = on, false = off)
    ///
    /// This allows complex procedural generation with per-pixel calculations.
    fn pattern_expression(&mut self) -> Result<Expression> {
        // Expect opening parenthesis
        if self.peek() != &Token::LeftParen {
            return Err(GizmoError::ParseError(format!(
                "Expected '(' after 'pattern', found '{:?}'", self.peek()
            )));
        }
        self.advance(); // consume '('
        
        // Parse width expression
        let width = self.expression()?;
        
        // Expect comma separator
        if self.peek() != &Token::Comma {
            return Err(GizmoError::ParseError(format!(
                "Expected ',' after pattern width, found '{:?}'", self.peek()
            )));
        }
        self.advance(); // consume ','
        
        // Parse height expression
        let height = self.expression()?;
        
        // Expect closing parenthesis
        if self.peek() != &Token::RightParen {
            return Err(GizmoError::ParseError(format!(
                "Expected ')' after pattern height, found '{:?}'", self.peek()
            )));
        }
        self.advance(); // consume ')'
        
        // Expect opening brace for pattern body
        if self.peek() != &Token::LeftBrace {
            return Err(GizmoError::ParseError(format!(
                "Expected '{{' after pattern parameters, found '{:?}'", self.peek()
            )));
        }
        self.advance(); // consume '{'
        
        self.skip_newlines(); // Allow flexible formatting after opening brace
        
        // Parse the pattern body: statements + mandatory return expression
        let mut body = Vec::new();
        let mut return_expr = None;
        
        while self.peek() != &Token::RightBrace && !self.is_at_end() {
            if self.peek() == &Token::Newline {
                self.advance();
                continue;
            }
            
            // Check for return statement (mandatory)
            if self.peek() == &Token::Return {
                self.advance(); // consume 'return'
                return_expr = Some(Box::new(self.expression()?));
                
                // Optional semicolon after return expression
                if self.peek() == &Token::Semicolon {
                    self.advance();
                }
                break;
            } else {
                // Regular statement in pattern body
                body.push(self.statement()?);
            }
        }
        
        // Return expression is mandatory for pattern generators
        let return_expr = return_expr.ok_or_else(|| {
            GizmoError::ParseError("Pattern body must end with a return expression".to_string())
        })?;
        
        self.skip_newlines(); // Allow flexible formatting before closing brace
        
        // Expect closing brace
        if self.peek() != &Token::RightBrace {
            return Err(GizmoError::ParseError(format!(
                "Expected '}}' to close pattern body, found '{:?}'", self.peek()
            )));
        }
        self.advance(); // consume '}'
        
        Ok(Expression::PatternGenerator {
            width: Box::new(width),
            height: Box::new(height),
            body,
            return_expr,
        })
    }
    
    fn array_literal(&mut self) -> Result<Expression> {
        let mut elements = Vec::new();
        
        self.skip_newlines(); // Skip newlines after opening bracket
        
        if self.peek() != &Token::RightBracket {
            elements.push(self.expression()?);
            
            while self.peek() == &Token::Comma {
                self.advance();
                self.skip_newlines(); // Skip newlines after comma
                if self.peek() == &Token::RightBracket {
                    break; // Allow trailing comma
                }
                elements.push(self.expression()?);
            }
        }
        
        self.skip_newlines(); // Skip newlines before closing bracket
        
        if self.peek() != &Token::RightBracket {
            return Err(GizmoError::ParseError(format!(
                "Expected ']', found '{:?}'", self.peek()
            )));
        }
        self.advance();
        
        Ok(Expression::Array(elements))
    }
    
    fn argument_list(&mut self) -> Result<Vec<Expression>> {
        let mut args = Vec::new();
        
        self.skip_newlines(); // Skip newlines after opening paren
        
        if self.peek() != &Token::RightParen {
            args.push(self.expression()?);
            
            while self.peek() == &Token::Comma {
                self.advance();
                self.skip_newlines(); // Skip newlines after comma
                if self.peek() == &Token::RightParen {
                    break; // Allow trailing comma
                }
                args.push(self.expression()?);
            }
        }
        
        self.skip_newlines(); // Skip newlines before closing paren
        
        Ok(args)
    }
    
    /// Skips any newline tokens at the current position.
    ///
    /// Newlines are significant for statement separation but are often
    /// optional for formatting. This method is used throughout the parser
    /// to handle flexible whitespace in block structures.
    ///
    /// # Usage
    /// Called after keywords like `then`, `do`, `{` to allow:
    /// ```gzmo
    /// if condition then
    ///     statement
    /// end
    /// ```
    /// Instead of requiring: `if condition then statement end`
    fn skip_newlines(&mut self) {
        while self.peek() == &Token::Newline {
            self.advance();
        }
    }
    
    /// Checks if the parser has reached the end of the token stream.
    ///
    /// Returns true if either:
    /// - We've consumed all tokens
    /// - The current token is EOF
    ///
    /// Used throughout parsing to prevent reading beyond the input.
    fn is_at_end(&self) -> bool {
        self.current >= self.tokens.len() || self.peek() == &Token::Eof
    }
    
    /// Returns the current token without consuming it.
    ///
    /// Provides lookahead for parsing decisions. Returns EOF token
    /// if at or past the end of the token stream.
    ///
    /// # Returns
    /// Reference to the current token, or `&Token::Eof` if at end
    fn peek(&self) -> &Token {
        if self.current >= self.tokens.len() {
            &Token::Eof
        } else {
            &self.tokens[self.current]
        }
    }
    
    /// Consumes and returns the current token.
    ///
    /// Advances the parser position and returns the token that was consumed.
    /// If already at the end, does not advance further.
    ///
    /// # Returns
    /// Reference to the token that was just consumed
    fn advance(&mut self) -> &Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.previous()
    }
    
    /// Returns the previously consumed token.
    ///
    /// Used by `advance()` to return the token that was just consumed.
    /// Returns EOF if at the beginning of the stream.
    ///
    /// # Returns
    /// Reference to the previous token, or `&Token::Eof` if at start
    fn previous(&self) -> &Token {
        if self.current == 0 {
            &Token::Eof
        } else {
            &self.tokens[self.current - 1]
        }
    }
    
    /// Checks if the next token indicates an assignment statement.
    ///
    /// Used to distinguish between assignments (`var = expr`) and expression
    /// statements (`function_call()`) when the current token is an identifier.
    ///
    /// # Returns
    /// `true` if the token after the current identifier is `=`, `false` otherwise
    ///
    /// # Lookahead Strategy
    /// This simple one-token lookahead is sufficient because assignment
    /// always follows the pattern: `IDENTIFIER = expression`
    fn peek_ahead_is_assignment(&self) -> bool {
        // Look ahead to see if the next token after the identifier is '='
        if self.current + 1 < self.tokens.len() {
            matches!(self.tokens[self.current + 1], Token::Equal)
        } else {
            false
        }
    }
    
    fn peek_ahead_for_return(&self) -> bool {
        // Simple heuristic: if we see "return" keyword or if the next statement 
        // looks like it's the last expression (followed by } or end of file)
        // For now, we'll look for the pattern where there's no assignment
        if matches!(self.peek(), Token::Return) {
            return true;
        }
        
        // Look ahead to see if this looks like a final expression
        // (not an assignment or declaration)
        let mut lookahead = self.current;
        let mut depth = 0;
        while lookahead < self.tokens.len() {
            match &self.tokens[lookahead] {
                Token::LeftBrace | Token::LeftParen | Token::LeftBracket => depth += 1,
                Token::RightBrace => {
                    if depth == 0 {
                        return true; // Found closing brace, likely final expression
                    }
                    depth -= 1;
                }
                Token::RightParen | Token::RightBracket => {
                    if depth > 0 {
                        depth -= 1;
                    }
                }
                Token::Equal => {
                    if depth == 0 {
                        return false; // Found assignment, not a return expression
                    }
                }
                Token::Semicolon | Token::Newline => {
                    if depth == 0 {
                        // This suggests it's a statement, not the final expression
                        // But we need to check if there are more statements after
                        let mut next_lookahead = lookahead + 1;
                        while next_lookahead < self.tokens.len() && 
                              matches!(self.tokens[next_lookahead], Token::Newline) {
                            next_lookahead += 1;
                        }
                        if next_lookahead < self.tokens.len() && 
                           matches!(self.tokens[next_lookahead], Token::RightBrace) {
                            return true; // Last statement before closing brace
                        }
                        return false;
                    }
                }
                Token::Eof => return true,
                _ => {}
            }
            lookahead += 1;
        }
        false
    }
}