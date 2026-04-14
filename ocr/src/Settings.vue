<template>
  <div class="ocr-settings">
    <n-divider title-placement="left">{{ language.pages.plugins.ocr.settingsTitle }}</n-divider>

    <!-- OCR 模式选择 -->
    <div class="setting-item">
      <div class="setting-label">
        <span>{{ language.pages.plugins.ocr.ocrMode }}</span>
        <n-tooltip trigger="hover">
          <template #trigger>
            <svg class="hint-icon" viewBox="0 0 1024 1024">
              <path
                  d="M512 64C264.6 64 64 264.6 64 512s200.6 448 448 448 448-200.6 448-448S759.4 64 512 64z m0 820c-205.4 0-372-166.6-372-372s166.6-372 372-372 372 166.6 372 372-166.6 372-372 372z"
                  fill="currentColor"/>
              <path d="M512 336m-40 0a40 40 0 1 0 80 0 40 40 0 1 0-80 0Z" fill="currentColor"/>
              <path d="M536 448h-48c-4.4 0-8 3.6-8 8v272c0 4.4 3.6 8 8 8h48c4.4 0 8-3.6 8-8V456c0-4.4-3.6-8-8-8z"
                    fill="currentColor"/>
            </svg>
          </template>
          <span>{{ language.pages.plugins.ocr.ocrModeHint }}</span>
        </n-tooltip>
      </div>
      <n-select
          v-model:value="config.ocrMode"
          :options="ocrModeOptions"
          class="setting-select"
      />
    </div>

    <!-- 保存按钮 -->
    <div class="setting-actions">
      <n-button type="primary" :loading="saving" @click="saveConfig">
        {{ language.pages.plugins.ocr.saveBtn }}
      </n-button>
    </div>

    <!-- 说明 -->
    <n-alert type="info" class="info-hint">
      <template #header>
        {{ language.pages.plugins.ocr.infoTitle }}
      </template>
      <p>{{ language.pages.plugins.ocr.desc }}</p>
      <ul>
        <li>{{ language.pages.plugins.ocr.rule1 }}</li>
        <li>{{ language.pages.plugins.ocr.rule2 }}</li>
        <li>{{ language.pages.plugins.ocr.rule3 }}</li>
        <li>{{ language.pages.plugins.ocr.rule4 }}</li>
      </ul>
    </n-alert>
  </div>
</template>

<script setup lang="ts">
import {computed, onMounted, ref} from 'vue'
import {NAlert, NButton, NDivider, NSelect, NTooltip, useMessage} from 'naive-ui'
import {invoke} from '@tauri-apps/api/core'

const message = useMessage()

// 配置状态
const config = ref({
  ocrMode: 'window',
  ocrLanguage: 'chi_sim+eng',
})

const saving = ref(false)

// @ts-ignore
const language = computed(() => window.currentLanguage?.value || window.currentLanguage)

// OCR 模式选项
const ocrModeOptions = computed(() => [
  {label: language.value.pages.plugins.ocr.windowModel, value: 'window'},
  {label: language.value.pages.plugins.ocr.quickModel, value: 'quick'},
])

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
      config.value = {...config.value, ...savedConfig}
    }
  } catch (e) {
    console.error('加载配置失败:', e)
    message.error(language.value.pages.plugins.ocr.loadFailed)
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
      message.success(language.value.pages.plugins.ocr.saveSuccess)
    } else {
      message.error(language.value.pages.plugins.ocr.saveFailed)
    }
  } catch (e) {
    console.error('保存配置失败:', e)
    message.error(language.value.pages.plugins.ocr.saveFailed)
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
