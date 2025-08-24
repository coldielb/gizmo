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
    Assignment {
        name: String,
        value: Expression,
    },
    RepeatLoop {
        count: Box<Expression>,
        body: Vec<Statement>,
    },
    IfStatement {
        condition: Expression,
        then_body: Vec<Statement>,
        else_body: Option<Vec<Statement>>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum VariableType {
    Frame,
    Frames,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    Number(f64),
    String(String),
    Identifier(String),
    Array(Vec<Expression>),
    FunctionCall {
        name: String,
        args: Vec<Expression>,
    },
    BinaryOperation {
        left: Box<Expression>,
        operator: BinaryOperator,
        right: Box<Expression>,
    },
    PatternGenerator {
        width: Box<Expression>,
        height: Box<Expression>,
        body: Vec<Statement>,
        return_expr: Box<Expression>,
    },
    TernaryOperation {
        condition: Box<Expression>,
        true_expr: Box<Expression>,
        false_expr: Box<Expression>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum BinaryOperator {
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
    Equal,
    NotEqual,
    Greater,
    Less,
    GreaterEqual,
    LessEqual,
    And,
    Or,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Number(f64),
    String(String),
    Frame(Frame),
    Frames(Vec<Frame>),
}


#[derive(Debug, Clone, PartialEq)]
pub struct Frame {
    pub width: usize,
    pub height: usize,
    pub pixels: Vec<Vec<bool>>, // true = on (1), false = off (0)
}

impl Frame {
    pub fn new(data: Vec<Vec<bool>>) -> Self {
        if data.is_empty() {
            Self {
                width: 0,
                height: 0,
                pixels: vec![],
            }
        } else {
            let height = data.len();
            let width = data[0].len();
            Self {
                width,
                height,
                pixels: data,
            }
        }
    }
    
    pub fn new_blank(width: usize, height: usize) -> Self {
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
    
    pub fn get_data(&self) -> &Vec<Vec<bool>> {
        &self.pixels
    }
}

impl Value {
    pub fn to_number(&self) -> Result<f64, crate::error::GizmoError> {
        match self {
            Value::Number(n) => Ok(*n),
            _ => Err(crate::error::GizmoError::TypeError(
                "Cannot convert to number".to_string()
            )),
        }
    }
}