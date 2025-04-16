use super::expr::{Expr,OrderByExpr,LimitClause};
use super::common::TableReference;

/// SELECT语句结构
#[derive(Debug, Clone,PartialEq)]
pub struct SelectStatement {
    /// 选择的列
    pub columns: Vec<SelectColumn>,
    pub distinct: bool, // false表示ALL，true表示DISTINCT
    /// FROM子句中的表
    pub from: TableReference,
    /// WHERE子句
    pub where_clause: Option<Expr>,
    /// GROUP BY子句
    pub group_by: Option<Vec<Expr>>,
    /// HAVING子句
    pub having: Option<Expr>,
    /// ORDER BY子句
    pub order_by: Option<Vec<OrderByExpr>>,
    /// LIMIT子句
    pub limit: Option<LimitClause>,
}


/// 表示选择的列
#[derive(Debug, Clone,PartialEq)]
pub enum SelectColumn {
    /// 所有列 (*)
    Wildcard,
    /// 指定列，可能包含别名
    Column {
        name: String,
        alias: Option<String>,
    },
}
