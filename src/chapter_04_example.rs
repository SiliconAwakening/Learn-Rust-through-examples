// 配置管理工具基础使用示例
use std::collections::HashMap;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    // 创建配置管理器
    let mut manager = create_config_manager()?;
    
    // 加载配置文件
    load_sample_config(&mut manager)?;
    
    // 获取配置值
    let app_name: String = manager.get("app.name")?;
    let server_port: i32 = manager.get("server.port")?;
    
    println!("App: {}, Port: {}", app_name, server_port);
    
    // 设置新配置
    set_sample_config(&mut manager)?;
    
    // 添加验证规则
    add_validation_rules(&mut manager)?;
    
    // 添加监听器
    let watcher = create_watcher();
    manager.add_watcher(watcher);
    
    // 导出配置
    let json = manager.export_json()?;
    println!("Configuration: {}", json);
    
    // 验证配置
    manager.validate_all()?;
    println!("All configurations are valid");
    
    Ok(())
}

fn create_config_manager() -> Result<ConfigManager, Box<dyn Error>> {
    use slog::Drain;
    
    let decorator = slog_term::TermDecorator::new().stdout().build();
    let drain = slog_term::FullFormat::new(decorator).build().fuse();
    let drain = slog_async::Async::new(drain).build().fuse();
    let logger = slog::Logger::root(drain, slog::o!());
    
    Ok(ConfigManager::new(logger))
}

fn load_sample_config(manager: &mut ConfigManager) -> Result<(), Box<dyn Error>> {
    // 创建示例配置内容
    let config_content = r#"
{
    "app": {
        "name": "MyApp",
        "version": "1.0.0",
        "description": "A sample application"
    },
    "server": {
        "port": 8080,
        "host": "localhost",
        "ssl": true
    },
    "database": {
        "url": "postgresql://localhost:5432/mydb",
        "max_connections": 10,
        "timeout": 30000
    }
}
"#;
    
    // 解析JSON配置
    use serde_json::Value as JsonValue;
    let config: JsonValue = serde_json::from_str(config_content)?;
    let mut configs = HashMap::new();
    
    // 解析配置结构
    parse_json_to_configs(&config, "", &mut configs)?;
    
    // 更新配置
    manager.update_configurations(configs)?;
    
    Ok(())
}

fn parse_json_to_configs(
    value: &JsonValue,
    prefix: &str,
    configs: &mut HashMap<String, ConfigValue>,
) -> Result<(), Box<dyn Error>> {
    use serde_json::Value as JsonValue;
    
    if let JsonValue::Object(obj) = value {
        for (key, val) in obj {
            let full_key = if prefix.is_empty() {
                key.clone()
            } else {
                format!("{}.{}", prefix, key)
            };
            
            match val {
                JsonValue::Object(_) => {
                    parse_json_to_configs(&val, &full_key, configs)?;
                }
                JsonValue::Array(arr) => {
                    if let Some(first) = arr.first() {
                        let data_type = DataType::from_json_value(first);
                        let config_value = ConfigValue::new(val.clone(), data_type, false);
                        configs.insert(full_key, config_value);
                    }
                }
                _ => {
                    let data_type = DataType::from_json_value(&val);
                    let config_value = ConfigValue::new(val.clone(), data_type, false);
                    configs.insert(full_key, config_value);
                }
            }
        }
    }
    
    Ok(())
}

fn set_sample_config(manager: &mut ConfigManager) -> Result<(), Box<dyn Error>> {
    // 设置环境配置
    let env_config = ConfigValue::new(
        JsonValue::String("production".to_string()),
        DataType::String,
        false,
    );
    manager.set("app.environment".to_string(), env_config)?;
    
    // 设置API密钥
    let api_key_config = ConfigValue::new(
        JsonValue::String("secret_key_12345".to_string()),
        DataType::String,
        true,
    );
    manager.set("app.api_key".to_string(), api_key_config)?;
    
    // 设置日志级别
    let log_level_config = ConfigValue::new(
        JsonValue::String("info".to_string()),
        DataType::String,
        false,
    );
    manager.set("logging.level".to_string(), log_level_config)?;
    
    println!("Sample configurations added");
    Ok(())
}

