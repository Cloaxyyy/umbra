<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref, watch } from 'vue'
import Clients from '@/components/Clients.vue'
import GameSection from '@/components/GameSection.vue'
import ItemSection from '@/components/ItemSection.vue'
import Palette from '@/components/Palette.vue'
import PlayAs from '@/components/PlayAs.vue'
import UserSection from '@/components/UserSection.vue'
import { OpenOptions } from '@/utils/messages'
import { currentUrl } from '@/utils/page'
import { PageKind } from '@/utils/roblox'
import { DEFAULT_FEATURES, features, LoadFeatures, type Features } from '@/utils/settings'
import { RefreshUmbra, umbra } from '@/utils/umbra-store'

const f = ref<Features>(DEFAULT_FEATURES)
const open = ref(false)
const palette = ref(false)
const page = computed(() => PageKind(currentUrl.value))
const key = computed(() => JSON.stringify(page.value))
const heading = computed(() => ({ game: 'This game', user: 'This player', item: 'This item', other: 'Umbra' })[page.value.kind] ?? 'Umbra')

let poll = 0
function SetOpen(v: boolean) {
    open.value = v
    clearInterval(poll)
    if (v) {
        RefreshUmbra()
        poll = window.setInterval(RefreshUmbra, 10_000)
    }
}

function OnKey(e: KeyboardEvent) {
    if ((e.ctrlKey || e.metaKey) && !e.shiftKey && !e.altKey && e.key.toLowerCase() === 'k' && f.value.search) {
        e.preventDefault()
        palette.value = !palette.value
    } else if (e.altKey && !e.ctrlKey && e.key.toLowerCase() === 'u' && f.value.drawer) {
        e.preventDefault()
        SetOpen(!open.value)
    } else if (e.key === 'Escape' && open.value && !palette.value) SetOpen(false)
}

watch(
    () => currentUrl.value,
    () => {
        if (f.value.autoOpen && page.value.kind === 'game') SetOpen(true)
    },
)

onMounted(async () => {
    f.value = await LoadFeatures()
    features.watch(async () => (f.value = await LoadFeatures()))
    window.addEventListener('keydown', OnKey, true)
    if (f.value.autoOpen && page.value.kind === 'game') SetOpen(true)
    else RefreshUmbra().catch(() => {})
})
onUnmounted(() => {
    window.removeEventListener('keydown', OnKey, true)
    clearInterval(poll)
})
</script>

<template>
    <div class="u">
        <button v-if="f.drawer && !open" class="handle" aria-label="Open Umbra" title="Umbra (Alt U)" @click="SetOpen(true)">
            <svg viewBox="0 0 32 32" width="22" height="22" aria-hidden="true">
                <defs>
                    <linearGradient id="umbra-ring" x1="0" y1="0" x2="1" y2="1">
                        <stop offset="0" stop-color="#b7a6ff" />
                        <stop offset="1" stop-color="#6aa8ff" />
                    </linearGradient>
                </defs>
                <circle cx="16" cy="16" r="12" fill="none" stroke="url(#umbra-ring)" stroke-width="2.4" />
                <circle cx="16" cy="16" r="10.6" fill="#000" />
            </svg>
            <span v-if="umbra.state?.running.length" class="count">{{ umbra.state.running.length }}</span>
        </button>

        <aside v-if="f.drawer && open" class="drawer" aria-label="Umbra">
            <header class="dh">
                <div class="row">
                    <svg viewBox="0 0 32 32" width="20" height="20" aria-hidden="true">
                        <circle cx="16" cy="16" r="12" fill="none" stroke="#b7a6ff" stroke-width="2.4" />
                        <circle cx="16" cy="16" r="10.6" fill="#000" />
                    </svg>
                    <b>{{ heading }}</b>
                </div>
                <button class="btn btn-ghost btn-sm" aria-label="Close Umbra" title="Close (Esc)" @click="SetOpen(false)">Close</button>
            </header>
            <div class="body">
                <Transition name="note">
                    <p v-if="umbra.note" :key="umbra.note.text" class="note" :class="umbra.note.kind" role="status">
                        {{ umbra.note.text }}
                    </p>
                </Transition>
                <GameSection v-if="page.kind === 'game'" :key="key" :place-id="page.placeId" :servers="f.servers" />
                <UserSection v-else-if="page.kind === 'user'" :key="key" :user-id="page.userId" :values="f.values" />
                <ItemSection v-else-if="page.kind === 'item'" :key="key" :id="page.id" :item-type="page.itemType" :values="f.values" />
                <PlayAs v-else />
                <Clients v-if="umbra.state" />
                <div class="card row between">
                    <span class="muted small">Themes, features and the Umbra connection</span>
                    <button class="btn btn-sm" @click="OpenOptions">Open</button>
                </div>
                <p class="faint small center">Ctrl K to search · Alt U to toggle</p>
            </div>
        </aside>

        <Palette v-if="palette" @close="palette = false" />
    </div>
</template>

<style scoped>
.handle {
    position: fixed;
    right: 0;
    top: 50%;
    z-index: 2147483640;
    display: grid;
    place-items: center;
    width: 40px;
    height: 48px;
    transform: translateY(-50%);
    border-radius: 12px 0 0 12px;
    border: 1px solid rgba(255, 255, 255, 0.14) !important;
    border-right: 0 !important;
    background: #000 !important;
    box-shadow: 0 10px 30px rgba(0, 0, 0, 0.4);
    transition: width 0.2s var(--ease);
}
.handle:hover {
    width: 46px;
}
.count {
    position: absolute;
    top: 4px;
    left: 4px;
    min-width: 16px;
    height: 16px;
    padding: 0 4px;
    border-radius: 8px;
    background: #fff;
    color: #000;
    font-size: 10px;
    font-weight: 700;
    line-height: 16px;
}
.drawer {
    position: fixed;
    top: 0;
    right: 0;
    bottom: 0;
    z-index: 2147483641;
    width: min(400px, 100vw);
    display: flex;
    flex-direction: column;
    background: var(--bg);
    border-left: 1px solid var(--line-2);
    box-shadow: -30px 0 80px rgba(0, 0, 0, 0.5);
    animation: slide 0.26s var(--ease) both;
}
.dh {
    display: flex;
    align-items: center;
    justify-content: space-between;
    height: 52px;
    padding: 0 12px 0 16px;
    border-bottom: 1px solid var(--line);
}
.dh b {
    font-size: 14px;
}
.body {
    flex: 1;
    overflow-y: auto;
    display: flex;
    flex-direction: column;
    gap: 10px;
    padding: 12px;
    overscroll-behavior: contain;
}
.center {
    text-align: center;
}
.note-enter-active,
.note-leave-active {
    transition:
        opacity 0.2s,
        transform 0.2s;
}
.note-enter-from,
.note-leave-to {
    opacity: 0;
    transform: translateY(-6px);
}
</style>
