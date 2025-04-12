use super::{ParseError, Parser};
use crate::ast::{
    select::{SelectStatement,TableReference, SelectColumn},
    expr::{Expr, OrderByExpr, LimitClause},
};
use crate::token::Token;

// SELECT语句解析器接口
pub trait SelectStatementParser {
    type Error;
    // 解析SELECT语句
    fn parse_select_statement(&mut self) -> Result<SelectStatement, Self::Error>;
}


// 子句优先级/索引
const WHERE_IDX: u8 = 1;
const GROUP_BY_IDX: u8 = 2;
const HAVING_IDX: u8 = 3;
const ORDER_BY_IDX: u8 = 4;
const LIMIT_IDX: u8 = 5;
const FROM_IDX: u8 = 0;


// 实现其它解析功能
impl Parser {
    // 解析表名
    fn parse_table_reference(&mut self) -> Result<TableReference, ParseError> {
        // 获取表名
        let name = match self.peek() {
            Some(Token::Identifier(ident)) => {
                let name = ident.to_owned();
                self.next();
                name
            },
            _ => {
                return Err(self.get_parse_error(&format!("Expected table name, found {:?}", self.peek())));
            }
        };
        
        // 检查是否有别名
        let alias = if self.match_keyword("AS") {
            if let Some(Token::Identifier(ident)) = self.peek() {
                let alias_name = ident.to_owned();
                self.next();
                Some(alias_name)
            } else {
                return Err(self.get_parse_error(&format!("Expected alias after AS, found {:?}", self.peek())));
            }
        } else {
            None
        };
        
        Ok(TableReference { name, alias })
    }

    // 解析单个选择列
    fn parse_select_column(&mut self) -> Result<SelectColumn, ParseError> {
        // 获取列名
        let name = match self.peek() {
            Some(Token::Identifier(ident)) => {
                let column_name = ident.to_owned();
                self.next();
                column_name
            },
            _ => {
                return Err(self.get_parse_error(&format!("Expected column name, found {:?}", self.peek())));
            }
        };
        
        // 检查是否有AS别名
        let alias = if self.match_keyword("AS") {
            if let Some(Token::Identifier(ident)) = self.peek() {
                let alias_name = ident.to_owned();
                self.next();
                Some(alias_name)
            } else {
                return Err(self.get_parse_error(&format!("Expected alias after AS, found {:?}", self.peek())));
            }
        } else {
            None
        };
        
        Ok(SelectColumn::Column { name, alias })
    }


    fn parse_select_columns(&mut self) -> Result<Vec<SelectColumn>, ParseError> {
        let mut columns = Vec::new();
        if self.match_keyword("*") {
            columns.push(SelectColumn::Wildcard);
            return Ok(columns);
        } 

        // 解析列列表
        loop {
            // 解析单个列
            columns.push(self.parse_select_column()?);
            
            // 如果后面是逗号，继续解析下一个列
            if !self.match_punctuator(',') {
                break;
            }
        }

        Ok(columns)
    }

    fn parse_expr(&mut self) -> Result<Expr, ParseError> {
        todo!()
        // // 这里可以实现更复杂的表达式解析
        // // todo 目前仅返回一个简单的标识符
        // if let Some(Token::Identifier(ident)) = self.peek() {
        //     let expr = Expr::Identifier(ident.to_owned());
        //     self.next();
        //     Ok(expr)
        // } else {
        //     Err(ParseError {
        //         message: format!("Expected expression, found {:?}", self.peek()),
        //         token_position: self.current,
        //     })
        // }
    }

    fn parse_group_exr(&mut self)  -> Result<Vec<Expr>,ParseError> {
        todo!()
    }

    fn parse_order_by(&mut self) -> Result<Vec<OrderByExpr>, ParseError> {
        todo!()
    }
    fn parse_limit(&mut self) -> Result<LimitClause, ParseError> {
        todo!()
    }

