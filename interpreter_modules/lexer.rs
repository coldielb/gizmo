use crate::error::GizmoError;
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // Literals
    Number(f64),
    String(String),
    Identifier(String),
    
    // Keywords
    Frame,
    Frames,
    Num,
    Text,
    Pattern,
    Animate,
    Evolve,
    Using,
    From,
    If,
    Then,
    Elsif,
    Else,
    End,
    Repeat,
    Do,
    Times,
    When,
    Clicked,
    Idle,
    Play,
    Loop,
    Stop,
    Return,
    And,
    Or,
    Not,
    
    // Operators
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    Caret,
    Equal,
    EqualEqual,
    NotEqual,
    Less,
    Greater,
    LessEqual,
    GreaterEqual,
    
    // Delimiters
    LeftParen,
    RightParen,
    LeftBracket,
    RightBracket,
    LeftBrace,
    RightBrace,
    Comma,
    Semicolon,
    Colon,
    
    // Special
    Newline,
    Eof,
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Token::Number(n) => write!(f, "{}", n),
            Token::String(s) => write!(f, "\"{}\"", s),
            Token::Identifier(s) => write!(f, "{}", s),
            Token::Frame => write!(f, "frame"),
            Token::Frames => write!(f, "frames"),
            Token::Num => write!(f, "num"),
            Token::Text => write!(f, "text"),
            Token::Pattern => write!(f, "pattern"),
            Token::Animate => write!(f, "animate"),
            Token::Evolve => write!(f, "evolve"),
            Token::Using => write!(f, "using"),
            Token::From => write!(f, "from"),
            Token::If => write!(f, "if"),
            Token::Then => write!(f, "then"),
            Token::Elsif => write!(f, "elsif"),
            Token::Else => write!(f, "else"),
            Token::End => write!(f, "end"),
            Token::Repeat => write!(f, "repeat"),
            Token::Do => write!(f, "do"),
            Token::Times => write!(f, "times"),
            Token::When => write!(f, "when"),
            Token::Clicked => write!(f, "clicked"),
            Token::Idle => write!(f, "idle"),
            Token::Play => write!(f, "play"),
            Token::Loop => write!(f, "loop"),
            Token::Stop => write!(f, "stop"),
            Token::Return => write!(f, "return"),
            Token::And => write!(f, "and"),
            Token::Or => write!(f, "or"),
            Token::Not => write!(f, "not"),
            Token::Plus => write!(f, "+"),
            Token::Minus => write!(f, "-"),
            Token::Star => write!(f, "*"),
            Token::Slash => write!(f, "/"),
            Token::Percent => write!(f, "%"),
            Token::Caret => write!(f, "^"),
            Token::Equal => write!(f, "="),
            Token::EqualEqual => write!(f, "=="),
            Token::NotEqual => write!(f, "!="),
            Token::Less => write!(f, "<"),
            Token::Greater => write!(f, ">"),
            Token::LessEqual => write!(f, "<="),
            Token::GreaterEqual => write!(f, ">="),
            Token::LeftParen => write!(f, "("),
            Token::RightParen => write!(f, ")"),
            Token::LeftBracket => write!(f, "["),
            Token::RightBracket => write!(f, "]"),
            Token::LeftBrace => write!(f, "{{"),
            Token::RightBrace => write!(f, "}}"),
            Token::Comma => write!(f, ","),
            Token::Semicolon => write!(f, ";"),
            Token::Colon => write!(f, ":"),
            Token::Newline => write!(f, "\\n"),
            Token::Eof => write!(f, "EOF"),
        }
    }
}

pub struct Lexer {
    input: Vec<char>,
    position: usize,
    line: usize,
    column: usize,
}

impl Lexer {
    pub fn new(input: &str) -> Self {
        Self {
            input: input.chars().collect(),
            position: 0,
            line: 1,
            column: 1,
        }
    }
    
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
            ':' => Ok(Token::Colon),
            '+' => Ok(Token::Plus),
            '-' => Ok(Token::Minus),
            '*' => Ok(Token::Star),
            '/' => {
                if self.peek() == '/' {
                    // Single-line comment
                    while self.peek() != '\n' && !self.is_at_end() {
                        self.advance();
                    }
                    self.next_token()
                } else {
                    Ok(Token::Slash)
                }
            }
            '%' => Ok(Token::Percent),
            '^' => Ok(Token::Caret),
            '=' => {
                if self.peek() == '=' {
                    self.advance();
                    Ok(Token::EqualEqual)
                } else {
                    Ok(Token::Equal)
                }
            }
            '!' => {
                if self.peek() == '=' {
                    self.advance();
                    Ok(Token::NotEqual)
                } else {
                    Err(GizmoError::LexError(format!(
                        "Unexpected character '!' at line {}, column {}",
                        self.line, self.column
                    )))
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
            '>' => {
                if self.peek() == '=' {
                    self.advance();
                    Ok(Token::GreaterEqual)
                } else {
                    Ok(Token::Greater)
                }
            }
            '\"' => self.string_literal(),
            c if c.is_ascii_digit() => self.number_literal(c),
            c if c.is_ascii_alphabetic() || c == '_' => self.identifier_or_keyword(c),
            _ => Err(GizmoError::LexError(format!(
                "Unexpected character '{}' at line {}, column {}",
                c, self.line, self.column
            ))),
        }
    }
    
