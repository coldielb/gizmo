//! Interpreter for the Gizmo Scripting Language
//!
//! This module implements the execution engine that takes parsed AST nodes and
//! executes them to generate pixel art animations. The interpreter is responsible
//! for the runtime evaluation of expressions, execution of statements, and the
//! specialized pattern generation that makes Gizmo unique.
//!
//! ## Core Responsibilities
//!
//! ### Expression Evaluation
//! - **Mathematical Operations**: Full arithmetic with operator precedence
//! - **Logical Operations**: Boolean logic with short-circuit evaluation
//! - **Function Calls**: Built-in mathematical and animation functions
//! - **Pattern Generation**: Per-pixel expression evaluation for procedural art
//!
//! ### Statement Execution
//! - **Variable Management**: Scoped variable declarations and assignments
//! - **Control Flow**: If statements and repeat loops with proper scoping
//! - **Animation Functions**: Special handling for `add_frame()`, `loop_speed()`, `play()`
//!
//! ### Pattern Generation Model
//!
//! The interpreter's most sophisticated feature is pattern generation:
//!
//! ```text
//! For each pixel (col, row) in pattern(width, height):
//!   1. Set environment variables: col = x, row = y
//!   2. Execute all statements in pattern body
//!   3. Evaluate return expression → boolean (true = pixel on)
//!   4. Store result in frame[row][col]
//! ```
//!
//! This allows complex procedural generation with mathematical expressions,
//! trigonometry, distance calculations, and more.
//!
//! ## Animation System
//!
//! The interpreter manages animation state including:
//! - **Frame Collection**: Accumulates frames from `add_frame()` calls
//! - **Timing Control**: Frame duration from `loop_speed()` (1ms to 10000ms)
//! - **Playback State**: Final animation frames and timing for the window system
//!
//! ## Error Handling
//!
//! Provides detailed runtime error reporting for:
//! - Type mismatches (e.g., using string in math operation)
//! - Division by zero
//! - Undefined variables and functions
//! - Invalid function arguments

use crate::ast::*;
use crate::builtin::BuiltinFunctions;
use crate::error::{GizmoError, Result};
use crate::frame::FrameRenderer;
use std::collections::HashMap;

/// Runtime environment for variable storage and scoping.
///
/// The environment maintains a mapping from variable names to their values
/// during script execution. In the current implementation, there's a single
/// global scope, but the structure supports future scoping extensions.
#[derive(Clone)]
pub struct Environment {
    /// Map of variable names to their current values
    variables: HashMap<String, Value>,
}

impl Environment {
    /// Creates a new empty environment.
    ///
    /// Initializes an environment with no variables defined.
    /// Variables will be added through `define()` during script execution.
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
        }
    }

    /// Defines or updates a variable in the environment.
    ///
    /// This method is used for both variable declarations and assignments.
    /// If the variable already exists, it will be overwritten with the new value.
    ///
    /// # Arguments
    /// * `name` - Variable name to define
    /// * `value` - Value to associate with the variable
    pub fn define(&mut self, name: String, value: Value) {
        self.variables.insert(name, value);
    }

    /// Retrieves a variable value from the environment.
    ///
    /// Looks up the variable name and returns a copy of its value.
    /// Returns an error if the variable has not been defined.
    ///
    /// # Arguments
    /// * `name` - Variable name to look up
    ///
    /// # Returns
    /// * `Ok(Value)` - The variable's current value
    /// * `Err(GizmoError::UndefinedVariable)` - Variable not found
    pub fn get(&self, name: &str) -> Result<Value> {
        if let Some(value) = self.variables.get(name) {
            Ok(value.clone())
        } else {
            Err(GizmoError::UndefinedVariable(name.to_string()))
        }
    }
}

/// The main interpreter that executes Gizmo scripts.
///
/// The interpreter maintains all the runtime state needed to execute a script:
/// - Variable environment for storing script variables
/// - Built-in function registry for mathematical and animation functions
/// - Frame renderer for ASCII debugging output
/// - Animation state including frames and timing
pub struct Interpreter {
    /// Runtime environment for variable storage
    environment: Environment,
    /// Registry of built-in functions (sin, cos, random, etc.)
    builtins: BuiltinFunctions,
    /// Renderer for ASCII frame output (used for debugging)
    frame_renderer: FrameRenderer,
    /// Accumulated animation frames from script execution
    output_frames: Vec<Frame>,
    /// Frame duration in milliseconds (default 100ms)
    frame_duration_ms: u64,
}

