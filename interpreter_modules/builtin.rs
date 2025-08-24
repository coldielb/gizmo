use crate::ast::{Value, Frame};
use crate::error::{GizmoError, Result};
use std::collections::HashMap;
use rand::Rng;

pub struct BuiltinFunctions {
    functions: HashMap<String, fn(&[Value]) -> Result<Value>>,
}

impl BuiltinFunctions {
    pub fn new() -> Self {
        let mut functions: HashMap<String, fn(&[Value]) -> Result<Value>> = HashMap::new();
        
        // Math functions
        functions.insert("sin".to_string(), math_sin);
        functions.insert("cos".to_string(), math_cos);
        functions.insert("tan".to_string(), math_tan);
        functions.insert("sqrt".to_string(), math_sqrt);
        functions.insert("floor".to_string(), math_floor);
        functions.insert("ceil".to_string(), math_ceil);
        functions.insert("abs".to_string(), math_abs);
        functions.insert("random".to_string(), math_random);
        
        // Frame operations
        functions.insert("count_neighbors".to_string(), frame_count_neighbors);
        functions.insert("get_pixel".to_string(), frame_get_pixel);
        functions.insert("flip".to_string(), frame_flip);
        functions.insert("rotate".to_string(), frame_rotate);
        functions.insert("place_sprite".to_string(), frame_place_sprite);
        
        // Array operations
        functions.insert("add_frame".to_string(), array_add_frame);
        functions.insert("last_frame".to_string(), array_last_frame);
        functions.insert("evolve_from".to_string(), frame_evolve_from);
        
        // Animation controls - these would be handled by the interpreter
        functions.insert("play".to_string(), animation_play);
        functions.insert("loop".to_string(), animation_loop);
        functions.insert("play_speed".to_string(), animation_play_speed);
        functions.insert("stop".to_string(), animation_stop);
        
        Self { functions }
    }
    
    pub fn get(&self, name: &str) -> Option<&fn(&[Value]) -> Result<Value>> {
        self.functions.get(name)
    }
    
    pub fn has_function(&self, name: &str) -> bool {
        self.functions.contains_key(name)
    }
    
    pub fn call(&self, name: &str, args: &[Value]) -> Result<Value> {
        if let Some(func) = self.functions.get(name) {
            func(args)
        } else {
            Err(GizmoError::UndefinedFunction(name.to_string()))
        }
    }
}

impl Default for BuiltinFunctions {
    fn default() -> Self {
        Self::new()
    }
}

// Math functions
fn math_sin(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(GizmoError::ArgumentError(
            format!("sin expects 1 argument, got {}", args.len())
        ));
    }
    
    let n = args[0].to_number()?;
    Ok(Value::Number(n.sin()))
}

fn math_cos(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(GizmoError::ArgumentError(
            format!("cos expects 1 argument, got {}", args.len())
        ));
    }
    
    let n = args[0].to_number()?;
    Ok(Value::Number(n.cos()))
}

fn math_tan(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(GizmoError::ArgumentError(
            format!("tan expects 1 argument, got {}", args.len())
        ));
    }
    
    let n = args[0].to_number()?;
    Ok(Value::Number(n.tan()))
}

fn math_sqrt(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(GizmoError::ArgumentError(
            format!("sqrt expects 1 argument, got {}", args.len())
        ));
    }
    
    let n = args[0].to_number()?;
    if n < 0.0 {
        return Err(GizmoError::RuntimeError(
            "sqrt of negative number".to_string()
        ));
    }
    Ok(Value::Number(n.sqrt()))
}

fn math_floor(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(GizmoError::ArgumentError(
            format!("floor expects 1 argument, got {}", args.len())
        ));
    }
    
    let n = args[0].to_number()?;
    Ok(Value::Number(n.floor()))
}

fn math_ceil(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(GizmoError::ArgumentError(
            format!("ceil expects 1 argument, got {}", args.len())
        ));
    }
    
    let n = args[0].to_number()?;
    Ok(Value::Number(n.ceil()))
}

