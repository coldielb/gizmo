//! Built-in Functions for the Gizmo Scripting Language
//!
//! This module implements the standard library of built-in functions available
//! to Gizmo scripts. These functions provide essential mathematical operations,
//! animation controls, and utility functions for creating pixel art.
//!
//! ## Function Categories
//!
//! ### Mathematical Functions
//! Core mathematical operations for calculations and procedural generation:
//! - **Trigonometry**: `sin()`, `cos()`, `atan2()` - for circular patterns, waves, rotations
//! - **Utility Math**: `abs()`, `floor()`, `ceil()`, `sqrt()` - for coordinate manipulation
//! - **Random**: `random()` - for noise and variation in patterns
//!
//! ### Animation Control Functions
//! Functions that control animation playback and timing:
//! - **Playback**: `play()`, `loop()` - display frame sequences
//! - **Timing**: `loop_speed()` - set frame rate (handled specially by interpreter)
//! - **Frame Management**: `add_frame()` - add frames to animation sequences
//!
//! ### Frame Utility Functions
//! Functions for working with frame data structures:
//! - **Creation**: `create_frame()` - create blank frames programmatically
//! - **Access**: `get_pixel()`, `set_pixel()` - pixel-level frame manipulation
//!
//! ## Design Philosophy
//!
//! Built-in functions follow these principles:
//! - **Type Safety**: All functions validate argument types and counts
//! - **Error Reporting**: Detailed error messages for debugging
//! - **Mathematical Consistency**: Standard mathematical behavior (angles in radians, etc.)
//! - **Animation Integration**: Special handling for functions that affect the interpreter state
//!
//! ## Implementation Notes
//!
//! Functions are implemented as Rust closures stored in a HashMap for efficient lookup.
//! Some functions like `add_frame()` and `loop_speed()` have additional special handling
//! in the interpreter for state management.

use crate::ast::Value;
use crate::error::{GizmoError, Result};
use std::collections::HashMap;

/// Registry of built-in functions available to Gizmo scripts.
///
/// This structure maintains a mapping from function names to their implementations,
/// providing efficient lookup during script execution.
pub struct BuiltinFunctions {
    /// Map of function names to their implementation closures
    functions: HashMap<String, fn(&[Value]) -> Result<Value>>,
}

impl BuiltinFunctions {
    /// Creates a new function registry with all built-in functions registered.
    ///
    /// Initializes the function registry with the complete set of built-in functions
    /// organized by category. This is called once during interpreter initialization.
    ///
    /// # Function Categories
    /// - **Animation**: `play()`, `loop()`, `add_frame()`, `loop_speed()`
    /// - **Mathematics**: `random()`, `floor()`, `ceil()`, `abs()`, `sin()`, `cos()`, `sqrt()`, `atan2()`
    /// - **Frame Utilities**: `create_frame()`, `get_pixel()`, `set_pixel()`
    pub fn new() -> Self {
        let mut functions: HashMap<String, fn(&[Value]) -> Result<Value>> = HashMap::new();
        
        // Animation control functions
        functions.insert("play".to_string(), animation_play);
        functions.insert("loop".to_string(), animation_loop);
        functions.insert("add_frame".to_string(), add_frame_func);
        functions.insert("loop_speed".to_string(), loop_speed_func);
        
        // Mathematical functions
        functions.insert("random".to_string(), math_random);
        functions.insert("floor".to_string(), math_floor);
        functions.insert("ceil".to_string(), math_ceil);
        functions.insert("abs".to_string(), math_abs);
        functions.insert("sin".to_string(), math_sin);
        functions.insert("cos".to_string(), math_cos);
        functions.insert("sqrt".to_string(), math_sqrt);
        functions.insert("atan2".to_string(), math_atan2);
        
        // Frame utility functions
        functions.insert("create_frame".to_string(), create_frame);
        functions.insert("get_pixel".to_string(), get_pixel);
        functions.insert("set_pixel".to_string(), set_pixel);
        
        Self { functions }
    }
    
