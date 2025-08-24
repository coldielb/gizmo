use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub statements: Vec<Statement>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    VariableDeclaration {
        var_type: VariableType,
        name: String,
        value: Expression,
    },
    ExpressionStatement(Expression),
    IfStatement {
        condition: Expression,
        then_block: Vec<Statement>,
        elsif_blocks: Vec<(Expression, Vec<Statement>)>,
        else_block: Option<Vec<Statement>>,
    },
    RepeatStatement {
        times: Expression,
        body: Vec<Statement>,
    },
    WhenStatement {
        event: Event,
        body: Vec<Statement>,
    },
    ReturnStatement(Expression),
}

#[derive(Debug, Clone, PartialEq)]
pub enum VariableType {
    Frame,
    Frames,
    Num,
    Text,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Event {
    Clicked,
    Idle(Expression), // time in milliseconds
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    // Literals
    Number(f64),
    String(String),
    Boolean(bool),
    
    // Variables and identifiers
    Identifier(String),
    
    // Array/Frame literals
    Array(Vec<Expression>),
    
    // Function calls
    FunctionCall {
        name: String,
        args: Vec<Expression>,
    },
    
    // Binary operations
    Binary {
        left: Box<Expression>,
        operator: BinaryOperator,
        right: Box<Expression>,
    },
    
    // Unary operations
    Unary {
        operator: UnaryOperator,
        operand: Box<Expression>,
    },
    
    // Array/object access
    Index {
        object: Box<Expression>,
        index: Box<Expression>,
    },
    
    // Frame generators
    PatternGenerator {
        width: Box<Expression>,
        height: Box<Expression>,
        body: Vec<Statement>,
    },
    
    AnimatedGenerator {
        width: Box<Expression>,
        height: Box<Expression>,
        time_var: String,
        body: Vec<Statement>,
    },
    
    CellularGenerator {
        width: Box<Expression>,
        height: Box<Expression>,
        prev_var: String,
        body: Vec<Statement>,
    },
    
    // Conditional expression
    Conditional {
        condition: Box<Expression>,
        then_expr: Box<Expression>,
        else_expr: Box<Expression>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum BinaryOperator {
    // Arithmetic
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
    Power,
    
    // Comparison
    Equal,
    NotEqual,
    Less,
    Greater,
    LessEqual,
    GreaterEqual,
    
    // Logical
    And,
    Or,
}

#[derive(Debug, Clone, PartialEq)]
pub enum UnaryOperator {
    Not,
    Minus,
}

// Value types for the interpreter
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Number(f64),
    String(String),
    Boolean(bool),
    Frame(Frame),
    Frames(Vec<Frame>),
    Function {
        params: Vec<String>,
        body: Vec<Statement>,
        closure: HashMap<String, Value>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct Frame {
    pub width: usize,
    pub height: usize,
    pub pixels: Vec<Vec<bool>>, // true = on (1), false = off (0)
}

impl Frame {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            pixels: vec![vec![false; width]; height],
        }
    }
    
    pub fn from_array(data: Vec<Vec<bool>>) -> Result<Self, crate::error::GizmoError> {
        if data.is_empty() {
            return Err(crate::error::GizmoError::InvalidFrameSize(
                "Frame cannot be empty".to_string()
            ));
        }
        
        let height = data.len();
        let width = data[0].len();
        
        // Validate all rows have the same width
        for (i, row) in data.iter().enumerate() {
            if row.len() != width {
                return Err(crate::error::GizmoError::InvalidFrameSize(
                    format!("Row {} has length {} but expected {}", i, row.len(), width)
                ));
            }
        }
        
        Ok(Self {
            width,
            height,
            pixels: data,
        })
    }
    
    pub fn get_pixel(&self, row: usize, col: usize) -> Result<bool, crate::error::GizmoError> {
        if row >= self.height || col >= self.width {
            return Err(crate::error::GizmoError::IndexError(
                format!("Pixel ({}, {}) is out of bounds for {}x{} frame", 
                        row, col, self.width, self.height)
            ));
        }
        Ok(self.pixels[row][col])
    }
    
    pub fn set_pixel(&mut self, row: usize, col: usize, value: bool) -> Result<(), crate::error::GizmoError> {
        if row >= self.height || col >= self.width {
            return Err(crate::error::GizmoError::IndexError(
                format!("Pixel ({}, {}) is out of bounds for {}x{} frame", 
                        row, col, self.width, self.height)
            ));
        }
        self.pixels[row][col] = value;
        Ok(())
    }
    
    pub fn count_neighbors(&self, row: usize, col: usize) -> usize {
        let mut count = 0;
        let row = row as i32;
        let col = col as i32;
        
        for dr in -1..=1 {
            for dc in -1..=1 {
                if dr == 0 && dc == 0 {
                    continue; // Skip the center cell
                }
                
                let nr = row + dr;
                let nc = col + dc;
                
                if nr >= 0 && nr < self.height as i32 && nc >= 0 && nc < self.width as i32 {
                    if self.pixels[nr as usize][nc as usize] {
                        count += 1;
                    }
                }
            }
        }
        
        count
    }
    
    pub fn flip(&mut self) {
        for row in &mut self.pixels {
            for pixel in row {
                *pixel = !*pixel;
            }
        }
    }
    
    pub fn rotate_90(&self) -> Self {
        let new_width = self.height;
        let new_height = self.width;
        let mut new_pixels = vec![vec![false; new_width]; new_height];
        
        for row in 0..self.height {
            for col in 0..self.width {
                new_pixels[col][self.height - 1 - row] = self.pixels[row][col];
            }
        }
        
        Self {
            width: new_width,
            height: new_height,
            pixels: new_pixels,
        }
    }
    
    pub fn place_sprite(&mut self, sprite: &Frame, x: i32, y: i32) {
        for row in 0..sprite.height {
            for col in 0..sprite.width {
                let target_row = y + row as i32;
                let target_col = x + col as i32;
                
                if target_row >= 0 && target_row < self.height as i32 &&
                   target_col >= 0 && target_col < self.width as i32 {
                    if sprite.pixels[row][col] {
                        self.pixels[target_row as usize][target_col as usize] = true;
                    }
                }
            }
        }
    }
}

impl Value {
    pub fn type_name(&self) -> &'static str {
        match self {
            Value::Number(_) => "number",
            Value::String(_) => "string",
            Value::Boolean(_) => "boolean",
            Value::Frame(_) => "frame",
            Value::Frames(_) => "frames",
            Value::Function { .. } => "function",
        }
    }
    
    pub fn is_truthy(&self) -> bool {
        match self {
            Value::Boolean(b) => *b,
            Value::Number(n) => *n != 0.0,
            Value::String(s) => !s.is_empty(),
            Value::Frame(_) => true,
            Value::Frames(frames) => !frames.is_empty(),
            Value::Function { .. } => true,
        }
    }
    
    pub fn to_number(&self) -> Result<f64, crate::error::GizmoError> {
        match self {
            Value::Number(n) => Ok(*n),
            Value::Boolean(b) => Ok(if *b { 1.0 } else { 0.0 }),
            Value::String(s) => {
                s.parse::<f64>().map_err(|_| {
                    crate::error::GizmoError::TypeError(
                        format!("Cannot convert string '{}' to number", s)
                    )
                })
            }
            _ => Err(crate::error::GizmoError::TypeError(
                format!("Cannot convert {} to number", self.type_name())
            )),
        }
    }
    
    pub fn to_string(&self) -> String {
        match self {
            Value::Number(n) => {
                if n.fract() == 0.0 {
                    format!("{}", *n as i64)
                } else {
                    format!("{}", n)
                }
            }
            Value::String(s) => s.clone(),
            Value::Boolean(b) => b.to_string(),
            Value::Frame(frame) => format!("Frame({}x{})", frame.width, frame.height),
            Value::Frames(frames) => format!("Frames({})", frames.len()),
            Value::Function { .. } => "Function".to_string(),
        }
    }
}