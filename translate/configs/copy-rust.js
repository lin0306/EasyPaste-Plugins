const fs = require('fs');
const path = require('path');

const distDir = 'dist';

// 确保 dist 目录存在
if (!fs.existsSync(distDir)) {
    fs.mkdirSync(distDir, { recursive: true });
}

// 创建 rust 子目录
const rustDir = path.join(distDir, 'rust');
if (!fs.existsSync(rustDir)) {
    fs.mkdirSync(rustDir, { recursive: true });
}

// 复制插件二进制文件
const platform = process.platform;
let exeName;

if (platform === 'win32') {
    exeName = 'translate_plugin.exe';
} else if (platform === 'darwin' || platform === 'linux') {
    exeName = 'translate_plugin';
} else {
    console.warn(`不支持的平台: ${platform}，跳过插件二进制文件复制。`);
    return;
}

const srcExePath = path.join('rust', 'target', 'release', exeName);
const destExePath = path.join(rustDir, exeName);

if (fs.existsSync(srcExePath)) {
    fs.copyFileSync(srcExePath, destExePath);
    console.log(`${exeName} 已复制到 dist/rust 文件夹`);
} else {
    console.error(`⚠️ 未找到可执行文件: ${srcExePath}`);
    console.error('请确保已使用 `cargo build --release` 构建插件。');
}

console.log('构建复制完成！');
