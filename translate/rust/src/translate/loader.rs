use crate::translate::config;

/// 获取当前引擎的API key
pub fn get_api_key_for_engine(engine: &str) -> String {
    let api_keys = config::load_api_keys();
    match engine {
        "google" => api_keys.google,
        "deepl" => api_keys.deepl,
        "baidu" => api_keys.baidu,
        "youdao" => api_keys.youdao,
        _ => "".to_string(),
    }
}
