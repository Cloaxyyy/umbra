<script setup lang="ts">
import { openUrl } from '@tauri-apps/plugin-opener'
import { computed, onMounted, ref, watch } from 'vue'
import { api, type Collection, type Limited, type TradeDetail, type TradeKind, type TradeSummary } from '../api'
import Icon from '../components/Icon.vue'
import { Ago, Compact } from '../format'
import { ActiveAccount, Go, store, Toast } from '../store'

type Tab = 'collection' | TradeKind

const TABS: { id: Tab; label: string }[] = [
    { id: 'collection', label: 'Collection' },
    { id: 'inbound', label: 'Incoming' },
    { id: 'outbound', label: 'Outgoing' },
    { id: 'completed', label: 'Completed' },
]
const DEMAND = ['Terrible', 'Low', 'Normal', 'High', 'Amazing']
const TREND = ['Lowering', 'Unstable', 'Stable', 'Raising', 'Fluctuating']

const userId = ref<number | null>(ActiveAccount()?.userId ?? null)
const tab = ref<Tab>('collection')
const collection = ref<Collection | null>(null)
const list = ref<TradeSummary[] | null>(null)
const open = ref<TradeDetail | null>(null)
const openId = ref<number | null>(null)
const loading = ref(false)
const error = ref('')
const confirmDecline = ref(false)
const declining = ref(false)

const account = computed(() => store.accounts.find((a) => a.userId === userId.value) ?? null)

function Signed(n: number) {
    return (n > 0 ? '+' : n < 0 ? '−' : '') + Compact(Math.abs(n))
}

function Badges(i: Limited) {
    const out: { text: string; cls: string }[] = []
    if (i.info?.projected) out.push({ text: 'Projected', cls: 'bad' })
    if (i.info?.rare) out.push({ text: 'Rare', cls: 'gold' })
    if (i.info?.hyped) out.push({ text: 'Hyped', cls: 'warn' })
    if (i.info && i.info.demand >= 0) out.push({ text: `${DEMAND[i.info.demand]} demand`, cls: i.info.demand <= 1 ? 'warn' : '' })
    return out
}

async function LoadCollection() {
    if (!userId.value) return
    loading.value = true
    error.value = ''
    collection.value = null
    try {
        collection.value = await api.collection(userId.value)
    } catch (e) {
        error.value = String(e)
    }
    loading.value = false
}

async function LoadList(kind: TradeKind) {
    if (!userId.value) return
    loading.value = true
    error.value = ''
    list.value = null
    open.value = null
    openId.value = null
    try {
        list.value = await api.trades(userId.value, kind)
    } catch (e) {
        error.value = String(e)
        list.value = []
    }
    loading.value = false
}

async function OpenTrade(t: TradeSummary) {
    if (!userId.value) return
    openId.value = t.id
    open.value = null
    confirmDecline.value = false
    try {
        const d = await api.trade(userId.value, t.id)
        if (openId.value === t.id) open.value = d
    } catch (e) {
        Toast('bad', String(e))
    }
}

async function Decline() {
    if (!userId.value || !open.value) return
    declining.value = true
    try {
        await api.declineTrade(userId.value, open.value.id)
        Toast('good', 'Trade declined.')
        LoadList('inbound')
    } catch (e) {
        Toast('bad', String(e))
    }
    declining.value = false
    confirmDecline.value = false
}

async function OpenOnRoblox() {
    try {
        await openUrl('https://www.roblox.com/trades')
    } catch (e) {
        Toast('bad', String(e))
    }
}

function Load() {
    if (tab.value === 'collection') LoadCollection()
    else LoadList(tab.value)
}

watch([userId, tab], Load)
watch(
    () => store.accounts.length,
    () => {
        if (!userId.value) userId.value = ActiveAccount()?.userId ?? null
    },
)
onMounted(Load)
</script>

