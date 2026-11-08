<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue'
import { api, type Stats } from '../api'
import Icon from '../components/Icon.vue'
import { Ago, Dur } from '../format'
import { QuickPlay, store } from '../store'

const DAY_NAMES = ['Sun', 'Mon', 'Tue', 'Wed', 'Thu', 'Fri', 'Sat']

const who = ref<number | 0>(0)
const stats = ref<Stats | null>(null)
const error = ref('')
const loading = ref(false)

const week = computed(() => {
    const w = stats.value?.week ?? []
    const max = Math.max(1, ...w)
    const today = new Date().getDay()
    return w.map((secs, i) => ({ secs, pct: (secs / max) * 100, day: i === 6 ? 'Today' : DAY_NAMES[(today - 6 + i + 7) % 7] }))
})
const topSecs = computed(() => stats.value?.games[0]?.secs ?? 1)
const weekTotal = computed(() => (stats.value?.week ?? []).reduce((a, b) => a + b, 0))
const Name = (id: number) => store.accounts.find((a) => a.userId === id)?.displayName ?? `Account ${id}`
const Hours = (secs: number) => (secs >= 3600 ? `${(secs / 3600).toFixed(secs >= 36000 ? 0 : 1)}h` : Dur(secs))

async function Load() {
    loading.value = true
    error.value = ''
    try {
        stats.value = await api.stats(who.value || null, -new Date().getTimezoneOffset())
    } catch (e) {
        error.value = String(e)
    }
    loading.value = false
}

watch(who, Load)
onMounted(Load)
</script>

<template>
    <div class="stack" style="gap: 22px">
        <header class="row between" style="align-items: flex-end; flex-wrap: wrap; gap: 12px">
            <div>
                <h1>Your stats</h1>
                <p class="muted">
                    Counted from Roblox's logs on this PC, including play outside Umbra<template v-if="stats?.first">
                        since {{ new Date(stats.first * 1000).toLocaleDateString() }}</template
                    >.
                </p>
            </div>
            <select id="stats-who" v-model.number="who" class="select" aria-label="Whose stats">
                <option :value="0">Everyone on this PC</option>
                <option v-for="a in store.accounts" :key="a.userId" :value="a.userId">{{ a.displayName }}</option>
            </select>
        </header>

        <p v-if="error" class="err">{{ error }}</p>
        <div v-else-if="!stats" class="stack" style="gap: 14px">
            <div class="skeleton" style="height: 96px; border-radius: 14px"></div>
            <div class="skeleton" style="height: 220px; border-radius: 14px"></div>
        </div>
        <section v-else-if="!stats.sessions" class="card empty">
            <h2>Nothing yet</h2>
            <p class="muted">Play a game and your stats show up here. Umbra reads Roblox's own logs, so no setup is needed.</p>
        </section>
        <template v-else>
            <section class="tiles" :class="{ dim: loading }">
                <div class="tile card">
                    <span class="faint small">Total playtime</span><b class="mono">{{ Hours(stats.totalSecs) }}</b>
                </div>
                <div class="tile card">
                    <span class="faint small">This week</span><b class="mono">{{ Hours(weekTotal) }}</b>
                </div>
                <div class="tile card">
                    <span class="faint small">Sessions</span><b class="mono">{{ stats.sessions.toLocaleString() }}</b>
                </div>
                <div class="tile card">
                    <span class="faint small">Days played</span><b class="mono">{{ stats.daysPlayed }}</b>
                </div>
                <div class="tile card">
                    <span class="faint small">Streak</span><b class="mono">{{ stats.streak }} day{{ stats.streak === 1 ? '' : 's' }}</b>
                </div>
                <div class="tile card">
                    <span class="faint small">Longest session</span><b class="mono">{{ Hours(stats.longestSecs) }}</b>
                </div>
            </section>

            <section class="card stack" style="gap: 14px">
                <h2>Last 7 days</h2>
                <div class="week" role="img" :aria-label="`Playtime for the last 7 days, ${Hours(weekTotal)} in total`">
                    <div v-for="d in week" :key="d.day" class="col">
                        <span class="faint small mono">{{ d.secs ? Hours(d.secs) : '' }}</span>
                        <div class="track"><div class="fill" :style="{ height: d.pct + '%' }"></div></div>
                        <span class="small" :class="{ faint: d.day !== 'Today' }">{{ d.day }}</span>
                    </div>
                </div>
            </section>

            <section class="card stack" style="gap: 6px">
                <h2 style="margin-bottom: 8px">Most played</h2>
                <div v-for="(g, i) in stats.games" :key="g.universeId" class="game">
                    <span class="rank mono faint">{{ i + 1 }}</span>
                    <img v-if="g.icon" :src="g.icon" alt="" class="gicon" /><span v-else class="gicon"></span>
                    <div class="gtext">
                        <b>{{ g.name ?? `Game ${g.universeId}` }}</b>
                        <div class="pbar"><div :style="{ width: (g.secs / topSecs) * 100 + '%' }"></div></div>
                        <span class="faint small">{{ g.sessions }} sessions · last played {{ Ago(g.last) }}</span>
                    </div>
                    <b class="mono hours">{{ Hours(g.secs) }}</b>
                    <button class="btn btn-sm" :aria-label="'Play ' + (g.name ?? 'this game')" @click="QuickPlay(g.rootPlaceId)">
                        <Icon name="play" :size="12" />Play
                    </button>
                </div>
            </section>

            <section v-if="!who && stats.accounts.length > 1" class="card stack" style="gap: 6px">
                <h2 style="margin-bottom: 8px">By account</h2>
                <div v-for="a in stats.accounts" :key="a.userId" class="acct-row">
                    <span>{{ Name(a.userId) }}</span>
                    <span class="faint small">{{ a.sessions }} sessions</span>
                    <b class="mono">{{ Hours(a.secs) }}</b>
                </div>
            </section>
        </template>
    </div>
