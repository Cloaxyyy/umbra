import '@/components/ui.css'
import { createApp } from 'vue'
import { currentUrl } from '@/utils/page'
import App from './App.vue'

export default defineContentScript({
    matches: ['*://www.roblox.com/*', '*://web.roblox.com/*'],
    cssInjectionMode: 'ui',
    async main(ctx) {
        ctx.addEventListener(window, 'wxt:locationchange', ({ newUrl }) => (currentUrl.value = newUrl.href))
        const ui = await createShadowRootUi(ctx, {
            name: 'umbra-extension',
            position: 'inline',
            anchor: 'body',
            append: 'last',
            onMount(container) {
                const app = createApp(App)
                app.mount(container)
                return app
            },
            onRemove(app) {
                app?.unmount()
            },
        })
        ui.mount()
    },
})
