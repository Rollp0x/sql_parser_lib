use crate::kerwords::{TYPES, KEYWORDS};
use regex::Regex;
use lazy_static::lazy_static;

#[non_exhaustive]
#[derive(Debug, Clone,PartialEq)]
pub enum Token {
    /// MySQL 关键字，如 SELECT, FROM, WHERE 等
    Keyword(String),
    ///  表示标识符，比如表名、列名
    Identifier(String),
    /// 字符串字面量，例如 'hello'
    StringLiteral(String),
    /// 数字字面量，例如 123 或 45.67
    NumericLiteral(String),
    /// 操作符，如 =, <, >, <=, >=, != 等
    Operator(String),
    /// 标点符号，如逗号、分号、括号等
    Punctuator(char),
    /// 数据类型，例如 VARCHAR(36)。name 为类型名称，length 为可选长度参数
    DataType { name: String, length: Option<String> },

    // 已有的 Token 类型
    QualifiedIdentifier { qualifier: String, name: String },
}

const OPERATOR_SET: &[&str] = &["=", "<", ">", "<=", ">=", "!=", "+", "-", "*", "/", "%"];
const PUNCTUATORS: &[char] = &[',', ';', '(', ')','.'];

lazy_static! {
    pub static ref RE_BLOCK: Regex = Regex::new(r"(?s)/\*.*?\*/").unwrap();
    pub static ref RE_LINE: Regex = Regex::new(r"(?m)--.*$").unwrap();
    // 用于压缩连续空白字符：\s+ 表示一个或多个空白字符
    pub static ref RE_SPACES: Regex = Regex::new(r"\s+").unwrap();
}

/// 对输入字符串预处理，去除其中的注释，并将换行符替换为空格，
/// 然后进一步压缩多个连续空白为一个。
pub fn preprocess_input(input: &str) -> String {
    // 去除多行注释：使用 (?s) 模式使 `.` 匹配换行符
    let without_block = RE_BLOCK.replace_all(input, "");
    // 去除行注释
    let without_line = RE_LINE.replace_all(&without_block, "");
    // 将换行符替换为空格
    let mut replaced = without_line.replace('\n', " ");
    // 压缩多个连续空白为一个空格，然后 trim 去除首尾空白
    replaced = RE_SPACES.replace_all(&replaced, " ").trim().to_string();
    // 将三个'替换成一个'
    replaced = replaced.replace("'''", "'");

    // 将单引号内的空格替换为特殊标记 "___"
    let mut result = String::new();
    let mut in_quotes = false;

    for ch in replaced.chars() {
        if ch == '\'' {
            in_quotes = !in_quotes;
            result.push(ch);
        } else if ch == ' ' && in_quotes {
            // 在单引号内，用特殊标记替换空格
            result.push_str("___");
        } else if ch == ',' && in_quotes{
            // 在单引号内，用特殊标记替换逗号
            result.push_str("---");
        } else {
            // 其他情况直接添加字符
            result.push(ch);
        }
    }

    result

}

/// 尝试解析数据类型。比如对于 "VARCHAR(36)" 这种形式，将返回 Some(Token::DataType { … })。
fn try_parse_data_type(word: &str) -> Option<Token> {
    // 如果是无参数据类型，如 VARCHAR、INT 等
    if TYPES.contains(&word.to_uppercase()) {
        return Some(Token::DataType {
            name: word.to_string(),
            length: None,
        });
    }
    if let Some(start) = word.find('(') {
        if word.ends_with(')') {
            let name = &word[..start];
            if !TYPES.contains(&name.to_uppercase()) {
                return None; // 不是有效的数据类型
            }
            let inside = &word[start+1..word.len()-1];
            // 这里可以进一步验证 inside 是否为数字或符合其它要求
            return Some(Token::DataType {
                name: name.to_string(),
                length: if inside.is_empty() { None } else { Some(inside.to_string()) },
            });
        }
    }
    None
}

