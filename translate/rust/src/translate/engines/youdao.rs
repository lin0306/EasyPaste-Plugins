use crate::translate::loader;
use crate::translate::models::TranslateConfig;
use log::info;
use serde_json::Value;
use crate::utils;

pub async fn translate(text: &str, config: TranslateConfig) -> String {
    info!("有道翻译请求: {} -> {}", text, config.target_language);

    // 获取当前引擎的API key
    let api_key = loader::get_api_key_for_engine(&config.translation_engine);

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
    let sign = utils::sha256::digest(sign);

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
