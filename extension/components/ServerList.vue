<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { JoinUrl, Servers, type Server } from '@/utils/roblox'
import { Join, Note, umbra } from '@/utils/umbra-store'

const props = defineProps<{ placeId: number; maxPlayers: number }>()

type Sort = 'most' | 'fewest' | 'ping' | 'fps'
const servers = ref<Server[]>([])
const next = ref<string | null>(null)
const loading = ref(false)
const error = ref('')
const sort = ref<Sort>('most')
const minFree = ref(0)
const copied = ref('')

const shown = computed(() => {
    const list = servers.value.filter((s) => s.maxPlayers - s.playing >= minFree.value)
    const by: Record<Sort, (a: Server, b: Server) => number> = {
        most: (a, b) => b.playing - a.playing,
        fewest: (a, b) => a.playing - b.playing,
        ping: (a, b) => (a.ping ?? 9999) - (b.ping ?? 9999),
        fps: (a, b) => (b.fps ?? 0) - (a.fps ?? 0),
    }
    return [...list].sort(by[sort.value]).slice(0, 60)
})

async function Load(reset: boolean) {
    loading.value = true
    error.value = ''
    try {
        const page = await Servers(props.placeId, reset ? null : next.value)
        servers.value = reset ? page.servers : [...servers.value, ...page.servers]
        next.value = page.next
    } catch (e) {
        error.value = e instanceof Error ? e.message : String(e)
    }
    loading.value = false
}

function Pick(kind: 'smallest' | 'ping' | 'random') {
    const open = servers.value.filter((s) => s.playing < s.maxPlayers)
    if (!open.length) return Note('bad', 'No open servers loaded yet.')
    const s =
        kind === 'smallest'
            ? [...open].sort((a, b) => a.playing - b.playing)[0]
            : kind === 'ping'
              ? [...open].sort((a, b) => (a.ping ?? 9999) - (b.ping ?? 9999))[0]
              : open[Math.floor(Math.random() * open.length)]
    if (s) Join(props.placeId, s.id)
}

async function Copy(id: string) {
    try {
        await navigator.clipboard.writeText(JoinUrl(props.placeId, id))
        copied.value = id
        setTimeout(() => (copied.value = ''), 1400)
    } catch {
        Note('bad', "Couldn't copy the link.")
    }
}

const PingClass = (p: number | null) => (p == null ? '' : p < 80 ? 'good' : p < 150 ? 'warn' : 'bad')

onMounted(() => Load(true))
</script>

<template>
    <div class="card stack">
        <div class="row between">
            <span class="label">Servers</span>
            <button class="btn btn-ghost btn-sm" :disabled="loading" @click="Load(true)">{{ loading ? 'Loading…' : 'Refresh' }}</button>
        </div>
        <div class="row qs">
            <button class="btn btn-sm" :disabled="umbra.busy" @click="Pick('smallest')">Smallest</button>
            <button class="btn btn-sm" :disabled="umbra.busy" @click="Pick('ping')">Best ping</button>
            <button class="btn btn-sm" :disabled="umbra.busy" @click="Pick('random')">Random</button>
        </div>
        <div class="seg" role="group" aria-label="Sort servers">
            <button :aria-pressed="sort === 'most'" @click="sort = 'most'">Most</button>
            <button :aria-pressed="sort === 'fewest'" @click="sort = 'fewest'">Fewest</button>
            <button :aria-pressed="sort === 'ping'" @click="sort = 'ping'">Ping</button>
            <button :aria-pressed="sort === 'fps'" @click="sort = 'fps'">FPS</button>
        </div>
        <label class="row between small">
            <span class="faint">Free slots at least</span>
            <select v-model.number="minFree" class="select" aria-label="Minimum free slots">
                <option :value="0">Any</option>
                <option :value="1">1</option>
                <option :value="2">2</option>
                <option :value="4">4</option>
                <option :value="Math.max(1, Math.floor(maxPlayers / 2))">Half empty</option>
            </select>
        </label>
        <p v-if="error" class="note bad">{{ error }}</p>
        <p v-else-if="!loading && !shown.length" class="muted small">No open public servers match.</p>
        <ul class="servers">
            <li v-for="s in shown" :key="s.id" class="server rise">
                <div class="who">
                    <span class="small"
                        ><b class="mono">{{ s.playing }}</b
                        ><span class="faint"> / {{ s.maxPlayers }}</span></span
                    >
                    <div class="bar">
                        <div :style="{ width: Math.min(100, (s.playing / Math.max(1, s.maxPlayers)) * 100) + '%' }"></div>
                    </div>
                </div>
                <span class="mono small" :class="['pingv', PingClass(s.ping)]">{{ s.ping ?? '?' }}<span class="faint">ms</span></span>
                <span class="mono small">{{ s.fps ? Math.round(s.fps) : '?' }}<span class="faint">fps</span></span>
                <button class="btn btn-ghost btn-sm" :title="'Copy an invite link to ' + s.id" @click="Copy(s.id)">
                    {{ copied === s.id ? 'Copied' : 'Link' }}
                </button>
                <button class="btn btn-sm" :disabled="umbra.busy" @click="Join(placeId, s.id)">Join</button>
            </li>
        </ul>
        <button v-if="next" class="btn btn-wide" :disabled="loading" @click="Load(false)">{{ loading ? 'Loading…' : 'Load more' }}</button>
    </div>
</template>

<style scoped>
.qs .btn {
    flex: 1;
}
.servers {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 6px;
}
.server {
    display: grid;
    grid-template-columns: minmax(0, 1fr) 54px 50px auto auto;
    align-items: center;
    gap: 8px;
    padding: 8px 8px 8px 10px;
    border-radius: 10px;
    background: var(--raised);
}
.who {
    display: flex;
    flex-direction: column;
    gap: 4px;
}
.pingv.good {
    color: var(--good);
}
.pingv.warn {
    color: var(--warn);
}
.pingv.bad {
    color: var(--bad);
}
</style>
