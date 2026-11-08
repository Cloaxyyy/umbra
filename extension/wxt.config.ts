import { defineConfig } from 'wxt'

export default defineConfig({
    modules: ['@wxt-dev/module-vue'],
    manifest: {
        name: 'Umbra for Roblox',
        description: 'Custom themes, a better server browser, item values and one-click launches with the Umbra desktop app.',
        permissions: ['storage'],
        host_permissions: ['https://*.roblox.com/*', 'https://api.rolimons.com/*', 'http://127.0.0.1:27150/*'],
        action: { default_title: 'Umbra' },
    },
})
