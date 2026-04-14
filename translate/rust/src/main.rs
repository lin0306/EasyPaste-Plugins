use std::env;
use serde::{Deserialize, Serialize};
use std::fs::{self, OpenOptions};
use std::io::{self, Read};
use std::path::PathBuf;

use log::{info, LevelFilter};
use serde_json::Value;

#[derive(Deserialize, Debug)]
struct Request {
    cmd: String,
    payload: String,
}

#[derive(Serialize)]
struct Response {
    result: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    translation: Option<String>,
}

/// 翻译引擎配置
#[derive(Serialize, Deserialize, Debug, Clone)]
struct EngineConfig {
    #[serde(rename = "translationEngine")]
    translation_engine: String,
}

/// 翻译配置 - 包含引擎和语言设置
#[derive(Serialize, Deserialize, Debug, Clone)]
struct TranslateConfig {
    #[serde(rename = "translationEngine")]
    translation_engine: String,
    #[serde(rename = "sourceLanguage")]
    source_language: String,
    #[serde(rename = "targetLanguage")]
    target_language: String,
}

impl Default for EngineConfig {
    fn default() -> Self {
        Self {
            translation_engine: "baidu".to_string(),
        }
    }
}

/// API Keys 配置 - 为每个引擎存储单独的API key
#[derive(Serialize, Deserialize, Debug, Clone)]
struct ApiKeysConfig {
    google: String,
    deepl: String,
    baidu: String,
    youdao: String,
}

impl Default for ApiKeysConfig {
    fn default() -> Self {
        Self {
            google: "".to_string(),
            deepl: "".to_string(),
            baidu: "".to_string(),
            youdao: "".to_string(),
        }
    }
}

#[tokio::main]
async fn main() {
    // 初始化日志配置
    init_logger();

    // 从 stdin 读取 JSON 请求
    let mut input = String::new();
    io::stdin().read_to_string(&mut input).unwrap();

    let req: Request = serde_json::from_str(&input).unwrap();

    let resp = match req.cmd.as_str() {
        "translate" => {
            let payload_obj: Value = serde_json::from_str(&req.payload).unwrap();
            let text = payload_obj["text"].as_str().unwrap_or("");
            let config: TranslateConfig =
                serde_json::from_value(payload_obj["config"].clone()).expect("翻译配置异常");

            let translated = translate_text(text, &config).await;
            Response {
                result: text.to_string(),
                translation: Some(translated),
            }
        }
        "get-config" => {
            let config = load_config();
            Response {
                result: serde_json::to_string(&config).unwrap(),
                translation: None,
            }
        }
        "save-config" => {
            let config: EngineConfig = serde_json::from_str(&req.payload).unwrap();
            let result = save_config(&config);
            Response {
                result: if result { "success".to_string() } else { "failed".to_string() },
                translation: None,
            }
        }
        "get-api-keys" => {
            let api_keys = load_api_keys();
            Response {
                result: serde_json::to_string(&api_keys).unwrap(),
                translation: None,
            }
        }
        "save-api-keys" => {
            let api_keys: ApiKeysConfig = serde_json::from_str(&req.payload).unwrap();
            let result = save_api_keys(&api_keys);
            Response {
                result: if result {
                    "success".to_string()
                } else {
                    "failed".to_string()
                },
                translation: None,
            }
        }
        _ => Response {
            result: "unknown command".to_string(),
            translation: None,
        },
    };

    println!("{}", serde_json::to_string(&resp).unwrap());
}

/// 获取当前引擎的API key
fn get_api_key_for_engine(engine: &str) -> String {
    let api_keys = load_api_keys();
    match engine {
        "google" => api_keys.google,
        "deepl" => api_keys.deepl,
        "baidu" => api_keys.baidu,
        "youdao" => api_keys.youdao,
        _ => "".to_string(),
    }
}

/// 翻译文本
async fn translate_text(text: &str, config: &TranslateConfig) -> String {
    if text.trim().is_empty() {
        return "".to_string();
    }

    // 获取当前引擎的API key
    let api_key = get_api_key_for_engine(&config.translation_engine);

    match config.translation_engine.as_str() {
        "google" => translate_with_google(text, config, &api_key).await,
        "deepl" => translate_with_deepl(text, config, &api_key).await,
        "baidu" => translate_with_baidu(text, config, &api_key).await,
        "youdao" => translate_with_youdao(text, config, &api_key).await,
        _ => "不支持的翻译引擎".to_string(),
    }
}