impl Interpreter {
    /// Creates a new interpreter instance.
    ///
    /// Initializes the interpreter with:
    /// - Empty variable environment
    /// - Built-in function registry
    /// - Frame renderer for 128x128 output
    /// - Empty animation frame list
    /// - Default frame timing of 100ms per frame
    pub fn new() -> Self {
        Self {
            environment: Environment::new(),
            builtins: BuiltinFunctions::new(),
            frame_renderer: FrameRenderer::new(128, 128),
            output_frames: Vec::new(),
            frame_duration_ms: 100, // Default 100ms per frame
        }
    }

    /// Executes a complete Gizmo program.
    ///
    /// Processes all statements in the program sequentially, maintaining
    /// runtime state and accumulating animation frames. This is the main
    /// entry point for script execution.
    ///
    /// # Arguments
    /// * `program` - The parsed AST representing the complete script
    ///
    /// # Returns
    /// * `Ok(())` - Program executed successfully
    /// * `Err(GizmoError)` - Runtime error during execution
    ///
    /// # Side Effects
    /// - Updates interpreter state (variables, animation frames)
    /// - May produce animation frames via `add_frame()`, `play()`, etc.
    /// - Sets frame timing via `loop_speed()`
    pub fn execute(&mut self, program: &Program) -> Result<()> {
        for statement in &program.statements {
            self.execute_statement(statement)?;
        }
        Ok(())
    }

    /// Renders the current frame as ASCII text for debugging.
    ///
    /// Converts the first animation frame (if any) to ASCII representation
    /// using '#' for on pixels and '.' for off pixels.
    ///
    /// # Returns
    /// * `Some(String)` - ASCII representation of the frame
    /// * `None` - No frames available to render
    pub fn render_current_frame(&self) -> Option<String> {
        if !self.output_frames.is_empty() {
            Some(self.frame_renderer.render_ascii(&self.output_frames[0]))
        } else {
            None
        }
    }

    /// Returns all animation frames produced by the script.
    ///
    /// Provides access to the complete animation sequence accumulated
    /// during script execution. Used by the main application to drive
    /// the animation display.
    ///
    /// # Returns
    /// Vector of frames in animation order
    pub fn get_animation_frames(&self) -> Vec<crate::ast::Frame> {
        self.output_frames.clone()
    }

    /// Returns the current single frame if available.
    ///
    /// Used when a script produces a single static frame rather than
    /// an animation sequence. Provides fallback content for display.
    ///
    /// # Returns
    /// * `Some(Frame)` - First frame if available
    /// * `None` - No frames produced by script
    pub fn get_current_frame(&self) -> Option<crate::ast::Frame> {
        if !self.output_frames.is_empty() {
            Some(self.output_frames[0].clone())
        } else {
            None
        }
    }

    /// Returns the frame duration for animation timing.
    ///
    /// Provides the timing value set by `loop_speed()` function calls,
    /// or the default of 100ms if no timing was specified.
    ///
    /// # Returns
    /// Frame duration in milliseconds (clamped to 1-10000ms range)
    pub fn get_frame_duration_ms(&self) -> u64 {
        self.frame_duration_ms
    }

