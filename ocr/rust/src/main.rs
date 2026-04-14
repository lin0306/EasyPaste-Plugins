use serde::{Deserialize, Serialize};
use std::env;
use std::fs::{self, OpenOptions};
use std::io::{self, Read};
use std::path::PathBuf;

use log::{info, LevelFilter};
use ocr_rs::{DetModel, DetOptions, RecModel};

#[derive(Deserialize, Debug)]
struct Request {
    cmd: String,
    payload: String,
}

#[derive(Serialize)]
struct Response {
    result: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct OcrConfig {
    #[serde(rename = "ocrMode")]
    ocr_mode: String,
}

impl Default for OcrConfig {
    fn default() -> Self {
        Self {
            ocr_mode: "window".to_string(),
        }
    }
}

/// 操作结果响应
#[derive(Serialize)]
struct ActionResponse {
    action: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    width: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    height: Option<u32>,
}

fn main() {
    // 初始化日志配置
    init_logger();

    // 从 stdin 读取 JSON 请求
    let mut input = String::new();
    io::stdin().read_to_string(&mut input).unwrap();

    let req: Request = serde_json::from_str(&input).unwrap();

    let resp = match req.cmd.as_str() {
        "ocr" => {
            let payload_obj: serde_json::Value = serde_json::from_str(&req.payload).unwrap();
            let file_path = payload_obj["filePath"].as_str().unwrap_or("");
            let text = do_ocr(file_path);
            Response { result: text }
        }
        "ocr-image" => {
            // 新的右键菜单命令 - 根据配置决定行为
            let payload_obj: serde_json::Value = serde_json::from_str(&req.payload).unwrap();
            let file_path = payload_obj["filePath"].as_str().unwrap_or("");

            // 加载配置
            let config = load_config();

            if config.ocr_mode == "quick" {
                // 快速模式：直接识别并返回文本，前端负责复制到剪贴板
                info!("OCR 快速模式识别: {}", file_path);

                // 执行 OCR 识别
                match recognize_text(file_path) {
                    Ok(text) => {
                        let result_text = if text.trim().is_empty() {
                            "未能识别出文字".to_string()
                        } else {
                            text.trim().to_string()
                        };

                        let action_resp = ActionResponse {
                            action: "copyToClipboard".to_string(),
                            text: Some(result_text),
                            title: None,
                            width: None,
                            height: None,
                        };
                        Response {
                            result: serde_json::to_string(&action_resp).unwrap(),
                        }
                    }
                    Err(e) => {
                        info!("OCR 识别失败: {}", e);
                        let action_resp = ActionResponse {
                            action: "error".to_string(),
                            text: Some(format!("OCR 识别失败: {}", e)),
                            title: None,
                            width: None,
                            height: None,
                        };
                        Response {
                            result: serde_json::to_string(&action_resp).unwrap(),
                        }
                    }
                }
            } else {
                // 窗口模式：返回打开窗口的指令
                let action_resp = ActionResponse {
                    action: "openWindow".to_string(),
                    text: None,
                    title: Some("图片识别".to_string()),
                    width: Some(800),
                    height: Some(600),
                };
                Response {
                    result: serde_json::to_string(&action_resp).unwrap(),
                }
            }
        }
        "get-config" => {
            let config = load_config();
            Response {
                result: serde_json::to_string(&config).unwrap(),
            }
        }
        "save-config" => {
            let config: OcrConfig = serde_json::from_str(&req.payload).unwrap();
            let result = save_config(&config);
            Response {
                result: if result {
                    "success".to_string()
                } else {
                    "failed".to_string()
                },
            }
        }
        _ => Response {
            result: "unknown command".to_string(),
        },
    };

    println!("{}", serde_json::to_string(&resp).unwrap());
}

/// 执行 OCR 识别
fn do_ocr(file_path: &str) -> String {
    info!("OCR 图片识别，图片路径: {}", file_path);

    // 检查文件是否存在
    if !std::path::Path::new(file_path).exists() {
        return "文件不存在".to_string();
    }

    // 使用 ocrs 进行 OCR 识别
    match recognize_text(file_path) {
        Ok(text) => {
            info!("OCR 识别结果: {}", text);
            if text.trim().is_empty() {
                "未能识别出文字".to_string()
            } else {
                text.trim().to_string()
            }
        }
        Err(e) => {
            info!("OCR 识别失败: {}", e);
            format!("OCR 识别失败: {}\n\n请确保图片格式正确。", e)
        }
    }
}

/// 使用 ocrs 识别图片文字
fn recognize_text(image_path: &str) -> anyhow::Result<String> {
    let exe_path = env::current_exe().expect("获取OCR插件文件地址失败");
    let models_path = exe_path
        .parent()
        .expect("获取OCR程序文件目录失败")
        .parent()
        .expect("获取OCR插件文件目录失败")
        .join("models");
    info!("模型文件目录: {:?}", models_path);
    let det_model_path = models_path.join("PP-OCRv5_mobile_det.mnn");
    info!("检测模型文件路径: {:?}", det_model_path);
    let rec_model_path = models_path.join("PP-OCRv5_mobile_rec.mnn");
    info!("识别模型文件路径: {:?}", rec_model_path);
    let keys_model_path = models_path.join("ppocr_keys_v5.txt");
    info!("字典文件路径: {:?}", keys_model_path);
    if !det_model_path.exists() || !rec_model_path.exists() || !keys_model_path.exists() {
        return Err(anyhow::anyhow!("模型文件不存在，请重新下载"));
    }

    // 创建检测模型
    let det = DetModel::from_file(det_model_path, None)?.with_options(DetOptions::fast());
    info!("检测模型加载成功");

    // 创建识别模型
    let rec = RecModel::from_file(rec_model_path, keys_model_path, None)?;
    info!("识别模型加载成功");

    // 加载图片
    let image = image::open(image_path)?;
    info!("图片加载成功");

    // 检测并裁剪文本区域
    let detections = det.detect_and_crop(&image)?;
    info!("检测结果获取完成");

    // 识别每个文本区域
    let mut all_texts: Vec<String> = Vec::new();
    for (cropped_img, _bbox) in detections {
        let result = rec.recognize(&cropped_img)?;
        all_texts.push(result.text);
    }
    info!("识别结果: {}", all_texts.join("\n"));
    Ok(all_texts.join("\n"))
}

/// 加载配置
fn load_config() -> OcrConfig {
    let config_path = get_config_path();

    if !config_path.exists() {
        return OcrConfig::default();
    }

    match fs::read_to_string(&config_path) {
        Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
        Err(_) => OcrConfig::default(),
    }
}

/// 保存配置
fn save_config(config: &OcrConfig) -> bool {
    let config_path = get_config_path();
    info!("保存配置，路径: {}", config_path.display());

    if let Some(parent) = config_path.parent() {
        if !parent.exists() {
            if fs::create_dir_all(parent).is_err() {
                return false;
            }
        }
    }

    match serde_json::to_string_pretty(config) {
        Ok(json) => fs::write(&config_path, json).is_ok(),
        Err(_) => false,
    }
}

/// 获取应用数据目录
fn get_app_data_dir() -> PathBuf {
    // 首先尝试从环境变量获取（由主应用传递）
    if let Ok(data_dir) = env::var("EASYPASTE_DATA_DIR") {
        let path = PathBuf::from(data_dir).join("plugins").join("ocr");
        if !path.exists() {
            let _ = fs::create_dir_all(&path);
        }
        return path;
    }

    // 获取主程序的Identifier
    let result_path = match env::var("EASYPASTE_IDENTIFIER") {
        Ok(identifier) => identifier,
        _ => "com.lin.EasyPaste".to_string(),
    };

    // 回退到系统应用数据目录
    #[cfg(target_os = "windows")]
    {
        if let Some(app_data) = dirs::data_dir() {
            let path = app_data.join(result_path).join("plugins").join("ocr");
            if !path.exists() {
                let _ = fs::create_dir_all(&path);
            }
            return path;
        }
    }

    #[cfg(target_os = "macos")]
    {
        if let Some(home) = dirs::home_dir() {
            let path = home
                .join("Library/Application Support")
                .join(result_path)
                .join("plugins/ocr");

            if !path.exists() {
                let _ = fs::create_dir_all(&path);
            }
            return path;
        }
    }

    // 最后回退到插件目录
    let exe_path = env::current_exe().unwrap_or_default();
    let exe_dir = exe_path.parent().unwrap_or(std::path::Path::new("."));
    exe_dir.join("config")
}

/// 获取配置文件路径
fn get_config_path() -> PathBuf {
    get_app_data_dir().join("ocr-config.json")
}

/// 初始化日志配置
pub fn init_logger() {
    let log_path = match env::var("EASYPASTE_LOGS") {
        Ok(log_dir) => PathBuf::from(log_dir).join("ocr").join("ocr.log"),
        _ => get_app_data_dir().join("ocr").join("ocr.log"),
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
