<template>
  <div class="translate-container">
    <!-- 语言选择栏 -->
    <div class="language-bar">
      <n-select
          v-model:value="config.sourceLanguage"
          :options="sourceLangOptions"
          size="small"
          class="lang-select"
          :teleported="true"
      />
      <n-button text size="small" @click="swapLanguages" class="swap-btn">
        <template #icon>
          <font-awesome-icon :icon="faRightLeft" class="btn-icon"/>
        </template>
      </n-button>
      <n-select
          v-model:value="config.targetLanguage"
          :options="targetLangOptions"
          size="small"
          class="lang-select"
          :teleported="true"
      />
    </div>

    <!-- 源文本 -->
    <div class="text-section">
      <div class="section-header">
        <span class="section-title">{{ $t('sourceText') }}</span>
        <n-button text size="small" @click="copyText(sourceText)">
          <template #icon>
            <font-awesome-icon :icon="faCopy" class="btn-icon" />
          </template>
          {{ $t('copy') }}
        </n-button>
      </div>
      <n-input
          v-model:value="sourceText"
          type="textarea"
          :rows="5"
          :placeholder="$t('sourcePlaceholder')"
          class="text-input"
      />
    </div>

    <!-- 操作栏 -->
    <div class="action-bar">
      <n-button type="primary" :loading="translating" @click="doTranslate">
        {{ $t('translate') }}
      </n-button>
    </div>

    <!-- 翻译结果 -->
    <div class="text-section">
      <div class="section-header">
        <span class="section-title">{{ $t('translatedText') }}</span>
        <n-button text size="small" @click="copyText(translatedText)">
          <template #icon>
            <font-awesome-icon :icon="faCopy" class="btn-icon" />
          </template>
          {{ $t('copy') }}
        </n-button>
      </div>
      <n-input
          v-model:value="translatedText"
          type="textarea"
          :rows="5"
          :placeholder="$t('translatedPlaceholder')"
          class="text-input"
          readonly
      />
    </div>

    <!-- 状态栏 -->
    <div class="status-bar">
      <span v-if="config.translationEngine">
        {{ $t('engine') }}: {{ engineName }}
      </span>
    </div>
  </div>
</template>

<script setup lang="ts">
import {computed, onMounted, onUnmounted, ref} from 'vue'
import {NButton, NInput, NSelect, useMessage} from 'naive-ui'
import {invoke} from '@tauri-apps/api/core'
import {listen} from '@tauri-apps/api/event'
import {writeText} from '@tauri-apps/plugin-clipboard-manager'
import {faRightLeft} from "@fortawesome/free-solid-svg-icons";
import {faCopy} from "@fortawesome/free-regular-svg-icons";

const message = useMessage()

// 状态
const sourceText = ref('')
const translatedText = ref('')
const translating = ref(false)
const config = ref({
  translationEngine: 'baidu',
  sourceLanguage: 'auto',
  targetLanguage: 'zh',
})

// 源语言选项（包含自动检测）
const sourceLangOptions = [
  {label: '自动检测', value: 'auto'},
  {label: '中文', value: 'zh'},
  {label: '英文', value: 'en'},
  {label: '日语', value: 'ja'},
  {label: '韩语', value: 'ko'},
  {label: '法语', value: 'fr'},
  {label: '德语', value: 'de'},
  {label: '西班牙语', value: 'es'},
  {label: '俄语', value: 'ru'},
]

// 目标语言选项
const targetLangOptions = [
  {label: '中文', value: 'zh'},
  {label: '英文', value: 'en'},
  {label: '日语', value: 'ja'},
  {label: '韩语', value: 'ko'},
  {label: '法语', value: 'fr'},
  {label: '德语', value: 'de'},
  {label: '西班牙语', value: 'es'},
  {label: '俄语', value: 'ru'},
]

// 计算属性
const engineName = computed(() => {
  const names: Record<string, string> = {
    google: 'Google',
    deepl: 'DeepL',
    baidu: '百度',
    youdao: '有道',
  }
  return names[config.value.translationEngine] || config.value.translationEngine
})

// 多语言支持
const messages: Record<string, Record<string, string>> = {
  zh: {
    sourceText: '原文',
    translatedText: '译文',
    sourcePlaceholder: '请输入要翻译的文本',
    translatedPlaceholder: '翻译结果将显示在这里',
    translate: '翻译',
    copy: '复制',
    copySuccess: '已复制到剪贴板',
    engine: '引擎',
    translateFailed: '翻译失败',
  },
  en: {
    sourceText: 'Source',
    translatedText: 'Translation',
    sourcePlaceholder: 'Enter text to translate',
    translatedPlaceholder: 'Translation will appear here',
    translate: 'Translate',
    copy: 'Copy',
    copySuccess: 'Copied to clipboard',
    engine: 'Engine',
    translateFailed: 'Translation failed',
  },
}

