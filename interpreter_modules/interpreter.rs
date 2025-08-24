use crate::ast::*;
use crate::builtin::BuiltinFunctions;
use crate::error::{GizmoError, Result};
use crate::frame::{AnimationState, FrameRenderer};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Clone)]
pub struct Environment {
    variables: HashMap<String, Value>,
    parent: Option<Box<Environment>>,
}

impl Environment {
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
            parent: None,
        }
    }
    
    pub fn new_enclosed(parent: Environment) -> Self {
        Self {
            variables: HashMap::new(),
            parent: Some(Box::new(parent)),
        }
    }
    
    pub fn define(&mut self, name: String, value: Value) {
        self.variables.insert(name, value);
    }
    
    pub fn get(&self, name: &str) -> Result<Value> {
        if let Some(value) = self.variables.get(name) {
            Ok(value.clone())
        } else if let Some(parent) = &self.parent {
            parent.get(name)
        } else {
            Err(GizmoError::UndefinedVariable(name.to_string()))
        }
    }
    
    pub fn assign(&mut self, name: &str, value: Value) -> Result<()> {
        if self.variables.contains_key(name) {
            self.variables.insert(name.to_string(), value);
            Ok(())
        } else if let Some(parent) = &mut self.parent {
            parent.assign(name, value)
        } else {
            Err(GizmoError::UndefinedVariable(name.to_string()))
        }
    }
}

pub struct Interpreter {
    globals: Environment,
    environment: Environment,
    builtins: BuiltinFunctions,
    animation_state: Option<AnimationState>,
    current_time: u64,
    event_handlers: HashMap<String, Vec<Statement>>,
    frame_renderer: FrameRenderer,
    output_frames: Vec<Frame>,
}

impl Interpreter {
    pub fn new() -> Self {
        let globals = Environment::new();
        let environment = Environment::new();
        let builtins = BuiltinFunctions::new();
        let frame_renderer = FrameRenderer::new(128, 128);
        
        Self {
            globals,
            environment,
            builtins,
            animation_state: None,
            current_time: Self::get_current_time(),
            event_handlers: HashMap::new(),
            frame_renderer,
            output_frames: Vec::new(),
        }
    }
    
