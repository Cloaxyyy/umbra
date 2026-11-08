<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue'
import { api, type Spending, type Stats } from '../api'
import Icon from '../components/Icon.vue'
import { Ago, Dur } from '../format'
import { ActiveAccount, QuickPlay, store } from '../store'

const DAY_NAMES = ['Sun', 'Mon', 'Tue', 'Wed', 'Thu', 'Fri', 'Sat']
const MONTH_NAMES = ['Jan', 'Feb', 'Mar', 'Apr', 'May', 'Jun', 'Jul', 'Aug', 'Sep', 'Oct', 'Nov', 'Dec']
const TZ = () => -new Date().getTimezoneOffset()

const tab = ref<'play' | 'robux'>('play')
const spend = ref<Spending | null>(null)
const spendError = ref('')
const spendFor = computed(() => who.value || ActiveAccount()?.userId || 0)
const monthsMax = computed(() => Math.max(1, ...(spend.value?.months.map((m) => m[1]) ?? [])))
const kindsMax = computed(() => spend.value?.kinds[0]?.robux || 1)
const Rbx = (n: number) => `${n.toLocaleString()} R$`
const MonthLabel = (key: string) => MONTH_NAMES[Number(key.slice(5)) - 1] ?? key
const When = (at: number) => new Date(at * 1000).toLocaleDateString()

async function LoadSpend() {
    spend.value = null
    spendError.value = ''
    if (!spendFor.value) {
        spendError.value = 'Add a Roblox account to see what you spent.'
        return
    }
    try {
        spend.value = await api.spending(spendFor.value, TZ())
    } catch (e) {
        spendError.value = String(e)
    }
}

const who = ref<number>(0)
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
watch([tab, spendFor], () => tab.value === 'robux' && LoadSpend())
onMounted(Load)
</script>