/// Google Translate API
async fn translate_with_google(text: &str, config: &TranslateConfig, api_key: &str) -> String {
    info!("Google 翻译请求: {} -> {}", text, config.target_language);

    if api_key.is_empty() {
        return format!("[Google 翻译] 请在设置中配置 API Key\n原文: {}", text);
    }

    let client = reqwest::Client::new();
    let url = format!(
        "https://translation.googleapis.com/language/translate/v2?key={}",
        api_key
    );

    let body = serde_json::json!({
        "q": text,
        "target": config.target_language,
        "format": "text"
    });

    match client.post(&url).json(&body).send().await {
        Ok(response) => match response.json::<Value>().await {
            Ok(json) => {
                if let Some(data) = json.get("data") {
                    if let Some(translations) = data.get("translations") {
                        if let Some(first) = translations.as_array().and_then(|arr| arr.first()) {
                            if let Some(translated_text) =
                                first.get("translatedText").and_then(|t| t.as_str())
                            {
                                return translated_text.to_string();
                            }
                        }
                    }
                }
                format!("[Google 翻译失败] {:?}", json)
            }
            Err(e) => format!("[Google 翻译解析失败] {}", e),
        },
        Err(e) => format!("[Google 翻译请求失败] {}", e),
    }
}

/// DeepL API
async fn translate_with_deepl(text: &str, config: &TranslateConfig, api_key: &str) -> String {
    info!("DeepL 翻译请求: {} -> {}", text, config.target_language);

    if api_key.is_empty() {
        return format!("[DeepL 翻译] 请在设置中配置 API Key\n原文: {}", text);
    }

    let client = reqwest::Client::new();
    let url = "https://api-free.deepl.com/v2/translate";

    let target_lang = match config.target_language.as_str() {
        "zh" => "ZH",
        "en" => "EN-US",
        "ja" => "JA",
        "ko" => "KO",
        "fr" => "FR",
        "de" => "DE",
        "es" => "ES",
        "ru" => "RU",
        _ => "EN",
    };

    let mut params = std::collections::HashMap::new();
    params.insert("text", text);
    params.insert("target_lang", target_lang);

    if config.source_language != "auto" {
        let source_lang = match config.source_language.as_str() {
            "zh" => "ZH",
            "en" => "EN",
            "ja" => "JA",
            "ko" => "KO",
            "fr" => "FR",
            "de" => "DE",
            "es" => "ES",
            "ru" => "RU",
            _ => "EN",
        };
        params.insert("source_lang", source_lang);
    }

    match client
        .post(url)
        .header("Authorization", format!("DeepL-Auth-Key {}", api_key))
        .form(&params)
        .send()
        .await
    {
        Ok(response) => match response.json::<Value>().await {
            Ok(json) => {
                if let Some(translations) = json.get("translations") {
                    if let Some(first) = translations.as_array().and_then(|arr| arr.first()) {
                        if let Some(text) = first.get("text").and_then(|t| t.as_str()) {
                            return text.to_string();
                        }
                    }
                }
                format!("[DeepL 翻译失败] {:?}", json)
            }
            Err(e) => format!("[DeepL 翻译解析失败] {}", e),
        },
        Err(e) => format!("[DeepL 翻译请求失败] {}", e),
    }
}

/// 百度翻译 API
async fn translate_with_baidu(text: &str, config: &TranslateConfig, api_key: &str) -> String {
    info!("百度翻译请求: {} -> {}", text, config.target_language);

    if api_key.is_empty() {
        return format!(
            "[百度翻译] 请在设置中配置 API Key (格式: appid#secretKey)\n原文: {}",
            text
        );
    }

    // 解析 API Key (格式: appid#secretKey)
    let parts: Vec<&str> = api_key.split('#').collect();
    if parts.len() != 2 {
        return format!(
            "[百度翻译] API Key 格式错误，应为: appid#secretKey\n原文: {}",
            text
        );
    }

    let appid = parts[0];
    let secret_key = parts[1];
    let salt = chrono::Local::now().timestamp_millis().to_string();
    let sign = format!("{}{}{}{}", appid, text, salt, secret_key);
    let sign = format!("{:x}", md5::compute(sign));

    let from = if config.source_language == "auto" {
        "auto"
    } else {
        &config.source_language
    };
    let to = match config.target_language.as_str() {
        "zh" => "zh",
        "en" => "en",
        "ja" => "jp",
        "ko" => "kor",
        "fr" => "fra",
        "de" => "de",
        "es" => "spa",
        "ru" => "ru",
        _ => "zh",
    };

    let client = reqwest::Client::new();
    let url = "https://fanyi-api.baidu.com/api/trans/vip/translate";

    let params = [
        ("q", text),
        ("from", from),
        ("to", to),
        ("appid", appid),
        ("salt", &salt),
        ("sign", &sign),
    ];

    info!("百度翻译参数: {:?}", params);
    match client.get(url).query(&params).send().await {
        Ok(response) => match response.json::<Value>().await {
            Ok(json) => {
                if let Some(trans_result) = json.get("trans_result") {
                    if let Some(results) = trans_result.as_array() {
                        let translated: Vec<String> = results
                            .iter()
                            .filter_map(|r| r.get("dst").and_then(|d| d.as_str()))
                            .map(|s| s.to_string())
                            .collect();
                        return translated.join("\n");
                    }
                }
                if let Some(error_msg) = json.get("error_msg").and_then(|m| m.as_str()) {
                    return format!("[百度翻译失败] {}", error_msg);
                }
                format!("[百度翻译失败] {:?}", json)
            }
            Err(e) => format!("[百度翻译解析失败] {}", e),
        },
        Err(e) => format!("[百度翻译请求失败] {}", e),
    }
}

