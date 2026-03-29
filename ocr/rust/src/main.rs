use serde::{Deserialize, Serialize};
use std::fs::{self, OpenOptions};
use std::io::{self, Read};
use std::path::PathBuf;

use log::{info, LevelFilter};
use rten::Model;

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
    #[serde(rename = "ocrLanguage")]
    ocr_language: String,
}

impl Default for OcrConfig {
    fn default() -> Self {
        Self {
            ocr_mode: "window".to_string(),
            ocr_language: "chi_sim+eng".to_string(),
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
                
                // 确保模型已下载
                if let Err(e) = ensure_models() {
                    let action_resp = ActionResponse {
                        action: "error".to_string(),
                        text: Some(format!("模型加载失败: {}", e)),
                        title: None,
                        width: None,
                        height: None,
                    };
                    Response { result: serde_json::to_string(&action_resp).unwrap() }
                } else {
                    // 执行 OCR 识别
                    match recognize_text(file_path) {
                        Ok(text) => {
                            let cleaned = clean_ocr_text(&text);
                            let result_text = if cleaned.trim().is_empty() {
                                "未能识别出文字".to_string()
                            } else {
                                cleaned
                            };
                            
                            let action_resp = ActionResponse {
                                action: "copyToClipboard".to_string(),
                                text: Some(result_text),
                                title: None,
                                width: None,
                                height: None,
                            };
                            Response { result: serde_json::to_string(&action_resp).unwrap() }
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
                            Response { result: serde_json::to_string(&action_resp).unwrap() }
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
                Response { result: serde_json::to_string(&action_resp).unwrap() }
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
                result: if result { "success".to_string() } else { "failed".to_string() },
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

    // 确保模型已下载
    if let Err(e) = ensure_models() {
        return format!("模型加载失败: {}\n\n请检查网络连接后重试。", e);
    }

    // 使用 ocrs 进行 OCR 识别
    match recognize_text(file_path) {
        Ok(text) => {
            let cleaned = clean_ocr_text(&text);
            if cleaned.trim().is_empty() {
                "未能识别出文字".to_string()
            } else {
                cleaned
            }
        }
        Err(e) => {
            info!("OCR 识别失败: {}", e);
            format!("OCR 识别失败: {}\n\n请确保图片格式正确。", e)
        }
    }
}

/// 模型文件信息
const DETECTION_MODEL_URL: &str = "https://ocrs-models.s3-accelerate.amazonaws.com/text-detection.rten";
const RECOGNITION_MODEL_URL: &str = "https://ocrs-models.s3-accelerate.amazonaws.com/text-recognition.rten";
const DETECTION_MODEL_NAME: &str = "text-detection.rten";
const RECOGNITION_MODEL_NAME: &str = "text-recognition.rten";

/// 获取模型目录路径
fn get_models_dir() -> PathBuf {
    get_app_data_dir().join("models")
}

/// 确保模型文件已下载
fn ensure_models() -> anyhow::Result<()> {
    let models_dir = get_models_dir();
    let detection_path = models_dir.join(DETECTION_MODEL_NAME);
    let recognition_path = models_dir.join(RECOGNITION_MODEL_NAME);

    // 创建模型目录
    if !models_dir.exists() {
        fs::create_dir_all(&models_dir)?;
    }

    // 下载检测模型（如果不存在）
    if !detection_path.exists() {
        info!("正在下载检测模型...");
        download_file(DETECTION_MODEL_URL, &detection_path)?;
        info!("检测模型下载完成");
    }

    // 下载识别模型（如果不存在）
    if !recognition_path.exists() {
        info!("正在下载识别模型...");
        download_file(RECOGNITION_MODEL_URL, &recognition_path)?;
        info!("识别模型下载完成");
    }

    Ok(())
}

/// 下载文件
fn download_file(url: &str, path: &PathBuf) -> anyhow::Result<()> {
    let response = ureq::get(url)
        .timeout(std::time::Duration::from_secs(120))
        .call()?;

    let mut file = fs::File::create(path)?;
    
    // ureq 2.x API
    let mut reader = response.into_reader();
    io::copy(&mut reader, &mut file)?;
    
    Ok(())
}

/// 使用 ocrs 识别图片文字
fn recognize_text(image_path: &str) -> anyhow::Result<String> {
    use ocrs::{OcrEngine, OcrEngineParams};

    // 加载模型
    let models_dir = get_models_dir();
    let detection_model_data = fs::read(models_dir.join(DETECTION_MODEL_NAME))?;
    let recognition_model_data = fs::read(models_dir.join(RECOGNITION_MODEL_NAME))?;
    
    let detection_model = Model::load(detection_model_data)?;
    let recognition_model = Model::load(recognition_model_data)?;

    // 创建 OCR 引擎
    let engine = OcrEngine::new(OcrEngineParams {
        detection_model: Some(detection_model),
        recognition_model: Some(recognition_model),
        ..Default::default()
    })?;
    
    // 读取图片
    let img = image::open(image_path)?;
    let img_rgb = img.to_rgb8();
    
    // 转换图片为输入格式
    let (width, height) = img_rgb.dimensions();
    let pixels: Vec<u8> = img_rgb.into_raw();
    
    // 创建图像源 (RGBA 格式，每个像素 4 字节)
    let rgba_pixels: Vec<u8> = pixels.chunks(3)
        .flat_map(|rgb| vec![rgb[0], rgb[1], rgb[2], 255])
        .collect();
    
    let input = ocrs::ImageSource::from_bytes(&rgba_pixels, (width, height))?;
    
    // 预处理图片
    let ocr_input = engine.prepare_input(input)?;
    
    // 使用简便 API 获取所有文本
    let text = engine.get_text(&ocr_input)?;
    
    Ok(text)
}

/// 清理 OCR 文本
fn clean_ocr_text(s: &str) -> String {
    let chars: Vec<char> = s.chars().collect();
    let mut result = String::new();
    let len = chars.len();

    for i in 0..len {
        let c = chars[i];

        // 如果当前字符是 ASCII 空格
        if c.is_ascii_whitespace() {
            // 检查前一个和后一个字符是否都是汉字
            let prev_is_han = i > 0 && is_han_char(chars[i - 1]);
            let next_is_han = i + 1 < len && is_han_char(chars[i + 1]);

            // 如果前后都是汉字，则跳过这个空格
            if prev_is_han && next_is_han {
                continue;
            }
        }

        result.push(c);
    }

    result.trim().to_string()
}

fn is_han_char(c: char) -> bool {
    ('\u{4e00}'..='\u{9fff}').contains(&c)
        || ('\u{3400}'..='\u{4dbf}').contains(&c)
        || ('\u{20000}'..='\u{2a6df}').contains(&c)
        || is_chinese_punctuation(c)
}

fn is_chinese_punctuation(c: char) -> bool {
    matches!(c, '，' | '。' | '！' | '？' | '；' | '：' | '"' | '\'' | '（' | '）' | '【' | '】' | '《' | '》' | '〈' | '〉' | '「' | '」' | '『' | '』' | '、' | '·' | '…' | '—' | '–')
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
    if let Ok(data_dir) = std::env::var("EASYPASTE_DATA_DIR") {
        let path = PathBuf::from(data_dir).join("plugins").join("ocr");
        if !path.exists() {
            let _ = fs::create_dir_all(&path);
        }
        return path;
    }
    
    // 回退到系统应用数据目录
    #[cfg(target_os = "windows")]
    {
        if let Some(app_data) = dirs::data_dir() {
            let path = app_data.join("com.lin.EasyPaste").join("plugins").join("ocr");
            if !path.exists() {
                let _ = fs::create_dir_all(&path);
            }
            return path;
        }
    }
    
    #[cfg(target_os = "macos")]
    {
        if let Some(home) = dirs::home_dir() {
            let path = home.join("Library/Application Support/com.lin.EasyPaste/plugins/ocr");
            if !path.exists() {
                let _ = fs::create_dir_all(&path);
            }
            return path;
        }
    }
    
    // 最后回退到插件目录
    let exe_path = std::env::current_exe().unwrap_or_default();
    let exe_dir = exe_path.parent().unwrap_or(std::path::Path::new("."));
    exe_dir.join("config")
}

/// 获取配置文件路径
fn get_config_path() -> PathBuf {
    get_app_data_dir().join("ocr-config.json")
}

/// 初始化日志配置
pub fn init_logger() {
    let log_path = get_app_data_dir().join("logs").join("ocr.log");

    let logs_dir = log_path.parent().unwrap();
    std::fs::create_dir_all(logs_dir).expect("无法创建日志目录");

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
