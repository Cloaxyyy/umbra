<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { api, type GameCard } from '../api'
import GameTile from '../components/GameTile.vue'
import Icon from '../components/Icon.vue'
import { LoadCharts, store, Toast } from '../store'

const chartId = ref('top-trending')
const query = ref('')
const results = ref<GameCard[] | null>(null)
const searching = ref(false)
const loadError = ref('')

const chart = computed(() => store.charts?.find((c) => c.id === chartId.value) ?? store.charts?.[0])
const games = computed(() => results.value ?? chart.value?.games ?? [])
const heading = computed(() => (results.value ? `Results for “${query.value.trim()}”` : (chart.value?.name ?? 'Games')))

let debounce = 0
function OnType() {
    clearTimeout(debounce)
    if (!query.value.trim()) {
        results.value = null
        return
    }
    debounce = window.setTimeout(RunSearch, 450)
}
async function RunSearch() {
    clearTimeout(debounce)
    const q = query.value.trim()
    if (!q) {
        results.value = null
        return
    }
    searching.value = true
    try {
        results.value = await api.search(q)
    } catch (e) {
        Toast('bad', String(e))
    }
    searching.value = false
}
function Clear() {
    query.value = ''
    results.value = null
}

async function Load(force = false) {
    loadError.value = ''
    try {
        await LoadCharts(force)
    } catch (e) {
        loadError.value = String(e)
    }
}
onMounted(() => Load())
</script>

<template>
    <div class="stack" style="gap: 18px">
        <div class="toolbar">
            <select
                id="chart"
                v-model="chartId"
                class="select chart-pick"
                aria-label="Chart"
                :disabled="!!results"
                @change="results = null"
            >
                <option v-for="c in store.charts ?? []" :key="c.id" :value="c.id">{{ c.name }}</option>
            </select>
            <span class="divider" aria-hidden="true"></span>
            <label class="search">
                <Icon name="search" :size="16" />
                <input
                    id="game-search"
                    v-model="query"
                    class="search-input"
                    placeholder="Search games…"
                    autocomplete="off"
                    @input="OnType"
                    @keydown.enter="RunSearch"
                    @keydown.esc="Clear"
                />
                <button v-if="query" class="clear" aria-label="Clear search" @click="Clear"><Icon name="close" :size="14" /></button>
            </label>
            <button class="btn btn-ghost btn-sm" title="Reload charts" aria-label="Reload charts" @click="Load(true)">
                <Icon name="refresh" :size="14" />
            </button>
        </div>

        <div class="sec-head">
            <h2>{{ heading }}</h2>
            <span v-if="searching" class="faint" style="font-size: 12px">Searching…</span>
            <span v-else class="faint" style="font-size: 12px">{{ games.length }} games</span>
        </div>

        <p v-if="loadError" class="err">{{ loadError }}</p>
        <p v-else-if="results && !results.length" class="muted">No games matched. Try a shorter name.</p>

        <div class="tiles">
            <template v-if="store.charts || results">
                <GameTile v-for="g in games" :key="g.universeId" :game="g" />
            </template>
            <template v-else>
                <div v-for="i in 18" :key="i" class="ph">
                    <div class="skeleton ph-img"></div>
                    <div class="skeleton ph-line"></div>
                </div>
            </template>
        </div>
    </div>
</template>

<style scoped>
.toolbar {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 6px;
    border-radius: 14px;
    background: var(--surface);
    border: 1px solid var(--line);
}
.chart-pick {
    border-color: transparent;
    background-color: transparent;
    height: 38px;
    min-width: 180px;
}
.chart-pick:hover:not(:disabled) {
    background-color: var(--surface-2);
}
.chart-pick:disabled {
    opacity: 0.45;
}
.divider {
    width: 1px;
    height: 22px;
    background: var(--line-strong);
}
.search {
    flex: 1;
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 0 10px;
    color: var(--fg-3);
    min-width: 0;
}
.search-input {
    flex: 1;
    height: 38px;
    background: transparent;
    border: 0;
    outline: none;
    font: inherit;
    font-size: 14px;
    font-weight: 600;
    color: var(--fg);
    user-select: text;
    min-width: 0;
}
.search-input::placeholder {
    color: var(--fg-3);
}
.clear {
    width: 24px;
    height: 24px;
    border-radius: 6px;
    display: grid;
    place-items: center;
    color: var(--fg-3);
}
.clear:hover {
    background: var(--surface-2);
    color: var(--fg);
}
.tiles {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(168px, 1fr));
    gap: 20px 16px;
}
.ph {
    display: flex;
    flex-direction: column;
    gap: 10px;
}
.ph-img {
    aspect-ratio: 1;
    border-radius: 14px;
}
.ph-line {
    height: 12px;
    width: 70%;
    border-radius: 6px;
}
.err {
    color: var(--bad);
    user-select: text;
}
</style>