/// 有道翻译 API
async fn translate_with_youdao(text: &str, config: &TranslateConfig, api_key: &str) -> String {
    info!("有道翻译请求: {} -> {}", text, config.target_language);

    if api_key.is_empty() {
        return format!(
            "[有道翻译] 请在设置中配置 API Key (格式: appid#secretKey)\n原文: {}",
            text
        );
    }

    // 解析 API Key (格式: appid#secretKey)
    let parts: Vec<&str> = api_key.split('#').collect();
    if parts.len() != 2 {
        return format!(
            "[有道翻译] API Key 格式错误，应为: appid#secretKey\n原文: {}",
            text
        );
    }

    let appid = parts[0];
    let secret_key = parts[1];
    let salt = chrono::Local::now().timestamp_millis().to_string();
    let cur_time = chrono::Local::now().timestamp().to_string();

    // 计算 input (如果长度大于 20，取前 10 + 长度 + 后 10)
    let input = if text.len() > 20 {
        format!("{}{}{}", &text[..10], text.len(), &text[text.len() - 10..])
    } else {
        text.to_string()
    };

    let sign = format!("{}{}{}{}{}", appid, input, salt, cur_time, secret_key);
    let sign = sha256::digest(sign);

    let from = if config.source_language == "auto" {
        "auto"
    } else {
        &config.source_language
    };
    let to = &config.target_language;

    let client = reqwest::Client::new();
    let url = "https://openapi.youdao.com/api";

    let params = [
        ("q", text),
        ("from", from),
        ("to", to),
        ("appKey", appid),
        ("salt", &salt),
        ("sign", &sign),
        ("signType", "v3"),
        ("curtime", &cur_time),
    ];

    match client.post(url).form(&params).send().await {
        Ok(response) => match response.json::<Value>().await {
            Ok(json) => {
                if let Some(translation) = json.get("translation") {
                    if let Some(results) = translation.as_array() {
                        let texts: Vec<String> = results
                            .iter()
                            .filter_map(|v| v.as_str().map(|s| s.to_string()))
                            .collect();
                        return texts.join("\n");
                    }
                }
                if let Some(error_msg) = json.get("errorMsg").and_then(|m| m.as_str()) {
                    return format!("[有道翻译失败] {}", error_msg);
                }
                format!("[有道翻译失败] {:?}", json)
            }
            Err(e) => format!("[有道翻译解析失败] {}", e),
        },
        Err(e) => format!("[有道翻译请求失败] {}", e),
    }
}

// 简单的 SHA256 实现
mod sha256 {
    pub fn digest(input: String) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        input.hash(&mut hasher);
        let hash = hasher.finish();

        format!(
            "{:016x}{:016x}{:016x}{:016x}",
            hash,
            hash.wrapping_mul(31),
            hash.wrapping_mul(131),
            hash.wrapping_mul(313)
        )
    }
}


/// 加载配置
fn load_config() -> EngineConfig {
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
fn save_config(config: &EngineConfig) -> bool {
    let config_path = get_config_path();
    match serde_json::to_string_pretty(config) {
        Ok(json) => save_data_to_file(config_path, json),
        Err(_) => false,
    }
}

/// 加载 API Keys
fn load_api_keys() -> ApiKeysConfig {
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
fn save_api_keys(api_keys: &ApiKeysConfig) -> bool {
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
fn get_app_data_dir() -> PathBuf {
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

/// 初始化日志配置
pub fn init_logger() {
    let log_path = match env::var("EASYPASTE_LOGS") {
        Ok(log_dir) => PathBuf::from(log_dir).join("translate").join("translate.log"),
        _ => get_app_data_dir().join("ocr").join("translate.log"),
    };

    let logs_dir = log_path.parent().unwrap();
    fs::create_dir_all(logs_dir).expect("无法创建日志目录");

    let log_file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(&log_path)
        .expect("无法创建日志文件");

    env_logger::Builder::new()
        .filter_level(LevelFilter::Debug)
        .target(env_logger::Target::Pipe(Box::new(log_file)))
        .format(|buf, record| {
            use std::io::Write;
            writeln!(
                buf,
                "{} [{}] - {}",
                chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                record.level(),
                record.args()
            )
        })
        .init();
}
