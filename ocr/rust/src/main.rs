use serde::{Deserialize, Serialize};
use std::fs::OpenOptions;
use std::io::{self, Read};
use windows::{
    core::*,
    Graphics::Imaging::BitmapDecoder,
    Media::Ocr::OcrEngine,
    Storage::{FileAccessMode, StorageFile},
};

use log::{info, LevelFilter};

#[derive(Deserialize, Debug)]
struct Request {
    cmd: String,
    payload: String,
}

#[derive(Serialize)]
struct Response {
    result: String,
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
            let text = do_ocr(file_path); // 你的 OCR 逻辑
            Response { result: text }
        }
        _ => Response {
            result: "unknown command".into(),
        },
    };

    println!("{}", serde_json::to_string(&resp).unwrap());
}

fn do_ocr(file_path: &str) -> String {
    info!("OCR 图片识别，图片路径: {}", file_path);

    let file = StorageFile::GetFileFromPathAsync(&HSTRING::from(file_path))
        .expect("文件打开异常")
        .get()
        .expect("文件打开失败");
    let stream = file
        .OpenAsync(FileAccessMode::Read)
        .expect("获取文件流异常")
        .get()
        .expect("获取文件流失败");
    let decoder = BitmapDecoder::CreateAsync(&stream)
        .expect("创建 BitmapDecoder 失败")
        .get()
        .expect("创建 BitmapDecoder 失败");
    let bitmap = decoder
        .GetSoftwareBitmapAsync()
        .expect("创建 SoftwareBitmap 异常")
        .get()
        .expect("创建 SoftwareBitmap 失败");

    // 2. 获取中文 OCR 引擎
    let languages = OcrEngine::AvailableRecognizerLanguages().expect("获取 OCR 语言列表失败");
    for l in &languages {
        info!("OCR 语言: {}", l.LanguageTag().unwrap_or_default());
    }
    let zh_lang = languages
        .into_iter()
        .find(|l| {
            l.LanguageTag()
                .unwrap_or_default()
                .to_string()
                .starts_with("zh-Hans")
        })
        .expect("未找到简体中文 OCR 语言包");

    let engine = OcrEngine::TryCreateFromLanguage(&zh_lang).expect("创建 OCR 引擎失败");

    // 3. 识别
    let result = engine
        .RecognizeAsync(&bitmap)
        .expect("OCR 识别异常")
        .get()
        .expect("OCR 识别失败");

    if result.Text().is_ok() {
        let text = result.Text().unwrap().to_string();
        return clean_ocr_text(&*text);
    }

    "OCR 识别失败".parse().unwrap()
}

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

            // 如果前后都是汉字，则跳过这个空格（不 push）
            if prev_is_han && next_is_han {
                continue;
            }
        }

        // 其他情况：保留字符（包括非空格，或非汉字之间的空格）
        result.push(c);
    }

    result
}

fn is_han_char(c: char) -> bool {
    // 覆盖常用汉字范围（可根据需要扩展）
    ('\u{4e00}'..='\u{9fff}').contains(&c)    // CJK Unified Ideographs
        || ('\u{3400}'..='\u{4dbf}').contains(&c)    // CJK Extension A
        || ('\u{20000}'..='\u{2a6df}').contains(&c)  // CJK Extension B (罕见)
        || ('\u{2a700}'..='\u{2b73f}').contains(&c)  // CJK Extension C
        || ('\u{2b740}'..='\u{2b81f}').contains(&c)  // CJK Extension D
        || ('\u{2b820}'..='\u{2ceaf}').contains(&c)  // CJK Extension E, F 等
        || ('\u{3000}'..='\u{303f}').contains(&c)      // CJK 标点（如顿号、句号等，可选）
        || "，。！？；：\"\"''（）【】《》〈〉「」『』、·…—–——".contains(c) // 常用标点符号
}

/**
 * 初始化日志配置
 */
pub fn init_logger() {
    // 创建日志文件
    let log_path = std::env::current_exe()
        .unwrap()
        .parent()
        .unwrap()
        .join("logs")
        .join("ocr.log");

    // 创建 logs 目录（如果不存在）
    let logs_dir = log_path.parent().unwrap();
    std::fs::create_dir_all(logs_dir).expect("无法创建日志目录");

    let log_file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(&log_path)
        .expect("无法创建日志文件");

    // 配置 env_logger
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
