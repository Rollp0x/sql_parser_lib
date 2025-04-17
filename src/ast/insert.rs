use super::expr::{Expr,OrderByExpr,LimitClause};
use super::common::TableReference;
use super::select::SelectStatement;

/// insert 语句结构
#[derive(Debug, Clone,PartialEq)]
pub struct InsertStatement {
    pub table: TableReference,  // 表名
    pub columns: Option<Vec<String>>,  // 可选列名
    pub values: Option<Vec<Vec<Expr>>>, // 插入的值(可以插入多个记录)
    pub select_clause: Option<SelectStatement>, // 当没有values时，使用select语句插入
    pub set_clause: Option<Vec<(String, Expr)>>, // 当没有values时，使用set语句插入
    pub on_duplicate: Option<OnDuplicateClause>, // 冲突处理
    pub is_default_values: bool,  // 是否为 INSERT ... DEFAULT VALUES
    pub is_return_count:bool,
}


// 冲突处理子句
#[derive(Debug, Clone,PartialEq)]
pub struct OnDuplicateClause {
    pub updates: Vec<(String, Expr)>,  // 列名和新值对
}
