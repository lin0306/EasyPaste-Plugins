use std::env;
use std::fs::{self, OpenOptions};
use std::io::{self, Read};
use std::path::PathBuf;

use log::LevelFilter;
use serde_json::Value;
use crate::translate::{get_app_data_dir, load_api_keys, load_config, save_api_keys, save_config, ApiKeysConfig, EngineConfig, Request, Response, TranslateConfig};

mod translate;
mod utils;

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

            let translated = translate::translate_text(text, config).await;
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

/// 初始化日志配置
pub fn init_logger() {
    let log_path = match env::var("EASYPASTE_LOGS") {
        Ok(log_dir) => PathBuf::from(log_dir).join("translate").join("translate.log"),
        _ => get_app_data_dir().join("translate").join("translate.log"),
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
