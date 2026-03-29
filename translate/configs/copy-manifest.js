const fs = require('fs');
const path = require('path');

// 复制 manifest.json 到 dist 目录
const sourceManifest = path.join(__dirname, '..', 'manifest.json');
const distDir = path.join(__dirname, '..', 'dist');
const targetManifest = path.join(distDir, 'manifest.json');

// 确保 dist 目录存在
if (!fs.existsSync(distDir)) {
    fs.mkdirSync(distDir, { recursive: true });
}

// 复制文件
fs.copyFileSync(sourceManifest, targetManifest);
console.log('manifest.json copied to dist directory');

// 复制 Rust 可执行文件
const rustSourceDir = path.join(__dirname, '..', 'rust', 'target', 'release');
const rustTargetDir = path.join(distDir, 'rust');

if (!fs.existsSync(rustTargetDir)) {
    fs.mkdirSync(rustTargetDir, { recursive: true });
}

// 根据平台复制可执行文件
const platform = process.platform;
let exeName = 'translate_plugin';
if (platform === 'win32') {
    exeName += '.exe';
}

const sourceExe = path.join(rustSourceDir, exeName);
const targetExe = path.join(rustTargetDir, exeName);

if (fs.existsSync(sourceExe)) {
    fs.copyFileSync(sourceExe, targetExe);
    console.log(`Rust executable copied: ${exeName}`);
} else {
    console.warn(`Rust executable not found: ${sourceExe}`);
}

console.log('Build copy completed!');
