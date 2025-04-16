


/// 表示选择的表,暂时不考虑多个表
#[derive(Debug, Clone,PartialEq)]
pub struct TableReference {
    pub name: String,
    pub alias: Option<String>,
}