</template>

<style scoped>
.tiles {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(160px, 1fr));
    gap: 12px;
    transition: opacity 0.15s;
}
.tiles.dim {
    opacity: 0.6;
}
.tile {
    display: flex;
    flex-direction: column;
    gap: 6px;
    padding: 16px;
}
.tile b {
    font-size: 24px;
}
.week {
    display: grid;
    grid-template-columns: repeat(7, 1fr);
    gap: 10px;
    height: 200px;
}
.col {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 6px;
}
.track {
    flex: 1;
    width: 100%;
    max-width: 56px;
    display: flex;
    align-items: flex-end;
    border-radius: 8px;
    background: var(--surface-2);
    overflow: hidden;
}
.fill {
    width: 100%;
    background: var(--accent);
    border-radius: 8px 8px 0 0;
    transition: height 0.4s var(--ease-out);
}
.game {
    display: grid;
    grid-template-columns: 24px 44px minmax(0, 1fr) 70px auto;
    align-items: center;
    gap: 12px;
    padding: 8px 0;
    border-top: 1px solid var(--line);
}
.game:first-of-type {
    border-top: 0;
}
.rank {
    text-align: right;
}
.gicon {
    width: 44px;
    height: 44px;
    border-radius: 10px;
    background: var(--surface-2);
}
.gtext {
    display: flex;
    flex-direction: column;
    gap: 4px;
    min-width: 0;
}
.gtext b {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
}
.pbar {
    height: 4px;
    border-radius: 2px;
    background: var(--surface-2);
    overflow: hidden;
}
.pbar div {
    height: 100%;
    background: var(--accent);
}
.hours {
    text-align: right;
}
.acct-row {
    display: grid;
    grid-template-columns: minmax(0, 1fr) auto 80px;
    gap: 12px;
    align-items: center;
    padding: 8px 0;
    border-top: 1px solid var(--line);
}
.acct-row:first-of-type {
    border-top: 0;
}
.acct-row b {
    text-align: right;
}
.empty {
    padding: 28px;
}
</style>
