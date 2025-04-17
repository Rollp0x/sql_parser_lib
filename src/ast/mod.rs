pub mod expr;
pub mod common;
pub mod select;
pub mod insert;

pub mod delete;

pub use select::{SelectStatement, SelectColumn};
use delete::DeleteStatement;

#[non_exhaustive]
#[derive(Debug, Clone)]
pub enum SQLStatement {
    Select(SelectStatement),
    // Insert(InsertStatement),
    // Update(UpdateStatement),
    Delete(DeleteStatement),
    // Create(CreateStatement),
    // Drop(DropStatement),
    // Alter(AlterStatement),
    // Use(UseStatement),
    // Show(ShowStatement),
    // Explain(ExplainStatement),
    // BeginTransaction(BeginTransactionStatement),
    // Commit(CommitStatement),
    // Rollback(RollbackStatement),
}