    fn string_literal(&mut self) -> Result<Token, GizmoError> {
        let mut value = String::new();
        
        while self.peek() != '\"' && !self.is_at_end() {
            if self.peek() == '\n' {
                self.line += 1;
                self.column = 1;
            }
            let c = self.advance();
            if c == '\\' {
                // Handle escape sequences
                match self.advance() {
                    'n' => value.push('\n'),
                    't' => value.push('\t'),
                    'r' => value.push('\r'),
                    '\\' => value.push('\\'),
                    '\"' => value.push('\"'),
                    c => {
                        return Err(GizmoError::LexError(format!(
                            "Invalid escape sequence '\\{}' at line {}, column {}",
                            c, self.line, self.column
                        )));
                    }
                }
            } else {
                value.push(c);
            }
        }
        
        if self.is_at_end() {
            return Err(GizmoError::LexError(format!(
                "Unterminated string at line {}, column {}",
                self.line, self.column
            )));
        }
        
        // Consume the closing quote
        self.advance();
        Ok(Token::String(value))
    }
    
    fn number_literal(&mut self, first_digit: char) -> Result<Token, GizmoError> {
        let mut value = String::from(first_digit);
        
        while self.peek().is_ascii_digit() {
            value.push(self.advance());
        }
        
        // Check for decimal point
        if self.peek() == '.' && self.peek_next().is_ascii_digit() {
            value.push(self.advance()); // consume '.'
            while self.peek().is_ascii_digit() {
                value.push(self.advance());
            }
        }
        
        match value.parse::<f64>() {
            Ok(num) => Ok(Token::Number(num)),
            Err(_) => Err(GizmoError::LexError(format!(
                "Invalid number '{}' at line {}, column {}",
                value, self.line, self.column
            ))),
        }
    }
    
    fn identifier_or_keyword(&mut self, first_char: char) -> Result<Token, GizmoError> {
        let mut value = String::from(first_char);
        
        while self.peek().is_ascii_alphanumeric() || self.peek() == '_' {
            value.push(self.advance());
        }
        
        let token = match value.as_str() {
            "frame" => Token::Frame,
            "frames" => Token::Frames,
            "num" => Token::Num,
            "text" => Token::Text,
            "pattern" => Token::Pattern,
            "animate" => Token::Animate,
            "evolve" => Token::Evolve,
            "using" => Token::Using,
            "from" => Token::From,
            "if" => Token::If,
            "then" => Token::Then,
            "elsif" => Token::Elsif,
            "else" => Token::Else,
            "end" => Token::End,
            "repeat" => Token::Repeat,
            "do" => Token::Do,
            "times" => Token::Times,
            "when" => Token::When,
            "clicked" => Token::Clicked,
            "idle" => Token::Idle,
            "play" => Token::Play,
            "loop" => Token::Loop,
            "stop" => Token::Stop,
            "return" => Token::Return,
            "and" => Token::And,
            "or" => Token::Or,
            "not" => Token::Not,
            _ => Token::Identifier(value),
        };
        
        Ok(token)
    }
    
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
    
    fn is_at_end(&self) -> bool {
        self.position >= self.input.len()
    }
    
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
    
    fn peek(&self) -> char {
        if self.is_at_end() {
            '\0'
        } else {
            self.input[self.position]
        }
    }
    
    fn peek_next(&self) -> char {
        if self.position + 1 >= self.input.len() {
            '\0'
        } else {
            self.input[self.position + 1]
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_basic_tokens() {
        let mut lexer = Lexer::new("frame eye = [0, 1];");
        let tokens = lexer.tokenize().unwrap();
        
        assert_eq!(tokens[0], Token::Frame);
        assert_eq!(tokens[1], Token::Identifier("eye".to_string()));
        assert_eq!(tokens[2], Token::Equal);
        assert_eq!(tokens[3], Token::LeftBracket);
        assert_eq!(tokens[4], Token::Number(0.0));
        assert_eq!(tokens[5], Token::Comma);
        assert_eq!(tokens[6], Token::Number(1.0));
        assert_eq!(tokens[7], Token::RightBracket);
        assert_eq!(tokens[8], Token::Semicolon);
        assert_eq!(tokens[9], Token::Eof);
    }
    
    #[test]
    fn test_string_literal() {
        let mut lexer = Lexer::new("text name = \"hello world\";");
        let tokens = lexer.tokenize().unwrap();
        
        assert_eq!(tokens[0], Token::Text);
        assert_eq!(tokens[1], Token::Identifier("name".to_string()));
        assert_eq!(tokens[2], Token::Equal);
        assert_eq!(tokens[3], Token::String("hello world".to_string()));
        assert_eq!(tokens[4], Token::Semicolon);
    }
    
    #[test]
    fn test_operators() {
        let mut lexer = Lexer::new("== != <= >= + - * / % ^");
        let tokens = lexer.tokenize().unwrap();
        
        let expected = vec![
            Token::EqualEqual,
            Token::NotEqual,
            Token::LessEqual,
            Token::GreaterEqual,
            Token::Plus,
            Token::Minus,
            Token::Star,
            Token::Slash,
            Token::Percent,
            Token::Caret,
            Token::Eof,
        ];
        
        assert_eq!(tokens, expected);
    }
    
    #[test]
    fn test_comments() {
        let mut lexer = Lexer::new("frame // this is a comment\\ntest");
        let tokens = lexer.tokenize().unwrap();
        
        assert_eq!(tokens[0], Token::Frame);
        assert_eq!(tokens[1], Token::Newline);
        assert_eq!(tokens[2], Token::Identifier("test".to_string()));
    }
}