<script setup lang="ts">
import { computed, nextTick, onMounted, ref, watch } from 'vue'
import { api } from '../api'
import type { IconName } from '../icons'
import { Go, store, type Route } from '../store'
import Icon from './Icon.vue'

interface Hit {
    key: string
    group: string
    label: string
    sub: string
    icon: IconName
    image: string | null
    to: Route
}

const PAGES: { label: string; icon: IconName; to: Route; words: string }[] = [
    { label: 'Launchpad', icon: 'rocket', to: { view: 'home' }, words: 'home play start' },
    { label: 'Discover', icon: 'gamepad', to: { view: 'discover' }, words: 'games charts trending' },
    { label: 'Friends', icon: 'users', to: { view: 'friends' }, words: 'players people profile' },
    { label: 'Catalog', icon: 'bag', to: { view: 'catalog' }, words: 'shop items avatar shop' },
    { label: 'Avatar', icon: 'shirt', to: { view: 'avatar' }, words: 'outfit wear skin' },
    { label: 'Trades', icon: 'swap', to: { view: 'trades' }, words: 'limiteds values rolimons' },
    { label: 'Accounts', icon: 'idcard', to: { view: 'accounts' }, words: 'alts login' },
    { label: 'Instances', icon: 'layers', to: { view: 'instances' }, words: 'mods flags fps profiles' },
    { label: 'Stats', icon: 'activity', to: { view: 'stats' }, words: 'playtime hours most played streak' },
    { label: 'Settings', icon: 'settings', to: { view: 'settings' }, words: 'updates discord tray theme' },
]

const input = ref<HTMLInputElement | null>(null)
const query = ref('')
const games = ref<Hit[]>([])
const players = ref<Hit[]>([])
const items = ref<Hit[]>([])
const busy = ref(false)
const index = ref(0)

const pages = computed<Hit[]>(() => {
    const q = query.value.trim().toLowerCase()
    return PAGES.filter((p) => !q || p.label.toLowerCase().includes(q) || p.words.includes(q)).map((p) => ({
        key: 'page-' + p.label,
        group: 'Pages',
        label: p.label,
        sub: '',
        icon: p.icon,
        image: null,
        to: p.to,
    }))
})
const hits = computed(() => [...pages.value, ...games.value, ...players.value, ...items.value])
const groups = computed(() => {
    const out: { name: string; items: { hit: Hit; i: number }[] }[] = []
    hits.value.forEach((hit, i) => {
        let g = out.find((x) => x.name === hit.group)
        if (!g) out.push((g = { name: hit.group, items: [] }))
        g.items.push({ hit, i })
    })
    return out
})

let timer = 0
let seq = 0
watch(query, () => {
    index.value = 0
    clearTimeout(timer)
    const q = query.value.trim()
    if (q.length < 2) {
        games.value = []
        players.value = []
        items.value = []
        return
    }
    timer = window.setTimeout(() => Search(q), 250)
})

async function Search(q: string) {
    const mine = ++seq
    busy.value = true
    const [g, p, c] = await Promise.allSettled([
        api.search(q),
        q.length >= 3 ? api.searchPlayers(q) : Promise.resolve([]),
        api.searchCatalog(q, 1, null, 0),
    ])
    if (mine !== seq) return
    games.value =
        g.status === 'fulfilled'
            ? g.value.slice(0, 5).map((x) => ({
                  key: 'game-' + x.universeId,
                  group: 'Games',
                  label: x.name,
                  sub: `${x.playerCount.toLocaleString()} playing`,
                  icon: 'gamepad' as IconName,
                  image: x.icon,
                  to: { view: 'game' as const, universeId: x.universeId, name: x.name },
              }))
            : []
    players.value =
        p.status === 'fulfilled'
            ? p.value.slice(0, 5).map((x) => ({
                  key: 'player-' + x.userId,
                  group: 'Players',
                  label: x.displayName,
                  sub: '@' + x.name,
                  icon: 'user' as IconName,
                  image: x.avatar,
                  to: { view: 'friends' as const, userId: x.userId },
              }))
            : []
    items.value =
        c.status === 'fulfilled'
            ? c.value.items.slice(0, 5).map((x) => ({
                  key: 'item-' + x.itemType + x.id,
                  group: 'Catalog',
                  label: x.name,
                  sub: x.creator,
                  icon: 'bag' as IconName,
                  image: x.thumb,
                  to: { view: 'catalog' as const, query: x.name },
              }))
            : []
    busy.value = false
}

function Close() {
    store.palette = false
}
function Pick(hit: Hit | undefined) {
    if (!hit) return
    Close()
    Go(hit.to)
}
function OnKey(e: KeyboardEvent) {
    if (e.key === 'Escape') Close()
    else if (e.key === 'ArrowDown') {
        e.preventDefault()
        index.value = Math.min(hits.value.length - 1, index.value + 1)
        Reveal()
    } else if (e.key === 'ArrowUp') {
        e.preventDefault()
        index.value = Math.max(0, index.value - 1)
        Reveal()
    } else if (e.key === 'Enter') Pick(hits.value[index.value])
}
function Reveal() {
    nextTick(() => document.querySelector('.palette .hit.on')?.scrollIntoView({ block: 'nearest' }))
}