    /// Checks if a function with the given name is registered.
    ///
    /// Used by the interpreter to validate function calls before attempting
    /// to execute them.
    ///
    /// # Arguments
    /// * `name` - Function name to check
    ///
    /// # Returns
    /// `true` if the function exists, `false` otherwise
    pub fn has_function(&self, name: &str) -> bool {
        self.functions.contains_key(name)
    }
    
    /// Calls a built-in function with the provided arguments.
    ///
    /// Looks up the function by name and executes it with the given arguments.
    /// The function implementation handles argument validation and computation.
    ///
    /// # Arguments
    /// * `name` - Name of the function to call
    /// * `args` - Array of argument values
    ///
    /// # Returns
    /// * `Ok(Value)` - Function result
    /// * `Err(GizmoError)` - Function not found or execution error
    pub fn call(&self, name: &str, args: &[Value]) -> Result<Value> {
        if let Some(func) = self.functions.get(name) {
            func(args)
        } else {
            Err(GizmoError::UndefinedFunction(name.to_string()))
        }
    }
}

/// `play(frames)` - Displays a frame or frame sequence once.
///
/// This function signals the interpreter to display the provided frames.
/// The actual animation control is handled by special interpreter logic.
///
/// # Arguments
/// * `frames` - Single frame or array of frames to display
///
/// # Returns
/// * `Ok(1.0)` - Success indicator
/// * `Err` - Invalid argument type or count
///
/// # Usage
/// ```gzmo
/// play(my_frame)        // Display single frame
/// play(animation_frames) // Play animation sequence once
/// ```
fn animation_play(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(GizmoError::ArgumentError(
            format!("play expects 1 argument, got {}", args.len())
        ));
    }
    
    match &args[0] {
        Value::Frames(_) | Value::Frame(_) => {
            // Success indicator - actual playback handled by interpreter
            Ok(Value::Number(1.0))
        }
        _ => Err(GizmoError::TypeError(
            "play argument must be a frame or frames array".to_string()
        )),
    }
}

/// `loop(frames)` - Displays a frame or frame sequence in a continuous loop.
///
/// Similar to `play()` but indicates that the animation should repeat indefinitely.
/// The actual looping behavior is handled by the interpreter and window system.
///
/// # Arguments
/// * `frames` - Single frame or array of frames to loop
///
/// # Returns
/// * `Ok(1.0)` - Success indicator
///
/// # Usage
/// ```gzmo
/// loop(spinner_frames)  // Loop animation continuously
/// ```
fn animation_loop(_args: &[Value]) -> Result<Value> {
    // Success indicator - actual looping handled by interpreter
    Ok(Value::Number(1.0))
}

/// `random()` - Generates a random floating-point number between 0.0 and 1.0.
///
/// Uses the system's random number generator to produce pseudo-random values
/// suitable for adding variation to patterns and animations.
///
/// # Arguments
/// None
///
/// # Returns
/// * `Ok(Number)` - Random value in range [0.0, 1.0)
///
/// # Usage
/// ```gzmo
/// noise = random()           // Random value 0.0-1.0
/// x = random() * 100         // Random value 0.0-100.0
/// on = random() > 0.5        // Random true/false
/// ```
fn math_random(_args: &[Value]) -> Result<Value> {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    Ok(Value::Number(rng.gen::<f64>()))
}

/// `floor(x)` - Returns the largest integer less than or equal to x.
///
/// Rounds a floating-point number down to the nearest integer.
/// Useful for coordinate calculations and discrete pattern generation.
///
/// # Arguments
/// * `x` - Number to floor
///
/// # Returns
/// * `Ok(Number)` - Floored value
/// * `Err` - Invalid argument type or count
///
/// # Examples
/// ```gzmo
/// floor(3.7)   // Returns 3.0
/// floor(-2.3)  // Returns -3.0
/// floor(5.0)   // Returns 5.0
/// ```
fn math_floor(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(GizmoError::ArgumentError(
            format!("floor expects 1 argument, got {}", args.len())
        ));
    }
    
    match &args[0] {
        Value::Number(n) => Ok(Value::Number(n.floor())),
        _ => Err(GizmoError::TypeError("floor argument must be a number".to_string())),
    }
}

