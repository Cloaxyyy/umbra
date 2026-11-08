<script setup lang="ts">
import { getCurrentWindow } from '@tauri-apps/api/window'
import { computed, onMounted, onUnmounted, ref } from 'vue'
import { ActiveAccount, AddAccount, Back, Forward, Go, store } from '../store'
import { InstallUpdate, updates } from '../updater'
import Icon from './Icon.vue'

const win = getCurrentWindow()
const menuOpen = ref(false)
const menuEl = ref<HTMLElement | null>(null)

const LABELS: Record<string, string> = {
    home: 'Launchpad',
    discover: 'Discover',
    friends: 'Friends',
    catalog: 'Catalog',
    avatar: 'Avatar',
    trades: 'Trades',
    accounts: 'Accounts',
    instances: 'Instances',
    settings: 'Settings',
}
const crumbs = computed(() => {
    const r = store.route
    if (r.view === 'game') return [{ label: 'Discover', to: { view: 'discover' as const } }, { label: r.name ?? 'Game' }]
    return [{ label: LABELS[r.view] }]
})
const active = computed(ActiveAccount)

function Pick(id: number) {
    store.activeId = id
    menuOpen.value = false
}
function MenuAdd() {
    menuOpen.value = false
    AddAccount()
}
function MenuManage() {
    menuOpen.value = false
    Go({ view: 'accounts' })
}
function OnDocClick(e: MouseEvent) {
    if (menuEl.value && !menuEl.value.contains(e.target as Node)) menuOpen.value = false
}
onMounted(() => document.addEventListener('mousedown', OnDocClick))
onUnmounted(() => document.removeEventListener('mousedown', OnDocClick))
</script>

<template>
    <header class="titlebar" data-tauri-drag-region>
        <div class="left" data-tauri-drag-region>
            <div class="brand" data-tauri-drag-region>
                <img src="/logo.png" alt="" class="logo" data-tauri-drag-region />
                <span data-tauri-drag-region>Umbra</span>
            </div>
            <button class="nav-btn" :disabled="!store.back.length" aria-label="Back" @click="Back"><Icon name="left" :size="16" /></button>
            <button class="nav-btn" :disabled="!store.fwd.length" aria-label="Forward" @click="Forward">
                <Icon name="right" :size="16" />
            </button>
            <nav class="crumbs" aria-label="Breadcrumb" data-tauri-drag-region>
                <template v-for="(c, i) in crumbs" :key="i">
                    <Icon v-if="i" name="chevronRight" :size="13" class="sep" />
                    <button v-if="c.to" class="crumb" @click="Go(c.to)">{{ c.label }}</button>
                    <span v-else class="crumb current" data-tauri-drag-region>{{ c.label }}</span>
                </template>
            </nav>
        </div>

        <div class="right">
            <button class="find" title="Search games, players, items and pages" @click="store.palette = true">
                <Icon name="search" :size="14" />Search<kbd>Ctrl K</kbd>
            </button>
            <button
                v-if="updates.available"
                class="update"
                :disabled="updates.installing"
                :title="'Install Umbra ' + updates.available.version + ' and restart'"
                @click="InstallUpdate"
            >
                <Icon name="upload" :size="13" />{{ updates.installing ? 'Updating…' : 'Update to ' + updates.available.version }}
            </button>
            <button class="status" :class="{ live: store.running.length }" title="Roblox clients running" @click="Go({ view: 'accounts' })">
                <span class="dot"></span>{{ store.running.length }} running
                <Icon name="activity" :size="14" />
            </button>

            <div ref="menuEl" class="acct">
                <button class="acct-btn" :aria-expanded="menuOpen" aria-haspopup="menu" @click="menuOpen = !menuOpen">
                    <img v-if="active?.avatar" :src="active.avatar" alt="" class="av" />
                    <span v-else class="av blank"><Icon name="user" :size="13" /></span>
                    <span class="acct-name">{{ active ? active.displayName : 'No account' }}</span>
                    <Icon name="chevronDown" :size="14" />
                </button>
                <div v-if="menuOpen" class="menu" role="menu">
                    <p v-if="store.accounts.length" class="eyebrow menu-head">Play as</p>
                    <button
                        v-for="a in store.accounts"
                        :key="a.userId"
                        class="menu-item"
                        role="menuitemradio"
                        :aria-checked="a.userId === active?.userId"
                        @click="Pick(a.userId)"
                    >
                        <img v-if="a.avatar" :src="a.avatar" alt="" class="av" /><span v-else class="av blank"></span>
                        <span class="mi-text"
                            ><b>{{ a.displayName }}</b
                            ><span class="faint">@{{ a.name }}</span></span
                        >
                        <span v-if="a.userId === active?.userId" class="tick" aria-hidden="true"></span>
                    </button>
                    <div v-if="store.accounts.length" class="menu-sep"></div>
                    <button class="menu-item" role="menuitem" :disabled="store.adding" @click="MenuAdd">
                        <Icon name="plus" :size="15" />{{ store.adding ? 'Waiting for login…' : 'Add account' }}
                    </button>
                    <button class="menu-item" role="menuitem" @click="MenuManage"><Icon name="users" :size="15" />Manage accounts</button>
                </div>
            </div>

            <div class="win">
                <button aria-label="Minimize" @click="win.minimize()"><Icon name="minimize" :size="14" /></button>
                <button aria-label="Maximize" @click="win.toggleMaximize()"><Icon name="maximize" :size="12" /></button>
                <button class="close" aria-label="Close" @click="win.close()"><Icon name="close" :size="15" /></button>
            </div>
        </div>
    </header>
