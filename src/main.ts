import { createApp } from 'vue'
import App from './App.vue'
import Overlay from './Overlay.vue'
import './style.css'

createApp(location.hash === '#overlay' ? Overlay : App).mount('#app')