<template>
    <section v-if="!store.accounts.length" class="card empty">
        <h1>Trades</h1>
        <p class="muted">
            Add a Roblox account to see its limiteds, what they're worth, and every trade it sends or gets with a value check.
        </p>
        <button class="btn btn-primary" style="align-self: flex-start" @click="Go({ view: 'accounts' })">
            <Icon name="plus" :size="14" />Add account
        </button>
    </section>

    <div v-else class="stack" style="gap: 16px">
        <header class="row between" style="flex-wrap: wrap">
            <div class="row" style="gap: 10px">
                <select id="tr-account" v-model.number="userId" class="select" aria-label="Account">
                    <option v-for="a in store.accounts" :key="a.userId" :value="a.userId">{{ a.displayName }} (@{{ a.name }})</option>
                </select>
                <nav class="seg" role="tablist">
                    <button v-for="t in TABS" :key="t.id" role="tab" :aria-selected="tab === t.id" @click="tab = t.id">
                        {{ t.label }}
                    </button>
                </nav>
            </div>
            <button class="btn btn-sm" :disabled="loading" @click="Load"><Icon name="refresh" :size="13" />Refresh</button>
        </header>

        <p v-if="error" class="err">{{ error }}</p>

        <template v-if="tab === 'collection'">
            <div v-if="collection" class="totals">
                <div>
                    <p class="eyebrow">Items</p>
                    <p class="mono big">{{ collection.items.length }}</p>
                </div>
                <div>
                    <p class="eyebrow">Value</p>
                    <p class="mono big good">{{ Compact(collection.totalValue) }}</p>
                </div>
                <div>
                    <p class="eyebrow">Recent average price</p>
                    <p class="mono big">{{ Compact(collection.totalRap) }}</p>
                </div>
                <p class="faint small src">Values from Rolimons. Unvalued items count at their recent average price.</p>
            </div>
            <p v-else-if="loading" class="faint small">Loading {{ account?.displayName }}'s limiteds…</p>
            <p v-if="collection && !collection.items.length" class="muted">{{ account?.displayName }} doesn't own any limited items.</p>
            <div v-if="collection?.items.length" class="grid">
                <div v-for="i in collection.items" :key="i.userAssetId || i.assetId" class="lim">
                    <div class="thumb"><img v-if="i.thumb" :src="i.thumb" alt="" loading="lazy" /></div>
                    <p class="name" :title="i.name">{{ i.name }}</p>
                    <p class="mono val">
                        <span class="good">{{ Compact(i.shown) }}</span
                        ><span class="faint"> · RAP {{ Compact(i.rap) }}</span>
                    </p>
                    <div class="badges">
                        <span v-for="b in Badges(i)" :key="b.text" class="badge" :class="b.cls">{{ b.text }}</span>
                    </div>
                </div>
            </div>
        </template>

        <div v-else class="split">
            <aside class="tlist">
                <p v-if="!list" class="faint small">Loading trades…</p>
                <p v-else-if="!list.length" class="muted small">No {{ TABS.find((t) => t.id === tab)?.label.toLowerCase() }} trades.</p>
                <button v-for="t in list ?? []" :key="t.id" class="trow" :aria-pressed="openId === t.id" @click="OpenTrade(t)">
                    <img v-if="t.partner.avatar" :src="t.partner.avatar" alt="" class="av" /><span v-else class="av"></span>
                    <span class="tname">
                        <b>{{ t.partner.displayName }}</b>
                        <span class="faint">@{{ t.partner.name }} · {{ Ago(Date.parse(t.created) / 1000) }}</span>
                    </span>
                    <span v-if="!t.isActive" class="faint small">{{ t.status }}</span>
                </button>
            </aside>

            <section class="detail">
                <p v-if="!openId" class="muted">Pick a trade to see what each side is worth.</p>
                <p v-else-if="!open" class="faint small">Checking values…</p>
                <template v-else>
                    <div class="net" :class="open.netValue >= 0 ? 'win' : 'loss'">
                        <p class="eyebrow">{{ open.netValue >= 0 ? 'You gain' : 'You lose' }}</p>
                        <p class="mono huge">{{ Signed(open.netValue) }}</p>
                        <p class="faint small">value · {{ Signed(open.netRap) }} recent average price</p>
                    </div>
                    <ul v-if="open.warnings.length" class="warns">
                        <li v-for="w in open.warnings" :key="w"><Icon name="flame" :size="13" />{{ w }}</li>
                    </ul>
                    <div class="sides">
                        <div v-for="(s, k) in { give: open.give, get: open.get }" :key="k" class="side">
                            <div class="row between">
                                <h2>{{ k === 'give' ? 'You give' : 'You get' }}</h2>
                                <span class="mono">{{ Compact(s.value) }}</span>
                            </div>
                            <div v-for="i in s.items" :key="i.userAssetId || i.assetId" class="titem">
                                <div class="tthumb"><img v-if="i.thumb" :src="i.thumb" alt="" /></div>
                                <div class="tinfo">
                                    <p class="name" :title="i.name">{{ i.name }}</p>
                                    <p class="mono small">
                                        <span class="good">{{ Compact(i.shown) }}</span
                                        ><span class="faint"> · RAP {{ Compact(i.rap) }}</span>
                                        <span v-if="i.info && i.info.trend >= 0" class="faint"> · {{ TREND[i.info.trend] }}</span>
                                    </p>
                                    <div class="badges">
                                        <span v-for="b in Badges(i)" :key="b.text" class="badge" :class="b.cls">{{ b.text }}</span>
                                    </div>
                                </div>
                            </div>
                            <p v-if="s.robux" class="mono small">+ R$ {{ s.robux.toLocaleString() }}</p>
                        </div>
                    </div>
                    <div class="actions">
                        <template v-if="tab === 'inbound' && open.isActive">
                            <template v-if="confirmDecline">
                                <span class="muted small">Decline this trade?</span>
                                <button class="btn btn-danger btn-sm" :disabled="declining" @click="Decline">Decline</button>
                                <button class="btn btn-ghost btn-sm" @click="confirmDecline = false">Keep</button>
                            </template>
                            <button v-else class="btn" @click="confirmDecline = true">Decline…</button>
                        </template>
                        <button class="btn" @click="OpenOnRoblox"><Icon name="external" :size="14" />Accept or counter on Roblox</button>
                    </div>
                    <p class="faint small">Accepting and countering happen on roblox.com, where Roblox checks your 2-step verification.</p>
                </template>
            </section>
        </div>
    </div>
