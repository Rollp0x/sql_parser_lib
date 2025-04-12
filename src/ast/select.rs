use super::expr::{Expr,OrderByExpr,LimitClause};


/// SELECT语句结构
#[derive(Debug, Clone)]
pub struct SelectStatement {
    /// 选择的列
    pub columns: Vec<SelectColumn>,
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


/// 表示选择的表,暂时不考虑多个表
#[derive(Debug, Clone)]
pub struct TableReference {
    pub name: String,
    pub alias: Option<String>,
}

/// 表示选择的列
#[derive(Debug, Clone)]
pub enum SelectColumn {
    /// 所有列 (*)
    Wildcard,
    /// 指定列，可能包含别名
    Column {
        name: String,
        alias: Option<String>,
    },
}