    fn move_current_idx(&self, current_idx: u8, clause_idx: u8) -> Result<u8, ParseError> {
        if clause_idx > current_idx {
            Ok(clause_idx)
        } else {
            // 获取错误上下文信息
            let context = self.get_error_context();
            
            Err(ParseError {
                message: format!("{} clause out of order, expected after {}. Near: {}", 
                    get_clause_name(clause_idx), 
                    get_clause_name(current_idx),
                    context),
                token_position: self.current,
            })
        }
    }

    // 辅助方法：生成错误上下文
    fn get_error_context(&self) -> String {
        // 获取当前位置前后的几个token
        let start = if self.current > 3 { self.current - 3 } else { 0 };
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
    
    // 将token格式化为更可读的形式
    fn format_token(&self, token: &Token) -> String {
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

    fn get_parse_error(&self, message: &str) -> ParseError {
        let context = self.get_error_context();
        ParseError {
            message: format!("{}. Near: {}", message, context),
            token_position: self.current,
        }
    }
}

impl SelectStatementParser for Parser {
    type Error = ParseError;
    fn parse_select_statement(&mut self) -> Result<SelectStatement, Self::Error> {
        // 期望以SELECT关键字开始
        if !self.match_keyword("SELECT") {
            return Err(self.get_parse_error(&format!("Expected SELECT, found{:?}", self.peek())));
        }
        // 解析列
        let columns = self.parse_select_columns()?;
        // 必须有FROM子句
        if !self.match_keyword("FROM") {
            return Err(self.get_parse_error(&format!("Expected FROM, found {:?}", self.peek())));
        }
        // 解析FROM的表引用
        let from = self.parse_table_reference()?;
        // 跟踪当前已处理的最高子句索引
        let mut current_idx: u8 = FROM_IDX;
        // 可选的WHERE子句
        let where_clause = if self.match_keyword("WHERE") {
            current_idx = self.move_current_idx(current_idx, WHERE_IDX)?;
            Some(self.parse_expr()?)
        } else {
            None
        };
        // 可选的GROUP BY子句
        let group_by = if self.match_keyword("GROUP") {
            if !self.match_keyword("BY") {
                return Err(self.get_parse_error(&format!("Expected BY after GROUP, found {:?}", self.peek())));
            }
            current_idx = self.move_current_idx(current_idx, GROUP_BY_IDX)?;
            Some(self.parse_group_exr()?)
        } else {
            None
        };
        // 可选的HAVING子句
        let having = if self.match_keyword("HAVING") {
            current_idx = self.move_current_idx(current_idx, HAVING_IDX)?;
            Some(self.parse_expr()?)
        } else {
            None
        };
        // 可选的ORDER BY子句
        let order_by = if self.match_keyword("ORDER")  {
            if !self.match_keyword("BY") {
                return Err(self.get_parse_error(&format!("Expected BY after ORDER, found {:?}", self.peek())));
            }
            current_idx = self.move_current_idx(current_idx, ORDER_BY_IDX)?;
            Some(self.parse_order_by()?)
        } else {
            None
        };
        // 可选的LIMIT子句
        let limit = if self.match_keyword("LIMIT") {
            // Since this is the last clause, we don't need to store the updated index
            self.move_current_idx(current_idx, LIMIT_IDX)?;
            Some(self.parse_limit()?)
        } else {
            None
        };

        Ok(SelectStatement {
            columns,
            from,
            where_clause,
            group_by,
            having,
            order_by,
            limit,
        })
    }
}



// 可选的辅助函数，将索引转换为子句名称
fn get_clause_name(idx: u8) -> &'static str {
    match idx {
        WHERE_IDX => "WHERE",
        GROUP_BY_IDX => "GROUP BY",
        HAVING_IDX => "HAVING",
        ORDER_BY_IDX => "ORDER BY",
        LIMIT_IDX => "LIMIT",
        _ => "FROM",
    }
}