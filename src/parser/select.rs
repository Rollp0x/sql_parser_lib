use super::{ParseError, Parser};
use crate::ast::{
    expr::Expr,
    select::{SelectColumn, SelectStatement},
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
    // 解析单个选择列
    fn parse_select_column(&mut self) -> Result<SelectColumn, ParseError> {
        // 获取列名
        let name = match self.peek() {
            Some(Token::Identifier(ident)) => {
                let column_name = ident.to_owned();
                self.next();
                column_name
            }
            _ => {
                return Err(
                    self.get_parse_error(&format!("Expected column name, found {:?}", self.peek()))
                );
            }
        };
        // 检查是否有AS别名
        let alias = if self.match_keyword("AS") {
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
        } else {
            None
        };

        Ok(SelectColumn::Column { name, alias })
    }

    fn parse_select_columns(&mut self) -> Result<(Vec<SelectColumn>, bool), ParseError> {
        let mut columns = Vec::new();
        // 更清晰的写法
        let distinct = if self.match_keyword("DISTINCT") {
            true
        } else if self.match_keyword("ALL") {
            false
        } else {
            false // 默认为非DISTINCT
        };
        // 判断是否为*
        if self.match_operator("*") {
            columns.push(SelectColumn::Wildcard);
            return Ok((columns, distinct));
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

        Ok((columns, distinct))
    }

    fn parse_group_exr(&mut self) -> Result<Vec<Expr>, ParseError> {
        let mut group_by = Vec::new();
        // 解析列列表
        loop {
            // 解析单个列
            group_by.push(self.parse_expr(0)?);

            // 如果后面是逗号，继续解析下一个列
            if !self.match_punctuator(',') {
                break;
            }
        }
        Ok(group_by)
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
        let (columns, distinct) = self.parse_select_columns()?;
        // 必须有FROM子句
        if !self.match_keyword("FROM") {
            return Err(self.get_parse_error(&format!("Expected FROM, found {:?}", self.peek())));
        }
        // 解析FROM的表引用
        let from = self.parse_table_reference(true)?;
        // 跟踪当前已处理的最高子句索引
        let mut current_idx: u8 = FROM_IDX;
        // 可选的WHERE子句
        let where_clause = if self.match_keyword("WHERE") {
            current_idx = self.move_current_idx(current_idx, WHERE_IDX,get_clause_name)?;
            Some(self.parse_expr(0)?)
        } else {
            None
        };
        // 可选的GROUP BY子句
        let group_by = if self.match_keyword("GROUP") {
            if !self.match_keyword("BY") {
                return Err(self.get_parse_error(&format!(
                    "Expected BY after GROUP, found {:?}",
                    self.peek()
                )));
            }
            current_idx = self.move_current_idx(current_idx, GROUP_BY_IDX,get_clause_name)?;
            Some(self.parse_group_exr()?)
        } else {
            None
        };
        // 可选的HAVING子句
        let having = if self.match_keyword("HAVING") {
            current_idx = self.move_current_idx(current_idx, HAVING_IDX,get_clause_name)?;
            Some(self.parse_expr(0)?)
        } else {
            None
        };
        // 可选的ORDER BY子句
        let order_by = if self.match_keyword("ORDER") {
            if !self.match_keyword("BY") {
                return Err(self.get_parse_error(&format!(
                    "Expected BY after ORDER, found {:?}",
                    self.peek()
                )));
            }
            current_idx = self.move_current_idx(current_idx, ORDER_BY_IDX,get_clause_name)?;
            Some(self.parse_order_by()?)
        } else {
            None
        };
        // 可选的LIMIT子句
        let limit = if self.match_keyword("LIMIT") {
            // Since this is the last clause, we don't need to store the updated index
            self.move_current_idx(current_idx, LIMIT_IDX,get_clause_name)?;
            Some(self.parse_limit()?)
        } else {
            None
        };

        Ok(SelectStatement {
            columns,
            distinct,
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


#[cfg(test)]
mod test {
    use super::*;
    use crate::token::tokenize;
    use crate::ast::common::TableReference;
    use crate::ast::select::{SelectStatement, SelectColumn};
    use crate::ast::expr::{BinaryOperator, Expr, LimitClause, OrderByExpr, Value};

    #[test]
    fn test_select_parser() {
        let sql = "SELECT id, name AS user_name FROM users WHERE age >= 18 ORDER BY name DESC, age  LIMIT 10";
        // 词法分析
        let tokens = tokenize(sql);
        let mut parser = Parser::new(tokens);
        // 解析SELECT语句
        let result = parser.parse_select_statement();
        if let Ok(select) = result {
            let expect = SelectStatement {
                columns: vec![
                    SelectColumn::Column {
                        name: "id".to_string(),
                        alias: None,
                    },
                    SelectColumn::Column {
                        name: "name".to_string(),
                        alias: Some("user_name".to_string()),
                    },
                ],
                distinct: false,
                from: TableReference {
                    name: "users".to_string(),
                    alias: None,
                },
                where_clause: Some(Expr::BinaryOp {
                    left: Box::new(Expr::Identifier("age".to_string())),
                    op: BinaryOperator::GtEq,
                    right: Box::new(Expr::Literal(Value::Integer(18))),
                }),
                group_by: None,
                having: None,
                order_by:Some(vec![
                    OrderByExpr {
                        expr: Expr::Identifier("name".to_string()),
                        asc:false,
                    },
                    OrderByExpr {
                        expr: Expr::Identifier("age".to_string()),
                        asc:true,
                    },
                ]),
                limit: Some(LimitClause {
                    limit: 10,
                    offset: None,
                }),
            };
            assert_eq!(select, expect);
        } else {
            println!("Error: {:?}", result.unwrap_err());
        }
    }
}
