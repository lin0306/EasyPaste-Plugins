use log::info;
use serde_json::Value;
use crate::translate::loader;
use crate::translate::models::TranslateConfig;

pub async fn translate(text: &str, config: TranslateConfig) -> String {
    info!("Google 翻译请求: {} -> {}", text, config.target_language);

    // 获取当前引擎的API key
    let api_key = loader::get_api_key_for_engine(&config.translation_engine);

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