// 配置管理工具的解析器模块
use crate::config::value::{ConfigValue, DataType};
use std::path::Path;
use std::collections::HashMap;
use crate::config::value::{ConfigError, JsonValue};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ConfigFormat {
    Json,
    Yaml,
    Toml,
    Custom,
}

impl ConfigFormat {
    pub fn from_file_extension(path: &Path) -> Option<Self> {
        match path.extension()?.to_str()? {
            "json" => Some(ConfigFormat::Json),
            "yaml" | "yml" => Some(ConfigFormat::Yaml),
            "toml" => Some(ConfigFormat::Toml),
            _ => None,
        }
    }
}

pub fn load_config_file(content: &str, format: ConfigFormat) -> Result<HashMap<String, ConfigValue>, ConfigError> {
    match format {
        ConfigFormat::Json => load_from_json(content),
        ConfigFormat::Yaml => load_from_yaml(content),
        ConfigFormat::Toml => load_from_toml(content),
        ConfigFormat::Custom => load_from_custom(content),
    }
}

fn load_from_json(content: &str) -> Result<HashMap<String, ConfigValue>, ConfigError> {
    let json_value: JsonValue = serde_json::from_str(content)?;
    let mut configs = HashMap::new();
    
    parse_json_object(&json_value, "", &mut configs)?;
    Ok(configs)
}

fn load_from_yaml(content: &str) -> Result<HashMap<String, ConfigValue>, ConfigError> {
    let yaml_value: serde_yaml::Value = serde_yaml::from_str(content)?;
    let mut configs = HashMap::new();
    
    parse_yaml_object(&yaml_value, "", &mut configs)?;
    Ok(configs)
}

fn load_from_toml(content: &str) -> Result<HashMap<String, ConfigValue>, ConfigError> {
    let toml_value: toml::Value = toml::from_str(content)?;
    let mut configs = HashMap::new();
    
    parse_toml_object(&toml_value, "", &mut configs)?;
    Ok(configs)
}

fn load_from_custom(content: &str) -> Result<HashMap<String, ConfigValue>, ConfigError> {
    Err(ConfigError::InvalidFormat("Custom format not implemented".to_string()))
}

fn parse_json_object(
    value: &JsonValue,
    prefix: &str,
    configs: &mut HashMap<String, ConfigValue>,
) -> Result<(), ConfigError> {
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
) -> Result<(), ConfigError> {
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
) -> Result<(), ConfigError> {
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