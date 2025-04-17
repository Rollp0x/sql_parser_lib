use crate::ast::SQLStatement;
use crate::token::{Token,self};
use std::error::Error;
use std::fmt;

pub mod expr;
pub mod common;
pub mod select;
pub mod delete;
pub mod insert;

// 解析错误
#[derive(Debug)]
pub struct ParseError {
    pub message: String,
    pub token_position: usize,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Parse error at position {}: {}",
            self.token_position, self.message
        )
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

// 添加基本功能
impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser { tokens, current: 0 }
    }
    pub fn new_from_sql(sql: &str) -> Self {
        let tokens = token::tokenize(sql);
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
    pub fn consume_token(&mut self) -> Option<Token> {
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

    // 尝试匹配一个标点符号
    pub fn match_punctuator(&mut self, punctuator: char) -> bool {
        if let Some(Token::Punctuator(p)) = self.peek() {
            if *p == punctuator {
                self.consume_token(); // 消费匹配的token
                return true;
            }
        }
        false
    }

    pub fn is_punctuator(&self, punctuator: char) -> bool {
        if let Some(Token::Punctuator(p)) = self.peek() {
            return *p == punctuator;
        }
        false
    }

    // 尝试匹配一个关键字
    pub fn match_keyword(&mut self, keyword: &str) -> bool {
        if let Some(Token::Keyword(k)) = self.peek() {
            if k.to_uppercase() == keyword.to_uppercase() {
                self.consume_token(); // 消费匹配的token
                return true;
            }
        }
        false
    }

    pub fn is_keyword(&self, keyword: &str) -> bool {
        if let Some(Token::Keyword(k)) = self.peek() {
            return k.to_uppercase() == keyword.to_uppercase();
        }
        false
    }

    // 尝试匹配一个操作符
    pub fn match_operator(&mut self, operator: &str) -> bool {
        if let Some(Token::Operator(op)) = self.peek() {
            if op == operator {
                self.consume_token(); // 消费匹配的token
                return true;
            }
        }
        false
    }
    pub fn is_operator(&self, operator: &str) -> bool {
        if let Some(Token::Operator(op)) = self.peek() {
            return op == operator;
        }
        false
    }

    // 将token格式化为更可读的形式
    pub fn format_token(&self, token: &Token) -> String {
        match token {
            Token::Keyword(k) => k.clone(),
            Token::Identifier(id) => id.clone(),
            Token::StringLiteral(s) => format!("'{}'", s),
            Token::NumericLiteral(n) => n.to_string(),
            Token::Punctuator(c) => c.to_string(),
            _ => {
                // 其他token类型...
                format!("{:?}", token)
            }
        }
    }

    // 辅助方法：生成错误上下文
    pub fn get_error_context(&self) -> String {
        // 获取当前位置前后的几个token
        let start = if self.current > 3 {
            self.current - 3
        } else {
            0
        };
        let end = std::cmp::min(self.current + 2, self.tokens.len());

        // 将tokens转换为可读字符串
        let context_tokens: Vec<String> = self.tokens[start..end]
            .iter()
            .enumerate()
            .map(|(i, t)| {
                let pos = start + i;
                let marker = if pos == self.current { "👉 " } else { "" };
                format!("{}{}", marker, self.format_token(t))
            })
            .collect();

        format!("\"{}\"", context_tokens.join(" "))
    }

    pub fn get_parse_error(&self, message: &str) -> ParseError {
        let context = self.get_error_context();
        ParseError {
            message: format!("{}. Near: {}", message, context),
            token_position: self.current,
        }
    }
}