fn add_validation_rules(manager: &mut ConfigManager) -> Result<(), Box<dyn Error>> {
    // 创建端口验证
    let mut port_config = ConfigValue::new(
        JsonValue::Number(8080.into()),
        DataType::Integer,
        true,
    );
    port_config.validation_rules.push(ValidationRule::MinValue(1));
    port_config.validation_rules.push(ValidationRule::MaxValue(65535));
    manager.set("server.port".to_string(), port_config)?;
    
    // 创建URL验证
    let mut url_config = ConfigValue::new(
        JsonValue::String("postgresql://localhost:5432/mydb".to_string()),
        DataType::String,
        true,
    );
    url_config.validation_rules.push(ValidationRule::Pattern(r"^[a-z]+://.*$".to_string()));
    manager.set("database.url".to_string(), url_config)?;
    
    // 创建API密钥验证
    let mut api_key_config = ConfigValue::new(
        JsonValue::String("secret_key_12345".to_string()),
        DataType::String,
        true,
    );
    api_key_config.validation_rules.push(ValidationRule::MinLength(10));
    manager.set("app.api_key".to_string(), api_key_config)?;
    
    println!("Validation rules added");
    Ok(())
}

fn create_watcher() -> Box<dyn ConfigWatcher> {
    struct SimpleWatcher;
    
    impl ConfigWatcher for SimpleWatcher {
        fn on_config_change(&self, key: &str, new_value: &ConfigValue) {
            println!("🔧 Config changed: {} = {:?}", key, new_value.value);
        }
        
        fn on_config_removed(&self, key: &str) {
            println!("🗑️  Config removed: {}", key);
        }
        
        fn on_validation_error(&self, key: &str, error: &ValidationError) {
            println!("❌ Validation error in {}: {}", key, error);
        }
    }
    
    Box::new(SimpleWatcher)
}

// 这里需要包含所有相关的类型定义
// 在实际项目中，这些会在不同的模块中定义

// 简化的类型定义用于示例
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
    
    pub fn update_configurations(&mut self, configs: std::collections::HashMap<String, ConfigValue>) -> Result<(), Box<dyn Error>> {
        self.configs.extend(configs);
        Ok(())
    }
    
    pub fn get<T>(&self, key: &str) -> Result<T, Box<dyn Error>>
    where
        T: serde::de::DeserializeOwned,
    {
        let config_value = self.configs.get(key)
            .ok_or_else(|| format!("Key not found: {}", key))?;
        
        let value: T = serde_json::from_value(config_value.value.clone())?;
        Ok(value)
    }
    
    pub fn set(&mut self, key: String, value: ConfigValue) -> Result<(), Box<dyn Error>> {
        self.configs.insert(key, value);
        Ok(())
    }
    
    pub fn add_watcher(&mut self, _watcher: Box<dyn ConfigWatcher>) {
        // 在真实实现中会存储监听器
    }
    
    pub fn export_json(&self) -> Result<String, Box<dyn Error>> {
        let export_data: std::collections::HashMap<String, JsonValue> = self.configs
            .iter()
            .map(|(k, v)| (k.clone(), v.value.clone()))
            .collect();
        
        Ok(serde_json::to_string_pretty(&export_data)?)
    }
    
    pub fn validate_all(&self) -> Result<(), Box<dyn Error>> {
        for (key, config) in &self.configs {
            if let Err(error) = config.validate() {
                return Err(format!("Validation error in {}: {}", key, error).into());
            }
        }
        Ok(())
    }
}

// 其他必要的类型定义
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
    pub fn apply(&self, value: &JsonValue) -> Result<(), Box<dyn Error>> {
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
    
    pub fn validate(&self) -> Result<(), Box<dyn Error>> {
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