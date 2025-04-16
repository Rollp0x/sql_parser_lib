use super::{ParseError, Parser};
use crate::ast::expr::{BinaryOperator, Expr, LogicalOperator, UnaryOperator, Value};
use crate::token::Token;

const MAX_EXPR_DEPTH: usize = 100;

/**
* 递归下降解析器
* parse_expr()
* → parse_logical_or()      // 优先级最低
*   → parse_logical_and()
*     → parse_comparison()
*       → parse_additive()
*         → parse_multiplicative()
*           → parse_primary()  // 优先级最高
*/
impl Parser {
    pub fn parse_expr(&mut self, depth: usize) -> Result<Expr, ParseError> {
        if depth > MAX_EXPR_DEPTH {
            Err(self.get_parse_error("Expression nesting too deep"))
        } else {
            // 先从最低优先级开始解析
            self.parse_logical_or(depth)
        }
    }

    // 解析OR表达式（最低优先级）
    fn parse_logical_or(&mut self, depth: usize) -> Result<Expr, ParseError> {
        let mut expr = self.parse_logical_and(depth)?;
        // 这里使用while是因为or可以连续使用
        while self.match_keyword("OR") {
            let right = self.parse_logical_and(depth)?;
            expr = Expr::LogicalOp {
                op: LogicalOperator::Or,
                expressions: vec![expr, right],
            };
        }

        Ok(expr)
    }

    // 下一优先级：AND
    fn parse_logical_and(&mut self, depth: usize) -> Result<Expr, ParseError> {
        let mut expr = self.parse_not(depth)?;
        // 这里使用while是因为and可以连续使用
        while self.match_keyword("AND") {
            let right = self.parse_not(depth)?;
            expr = Expr::LogicalOp {
                op: LogicalOperator::And,
                expressions: vec![expr, right],
            };
        }
        Ok(expr)
    }

    // 在parse_logical_and之前添加
    fn parse_not(&mut self, depth: usize) -> Result<Expr, ParseError> {
        // 这里使用if是因为not不能连续使用
        if self.match_keyword("NOT") {
            let expr = self.parse_comparison(depth)?;
            return Ok(Expr::LogicalOp {
                op: LogicalOperator::Not,
                expressions: vec![expr],
            });
        }
        self.parse_comparison(depth)
    }

    // 下一优先级：比较
    fn parse_comparison(&mut self, depth: usize) -> Result<Expr, ParseError> {
        let left = self.parse_additive(depth)?; // 先解析加减法表达式

        // 检查是否有比较运算符，这时不用while是因为不会有连续比较运算符
        if let Some(op) = self.match_comparison_operator() {
            let right = self.parse_additive(depth)?;
            return Ok(Expr::BinaryOp {
                left: Box::new(left),
                op,
                right: Box::new(right),
            });
        }

        Ok(left)
    }

    // 下一优先级,解析加法和减法
    fn parse_additive(&mut self, depth: usize) -> Result<Expr, ParseError> {
        let mut expr = self.parse_multiplicative(depth)?;

        while let Some(token) = self.peek() {
            match token.clone() {
                Token::Operator(op) if op == "+" || op == "-" => {
                    self.consume_token(); // 消费token

                    let binary_op = if op == "+" {
                        BinaryOperator::Plus
                    } else {
                        BinaryOperator::Minus
                    };

                    let right = self.parse_multiplicative(depth)?;
                    expr = Expr::BinaryOp {
                        left: Box::new(expr),
                        op: binary_op,
                        right: Box::new(right),
                    };
                }
                _ => break,
            }
        }

        Ok(expr)
    }

    // 解析乘法和除法
    fn parse_multiplicative(&mut self, depth: usize) -> Result<Expr, ParseError> {
        let mut expr = self.parse_unary(depth)?;

        while let Some(token) = self.peek() {
            match token.clone() {
                Token::Operator(op) if op == "*" || op == "/" => {
                    self.consume_token(); // 消费token

                    let binary_op = if op == "*" {
                        BinaryOperator::Multiply
                    } else {
                        BinaryOperator::Divide
                    };

                    let right = self.parse_unary(depth)?;
                    expr = Expr::BinaryOp {
                        left: Box::new(expr),
                        op: binary_op,
                        right: Box::new(right),
                    };
                }
                _ => break,
            }
        }

        Ok(expr)
    }