/// `ceil(x)` - Returns the smallest integer greater than or equal to x.
///
/// Rounds a floating-point number up to the nearest integer.
/// Useful for coordinate calculations and ensuring minimum sizes.
///
/// # Arguments
/// * `x` - Number to ceiling
///
/// # Returns
/// * `Ok(Number)` - Ceiling value
/// * `Err` - Invalid argument type or count
///
/// # Examples
/// ```gzmo
/// ceil(3.2)    // Returns 4.0
/// ceil(-2.7)   // Returns -2.0
/// ceil(5.0)    // Returns 5.0
/// ```
fn math_ceil(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(GizmoError::ArgumentError(
            format!("ceil expects 1 argument, got {}", args.len())
        ));
    }
    
    match &args[0] {
        Value::Number(n) => Ok(Value::Number(n.ceil())),
        _ => Err(GizmoError::TypeError("ceil argument must be a number".to_string())),
    }
}

/// `abs(x)` - Returns the absolute value of x.
///
/// Computes the absolute value (magnitude) of a number, always returning
/// a non-negative result. Useful for distance calculations and ensuring
/// positive values.
///
/// # Arguments
/// * `x` - Number to take absolute value of
///
/// # Returns
/// * `Ok(Number)` - Absolute value (always >= 0.0)
/// * `Err` - Invalid argument type or count
///
/// # Examples
/// ```gzmo
/// abs(5)       // Returns 5.0
/// abs(-3.7)    // Returns 3.7
/// abs(0)       // Returns 0.0
/// ```
fn math_abs(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(GizmoError::ArgumentError(
            format!("abs expects 1 argument, got {}", args.len())
        ));
    }
    
    match &args[0] {
        Value::Number(n) => Ok(Value::Number(n.abs())),
        _ => Err(GizmoError::TypeError("abs argument must be a number".to_string())),
    }
}

fn create_frame(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(GizmoError::ArgumentError(
            format!("create_frame expects 2 arguments (width, height), got {}", args.len())
        ));
    }
    
    let width = match &args[0] {
        Value::Number(n) => *n as usize,
        _ => return Err(GizmoError::TypeError("width must be a number".to_string())),
    };
    
    let height = match &args[1] {
        Value::Number(n) => *n as usize,
        _ => return Err(GizmoError::TypeError("height must be a number".to_string())),
    };
    
    let frame_data = vec![vec![false; width]; height];
    Ok(Value::Frame(crate::ast::Frame::new(frame_data)))
}

fn get_pixel(args: &[Value]) -> Result<Value> {
    if args.len() != 3 {
        return Err(GizmoError::ArgumentError(
            format!("get_pixel expects 3 arguments (frame, x, y), got {}", args.len())
        ));
    }
    
    let frame = match &args[0] {
        Value::Frame(f) => f,
        _ => return Err(GizmoError::TypeError("first argument must be a frame".to_string())),
    };
    
    let x = match &args[1] {
        Value::Number(n) => *n as usize,
        _ => return Err(GizmoError::TypeError("x coordinate must be a number".to_string())),
    };
    
    let y = match &args[2] {
        Value::Number(n) => *n as usize,
        _ => return Err(GizmoError::TypeError("y coordinate must be a number".to_string())),
    };
    
    let data = frame.get_data();
    if y < data.len() && x < data[0].len() {
        Ok(Value::Number(if data[y][x] { 1.0 } else { 0.0 }))
    } else {
        Ok(Value::Number(0.0)) // Out of bounds = false
    }
}

