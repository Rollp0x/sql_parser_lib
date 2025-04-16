pub mod expr;
pub mod common;
pub mod select;

pub mod delete;

pub use select::{SelectStatement, SelectColumn};


#[non_exhaustive]
#[derive(Debug, Clone)]
pub enum SQLStatement {
    Select(SelectStatement),
    // Insert(InsertStatement),
    // Update(UpdateStatement),
    // Delete(DeleteStatement),
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

