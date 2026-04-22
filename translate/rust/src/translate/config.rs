use std::{env, fs};
use std::path::PathBuf;
use crate::translate::{ApiKeysConfig, EngineConfig};

/// 加载配置
pub fn load_config() -> EngineConfig {
    let config_path = get_config_path();

    if !config_path.exists() {
        return EngineConfig::default();
    }

    match fs::read_to_string(&config_path) {
        Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
        Err(_) => EngineConfig::default(),
    }
}

/// 保存配置
pub fn save_config(config: &EngineConfig) -> bool {
    let config_path = get_config_path();
    match serde_json::to_string_pretty(config) {
        Ok(json) => save_data_to_file(config_path, json),
        Err(_) => false,
    }
}


/// 加载 API Keys
pub fn load_api_keys() -> ApiKeysConfig {
    let config_path = get_api_keys_path();

    if !config_path.exists() {
        return ApiKeysConfig::default();
    }

    match fs::read_to_string(&config_path) {
        Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
        Err(_) => ApiKeysConfig::default(),
    }
}

/// 保存 API Keys
pub fn save_api_keys(api_keys: &ApiKeysConfig) -> bool {
    let config_path = get_api_keys_path();
    match serde_json::to_string_pretty(api_keys) {
        Ok(json) => save_data_to_file(config_path, json),
        Err(_) => false,
    }
}

/// 保存数据到文件
fn save_data_to_file(file_path: PathBuf, data: String) -> bool {
    if let Some(parent) = file_path.parent() {
        if !parent.exists() {
            if fs::create_dir_all(parent).is_err() {
                return false;
            }
        }
    }

    fs::write(&file_path, data).is_ok()
}

/// 获取应用数据目录
pub fn get_app_data_dir() -> PathBuf {
    // 首先尝试从环境变量获取（由主应用传递）
    if let Ok(data_dir) = env::var("EASYPASTE_DATA_DIR") {
        let path = PathBuf::from(data_dir).join("plugins").join("translate");
        if !path.exists() {
            let _ = fs::create_dir_all(&path);
        }
        return path;
    }

    // 获取主程序的Identifier
    let result_path = match env::var("EASYPASTE_IDENTIFIER") {
        Ok(identifier) => identifier,
        _ => "com.lin.EasyPaste".to_string(),
    };

    // 回退到系统应用数据目录
    #[cfg(target_os = "windows")]
    {
        if let Some(app_data) = dirs::data_dir() {
            let path = app_data
                .join(result_path)
                .join("plugins")
                .join("translate");
            if !path.exists() {
                let _ = fs::create_dir_all(&path);
            }
            return path;
        }
    }

    #[cfg(target_os = "macos")]
    {
        if let Some(home) = dirs::home_dir() {
            let path = home.join("Library/Application Support").join(result_path).join("plugins/translate");

            if !path.exists() {
                let _ = fs::create_dir_all(&path);
            }
            return path;
        }
    }

    // 最后回退到插件目录
    let exe_path = env::current_exe().unwrap_or_default();
    let exe_dir = exe_path.parent().unwrap_or(std::path::Path::new("."));
    exe_dir.join("config")
}

/// 获取配置文件路径
fn get_config_path() -> PathBuf {
    get_app_data_dir().join("translate-config.json")
}

/// 获取 API Keys 文件路径
fn get_api_keys_path() -> PathBuf {
    get_app_data_dir().join("translate-api-keys.json")
}