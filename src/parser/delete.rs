use super::{ParseError, Parser};

use crate::ast::{
    common::TableReference,
    delete::DeleteStatement,
};

// delete语句解析器接口
pub trait DeleteStatementParser {
    type Error;
    // 解析delete语句
    fn parse_delete_statement(&mut self) -> Result<DeleteStatement, Self::Error>;
}


// 子句优先级/索引
const FROM_IDX: u8 = 0;
const WHERE_IDX: u8 = 1;
const ORDER_BY_IDX: u8 = 2;
const LIMIT_IDX: u8 = 3;


impl DeleteStatementParser for Parser {
    type Error = ParseError;
    // 解析DELETE语句
    fn parse_delete_statement(&mut self) -> Result<DeleteStatement, Self::Error> {
        // 期望以DELETE关键字开始
        if !self.match_keyword("DELETE") {
            return Err(self.get_parse_error(&format!("Expected DELETE, found{:?}", self.peek())));
        }

        // 必须有FROM子句
        if !self.match_keyword("FROM") {
            return Err(self.get_parse_error(&format!("Expected FROM, found {:?}", self.peek())));
        }

        // 解析FROM的表引用
        let table: TableReference = self.parse_table_reference(false)?;

        // 跟踪当前已处理的最高子句索引
        let mut current_idx: u8 = FROM_IDX;

        // 可选的WHERE子句
        let where_clause = if self.match_keyword("WHERE") {
            current_idx = self.move_current_idx(current_idx, WHERE_IDX,get_clause_name)?;
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

        // 完成DELETE语句解析
        Ok(DeleteStatement {
            table,
            where_clause,
            order_by,
            limit,
            is_return_count: true, // 默认行为
        })
    }
}

// 可选的辅助函数，将索引转换为子句名称
fn get_clause_name(idx: u8) -> &'static str {
    match idx {
        WHERE_IDX => "WHERE",
        ORDER_BY_IDX => "ORDER BY",
        LIMIT_IDX => "LIMIT",
        _ => "FROM",
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::ast::expr::{BinaryOperator, Expr, LimitClause, OrderByExpr, Value,LogicalOperator};

    #[test]
    fn test_delete_parser()  {
        let sql = "DELETE FROM users WHERE id = 1 ORDER BY name LIMIT 10";
        let mut parser = Parser::new_from_sql(sql);
        let result = parser.parse_delete_statement();
        if let Ok(delete) = result {
            let expect = DeleteStatement {
                table: TableReference {
                    name: "users".to_string(),
                    alias: None,
                },
                where_clause: Some(Expr::BinaryOp {
                    left: Box::new(Expr::Identifier("id".to_string())),
                    op: BinaryOperator::Eq,
                    right: Box::new(Expr::Literal(Value::Integer(1))),
                }),
                order_by:Some(vec![
                    OrderByExpr {
                        expr: Expr::Identifier("name".to_string()),
                        asc:true,
                    }
                ]),
                limit: Some(LimitClause {
                    limit: 10,
                    offset: None,
                }),
                is_return_count: true,
            };
            assert_eq!(delete, expect);
        } else {
            println!("Error parsing delete statement: {:?}", result.unwrap_err());
        }
    }

    #[test]
    fn test_complex_delete_parser()  {
        // 包含表别名、复杂WHERE条件、多字段排序和LIMIT子句
        let sql = "DELETE FROM employees e WHERE (e.department = 'IT' AND e.salary > 100000) OR (e.last_active < '2023-01-01' AND e.status = 'inactive') ORDER BY e.last_active DESC, e.name LIMIT 50";
        let mut parser = Parser::new_from_sql(sql);
        let result = parser.parse_delete_statement();
        if let Ok(delete) = result {
            let expect = DeleteStatement {
                table: TableReference {
                    name: "employees".to_string(),
                    alias: Some("e".to_string()),
                },
                where_clause: Some(Expr::LogicalOp {
                    op:LogicalOperator::Or,
                    expressions:vec![
                        Expr::LogicalOp {
                            op:LogicalOperator::And,
                            expressions:vec![
                                Expr::BinaryOp {
                                    left: Box::new(Expr::Identifier("e.department".to_string())),
                                    op: BinaryOperator::Eq,
                                    right: Box::new(Expr::Literal(Value::String("IT".to_string()))),
                                },
                                Expr::BinaryOp {
                                    left: Box::new(Expr::Identifier("e.salary".to_string())),
                                    op: BinaryOperator::Gt,
                                    right: Box::new(Expr::Literal(Value::Integer(100000))),
                                }
                            ]
                        },
                        Expr::LogicalOp {
                            op:LogicalOperator::And,
                            expressions:vec![
                                Expr::BinaryOp {
                                    left: Box::new(Expr::Identifier("e.last_active".to_string())),
                                    op: BinaryOperator::Lt,
                                    right: Box::new(Expr::Literal(Value::String("2023-01-01".to_string()))),
                                },
                                Expr::BinaryOp {
                                    left: Box::new(Expr::Identifier("e.status".to_string())),
                                    op: BinaryOperator::Eq,
                                    right: Box::new(Expr::Literal(Value::String("inactive".to_string()))),
                                }
                            ]
                        }

                    ]
                }),
                order_by: Some(vec![
                    OrderByExpr {
                        expr: Expr::Identifier("e.last_active".to_string()),
                        asc: false,
                    },
                    OrderByExpr {
                        expr: Expr::Identifier("e.name".to_string()),
                        asc: true,
                    }
                ]),
                limit: Some(LimitClause {
                    limit: 50,
                    offset: None,
                }),
                is_return_count: true,
            };
            assert_eq!(delete, expect);
        } else {
            println!("Error parsing delete statement: {:?}", result.unwrap_err());
        }
           
    }
}