</template>

<style scoped>
.empty {
    display: flex;
    flex-direction: column;
    gap: 10px;
    max-width: 60ch;
}
.small {
    font-size: 12px;
}
.err {
    color: var(--bad);
    user-select: text;
}
.good {
    color: var(--good);
}
.seg {
    display: inline-flex;
    background: var(--bg-2);
    border: 1px solid var(--line);
    border-radius: 10px;
    padding: 3px;
}
.seg button {
    padding: 6px 12px;
    border-radius: 7px;
    color: var(--fg-2);
    font-weight: 700;
    font-size: 13px;
}
.seg button[aria-selected='true'] {
    background: var(--surface-3);
    color: var(--fg);
}
.totals {
    display: flex;
    flex-wrap: wrap;
    align-items: flex-end;
    gap: 12px;
}
.totals > div {
    padding: 12px 16px;
    border-radius: 12px;
    background: var(--surface);
    border: 1px solid var(--line);
}
.big {
    font-size: 20px;
    font-weight: 700;
}
.src {
    margin-left: auto;
}
.grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(140px, 1fr));
    gap: 16px 12px;
}
.lim {
    display: flex;
    flex-direction: column;
    gap: 3px;
    min-width: 0;
}
.thumb,
.tthumb {
    aspect-ratio: 1;
    border-radius: 12px;
    border: 1px solid var(--line);
    background: radial-gradient(80% 80% at 50% 35%, #232036, #14121f);
    overflow: hidden;
}
.thumb {
    margin-bottom: 5px;
}
.thumb img,
.tthumb img {
    width: 100%;
    height: 100%;
    object-fit: contain;
}
.name {
    font-weight: 700;
    font-size: 13px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
}
.val {
    font-size: 12px;
    font-weight: 600;
}
.badges {
    display: flex;
    flex-wrap: wrap;
    gap: 4px;
}
.badge {
    font-size: 10px;
    font-weight: 800;
    padding: 1px 7px;
    border-radius: 999px;
    background: var(--surface-2);
    border: 1px solid var(--line);
    color: var(--fg-2);
}
.badge.bad {
    color: var(--bad);
    border-color: rgba(255, 98, 131, 0.4);
}
.badge.warn {
    color: var(--warn);
}
.badge.gold {
    color: #1a1200;
    background: linear-gradient(90deg, #ffd36e, #ffb547);
    border-color: transparent;
}
.split {
    display: grid;
    grid-template-columns: 300px minmax(0, 1fr);
    gap: 16px;
    align-items: start;
}
@media (max-width: 980px) {
    .split {
        grid-template-columns: 1fr;
    }
}
.tlist {
    display: flex;
    flex-direction: column;
    gap: 4px;
    padding: 10px;
    border-radius: var(--r-lg);
    background: var(--surface);
    border: 1px solid var(--line);
    max-height: 640px;
    overflow-y: auto;
}
.trow {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 8px;
    border-radius: 10px;
    text-align: left;
    border: 1px solid transparent;
}
.trow:hover {
    background: var(--surface-2);
}
.trow[aria-pressed='true'] {
    background: var(--surface-2);
    border-color: var(--line-strong);
}
.av {
    width: 34px;
    height: 34px;
    border-radius: 10px;
    background: var(--surface-3);
    flex: none;
}
.tname {
    display: flex;
    flex-direction: column;
    min-width: 0;
    flex: 1;
    line-height: 1.3;
    font-size: 12px;
}
.tname b {
    font-size: 13px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
}
.detail {
    display: flex;
    flex-direction: column;
    gap: 14px;
    padding: 18px;
    border-radius: var(--r-lg);
    background: var(--surface);
    border: 1px solid var(--line);
    min-width: 0;
}
.net {
    padding: 14px 18px;
    border-radius: 14px;
    border: 1px solid var(--line);
}
.net.win {
    background: rgba(63, 224, 160, 0.08);
    border-color: rgba(63, 224, 160, 0.3);
}
.net.win .huge {
    color: var(--good);
}
.net.loss {
    background: rgba(255, 98, 131, 0.08);
    border-color: rgba(255, 98, 131, 0.35);
}
.net.loss .huge {
    color: var(--bad);
}
.huge {
    font-size: 30px;
    font-weight: 800;
}
.warns {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 6px;
}
.warns li {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 8px 12px;
    border-radius: 10px;
    font-size: 13px;
    font-weight: 600;
    color: var(--warn);
    background: rgba(255, 189, 90, 0.08);
    border: 1px solid rgba(255, 189, 90, 0.3);
}
.sides {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 14px;
}
@media (max-width: 760px) {
    .sides {
        grid-template-columns: 1fr;
    }
}
.side {
    display: flex;
    flex-direction: column;
    gap: 8px;
    padding: 12px;
    border-radius: 12px;
    background: var(--bg-2);
    border: 1px solid var(--line);
    min-width: 0;
}
.titem {
    display: flex;
    gap: 10px;
    align-items: center;
}
.tthumb {
    width: 56px;
    flex: none;
}
.tinfo {
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 2px;
}
.actions {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
    align-items: center;
}
</style>
