use log::info;
use serde_json::Value;
use crate::translate::loader;
use crate::translate::models::TranslateConfig;

pub async fn translate(text: &str, config: TranslateConfig) -> String {
    info!("DeepL 翻译请求: {} -> {}", text, config.target_language);
    
    // 获取当前引擎的API key
    let api_key = loader::get_api_key_for_engine(&config.translation_engine);

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