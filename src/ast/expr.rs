/// 表示SQL表达式
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    /// 标识符（列名）
    Identifier(String),

    Wildcard,  // * 通配符
    
    /// 字面量（数字、字符串等）
    Literal(Value),
    
    /// 二元操作表达式（如 a = b, x > y）
    BinaryOp {
        left: Box<Expr>,
        op: BinaryOperator,
        right: Box<Expr>,
    },
    
    /// IN 表达式（如 id IN (1, 2, 3)）
    In {
        expr: Box<Expr>,
        list: Vec<Expr>,
        negated: bool,  // 表示是否有 NOT: NOT IN
    },
    
    /// BETWEEN 表达式（如 age BETWEEN 18 AND 30）
    Between {
        expr: Box<Expr>,
        low: Box<Expr>,
        high: Box<Expr>,
        negated: bool,  // 表示是否有 NOT: NOT BETWEEN
    },
    
    /// IS NULL 表达式
    IsNull {
        expr: Box<Expr>,
        negated: bool,  // 表示 IS NULL 或 IS NOT NULL
    },
    
    /// 函数调用（如 COUNT(*), SUM(price)）
    FunctionCall {
        name: String,
        args: Vec<Expr>,
    },
    
    /// 逻辑操作符表达式
    LogicalOp {
        op: LogicalOperator,
        expressions: Vec<Expr>,  // 对于 AND/OR 可能有多个表达式
    },

    /// 一元操作符表达式（如 -x, NOT x）
    UnaryOp {
        op: UnaryOperator,
        expr: Box<Expr>,
    }
}

/// 二元操作符
#[derive(Debug, Clone, PartialEq)]
pub enum BinaryOperator {
    Eq,      // =
    NotEq,   // !=, <>
    Lt,      // <
    LtEq,    // <=
    Gt,      // >
    GtEq,    // >=
    Plus,    // +
    Minus,   // -
    Multiply, // *
    Divide,   // /
    Like,    // LIKE
}

/// 一元操作符
#[derive(Debug, Clone, PartialEq)]
pub enum UnaryOperator {
    Plus,    // +
    Minus,   // -
}

/// 逻辑操作符
#[derive(Debug, Clone, PartialEq)]
pub enum LogicalOperator {
    And,
    Or,
    Not,
}

/// 表示值的类型
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    String(String),
    Integer(i64),
    Float(f64),
    Boolean(bool),
    Null,
    DEFAULT, // 用于DEFAULT关键字
}


/// 表示ORDER BY子句中的表达式
#[derive(Debug, Clone,PartialEq)]
pub struct OrderByExpr {
    pub expr: Expr,    // 允许任何表达式类型
    pub asc: bool,     // true表示ASC，false表示DESC
}

/// 表示LIMIT子句
#[derive(Debug, Clone,PartialEq)]
pub struct LimitClause {
    /// 要返回的最大行数
    pub limit: u64,
    /// 要跳过的行数（用于分页）
    pub offset: Option<u64>,
}