<script setup lang="ts">
import { onMounted, onUnmounted, ref } from 'vue'
import { api, type BridgeRequest, type Disconnect } from './api'
import CommandPalette from './components/CommandPalette.vue'
import Icon from './components/Icon.vue'
import TitleBar from './components/TitleBar.vue'
import type { IconName } from './icons'
import { LoadAllowlist } from './flags'
import {
    Back,
    Forward,
    Go,
    RefreshAccounts,
    RefreshFavorites,
    RefreshProfiles,
    RefreshRunning,
    RunToastAction,
    StartFriendAlerts,
    StartScheduler,
    StartUpdateAlerts,
    store,
    Toast,
    type View,
} from './store'
import { ApplyAccent } from './theme'
import { CheckForUpdate } from './updater'
import Accounts from './views/Accounts.vue'
import Avatar from './views/Avatar.vue'
import Trades from './views/Trades.vue'
import Catalog from './views/Catalog.vue'
import Discover from './views/Discover.vue'
import Friends from './views/Friends.vue'
import Game from './views/Game.vue'
import Home from './views/Home.vue'
import Instances from './views/Instances.vue'
import Settings from './views/Settings.vue'
import Stats from './views/Stats.vue'

const NAV: { id: View; label: string; icon: IconName }[] = [
    { id: 'home', label: 'Launchpad', icon: 'rocket' },
    { id: 'discover', label: 'Discover', icon: 'gamepad' },
    { id: 'friends', label: 'Friends', icon: 'users' },
    { id: 'catalog', label: 'Catalog', icon: 'bag' },
    { id: 'avatar', label: 'Avatar', icon: 'shirt' },
    { id: 'trades', label: 'Trades', icon: 'swap' },
    { id: 'accounts', label: 'Accounts', icon: 'idcard' },
    { id: 'instances', label: 'Instances', icon: 'layers' },
    { id: 'stats', label: 'Stats', icon: 'activity' },
]
const IsActive = (id: View) => store.route.view === id || (id === 'discover' && store.route.view === 'game')

function OnMouse(e: MouseEvent) {
    if (e.button === 3) Back()
    if (e.button === 4) Forward()
}
function OnKey(e: KeyboardEvent) {
    if (e.altKey && e.key === 'ArrowLeft') Back()
    if (e.altKey && e.key === 'ArrowRight') Forward()
    if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === 'k') {
        e.preventDefault()
        store.palette = !store.palette
    }
}

