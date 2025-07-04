use regex::Regex;
use std::collections::HashMap;
use std::sync::Mutex;
use std::sync::Once;
use crate::dateutil::format_now_with_diff;

/// 全局占位符工具类
pub struct PlaceholderUtil {
    pub mapping: HashMap<String, String>,
}

impl PlaceholderUtil {
    /// 创建新的占位符工具实例
    pub fn new() -> Self {
        Self {
            mapping: HashMap::new(),
        }
    }
    
    /// 初始化占位符映射
    /// 
    /// # Arguments
    /// 
    /// * `mapping` - 键值对映射，用于 simple_mapping UDF
    pub fn init(&mut self, mapping: HashMap<String, String>) {
        self.mapping = mapping;
    }
    
    /// simple_mapping UDF：从映射中查找值
    fn simple_mapping(&self, args: &[&str]) -> String {
        if args.is_empty() {
            return String::new();
        }
        
        let key = args[0];
        self.mapping.get(key).cloned().unwrap_or_default()
    }
    
    /// get_time UDF：格式化时间
    fn get_time(&self, args: &[&str]) -> String {
        if args.is_empty() {
            return String::new();
        }
        
        let format = args[0];
        let offset_ms: i64 = if args.len() > 1 {
            args[1].parse().unwrap_or(0)
        } else {
            0
        };
        
        format_now_with_diff(format, offset_ms)
    }
    
    /// 解析字符串中的占位符并替换为实际值
    /// 
    /// 支持的占位符格式：
    /// - `{{placeholder|:|udf_name|:|arg1|:|arg2}}` - 调用指定的 UDF
    /// 
    /// 目前支持的 UDF：
    /// - `simple_mapping|:|key` - 从映射中查找值
    /// - `get_time|:|format|:|offset_ms` - 格式化时间
    /// 
    /// # Arguments
    /// 
    /// * `template` - 包含占位符的模板字符串
    /// 
    /// # Returns
    /// 
    /// 替换了所有占位符的字符串
    pub fn replace_placeholders(&self, template: &str) -> String {
        let re = Regex::new(r"\{\{placeholder\|:\|([^}]+)\}\}").unwrap();
        re.replace_all(template, |caps: &regex::Captures| {
            let content = &caps[1];
            let parts: Vec<&str> = content.split("|:|").collect();
            
            if parts.is_empty() {
                return String::new();
            }
            
            let udf_name = parts[0];
            let args = &parts[1..];
            
            match udf_name {
                "simple_mapping" => self.simple_mapping(args),
                "get_time" => self.get_time(args),
                _ => format!("{{placeholder|:|{}|:|{}", udf_name, args.join("|:|")),
            }
        }).to_string()
    }
}

// 全局实例
static GLOBAL_PLACEHOLDER: Mutex<Option<PlaceholderUtil>> = Mutex::new(None);
static INIT: Once = Once::new();

/// 初始化全局占位符工具
pub fn init_global(mapping: HashMap<String, String>) {
    INIT.call_once(|| {
        let mut util = PlaceholderUtil::new();
        util.init(mapping);
        if let Ok(mut global) = GLOBAL_PLACEHOLDER.lock() {
            *global = Some(util);
        }
    });
}


/// 添加单个键值对到全局映射
pub fn add_global_mapping(key: String, value: String) {
    if let Ok(mut global) = GLOBAL_PLACEHOLDER.lock() {
        if let Some(util) = global.as_mut() {
            util.mapping.insert(key, value);
        } else {
            // 如果还没有初始化，先初始化
            let mut util = PlaceholderUtil::new();
            util.mapping.insert(key, value);
            *global = Some(util);
        }
    }
}

/// 全局占位符替换函数
pub fn replace_placeholders(template: &str) -> String {
    if let Ok(global) = GLOBAL_PLACEHOLDER.lock() {
        if let Some(util) = global.as_ref() {
            util.replace_placeholders(template)
        } else {
            // 如果没有初始化，返回原字符串
            template.to_string()
        }
    } else {
        // 如果获取锁失败，返回原字符串
        template.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_placeholder_util() {
        // 测试实例方法
        let mut util = PlaceholderUtil::new();
        
        let mut mapping = HashMap::new();
        mapping.insert("key1".to_string(), "value1".to_string());
        mapping.insert("key2".to_string(), "value2".to_string());
        util.init(mapping);
        
        // 测试 simple_mapping UDF
        let result = util.replace_placeholders("/var/log/nginx/access_{{placeholder|:|simple_mapping|:|key1}}.log");
        assert!(result.contains("/var/log/nginx/access_value1.log"));
        
        // 测试 get_time UDF
        let result = util.replace_placeholders("/var/log/nginx/access_{{placeholder|:|get_time|:|%Y%m%d}}.log");
        assert!(result.contains("/var/log/nginx/access_"));
        assert!(result.contains(".log"));
        
        // 测试带偏移量的 get_time UDF
        let result = util.replace_placeholders("/var/log/nginx/access_{{placeholder|:|get_time|:|%Y%m%d|:|-86400000}}.log");
        assert!(result.contains("/var/log/nginx/access_"));
        assert!(result.contains(".log"));
        
        // 测试没有占位符的字符串
        let result = util.replace_placeholders("/var/log/nginx/access.log");
        assert_eq!(result, "/var/log/nginx/access.log");
        
        // 测试多个占位符
        let result = util.replace_placeholders("/var/log/nginx/access_{{placeholder|:|get_time|:|%Y%m%d}}_{{placeholder|:|simple_mapping|:|key2}}.log");
        assert!(result.contains("/var/log/nginx/access_"));
        assert!(result.contains("_value2.log"));
        
        // 测试未知的 UDF
        let result = util.replace_placeholders("/var/log/nginx/access_{{placeholder|:|unknown_udf|:|arg1}}.log");
        assert!(result.contains("{{placeholder|:|unknown_udf|:|arg1"));
    }
    
    #[test]
    fn test_global_placeholder() {
        // 测试全局函数
        let mut mapping = HashMap::new();
        mapping.insert("key1".to_string(), "value1".to_string());
        init_global(mapping);
        
        let result = replace_placeholders("/var/log/nginx/access_{{placeholder|:|simple_mapping|:|key1}}.log");
        assert!(result.contains("/var/log/nginx/access_value1.log"));
    }
} 