const $t = (key: string): string => {
  const lang = navigator.language.startsWith('zh') ? 'zh' : 'en'
  return messages[lang][key] || key
}

// 加载配置
async function loadConfig() {
  try {
    const result = await invoke('invoke_external_plugin', {
      pluginId: 'translate',
      pluginName: 'translate_plugin.exe',
      cmd: 'get-config',
      payload: '{}',
    }) as string

    const response = JSON.parse(result)
    if (response.result) {
      const savedConfig = JSON.parse(response.result)
      config.value = {...config.value, ...savedConfig}
    }
  } catch (e) {
    console.error('加载配置失败:', e)
  }
}

// 保存配置
async function saveConfig() {
  try {
    await invoke('invoke_external_plugin', {
      pluginId: 'translate',
      pluginName: 'translate_plugin.exe',
      cmd: 'save-config',
      payload: JSON.stringify({
        translationEngine: config.value.translationEngine,
        sourceLanguage: config.value.sourceLanguage,
        targetLanguage: config.value.targetLanguage,
      }),
    })
  } catch (e) {
    console.error('保存配置失败:', e)
  }
}

// 执行翻译
async function doTranslate() {
  if (!sourceText.value.trim()) {
    return
  }

  translating.value = true
  translatedText.value = ''

  try {
    const result = await invoke('invoke_external_plugin', {
      pluginId: 'translate',
      pluginName: 'translate_plugin.exe',
      cmd: 'translate',
      payload: JSON.stringify({
        text: sourceText.value,
        config: config.value,
      }),
    }) as string

    const response = JSON.parse(result)
    if (response.translation) {
      translatedText.value = response.translation
    } else {
      translatedText.value = $t('translateFailed')
    }
  } catch (e) {
    console.error('翻译失败:', e)
    translatedText.value = $t('translateFailed')
  } finally {
    translating.value = false
  }
}

// 交换语言
function swapLanguages() {
  if (config.value.sourceLanguage !== 'auto') {
    const temp = config.value.sourceLanguage
    config.value.sourceLanguage = config.value.targetLanguage
    config.value.targetLanguage = temp
    // 保存配置
    saveConfig()
  } else {
    // 如果源语言是自动，则只交换到源语言
    config.value.sourceLanguage = config.value.targetLanguage
    saveConfig()
  }
}

// 复制文本
async function copyText(text: string) {
  if (!text) return
  try {
    await writeText(text)
    message.success($t('copySuccess'))
  } catch (e) {
    console.error('复制失败:', e)
  }
}

// 监听重新加载事件
let reloadListener: any = null

async function initReloadListener() {
  reloadListener = await listen('reload-translate-text', async (event: any) => {
    console.log('加载新的翻译文本', event.payload)
    const itemId = event.payload.itemId;
    if (itemId) {
      // @ts-ignore
      sourceText.value = await window.getItemContent(itemId);
      // 更新窗口url
      window.history.replaceState(null, '', `${window.location.origin}${window.location.pathname}?pluginId=ocr&itemId=${itemId}`);
      // 自动翻译
      await doTranslate()
    }
  })
}

onMounted(async () => {
  await loadConfig()
  await initReloadListener()

  // 从 URL 参数获取文本
  const searchParams = new URLSearchParams(window.location.search)
  const itemId = searchParams.get('itemId')
  if (itemId) {
    console.log('加载新的翻译文本', itemId)
    // @ts-ignore
    sourceText.value = await window.getItemContent(itemId);
    await doTranslate()
  }
})

onUnmounted(() => {
  reloadListener?.();
})
</script>

<style scoped>
.translate-container {
  padding: 16px;
  display: flex;
  flex-direction: column;
  gap: 12px;
  height: 100vh;
  box-sizing: border-box;
}

.language-bar {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  padding-bottom: 8px;
  border-bottom: 1px solid var(--theme-universal-border);
}

.lang-select {
  width: 140px;
}

.swap-btn {
  padding: 0 8px;
}

.text-section {
  flex: 1;
  display: flex;
  flex-direction: column;
}

.section-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 8px;
}

.section-title {
  font-size: 14px;
  font-weight: 500;
  color: var(--theme-universal-text);
}

.text-input {
  flex: 1;
}

.text-input :deep(.n-input__textarea-el) {
  font-size: 14px;
  line-height: 1.6;
}

.action-bar {
  display: flex;
  justify-content: center;
  gap: 16px;
  padding: 4px 0;
}

.btn-icon {
  width: 16px;
  height: 16px;
  fill: currentColor;
}

.status-bar {
  font-size: 12px;
  color: var(--theme-universal-text);
  opacity: 0.6;
  text-align: center;
  padding-top: 4px;
}
</style>
