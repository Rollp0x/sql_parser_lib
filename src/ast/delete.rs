use super::expr::{Expr,OrderByExpr,LimitClause};
use super::common::TableReference;

/// delete 语句结构
#[derive(Debug, Clone,PartialEq)]
pub struct DeleteStatement {
    pub table: TableReference,
    pub where_clause: Option<Expr>,
    pub order_by: Option<Vec<OrderByExpr>>,
    pub limit: Option<LimitClause>,
    pub is_return_count:bool,
}