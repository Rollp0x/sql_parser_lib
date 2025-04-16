use super::{ParseError, Parser};
use crate::ast::{
    expr::{Expr, LimitClause, OrderByExpr},
    common::TableReference,
};
use crate::token::Token;
// 实现公共解析功能
impl Parser {
    pub fn move_current_idx(
        &self, 
        current_idx: u8, 
        clause_idx: u8,
        get_clause_name: fn(u8) -> &'static str,
    ) -> Result<u8, ParseError> {
        if clause_idx > current_idx {
            Ok(clause_idx)
        } else {
            // 获取错误上下文信息
            let context = self.get_error_context();

            Err(ParseError {
                message: format!(
                    "{} clause out of order, expected after {}. Near: {}",
                    get_clause_name(clause_idx),
                    get_clause_name(current_idx),
                    context
                ),
                token_position: self.current,
            })
        }
    }
    // 解析表名
    pub fn parse_table_reference(&mut self,allow_as_keyword:bool) -> Result<TableReference, ParseError> {
        // 获取表名
        let name = match self.peek() {
            Some(Token::Identifier(ident)) => {
                let name = ident.to_owned();
                self.next();
                name
            }
            _ => {
                return Err(
                    self.get_parse_error(&format!("Expected table name, found {:?}", self.peek()))
                );
            }
        };

        // 检查是否有别名
        let alias = if allow_as_keyword && self.match_keyword("AS") {
            if let Some(Token::Identifier(ident)) = self.peek() {
                let alias_name = ident.to_owned();
                self.next();
                Some(alias_name)
            } else {
                return Err(self.get_parse_error(&format!(
                    "Expected alias after AS, found {:?}",
                    self.peek()
                )));
            }
        } else if let Some(Token::Identifier(ident)) = self.peek() {
            let alias = ident.clone();
            self.next();
            Some(alias)
        } else {
            None
        };

        Ok(TableReference { name, alias })
    }

    pub fn parse_order_by(&mut self) -> Result<Vec<OrderByExpr>, ParseError> {
        let mut order_by = Vec::new();
        // 解析列列表
        loop {
            // 解析单个列
            let expr = self.parse_expr(0)?;
            let order = if self.match_keyword("DESC") {
                OrderByExpr {
                    expr: expr.clone(),
                    asc: false,
                }
            } else if self.match_keyword("ASC") {
                OrderByExpr {
                    expr: expr.clone(),
                    asc: true,
                }
            } else {
                OrderByExpr {
                    expr: expr.clone(),
                    asc: true,
                }
            };
            order_by.push(order);
            // 如果后面是逗号，继续解析下一个列
            if !self.match_punctuator(',') {
                break;
            }
        }
        Ok(order_by)
    }

    pub fn parse_limit(&mut self) -> Result<LimitClause, ParseError> {
        // 解析LIMIT值
        let limit = if let Some(Token::NumericLiteral(value)) = self.peek() {
            let limit_value = value.parse::<u64>().map_err(|_| {
                self.get_parse_error(&format!(
                    "Invalid number after LIMIT, found {:?}",
                    self.peek()
                ))
            })?;
            self.next(); // 消费LIMIT值
            limit_value
        } else {
            return Err(self.get_parse_error(&format!(
                "Expected integer after LIMIT, found {:?}",
                self.peek()
            )));
        };
        // 检查是否有OFFSET
        let offset = if self.match_keyword("OFFSET") {
            if let Some(Token::NumericLiteral(value)) = self.peek() {
                let offset_value = value.parse::<u64>().map_err(|_| {
                    self.get_parse_error(&format!(
                        "Invalid number after OFFSET, found {:?}",
                        self.peek()
                    ))
                })?;
                self.next(); // 消费OFFSET值
                Some(offset_value)
            } else {
                return Err(self.get_parse_error(&format!(
                    "Expected integer after OFFSET, found {:?}",
                    self.peek()
                )));
            }
        } else {
            None
        };

        Ok(LimitClause { limit, offset })
    }
}