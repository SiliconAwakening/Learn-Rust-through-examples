// 配置管理工具的主程序
use config_manager::config::manager::ConfigManager;
use config_manager::config::value::{ConfigWatcher, ConfigValue, DataType, ValidationRule, LoggingWatcher};
use config_manager::parsers::ConfigFormat;
use clap::{Arg, Command};
use serde_json::Value as JsonValue;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    // 设置日志
    let logger = create_logger()?;
    
    let matches = Command::new("config-manager")
        .version("1.0")
        .about("Enterprise configuration management tool")
        .subcommand_required(true)
        .arg_required_else_help(true)
        
        .subcommand(
            Command::new("load")
                .about("Load configuration from file")
                .arg(Arg::new("file")
                    .required(true)
                    .help("Configuration file path"))
                .arg(Arg::new("watch")
                    .long("watch")
                    .help("Enable file watching for hot reload"))
        )
        
        .subcommand(
            Command::new("get")
                .about("Get configuration value")
                .arg(Arg::new("key")
                    .required(true)
                    .help("Configuration key"))
        )
        
        .subcommand(
            Command::new("set")
                .about("Set configuration value")
                .arg(Arg::new("key")
                    .required(true)
                    .help("Configuration key"))
                .arg(Arg::new("value")
                    .required(true)
                    .help("Configuration value"))
                .arg(Arg::new("type")
                    .long("type")
                    .value_name("TYPE")
                    .help("Value type (string, integer, float, boolean)")
                    .default_value("string"))
        )
        
        .subcommand(
            Command::new("validate")
                .about("Validate configuration file")
                .arg(Arg::new("file")
                    .required(true)
                    .help("Configuration file to validate"))
        )
        
        .subcommand(
            Command::new("export")
                .about("Export configuration as JSON")
                .arg(Arg::new("output")
                    .long("output")
                    .value_name("FILE")
                    .help("Output file (default: stdout)"))
        )
        
        .get_matches();
    
    let config_manager = Arc::new(ConfigManager::new(logger.clone()));
    
    match matches.subcommand() {
        Some(("load", args)) => {
            let file = args.value_of("file").unwrap();
            let watch = args.is_present("watch");
            
            if !PathBuf::from(file).exists() {
                return Err(format!("Configuration file not found: {}", file).into());
            }
            
            // 创建监听器
            let watcher = Box::new(LoggingWatcher::new(logger.clone()));
            let mut manager = Arc::try_unwrap(config_manager)
                .map_err(|_| "Config manager is in use")?;
            manager.add_watcher(watcher);
            
            manager.load_from_file(file)?;
            println!("Configuration loaded from: {}", file);
            
            if watch {
                println!("Watching for changes... (Press Ctrl+C to stop)");
                // 等待变化
                let change_rx = manager.get_change_receiver().unwrap();
                for change in change_rx {
                    match change.change_type {
                        config_manager::config::manager::ChangeType::Added => {
                            println!("➕ Added: {}", change.key);
                        }
                        config_manager::config::manager::ChangeType::Modified => {
                            println!("✏️  Modified: {}", change.key);
                        }
                        config_manager::config::manager::ChangeType::Removed => {
                            println!("➖ Removed: {}", change.key);
                        }
                    }
                }
            }
        }
        
        Some(("get", args)) => {
            let key = args.value_of("key").unwrap();
            let value = config_manager.get::<JsonValue>(key)
                .map_err(|e| format!("Failed to get config '{}': {}", key, e))?;
            println!("{}", serde_json::to_string_pretty(&value)?);
        }
        
        Some(("set", args)) => {
            let key = args.value_of("key").unwrap();
            let value = args.value_of("value").unwrap();
            let value_type = args.value_of("type").unwrap();
            
            let json_value = match value_type {
                "string" => JsonValue::String(value.to_string()),
                "integer" => {
                    let num: i64 = value.parse()
                        .map_err(|e| format!("Invalid integer: {}", e))?;
                    JsonValue::Number(num.into())
                }
                "float" => {
                    let num: f64 = value.parse()
                        .map_err(|e| format!("Invalid float: {}", e))?;
                    JsonValue::Number(serde_json::Number::from_f64(num).unwrap())
                }
                "boolean" => {
                    let bool_val: bool = value.parse()
                        .map_err(|e| format!("Invalid boolean: {}", e))?;
                    JsonValue::Bool(bool_val)
                }
                _ => return Err(format!("Unknown type: {}", value_type).into()),
            };
            
            let data_type = match value_type {
                "string" => DataType::String,
                "integer" => DataType::Integer,
                "float" => DataType::Float,
                "boolean" => DataType::Boolean,
                _ => DataType::String,
            };
            
            let config_value = ConfigValue::new(json_value, data_type, false);
            config_manager.set(key.to_string(), config_value)
                .map_err(|e| format!("Failed to set config '{}': {}", key, e))?;
            println!("Configuration set: {} = {}", key, value);
        }
        
        Some(("validate", args)) => {
            let file = args.value_of("file").unwrap();
            let content = fs::read_to_string(file)
                .map_err(|e| format!("Failed to read file: {}", e))?;
            
            let format = ConfigFormat::from_file_extension(PathBuf::from(file).as_path())
                .ok_or("Unsupported file format")?;
            
            let configs = load_config_file(&content, format)
                .map_err(|e| format!("Failed to parse configuration: {}", e))?;
            
            // 验证所有配置
            for (key, config) in configs {
                if let Err(error) = config.validate() {
                    println!("❌ Validation failed for '{}': {}", key, error);
                } else {
                    println!("✅ '{}' is valid", key);
                }
            }
            
            println!("Configuration validation completed");
        }
        
        Some(("export", args)) => {
            let output = args.value_of("output");
            let json = config_manager.export_json()
                .map_err(|e| format!("Failed to export configuration: {}", e))?;
            
            if let Some(file) = output {
                fs::write(file, json)
                    .map_err(|e| format!("Failed to write output file: {}", e))?;
                println!("Configuration exported to: {}", file);
            } else {
                println!("{}", json);
            }
        }
        
        _ => unreachable!(),
    }
    
    Ok(())
}