    /// Executes a single statement.
    ///
    /// Handles all statement types including variable operations, control flow,
    /// and expression statements with special animation function handling.
    ///
    /// # Arguments
    /// * `stmt` - The statement AST node to execute
    ///
    /// # Returns
    /// * `Ok(())` - Statement executed successfully
    /// * `Err(GizmoError)` - Runtime error during execution
    fn execute_statement(&mut self, stmt: &Statement) -> Result<()> {
        match stmt {
            Statement::VariableDeclaration {
                var_type: _,
                name,
                value,
            } => {
                let val = self.evaluate_expression(value)?;
                self.environment.define(name.clone(), val);
                Ok(())
            }

            Statement::Assignment { name, value } => {
                let val = self.evaluate_expression(value)?;
                self.environment.define(name.clone(), val);
                Ok(())
            }

            Statement::ExpressionStatement(expr) => {
                let _result = self.evaluate_expression(expr)?;

                // Special handling for animation control functions
                // These functions have side effects on the interpreter's animation state
                if let Expression::FunctionCall { name, args } = expr {
                    match name.as_str() {
                        "add_frame" => {
                            // add_frame(frames_array_name, frame) - adds frame to mutable array
                            // This is special because it modifies arrays in-place
                            if args.len() == 2 {
                                if let Expression::Identifier(array_name) = &args[0] {
                                    let frame_value = self.evaluate_expression(&args[1])?;
                                    if let Value::Frame(frame) = frame_value {
                                        // Get current frames array or create empty one
                                        let mut frames = match self.environment.get(array_name) {
                                            Ok(Value::Frames(existing_frames)) => existing_frames,
                                            _ => Vec::new(),
                                        };
                                        frames.push(frame);
                                        self.environment
                                            .define(array_name.clone(), Value::Frames(frames));
                                    }
                                }
                            }
                        }
                        "loop_speed" => {
                            // loop_speed(frames, ms) - sets animation frames and timing
                            if args.len() == 2 {
                                let frame_value = self.evaluate_expression(&args[0])?;
                                let timing_value = self.evaluate_expression(&args[1])?;

                                // Set output frames for animation
                                if let Value::Frames(frames) = frame_value {
                                    self.output_frames = frames;
                                } else if let Value::Frame(frame) = frame_value {
                                    self.output_frames = vec![frame];
                                }

                                // Set frame timing with safety bounds
                                if let Value::Number(ms) = timing_value {
                                    // Clamp to 1-10000ms range for safety and performance
                                    self.frame_duration_ms = (ms as u64).max(1).min(10000);
                                }
                            }
                        }
                        "play" | "loop" => {
                            // play(frames) / loop(frames) - sets frames for display
                            if !args.is_empty() {
                                let frame_value = self.evaluate_expression(&args[0])?;
                                if let Value::Frames(frames) = frame_value {
                                    self.output_frames = frames;
                                } else if let Value::Frame(frame) = frame_value {
                                    self.output_frames = vec![frame];
                                }
                            }
                        }
                        _ => {} // Other functions handled by builtin system
                    }
                }

                Ok(())
            }

            Statement::IfStatement {
                condition,
                then_body,
                else_body,
            } => {
                // Evaluate condition expression
                let condition_val = self.evaluate_expression(condition)?;
                let condition_true = match condition_val {
                    Value::Number(n) => n != 0.0, // 0.0 = false, anything else = true
                    _ => {
                        return Err(GizmoError::TypeError(
                            "if condition must be a number".to_string(),
                        ))
                    }
                };

                // Execute appropriate branch
                if condition_true {
                    // Execute then branch
                    for stmt in then_body {
                        self.execute_statement(stmt)?;
                    }
                } else if let Some(else_statements) = else_body {
                    // Execute else branch if present
                    for stmt in else_statements {
                        self.execute_statement(stmt)?;
                    }
                }

                Ok(())
            }

            Statement::RepeatLoop { count, body } => {
                // Evaluate loop count expression
                let count_value = self.evaluate_expression(count)?;
                let repeat_count = match count_value {
                    Value::Number(n) => n as usize,
                    _ => {
                        return Err(GizmoError::TypeError(
                            "repeat count must be a number".to_string(),
                        ))
                    }
                };

                // Execute loop body for specified number of iterations
                for i in 0..repeat_count {
                    // Provide 'time' variable with current iteration (0-based)
                    // This is useful for creating animated sequences
                    self.environment
                        .define("time".to_string(), Value::Number(i as f64));

                    // Execute all statements in loop body
                    for stmt in body {
                        self.execute_statement(stmt)?;
                    }
                }

                Ok(())
            }
        }
    }

