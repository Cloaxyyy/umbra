import { defineConfig, type Plugin } from 'vite'
import vue from '@vitejs/plugin-vue'

const CSP = [
    "default-src 'self'",
    "script-src 'self'",
    "style-src 'self'",
    "img-src 'self' data:",
    "font-src 'self' data:",
    "connect-src 'none'",
    "object-src 'none'",
    "base-uri 'none'",
    "form-action 'none'",
].join('; ')

function ContentSecurityPolicy(): Plugin {
    return {
        name: 'umbra-csp',
        apply: 'build',
        transformIndexHtml: () => [
            { tag: 'meta', attrs: { 'http-equiv': 'Content-Security-Policy', content: CSP }, injectTo: 'head-prepend' },
        ],
    }
}

export default defineConfig({
    base: './',
    plugins: [vue(), ContentSecurityPolicy()],
    server: { port: 5180, strictPort: true },
})