fn math_abs(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(GizmoError::ArgumentError(
            format!("abs expects 1 argument, got {}", args.len())
        ));
    }
    
    let n = args[0].to_number()?;
    Ok(Value::Number(n.abs()))
}

fn math_random(args: &[Value]) -> Result<Value> {
    if !args.is_empty() {
        return Err(GizmoError::ArgumentError(
            format!("random expects 0 arguments, got {}", args.len())
        ));
    }
    
    let mut rng = rand::thread_rng();
    Ok(Value::Number(rng.gen::<f64>()))
}

// Frame operations
fn frame_count_neighbors(args: &[Value]) -> Result<Value> {
    if args.len() != 3 {
        return Err(GizmoError::ArgumentError(
            format!("count_neighbors expects 3 arguments, got {}", args.len())
        ));
    }
    
    let frame = match &args[0] {
        Value::Frame(f) => f,
        _ => return Err(GizmoError::TypeError(
            "count_neighbors first argument must be a frame".to_string()
        )),
    };
    
    let row = args[1].to_number()? as usize;
    let col = args[2].to_number()? as usize;
    
    if row >= frame.height || col >= frame.width {
        return Err(GizmoError::IndexError(
            format!("Position ({}, {}) is out of bounds", row, col)
        ));
    }
    
    let count = frame.count_neighbors(row, col);
    Ok(Value::Number(count as f64))
}

fn frame_get_pixel(args: &[Value]) -> Result<Value> {
    if args.len() != 3 {
        return Err(GizmoError::ArgumentError(
            format!("get_pixel expects 3 arguments, got {}", args.len())
        ));
    }
    
    let frame = match &args[0] {
        Value::Frame(f) => f,
        _ => return Err(GizmoError::TypeError(
            "get_pixel first argument must be a frame".to_string()
        )),
    };
    
    let row = args[1].to_number()? as usize;
    let col = args[2].to_number()? as usize;
    
    let pixel = frame.get_pixel(row, col)?;
    Ok(Value::Number(if pixel { 1.0 } else { 0.0 }))
}

fn frame_flip(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(GizmoError::ArgumentError(
            format!("flip expects 1 argument, got {}", args.len())
        ));
    }
    
    let mut frame = match &args[0] {
        Value::Frame(f) => f.clone(),
        _ => return Err(GizmoError::TypeError(
            "flip argument must be a frame".to_string()
        )),
    };
    
    frame.flip();
    Ok(Value::Frame(frame))
}

fn frame_rotate(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(GizmoError::ArgumentError(
            format!("rotate expects 1 argument, got {}", args.len())
        ));
    }
    
    let frame = match &args[0] {
        Value::Frame(f) => f,
        _ => return Err(GizmoError::TypeError(
            "rotate argument must be a frame".to_string()
        )),
    };
    
    let rotated = frame.rotate_90();
    Ok(Value::Frame(rotated))
}

fn frame_place_sprite(args: &[Value]) -> Result<Value> {
    if args.len() != 3 {
        return Err(GizmoError::ArgumentError(
            format!("place_sprite expects 3 arguments, got {}", args.len())
        ));
    }
    
    let sprite = match &args[0] {
        Value::Frame(f) => f,
        _ => return Err(GizmoError::TypeError(
            "place_sprite first argument must be a frame (sprite)".to_string()
        )),
    };
    
    let x = args[1].to_number()? as i32;
    let y = args[2].to_number()? as i32;
    
    // Create a 128x128 canvas for sprite placement
    let mut canvas = Frame::new(128, 128);
    canvas.place_sprite(sprite, x, y);
    
    Ok(Value::Frame(canvas))
}

// Array operations
fn array_add_frame(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(GizmoError::ArgumentError(
            format!("add_frame expects 2 arguments, got {}", args.len())
        ));
    }
    
    let mut frames = match &args[0] {
        Value::Frames(f) => f.clone(),
        _ => return Err(GizmoError::TypeError(
            "add_frame first argument must be a frames array".to_string()
        )),
    };
    
    let frame = match &args[1] {
        Value::Frame(f) => f.clone(),
        _ => return Err(GizmoError::TypeError(
            "add_frame second argument must be a frame".to_string()
        )),
    };
    
    frames.push(frame);
    Ok(Value::Frames(frames))
}

