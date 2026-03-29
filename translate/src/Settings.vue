<template>
  <div class="translate-settings">
    <n-divider title-placement="left">{{ $t('translationSettingsTitle') }}</n-divider>
    
    <!-- 翻译引擎 -->
    <div class="setting-item">
      <div class="setting-label">
        <span>{{ $t('translationEngine') }}</span>
      </div>
      <n-select
        v-model:value="currentEngine"
        :options="engineOptions"
        class="setting-select"
        @update:value="onEngineChange"
        :teleported="false"
      />
    </div>

    <!-- API Key -->
    <div class="setting-item">
      <div class="setting-label">
        <span>{{ $t('apiKey') }}</span>
        <n-tooltip trigger="hover">
          <template #trigger>
            <svg class="hint-icon" viewBox="0 0 1024 1024">
              <path d="M512 64C264.6 64 64 264.6 64 512s200.6 448 448 448 448-200.6 448-448S759.4 64 512 64z m0 820c-205.4 0-372-166.6-372-372s166.6-372 372-372 372 166.6 372 372-166.6 372-372 372z" fill="currentColor"/>
              <path d="M512 336m-40 0a40 40 0 1 0 80 0 40 40 0 1 0-80 0Z" fill="currentColor"/>
              <path d="M536 448h-48c-4.4 0-8 3.6-8 8v272c0 4.4 3.6 8 8 8h48c4.4 0 8-3.6 8-8V456c0-4.4-3.6-8-8-8z" fill="currentColor"/>
            </svg>
          </template>
          <span>{{ getApiKeyHint() }}</span>
        </n-tooltip>
      </div>
      <n-input
        v-model:value="apiKeys[currentEngine]"
        type="password"
        :show-password-on="'click'"
        :placeholder="getApiKeyPlaceholder()"
        class="setting-input"
      />
    </div>

    <!-- 保存按钮 -->
    <div class="setting-actions">
      <n-button type="primary" :loading="saving" @click="saveConfig">
        {{ $t('saveBtn') }}
      </n-button>
    </div>

    <!-- API 说明 -->
    <n-alert type="info" class="api-hint">
      <template #header>
        {{ $t('apiHintTitle') }}
      </template>
      <div v-html="getApiHint()"></div>
    </n-alert>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { NSelect, NInput, NButton, NDivider, NTooltip, NAlert, useMessage } from 'naive-ui'
import { invoke } from '@tauri-apps/api/core'

const message = useMessage()

// 当前选中的引擎
const currentEngine = ref('baidu')

// 每个引擎的API key
const apiKeys = ref<Record<string, string>>({
  google: '',
  deepl: '',
  baidu: '',
  youdao: '',
})

const saving = ref(false)

// 引擎选项
const engineOptions = [
  { label: 'Google Translate', value: 'google' },
  { label: 'DeepL', value: 'deepl' },
  { label: '百度翻译', value: 'baidu' },
  { label: '有道翻译', value: 'youdao' },
]

// 多语言支持
const messages: Record<string, Record<string, string>> = {
  zh: {
    translationSettingsTitle: '翻译设置',
    translationEngine: '翻译引擎',
    apiKey: 'API Key',
    saveBtn: '保存设置',
    saveSuccess: '设置保存成功',
    saveFailed: '设置保存失败',
    loadFailed: '加载配置失败',
    apiHintTitle: 'API 配置说明',
    googleApiKeyHint: 'Google Cloud Translation API Key',
    deeplApiKeyHint: 'DeepL API Key',
    baiduApiKeyHint: '百度翻译 API Key，格式：appid#secretKey',
    youdaoApiKeyHint: '有道翻译 API Key，格式：appid#secretKey',
    apiKeyPlaceholder: '请输入 API Key',
  },
  en: {
    translationSettingsTitle: 'Translation Settings',
    translationEngine: 'Translation Engine',
    apiKey: 'API Key',
    saveBtn: 'Save Settings',
    saveSuccess: 'Settings saved successfully',
    saveFailed: 'Failed to save settings',
    loadFailed: 'Failed to load configuration',
    apiHintTitle: 'API Configuration Guide',
    googleApiKeyHint: 'Google Cloud Translation API Key',
    deeplApiKeyHint: 'DeepL API Key',
    baiduApiKeyHint: 'Baidu Translate API Key, format: appid#secretKey',
    youdaoApiKeyHint: 'Youdao Translate API Key, format: appid#secretKey',
    apiKeyPlaceholder: 'Enter API Key',
  },
}