/// 将输入字符串简单拆分为 Token 数组。
/// 注意：这是一个非常基础的实现，仅供学习使用，后续可扩展处理更多语法细节。
pub fn tokenize(input: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    // 预处理后，输入变为统一格式
    let processed = preprocess_input(input);
    for raw_word in processed.split_whitespace() {
        // 看最后一个字符是否是标点符号
        let  mut last_char = None;
        if !raw_word.is_empty()  {
            let c = raw_word.chars().last().unwrap();
            if c == ',' || c == ';' {
                last_char = Some(Token::Punctuator(c));
            }
        }
        let word = if last_char.is_some() {
            &raw_word[..raw_word.len()-1]
        } else {
            raw_word
        };
        // 如果 word 为空，则跳过
        if word.is_empty() {
            if let Some(t) = last_char {
                tokens.push(t);
            }
            continue; // 跳过空单词
        }
        // 如果能作为数据类型识别，则直接处理
        if let Some(t) = try_parse_data_type(word) {
            tokens.push(t);
        }
        // 关键字判断（忽略大小写）
        else if KEYWORDS.contains(&word.to_uppercase()) {
            tokens.push(Token::Keyword(word.to_string()));
        }
        // 数字字面量（仅简单判断所有字符均为数字）
        else if word.chars().all(|c| c.is_ascii_digit()) {
            tokens.push(Token::NumericLiteral(word.to_string()));
        }
        // 字符串字面量（简单检查是否以单引号包裹）
        else if word.starts_with('\'') && word.ends_with('\'') && word.len() >= 2 {
            let inner = &word[1..word.len()-1];
            // 将特殊标记 "___" 替换回空格,将 "---" 替换回逗号
            let restored_inner = inner
                .replace("___", " ")
                .replace("''", "'")
                .replace("---", ",");
            tokens.push(Token::StringLiteral(restored_inner.to_string()));
        }
        // 操作符判断：如果该单词正好匹配预定义操作符之一
        else if OPERATOR_SET.contains(&word) {
            tokens.push(Token::Operator(word.to_string()));
        }
        // 标点符号：如果单词是单个字符且在标点符号集合中
        else if word.len() == 1 && PUNCTUATORS.contains(&word.chars().next().unwrap()) {
            tokens.push(Token::Punctuator(word.chars().next().unwrap()));
        } 
        // 标识符：如果单词是以反引号包裹的标识符
        // 例如 `table_name` 或 `column_name`
        else if word.starts_with('`') && word.ends_with('`') && word.len() >= 2 {
            let inner = &word[1..word.len()-1];
            tokens.push(Token::Identifier(inner.to_string()));
        } 
        // 默认处理为标识符
        else {
            let parsed_tokens = parse_identifier(word);
            for token in parsed_tokens {
                tokens.push(token);
            }
        }
        if let Some(t) = last_char {
            tokens.push(t);
        }
    }

    tokens
}


