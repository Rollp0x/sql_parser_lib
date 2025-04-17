use super::{ParseError, Parser};
use crate::ast::expr::Expr;
use crate::ast::select::SelectStatement;
use crate::token::Token;
use crate::ast::{
    common::TableReference,
    insert::{InsertStatement, OnDuplicateClause},
};
use super::select::SelectStatementParser;

/// insert语句解析器接口
pub trait InsertStatementParser {
    type Error;
    // 解析insert语句
    fn parse_insert_statement(&mut self) -> Result<InsertStatement, Self::Error>;
}


// 子句优先级/索引
const INTO_IDX: u8 = 0;
const VALUES_IDX: u8 = 1;
const ON_DUPLICATE_KEY_UPDATE_IDX: u8 = 2;

impl Parser {
    fn parse_select_clause(&mut self) -> Result<Option<SelectStatement>, ParseError> {
        if self.is_keyword("SELECT") {
            // 解析SELECT子句
            let select_statement = self.parse_select_statement()?;
            return Ok(Some(select_statement));
        } else {
            return Ok(None);
        }
    }
    fn parse_values_clause(&mut self) -> Result<Option<Vec<Vec<Expr>>>,ParseError> {
        if self.match_keyword("VALUES") {
            let mut values = Vec::new();
            loop {
                // 解析值列表
                if !self.match_punctuator('(') {
                    return Err(self.get_parse_error("Expected opening parenthesis"));
                }
                
                // 新增: 检查是否是空括号对
                if self.match_punctuator(')') {
                    // 空值列表
                    values.push(Vec::new()); // 添加空的值列表
                } else {
                    let mut value_list = Vec::new();
                    loop {
                        let value = self.parse_expr(0)?;
                        value_list.push(value);
                        
                        if !self.match_punctuator(',') {
                            break;
                        }
                    }
                    
                    if !self.match_punctuator(')') {
                        return Err(self.get_parse_error("Expected closing parenthesis"));
                    }
                    
                    values.push(value_list);
                }
                    
                // 检查是否有更多的值列表
                if !self.match_punctuator(',') {
                    break;
                }
            }

            return Ok(Some(values));
        } else {
            return Ok(None);
        }
    }

    fn parse_set_clause(&mut self) -> Result<Option<Vec<(String, Expr)>>, ParseError> {
        if self.match_keyword("SET") {
            let mut set_clause = Vec::new();
            loop {
                // 解析列名
                let column = match self.peek() {
                    Some(Token::Identifier(ident)) => {
                        let name = ident.to_owned();
                        self.consume_token();
                        name
                    }
                    _ => return Err(self.get_parse_error("Expected column name"))
                };
                
                // 解析等号
                if !self.match_operator("=") {
                    return Err(self.get_parse_error("Expected = after column name"));
                }
                
                // 解析表达式
                let value = self.parse_expr(0)?;
                
                // 添加到SET子句
                set_clause.push((column, value));
                
                // 检查是否有更多的赋值
                if !self.match_punctuator(',') {
                    break;
                }
            }

            return Ok(Some(set_clause));
        } else {
            return Ok(None);
        }
    }

    // 解析插入的列名
    fn parse_insert_columns(&mut self) -> Result<Option<Vec<String>>, ParseError> {
       // 解析可选的列名列表
        let columns = if self.match_punctuator('(') {
            let mut column_list = Vec::new();
            // 如果没有列名，直接返回
            if self.match_punctuator(')') {
                return Ok(Some(column_list));
            }
            // 循环解析列名
            loop {
                match self.peek() {
                    Some(Token::Identifier(ident)) => {
                        let column = ident.to_owned();
                        self.consume_token();
                        column_list.push(column);
                    }
                    _ => return Err(self.get_parse_error("Expected column name"))
                };
                
                if !self.match_punctuator(',') {
                    break;
                }
            }
            
            if !self.match_punctuator(')') {
                return Err(self.get_parse_error("Expected closing parenthesis"));
            }
            
            Some(column_list)
        } else {
            None
        };
        Ok(columns)
    }

    fn parse_default_values(&mut self) -> Result<bool, ParseError> {
        if self.match_keyword("DEFAULT") {
            if self.match_keyword("VALUES") {
                return Ok(true);
            } else {
                return Err(self.get_parse_error(&format!(
                    "Expected VALUES after DEFAULT, found {:?}",
                    self.peek()
                )));
            }
        } else {
            return Ok(false);
        }
    }

