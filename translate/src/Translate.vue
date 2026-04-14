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
      <n-button text size="small" @click="swapLanguages" class="swap-btn" :disabled="config.sourceLanguage===`auto`">
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
        <span class="section-title">{{ language.pages.plugins.translate.sourceText }}</span>
        <n-button text size="small" @click="copyText(sourceText)">
          <template #icon>
            <font-awesome-icon :icon="faCopy" class="btn-icon"/>
          </template>
          {{ language.pages.plugins.translate.copy }}
        </n-button>
      </div>
      <n-input
          v-model:value="sourceText"
          type="textarea"
          :rows="4"
          :placeholder="language.pages.plugins.translate.sourcePlaceholder"
          class="text-input"
      />
    </div>

    <!-- 操作栏 -->
    <div class="action-bar">
      <n-button type="primary" :loading="translating" @click="doTranslate">
        {{ language.pages.plugins.translate.translate }}
      </n-button>
    </div>

    <!-- 翻译结果 -->
    <div class="text-section">
      <div class="section-header">
        <span class="section-title">{{ language.pages.plugins.translate.translatedText }}</span>
        <n-button text size="small" @click="copyText(translatedText)">
          <template #icon>
            <font-awesome-icon :icon="faCopy" class="btn-icon"/>
          </template>
          {{ language.pages.plugins.translate.copy }}
        </n-button>
      </div>
      <n-input
          v-model:value="translatedText"
          type="textarea"
          :rows="4"
          :placeholder="language.pages.plugins.translate.translatedPlaceholder"
          class="text-input"
          readonly
      />
    </div>

    <!-- 状态栏 -->
    <div class="status-bar">
      <span v-if="config.translationEngine">
        {{ language.pages.plugins.translate.engine }}: {{ engineName }}
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

// @ts-ignore
const language = computed(() => window.currentLanguage?.value || window.currentLanguage)

// 源语言选项（包含自动检测）
const sourceLangOptions = computed(() => [
  {label: language.value.pages.plugins.translate.auto, value: 'auto'},
  {label: language.value.pages.plugins.translate.zh, value: 'zh'},
  {label: language.value.pages.plugins.translate.en, value: 'en'},
  {label: language.value.pages.plugins.translate.ja, value: 'ja'},
  {label: language.value.pages.plugins.translate.ko, value: 'ko'},
  {label: language.value.pages.plugins.translate.fr, value: 'fr'},
  {label: language.value.pages.plugins.translate.de, value: 'de'},
  {label: language.value.pages.plugins.translate.es, value: 'es'},
  {label: language.value.pages.plugins.translate.ru, value: 'ru'},
])

// 目标语言选项
const targetLangOptions = computed(() => [
  {label: language.value.pages.plugins.translate.zh, value: 'zh'},
  {label: language.value.pages.plugins.translate.en, value: 'en'},
  {label: language.value.pages.plugins.translate.ja, value: 'ja'},
  {label: language.value.pages.plugins.translate.ko, value: 'ko'},
  {label: language.value.pages.plugins.translate.fr, value: 'fr'},
  {label: language.value.pages.plugins.translate.de, value: 'de'},
  {label: language.value.pages.plugins.translate.es, value: 'es'},
  {label: language.value.pages.plugins.translate.ru, value: 'ru'},
])

/**
 * 计算属性
 */
const engineName = computed(() => {
  const names: Record<string, string> = {
    google: language.value.pages.plugins.translate.GoogleTranslate,
    deepl: language.value.pages.plugins.translate.DeepL,
    baidu: language.value.pages.plugins.translate.BaiDuTranslate,
    youdao: language.value.pages.plugins.translate.YouDaoTranslate,
  }
  return names[config.value.translationEngine] || config.value.translationEngine
})

/**
 * 执行翻译
 */
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
      translatedText.value = language.value.pages.plugins.translate.translateFailed
    }
  } catch (e) {
    console.error('翻译失败:', e)
    translatedText.value = language.value.pages.plugins.translate.translateFailed
  } finally {
    translating.value = false
  }
}

/**
 * 交换语言
 */
function swapLanguages() {
  if (config.value.sourceLanguage !== 'auto') {
    const temp = config.value.sourceLanguage
    config.value.sourceLanguage = config.value.targetLanguage
    config.value.targetLanguage = temp
  } else {
    // 如果源语言是自动，则只交换到源语言
    config.value.sourceLanguage = config.value.targetLanguage
  }
}

/**
 * 复制文本
 * @param text 要复制的文本
 */
async function copyText(text: string) {
  if (!text) return
  try {
    await writeText(text)
    message.success(language.value.pages.plugins.translate.copySuccess)
  } catch (e) {
    console.error('复制失败:', e)
  }
}

/**
 * 加载当前设置的翻译引擎
 */
async function loadEngine() {
  const configResult = await invoke('invoke_external_plugin', {
    pluginId: 'translate',
    pluginName: 'translate_plugin.exe',
    cmd: 'get-config',
    payload: '{}',
  }) as string

  const configResponse = JSON.parse(configResult)
  if (configResponse.result) {
    const savedConfig = JSON.parse(configResponse.result)
    config.value.translationEngine = savedConfig.translationEngine
  }
}

/**
 * 初始化重载监听器
 */
let reloadListener: any = null

async function initReloadListener(): Promise<void> {
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

/**
 * 初始化翻译引擎切换监听器
 */
let changeTranslateEngineListener: any = null
async function initChangeTranslateEngineListener(): Promise<void> {
  changeTranslateEngineListener = await listen('change-translate-engine', async (_event: any) => {
    console.log('翻译引擎已切换')
    await loadEngine();
    console.log(config.value)
    await doTranslate()
  })
}

onMounted(async () => {
  // 加载当前翻译引擎
  await loadEngine();
  // 监听数据重新加载事件
  await initReloadListener();
  // 监听翻译引擎切换事件
  await initChangeTranslateEngineListener();

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
  changeTranslateEngineListener?.();
})
</script>

<style scoped>
.translate-container {
  padding: 16px;
  display: flex;
  flex-direction: column;
  gap: 12px;
  height: calc(100vh - 25px);
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
