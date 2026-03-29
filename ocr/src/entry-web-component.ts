import OCR from './OCR.vue'
import Settings from './Settings.vue'

// 延迟检查 naive-ui，给宿主环境一些初始化时间
setTimeout(() => {
  // @ts-ignore
  if (typeof window.naive === 'undefined') {
    console.warn('naive-ui 可能未正确加载，插件可能无法正常工作')
  } else {
    console.log('naive-ui 加载成功')
  }
}, 100)

// 挂载应用到指定容器
function mountApp(container: HTMLElement, component: any) {
  console.log('正在挂载组件:', component.name)
  // @ts-ignore
  const app = window.createPluginVueApp(container, component)
  if (!app) {
    console.error('创建 Vue 应用失败：createPluginVueApp 返回 null 或 undefined')
  }
  return app
}

// 添加样式
function addStyles() {
  const styleId = 'ocr-plugin-styles'
  if (!document.getElementById(styleId)) {
    const style = document.createElement('style')
    style.id = styleId
    style.textContent = `
      #plugin-container .n-divider:not(.n-divider--vertical) {
        margin-top: 0 !important;
        margin-bottom: 2px !important;
      }
      #plugin-container .n-input__suffix {
        align-items: flex-end !important;
      }
    `
    document.head.appendChild(style)
  }
}

// 尝试挂载的主逻辑
function tryMount() {
  // 检查 createPluginVueApp 是否存在
  // @ts-ignore
  if (typeof window.createPluginVueApp !== 'function') {
    console.warn('createPluginVueApp 方法尚未定义，等待宿主环境初始化...')
    return false
  }

  // 尝试挂载窗口组件
  const windowContainer = document.getElementById('plugin-container')
  if (windowContainer && !(windowContainer as any).__vue_app__) {
    console.log('找到窗口容器，挂载 OCR 组件')
    mountApp(windowContainer, OCR)
    addStyles()
    return true
  }

  // 尝试挂载设置组件（支持动态 ID）
  const settingsContainer =
    document.getElementById('plugin-settings-container') ||
    (document.querySelector('[id^="plugin-settings-container-"]') as HTMLElement)

  if (settingsContainer && !(settingsContainer as any).__vue_app__) {
    console.log('找到设置容器，挂载 Settings 组件:', settingsContainer.id)
    mountApp(settingsContainer, Settings)
    return true
  }

  return false
}

// 使用 MutationObserver 监听 DOM 变化
function initWithObserver() {
  // 先尝试一次立即挂载
  if (tryMount()) return

  console.log('未找到容器，开始监听 DOM 变化...')

  // 检查 naive-ui 是否可用
  // @ts-ignore
  if (typeof window.naive === 'undefined') {
    console.error('naive-ui 不可用，请确保主程序已正确注入')
    return
  }

  const observer = new MutationObserver(() => {
    const mounted = tryMount()
    if (mounted) {
      console.log('挂载成功，停止监听')
      observer.disconnect()
    }
  })

  observer.observe(document.body, {
    childList: true,
    subtree: true,
  })

  // 5秒后自动停止监听，防止内存泄漏
  setTimeout(() => {
    observer.disconnect()
    console.log('监听超时，停止观察')

    // 超时后再次检查
    console.log('最终状态检查:')
    console.log('- plugin-container:', document.getElementById('plugin-container'))
    // @ts-ignore
    console.log('- createPluginVueApp:', typeof window.createPluginVueApp)
    // @ts-ignore
    console.log('- naive:', typeof window.naive)
  }, 5000)
}

// 立即执行
initWithObserver()

// 暴露全局方法，供外部手动触发挂载
// @ts-ignore
window.__mountPluginApp = tryMount
