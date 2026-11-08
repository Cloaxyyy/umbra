<script setup lang="ts">
import { onMounted, onUnmounted } from 'vue'
import { api } from './api'
import Icon from './components/Icon.vue'
import TitleBar from './components/TitleBar.vue'
import type { IconName } from './icons'
import { Back, Forward, Go, RefreshAccounts, RefreshProfiles, RefreshRunning, store, Toast, type View } from './store'
import { CheckForUpdate } from './updater'
import Accounts from './views/Accounts.vue'
import Avatar from './views/Avatar.vue'
import Trades from './views/Trades.vue'
import Catalog from './views/Catalog.vue'
import Discover from './views/Discover.vue'
import Game from './views/Game.vue'
import Home from './views/Home.vue'
import Instances from './views/Instances.vue'
import Settings from './views/Settings.vue'

const NAV: { id: View; label: string; icon: IconName }[] = [
    { id: 'home', label: 'Launchpad', icon: 'rocket' },
    { id: 'discover', label: 'Discover', icon: 'gamepad' },
    { id: 'catalog', label: 'Catalog', icon: 'bag' },
    { id: 'avatar', label: 'Avatar', icon: 'shirt' },
    { id: 'trades', label: 'Trades', icon: 'swap' },
    { id: 'accounts', label: 'Accounts', icon: 'users' },
    { id: 'instances', label: 'Instances', icon: 'layers' },
]
const IsActive = (id: View) => store.route.view === id || (id === 'discover' && store.route.view === 'game')

function OnMouse(e: MouseEvent) {
    if (e.button === 3) Back()
    if (e.button === 4) Forward()
}
function OnKey(e: KeyboardEvent) {
    if (e.altKey && e.key === 'ArrowLeft') Back()
    if (e.altKey && e.key === 'ArrowRight') Forward()
}

let timer = 0
let unlisten: (() => void) | undefined
onMounted(async () => {
    window.addEventListener('mouseup', OnMouse)
    window.addEventListener('keydown', OnKey)
    timer = window.setInterval(RefreshRunning, 3000)
    unlisten = await api.onLaunchError((m) => Toast('bad', m))
    RefreshAccounts().catch((e) => Toast('bad', String(e)))
    RefreshProfiles()
    RefreshRunning()

    api.getSettings()
        .then((s) => {
            if (s.autoCheckUpdates) setTimeout(() => CheckForUpdate(false), 5000)
        })
        .catch(() => {})
})
onUnmounted(() => {
    window.removeEventListener('mouseup', OnMouse)
    window.removeEventListener('keydown', OnKey)
    clearInterval(timer)
    unlisten?.()
})
</script>

<template>
    <div class="app">
        <TitleBar />
        <div class="body">
            <aside class="rail" aria-label="Main">
                <nav class="rail-group">
                    <button
                        v-for="n in NAV"
                        :key="n.id"
                        class="rail-btn"
                        :class="{ active: IsActive(n.id) }"
                        :aria-label="n.label"
                        :title="n.label"
                        @click="Go({ view: n.id })"
                    >
                        <Icon :name="n.icon" :size="20" />
                    </button>
                </nav>
                <nav class="rail-group">
                    <button
                        class="rail-btn"
                        :class="{ active: IsActive('settings') }"
                        aria-label="Settings"
                        title="Settings"
                        @click="Go({ view: 'settings' })"
                    >
                        <Icon name="settings" :size="20" />
                    </button>
                </nav>
            </aside>
            <main class="content">
                <Home v-if="store.route.view === 'home'" />
                <Discover v-else-if="store.route.view === 'discover'" />
                <Game
                    v-else-if="store.route.view === 'game' && store.route.universeId"
                    :key="store.route.universeId"
                    :universe-id="store.route.universeId"
                />
                <Catalog v-else-if="store.route.view === 'catalog'" />
                <Avatar v-else-if="store.route.view === 'avatar'" />
                <Trades v-else-if="store.route.view === 'trades'" />
                <Accounts v-else-if="store.route.view === 'accounts'" />
                <Instances v-else-if="store.route.view === 'instances'" />
                <Settings v-else />
            </main>
        </div>
        <Transition name="toast">
            <div v-if="store.toast" :key="store.toast.id" class="toast" :class="store.toast.kind" role="status">{{ store.toast.text }}</div>
        </Transition>
    </div>
</template>

<style scoped>
.app {
    display: grid;
    grid-template-rows: auto 1fr;
    height: 100%;
}
.body {
    display: grid;
    grid-template-columns: 64px 1fr;
    min-height: 0;
}
.rail {
    display: flex;
    flex-direction: column;
    justify-content: space-between;
    align-items: center;
    padding: 14px 0;
    background: var(--bg-2);
    border-right: 1px solid var(--line);
}
.rail-group {
    display: flex;
    flex-direction: column;
    gap: 6px;
}
.rail-btn {
    width: 42px;
    height: 42px;
    border-radius: 11px;
    display: grid;
    place-items: center;
    color: var(--fg-3);
    transition:
        background 0.12s,
        color 0.12s;
    position: relative;
}
.rail-btn:hover:not(:disabled) {
    background: var(--surface);
    color: var(--fg);
}
.rail-btn.active {
    background: var(--surface-2);
    color: var(--fg);
}
.rail-btn.active::before {
    content: '';
    position: absolute;
    left: -11px;
    top: 11px;
    bottom: 11px;
    width: 3px;
    border-radius: 0 3px 3px 0;
    background: var(--grad);
}
.rail-btn:disabled {
    opacity: 0.4;
    cursor: default;
}
.content {
    overflow-y: auto;
    padding: 22px 26px 32px;
    min-width: 0;
}
.toast {
    position: fixed;
    right: 20px;
    bottom: 20px;
    max-width: 420px;
    padding: 12px 16px;
    border-radius: 12px;
    background: var(--surface-2);
    border: 1px solid var(--line-strong);
    box-shadow: 0 16px 40px rgba(0, 0, 0, 0.5);
    font-size: 13px;
    font-weight: 600;
    user-select: text;
    z-index: 50;
}
.toast.good {
    border-color: rgba(63, 224, 160, 0.35);
}
.toast.bad {
    border-color: rgba(255, 98, 131, 0.45);
    color: #ffc2cf;
}
.toast-enter-active,
.toast-leave-active {
    transition:
        opacity 0.2s,
        transform 0.2s;
}
.toast-enter-from,
.toast-leave-to {
    opacity: 0;
    transform: translateY(8px);
}
</style>