    fn parse_on_duplicate_key_update(&mut self) -> Result<Option<OnDuplicateClause>, ParseError>  {
        // 如果没有ON关键字，表示没有这个子句
        if !self.match_keyword("ON") {
            return Ok(None);
        }
        // 检查完整的关键字序列
        if !self.match_keyword("DUPLICATE") {
            return Err(self.get_parse_error("Expected DUPLICATE after ON"));
        }

        if !self.match_keyword("KEY") {
            return Err(self.get_parse_error("Expected KEY after ON DUPLICATE"));
        }

        if !self.match_keyword("UPDATE") {
            return Err(self.get_parse_error("Expected UPDATE after ON DUPLICATE KEY"));
        }

        // 解析赋值列表
        let mut updates = Vec::new();
        
        loop {
            // 解析列名
            let column = match self.peek() {
                Some(Token::Identifier(ident)) => {
                    let name = ident.to_owned();
                    self.consume_token();
                    name
                }
                _ => return Err(self.get_parse_error("Expected column name"))
            };
            
            // 解析等号
            if !self.match_operator("=") {
                return Err(self.get_parse_error("Expected = after column name"));
            }
            
            // 解析表达式
            let value = self.parse_expr(0)?;
            
            // 添加到更新列表
            updates.push((column, value));
            
            // 检查是否有更多的赋值
            if !self.match_punctuator(',') {
                break;
            }
        }

        Ok(Some(OnDuplicateClause { updates }))
    }
}

impl InsertStatementParser for Parser {
    type Error = ParseError;
    // 解析INSERT语句
    fn parse_insert_statement(&mut self) -> Result<InsertStatement, Self::Error> {
        // 期望以insert关键字开始
        if !self.match_keyword("INSERT") {
            return Err(self.get_parse_error(&format!("Expected INSERT, found{:?}", self.peek())));
        }

        // 必须有into子句
        if !self.match_keyword("INTO") {
            return Err(self.get_parse_error(&format!("Expected INTO, found {:?}", self.peek())));
        }

        // 解析INTO的表引用
        let table: TableReference = self.parse_table_reference(false)?;

        let columns = self.parse_insert_columns()?;
        // 先判断是否为全部的默认值
        let is_default_values = self.parse_default_values()?;

        if columns.is_some() && is_default_values {
            return Err(self.get_parse_error("Cannot specify columns with DEFAULT VALUES"));
        }

        let values = self.parse_values_clause()?;

        let set_clause = self.parse_set_clause()?;
        let select_clause = self.parse_select_clause()?;

        let data_sources = [
            is_default_values,
            values.is_some(),
            set_clause.is_some(),
            select_clause.is_some()
        ].iter().filter(|&&x| x).count();
        // 检查数据源的数量
        if data_sources > 1 {
            return Err(self.get_parse_error("Cannot specify multiple value sources"));
        }
        if data_sources == 0 {
            return Err(self.get_parse_error("Expected VALUES, SELECT, DEFAULT VALUES or SET"));
        }
        // 跟踪当前已处理的最高子句索引
        let current_idx: u8 = VALUES_IDX;

        let on_duplicate = self.parse_on_duplicate_key_update()?;
        if on_duplicate.is_some() {
            self.move_current_idx(current_idx, ON_DUPLICATE_KEY_UPDATE_IDX,get_clause_name)?;
        }

        Ok(InsertStatement {
            table,
            columns,
            values,
            select_clause,
            set_clause,
            on_duplicate,
            is_default_values,
            is_return_count: true, // 默认返回行数
        })

    }

    
}