// 
fn parse_single_identifier(identifier: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut acc = String::new();
    let mut chars = identifier.chars().peekable();
    
    // 添加一个状态变量，用于跟踪是否在反引号内
    let mut in_backticks = false;
    // 添加一个状态变量，用于跟踪是否在单引号内
    let mut in_quotes = false;
    // 用于存储反引号内的内容
    let mut backtick_content = String::new();
    // 用于存储单引号内的内容
    let mut quote_content = String::new();

    while let Some(ch) = chars.next() {
        // 
        if ch == '\'' {
            if in_quotes {
                // 结束引号
                in_quotes = false;
                tokens.push(Token::StringLiteral(quote_content.clone()));
                quote_content.clear();
            } else {
                // 开始引号
                if !acc.is_empty() {
                    // 处理之前的字符
                    let token = if KEYWORDS.contains(&acc.to_uppercase()) {
                        Token::Keyword(acc.clone())
                    } else if acc.chars().all(|c| c.is_ascii_digit()) {
                        Token::NumericLiteral(acc.clone())
                    } else {
                        Token::Identifier(acc.clone())
                    };
                    tokens.push(token);
                    acc.clear();
                }
                in_quotes = true;
            }
        } else if in_quotes {
            // 如果在引号内，则累积字符
            quote_content.push(ch);
        }
        // 检测反引号
        else if ch == '`' {
            if in_backticks {
                // 如果已经在反引号内，则这是结束反引号
                in_backticks = false;
                // 将反引号内的内容作为一个标识符添加
                tokens.push(Token::Identifier(backtick_content.clone()));
                backtick_content.clear();
            } else {
                // 如果不在反引号内，则这是开始反引号
                // 先处理之前可能累积的字符
                if !acc.is_empty() {
                    let token = if KEYWORDS.contains(&acc.to_uppercase()) {
                        Token::Keyword(acc.clone())
                    } else if acc.chars().all(|c| c.is_ascii_digit()) {
                        Token::NumericLiteral(acc.clone())
                    } else {
                        Token::Identifier(acc.clone())
                    };
                    tokens.push(token);
                    acc.clear();
                }
                in_backticks = true;
            }
        } else if in_backticks {
            // 如果在反引号内，则累积字符
            backtick_content.push(ch);
        } else if ch.is_alphanumeric() || ch == '_' {
            // 正常的标识符字符累积
            acc.push(ch);
        } else if ch == '.' {
            // 保存之前累积的标识符作为限定符
            let qualifier = acc.clone();
            acc.clear();

            // 收集点号后的标识符
            while let Some(&next_ch) = chars.peek() {
                if next_ch.is_alphanumeric() || next_ch == '_' {
                    chars.next();
                    acc.push(next_ch);
                } else {
                    break;
                }
            }
            
            // 如果点号前后内容均为数字，则解析为浮点数
            if qualifier.is_empty() || qualifier.chars().all(|c| c.is_ascii_digit())  {
                // 构建完整的浮点数字符串
                let float_str = format!("{}.{}", qualifier, acc);
                tokens.push(Token::NumericLiteral(float_str));
                acc.clear();
            }
            // 否则，如果点号后有内容，创建限定标识符
            else if !acc.is_empty() {
                tokens.push(Token::QualifiedIdentifier {
                    qualifier,
                    name: acc.clone()
                });
                acc.clear();
            } else {
                // 处理错误情况：点号后没有标识符
                tokens.push(Token::Identifier(qualifier));
                tokens.push(Token::Punctuator('.'));
            }
        } else {
            // 处理积累的普通标识符
            if !acc.is_empty() {
                let token = if KEYWORDS.contains(&acc.to_uppercase()) {
                    Token::Keyword(acc.clone())
                } else if acc.chars().all(|c| c.is_ascii_digit()) {
                    Token::NumericLiteral(acc.clone())
                } else {
                    Token::Identifier(acc.clone())
                };
                tokens.push(token);
                acc.clear();
            }
            
            // 处理标点符号和操作符
            if PUNCTUATORS.contains(&ch) {
                tokens.push(Token::Punctuator(ch));
            } else {
                let op_str = ch.to_string();
                if OPERATOR_SET.contains(&op_str.as_str()) {
                    tokens.push(Token::Operator(op_str));
                } else if !ch.is_whitespace() {
                    tokens.push(Token::Identifier(op_str));
                }
            }
        }
    }
    
    // 处理最后可能剩余的字符
    if !acc.is_empty() {
        let token = if KEYWORDS.contains(&acc.to_uppercase()) {
            Token::Keyword(acc)
        } else if acc.chars().all(|c| c.is_ascii_digit()) {
            Token::NumericLiteral(acc)
        } else {
            Token::Identifier(acc)
        };
        tokens.push(token);
    }
    
    // 确保任何未闭合的反引号内容也被处理
    if in_backticks && !backtick_content.is_empty() {
        // 可以选择报错或者将未闭合的反引号内容作为普通标识符处理
        tokens.push(Token::Identifier(backtick_content));
    }
    
    tokens
}

/// 解析标识符，处理可能的关键字、数字和操作符。
/// 该函数会将输入字符串拆分为多个 Token。
/**
 * @param identifier: 输入的未处理的标识符字符串,可能包含关键字、数字和操作符
 * @return: 返回一个 Token 向量，包含解析后的标识符、关键字、数字和操作符
 * @note: 该函数会将输入字符串拆分为多个 Token，处理可能的关键字、数字和操作符。
 */
