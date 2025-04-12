use crate::token::Token;
use crate::ast::SQLStatement;
use std::error::Error;
use std::fmt;

mod select;


// 解析错误
#[derive(Debug)]
pub struct ParseError {
    pub message: String,
    pub token_position: usize,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Parse error at position {}: {}", self.token_position, self.message)
    }
}

impl Error for ParseError {}

// 核心解析器结构
pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

// 语句解析接口
pub trait StatementParser {
    // 将解析的Token流转换为对应的语句类型
    fn parse(&mut self) -> Result<SQLStatement, ParseError>;
}



impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser { tokens, current: 0 }
    }
    
    // ===== 迭代器风格方法 =====
    
    // 返回当前token但不消费它
    pub fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.current)
    }
    
    // 返回当前token之后的第n个token
    pub fn peek_n(&self, n: usize) -> Option<&Token> {
        self.tokens.get(self.current + n)
    }
    
    // 消费当前token并返回它
    pub fn next(&mut self) -> Option<Token> {
        if self.current < self.tokens.len() {
            let token = self.tokens[self.current].clone();
            self.current += 1;
            Some(token)
        } else {
            None
        }
    }
    
    // 检查序列中是否还有更多token
    pub fn has_more(&self) -> bool {
        self.current < self.tokens.len()
    }
    
    // 消费n个token
    pub fn skip(&mut self, n: usize) {
        self.current = std::cmp::min(self.current + n, self.tokens.len());
    }
    
    // 回退一个token
    pub fn back(&mut self) {
        if self.current > 0 {
            self.current -= 1;
        }
    }
    
    // ===== 解析器特定方法 =====
    
    // 检查当前token是否是指定关键字，如果是则消费它
    pub fn match_keyword(&mut self, keyword: &str) -> bool {
        if let Some(Token::Keyword(k)) = self.peek() {
            if k.to_uppercase() == keyword.to_uppercase() {
                self.next();
                return true;
            }
        }
        false
    }
    
    // 期望当前token是指定关键字，否则返回错误
    pub fn expect_keyword(&mut self, keyword: &str) -> Result<(), ParseError> {
        if self.match_keyword(keyword) {
            Ok(())
        } else {
            Err(ParseError {
                message: format!("Expected keyword '{}', found {:?}", keyword, self.peek()),
                token_position: self.current,
            })
        }
    }
    
    // 尝试匹配一个标识符
    pub fn match_identifier(&mut self) -> Option<String> {
        if let Some(Token::Identifier(ident)) = self.peek() {
            let ident = ident.clone();
            self.next();
            Some(ident)
        } else {
            None
        }
    }
    
    // 期望一个标识符
    pub fn expect_identifier(&mut self) -> Result<String, ParseError> {
        if let Some(ident) = self.match_identifier() {
            Ok(ident)
        } else {
            Err(ParseError {
                message: format!("Expected identifier, found {:?}", self.peek()),
                token_position: self.current,
            })
        }
    }
    
    // 尝试匹配一个标点符号
    pub fn match_punctuator(&mut self, punctuator: char) -> bool {
        if let Some(Token::Punctuator(p)) = self.peek() {
            if *p == punctuator {
                self.next();
                return true;
            }
        }
        false
    }
    
    // 期望一个标点符号
    pub fn expect_punctuator(&mut self, punctuator: char) -> Result<(), ParseError> {
        if self.match_punctuator(punctuator) {
            Ok(())
        } else {
            Err(ParseError {
                message: format!("Expected '{}', found {:?}", punctuator, self.peek()),
                token_position: self.current,
            })
        }
    }
}