onMounted(() => input.value?.focus())
</script>

<template>
    <div class="scrim" @mousedown.self="Close">
        <div class="palette" role="dialog" aria-modal="true" aria-label="Search Umbra">
            <label class="bar">
                <Icon name="search" :size="17" />
                <input
                    id="palette-input"
                    ref="input"
                    v-model="query"
                    placeholder="Search games, players, items and pages"
                    autocomplete="off"
                    spellcheck="false"
                    role="combobox"
                    aria-controls="palette-results"
                    :aria-expanded="hits.length > 0"
                    @keydown="OnKey"
                />
                <span v-if="busy" class="faint small">Searching…</span>
                <kbd>Esc</kbd>
            </label>
            <div id="palette-results" class="results" role="listbox">
                <p v-if="!hits.length" class="faint small none">
                    {{ query.trim().length < 2 ? 'Type at least 2 letters.' : busy ? 'Searching…' : 'Nothing matched.' }}
                </p>
                <template v-for="g in groups" :key="g.name">
                    <p class="group">{{ g.name }}</p>
                    <button
                        v-for="{ hit, i } in g.items"
                        :key="hit.key"
                        class="hit"
                        :class="{ on: i === index }"
                        role="option"
                        :aria-selected="i === index"
                        @mousemove="index = i"
                        @click="Pick(hit)"
                    >
                        <img v-if="hit.image" :src="hit.image" alt="" class="pic" />
                        <span v-else class="pic ico"><Icon :name="hit.icon" :size="16" /></span>
                        <span class="txt">
                            <b>{{ hit.label }}</b>
                            <span v-if="hit.sub" class="faint">{{ hit.sub }}</span>
                        </span>
                        <Icon v-if="i === index" name="enter" :size="14" class="go" />
                    </button>
                </template>
            </div>
            <div class="foot faint">
                <span><kbd>↑</kbd><kbd>↓</kbd> to move</span><span><kbd>Enter</kbd> to open</span
                ><span><kbd>Ctrl</kbd><kbd>K</kbd> any time</span>
            </div>
        </div>
    </div>
</template>

<style scoped>
.scrim {
    position: fixed;
    inset: 0;
    z-index: 60;
    background: rgba(0, 0, 0, 0.6);
    animation: fade-in 0.15s ease both;
    display: flex;
    justify-content: center;
    align-items: flex-start;
    padding: 12vh 20px 20px;
}
.palette {
    width: min(620px, 100%);
    max-height: 70vh;
    display: flex;
    flex-direction: column;
    border-radius: 16px;
    background: var(--panel);
    border: 1px solid var(--line-strong);
    box-shadow: 0 30px 80px rgba(0, 0, 0, 0.6);
    overflow: hidden;
    animation: pop 0.22s var(--ease-out) both;
}
.bar {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 0 16px;
    height: 54px;
    border-bottom: 1px solid var(--line);
    color: var(--fg-3);
}
.bar input {
    flex: 1;
    min-width: 0;
    background: transparent;
    border: 0;
    outline: none;
    font: inherit;
    font-size: 15px;
    font-weight: 600;
    color: var(--fg);
    user-select: text;
}
kbd {
    font-family: var(--mono);
    font-size: 11px;
    padding: 1px 6px;
    border-radius: 5px;
    border: 1px solid var(--line-strong);
    color: var(--fg-2);
    margin-right: 3px;
}
.small {
    font-size: 12px;
}
.results {
    overflow-y: auto;
    padding: 6px;
}
.none {
    padding: 18px 12px;
}
.group {
    font-size: 12px;
    font-weight: 700;
    color: var(--fg-3);
    padding: 10px 10px 4px;
}
.hit {
    width: 100%;
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 7px 10px;
    border-radius: 10px;
    text-align: left;
}
.hit.on {
    background: var(--select);
    color: var(--select-fg);
}
.hit.on .faint {
    color: rgba(0, 0, 0, 0.6);
}
.pic {
    width: 34px;
    height: 34px;
    border-radius: 9px;
    object-fit: cover;
    background: var(--surface-3);
    flex: none;
}
.ico {
    display: grid;
    place-items: center;
    color: var(--fg-2);
}
.hit.on .ico {
    background: rgba(0, 0, 0, 0.08);
    color: var(--select-fg);
}
.txt {
    display: flex;
    flex-direction: column;
    min-width: 0;
    flex: 1;
    line-height: 1.3;
    font-size: 12px;
}
.txt b {
    font-size: 13px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
}
.txt span {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
}
.go {
    flex: none;
}
.foot {
    display: flex;
    gap: 16px;
    padding: 10px 16px;
    border-top: 1px solid var(--line);
    font-size: 12px;
}
</style>
