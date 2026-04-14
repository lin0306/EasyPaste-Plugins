# EasyPaste-Plugins

[EasyPaste](https://github.com/lin0306/EasyPaste) 剪贴板管理工具的插件集合，提供 OCR 文字识别和文本翻译等实用功能。

## 📦 插件列表

### 1. OCR 文字识别插件

基于 ocr-rs (MNN) 的离线 OCR 插件，支持 PP-OCRv5 模型，纯 Rust 实现，无需联网，开箱即用。

**特性：**
- ✅ 完全离线运行，保护隐私
- ✅ 支持中英文识别
- ✅ 基于 PP-OCRv5 高精度模型
- ✅ 纯 Rust 实现，性能优异
- ✅ 即装即用，无需配置

**版本：** 0.0.4  
**平台：** Windows  
**下载：** [ocr.zip](https://github.com/lin0306/EasyPaste-Plugins/releases/download/ocr-v0.0.4/ocr.zip)

### 2. 文本翻译插件

多引擎翻译插件，支持多种主流翻译服务，满足不同场景的翻译需求。

**特性：**
- ✅ 支持多个翻译引擎（Google、DeepL、百度、有道等）
- ✅ 可自定义翻译引擎配置
- ✅ 快速翻译选中文本
- ✅ 轻量级设计

**版本：** 0.0.3  
**平台：** Windows  
**下载：** [translate.zip](https://github.com/lin0306/EasyPaste-Plugins/releases/download/translate-v0.0.3/translate.zip)

## 🚀 快速开始

### 前置要求

- Node.js 16+
- pnpm 8+
- Rust 1.70+
- 
#### OCR 插件需要额外安装以下依赖

- CMake（用于编译 MNN 依赖）
- LLVM（用于编译 MNN 依赖）

### 安装依赖

```bash
# 进入插件目录
cd ocr  # 或 cd translate

# 安装前端依赖
pnpm install
```

### 开发模式

```bash
pnpm run dev
```

### 构建插件

```bash
# 构建完整插件（包含前端和 Rust 后端）
pnpm run build

# 单独构建
pnpm run build:vue    # 构建前端
pnpm run build:rust   # 构建 Rust 后端
pnpm run build:copy   # 复制 Rust 二进制文件
pnpm run package      # 打包为 zip 文件
```

## 📁 项目结构

```
EasyPaste-Plugins/
├── ocr/                    # OCR 文字识别插件
│   ├── src/               # Vue 前端源码
│   │   ├── OCR.vue        # 主界面组件
│   │   ├── Settings.vue   # 设置组件
│   │   └── entry-web-component.ts
│   ├── rust/              # Rust 后端
│   │   ├── src/main.rs    # Rust 入口
│   │   └── Cargo.toml     # Rust 依赖配置
│   ├── public/            # 静态资源（模型文件等）
│   ├── dist/              # 构建输出
│   ├── configs/           # 构建配置脚本
│   └── package.json
│
├── translate/             # 文本翻译插件
│   ├── src/               # Vue 前端源码
│   │   ├── Translate.vue  # 主界面组件
│   │   ├── Settings.vue   # 设置组件
│   │   └── entry-web-component.ts
│   ├── rust/              # Rust 后端
│   │   ├── src/main.rs    # Rust 入口
│   │   └── Cargo.toml     # Rust 依赖配置
│   ├── public/            # 静态资源
│   ├── dist/              # 构建输出
│   ├── configs/           # 构建配置脚本
│   └── package.json
│
├── LICENSE                # Apache License 2.0
├── plugins-list.json      # 插件列表配置
└── README.md              # 项目说明文档
```

## 🔧 技术栈

**前端：**
- Vue 3
- Vite 6
- Naive UI
- Font Awesome Icons
- TypeScript

**后端：**
- Rust
- Tauri API v2

**OCR 插件特有：**
- ocr-rs 2.2.2
- MNN 推理引擎
- PP-OCRv5 模型

**翻译插件特有：**
- reqwest（HTTP 客户端）
- tokio（异步运行时）

## 📝 插件开发规范

每个插件需要包含以下内容：

1. **前端部分**（Vue 组件）
   - 主界面组件
   - 设置页面（可选）
   - Web Component 入口

2. **后端部分**（Rust）
   - 业务逻辑实现
   - 与 Tauri 主应用的通信

3. **配置文件**
   - `package.json` - 项目配置和构建脚本
   - `manifest.json` - 插件元数据
   - 国际化文件（locales）

4. **构建脚本**
   - `configs/copy-rust.js` - 复制 Rust 二进制文件
   - `configs/package-zip.js` - 打包插件

## 🌐 国际化

插件支持多语言，目前支持：
- 简体中文（zhCN）
- 英文（enUS）

在 `public/locales/` 目录下添加对应的语言文件即可扩展更多语言。

## 📄 许可证

本项目采用 **Apache License 2.0** 开源许可证。

您可以在遵守许可证条款的前提下，自由使用、修改和分发本项目的代码。详细条款请查看 [LICENSE](LICENSE) 文件。

主要条款摘要：
- ✅ 允许商业使用
- ✅ 允许修改和分发
- ✅ 允许专利使用
- ⚠️ 需要保留版权声明和许可证副本
- ⚠️ 修改的文件需要注明变更说明

## 🔗 相关链接

- [EasyPaste 主项目](https://github.com/lin0306/EasyPaste)
- [ocr-rs](https://github.com/jam1garner/ocr-rs)
- [Tauri](https://tauri.app/)
- [PP-OCR](https://github.com/PaddlePaddle/PaddleOCR)
- [Apache License 2.0](http://www.apache.org/licenses/LICENSE-2.0)

## 🤝 贡献

欢迎提交 Issue 和 Pull Request！

贡献指南：
1. Fork 本仓库
2. 创建您的特性分支 (`git checkout -b feature/AmazingFeature`)
3. 提交您的更改 (`git commit -m 'Add some AmazingFeature'`)
4. 推送到分支 (`git push origin feature/AmazingFeature`)
5. 开启一个 Pull Request

## 📮 联系方式

如有问题或建议，请通过 GitHub Issues 联系。

---

**Copyright © 2026 EasyPaste-Plugins Contributors**

Licensed under the Apache License, Version 2.0.