const $t = (key: string): string => {
  const lang = navigator.language.startsWith('zh') ? 'zh' : 'en'
  return messages[lang][key] || key
}

// 获取当前引擎的API Key提示
const getApiKeyHint = () => {
  const hints: Record<string, string> = {
    google: messages[navigator.language.startsWith('zh') ? 'zh' : 'en'].googleApiKeyHint,
    deepl: messages[navigator.language.startsWith('zh') ? 'zh' : 'en'].deeplApiKeyHint,
    baidu: messages[navigator.language.startsWith('zh') ? 'zh' : 'en'].baiduApiKeyHint,
    youdao: messages[navigator.language.startsWith('zh') ? 'zh' : 'en'].youdaoApiKeyHint,
  }
  return hints[currentEngine.value] || ''
}

// 获取API Key占位符
const getApiKeyPlaceholder = () => {
  return $t('apiKeyPlaceholder')
}

// 获取当前引擎的 API 说明
const getApiHint = () => {
  const lang = navigator.language.startsWith('zh') ? 'zh' : 'en'
  const hints: Record<string, Record<string, string>> = {
    google: {
      zh: `
        <p><b>Google Translate:</b></p>
        <ul>
          <li>需要 Google Cloud Translation API Key</li>
          <li>访问 <a href="https://cloud.google.com/translate" target="_blank">Google Cloud</a> 获取</li>
          <li>每月前 50 万字符免费</li>
        </ul>
      `,
      en: `
        <p><b>Google Translate:</b></p>
        <ul>
          <li>Requires Google Cloud Translation API Key</li>
          <li>Visit <a href="https://cloud.google.com/translate" target="_blank">Google Cloud</a> to get</li>
          <li>First 500K characters free per month</li>
        </ul>
      `,
    },
    deepl: {
      zh: `
        <p><b>DeepL:</b></p>
        <ul>
          <li>需要 DeepL API Key</li>
          <li>访问 <a href="https://www.deepl.com/pro-api" target="_blank">DeepL API</a> 获取</li>
          <li>每月前 50 万字符免费</li>
        </ul>
      `,
      en: `
        <p><b>DeepL:</b></p>
        <ul>
          <li>Requires DeepL API Key</li>
          <li>Visit <a href="https://www.deepl.com/pro-api" target="_blank">DeepL API</a> to get</li>
          <li>First 500K characters free per month</li>
        </ul>
      `,
    },
    baidu: {
      zh: `
        <p><b>百度翻译:</b></p>
        <ul>
          <li>需要百度翻译开放平台账号</li>
          <li>访问 <a href="https://fanyi-api.baidu.com/" target="_blank">百度翻译 API</a> 获取</li>
          <li>标准版免费额度：5万字符/月</li>
          <li>API Key 格式：<code>appid#secretKey</code></li>
        </ul>
      `,
      en: `
        <p><b>Baidu Translate:</b></p>
        <ul>
          <li>Requires Baidu Translate Open Platform account</li>
          <li>Visit <a href="https://fanyi-api.baidu.com/" target="_blank">Baidu Translate API</a> to get</li>
          <li>Free tier: 50K characters/month</li>
          <li>API Key format: <code>appid#secretKey</code></li>
        </ul>
      `,
    },
    youdao: {
      zh: `
        <p><b>有道翻译:</b></p>
        <ul>
          <li>需要有道智云账号</li>
          <li>访问 <a href="https://ai.youdao.com/" target="_blank">有道智云</a> 获取</li>
          <li>体验版免费额度：10万字符/月</li>
          <li>API Key 格式：<code>appid#secretKey</code></li>
        </ul>
      `,
      en: `
        <p><b>Youdao Translate:</b></p>
        <ul>
          <li>Requires Youdao AI Cloud account</li>
          <li>Visit <a href="https://ai.youdao.com/" target="_blank">Youdao AI Cloud</a> to get</li>
          <li>Free tier: 100K characters/month</li>
          <li>API Key format: <code>appid#secretKey</code></li>
        </ul>
      `,
    },
  }
  return hints[currentEngine.value]?.[lang] || ''
}

