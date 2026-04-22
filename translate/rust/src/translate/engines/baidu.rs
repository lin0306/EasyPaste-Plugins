use log::info;
use serde_json::Value;
use crate::translate::loader;
use crate::translate::models::TranslateConfig;

pub async fn translate(text: &str, config: TranslateConfig) -> String {
    info!("百度翻译请求: {} -> {}", text, config.target_language);
    // 获取当前引擎的API key
    let api_key = loader::get_api_key_for_engine(&config.translation_engine);

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