let timer = 0
let unlisten: (() => void) | undefined
let unPair: (() => void) | undefined
let unFav: (() => void) | undefined
let unRejoin: (() => void) | undefined
let unDropped: (() => void) | undefined
const AccountName = (id: number) => store.accounts.find((a) => a.userId === id)?.displayName ?? 'An account'
function OnRejoin(d: Disconnect) {
    const who = AccountName(d.userId)
    if (d.action === 'rejoining') Toast('good', `${who} was disconnected (${d.reason}). Rejoining…`)
    else if (d.action === 'paused') Toast('bad', `Auto-rejoin paused for ${who} after too many tries. Last reason: ${d.reason}`)
    else Toast('bad', `${who}: ${d.reason}`)
}
function OnDropped(d: Disconnect) {
    const pid = store.running.find((r) => r.userId === d.userId)?.pid
    Toast(
        'bad',
        `${AccountName(d.userId)} was disconnected from ${d.gameName ?? 'a game'}: ${d.reason}`,
        pid ? { label: 'Rejoin', run: () => api.moveClient(pid, 'to', d.placeId).catch((e) => Toast('bad', String(e))) } : undefined,
    )
}
const pairing = ref<BridgeRequest | null>(null)
async function AnswerPair(allow: boolean) {
    const r = pairing.value
    pairing.value = null
    if (!r) return
    try {
        await api.bridgeAnswer(r.id, allow)
        Toast('good', allow ? 'The browser extension is connected.' : 'Connection refused.')
    } catch (e) {
        Toast('bad', String(e))
    }
}
onMounted(async () => {
    window.addEventListener('mouseup', OnMouse)
    window.addEventListener('keydown', OnKey)
    timer = window.setInterval(RefreshRunning, 3000)
    unlisten = await api.onLaunchError((m) => Toast('bad', m))
    unPair = await api.onBridgePair((r) => (pairing.value = r))
    unFav = await api.onFavoritesChanged(() => RefreshFavorites())
    unRejoin = await api.onAutoRejoin(OnRejoin)
    unDropped = await api.onClientDropped(OnDropped)
    StartScheduler()
    RefreshAccounts().catch((e) => Toast('bad', String(e)))
    RefreshProfiles()
    RefreshRunning()
    RefreshFavorites()
    LoadAllowlist()

    api.getSettings()
        .then((s) => {
            ApplyAccent(s.accent)
            StartFriendAlerts(s.friendAlerts)
            setTimeout(() => StartUpdateAlerts(s.updateAlerts), 8000)
            if (s.autoCheckUpdates) setTimeout(() => CheckForUpdate(false), 5000)
        })
        .catch(() => {})
})
onUnmounted(() => {
    window.removeEventListener('mouseup', OnMouse)
    window.removeEventListener('keydown', OnKey)
    clearInterval(timer)
    StartFriendAlerts(false)
    unlisten?.()
    unPair?.()
    unFav?.()
    unRejoin?.()
    unDropped?.()
    StartUpdateAlerts(false)
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
                <Transition name="page" mode="out-in">
                    <Home v-if="store.route.view === 'home'" />
                    <Discover v-else-if="store.route.view === 'discover'" />
                    <Friends v-else-if="store.route.view === 'friends'" />
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
                    <Stats v-else-if="store.route.view === 'stats'" />
                    <Settings v-else />
                </Transition>
            </main>
        </div>
        <CommandPalette v-if="store.palette" />
        <div v-if="pairing" class="pair-scrim">
            <div class="pair card" role="alertdialog" aria-modal="true" aria-labelledby="pair-title" aria-describedby="pair-desc">
                <h2 id="pair-title">Connect the browser extension?</h2>
                <p id="pair-desc" class="muted">
                    <b>{{ pairing.label }}</b> wants to connect to Umbra. It will see your account names and running clients, launch games
                    with your accounts, control clients and change favorites. It never sees your Roblox sessions.
                </p>
                <p class="faint small mono">{{ pairing.origin }}</p>
                <p class="faint small">Only allow this if you just pressed Connect in the Umbra extension.</p>
                <div class="confirm">
                    <button class="btn btn-danger" @click="AnswerPair(false)">Deny</button>
                    <button class="btn" @click="AnswerPair(true)">Allow</button>
                </div>
            </div>
        </div>
        <Transition name="toast">
            <div v-if="store.toast" :key="store.toast.id" class="toast" :class="store.toast.kind" role="status">
                <span>{{ store.toast.text }}</span>
                <button v-if="store.toast.action" class="btn btn-sm" @click="RunToastAction">{{ store.toast.action.label }}</button>
            </div>
        </Transition>
    </div>
</template>

<style scoped>
.pair-scrim {
    position: fixed;
    inset: 0;
    z-index: 70;
    display: grid;
    place-items: center;
    padding: 20px;
    background: rgba(0, 0, 0, 0.65);
    animation: fade-in 0.15s ease both;
}
.pair {
    width: min(460px, 100%);
    display: flex;
    flex-direction: column;
    gap: 12px;
    padding: 22px;
    animation: pop 0.22s var(--ease-out) both;
}
.pair h2 {
    font-size: 18px;
}
.pair .confirm {
    margin-top: 6px;
}
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
    background: var(--accent);
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
    background: var(--panel);
    border: 1px solid var(--line-strong);
    box-shadow: 0 16px 40px rgba(0, 0, 0, 0.5);
    font-size: 13px;
    font-weight: 600;
    user-select: text;
    z-index: 50;
    display: flex;
    align-items: center;
    gap: 12px;
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