// 引擎切换时，当前显示的API key已经绑定到对应的引擎
// 不需要额外处理，因为 apiKeys 是响应式对象
function onEngineChange(newEngine: string) {
  console.log('切换到引擎:', newEngine)
  // 保存当前引擎设置到后端（但不保存API keys，等用户点击保存按钮）
  saveEngineSetting(newEngine)
}

// 保存引擎设置
async function saveEngineSetting(engine: string) {
  try {
    // 先加载当前配置，保留用户的语言设置
    const configResult = await invoke('invoke_external_plugin', {
      pluginId: 'translate',
      pluginName: 'translate_plugin.exe',
      cmd: 'get-config',
      payload: '{}',
    }) as string

    let currentConfig = { sourceLanguage: 'auto', targetLanguage: 'zh' }
    try {
      const parsed = JSON.parse(JSON.parse(configResult).result || '{}')
      currentConfig = { ...currentConfig, ...parsed }
    } catch (e) {
      // 使用默认值
    }

    await invoke('invoke_external_plugin', {
      pluginId: 'translate',
      pluginName: 'translate_plugin.exe',
      cmd: 'save-config',
      payload: JSON.stringify({
        translationEngine: engine,
        sourceLanguage: currentConfig.source_language || currentConfig.sourceLanguage || 'auto',
        targetLanguage: currentConfig.target_language || currentConfig.targetLanguage || 'zh',
      }),
    })
  } catch (e) {
    console.error('保存引擎设置失败:', e)
  }
}

// 加载配置
async function loadConfig() {
  try {
    // 加载引擎设置
    const configResult = await invoke('invoke_external_plugin', {
      pluginId: 'translate',
      pluginName: 'translate_plugin.exe',
      cmd: 'get-config',
      payload: '{}',
    }) as string

    const configResponse = JSON.parse(configResult)
    if (configResponse.result) {
      const savedConfig = JSON.parse(configResponse.result)
      currentEngine.value = savedConfig.translationEngine || 'baidu'
    }

    // 加载API Keys
    const keysResult = await invoke('invoke_external_plugin', {
      pluginId: 'translate',
      pluginName: 'translate_plugin.exe',
      cmd: 'get-api-keys',
      payload: '{}',
    }) as string

    const keysResponse = JSON.parse(keysResult)
    if (keysResponse.result) {
      const savedKeys = JSON.parse(keysResponse.result)
      apiKeys.value = { ...apiKeys.value, ...savedKeys }
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
    // 先加载当前配置，保留用户的语言设置
    const configResult = await invoke('invoke_external_plugin', {
      pluginId: 'translate',
      pluginName: 'translate_plugin.exe',
      cmd: 'get-config',
      payload: '{}',
    }) as string

    let currentConfig = { sourceLanguage: 'auto', targetLanguage: 'zh' }
    try {
      const parsed = JSON.parse(JSON.parse(configResult).result || '{}')
      currentConfig = { ...currentConfig, ...parsed }
    } catch (e) {
      // 使用默认值
    }

    // 保存引擎设置
    await invoke('invoke_external_plugin', {
      pluginId: 'translate',
      pluginName: 'translate_plugin.exe',
      cmd: 'save-config',
      payload: JSON.stringify({
        translationEngine: currentEngine.value,
        sourceLanguage: currentConfig.source_language || currentConfig.sourceLanguage || 'auto',
        targetLanguage: currentConfig.target_language || currentConfig.targetLanguage || 'zh',
      }),
    })

    // 保存API Keys
    const result = await invoke('invoke_external_plugin', {
      pluginId: 'translate',
      pluginName: 'translate_plugin.exe',
      cmd: 'save-api-keys',
      payload: JSON.stringify(apiKeys.value),
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
.translate-settings {
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

.setting-input {
  width: 280px;
}

.setting-actions {
  display: flex;
  justify-content: flex-end;
  margin-top: 24px;
  padding-top: 16px;
  border-top: 1px solid var(--theme-universal-border);
}

.api-hint {
  margin-top: 24px;
}

.api-hint :deep(.n-alert-body) {
  font-size: 13px;
}

.api-hint :deep(code) {
  background: var(--theme-universal-secondary);
  padding: 2px 6px;
  border-radius: 3px;
  font-family: monospace;
}

.api-hint :deep(a) {
  color: var(--theme-primary-color);
}

.api-hint :deep(ul) {
  margin: 8px 0;
  padding-left: 20px;
}

.api-hint :deep(li) {
  margin: 4px 0;
}
</style>