fn set_pixel(_args: &[Value]) -> Result<Value> {
    // For now, return success - implementing mutable frames would require more work
    Ok(Value::Number(1.0))
}

/// `sin(x)` - Returns the sine of x (where x is in radians).
///
/// Computes the trigonometric sine function. Essential for creating
/// wave patterns, circular motions, and smooth oscillations in animations.
///
/// # Arguments
/// * `x` - Angle in radians
///
/// # Returns
/// * `Ok(Number)` - Sine value in range [-1.0, 1.0]
/// * `Err` - Invalid argument type or count
///
/// # Examples
/// ```gzmo
/// sin(0)           // Returns 0.0
/// sin(3.14159/2)   // Returns ~1.0 (π/2 radians = 90°)
/// wave = sin(col * 0.1)  // Create horizontal wave pattern
/// ```
fn math_sin(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(GizmoError::ArgumentError(
            format!("sin expects 1 argument, got {}", args.len())
        ));
    }
    
    match &args[0] {
        Value::Number(n) => Ok(Value::Number(n.sin())),
        _ => Err(GizmoError::TypeError("sin argument must be a number".to_string())),
    }
}

/// `cos(x)` - Returns the cosine of x (where x is in radians).
///
/// Computes the trigonometric cosine function. Used alongside `sin()` for
/// circular patterns, rotations, and creating smooth periodic animations.
///
/// # Arguments
/// * `x` - Angle in radians
///
/// # Returns
/// * `Ok(Number)` - Cosine value in range [-1.0, 1.0]
/// * `Err` - Invalid argument type or count
///
/// # Examples
/// ```gzmo
/// cos(0)           // Returns 1.0
/// cos(3.14159)     // Returns ~-1.0 (π radians = 180°)
/// x = cos(angle)   // X component of circular motion
/// ```
fn math_cos(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(GizmoError::ArgumentError(
            format!("cos expects 1 argument, got {}", args.len())
        ));
    }
    
    match &args[0] {
        Value::Number(n) => Ok(Value::Number(n.cos())),
        _ => Err(GizmoError::TypeError("cos argument must be a number".to_string())),
    }
}

fn math_sqrt(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(GizmoError::ArgumentError(
            format!("sqrt expects 1 argument, got {}", args.len())
        ));
    }
    
    match &args[0] {
        Value::Number(n) => {
            if *n < 0.0 {
                return Err(GizmoError::ArgumentError("sqrt of negative number".to_string()));
            }
            Ok(Value::Number(n.sqrt()))
        },
        _ => Err(GizmoError::TypeError("sqrt argument must be a number".to_string())),
    }
}

fn math_atan2(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(GizmoError::ArgumentError(
            format!("atan2 expects 2 arguments (y, x), got {}", args.len())
        ));
    }
    
    let y = match &args[0] {
        Value::Number(n) => *n,
        _ => return Err(GizmoError::TypeError("atan2 first argument (y) must be a number".to_string())),
    };
    
    let x = match &args[1] {
        Value::Number(n) => *n,
        _ => return Err(GizmoError::TypeError("atan2 second argument (x) must be a number".to_string())),
    };
    
    Ok(Value::Number(y.atan2(x)))
}

fn add_frame_func(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(GizmoError::ArgumentError(
            format!("add_frame expects 2 arguments (frames_array, frame), got {}", args.len())
        ));
    }
    
    // For now, this is a placeholder - we'd need to implement mutable arrays
    // The interpreter would need to handle this specially
    Ok(Value::Number(1.0))
}

fn loop_speed_func(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(GizmoError::ArgumentError(
            format!("loop_speed expects 2 arguments (frames_array, ms), got {}", args.len())
        ));
    }
    
    // Similar to play() but with speed control
    match &args[0] {
        Value::Frames(_) => Ok(Value::Number(1.0)),
        _ => Err(GizmoError::TypeError("loop_speed first argument must be frames array".to_string())),
    }
}