</template>

<style scoped>
.titlebar {
    height: 46px;
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding-left: 14px;
    background: var(--bg-2);
    border-bottom: 1px solid var(--line);
    position: relative;
    z-index: 20;
}
.left,
.right {
    display: flex;
    align-items: center;
    gap: 6px;
    height: 100%;
    min-width: 0;
}
.brand {
    display: flex;
    align-items: center;
    gap: 8px;
    font-weight: 800;
    font-size: 13px;
    margin-right: 10px;
    letter-spacing: 0.01em;
}
.logo {
    width: 20px;
    height: 20px;
}
.nav-btn {
    width: 28px;
    height: 28px;
    border-radius: 7px;
    display: grid;
    place-items: center;
    color: var(--fg-2);
}
.nav-btn:hover:not(:disabled) {
    background: var(--surface-2);
    color: var(--fg);
}
.nav-btn:disabled {
    color: var(--fg-3);
    opacity: 0.6;
}
.crumbs {
    display: flex;
    align-items: center;
    gap: 6px;
    margin-left: 8px;
    font-size: 13px;
    min-width: 0;
}
.crumb {
    color: var(--fg-3);
    font-weight: 600;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
}
button.crumb:hover {
    color: var(--fg);
}
.crumb.current {
    color: var(--fg-2);
    max-width: 36ch;
}
.sep {
    color: var(--fg-3);
    flex: none;
}

.find {
    display: inline-flex;
    align-items: center;
    gap: 8px;
    height: 28px;
    padding: 0 6px 0 10px;
    border-radius: 9px;
    background: var(--surface);
    border: 1px solid var(--line);
    font-size: 12px;
    font-weight: 600;
    color: var(--fg-3);
    min-width: 190px;
}
.find:hover {
    border-color: var(--line-strong);
    color: var(--fg-2);
}
.find kbd {
    margin-left: auto;
    font-family: var(--mono);
    font-size: 10px;
    padding: 1px 6px;
    border-radius: 5px;
    border: 1px solid var(--line-strong);
}
.update {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    height: 28px;
    padding: 0 12px;
    border-radius: 999px;
    background: #ffffff;
    color: #000000;
    font-size: 12px;
    font-weight: 800;
}
.update:disabled {
    opacity: 0.7;
}
.status {
    display: inline-flex;
    align-items: center;
    gap: 7px;
    height: 28px;
    padding: 0 10px;
    border-radius: 999px;
    background: var(--surface);
    border: 1px solid var(--line);
    font-size: 12px;
    font-weight: 700;
    color: var(--fg-2);
}
.status:hover {
    border-color: var(--line-strong);
    color: var(--fg);
}
.status .dot {
    width: 7px;
    height: 7px;
    border-radius: 50%;
    background: var(--fg-3);
}
.status.live .dot {
    background: var(--good);
    box-shadow: 0 0 0 3px rgba(63, 224, 160, 0.18);
}

.acct {
    position: relative;
}
.acct-btn {
    display: inline-flex;
    align-items: center;
    gap: 8px;
    height: 30px;
    padding: 0 8px 0 4px;
    border-radius: 9px;
    background: var(--surface);
    border: 1px solid var(--line);
    font-size: 12px;
    font-weight: 700;
    max-width: 200px;
}
.acct-btn:hover {
    border-color: var(--line-strong);
}
.acct-name {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
}
.av {
    width: 22px;
    height: 22px;
    border-radius: 6px;
    background: var(--surface-3);
    flex: none;
    display: grid;
    place-items: center;
    color: var(--fg-3);
}
.menu {
    position: absolute;
    right: 0;
    top: calc(100% + 8px);
    width: 250px;
    padding: 6px;
    border-radius: 12px;
    background: var(--panel);
    border: 1px solid var(--line-strong);
    box-shadow: 0 18px 50px rgba(0, 0, 0, 0.55);
    display: flex;
    flex-direction: column;
    gap: 2px;
}
.menu-head {
    padding: 6px 8px 4px;
}
.menu-item {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 7px 8px;
    border-radius: 8px;
    text-align: left;
    font-size: 13px;
    font-weight: 600;
    color: var(--fg-2);
}
.menu-item:hover:not(:disabled) {
    background: var(--surface-3);
    color: var(--fg);
}
.menu-item .av {
    width: 26px;
    height: 26px;
}
.mi-text {
    display: flex;
    flex-direction: column;
    line-height: 1.25;
    min-width: 0;
    flex: 1;
}
.mi-text b {
    color: var(--fg);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
}
.mi-text span {
    font-size: 11px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
}
.tick {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: var(--accent);
}
.menu-sep {
    height: 1px;
    background: var(--line);
    margin: 4px 2px;
}

.win {
    display: flex;
    height: 100%;
    margin-left: 6px;
}
.win button {
    width: 46px;
    height: 100%;
    display: grid;
    place-items: center;
    color: var(--fg-2);
}
.win button:hover {
    background: var(--surface-2);
    color: var(--fg);
}
.win .close:hover {
    background: #e5384f;
    color: #fff;
}
</style>
