import { createApp } from "vue";
import App from "./App.vue";
import router from './router';
import './styles.css';
import '@imengyu/vue3-context-menu/lib/vue3-context-menu.css'
import ContextMenu from '@imengyu/vue3-context-menu'

createApp(App)
  .use(router)
  .use(ContextMenu)  
  .mount('#app');
