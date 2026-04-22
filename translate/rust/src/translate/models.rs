use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug)]
pub struct Request {
    pub cmd: String,
    pub payload: String,
}

#[derive(Serialize)]
pub struct Response {
    pub result: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub translation: Option<String>,
}

/// 翻译引擎配置
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EngineConfig {
    #[serde(rename = "translationEngine")]
    pub translation_engine: String,
}

/// 翻译配置 - 包含引擎和语言设置
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TranslateConfig {
    #[serde(rename = "translationEngine")]
    pub translation_engine: String,
    #[serde(rename = "sourceLanguage")]
    pub source_language: String,
    #[serde(rename = "targetLanguage")]
    pub target_language: String,
}

/// API Keys 配置 - 为每个引擎存储单独的API key
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ApiKeysConfig {
    pub google: String,
    pub deepl: String,
    pub baidu: String,
    pub youdao: String,
}