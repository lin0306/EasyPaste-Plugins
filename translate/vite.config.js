import {defineConfig} from 'vite'
import vue from '@vitejs/plugin-vue'

export default definePluginBundleConfig()

function definePluginBundleConfig() {
    return defineConfig({
        plugins: [
            vue(),
        ],
        assetsInclude: ['assets/*'],
        define: {
            'process.env.NODE_ENV': JSON.stringify(process.env.NODE_ENV || 'production')
        },
        build: {
            cssCodeSplit: false,
            lib: {
                entry: 'src/entry-web-component.ts',
                name: 'TranslatePlugin',
                fileName: () => 'translate.bundle.js',
                formats: ['iife']
            },
            rollupOptions: {
                external: [
                    'vue',
                ],
                output: {
                    globals: {
                        vue: 'Vue',
                        '@tauri-apps/api/core': 'window.__TAURI__.core',
                        '@tauri-apps/api': 'window.__TAURI__',
                        '@tauri-apps/api/event': 'window.__TAURI__.event',
                        '@tauri-apps/plugin-clipboard-manager': 'window.__TAURI__.clipboardManager',
                    }
                }
            }
        }
    })
}
