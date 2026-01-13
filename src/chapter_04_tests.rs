// 配置管理工具测试文件
#[cfg(test)]
mod config_tests {
    use super::*;
    use std::collections::HashMap;
    use serde_json::Value as JsonValue;

    #[test]
    fn test_config_value_creation() {
        let value = JsonValue::String("test".to_string());
        let config = ConfigValue::new(value.clone(), DataType::String, false);
        
        assert_eq!(config.value, value);
        assert_eq!(config.data_type, DataType::String);
        assert!(!config.required);
        assert!(config.validation_rules.is_empty());
    }

    #[test]
    fn test_config_validation_required() {
        let value = JsonValue::Null;
        let mut config = ConfigValue::new(value, DataType::String, true);
        config.description = "Test field".to_string();
        
        let result = config.validate();
        assert!(result.is_err());
    }

    #[test]
    fn test_config_validation_min_max() {
        let value = JsonValue::Number(50.into());
        let mut config = ConfigValue::new(value, DataType::Integer, false);
        config.validation_rules.push(ValidationRule::MinValue(0));
        config.validation_rules.push(ValidationRule::MaxValue(100));
        
        assert!(config.validate().is_ok());
        
        // 测试超出范围
        let value = JsonValue::Number(150.into());
        let mut config = ConfigValue::new(value, DataType::Integer, false);
        config.validation_rules.push(ValidationRule::MinValue(0));
        config.validation_rules.push(ValidationRule::MaxValue(100));
        
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_config_validation_pattern() {
        let value = JsonValue::String("test@example.com".to_string());
        let mut config = ConfigValue::new(value, DataType::String, false);
        config.validation_rules.push(ValidationRule::Pattern(r"^[^@]+@[^@]+\.[^@]+$".to_string()));
        
        assert!(config.validate().is_ok());
        
        // 测试不匹配的模式
        let value = JsonValue::String("invalid-email".to_string());
        let mut config = ConfigValue::new(value, DataType::String, false);
        config.validation_rules.push(ValidationRule::Pattern(r"^[^@]+@[^@]+\.[^@]+$".to_string()));
        
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_data_type_from_json() {
        let json_str = JsonValue::String("test".to_string());
        assert_eq!(DataType::from_json_value(&json_str), DataType::String);
        
        let json_num = JsonValue::Number(42.into());
        assert_eq!(DataType::from_json_value(&json_num), DataType::Integer);
        
        let json_bool = JsonValue::Bool(true);
        assert_eq!(DataType::from_json_value(&json_bool), DataType::Boolean);
    }

    #[test]
    fn test_config_manager_basic_operations() {
        let mut manager = create_test_manager();
        
        // 测试设置配置
        let value = ConfigValue::new(
            JsonValue::String("test".to_string()),
            DataType::String,
            false,
        );
        manager.set("test.key".to_string(), value).unwrap();
        
        // 测试获取配置
        let retrieved: String = manager.get("test.key").unwrap();
        assert_eq!(retrieved, "test");
        
        // 测试检查存在
        assert!(manager.has("test.key"));
        assert!(!manager.has("nonexistent.key"));
    }

    #[test]
    fn test_config_manager_export_import() {
        let mut manager = create_test_manager();
        
        // 添加一些配置
        let configs = create_test_configs();
        manager.update_configurations(configs).unwrap();
        
        // 导出配置
        let json = manager.export_json().unwrap();
        let exported: HashMap<String, JsonValue> = serde_json::from_str(&json).unwrap();
        
        assert!(exported.contains_key("app.name"));
        assert!(exported.contains_key("server.port"));
        
        // 验证值
        if let Some(name) = exported.get("app.name") {
            assert_eq!(name, "TestApp");
        }
    }

    #[test]
    fn test_config_validation_integration() {
        let mut manager = create_test_manager();
        
        // 添加有效配置
        let port_config = ConfigValue::new(
            JsonValue::Number(8080.into()),
            DataType::Integer,
            true,
        );
        port_config.validation_rules.push(ValidationRule::MinValue(1));
        port_config.validation_rules.push(ValidationRule::MaxValue(65535));
        
        manager.set("server.port".to_string(), port_config).unwrap();
        
        // 验证应该成功
        assert!(manager.validate_all().is_ok());
    }

    #[test]
    fn test_hot_reload_simulation() {
        let mut manager = create_test_manager();
        
        // 初始配置
        let initial_config = JsonValue::String("v1".to_string());
        let config = ConfigValue::new(initial_config, DataType::String, false);
        manager.set("app.version".to_string(), config).unwrap();
        
        // 模拟更新
        let new_config = JsonValue::String("v2".to_string());
        let config = ConfigValue::new(new_config, DataType::String, false);
        manager.set("app.version".to_string(), config).unwrap();
        
        // 验证更新
        let version: String = manager.get("app.version").unwrap();
        assert_eq!(version, "v2");
    }

    // 辅助函数
    fn create_test_manager() -> ConfigManager {
        use slog::Drain;
        use slog_async::Async;
        use slog_term::{FullFormat, TermDecorator};
        
        let decorator = TermDecorator::new().stdout().build();
        let drain = FullFormat::new(decorator).build().fuse();
        let drain = Async::new(drain).build().fuse();
        let logger = slog::Logger::root(drain, slog::o!());
        
        ConfigManager::new(logger)
    }

    fn create_test_configs() -> HashMap<String, ConfigValue> {
        let mut configs = HashMap::new();
        
        configs.insert(
            "app.name".to_string(),
            ConfigValue::new(
                JsonValue::String("TestApp".to_string()),
                DataType::String,
                false,
            )
        );
        
        configs.insert(
            "server.port".to_string(),
            ConfigValue::new(
                JsonValue::Number(3000.into()),
                DataType::Integer,
                true,
            )
        );
        
        configs.insert(
            "server.ssl".to_string(),
            ConfigValue::new(
                JsonValue::Bool(true),
                DataType::Boolean,
                false,
            )
        );
        
        configs
    }
}

#[cfg(test)]
mod validation_tests {
    use super::*;
    use serde_json::Value as JsonValue;

    #[test]
    fn test_min_value_validation() {
        let rule = ValidationRule::MinValue(10);
        
        // 有效值
        let value = JsonValue::Number(15.into());
        assert!(rule.apply(&value).is_ok());
        
        // 无效值
        let value = JsonValue::Number(5.into());
        assert!(rule.apply(&value).is_err());
    }

    #[test]
    fn test_max_value_validation() {
        let rule = ValidationRule::MaxValue(100);
        
        // 有效值
        let value = JsonValue::Number(50.into());
        assert!(rule.apply(&value).is_ok());
        
        // 无效值
        let value = JsonValue::Number(150.into());
        assert!(rule.apply(&value).is_err());
    }

    #[test]
    fn test_length_validation() {
        let min_rule = ValidationRule::MinLength(3);
        let max_rule = ValidationRule::MaxLength(10);
        
        // 有效长度
        let value = JsonValue::String("hello".to_string());
        assert!(min_rule.apply(&value).is_ok());
        assert!(max_rule.apply(&value).is_ok());
        
        // 太短
        let value = JsonValue::String("hi".to_string());
        assert!(min_rule.apply(&value).is_err());
        
        // 太长
        let value = JsonValue::String("verylongstring".to_string());
        assert!(max_rule.apply(&value).is_err());
    }

    #[test]
    fn test_pattern_validation() {
        let rule = ValidationRule::Pattern(r"^\d{3}-\d{3}-\d{4}$".to_string());
        
        // 匹配模式
        let value = JsonValue::String("123-456-7890".to_string());
        assert!(rule.apply(&value).is_ok());
        
        // 不匹配模式
        let value = JsonValue::String("invalid".to_string());
        assert!(rule.apply(&value).is_err());
    }

    #[test]
    fn test_required_validation() {
        let rule = ValidationRule::Required;
        
        // 非空值
        let value = JsonValue::String("test".to_string());
        assert!(rule.apply(&value).is_ok());
        
        // 空值
        let value = JsonValue::Null;
        assert!(rule.apply(&value).is_err());
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::fs;

    #[test]
    fn test_file_loading_json() {
        let config_content = r#"{
    "app": {
        "name": "TestApp",
        "port": 8080
    }
}"#;
        
        let temp_file = NamedTempFile::new().unwrap();
        fs::write(temp_file.path(), config_content).unwrap();
        
        let mut manager = create_test_manager();
        let result = manager.load_from_file(temp_file.path());
        
        assert!(result.is_ok());
    }

    #[test]
    fn test_file_loading_yaml() {
        let config_content = r#"
app:
  name: TestApp
  port: 8080
"#;
        
        let temp_file = NamedTempFile::new().unwrap();
        fs::write(temp_file.path(), config_content).unwrap();
        
        let mut manager = create_test_manager();
        let result = manager.load_from_file(temp_file.path());
        
        assert!(result.is_ok());
    }

    #[test]
    fn test_file_loading_toml() {
        let config_content = r#"
[app]
name = "TestApp"
port = 8080
"#;
        
        let temp_file = NamedTempFile::new().unwrap();
        fs::write(temp_file.path(), config_content).unwrap();
        
        let mut manager = create_test_manager();
        let result = manager.load_from_file(temp_file.path());
        
        assert!(result.is_ok());
    }

    #[test]
    fn test_invalid_file_format() {
        let config_content = "invalid content that is not valid JSON, YAML, or TOML";
        
        let temp_file = NamedTempFile::new().unwrap();
        fs::write(temp_file.path(), config_content).unwrap();
        
        let mut manager = create_test_manager();
        let result = manager.load_from_file(temp_file.path());
        
        assert!(result.is_err());
    }

    #[test]
    fn test_nonexistent_file() {
        let mut manager = create_test_manager();
        let result = manager.load_from_file("/nonexistent/file.json");
        
        assert!(result.is_err());
    }

    // 辅助函数
    fn create_test_manager() -> ConfigManager {
        use slog::Drain;
        use slog_async::Async;
        use slog_term::{FullFormat, TermDecorator};
        
        let decorator = TermDecorator::new().stdout().build();
        let drain = FullFormat::new(decorator).build().fuse();
        let drain = Async::new(drain).build().fuse();
        let logger = slog::Logger::root(drain, slog::o!());
        
        ConfigManager::new(logger)
    }
}

#[cfg(test)]
mod performance_tests {
    use super::*;
    use std::time::{Duration, Instant};
    use std::collections::HashMap;

    #[test]
    fn test_large_config_loading() {
        let start = Instant::now();
        let mut manager = create_test_manager();
        
        // 创建大量配置项
        let mut configs = HashMap::new();
        for i in 0..1000 {
            let key = format!("config.item_{}", i);
            let value = ConfigValue::new(
                JsonValue::String(format!("value_{}", i)),
                DataType::String,
                false,
            );
            configs.insert(key, value);
        }
        
        let update_start = Instant::now();
        manager.update_configurations(configs).unwrap();
        let update_duration = update_start.elapsed();
        
        let validation_start = Instant::now();
        manager.validate_all().unwrap();
        let validation_duration = validation_start.elapsed();
        
        let total_duration = start.elapsed();
        
        println!("Total time: {:?}", total_duration);
        println!("Update time: {:?}", update_duration);
        println!("Validation time: {:?}", validation_duration);
        
        assert!(total_duration < Duration::from_millis(1000));
        assert!(update_duration < Duration::from_millis(500));
        assert!(validation_duration < Duration::from_millis(500));
    }

    #[test]
    fn test_concurrent_operations() {
        use std::sync::{Arc, Mutex};
        use std::thread;
        
        let manager = Arc::new(create_test_manager());
        let errors = Arc::new(Mutex::new(Vec::new()));
        
        // 启动多个线程并发操作
        let mut handles = Vec::new();
        for i in 0..10 {
            let manager = Arc::clone(&manager);
            let errors = Arc::clone(&errors);
            
            let handle = thread::spawn(move || {
                let result = thread::sleep(Duration::from_millis(i * 10));
                let mut local_errors = Vec::new();
                
                // 并发读取配置
                for j in 0..100 {
                    let key = format!("concurrent.key_{}", j);
                    if let Err(e) = manager.get::<String>(&key) {
                        local_errors.push(e);
                    }
                }
                
                let mut errors = errors.lock().unwrap();
                errors.extend(local_errors);
            });
            
            handles.push(handle);
        }
        
        // 等待所有线程完成
        for handle in handles {
            handle.join().unwrap();
        }
        
        let errors = errors.lock().unwrap();
        assert!(errors.is_empty(), "Concurrent operations should not cause errors");
    }

    // 辅助函数
    fn create_test_manager() -> ConfigManager {
        use slog::Drain;
        use slog_async::Async;
        use slog_term::{FullFormat, TermDecorator};
        
        let decorator = TermDecorator::new().stdout().build();
        let drain = FullFormat::new(decorator).build().fuse();
        let drain = Async::new(drain).build().fuse();
        let logger = slog::Logger::root(drain, slog::o!());
        
        ConfigManager::new(logger)
    }
}

// 性能基准测试
#[cfg(test)]
mod benchmarks {
    use super::*;
    use criterion::{black_box, criterion_group, criterion_main, Criterion};

    fn bench_config_get(c: &mut Criterion) {
        let mut manager = create_test_manager();
        
        // 预填充数据
        let value = ConfigValue::new(
            JsonValue::String("benchmark".to_string()),
            DataType::String,
            false,
        );
        manager.set("benchmark.key".to_string(), value).unwrap();
        
        c.bench_function("config_get", |b| {
            b.iter(|| {
                let _result = manager.get::<String>(black_box("benchmark.key"));
            })
        });
    }

    fn bench_config_set(c: &mut Criterion) {
        let mut manager = create_test_manager();
        
        c.bench_function("config_set", |b| {
            b.iter(|| {
                let value = ConfigValue::new(
                    JsonValue::String("test".to_string()),
                    DataType::String,
                    false,
                );
                let _result = manager.set(black_box("test.key".to_string()), value);
            })
        });
    }

    fn bench_validation(c: &mut Criterion) {
        let config = create_validating_config();
        
        c.bench_function("config_validation", |b| {
            b.iter(|| {
                let _result = config.validate();
            })
        });
    }

    // 辅助函数
    fn create_test_manager() -> ConfigManager {
        use slog::Drain;
        use slog_async::Async;
        use slog_term::{FullFormat, TermDecorator};
        
        let decorator = TermDecorator::new().stdout().build();
        let drain = FullFormat::new(decorator).build().fuse();
        let drain = Async::new(drain).build().fuse();
        let logger = slog::Logger::root(drain, slog::o!());
        
        ConfigManager::new(logger)
    }

    fn create_validating_config() -> ConfigValue {
        let value = JsonValue::String("test@example.com".to_string());
        let mut config = ConfigValue::new(value, DataType::String, true);
        config.validation_rules.push(ValidationRule::MinLength(1));
        config.validation_rules.push(ValidationRule::Pattern(r"^[^@]+@[^@]+\.[^@]+$".to_string()));
        config
    }

    criterion_group!(benches, bench_config_get, bench_config_set, bench_validation);
    criterion_main!(benches);
}

// 辅助函数实现
pub trait ConfigWatcher: Send + Sync {
    fn on_config_change(&self, key: &str, new_value: &ConfigValue);
    fn on_config_removed(&self, key: &str);
    fn on_validation_error(&self, key: &str, error: &ValidationError);
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum DataType {
    String,
    Integer,
    Float,
    Boolean,
    Array(Box<DataType>),
    Object,
    Custom(String),
}

impl DataType {
    pub fn from_json_value(value: &JsonValue) -> Self {
        match value {
            JsonValue::String(_) => DataType::String,
            JsonValue::Number(n) if n.is_i64() => DataType::Integer,
            JsonValue::Number(n) if n.is_f64() => DataType::Float,
            JsonValue::Bool(_) => DataType::Boolean,
            JsonValue::Array(_) => DataType::Object,
            JsonValue::Object(_) => DataType::Object,
            _ => DataType::String,
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum ValidationRule {
    MinValue(i64),
    MaxValue(i64),
    MinLength(usize),
    MaxLength(usize),
    Pattern(String),
    Required,
    Custom(String),
}

impl ValidationRule {
    pub fn apply(&self, value: &JsonValue) -> Result<(), Box<dyn std::error::Error>> {
        match self {
            ValidationRule::MinValue(min) => {
                if let Some(num) = value.as_i64() {
                    if num < *min {
                        return Err(format!("Expected value >= {}, got {}", min, num).into());
                    }
                }
            }
            ValidationRule::MaxValue(max) => {
                if let Some(num) = value.as_i64() {
                    if num > *max {
                        return Err(format!("Expected value <= {}, got {}", max, num).into());
                    }
                }
            }
            ValidationRule::MinLength(min) => {
                if let Some(text) = value.as_str() {
                    if text.len() < *min {
                        return Err(format!("Expected length >= {}, got {}", min, text.len()).into());
                    }
                }
            }
            ValidationRule::MaxLength(max) => {
                if let Some(text) = value.as_str() {
                    if text.len() > *max {
                        return Err(format!("Expected length <= {}, got {}", max, text.len()).into());
                    }
                }
            }
            ValidationRule::Pattern(pattern) => {
                if let Some(text) = value.as_str() {
                    let regex = regex::Regex::new(pattern)
                        .map_err(|e| format!("Invalid pattern: {}", e))?;
                    if !regex.is_match(text) {
                        return Err(format!("Pattern mismatch: expected {}, got {}", pattern, text).into());
                    }
                }
            }
            ValidationRule::Required => {
                if value.is_null() {
                    return Err("Value is required".to_string().into());
                }
            }
            ValidationRule::Custom(_) => {
                // 自定义验证逻辑
            }
        }
        
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ConfigValue {
    pub value: JsonValue,
    pub data_type: DataType,
    pub required: bool,
    pub validation_rules: Vec<ValidationRule>,
    pub description: String,
    pub default_value: Option<JsonValue>,
    pub env_override: Option<String>,
    pub depends_on: Option<String>,
}

impl ConfigValue {
    pub fn new(
        value: JsonValue,
        data_type: DataType,
        required: bool,
    ) -> Self {
        Self {
            value: value.clone(),
            data_type,
            required,
            validation_rules: vec![],
            description: String::new(),
            default_value: None,
            env_override: None,
            depends_on: None,
        }
    }
    
    pub fn validate(&self) -> Result<(), Box<dyn std::error::Error>> {
        // 检查必需值
        if self.required && self.value.is_null() {
            if let Some(default) = &self.default_value {
                return Ok(());
            }
            return Err("Required configuration value is missing".to_string().into());
        }
        
        // 应用验证规则
        for rule in &self.validation_rules {
            rule.apply(&self.value)?;
        }
        
        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ValidationError {
    #[error("Required value missing: {0}")]
    Required(String),
    
    #[error("Expected value >= {0}, got {1}")]
    MinValue(i64, i64),
    
    #[error("Expected value <= {0}, got {1}")]
    MaxValue(i64, i64),
    
    #[error("Expected length >= {0}, got {1}")]
    MinLength(usize, usize),
    
    #[error("Expected length <= {0}, got {1}")]
    MaxLength(usize, usize),
    
    #[error("Pattern mismatch: expected {0}, got {1}")]
    PatternMismatch(String, String),
    
    #[error("Invalid pattern: {0}")]
    InvalidPattern(String),
    
    #[error("Type mismatch: {0}")]
    TypeMismatch(String),
}

pub struct ConfigManager {
    configs: std::collections::HashMap<String, ConfigValue>,
    logger: slog::Logger,
}

impl ConfigManager {
    pub fn new(logger: slog::Logger) -> Self {
        Self {
            configs: std::collections::HashMap::new(),
            logger,
        }
    }
    
    pub fn load_from_file(&mut self, _path: &std::path::Path) -> Result<(), Box<dyn std::error::Error>> {
        // 简化实现
        Ok(())
    }
    
    pub fn update_configurations(&mut self, configs: std::collections::HashMap<String, ConfigValue>) -> Result<(), Box<dyn std::error::Error>> {
        self.configs.extend(configs);
        Ok(())
    }
    
    pub fn get<T>(&self, key: &str) -> Result<T, Box<dyn std::error::Error>>
    where
        T: serde::de::DeserializeOwned,
    {
        let config_value = self.configs.get(key)
            .ok_or_else(|| format!("Key not found: {}", key))?;
        
        let value: T = serde_json::from_value(config_value.value.clone())?;
        Ok(value)
    }
    
    pub fn set(&mut self, key: String, value: ConfigValue) -> Result<(), Box<dyn std::error::Error>> {
        self.configs.insert(key, value);
        Ok(())
    }
    
    pub fn has(&self, key: &str) -> bool {
        self.configs.contains_key(key)
    }
    
    pub fn export_json(&self) -> Result<String, Box<dyn std::error::Error>> {
        let export_data: std::collections::HashMap<String, JsonValue> = self.configs
            .iter()
            .map(|(k, v)| (k.clone(), v.value.clone()))
            .collect();
        
        Ok(serde_json::to_string_pretty(&export_data)?)
    }
    
    pub fn validate_all(&self) -> Result<(), Box<dyn std::error::Error>> {
        for (key, config) in &self.configs {
            if let Err(error) = config.validate() {
                return Err(format!("Validation error in {}: {}", key, error).into());
            }
        }
        Ok(())
    }
}