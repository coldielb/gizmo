use crate::lexer::Token;
use crate::ast::*;
use crate::error::{GizmoError, Result};

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, current: 0 }
    }
    
    pub fn parse(&mut self) -> Result<Program> {
        let mut statements = Vec::new();
        
        while !self.is_at_end() {
            // Skip newlines at the top level
            if self.peek() == &Token::Newline {
                self.advance();
                continue;
            }
            
            statements.push(self.statement()?);
        }
        
        Ok(Program { statements })
    }
    
    fn statement(&mut self) -> Result<Statement> {
        match self.peek() {
            Token::Frame | Token::Frames | Token::Num | Token::Text => {
                self.variable_declaration()
            }
            Token::If => self.if_statement(),
            Token::Repeat => self.repeat_statement(),
            Token::When => self.when_statement(),
            Token::Return => self.return_statement(),
            _ => self.expression_statement(),
        }
    }
    
    fn variable_declaration(&mut self) -> Result<Statement> {
        let var_type = match self.advance() {
            Token::Frame => VariableType::Frame,
            Token::Frames => VariableType::Frames,
            Token::Num => VariableType::Num,
            Token::Text => VariableType::Text,
            token => {
                return Err(GizmoError::ParseError(format!(
                    "Expected variable type, found '{}'", token
                )));
            }
        };
        
        let name = match self.advance() {
            Token::Identifier(name) => name.clone(),
            token => {
                return Err(GizmoError::ParseError(format!(
                    "Expected identifier, found '{}'", token
                )));
            }
        };
        
        if self.peek() != &Token::Equal {
            return Err(GizmoError::ParseError(format!(
                "Expected '=', found '{}'", self.peek()
            )));
        }
        self.advance(); // consume '='
        
        let value = self.expression()?;
        
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
    
    fn if_statement(&mut self) -> Result<Statement> {
        self.advance(); // consume 'if'
        
        let condition = self.expression()?;
        
        if self.peek() != &Token::Then {
            return Err(GizmoError::ParseError(format!(
                "Expected 'then', found '{}'", self.peek()
            )));
        }
        self.advance(); // consume 'then'
        self.skip_newlines();
        
        let then_block = self.block_until(&[Token::Elsif, Token::Else, Token::End])?;
        
        let mut elsif_blocks = Vec::new();
        while self.peek() == &Token::Elsif {
            self.advance(); // consume 'elsif'
            let elsif_condition = self.expression()?;
            
            if self.peek() != &Token::Then {
                return Err(GizmoError::ParseError(format!(
                    "Expected 'then', found '{}'", self.peek()
                )));
            }
            self.advance(); // consume 'then'
            self.skip_newlines();
            
            let elsif_body = self.block_until(&[Token::Elsif, Token::Else, Token::End])?;
            elsif_blocks.push((elsif_condition, elsif_body));
        }
        
        let else_block = if self.peek() == &Token::Else {
            self.advance(); // consume 'else'
            self.skip_newlines();
            Some(self.block_until(&[Token::End])?)
        } else {
            None
        };
        
        if self.peek() != &Token::End {
            return Err(GizmoError::ParseError(format!(
                "Expected 'end', found '{}'", self.peek()
            )));
        }
        self.advance(); // consume 'end'
        
        if self.peek() == &Token::Semicolon {
            self.advance();
        }
        self.skip_newlines();
        
        Ok(Statement::IfStatement {
            condition,
            then_block,
            elsif_blocks,
            else_block,
        })
    }
    
    fn repeat_statement(&mut self) -> Result<Statement> {
        self.advance(); // consume 'repeat'
        
        let times = self.expression()?;
        
        if self.peek() != &Token::Times {
            return Err(GizmoError::ParseError(format!(
                "Expected 'times', found '{}'", self.peek()
            )));
        }
        self.advance(); // consume 'times'
        
        if self.peek() != &Token::Do {
            return Err(GizmoError::ParseError(format!(
                "Expected 'do', found '{}'", self.peek()
            )));
        }
        self.advance(); // consume 'do'
        self.skip_newlines();
        
        let body = self.block_until(&[Token::End])?;
        
        if self.peek() != &Token::End {
            return Err(GizmoError::ParseError(format!(
                "Expected 'end', found '{}'", self.peek()
            )));
        }
        self.advance(); // consume 'end'
        
        if self.peek() == &Token::Semicolon {
            self.advance();
        }
        self.skip_newlines();
        
        Ok(Statement::RepeatStatement { times, body })
    }
    
    fn when_statement(&mut self) -> Result<Statement> {
        self.advance(); // consume 'when'
        
        let event = match self.peek() {
            Token::Clicked => {
                self.advance();
                Event::Clicked
            }
            Token::Idle => {
                self.advance();
                if self.peek() != &Token::Greater {
                    return Err(GizmoError::ParseError(format!(
                        "Expected '>' after 'idle', found '{}'", self.peek()
                    )));
                }
                self.advance(); // consume '>'
                let time = self.expression()?;
                Event::Idle(time)
            }
            token => {
                return Err(GizmoError::ParseError(format!(
                    "Expected event type, found '{}'", token
                )));
            }
        };
        
        if self.peek() != &Token::Do {
            return Err(GizmoError::ParseError(format!(
                "Expected 'do', found '{}'", self.peek()
            )));
        }
        self.advance(); // consume 'do'
        self.skip_newlines();
        
        let body = self.block_until(&[Token::End])?;
        
        if self.peek() != &Token::End {
            return Err(GizmoError::ParseError(format!(
                "Expected 'end', found '{}'", self.peek()
            )));
        }
        self.advance(); // consume 'end'
        
        if self.peek() == &Token::Semicolon {
            self.advance();
        }
        self.skip_newlines();
        
        Ok(Statement::WhenStatement { event, body })
    }
    
    fn return_statement(&mut self) -> Result<Statement> {
        self.advance(); // consume 'return'
        
        let value = self.expression()?;
        
        if self.peek() == &Token::Semicolon {
            self.advance();
        }
        self.skip_newlines();
        
        Ok(Statement::ReturnStatement(value))
    }
    
    fn expression_statement(&mut self) -> Result<Statement> {
        let expr = self.expression()?;
        
        if self.peek() == &Token::Semicolon {
            self.advance();
        }
        self.skip_newlines();
        
        Ok(Statement::ExpressionStatement(expr))
    }
    
    fn block_until(&mut self, terminators: &[Token]) -> Result<Vec<Statement>> {
        let mut statements = Vec::new();
        
        while !self.is_at_end() && !terminators.contains(self.peek()) {
            if self.peek() == &Token::Newline {
                self.advance();
                continue;
            }
            
            statements.push(self.statement()?);
        }
        
        Ok(statements)
    }
    
    fn expression(&mut self) -> Result<Expression> {
        self.logical_or()
    }
    
    fn logical_or(&mut self) -> Result<Expression> {
        let mut expr = self.logical_and()?;
        
        while self.peek() == &Token::Or {
            self.advance();
            let right = self.logical_and()?;
            expr = Expression::Binary {
                left: Box::new(expr),
                operator: BinaryOperator::Or,
                right: Box::new(right),
            };
        }
        
        Ok(expr)
    }
    
    fn logical_and(&mut self) -> Result<Expression> {
        let mut expr = self.equality()?;
        
        while self.peek() == &Token::And {
            self.advance();
            let right = self.equality()?;
            expr = Expression::Binary {
                left: Box::new(expr),
                operator: BinaryOperator::And,
                right: Box::new(right),
            };
        }
        
        Ok(expr)
    }
    
    fn equality(&mut self) -> Result<Expression> {
        let mut expr = self.comparison()?;
        
        while matches!(self.peek(), Token::EqualEqual | Token::NotEqual) {
            let operator = match self.advance() {
                Token::EqualEqual => BinaryOperator::Equal,
                Token::NotEqual => BinaryOperator::NotEqual,
                _ => unreachable!(),
            };
            let right = self.comparison()?;
            expr = Expression::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }
        
        Ok(expr)
    }
    
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
            expr = Expression::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }
        
        Ok(expr)
    }
    
    fn term(&mut self) -> Result<Expression> {
        let mut expr = self.factor()?;
        
        while matches!(self.peek(), Token::Plus | Token::Minus) {
            let operator = match self.advance() {
                Token::Plus => BinaryOperator::Add,
                Token::Minus => BinaryOperator::Subtract,
                _ => unreachable!(),
            };
            let right = self.factor()?;
            expr = Expression::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }
        
        Ok(expr)
    }
    
    fn factor(&mut self) -> Result<Expression> {
        let mut expr = self.power()?;
        
        while matches!(self.peek(), Token::Star | Token::Slash | Token::Percent) {
            let operator = match self.advance() {
                Token::Star => BinaryOperator::Multiply,
                Token::Slash => BinaryOperator::Divide,
                Token::Percent => BinaryOperator::Modulo,
                _ => unreachable!(),
            };
            let right = self.power()?;
            expr = Expression::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }
        
        Ok(expr)
    }
    
    fn power(&mut self) -> Result<Expression> {
        let mut expr = self.unary()?;
        
        if self.peek() == &Token::Caret {
            self.advance();
            let right = self.power()?; // Right associative
            expr = Expression::Binary {
                left: Box::new(expr),
                operator: BinaryOperator::Power,
                right: Box::new(right),
            };
        }
        
        Ok(expr)
    }
    
    fn unary(&mut self) -> Result<Expression> {
        match self.peek() {
            Token::Not => {
                self.advance();
                let expr = self.unary()?;
                Ok(Expression::Unary {
                    operator: UnaryOperator::Not,
                    operand: Box::new(expr),
                })
            }
            Token::Minus => {
                self.advance();
                let expr = self.unary()?;
                Ok(Expression::Unary {
                    operator: UnaryOperator::Minus,
                    operand: Box::new(expr),
                })
            }
            _ => self.call(),
        }
    }
    
    fn call(&mut self) -> Result<Expression> {
        let mut expr = self.primary()?;
        
        loop {
            match self.peek() {
                Token::LeftParen => {
                    self.advance();
                    let args = self.argument_list()?;
                    
                    if self.peek() != &Token::RightParen {
                        return Err(GizmoError::ParseError(format!(
                            "Expected ')', found '{}'", self.peek()
                        )));
                    }
                    self.advance();
                    
                    if let Expression::Identifier(name) = expr {
                        expr = Expression::FunctionCall { name, args };
                    } else {
                        return Err(GizmoError::ParseError(
                            "Can only call functions".to_string()
                        ));
                    }
                }
                Token::LeftBracket => {
                    self.advance();
                    let index = self.expression()?;
                    
                    if self.peek() != &Token::RightBracket {
                        return Err(GizmoError::ParseError(format!(
                            "Expected ']', found '{}'", self.peek()
                        )));
                    }
                    self.advance();
                    
                    expr = Expression::Index {
                        object: Box::new(expr),
                        index: Box::new(index),
                    };
                }
                _ => break,
            }
        }
        
        Ok(expr)
    }
    
    fn primary(&mut self) -> Result<Expression> {
        match self.advance().clone() {
            Token::Number(n) => Ok(Expression::Number(n)),
            Token::String(s) => Ok(Expression::String(s)),
            Token::Identifier(name) => Ok(Expression::Identifier(name)),
            Token::LeftParen => {
                let expr = self.expression()?;
                if self.peek() != &Token::RightParen {
                    return Err(GizmoError::ParseError(format!(
                        "Expected ')', found '{}'", self.peek()
                    )));
                }
                self.advance();
                Ok(expr)
            }
            Token::LeftBracket => self.array_literal(),
            Token::Pattern => self.pattern_generator(),
            Token::Animate => self.animated_generator(),
            Token::Evolve => self.cellular_generator(),
            token => Err(GizmoError::ParseError(format!(
                "Unexpected token '{}'", token
            ))),
        }
    }
    
    fn array_literal(&mut self) -> Result<Expression> {
        let mut elements = Vec::new();
        
        if self.peek() != &Token::RightBracket {
            elements.push(self.expression()?);
            
            while self.peek() == &Token::Comma {
                self.advance();
                if self.peek() == &Token::RightBracket {
                    break; // Allow trailing comma
                }
                elements.push(self.expression()?);
            }
        }
        
        if self.peek() != &Token::RightBracket {
            return Err(GizmoError::ParseError(format!(
                "Expected ']', found '{}'", self.peek()
            )));
        }
        self.advance();
        
        Ok(Expression::Array(elements))
    }
    
    fn pattern_generator(&mut self) -> Result<Expression> {
        // pattern(width, height) { ... }
        if self.peek() != &Token::LeftParen {
            return Err(GizmoError::ParseError(format!(
                "Expected '(', found '{}'", self.peek()
            )));
        }
        self.advance();
        
        let width = self.expression()?;
        
        if self.peek() != &Token::Comma {
            return Err(GizmoError::ParseError(format!(
                "Expected ',', found '{}'", self.peek()
            )));
        }
        self.advance();
        
        let height = self.expression()?;
        
        if self.peek() != &Token::RightParen {
            return Err(GizmoError::ParseError(format!(
                "Expected ')', found '{}'", self.peek()
            )));
        }
        self.advance();
        
        if self.peek() != &Token::LeftBrace {
            return Err(GizmoError::ParseError(format!(
                "Expected '{{', found '{}'", self.peek()
            )));
        }
        self.advance();
        self.skip_newlines();
        
        let body = self.block_until(&[Token::RightBrace])?;
        
        if self.peek() != &Token::RightBrace {
            return Err(GizmoError::ParseError(format!(
                "Expected '}}', found '{}'", self.peek()
            )));
        }
        self.advance();
        
        Ok(Expression::PatternGenerator {
            width: Box::new(width),
            height: Box::new(height),
            body,
        })
    }
    
    fn animated_generator(&mut self) -> Result<Expression> {
        // animate(width, height) using time_var { ... }
        if self.peek() != &Token::LeftParen {
            return Err(GizmoError::ParseError(format!(
                "Expected '(', found '{}'", self.peek()
            )));
        }
        self.advance();
        
        let width = self.expression()?;
        
        if self.peek() != &Token::Comma {
            return Err(GizmoError::ParseError(format!(
                "Expected ',', found '{}'", self.peek()
            )));
        }
        self.advance();
        
        let height = self.expression()?;
        
        if self.peek() != &Token::RightParen {
            return Err(GizmoError::ParseError(format!(
                "Expected ')', found '{}'", self.peek()
            )));
        }
        self.advance();
        
        if self.peek() != &Token::Using {
            return Err(GizmoError::ParseError(format!(
                "Expected 'using', found '{}'", self.peek()
            )));
        }
        self.advance();
        
        let time_var = match self.advance() {
            Token::Identifier(name) => name.clone(),
            token => {
                return Err(GizmoError::ParseError(format!(
                    "Expected identifier, found '{}'", token
                )));
            }
        };
        
        if self.peek() != &Token::LeftBrace {
            return Err(GizmoError::ParseError(format!(
                "Expected '{{', found '{}'", self.peek()
            )));
        }
        self.advance();
        self.skip_newlines();
        
        let body = self.block_until(&[Token::RightBrace])?;
        
        if self.peek() != &Token::RightBrace {
            return Err(GizmoError::ParseError(format!(
                "Expected '}}', found '{}'", self.peek()
            )));
        }
        self.advance();
        
        Ok(Expression::AnimatedGenerator {
            width: Box::new(width),
            height: Box::new(height),
            time_var,
            body,
        })
    }
    
    fn cellular_generator(&mut self) -> Result<Expression> {
        // evolve(width, height) from prev_var { ... }
        if self.peek() != &Token::LeftParen {
            return Err(GizmoError::ParseError(format!(
                "Expected '(', found '{}'", self.peek()
            )));
        }
        self.advance();
        
        let width = self.expression()?;
        
        if self.peek() != &Token::Comma {
            return Err(GizmoError::ParseError(format!(
                "Expected ',', found '{}'", self.peek()
            )));
        }
        self.advance();
        
        let height = self.expression()?;
        
        if self.peek() != &Token::RightParen {
            return Err(GizmoError::ParseError(format!(
                "Expected ')', found '{}'", self.peek()
            )));
        }
        self.advance();
        
        if self.peek() != &Token::From {
            return Err(GizmoError::ParseError(format!(
                "Expected 'from', found '{}'", self.peek()
            )));
        }
        self.advance();
        
        let prev_var = match self.advance() {
            Token::Identifier(name) => name.clone(),
            token => {
                return Err(GizmoError::ParseError(format!(
                    "Expected identifier, found '{}'", token
                )));
            }
        };
        
        if self.peek() != &Token::LeftBrace {
            return Err(GizmoError::ParseError(format!(
                "Expected '{{', found '{}'", self.peek()
            )));
        }
        self.advance();
        self.skip_newlines();
        
        let body = self.block_until(&[Token::RightBrace])?;
        
        if self.peek() != &Token::RightBrace {
            return Err(GizmoError::ParseError(format!(
                "Expected '}}', found '{}'", self.peek()
            )));
        }
        self.advance();
        
        Ok(Expression::CellularGenerator {
            width: Box::new(width),
            height: Box::new(height),
            prev_var,
            body,
        })
    }
    
    fn argument_list(&mut self) -> Result<Vec<Expression>> {
        let mut args = Vec::new();
        
        if self.peek() != &Token::RightParen {
            args.push(self.expression()?);
            
            while self.peek() == &Token::Comma {
                self.advance();
                if self.peek() == &Token::RightParen {
                    break; // Allow trailing comma
                }
                args.push(self.expression()?);
            }
        }
        
        Ok(args)
    }
    
    fn skip_newlines(&mut self) {
        while self.peek() == &Token::Newline {
            self.advance();
        }
    }
    
    fn is_at_end(&self) -> bool {
        self.current >= self.tokens.len() || self.peek() == &Token::Eof
    }
    
    fn peek(&self) -> &Token {
        if self.current >= self.tokens.len() {
            &Token::Eof
        } else {
            &self.tokens[self.current]
        }
    }
    
    fn advance(&mut self) -> &Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.previous()
    }
    
    fn previous(&self) -> &Token {
        if self.current == 0 {
            &Token::Eof
        } else {
            &self.tokens[self.current - 1]
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Lexer;
    
    #[test]
    fn test_variable_declaration() {
        let mut lexer = Lexer::new("frame eye = [0, 1];");
        let tokens = lexer.tokenize().unwrap();
        let mut parser = Parser::new(tokens);
        let program = parser.parse().unwrap();
        
        assert_eq!(program.statements.len(), 1);
        match &program.statements[0] {
            Statement::VariableDeclaration { var_type, name, .. } => {
                assert_eq!(*var_type, VariableType::Frame);
                assert_eq!(name, "eye");
            }
            _ => panic!("Expected variable declaration"),
        }
    }
    
    #[test]
    fn test_function_call() {
        let mut lexer = Lexer::new("play([eye_open, eye_closed]);");
        let tokens = lexer.tokenize().unwrap();
        let mut parser = Parser::new(tokens);
        let program = parser.parse().unwrap();
        
        assert_eq!(program.statements.len(), 1);
        match &program.statements[0] {
            Statement::ExpressionStatement(Expression::FunctionCall { name, args }) => {
                assert_eq!(name, "play");
                assert_eq!(args.len(), 1);
            }
            _ => panic!("Expected function call"),
        }
    }
}