use std::collections::HashSet;
use serde_json;
use lazy_static::lazy_static;


lazy_static! {
    pub static ref KEYWORDS: HashSet<String> = {
        // 使用 include_str! 宏在编译期内嵌入 JSON 数据
        let json_str = include_str!("../keywords.json");
        let keywords: Vec<String> =
            serde_json::from_str(json_str).expect("Failed to parse keyword.json");
        keywords.into_iter().collect()
    };

    pub static ref TYPES:HashSet<String> = {
        // 使用 include_str! 宏在编译期内嵌入 JSON 数据
        let json_str = include_str!("../types.json");
        let types: Vec<String> =
            serde_json::from_str(json_str).expect("Failed to parse types.json");
        types.into_iter().collect()
    };
}