fn create_logger() -> Result<slog::Logger, Box<dyn Error>> {
    use slog::Drain;
    use std::io;
    
    let decorator = slog_term::TermDecorator::new().stdout().build();
    let drain = slog_term::FullFormat::new(decorator).build().fuse();
    let drain = slog_async::Async::new(drain).build().fuse();
    
    Ok(slog::Logger::root(drain, slog::o!()))
}

// 辅助函数 - 需要添加到parsers模块
fn load_config_file(content: &str, format: ConfigFormat) -> Result<HashMap<String, ConfigValue>, config_manager::config::value::ConfigError> {
    match format {
        ConfigFormat::Json => load_from_json(content),
        ConfigFormat::Yaml => load_from_yaml(content),
        ConfigFormat::Toml => load_from_toml(content),
        ConfigFormat::Custom => Err(config_manager::config::value::ConfigError::InvalidFormat("Custom format not implemented".to_string())),
    }
}

fn load_from_json(content: &str) -> Result<HashMap<String, ConfigValue>, config_manager::config::value::ConfigError> {
    let json_value: JsonValue = serde_json::from_str(content)?;
    let mut configs = HashMap::new();
    
    parse_json_object(&json_value, "", &mut configs)?;
    Ok(configs)
}

fn load_from_yaml(content: &str) -> Result<HashMap<String, ConfigValue>, config_manager::config::value::ConfigError> {
    let yaml_value: serde_yaml::Value = serde_yaml::from_str(content)?;
    let mut configs = HashMap::new();
    
    parse_yaml_object(&yaml_value, "", &mut configs)?;
    Ok(configs)
}

fn load_from_toml(content: &str) -> Result<HashMap<String, ConfigValue>, config_manager::config::value::ConfigError> {
    let toml_value: toml::Value = toml::from_str(content)?;
    let mut configs = HashMap::new();
    
    parse_toml_object(&toml_value, "", &mut configs)?;
    Ok(configs)
}

fn parse_json_object(
    value: &JsonValue,
    prefix: &str,
    configs: &mut HashMap<String, ConfigValue>,
) -> Result<(), config_manager::config::value::ConfigError> {
    if let JsonValue::Object(obj) = value {
        for (key, val) in obj {
            let full_key = if prefix.is_empty() {
                key.clone()
            } else {
                format!("{}.{}", prefix, key)
            };
            
            match val {
                JsonValue::Object(_) => {
                    parse_json_object(&val, &full_key, configs)?;
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

fn parse_yaml_object(
    value: &serde_yaml::Value,
    prefix: &str,
    configs: &mut HashMap<String, ConfigValue>,
) -> Result<(), config_manager::config::value::ConfigError> {
    if let serde_yaml::Value::Mapping(mapping) = value {
        for (key, val) in mapping {
            let key_str = key.as_str().unwrap_or_default();
            let full_key = if prefix.is_empty() {
                key_str.to_string()
            } else {
                format!("{}.{}", prefix, key_str)
            };
            
            match val {
                serde_yaml::Value::Mapping(_) => {
                    parse_yaml_object(&val, &full_key, configs)?;
                }
                _ => {
                    let json_value: JsonValue = serde_yaml::from_value(val.clone())?;
                    let data_type = DataType::from_json_value(&json_value);
                    let config_value = ConfigValue::new(json_value, data_type, false);
                    configs.insert(full_key, config_value);
                }
            }
        }
    }
    
    Ok(())
}

fn parse_toml_object(
    value: &toml::Value,
    prefix: &str,
    configs: &mut HashMap<String, ConfigValue>,
) -> Result<(), config_manager::config::value::ConfigError> {
    if let toml::Value::Table(table) = value {
        for (key, val) in table {
            let full_key = if prefix.is_empty() {
                key
            } else {
                format!("{}.{}", prefix, key)
            };
            
            match val {
                toml::Value::Table(_) => {
                    parse_toml_object(&val, &full_key, configs)?;
                }
                toml::Value::Array(arr) => {
                    if let Some(first) = arr.first() {
                        let data_type = match first {
                            toml::Value::String(_) => DataType::Array(Box::new(DataType::String)),
                            toml::Value::Integer(_) => DataType::Array(Box::new(DataType::Integer)),
                            toml::Value::Float(_) => DataType::Array(Box::new(DataType::Float)),
                            toml::Value::Boolean(_) => DataType::Array(Box::new(DataType::Boolean)),
                            _ => DataType::Array(Box::new(DataType::String)),
                        };
                        let json_value: JsonValue = toml::to_value(val.clone())?.into();
                        let config_value = ConfigValue::new(json_value, data_type, false);
                        configs.insert(full_key, config_value);
                    }
                }
                _ => {
                    let json_value: JsonValue = toml::to_value(val.clone())?.into();
                    let data_type = match val {
                        toml::Value::String(_) => DataType::String,
                        toml::Value::Integer(_) => DataType::Integer,
                        toml::Value::Float(_) => DataType::Float,
                        toml::Value::Boolean(_) => DataType::Boolean,
                        _ => DataType::String,
                    };
                    let config_value = ConfigValue::new(json_value, data_type, false);
                    configs.insert(full_key, config_value);
                }
            }
        }
    }
    
    Ok(())
}