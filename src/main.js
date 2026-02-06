import { createApp } from 'vue'
import ElementPlus from 'element-plus'
import 'element-plus/dist/index.css'
import * as ElementPlusIconsVue from '@element-plus/icons-vue'
import App from './App.vue'

const app = createApp(App)

// Register all Element Plus icons
for (const [key, component] of Object.entries(ElementPlusIconsVue)) {
  app.component(key, component)
}

app.use(ElementPlus)
app.mount('#app')

if (typeof window !== 'undefined' && window.__TAURI__) {
  import('@tauri-apps/api/window')
    .then(({ appWindow }) => appWindow.maximize())
    .catch(() => {})
}
