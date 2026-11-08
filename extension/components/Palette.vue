<script setup lang="ts">
import { computed, nextTick, onMounted, ref, watch } from 'vue'
import { Compact } from '@/utils/format'
import { SearchGames, SearchUsers } from '@/utils/roblox'

const emit = defineEmits<{ close: [] }>()

interface Hit {
    group: string
    label: string
    hint: string
    url: string
}

const LINKS: Hit[] = [
    { group: 'Go to', label: 'Home', hint: 'roblox.com/home', url: 'https://www.roblox.com/home' },
    { group: 'Go to', label: 'Charts', hint: 'Top and trending games', url: 'https://www.roblox.com/charts' },
    { group: 'Go to', label: 'Friends', hint: 'Your friends list', url: 'https://www.roblox.com/users/friends' },
    { group: 'Go to', label: 'Avatar', hint: 'Avatar editor', url: 'https://www.roblox.com/my/avatar' },
    { group: 'Go to', label: 'Marketplace', hint: 'Avatar shop', url: 'https://www.roblox.com/catalog' },
    { group: 'Go to', label: 'Trades', hint: 'Your trades', url: 'https://www.roblox.com/trades' },
    { group: 'Go to', label: 'Inventory', hint: 'Your items', url: 'https://www.roblox.com/users/inventory' },
    { group: 'Go to', label: 'Settings', hint: 'Account settings', url: 'https://www.roblox.com/my/account' },
]

const q = ref('')
const games = ref<Hit[]>([])
const users = ref<Hit[]>([])
const sel = ref(0)
const input = ref<HTMLInputElement | null>(null)
let timer = 0
let seq = 0

const hits = computed(() => {
    const term = q.value.trim().toLowerCase()
    const links = LINKS.filter((l) => !term || l.label.toLowerCase().includes(term))
    return [...games.value, ...users.value, ...links]
})
const groups = computed(() => {
    const out: { name: string; items: { hit: Hit; i: number }[] }[] = []
    hits.value.forEach((hit, i) => {
        let g = out.find((x) => x.name === hit.group)
        if (!g) out.push((g = { name: hit.group, items: [] }))
        g.items.push({ hit, i })
    })
    return out
})

watch(q, (v) => {
    sel.value = 0
    clearTimeout(timer)
    const term = v.trim()
    if (term.length < 2) {
        games.value = []
        users.value = []
        return
    }
    timer = window.setTimeout(async () => {
        const my = ++seq
        const [g, u] = await Promise.all([SearchGames(term).catch(() => []), SearchUsers(term).catch(() => [])])
        if (my !== seq) return
        games.value = g.map((x) => ({
            group: 'Games',
            label: x.name,
            hint: `${Compact(x.playing)} playing`,
            url: `https://www.roblox.com/games/${x.rootPlaceId}`,
        }))
        users.value = u.map((x) => ({
            group: 'Players',
            label: x.displayName,
            hint: `@${x.name}`,
            url: `https://www.roblox.com/users/${x.id}/profile`,
        }))
    }, 220)
})

function Open(h: Hit | undefined) {
    if (!h) return
    location.href = h.url
    emit('close')
}

function OnKey(e: KeyboardEvent) {
    e.stopPropagation()
    if (e.key === 'Escape') emit('close')
    else if (e.key === 'ArrowDown') {
        e.preventDefault()
        sel.value = Math.min(hits.value.length - 1, sel.value + 1)
    } else if (e.key === 'ArrowUp') {
        e.preventDefault()
        sel.value = Math.max(0, sel.value - 1)
    } else if (e.key === 'Enter') Open(hits.value[sel.value])
}

onMounted(() => nextTick(() => input.value?.focus()))
</script>

<template>
    <div class="scrim" @mousedown.self="emit('close')">
        <div class="palette" role="dialog" aria-modal="true" aria-label="Search Roblox">
            <input
                ref="input"
                v-model="q"
                class="input big"
                placeholder="Search games, players and pages"
                aria-label="Search"
                @keydown="OnKey"
            />
            <div class="results" role="listbox">
                <template v-for="g in groups" :key="g.name">
                    <p class="group">{{ g.name }}</p>
                    <button
                        v-for="{ hit, i } in g.items"
                        :key="hit.url"
                        role="option"
                        :aria-selected="sel === i"
                        class="hit"
                        @mouseenter="sel = i"
                        @click="Open(hit)"
                    >
                        <span class="hl">{{ hit.label }}</span
                        ><span class="faint small">{{ hit.hint }}</span>
                    </button>
                </template>
            </div>
            <p class="foot faint small">Enter to open · Esc to close · Ctrl K anywhere on Roblox</p>
        </div>
    </div>
</template>

<style scoped>
.scrim {
    position: fixed;
    inset: 0;
    z-index: 2147483646;
    display: flex;
    justify-content: center;
    align-items: flex-start;
    padding: 12vh 16px 16px;
    background: rgba(0, 0, 0, 0.6);
    animation: fade 0.15s ease both;
}
@keyframes fade {
    from {
        opacity: 0;
    }
}
.palette {
    width: min(600px, 100%);
    max-height: 70vh;
    display: flex;
    flex-direction: column;
    gap: 6px;
    padding: 10px;
    border-radius: 16px;
    border: 1px solid var(--line-2);
    background: var(--panel);
    box-shadow: 0 30px 80px rgba(0, 0, 0, 0.6);
    animation: pop 0.2s var(--ease) both;
}
.big {
    height: 44px;
    font-size: 15px;
}
.results {
    overflow-y: auto;
    display: flex;
    flex-direction: column;
    gap: 2px;
}
.group {
    padding: 8px 8px 2px;
    font-size: 11px;
    font-weight: 600;
    color: var(--fg-3);
}
.hit {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    height: 36px;
    padding: 0 10px;
    border-radius: 9px;
    text-align: left;
}
.hit[aria-selected='true'] {
    background: #ffffff;
    color: #000000;
}
.hit[aria-selected='true'] .faint {
    color: #555;
}
.hl {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    font-weight: 600;
}
.foot {
    padding: 6px 8px 2px;
}
</style>