<template>
    <div class="stack" style="gap: 22px">
        <header class="row between" style="align-items: flex-end; flex-wrap: wrap; gap: 12px">
            <div>
                <h1>Your stats</h1>
                <p v-if="tab === 'robux'" class="muted">What you spent Robux on, from your account's purchase history on Roblox.</p>
                <p v-else class="muted">
                    Counted from Roblox's logs on this PC, including play outside Umbra<template v-if="stats?.first">
                        since {{ new Date(stats.first * 1000).toLocaleDateString() }}</template
                    >.
                </p>
            </div>
            <div class="row" style="gap: 8px">
                <div class="seg" role="tablist" aria-label="Kind of stats">
                    <button role="tab" :aria-selected="tab === 'play'" @click="tab = 'play'">Playtime</button>
                    <button role="tab" :aria-selected="tab === 'robux'" @click="tab = 'robux'">Robux</button>
                </div>
                <select id="stats-who" v-model.number="who" class="select" aria-label="Whose stats">
                    <option :value="0">{{ tab === 'robux' ? 'Active account' : 'Everyone on this PC' }}</option>
                    <option v-for="a in store.accounts" :key="a.userId" :value="a.userId">{{ a.displayName }}</option>
                </select>
            </div>
        </header>

        <template v-if="tab === 'robux'">
            <p v-if="spendError" class="err">{{ spendError }}</p>
            <div v-else-if="!spend" class="stack" style="gap: 14px">
                <div class="skeleton" style="height: 96px; border-radius: 14px"></div>
                <div class="skeleton" style="height: 220px; border-radius: 14px"></div>
            </div>
            <section v-else-if="!spend.count" class="card empty">
                <h2>No purchases</h2>
                <p class="muted">{{ Name(spendFor) }} hasn't bought anything with Robux, or Roblox didn't return any purchases.</p>
            </section>
            <template v-else>
                <p class="muted">
                    {{ Name(spendFor) }}'s last {{ spend.count.toLocaleString() }} purchases<template v-if="spend.since">
                        since {{ When(spend.since) }}</template
                    >.<template v-if="spend.partial"> Older purchases aren't included.</template>
                </p>
                <section class="tiles">
                    <div class="tile card">
                        <span class="faint small">This month</span><b class="mono">{{ Rbx(spend.thisMonth) }}</b>
                    </div>
                    <div class="tile card">
                        <span class="faint small">In total</span><b class="mono">{{ Rbx(spend.total) }}</b>
                    </div>
                    <div class="tile card">
                        <span class="faint small">Purchases</span><b class="mono">{{ spend.count.toLocaleString() }}</b>
                    </div>
                    <div v-if="spend.biggest[0]" class="tile card">
                        <span class="faint small">Biggest purchase</span><b class="mono">{{ Rbx(spend.biggest[0].robux) }}</b>
                    </div>
                </section>

                <section class="card stack" style="gap: 14px">
                    <h2>Last 12 months</h2>
                    <div class="week months" role="img" :aria-label="`Robux spent in each of the last 12 months`">
                        <div v-for="[key, robux] in spend.months" :key="key" class="col">
                            <span class="faint small mono">{{ robux ? robux.toLocaleString() : '' }}</span>
                            <div class="track">
                                <div class="fill" :style="{ height: robux ? Math.max(4, (robux / monthsMax) * 100) + '%' : '0' }"></div>
                            </div>
                            <span class="small faint">{{ MonthLabel(key) }}</span>
                        </div>
                    </div>
                </section>

                <div class="split">
                    <section class="card stack" style="gap: 6px">
                        <h2 style="margin-bottom: 8px">Where it went</h2>
                        <div v-for="k in spend.kinds" :key="k.key" class="kind">
                            <div class="row between">
                                <span>{{ k.key }}</span
                                ><b class="mono">{{ Rbx(k.robux) }}</b>
                            </div>
                            <div class="pbar"><div :style="{ width: (k.robux / kindsMax) * 100 + '%' }"></div></div>
                            <span class="faint small">{{ k.count }} purchase{{ k.count === 1 ? '' : 's' }}</span>
                        </div>
                    </section>
                    <section class="card stack" style="gap: 6px">
                        <h2 style="margin-bottom: 8px">Top games</h2>
                        <p v-if="!spend.games.length" class="muted small">No game passes or in-game purchases yet.</p>
                        <div v-for="g in spend.games" :key="g.key" class="acct-row">
                            <span class="row" style="gap: 10px; min-width: 0">
                                <img v-if="g.icon" :src="g.icon" alt="" class="gicon small-icon" /><span
                                    v-else
                                    class="gicon small-icon"
                                ></span>
                                <span class="ellipsis">{{ g.key }}</span>
                            </span>
                            <span class="faint small">{{ g.count }}x</span>
                            <b class="mono">{{ Rbx(g.robux) }}</b>
                        </div>
                    </section>
                </div>

                <div class="split">
                    <section class="card stack" style="gap: 6px">
                        <h2 style="margin-bottom: 8px">Biggest purchases</h2>
                        <div v-for="(b, i) in spend.biggest" :key="'b' + i + b.at" class="acct-row">
                            <span class="stack" style="gap: 0; min-width: 0">
                                <span class="ellipsis">{{ b.name }}</span>
                                <span class="faint small ellipsis">{{ b.game ?? b.kind }} · {{ When(b.at) }}</span>
                            </span>
                            <span></span>
                            <b class="mono">{{ Rbx(b.robux) }}</b>
                        </div>
                    </section>
                    <section class="card stack" style="gap: 6px">
                        <h2 style="margin-bottom: 8px">Recent purchases</h2>
                        <div v-for="(r, i) in spend.recent.slice(0, 8)" :key="'r' + i + r.at" class="acct-row">
                            <span class="stack" style="gap: 0; min-width: 0">
                                <span class="ellipsis">{{ r.name }}</span>
                                <span class="faint small ellipsis">{{ r.game ?? r.kind }} · {{ Ago(r.at) }}</span>
                            </span>
                            <span></span>
                            <b class="mono">{{ Rbx(r.robux) }}</b>
                        </div>
                    </section>
                </div>
            </template>
        </template>

        <template v-else>
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
                            <div class="track"><div class="fill" :style="{ height: d.secs ? Math.max(4, d.pct) + '%' : '0' }"></div></div>
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
        </template>
    </div>
</template>

<style scoped>
.seg {
    display: flex;
    gap: 2px;
    padding: 3px;
    border-radius: 10px;
    background: var(--surface-2);
}
.seg button {
    height: 30px;
    padding: 0 14px;
    border-radius: 8px;
    font-size: 13px;
    font-weight: 700;
    color: var(--fg-2);
}
.seg button[aria-selected='true'] {
    background: var(--select);
    color: var(--select-fg);
}
.week.months {
    grid-template-columns: repeat(12, 1fr);
}
.split {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(320px, 1fr));
    gap: 14px;
    align-items: start;
}
.kind {
    display: flex;
    flex-direction: column;
    gap: 4px;
    padding: 8px 0;
    border-top: 1px solid var(--line);
}
.kind:first-of-type {
    border-top: 0;
}
.small-icon {
    width: 32px;
    height: 32px;
    border-radius: 8px;
    flex: none;
}
.ellipsis {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
}
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
.col > span:first-child {
    min-height: 1.5em;
}
.track {
    flex: 1;
    width: 100%;
    max-width: 56px;
    display: flex;
    align-items: flex-end;
    border-radius: 8px;
    background: var(--surface);
    box-shadow: inset 0 -2px 0 var(--line-strong);
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