    /// Evaluates an expression to produce a runtime value.
    ///
    /// This is the core expression evaluation method that handles all expression
    /// types including literals, variables, function calls, binary operations,
    /// and the complex pattern generation system.
    ///
    /// # Arguments
    /// * `expr` - The expression AST node to evaluate
    ///
    /// # Returns
    /// * `Ok(Value)` - The computed value of the expression
    /// * `Err(GizmoError)` - Runtime error during evaluation
    fn evaluate_expression(&mut self, expr: &Expression) -> Result<Value> {
        match expr {
            // Literal values
            Expression::Number(n) => Ok(Value::Number(*n)),
            Expression::String(s) => Ok(Value::String(s.clone())),

            // Variable lookup
            Expression::Identifier(name) => self.environment.get(name),

            Expression::Array(elements) => {
                let mut values = Vec::new();
                for element in elements {
                    values.push(self.evaluate_expression(element)?);
                }

                // Check what type of array this is
                if values.iter().all(|v| matches!(v, Value::Number(_))) {
                    // All numbers - create a frame row
                    let pixel_row: Result<Vec<bool>> =
                        values.iter().map(|v| Ok(v.to_number()? != 0.0)).collect();
                    let frame = Frame::from_array(vec![pixel_row?])?;
                    Ok(Value::Frame(frame))
                } else if values.iter().all(|v| matches!(v, Value::Frame(_))) {
                    // All frames
                    if values.len() == 1 {
                        // Single frame
                        Ok(values.into_iter().next().unwrap())
                    } else if values.iter().all(|v| match v {
                        Value::Frame(f) => f.height == 1,
                        _ => false,
                    }) {
                        // Multiple single-row frames - combine into 2D frame
                        let mut frame_rows = Vec::new();
                        for value in values {
                            if let Value::Frame(frame) = value {
                                frame_rows.push(frame.pixels[0].clone());
                            }
                        }
                        let frame = Frame::from_array(frame_rows)?;
                        Ok(Value::Frame(frame))
                    } else {
                        // Array of frames
                        let frames: Vec<Frame> = values
                            .into_iter()
                            .map(|v| match v {
                                Value::Frame(f) => f,
                                _ => unreachable!(),
                            })
                            .collect();
                        Ok(Value::Frames(frames))
                    }
                } else {
                    Err(GizmoError::TypeError(format!(
                        "Cannot create array from mixed types"
                    )))
                }
            }

            Expression::FunctionCall { name, args } => {
                let arg_values: Result<Vec<Value>> = args
                    .iter()
                    .map(|arg| self.evaluate_expression(arg))
                    .collect();
                let arg_values = arg_values?;

                if self.builtins.has_function(name) {
                    self.builtins.call(name, &arg_values)
                } else {
                    Err(GizmoError::UndefinedFunction(name.clone()))
                }
            }

            // Binary operations - arithmetic, comparison, and logical
            Expression::BinaryOperation {
                left,
                operator,
                right,
            } => {
                let left_val = self.evaluate_expression(left)?;
                let right_val = self.evaluate_expression(right)?;

                match (left_val, right_val) {
                    (Value::Number(l), Value::Number(r)) => {
                        let result = match operator {
                            // Arithmetic operations
                            BinaryOperator::Add => l + r,
                            BinaryOperator::Subtract => l - r,
                            BinaryOperator::Multiply => l * r,
                            BinaryOperator::Divide => {
                                if r == 0.0 {
                                    return Err(GizmoError::DivisionByZero);
                                }
                                l / r
                            }
                            BinaryOperator::Modulo => l % r,

                            // Comparison operations (return 1.0 for true, 0.0 for false)
                            BinaryOperator::Greater => {
                                if l > r {
                                    1.0
                                } else {
                                    0.0
                                }
                            }
                            BinaryOperator::Less => {
                                if l < r {
                                    1.0
                                } else {
                                    0.0
                                }
                            }
                            BinaryOperator::GreaterEqual => {
                                if l >= r {
                                    1.0
                                } else {
                                    0.0
                                }
                            }
                            BinaryOperator::LessEqual => {
                                if l <= r {
                                    1.0
                                } else {
                                    0.0
                                }
                            }
                            BinaryOperator::Equal => {
                                if (l - r).abs() < f64::EPSILON {
                                    1.0
                                } else {
                                    0.0
                                }
                            }
                            BinaryOperator::NotEqual => {
                                if (l - r).abs() >= f64::EPSILON {
                                    1.0
                                } else {
                                    0.0
                                }
                            }

                            // Logical operations (using numeric true/false representation)
                            BinaryOperator::And => {
                                if l != 0.0 && r != 0.0 {
                                    1.0
                                } else {
                                    0.0
                                }
                            }
                            BinaryOperator::Or => {
                                if l != 0.0 || r != 0.0 {
                                    1.0
                                } else {
                                    0.0
                                }
                            }
                        };
                        Ok(Value::Number(result))
                    }
                    _ => Err(GizmoError::TypeError(
                        "Binary operations only supported for numbers".to_string(),
                    )),
                }
            }

            // Pattern generation - the heart of Gizmo's procedural pixel art
            Expression::PatternGenerator {
                width,
                height,
                body,
                return_expr,
            } => {
                // Evaluate dimensions
                let width_val = self.evaluate_expression(width)?;
                let height_val = self.evaluate_expression(height)?;

                let w = match width_val {
                    Value::Number(n) => n as usize,
                    _ => {
                        return Err(GizmoError::TypeError(
                            "pattern width must be a number".to_string(),
                        ))
                    }
                };

                let h = match height_val {
                    Value::Number(n) => n as usize,
                    _ => {
                        return Err(GizmoError::TypeError(
                            "pattern height must be a number".to_string(),
                        ))
                    }
                };

                // Initialize frame data matrix
                let mut frame_data = vec![vec![false; w]; h];

                // PATTERN EXECUTION MODEL:
                // For each pixel coordinate (col, row), execute the pattern body
                // and evaluate the return expression to determine if pixel is on/off
                for row in 0..h {
                    for col in 0..w {
                        // Set coordinate variables for current pixel
                        // These are available to all expressions in the pattern body
                        self.environment
                            .define("row".to_string(), Value::Number(row as f64));
                        self.environment
                            .define("col".to_string(), Value::Number(col as f64));

                        // Execute all setup statements in the pattern body
                        // These can declare variables, perform calculations, etc.
                        for stmt in body {
                            self.execute_statement(stmt)?;
                        }

                        // Evaluate the return expression to get pixel state
                        let pixel_value = self.evaluate_expression(return_expr)?;
                        let pixel_on = match pixel_value {
                            Value::Number(n) => n != 0.0, // 0.0 = off, non-zero = on
                            _ => {
                                return Err(GizmoError::TypeError(
                                    "pattern expression must return a number".to_string(),
                                ))
                            }
                        };

                        // Store pixel result in frame
                        frame_data[row][col] = pixel_on;
                    }
                }

                Ok(Value::Frame(Frame::new(frame_data)))
            }

            // Ternary conditional: condition ? true_expr : false_expr
            Expression::TernaryOperation {
                condition,
                true_expr,
                false_expr,
            } => {
                let condition_val = self.evaluate_expression(condition)?;
                let condition_true = match condition_val {
                    Value::Number(n) => n != 0.0, // 0.0 = false, non-zero = true
                    _ => {
                        return Err(GizmoError::TypeError(
                            "ternary condition must be a number".to_string(),
                        ))
                    }
                };

                // Evaluate only the selected branch (short-circuit evaluation)
                if condition_true {
                    self.evaluate_expression(true_expr)
                } else {
                    self.evaluate_expression(false_expr)
                }
            }
        }
    }

    fn try_create_2d_frame(&mut self, values: &[Value]) -> Result<Value> {
        // Handle nested arrays to create 2D frame
        let mut frame_rows = Vec::new();

        for value in values {
            match value {
                Value::Frame(frame) => {
                    // If it's already a frame with a single row, extract that row
                    if frame.height == 1 {
                        frame_rows.push(frame.pixels[0].clone());
                    } else {
                        // Multi-row frame, can't combine
                        return Err(GizmoError::TypeError(
                            "Cannot combine multi-row frames in array".to_string(),
                        ));
                    }
                }
                _ => {
                    return Err(GizmoError::TypeError(
                        "Mixed array types in frame definition".to_string(),
                    ));
                }
            }
        }

        if !frame_rows.is_empty() {
            let frame = Frame::from_array(frame_rows)?;
            Ok(Value::Frame(frame))
        } else {
            Err(GizmoError::TypeError(
                "Cannot create frame from empty array".to_string(),
            ))
        }
    }
}
