mod models;
mod state;
mod loader;
mod config;
mod engines;

pub use models::TranslateConfig;
pub use models::EngineConfig;
pub use models::ApiKeysConfig;
pub use models::Request;
pub use models::Response;
pub use config::*;



/// 翻译
/// # 字段
/// - `text`: 要翻译的文本。
/// - `config`: 翻译配置。
pub async fn translate_text(text: &str, config: TranslateConfig) -> String {
    if text.trim().is_empty() {
        return "".to_string();
    }

    match config.translation_engine.as_str() {
        "google" => engines::google::translate(text, config).await,
        "deepl" => engines::deepl::translate(text, config).await,
        "baidu" => engines::baidu::translate(text, config).await,
        "youdao" => engines::youdao::translate(text, config).await,
        _ => "不支持的翻译引擎".to_string(),
    }
}