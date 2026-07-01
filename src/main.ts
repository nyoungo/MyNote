import { createApp } from 'vue'
import { createPinia } from 'pinia'
import piniaPluginPersistedstate from 'pinia-plugin-persistedstate'
import Antd from 'ant-design-vue'
import 'ant-design-vue/dist/reset.css'
import App from './App.vue'
import router from './router'
import './styles/main.css'

/**
 * Vue 应用入口
 * 初始化 Pinia（带持久化插件）、Vue Router、ant-design-vue
 */
const app = createApp(App)

// Pinia 状态管理（带 localStorage 持久化）
const pinia = createPinia()
pinia.use(piniaPluginPersistedstate)

app.use(pinia)
app.use(router)
app.use(Antd)
app.mount('#app')
