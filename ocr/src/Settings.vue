<template>
  <div class="ocr-settings">
    <n-divider title-placement="left">{{ $t('ocrSettingsTitle') }}</n-divider>
    
    <!-- OCR 模式选择 -->
    <div class="setting-item">
      <div class="setting-label">
        <span>{{ $t('ocrMode') }}</span>
        <n-tooltip trigger="hover">
          <template #trigger>
            <svg class="hint-icon" viewBox="0 0 1024 1024">
              <path d="M512 64C264.6 64 64 264.6 64 512s200.6 448 448 448 448-200.6 448-448S759.4 64 512 64z m0 820c-205.4 0-372-166.6-372-372s166.6-372 372-372 372 166.6 372 372-166.6 372-372 372z" fill="currentColor"/>
              <path d="M512 336m-40 0a40 40 0 1 0 80 0 40 40 0 1 0-80 0Z" fill="currentColor"/>
              <path d="M536 448h-48c-4.4 0-8 3.6-8 8v272c0 4.4 3.6 8 8 8h48c4.4 0 8-3.6 8-8V456c0-4.4-3.6-8-8-8z" fill="currentColor"/>
            </svg>
          </template>
          <span>{{ $t('ocrModeHint') }}</span>
        </n-tooltip>
      </div>
      <n-select
        v-model:value="config.ocrMode"
        :options="ocrModeOptions"
        class="setting-select"
      />
    </div>

    <!-- 识别语言选择 -->
    <div class="setting-item">
      <div class="setting-label">
        <span>{{ $t('ocrLanguage') }}</span>
      </div>
      <n-select
        v-model:value="config.ocrLanguage"
        :options="ocrLanguageOptions"
        class="setting-select"
      />
    </div>

    <!-- 保存按钮 -->
    <div class="setting-actions">
      <n-button type="primary" :loading="saving" @click="saveConfig">
        {{ $t('saveBtn') }}
      </n-button>
    </div>

    <!-- 说明 -->
    <n-alert type="info" class="info-hint">
      <template #header>
        {{ $t('infoTitle') }}
      </template>
      <div v-html="$t('infoContent')"></div>
    </n-alert>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { NSelect, NButton, NDivider, NTooltip, NAlert, useMessage } from 'naive-ui'
import { invoke } from '@tauri-apps/api/core'

const message = useMessage()

// 配置状态
const config = ref({
  ocrMode: 'window',
  ocrLanguage: 'chi_sim+eng',
})

const saving = ref(false)

// OCR 模式选项
const ocrModeOptions = [
  { label: '窗口模式 - 打开窗口查看结果', value: 'window' },
  { label: '快速模式 - 直接保存到剪贴板', value: 'quick' },
]

// OCR 语言选项
const ocrLanguageOptions = [
  { label: '中文 + 英文', value: 'chi_sim+eng' },
  { label: '简体中文', value: 'chi_sim' },
  { label: '英文', value: 'eng' },
  { label: '繁体中文', value: 'chi_tra' },
  { label: '日语', value: 'jpn' },
  { label: '韩语', value: 'kor' },
]

// 多语言支持
const messages: Record<string, Record<string, string>> = {
  zh: {
    ocrSettingsTitle: 'OCR 设置',
    ocrMode: 'OCR 模式',
    ocrModeHint: '快速模式将直接识别图片文字并保存到剪贴板，不打开窗口',
    ocrLanguage: '识别语言',
    saveBtn: '保存设置',
    saveSuccess: '设置保存成功',
    saveFailed: '设置保存失败',
    loadFailed: '加载配置失败',
    infoTitle: '离线 OCR 说明',
    infoContent: `
      <p>本插件使用纯 Rust 实现的离线 OCR 引擎，无需联网即可使用。</p>
      <ul>
        <li>✅ 无需安装额外软件</li>
        <li>✅ 无需配置 API Key</li>
        <li>✅ 支持中英文混合识别</li>
        <li>⚠️ 首次识别可能需要加载模型（约 1-2 秒）</li>
      </ul>
    `,
  },
  en: {
    ocrSettingsTitle: 'OCR Settings',
    ocrMode: 'OCR Mode',
    ocrModeHint: 'Quick mode will save recognized text to clipboard directly without opening a window',
    ocrLanguage: 'Recognition Language',
    saveBtn: 'Save Settings',
    saveSuccess: 'Settings saved successfully',
    saveFailed: 'Failed to save settings',
    loadFailed: 'Failed to load configuration',
    infoTitle: 'Offline OCR Info',
    infoContent: `
      <p>This plugin uses a pure Rust offline OCR engine, no internet required.</p>
      <ul>
        <li>✅ No additional software installation</li>
        <li>✅ No API Key required</li>
        <li>✅ Supports Chinese and English mixed recognition</li>
        <li>⚠️ First recognition may take 1-2 seconds to load model</li>
      </ul>
    `,
  },
}

// 简单的 i18n 实现
const $t = (key: string): string => {
  const lang = navigator.language.startsWith('zh') ? 'zh' : 'en'
  return messages[lang][key] || key
}

// 加载配置
async function loadConfig() {
  try {
    const result = await invoke('invoke_external_plugin', {
      pluginId: 'ocr',
      pluginName: 'ocr_plugin.exe',
      cmd: 'get-config',
      payload: '{}',
    }) as string

    const response = JSON.parse(result)
    if (response.result) {
      const savedConfig = JSON.parse(response.result)
      config.value = { ...config.value, ...savedConfig }
    }
  } catch (e) {
    console.error('加载配置失败:', e)
    message.error($t('loadFailed'))
  }
}

// 保存配置
async function saveConfig() {
  saving.value = true
  try {
    const result = await invoke('invoke_external_plugin', {
      pluginId: 'ocr',
      pluginName: 'ocr_plugin.exe',
      cmd: 'save-config',
      payload: JSON.stringify(config.value),
    }) as string

    const response = JSON.parse(result)
    if (response.result === 'success') {
      message.success($t('saveSuccess'))
    } else {
      message.error($t('saveFailed'))
    }
  } catch (e) {
    console.error('保存配置失败:', e)
    message.error($t('saveFailed'))
  } finally {
    saving.value = false
  }
}

onMounted(() => {
  loadConfig()
})
</script>

<style scoped>
.ocr-settings {
  padding: 16px;
  max-width: 600px;
}

.setting-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 16px;
  padding: 0 4px;
}

.setting-label {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 14px;
  color: var(--theme-universal-text);
}

.hint-icon {
  width: 16px;
  height: 16px;
  opacity: 0.6;
  cursor: help;
}

.hint-icon:hover {
  opacity: 0.8;
}

.setting-select {
  width: 280px;
}

.setting-actions {
  display: flex;
  justify-content: flex-end;
  margin-top: 24px;
  padding-top: 16px;
  border-top: 1px solid var(--theme-universal-border);
}

.info-hint {
  margin-top: 24px;
}

.info-hint :deep(.n-alert-body) {
  font-size: 13px;
}

.info-hint :deep(ul) {
  margin: 8px 0;
  padding-left: 20px;
}

.info-hint :deep(li) {
  margin: 4px 0;
}
</style>
