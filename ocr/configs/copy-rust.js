const fs = require('fs');
const path = require('path');

const distDir = 'dist';

// 确保 dist 目录存在
if (!fs.existsSync(distDir)) {
    fs.mkdirSync(distDir, { recursive: true });
}

console.log('manifest.json copied to dist folder');

// 创建 rust 子目录
const rustDir = path.join(distDir, 'rust');
if (!fs.existsSync(rustDir)) {
    fs.mkdirSync(rustDir, { recursive: true });
}

// 复制插件二进制文件
const platform = process.platform;
let exeName;

if (platform === 'win32') {
    exeName = 'ocr_plugin.exe';
} else if (platform === 'darwin' || platform === 'linux') {
    exeName = 'ocr_plugin';
} else {
    console.warn(`Unsupported platform: ${platform}. Skipping plugin binary copy.`);
    return;
}

const srcExePath = path.join('rust', 'target', 'release', exeName);
const destExePath = path.join(rustDir, exeName);

if (fs.existsSync(srcExePath)) {
    fs.copyFileSync(srcExePath, destExePath);
    console.log(`${exeName} copied to dist/rust folder`);
} else {
    console.error(`⚠️ Source executable not found: ${srcExePath}`);
    console.error('Make sure you have built the plugin with `cargo build --release`.');
}

console.log('Build copy completed!');