fn parse_identifier(identifier: &str) -> Vec<Token> {
    // 对 identifier 进行预处理，给部分符号增加空格
    let identifier = identifier
        .replace("(", " ( ")
        .replace(")", " ) ")
        .replace(",", " , ")
        .replace(";", " ; ");
    let mut tokens = Vec::new();
    for word in identifier.split_whitespace() {
        if word.is_empty() {
            continue; // 跳过空单词
        }
        // 处理可能的标识符、关键字、数字和操作符
        let parsed_tokens = parse_single_identifier(word);
        for token in parsed_tokens {
            tokens.push(token);
        }
    }
    tokens
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_preprocess_input_block_comment() {
        let input = "SELECT * FROM users; /* block comment spanning multiple lines\ncontinued comment */";
        let expected = "SELECT * FROM users;";
        assert_eq!(preprocess_input(input), expected);
    }

    #[test]
    fn test_preprocess_input_line_comment() {
        let input = "SELECT * FROM users -- this is a line comment\nWHERE id = 1;";
        // 行注释删除后，会保留换行符
        let expected = "SELECT * FROM users WHERE id = 1;";
        assert_eq!(preprocess_input(input), expected);
    }

    #[test]
    fn test_preprocess_input_combined_comments() {
        let input = "/* first comment */\nSELECT * FROM users -- line comment\nWHERE id = 1; /* second comment */";
        let expected = "SELECT * FROM users WHERE id = 1;";
        assert_eq!(preprocess_input(input), expected);
    }

    #[test]
    fn test_preprocess_input_no_comment() {
        let input = "SELECT * FROM users WHERE id = 1;";
        let expected = "SELECT * FROM users WHERE id = 1;";
        assert_eq!(preprocess_input(input), expected);
    }

    #[test]
    fn test_parse_identifier() {
        let input = "value=500";
        let expected = vec![
            Token::Identifier("value".to_string()),
            Token::Operator("=".to_string()),
            Token::NumericLiteral("500".to_string()),
        ];
        let result = parse_identifier(input);
        assert_eq!(result, expected);

        let input = "values(1,2,3)";
        let expected = vec![
            Token::Keyword("values".to_string()),
            Token::Punctuator('('),
            Token::NumericLiteral("1".to_string()),
            Token::Punctuator(','),
            Token::NumericLiteral("2".to_string()),
            Token::Punctuator(','),
            Token::NumericLiteral("3".to_string()),
            Token::Punctuator(')'),
        ];
        let result = parse_identifier(input);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_tokenize2() {
        let sql = r#"
        --
        -- Table structure for table `async_task`
        --

        DROP TABLE IF EXISTS `async_task`;
        /*!40101 SET @saved_cs_client     = @@character_set_client */;
        /*!50503 SET character_set_client = utf8mb4 */;
        CREATE TABLE `async_task` (
        `id` bigint unsigned NOT NULL AUTO_INCREMENT,
        `num` bigint unsigned NOT NULL DEFAULT '0' COMMENT '''Start block number''',
        `end_num` bigint unsigned NOT NULL DEFAULT '0' COMMENT '''End block number''',
        `name` varchar(256) NOT NULL DEFAULT '' COMMENT '''Task name''',
        PRIMARY KEY (`id`)
        ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci;
        /*!40101 SET character_set_client = @saved_cs_client */;
        "#;

        let tokens = tokenize(sql);

        dbg!(tokens);
    }

    #[test]
    fn test_tokenize1() {
        let sql = r#"
            CREATE TABLE IF NOT EXISTS `new_address_infos` (
                `id` VARCHAR(36) PRIMARY KEY NOT NULL,  -- 使用的uuid
                `address` char(42) NOT NULL DEFAULT '',
                `chain_id` int unsigned NOT NULL DEFAULT '200901',
                `category` varchar(20) NOT NULL DEFAULT 'Unknown',
                `risk_level` varchar(20) NOT NULL DEFAULT 'Low',
                `risk_category` varchar(50) DEFAULT NULL,
                `risk_hash` char(66) DEFAULT NULL COMMENT 'Risk Transaction hash',
                `risk_at` int unsigned DEFAULT NULL,
                `risk_origin` varchar(50) DEFAULT NULL,
                `creator` char(42) DEFAULT NULL,
                `end_class` varchar(50) NOT NULL DEFAULT 'Point',
                `first_time` int unsigned DEFAULT NULL,
                `comments` varchar(255) DEFAULT NULL,
                `label_group` varchar(50) DEFAULT NULL,
                `label_origin` varchar(50) DEFAULT NULL,
                `label` varchar(50) DEFAULT NULL,
                UNIQUE KEY `address` (`address`),
                KEY `label_group` (`label_group`)
            ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci;
        "#;
        let tokens = tokenize(sql);

        dbg!(tokens);
    }   

    #[test]
    fn test_complex_tokens() {
        let sql = "DELETE FROM employees e WHERE (e.department = 'IT' AND e.salary > 100000) OR (e.last_active < '2023-01-01' AND e.status = 'inactive') ORDER BY e.last_active DESC, e.name LIMIT 50";
        let tokens = tokenize(sql);
        dbg!(tokens);
    }
}