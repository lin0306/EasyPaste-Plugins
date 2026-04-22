<template>
  <div class="translate-settings">
    <n-divider title-placement="left">{{ language.pages.plugins.translate.translationSettingsTitle }}</n-divider>

    <!-- 翻译引擎 -->
    <div class="setting-item">
      <div class="setting-label">
        <span>{{ language.pages.plugins.translate.translationEngine }}</span>
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
    <div class="setting-item" v-if="currentEngine && currentEngine !== 'microsoft'">
      <div class="setting-label">
        <span>{{ language.pages.plugins.translate.apiKey }}</span>
        <n-tooltip trigger="hover">
          <template #trigger>
            <font-awesome-icon :icon="faCircleQuestion" class="hint-icon" />
          </template>
          <span>{{ getApiKeyHint() }}</span>
        </n-tooltip>
      </div>
      <n-input
        v-model:value="apiKeys[currentEngine]"
        type="password"
        :show-password-on="'click'"
        :placeholder="language.pages.plugins.translate.apiKeyPlaceholder"
        class="setting-input"
      />
    </div>

    <!-- 保存按钮 -->
    <div class="setting-actions">
      <n-button type="primary" :loading="saving" @click="saveConfig">
        {{ language.pages.plugins.translate.saveBtn }}
      </n-button>
    </div>

    <!-- API 说明 -->
    <n-alert type="info" class="api-hint" v-if="currentEngine && currentEngine !== 'microsoft'">
      <template #header>
        {{ language.pages.plugins.translate.apiHintTitle }}
      </template>
      <div v-if="currentEngine === `google`">
        <p><b>{{ language.pages.plugins.translate.google.descTitle }}</b></p>
        <ul>
          <li>{{ language.pages.plugins.translate.google.require }}</li>
          <li v-html="language.pages.plugins.translate.google.access"></li>
          <li>{{ language.pages.plugins.translate.google.limit }}</li>
        </ul>
      </div>
      <div v-if="currentEngine === `deepl`">
        <p><b>{{ language.pages.plugins.translate.deepl.descTitle }}</b></p>
        <ul>
          <li>{{ language.pages.plugins.translate.deepl.require }}</li>
          <li v-html="language.pages.plugins.translate.deepl.access"></li>
          <li>{{ language.pages.plugins.translate.deepl.limit }}</li>
        </ul>
      </div>
      <div v-if="currentEngine === `baidu`">
        <p><b>{{ language.pages.plugins.translate.baidu.descTitle }}</b></p>
        <ul>
          <li>{{ language.pages.plugins.translate.baidu.require }}</li>
          <li v-html="language.pages.plugins.translate.baidu.access"></li>
          <li>{{ language.pages.plugins.translate.baidu.limit }}</li>
          <li v-html="language.pages.plugins.translate.baidu.formatRequire"></li>
        </ul>
      </div>
      <div v-if="currentEngine === `youdao`">
        <p><b>{{ language.pages.plugins.translate.youdao.descTitle }}</b></p>
        <ul>
          <li>{{ language.pages.plugins.translate.youdao.require }}</li>
          <li v-html="language.pages.plugins.translate.youdao.access"></li>
          <li>{{ language.pages.plugins.translate.youdao.limit }}</li>
          <li v-html="language.pages.plugins.translate.youdao.formatRequire"></li>
        </ul>
      </div>
    </n-alert>
  </div>
</template>

<script setup lang="ts">
import {computed, onMounted, ref} from 'vue'
import {NAlert, NButton, NDivider, NInput, NSelect, NTooltip, useMessage} from 'naive-ui'
import {invoke} from '@tauri-apps/api/core'
import {faCircleQuestion} from "@fortawesome/free-regular-svg-icons";
import {emit} from "@tauri-apps/api/event";

const message = useMessage()

// @ts-ignore
const language = computed(() => window.currentLanguage?.value || window.currentLanguage)

// 当前选中的引擎
const currentEngine = ref('microsoft')

// 每个引擎的API key
const apiKeys = ref<Record<string, string>>({
  google: '',
  deepl: '',
  baidu: '',
  youdao: '',
})

const saving = ref(false)

// 引擎选项
const engineOptions = computed(() => [
  {label: language.value.pages.plugins.translate.MicrosoftTranslate, value: 'microsoft'},
  {label: language.value.pages.plugins.translate.GoogleTranslate, value: 'google'},
  {label: language.value.pages.plugins.translate.DeepL, value: 'deepl'},
  {label: language.value.pages.plugins.translate.BaiDuTranslate, value: 'baidu'},
  {label: language.value.pages.plugins.translate.YouDaoTranslate, value: 'youdao'},
]);

// 获取当前引擎的API Key提示
const getApiKeyHint = () => {
  const hints: Record<string, string> = {
    google: language.value.pages.plugins.translate.googleApiKeyHint,
    deepl: language.value.pages.plugins.translate.deeplApiKeyHint,
    baidu: language.value.pages.plugins.translate.baiduApiKeyHint,
    youdao: language.value.pages.plugins.translate.youdaoApiKeyHint,
  }
  return hints[currentEngine.value] || ''
}

/**
 * 引擎切换
 * @param engine
 */
async function onEngineChange(engine: string) {
  console.log('切换到引擎:', engine)
}


// 保存配置
async function saveConfig() {
  saving.value = true
  try {
    // 保存引擎设置
    await invoke('invoke_external_plugin', {
      pluginId: 'translate',
      pluginName: 'translate_plugin.exe',
      cmd: 'save-config',
      payload: JSON.stringify({
        translationEngine: currentEngine.value,
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
      console.log('保存API Keys成功', currentEngine, apiKeys)
      await emit('change-translate-engine');
      message.success(language.value.pages.plugins.translate.saveSuccess)
    } else {
      message.error(language.value.pages.plugins.translate.saveFailed)
    }
  } catch (e) {
    console.error('保存配置失败:', e)
    message.error(language.value.pages.plugins.translate.saveFailed)
  } finally {
    saving.value = false
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
    currentEngine.value = savedConfig.translationEngine || 'microsoft'
  }
}

/**
 * 加载API Keys
 */
async function loadApiKeys() {
  try {
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
      apiKeys.value = {...apiKeys.value, ...savedKeys}
    }
  } catch (e) {
    console.error('加载API Keys失败:', e)
    message.error(language.value.pages.plugins.translate.loadFailed)
  }
}

/**
 * 加载翻译配置
 */
async function loadConfig() {
  try {
    // 加载引擎设置
    await loadEngine();
    // 加载API Keys
    await loadApiKeys();
  } catch (e) {
    console.error('加载配置失败:', e)
    message.error(language.value.pages.plugins.translate.loadFailed)
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
