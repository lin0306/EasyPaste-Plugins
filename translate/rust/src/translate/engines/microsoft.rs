use crate::translate::models::BingConfig;
use crate::translate::TranslateConfig;
use log::info;
use reqwest::header::{HeaderMap, HeaderValue, REFERER, USER_AGENT};
use std::collections::HashMap;

pub async fn translate(text: &str, config: TranslateConfig) -> String {
    info!("微软翻译请求: {} -> {}", text, config.target_language);

    let client = reqwest::Client::builder()
        .cookie_store(true) // 开启 Cookie 管理，这对保持会话很重要
        .build();
    if !client.is_ok() {
        return "[微软翻译] 翻译引擎初始化失败".to_string();
    }
    let client = client.expect("翻译引擎初始化失败");

    // 1. 自动获取配置
    info!("正在获取参数...");
    let bing_config = fetch_bing_config(&client).await;
    if !bing_config.is_ok() {
        return "[微软翻译] 翻译引擎参数初始化失败".to_string();
    }
    let bing_config = bing_config.expect("翻译引擎参数初始化失败");
    info!("获取成功: {:?}", bing_config);

    // 2. 使用获取到的参数进行翻译
    let translated_text = translate_text(&client, config, &bing_config, text).await;
    if !translated_text.is_ok() {
        return "[微软翻译] 获取翻译内容失败".to_string();
    }
    let translated_text = translated_text.expect("获取翻译内容失败");
    info!("最后的翻译结果为: {}", translated_text);

    translated_text
}

/// 获取参数
async fn fetch_bing_config(
    client: &reqwest::Client,
) -> Result<BingConfig, Box<dyn std::error::Error>> {
    let url = "https://cn.bing.com/translator";
    let response = client.get(url).send().await?;
    let body = response.text().await?;

    // 一次性提取 IG、key 和 token
    let mut ig = String::new();
    let mut key = String::new();
    let mut token = String::new();

    // 提取 IG - 使用更精确的匹配，避免匹配到小写的 ig
    if let Some(start) = body.find(r#",IG:"#) {
        let ig_start = start + 5; // 跳过 ,IG:"
        if let Some(ig_end) = body[ig_start..].find('"') {
            ig = body[ig_start..ig_start + ig_end].to_string();
            info!("IG 值: {}", ig);
        }
    }

    // 提取 params_AbusePreventionHelper
    if let Some(start) = body.find("params_AbusePreventionHelper = [") {
        let data_start = start + 32; // 跳过 "params_AbusePreventionHelper = ["

        // 找到第一个逗号（key 结束）
        if let Some(comma1) = body[data_start..].find(',') {
            let key_end = data_start + comma1;
            key = body[data_start..key_end].trim().to_string();

            // 找到引号开始（token 开始）
            let after_comma = key_end + 1;
            if let Some(quote_start) = body[after_comma..].find('"') {
                let token_start = after_comma + quote_start + 1;

                // 找到引号结束（token 结束）
                if let Some(quote_end) = body[token_start..].find('"') {
                    token = body[token_start..token_start + quote_end].to_string();
                    info!("Key: {}, Token: {}", key, token);
                }
            }
        }
    } else {
        info!("未找到 params_AbusePreventionHelper 参数");
    }

    // 验证是否都获取到了
    if ig.is_empty() || key.is_empty() || token.is_empty() {
        return Err(format!(
            "未能完整获取配置参数 - IG: {}, Key: {}, Token: {}",
            if ig.is_empty() { "缺失" } else { "正常" },
            if key.is_empty() { "缺失" } else { "正常" },
            if token.is_empty() { "缺失" } else { "正常" }
        ).into());
    }

    Ok(BingConfig { ig, key, token })
}

/// 翻译文本
async fn translate_text(
    client: &reqwest::Client,
    config: TranslateConfig,
    bing_config: &BingConfig,
    text: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let url = "https://cn.bing.com/ttranslatev3";

    let mut query = HashMap::new();
    query.insert("isVertical", "1");
    query.insert("IG", &bing_config.ig);
    query.insert("IID", "translator.5026");
    info!("query 请求参数: {:?}", query);

    let mut form = HashMap::new();
    form.insert("fromLang", get_language(config.source_language).to_string());
    form.insert("to", get_language(config.target_language));
    form.insert("text", text.to_string());
    form.insert("tryFetchingGenderDebiasedTranslations", "true".to_string());
    form.insert("token", bing_config.token.clone());
    form.insert("key", bing_config.key.clone());
    info!("form 请求参数: {:?}", form);

    let mut headers = HeaderMap::new();
    headers.insert(USER_AGENT, HeaderValue::from_static("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/147.0.0.0 Safari/537.36"));
    headers.insert(
        REFERER,
        HeaderValue::from_static("https://cn.bing.com/translator"),
    );
    headers.insert("Origin", HeaderValue::from_static("https://cn.bing.com"));
    headers.insert("Accept", HeaderValue::from_static("*/*"));
    headers.insert(
        "Accept-Language",
        HeaderValue::from_static("zh-CN,zh;q=0.9"),
    );

    let req = client.post(url).query(&query).form(&form).headers(headers);

    let res = req.send().await?.json::<serde_json::Value>().await?;

    info!("响应结果: {}", res);
    // 解析 JSON 响应
    let translation = res[0]["translations"][0]["text"]
        .as_str()
        .unwrap_or("")
        .to_string();
    Ok(translation)
}

/// 获取语言
fn get_language(resource_language: String) -> String {
    let target_language = match resource_language.as_str() {
        "auto" => "auto-detect",
        "zh" => "zh-Hans",
        "en" => "en-GB",
        "ja" => "ja",
        "ko" => "ko",
        "fr" => "fr",
        "de" => "de",
        "es" => "es",
        "ru" => "ru",
        _ => "zh",
    };

    target_language.to_string()
}