    fn get_current_time() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64
    }
    
    pub fn execute(&mut self, program: &Program) -> Result<()> {
        for statement in &program.statements {
            self.execute_statement(statement)?;
        }
        Ok(())
    }
    
    pub fn get_output_frames(&self) -> &[Frame] {
        &self.output_frames
    }
    
    pub fn update(&mut self, delta_ms: u64) -> Option<&Frame> {
        self.current_time += delta_ms;
        
        if let Some(ref mut anim) = self.animation_state {
            anim.update(self.current_time)
        } else {
            None
        }
    }
    
    pub fn handle_click_event(&mut self) -> Result<()> {
        if let Some(statements) = self.event_handlers.get("clicked") {
            let statements = statements.clone(); // Clone to avoid borrow checker issues
            for statement in &statements {
                self.execute_statement(statement)?;
            }
        }
        Ok(())
    }
    
    pub fn handle_idle_event(&mut self, idle_time: u64) -> Result<()> {
        let key = format!("idle_{}", idle_time);
        if let Some(statements) = self.event_handlers.get(&key) {
            let statements = statements.clone();
            for statement in &statements {
                self.execute_statement(statement)?;
            }
        }
        Ok(())
    }
    
    fn execute_statement(&mut self, stmt: &Statement) -> Result<Option<Value>> {
        match stmt {
            Statement::VariableDeclaration { var_type: _, name, value } => {
                let val = self.evaluate_expression(value)?;
                self.environment.define(name.clone(), val);
                Ok(None)
            }
            
            Statement::ExpressionStatement(expr) => {
                let result = self.evaluate_expression(expr)?;
                
                // Handle special animation functions
                if let Expression::FunctionCall { name, args } = expr {
                    match name.as_str() {
                        "play" | "loop" | "play_speed" => {
                            self.handle_animation_function(name, args)?;
                        }
                        _ => {}
                    }
                }
                
                Ok(Some(result))
            }
            
            Statement::IfStatement { condition, then_block, elsif_blocks, else_block } => {
                let condition_value = self.evaluate_expression(condition)?;
                
                if condition_value.is_truthy() {
                    self.execute_block(then_block)
                } else {
                    // Check elsif conditions
                    for (elsif_condition, elsif_body) in elsif_blocks {
                        let elsif_value = self.evaluate_expression(elsif_condition)?;
                        if elsif_value.is_truthy() {
                            return self.execute_block(elsif_body);
                        }
                    }
                    
                    // Execute else block if present
                    if let Some(else_body) = else_block {
                        self.execute_block(else_body)
                    } else {
                        Ok(None)
                    }
                }
            }
            
            Statement::RepeatStatement { times, body } => {
                let times_value = self.evaluate_expression(times)?;
                let count = times_value.to_number()? as i32;
                
                for _i in 0..count {
                    if let Some(return_value) = self.execute_block(body)? {
                        return Ok(Some(return_value));
                    }
                }
                
                Ok(None)
            }
            
            Statement::WhenStatement { event, body } => {
                let key = match event {
                    Event::Clicked => "clicked".to_string(),
                    Event::Idle(time_expr) => {
                        let time = self.evaluate_expression(time_expr)?;
                        let time_ms = time.to_number()? as u64;
                        format!("idle_{}", time_ms)
                    }
                };
                
                self.event_handlers.insert(key, body.clone());
                Ok(None)
            }
            
            Statement::ReturnStatement(expr) => {
                let value = self.evaluate_expression(expr)?;
                Ok(Some(value))
            }
        }
    }
    
    fn execute_block(&mut self, statements: &[Statement]) -> Result<Option<Value>> {
        for statement in statements {
            if let Some(return_value) = self.execute_statement(statement)? {
                return Ok(Some(return_value));
            }
        }
        Ok(None)
    }
    
    fn evaluate_expression(&mut self, expr: &Expression) -> Result<Value> {
        match expr {
            Expression::Number(n) => Ok(Value::Number(*n)),
            Expression::String(s) => Ok(Value::String(s.clone())),
            Expression::Boolean(b) => Ok(Value::Boolean(*b)),
            
            Expression::Identifier(name) => {
                // Check for special variables first
                match name.as_str() {
                    "time" => Ok(Value::Number(self.current_time as f64)),
                    "row" | "col" => {
                        // These are set during pattern/animation generation
                        self.environment.get(name)
                    }
                    _ => self.environment.get(name),
                }
            }
            
            Expression::Array(elements) => {
                let mut values = Vec::new();
                for element in elements {
                    values.push(self.evaluate_expression(element)?);
                }
                
                // Determine if this is a frame (2D array of numbers) or frames array
                if values.iter().all(|v| matches!(v, Value::Frame(_))) {
                    let frames: Result<Vec<Frame>> = values.into_iter()
                        .map(|v| match v {
                            Value::Frame(f) => Ok(f),
                            _ => unreachable!(),
                        })
                        .collect();
                    Ok(Value::Frames(frames?))
                } else if values.iter().all(|v| matches!(v, Value::Number(_))) {
                    // This is a 1D array of numbers - convert to frame row
                    let pixel_row: Result<Vec<bool>> = values.iter()
                        .map(|v| Ok(v.to_number()? != 0.0))
                        .collect();
                    let frame = Frame::from_array(vec![pixel_row?])?;
                    Ok(Value::Frame(frame))
                } else {
                    // Check if this is a 2D frame array (array of arrays)
                    self.try_create_2d_frame(&values)
                }
            }
            
            Expression::FunctionCall { name, args } => {
                let arg_values: Result<Vec<Value>> = args.iter()
                    .map(|arg| self.evaluate_expression(arg))
                    .collect();
                let arg_values = arg_values?;
                
                if self.builtins.has_function(name) {
                    self.builtins.call(name, &arg_values)
                } else {
                    Err(GizmoError::UndefinedFunction(name.clone()))
                }
            }
            
            Expression::Binary { left, operator, right } => {
                let left_val = self.evaluate_expression(left)?;
                let right_val = self.evaluate_expression(right)?;
                self.apply_binary_operator(&left_val, operator, &right_val)
            }
            
            Expression::Unary { operator, operand } => {
                let operand_val = self.evaluate_expression(operand)?;
                self.apply_unary_operator(operator, &operand_val)
            }
            
            Expression::Index { object, index } => {
                let obj_val = self.evaluate_expression(object)?;
                let idx_val = self.evaluate_expression(index)?;
                self.apply_index_operator(&obj_val, &idx_val)
            }
            
            Expression::PatternGenerator { width, height, body } => {
                let w = self.evaluate_expression(width)?.to_number()? as usize;
                let h = self.evaluate_expression(height)?.to_number()? as usize;
                
                self.generate_pattern_frame(w, h, body)
            }
            
            Expression::AnimatedGenerator { width, height, time_var, body } => {
                let w = self.evaluate_expression(width)?.to_number()? as usize;
                let h = self.evaluate_expression(height)?.to_number()? as usize;
                
                self.generate_animated_frame(w, h, time_var, body)
            }
            
            Expression::CellularGenerator { width, height, prev_var, body } => {
                let w = self.evaluate_expression(width)?.to_number()? as usize;
                let h = self.evaluate_expression(height)?.to_number()? as usize;
                
                self.generate_cellular_frame(w, h, prev_var, body)
            }
            
            Expression::Conditional { condition, then_expr, else_expr } => {
                let condition_val = self.evaluate_expression(condition)?;
                
                if condition_val.is_truthy() {
                    self.evaluate_expression(then_expr)
                } else {
                    self.evaluate_expression(else_expr)
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
                            "Cannot combine multi-row frames in array".to_string()
                        ));
                    }
                }
                _ => {
                    return Err(GizmoError::TypeError(
                        "Mixed array types in frame definition".to_string()
                    ));
                }
            }
        }
        
        if !frame_rows.is_empty() {
            let frame = Frame::from_array(frame_rows)?;
            Ok(Value::Frame(frame))
        } else {
            Err(GizmoError::TypeError(
                "Cannot create frame from empty array".to_string()
            ))
        }
    }
    
    
    fn generate_pattern_frame(&mut self, width: usize, height: usize, body: &[Statement]) -> Result<Value> {
        let mut frame = Frame::new(width, height);
        
        for row in 0..height {
            for col in 0..width {
                // Create new scope with row and col variables
                let mut pattern_env = Environment::new_enclosed(self.environment.clone());
                pattern_env.define("row".to_string(), Value::Number(row as f64));
                pattern_env.define("col".to_string(), Value::Number(col as f64));
                
                let old_env = std::mem::replace(&mut self.environment, pattern_env);
                
                // Execute pattern body
                let mut pixel_value = false;
                for statement in body {
                    if let Some(return_val) = self.execute_statement(statement)? {
                        pixel_value = return_val.is_truthy();
                        break;
                    }
                }
                
                self.environment = old_env;
                frame.set_pixel(row, col, pixel_value)?;
            }
        }
        
        Ok(Value::Frame(frame))
    }
    
    fn generate_animated_frame(&mut self, width: usize, height: usize, time_var: &str, body: &[Statement]) -> Result<Value> {
        let mut frame = Frame::new(width, height);
        
        for row in 0..height {
            for col in 0..width {
                // Create new scope with row, col, and time variables
                let mut anim_env = Environment::new_enclosed(self.environment.clone());
                anim_env.define("row".to_string(), Value::Number(row as f64));
                anim_env.define("col".to_string(), Value::Number(col as f64));
                anim_env.define(time_var.to_string(), Value::Number(self.current_time as f64));
                
                let old_env = std::mem::replace(&mut self.environment, anim_env);
                
                // Execute animation body
                let mut pixel_value = false;
                for statement in body {
                    if let Some(return_val) = self.execute_statement(statement)? {
                        pixel_value = return_val.is_truthy();
                        break;
                    }
                }
                
                self.environment = old_env;
                frame.set_pixel(row, col, pixel_value)?;
            }
        }
        
        Ok(Value::Frame(frame))
    }
    
    fn generate_cellular_frame(&mut self, width: usize, height: usize, prev_var: &str, body: &[Statement]) -> Result<Value> {
        // For cellular automata, we need a previous frame
        let prev_frame = match self.environment.get(prev_var) {
            Ok(Value::Frame(frame)) => frame,
            _ => return Err(GizmoError::RuntimeError(
                "Cellular automaton requires previous frame".to_string()
            )),
        };
        
        let mut frame = Frame::new(width, height);
        
        for row in 0..height {
            for col in 0..width {
                // Create new scope with row, col, and previous frame
                let mut cell_env = Environment::new_enclosed(self.environment.clone());
                cell_env.define("row".to_string(), Value::Number(row as f64));
                cell_env.define("col".to_string(), Value::Number(col as f64));
                cell_env.define(prev_var.to_string(), Value::Frame(prev_frame.clone()));
                
                let old_env = std::mem::replace(&mut self.environment, cell_env);
                
                // Execute cellular body
                let mut pixel_value = false;
                for statement in body {
                    if let Some(return_val) = self.execute_statement(statement)? {
                        pixel_value = return_val.is_truthy();
                        break;
                    }
                }
                
                self.environment = old_env;
                frame.set_pixel(row, col, pixel_value)?;
            }
        }
        
        Ok(Value::Frame(frame))
    }
    
    fn handle_animation_function(&mut self, func_name: &str, args: &[Expression]) -> Result<()> {
        let arg_values: Result<Vec<Value>> = args.iter()
            .map(|arg| self.evaluate_expression(arg))
            .collect();
        let arg_values = arg_values?;
        
        match func_name {
            "play" => {
                if let Some(Value::Frames(frames)) = arg_values.get(0) {
                    self.animation_state = Some(AnimationState::new(frames.clone(), false, 100));
                    self.output_frames = frames.clone();
                }
            }
            "loop" => {
                if let Some(Value::Frames(frames)) = arg_values.get(0) {
                    self.animation_state = Some(AnimationState::new(frames.clone(), true, 100));
                    self.output_frames = frames.clone();
                }
            }
            "play_speed" => {
                if let (Some(Value::Frames(frames)), Some(Value::Number(ms))) = 
                   (arg_values.get(0), arg_values.get(1)) {
                    let duration = *ms as u64;
                    self.animation_state = Some(AnimationState::new(frames.clone(), false, duration));
                    self.output_frames = frames.clone();
                }
            }
            _ => {}
        }
        
        Ok(())
    }
    
    fn apply_binary_operator(&self, left: &Value, op: &BinaryOperator, right: &Value) -> Result<Value> {
        match op {
            BinaryOperator::Add => {
                let left_num = left.to_number()?;
                let right_num = right.to_number()?;
                Ok(Value::Number(left_num + right_num))
            }
            BinaryOperator::Subtract => {
                let left_num = left.to_number()?;
                let right_num = right.to_number()?;
                Ok(Value::Number(left_num - right_num))
            }
            BinaryOperator::Multiply => {
                let left_num = left.to_number()?;
                let right_num = right.to_number()?;
                Ok(Value::Number(left_num * right_num))
            }
            BinaryOperator::Divide => {
                let left_num = left.to_number()?;
                let right_num = right.to_number()?;
                if right_num == 0.0 {
                    return Err(GizmoError::DivisionByZero);
                }
                Ok(Value::Number(left_num / right_num))
            }
            BinaryOperator::Modulo => {
                let left_num = left.to_number()?;
                let right_num = right.to_number()?;
                if right_num == 0.0 {
                    return Err(GizmoError::DivisionByZero);
                }
                Ok(Value::Number(left_num % right_num))
            }
            BinaryOperator::Power => {
                let left_num = left.to_number()?;
                let right_num = right.to_number()?;
                Ok(Value::Number(left_num.powf(right_num)))
            }
            BinaryOperator::Equal => {
                Ok(Value::Boolean(self.values_equal(left, right)))
            }
            BinaryOperator::NotEqual => {
                Ok(Value::Boolean(!self.values_equal(left, right)))
            }
            BinaryOperator::Less => {
                let left_num = left.to_number()?;
                let right_num = right.to_number()?;
                Ok(Value::Boolean(left_num < right_num))
            }
            BinaryOperator::Greater => {
                let left_num = left.to_number()?;
                let right_num = right.to_number()?;
                Ok(Value::Boolean(left_num > right_num))
            }
            BinaryOperator::LessEqual => {
                let left_num = left.to_number()?;
                let right_num = right.to_number()?;
                Ok(Value::Boolean(left_num <= right_num))
            }
            BinaryOperator::GreaterEqual => {
                let left_num = left.to_number()?;
                let right_num = right.to_number()?;
                Ok(Value::Boolean(left_num >= right_num))
            }
            BinaryOperator::And => {
                Ok(Value::Boolean(left.is_truthy() && right.is_truthy()))
            }
            BinaryOperator::Or => {
                Ok(Value::Boolean(left.is_truthy() || right.is_truthy()))
            }
        }
    }
    
    fn apply_unary_operator(&self, op: &UnaryOperator, operand: &Value) -> Result<Value> {
        match op {
            UnaryOperator::Not => Ok(Value::Boolean(!operand.is_truthy())),
            UnaryOperator::Minus => {
                let num = operand.to_number()?;
                Ok(Value::Number(-num))
            }
        }
    }
    
    fn apply_index_operator(&self, object: &Value, index: &Value) -> Result<Value> {
        match (object, index) {
            (Value::Frame(frame), Value::Number(idx)) => {
                let row_idx = *idx as usize;
                if row_idx >= frame.height {
                    return Err(GizmoError::IndexError(
                        format!("Row index {} out of bounds", row_idx)
                    ));
                }
                // Return the row as a frame
                let row_data = vec![frame.pixels[row_idx].clone()];
                let row_frame = Frame::from_array(row_data)?;
                Ok(Value::Frame(row_frame))
            }
            (Value::Frames(frames), Value::Number(idx)) => {
                let frame_idx = *idx as usize;
                if frame_idx >= frames.len() {
                    return Err(GizmoError::IndexError(
                        format!("Frame index {} out of bounds", frame_idx)
                    ));
                }
                Ok(Value::Frame(frames[frame_idx].clone()))
            }
            _ => Err(GizmoError::TypeError(
                "Invalid index operation".to_string()
            )),
        }
    }
    
    fn values_equal(&self, left: &Value, right: &Value) -> bool {
        match (left, right) {
            (Value::Number(a), Value::Number(b)) => (a - b).abs() < f64::EPSILON,
            (Value::String(a), Value::String(b)) => a == b,
            (Value::Boolean(a), Value::Boolean(b)) => a == b,
            (Value::Frame(a), Value::Frame(b)) => a == b,
            (Value::Frames(a), Value::Frames(b)) => a == b,
            _ => false,
        }
    }
    
    pub fn render_current_frame(&self) -> Option<String> {
        if let Some(ref anim_state) = self.animation_state {
            if let Some(frame) = anim_state.frames.get(anim_state.current_frame) {
                return Some(self.frame_renderer.render_ascii(frame));
            }
        }
        
        if !self.output_frames.is_empty() {
            Some(self.frame_renderer.render_ascii(&self.output_frames[0]))
        } else {
            None
        }
    }
}