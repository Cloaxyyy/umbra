import { activeTheme, customThemes, CurrentTheme, features, LoadFeatures } from '@/utils/settings'
import { ThemeCss, type Theme } from '@/utils/themes'

const STYLE_ID = 'umbra-theme'
const ADS_ID = 'umbra-hide-ads'
const ADS_CSS = `#Skyscraper-Abp-Left, #Skyscraper-Abp-Right, #Leaderboard-Abp, .abp, .ad, iframe[src*="googlesyndication"], #sponsored-catalog-items, [data-testid="sponsored-carousel"], .sponsored-carousel { display: none !important; }`

function Upsert(id: string, css: string) {
    let el = document.getElementById(id) as HTMLStyleElement | null
    if (!css) {
        el?.remove()
        return
    }
    if (!el) {
        el = document.createElement('style')
        el.id = id
        ;(document.head ?? document.documentElement).append(el)
    }
    if (el.textContent !== css) el.textContent = css
}

const MODE_CLASSES = ['dark-theme', 'light-theme', 'color-mode-dark', 'color-mode-light']
let mode: Theme['dark'] | null = null
let watcher: MutationObserver | null = null
let original: string[] | null = null

function ApplyMode() {
    const b = document.body
    if (!b) return
    if (mode === null) {
        if (original) {
            b.classList.remove(...MODE_CLASSES)
            b.classList.add(...original)
            original = null
        }
        return
    }
    if (!original) original = MODE_CLASSES.filter((c) => b.classList.contains(c))
    const want = mode ? ['dark-theme', 'color-mode-dark'] : ['light-theme', 'color-mode-light']
    const drop = mode ? ['light-theme', 'color-mode-light'] : ['dark-theme', 'color-mode-dark']
    if (drop.some((c) => b.classList.contains(c)) || want.some((c) => !b.classList.contains(c))) {
        b.classList.remove(...drop)
        b.classList.add(...want)
    }
}

function WatchBody() {
    if (watcher) return
    const start = () => {
        ApplyMode()
        watcher = new MutationObserver(ApplyMode)
        watcher.observe(document.body, { attributes: true, attributeFilter: ['class'] })
    }
    if (document.body) start()
    else document.addEventListener('DOMContentLoaded', start, { once: true })
}

async function Apply() {
    const t = await CurrentTheme()
    Upsert(STYLE_ID, t ? ThemeCss(t) : '')
    mode = t ? t.dark : null
    if (t) WatchBody()
    else ApplyMode()
    const f = await LoadFeatures()
    Upsert(ADS_ID, f.hideAds ? ADS_CSS : '')
}

export default defineContentScript({
    matches: ['*://*.roblox.com/*'],
    runAt: 'document_start',
    main() {
        Apply()
        activeTheme.watch(Apply)
        customThemes.watch(Apply)
        features.watch(Apply)
    },
})