// 可选的辅助函数，将索引转换为子句名称
fn get_clause_name(idx: u8) -> &'static str {
    match idx {
        INTO_IDX => "INTO",
        VALUES_IDX => "VALUES",
        ON_DUPLICATE_KEY_UPDATE_IDX => "ON_DUPLICATE_KEY_UPDATE",
        _ => "INTO",
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::ast::expr::{Expr, Value};

    #[test]
    fn test_basic_insert() {
        // 基本的INSERT语句，包含列名和VALUES
        let sql = "INSERT INTO users (id, name, email) VALUES (1, 'John', 'john@example.com')";
        let mut parser = Parser::new_from_sql(sql);
        let result = parser.parse_insert_statement();
        
        assert!(result.is_ok(), "解析失败: {:?}", result.err());
        let stmt = result.unwrap();
        
        // 验证表名
        assert_eq!(stmt.table.name, "users");
        assert_eq!(stmt.table.alias, None);
        
        // 验证列名
        assert_eq!(stmt.columns, Some(vec!["id".to_string(), "name".to_string(), "email".to_string()]));
        
        // 验证值
        assert!(stmt.values.is_some());
        let values = stmt.values.unwrap();
        assert_eq!(values.len(), 1); // 一行数据
        assert_eq!(values[0].len(), 3); // 三个值
        
        // 验证第一个值是数字1
        if let Expr::Literal(Value::Integer(num)) = &values[0][0] {
            assert_eq!(*num, 1);
        } else {
            panic!("Expected integer 1, found {:?}", values[0][0]);
        }
        
        // 验证第二个值是字符串"John"
        if let Expr::Literal(Value::String(s)) = &values[0][1] {
            assert_eq!(*s, "John".to_string());
        } else {
            panic!("Expected string 'John', found {:?}", values[0][1]);
        }

        // 验证第三个值是字符串"
        if let Expr::Literal(Value::String(s)) = &values[0][2] {
            assert_eq!(*s, "john@example.com".to_string());
        } else {
            panic!("Expected string 'john@example.com', found {:?}", values[0][2]); 
        }
    }


    #[test]
    fn test_insert_set() {
        // 测试MySQL特有的SET语法
        let sql = "INSERT INTO logs SET message = 'Error occurred', level = 'ERROR', `timestamp` = NOW()";
        let mut parser = Parser::new_from_sql(sql);
        let result = parser.parse_insert_statement();
        
        assert!(result.is_ok(), "解析失败: {:?}", result.err());
        let stmt = result.unwrap();
        
        // 验证表名
        assert_eq!(stmt.table.name, "logs");
        
        // 验证没有VALUES子句
        assert!(stmt.values.is_none());
        
        // 验证SET子句
        assert!(stmt.set_clause.is_some());
        let set_clause = stmt.set_clause.unwrap();
        assert_eq!(set_clause.len(), 3); // 三个赋值
        
        // 验证第一个赋值
        assert_eq!(set_clause[0].0, "message");
        if let Expr::Literal(Value::String(s)) = &set_clause[0].1 {
            assert_eq!(*s, "Error occurred".to_string());
        } else {
            panic!("Expected string 'Error occurred', found {:?}", set_clause[0].1);
        }
        
        // 验证第三个赋值是函数调用
        assert_eq!(set_clause[2].0, "timestamp");
        if let Expr::FunctionCall { name, args } = &set_clause[2].1 {
            assert_eq!(*name, "NOW".to_string());
            assert_eq!(args.len(), 0);
        } else {
            panic!("Expected function NOW(), found {:?}", set_clause[2].1);
        }
    }

    #[test]
    fn test_complex_insert() {
        // 复杂的INSERT语句，包含多行VALUES和ON DUPLICATE KEY UPDATE
        let sql = "INSERT INTO products (id, name, price, stock) 
                  VALUES 
                  (101, 'Laptop', 999.99, 50),
                  (102, 'Smartphone', 499.99, 100)
                  ON DUPLICATE KEY UPDATE 
                  stock = stock + `VALUES`(stock),
                  update_time = NOW()";
        
        let mut parser = Parser::new_from_sql(sql);
        let result = parser.parse_insert_statement();
        
        assert!(result.is_ok(), "解析失败: {:?}", result.err());
        let stmt = result.unwrap();
        
        // 验证表名
        assert_eq!(stmt.table.name, "products");
        
        // 验证列名
        assert_eq!(stmt.columns, Some(vec![
            "id".to_string(), 
            "name".to_string(), 
            "price".to_string(), 
            "stock".to_string()
        ]));
        
        // 验证VALUES
        assert!(stmt.values.is_some());
        let values = stmt.values.unwrap();
        assert_eq!(values.len(), 2); // 两行数据
        
        // 验证第一行的价格是999.99
        if let Expr::Literal(Value::Float(num)) = &values[0][2] {
            assert!((num - 999.99).abs() < 0.001); // 浮点数比较
        } else {
            panic!("Expected float 999.99, found {:?}", values[0][2]);
        }
        
        // 验证ON DUPLICATE KEY UPDATE子句
        assert!(stmt.on_duplicate.is_some());
        let on_duplicate = stmt.on_duplicate.unwrap();
        assert_eq!(on_duplicate.updates.len(), 2); // 两个更新表达式
        
        // 验证第一个更新是stock = stock + VALUES(stock)
        assert_eq!(on_duplicate.updates[0].0, "stock");
    }


}