fn array_last_frame(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(GizmoError::ArgumentError(
            format!("last_frame expects 1 argument, got {}", args.len())
        ));
    }
    
    let frames = match &args[0] {
        Value::Frames(f) => f,
        _ => return Err(GizmoError::TypeError(
            "last_frame argument must be a frames array".to_string()
        )),
    };
    
    if frames.is_empty() {
        return Err(GizmoError::RuntimeError(
            "Cannot get last frame from empty array".to_string()
        ));
    }
    
    Ok(Value::Frame(frames[frames.len() - 1].clone()))
}

fn frame_evolve_from(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(GizmoError::ArgumentError(
            format!("evolve_from expects 2 arguments, got {}", args.len())
        ));
    }
    
    // This is a placeholder - actual cellular automata evolution 
    // would be handled by the interpreter when executing CellularGenerator
    let _prev_frame = match &args[0] {
        Value::Frame(f) => f,
        _ => return Err(GizmoError::TypeError(
            "evolve_from first argument must be a frame".to_string()
        )),
    };
    
    // The second argument would be a cellular generator function
    // For now, just return the previous frame
    Ok(args[0].clone())
}

// Animation controls - these are handled specially by the interpreter
fn animation_play(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(GizmoError::ArgumentError(
            format!("play expects 1 argument, got {}", args.len())
        ));
    }
    
    match &args[0] {
        Value::Frames(_) => {
            // This would trigger the animation system in the interpreter
            Ok(Value::Boolean(true))
        }
        Value::Frame(_) => {
            // Single frame display
            Ok(Value::Boolean(true))
        }
        _ => Err(GizmoError::TypeError(
            "play argument must be a frame or frames array".to_string()
        )),
    }
}

fn animation_loop(args: &[Value]) -> Result<Value> {
    if args.len() != 1 {
        return Err(GizmoError::ArgumentError(
            format!("loop expects 1 argument, got {}", args.len())
        ));
    }
    
    match &args[0] {
        Value::Frames(_) => {
            // This would trigger looping animation in the interpreter
            Ok(Value::Boolean(true))
        }
        _ => Err(GizmoError::TypeError(
            "loop argument must be a frames array".to_string()
        )),
    }
}

fn animation_play_speed(args: &[Value]) -> Result<Value> {
    if args.len() != 2 {
        return Err(GizmoError::ArgumentError(
            format!("play_speed expects 2 arguments, got {}", args.len())
        ));
    }
    
    match (&args[0], &args[1]) {
        (Value::Frames(_), Value::Number(_ms)) => {
            // This would set animation speed in the interpreter
            Ok(Value::Boolean(true))
        }
        _ => Err(GizmoError::TypeError(
            "play_speed expects frames array and number".to_string()
        )),
    }
}

fn animation_stop(_args: &[Value]) -> Result<Value> {
    // This would stop current animation in the interpreter
    Ok(Value::Boolean(true))
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_math_functions() {
        let builtins = BuiltinFunctions::new();
        
        let args = vec![Value::Number(0.0)];
        let result = builtins.call("sin", &args).unwrap();
        assert_eq!(result, Value::Number(0.0));
        
        let args = vec![Value::Number(4.0)];
        let result = builtins.call("sqrt", &args).unwrap();
        assert_eq!(result, Value::Number(2.0));
        
        let args = vec![Value::Number(3.7)];
        let result = builtins.call("floor", &args).unwrap();
        assert_eq!(result, Value::Number(3.0));
    }
    
    #[test]
    fn test_frame_operations() {
        let builtins = BuiltinFunctions::new();
        
        let frame_data = vec![
            vec![true, false, true],
            vec![false, true, false],
            vec![true, false, true],
        ];
        let frame = Frame::from_array(frame_data).unwrap();
        
        let args = vec![
            Value::Frame(frame),
            Value::Number(1.0), // row
            Value::Number(1.0), // col
        ];
        
        let result = builtins.call("count_neighbors", &args).unwrap();
        assert_eq!(result, Value::Number(4.0));
    }
}