use crate::translate::{ApiKeysConfig, EngineConfig};

impl Default for EngineConfig {
    fn default() -> Self {
        Self {
            translation_engine: "baidu".to_string(),
        }
    }
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