    // 新增 parse_unary 函数，处理一元操作符
    fn parse_unary(&mut self, depth: usize) -> Result<Expr, ParseError> {
        // 检查是否有一元操作符
        if let Some(token) = self.peek() {
            match token.clone() {
                Token::Operator(op) if op == "+" || op == "-" => {
                    self.consume_token(); // 消费操作符

                    // 递归解析操作数
                    let operand = self.parse_unary(depth)?; // 递归处理连续的一元操作符

                    // 正号可以直接返回操作数，负号需要创建一元表达式
                    if op == "-" {
                        return Ok(Expr::UnaryOp {
                            op: UnaryOperator::Minus,
                            expr: Box::new(operand),
                        });
                    } else {
                        // +号在数值表达式中可以忽略
                        return Ok(operand);
                    }
                }
                _ => {}
            }
        }

        // 没有一元操作符，继续解析基本表达式
        self.parse_primary(depth)
    }

    // 解析无法再分解的表达式
    fn parse_primary(&mut self, depth: usize) -> Result<Expr, ParseError> {
        let c_token = self.consume_token()
            .ok_or_else(|| self.get_parse_error("Expected primary expression, but found none"))?
            .clone();

        match c_token {
            // 字面量处理
            Token::NumericLiteral(n) => {
                // 检查是否包含小数点
                if n.contains('.') {
                    // 尝试解析为浮点数
                    match n.parse::<f64>() {
                        Ok(f) => Ok(Expr::Literal(Value::Float(f))),
                        Err(_) => Err(self.get_parse_error(&format!("Invalid float: {}", n))),
                    }
                } else {
                    // 尝试解析为整数
                    match n.parse::<i64>() {
                        Ok(i) => Ok(Expr::Literal(Value::Integer(i))),
                        Err(_) => Err(self.get_parse_error(&format!("Invalid integer: {}", n))),
                    }
                }
            }
            Token::StringLiteral(s) => Ok(Expr::Literal(Value::String(s))),
            // 标识符处理
            Token::Identifier(ident) => {
                // todo 检查是否是函数调用
                Ok(Expr::Identifier(ident.clone()))
                // // 检查是否是函数调用
                // if self.match_punctuator('(') {
                //     let args = self.parse_function_args()?;
                //     Ok(Expr::FunctionCall {
                //         name: ident.clone(),
                //         args,
                //     })
                // } else {
                //     Ok(Expr::Identifier(ident.clone()))
                // }
            }
            // 处理带有限定符的标识符
            Token::QualifiedIdentifier { qualifier, name } => {
                Ok(Expr::Identifier(format!(
                    "{}.{}",
                    qualifier, name
                )))
            }
            // 括号表达式
            Token::Punctuator('(') => {
                let expr = self.parse_expr(depth + 1)?;

                if !self.match_punctuator(')') {
                    let next_token = self
                        .peek()
                        .map_or("end of input".to_string(), |t| format!("{:?}", t));
                    return Err(self.get_parse_error(&format!(
                        "Expect ')' expression, found: {:?}",
                        next_token
                    )));
                }
                // 如果上述检查通过，则右括号本身已经被消费
                Ok(expr)
            }
            // 处理其他可能的情况
            Token::Keyword(k) if k.to_uppercase() == "NULL" => Ok(Expr::Literal(Value::Null)),
            // 处理星号
            Token::Punctuator('*')  => Ok(Expr::Wildcard),
            // 如果没有匹配的情况，返回错误
            _ => Err(self.get_parse_error(&format!(
                "Unexpected token in primary expression: {:?}",
                c_token
            ))),
        }
    }

    fn match_comparison_operator(&mut self) -> Option<BinaryOperator> {
        if let Some(Token::Operator(op)) = self.peek() {
            let op = op.to_owned();
            // 仅在匹配到时才能消耗token
            let r = match op.as_str() {
                "=" => Some(BinaryOperator::Eq),
                "!=" | "<>" => Some(BinaryOperator::NotEq),
                "<" => Some(BinaryOperator::Lt),
                "<=" => Some(BinaryOperator::LtEq),
                ">" => Some(BinaryOperator::Gt),
                ">=" => Some(BinaryOperator::GtEq),
                _ => None,
            };
            if r.is_some() {
                self.consume_token();
            }
            r
        } else {
            None
        }
    }
}
