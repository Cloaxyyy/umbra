<script setup lang="ts">
import { openUrl } from '@tauri-apps/plugin-opener'
import { computed, onMounted, ref, watch } from 'vue'
import { api, type CatalogCategory, type CatalogItem, type ItemValue } from '../api'
import Icon from '../components/Icon.vue'
import { Compact } from '../format'
import { ActiveAccount, Go, store, Toast } from '../store'
import { WearItem } from '../wardrobe'

const SORTS = [
    { id: 0, label: 'Relevance' },
    { id: 1, label: 'Most favorited' },
    { id: 2, label: 'Bestselling' },
    { id: 3, label: 'Recently added' },
    { id: 4, label: 'Price: low to high' },
    { id: 5, label: 'Price: high to low' },
]

const categories = ref<CatalogCategory[]>([])
const category = ref(1)
const subcategory = ref<number | null>(null)
const sort = ref(1)
const keyword = ref(store.route.query ?? '')
const items = ref<CatalogItem[]>([])
const cursor = ref<string | null>(null)
const loading = ref(false)
const error = ref('')
const open = ref<CatalogItem | null>(null)
const owned = ref<boolean | null>(null)
const worth = ref<ItemValue | null>(null)
const DEMAND = ['Terrible', 'Low', 'Normal', 'High', 'Amazing']
const wearing = ref(false)

const active = computed(ActiveAccount)
const subs = computed(() => categories.value.find((c) => c.categoryId === category.value)?.subcategories ?? [])
const PriceLabel = (i: CatalogItem) => (i.price === null ? 'Off sale' : i.price === 0 ? 'Free' : Compact(i.price))
const ItemUrl = (i: CatalogItem) => `https://www.roblox.com/${i.itemType === 'Bundle' ? 'bundles' : 'catalog'}/${i.id}`

async function Load(reset: boolean) {
    loading.value = true
    error.value = ''
    try {
        const page = await api.searchCatalog(keyword.value, category.value, subcategory.value, sort.value, reset ? null : cursor.value)
        items.value = reset ? page.items : [...items.value, ...page.items]
        cursor.value = page.nextCursor
    } catch (e) {
        error.value = String(e)
    }
    loading.value = false
}

let debounce = 0
function OnType() {
    clearTimeout(debounce)
    debounce = window.setTimeout(() => Load(true), 450)
}
watch([category, subcategory, sort], () => Load(true))
watch(category, () => (subcategory.value = null))

async function Show(i: CatalogItem) {
    open.value = i
    owned.value = null
    worth.value = null
    if (i.limited && i.itemType === 'Asset') {
        api.itemValues([i.id])
            .then((v) => {
                if (open.value?.id === i.id) worth.value = v[i.id] ?? null
            })
            .catch(() => {})
    }
    const a = active.value
    if (!a) return
    try {
        owned.value = await api.avatarOwns(a.userId, i.itemType, i.id)
    } catch {
        owned.value = null
    }
}

async function Wear() {
    const i = open.value,
        a = active.value
    if (!i || !a || i.assetType == null) return
    wearing.value = true
    try {
        Toast('good', await WearItem(a.userId, i.id, i.assetType))
    } catch (e) {
        Toast('bad', String(e))
    }
    wearing.value = false
}

async function ViewOnRoblox(i: CatalogItem) {
    try {
        await openUrl(ItemUrl(i))
    } catch (e) {
        Toast('bad', String(e))
    }
}

watch(
    () => store.route.query,
    (q) => {
        if (q !== undefined && q !== keyword.value) {
            keyword.value = q
            Load(true)
        }
    },
)
onMounted(async () => {
    try {
        categories.value = await api.catalogCategories()
    } catch (e) {
        error.value = String(e)
    }
    Load(true)
})
</script>

<template>
    <div class="stack" style="gap: 16px">
        <div class="toolbar">
            <select id="cat" v-model.number="category" class="select pick" aria-label="Category">
                <option v-for="c in categories" :key="c.categoryId" :value="c.categoryId">{{ c.name }}</option>
            </select>
            <span class="divider" aria-hidden="true"></span>
            <label class="search">
                <Icon name="search" :size="16" />
                <input
                    id="catalog-search"
                    v-model="keyword"
                    class="search-input"
                    placeholder="Search the Avatar Shop…"
                    autocomplete="off"
                    @input="OnType"
                    @keydown.enter="Load(true)"
                />
            </label>
            <select id="sort" v-model.number="sort" class="select pick" aria-label="Sort">
                <option v-for="s in SORTS" :key="s.id" :value="s.id">{{ s.label }}</option>
            </select>
        </div>

        <div v-if="subs.length" class="chips">
            <button class="chip" :aria-pressed="subcategory === null" @click="subcategory = null">All</button>
            <button
                v-for="s in subs"
                :key="s.subcategoryId"
                class="chip"
                :aria-pressed="subcategory === s.subcategoryId"
                @click="subcategory = s.subcategoryId"
            >
                {{ s.name }}
            </button>
        </div>

        <p v-if="error" class="err">{{ error }}</p>
        <p v-else-if="!loading && !items.length" class="muted">Nothing matched. Try another word or category.</p>

        <div class="grid">
            <button v-for="i in items" :key="i.itemType + i.id" class="card-item" @click="Show(i)">
                <div class="thumb">
                    <img v-if="i.thumb" :src="i.thumb" alt="" loading="lazy" />
                    <span v-if="i.limited" class="tag limited">Limited</span>
                    <span v-else-if="i.itemType === 'Bundle'" class="tag">Bundle</span>
                </div>
                <p class="name">{{ i.name }}</p>
                <p class="by faint">{{ i.creator }}<Icon v-if="i.creatorVerified" name="check" :size="11" class="verified" /></p>
                <p class="price" :class="{ free: i.price === 0, off: i.price === null }">
                    <span v-if="i.price" class="robux" aria-label="Robux">R$</span>{{ PriceLabel(i) }}
                </p>
            </button>
            <template v-if="loading && !items.length">
                <div v-for="n in 18" :key="n" class="ph">
                    <div class="skeleton ph-img"></div>
                    <div class="skeleton ph-line"></div>
                </div>
            </template>
        </div>
        <div class="row" style="justify-content: center">
            <button v-if="cursor && items.length" class="btn" :disabled="loading" @click="Load(false)">
                {{ loading ? 'Loading…' : 'Load more' }}
            </button>
        </div>

        <div v-if="open" class="overlay" @click.self="open = null">
            <div class="sheet" role="dialog" aria-modal="true" :aria-label="open.name">
                <button class="x" aria-label="Close" @click="open = null"><Icon name="close" :size="16" /></button>
                <div class="big"><img v-if="open.thumb" :src="open.thumb" alt="" /></div>
                <div class="info">
                    <div>
                        <h2 class="title">{{ open.name }}</h2>
                        <p class="muted">
                            by <b>{{ open.creator }}</b>
                        </p>
                    </div>
                    <div class="facts">
                        <div>
                            <p class="eyebrow">Price</p>
                            <p class="mono fv">
                                <span v-if="open.price" class="robux">R$</span
                                >{{ open.price === null ? 'Off sale' : open.price === 0 ? 'Free' : open.price.toLocaleString() }}
                            </p>
                        </div>
                        <div v-if="open.lowestResale">
                            <p class="eyebrow">Best resale</p>
                            <p class="mono fv"><span class="robux">R$</span>{{ open.lowestResale.toLocaleString() }}</p>
                        </div>
                        <div>
                            <p class="eyebrow">Favorites</p>
                            <p class="mono fv">{{ Compact(open.favorites) }}</p>
                        </div>
                        <div v-if="worth">
                            <p class="eyebrow">Value</p>
                            <p class="mono fv">{{ worth.value === null ? 'Unvalued' : worth.value.toLocaleString() }}</p>
                        </div>
                        <div v-if="worth && worth.demand >= 0">
                            <p class="eyebrow">Demand</p>
                            <p class="fv">{{ DEMAND[worth.demand] }}</p>
                        </div>
                    </div>
                    <p v-if="worth?.projected" class="proj">Projected: this item's price was pushed up and isn't trusted.</p>
                    <p v-if="open.description" class="desc">{{ open.description }}</p>
                    <div class="actions">
                        <template v-if="active">
                            <button v-if="owned && open.itemType === 'Asset'" class="btn btn-primary" :disabled="wearing" @click="Wear">
                                <Icon name="shirt" :size="15" />{{ wearing ? 'Putting it on…' : `Wear on ${active.displayName}` }}
                            </button>
                            <span v-else-if="owned" class="pill pill-good"
                                ><span class="dot"></span>{{ active.displayName }} owns this</span
                            >
                            <span v-else-if="owned === false" class="faint small">{{ active.displayName }} doesn't own this.</span>
                        </template>
                        <button v-else class="btn" @click="Go({ view: 'accounts' })">Add an account to wear items</button>
                        <button class="btn" @click="ViewOnRoblox(open)">
                            <Icon name="external" :size="14" />{{ owned ? 'View on Roblox' : 'Get it on Roblox' }}
                        </button>
                    </div>
                    <p class="faint small">Buying happens on roblox.com, so Umbra never spends your Robux.</p>
                </div>
            </div>
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
.pick {
    border-color: transparent;
    background-color: transparent;
    height: 38px;
}
.pick:hover {
    background-color: var(--surface-2);
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
.chips {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
}
.chip {
    height: 30px;
    padding: 0 12px;
    border-radius: 999px;
    background: var(--surface);
    border: 1px solid var(--line);
    font-size: 12px;
    font-weight: 700;
    color: var(--fg-2);
}
.chip:hover {
    color: var(--fg);
}
.chip[aria-pressed='true'] {
    background: #ffffff;
    border-color: transparent;
    color: #000000;
}
.err {
    color: var(--bad);
    user-select: text;
}
.small {
    font-size: 12px;
}

.grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(150px, 1fr));
    gap: 18px 14px;
}
.card-item {
    display: flex;
    flex-direction: column;
    gap: 4px;
    text-align: left;
    min-width: 0;
}
.thumb {
    position: relative;
    aspect-ratio: 1;
    border-radius: 14px;
    overflow: hidden;
    border: 1px solid var(--line);
    background: rgba(255, 255, 255, 0.035);
    margin-bottom: 6px;
}
.thumb img {
    width: 100%;
    height: 100%;
    object-fit: contain;
    transition: transform 0.2s ease;
}
.card-item:hover .thumb img {
    transform: scale(1.05);
}
.card-item:hover .thumb {
    border-color: var(--line-strong);
}
.tag {
    position: absolute;
    top: 8px;
    left: 8px;
    padding: 2px 8px;
    border-radius: 999px;
    font-size: 11px;
    font-weight: 800;
    background: rgba(0, 0, 0, 0.75);
    color: var(--fg-2);
}
.tag.limited {
    color: #1a1200;
    background: var(--warn);
}
.name {
    font-weight: 700;
    font-size: 13px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
}
.by {
    font-size: 12px;
    display: flex;
    align-items: center;
    gap: 4px;
    overflow: hidden;
    white-space: nowrap;
}
.verified {
    color: var(--corona);
    flex: none;
}
.price {
    font-weight: 800;
    font-size: 13px;
    display: flex;
    align-items: center;
    gap: 4px;
    font-variant-numeric: tabular-nums;
}
.price.free {
    color: var(--good);
}
.price.off {
    color: var(--fg-3);
    font-weight: 600;
}
.robux {
    font-size: 11px;
    font-weight: 900;
    color: var(--fg-3);
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

.overlay {
    position: fixed;
    inset: 46px 0 0 64px;
    background: rgba(0, 0, 0, 0.66);
    backdrop-filter: blur(4px);
    display: grid;
    place-items: center;
    z-index: 30;
    padding: 24px;
}
.sheet {
    position: relative;
    width: min(820px, 100%);
    display: grid;
    grid-template-columns: 320px 1fr;
    gap: 22px;
    padding: 22px;
    border-radius: 18px;
    background: var(--panel);
    border: 1px solid var(--line-strong);
    box-shadow: 0 30px 80px rgba(0, 0, 0, 0.6);
}
@media (max-width: 860px) {
    .sheet {
        grid-template-columns: 1fr;
    }
}
.x {
    position: absolute;
    top: 12px;
    right: 12px;
    width: 30px;
    height: 30px;
    border-radius: 8px;
    display: grid;
    place-items: center;
    color: var(--fg-3);
}
.x:hover {
    background: var(--surface-2);
    color: var(--fg);
}
.big {
    aspect-ratio: 1;
    border-radius: 14px;
    background: rgba(255, 255, 255, 0.035);
    border: 1px solid var(--line);
}
.big img {
    width: 100%;
    height: 100%;
    object-fit: contain;
}
.info {
    display: flex;
    flex-direction: column;
    gap: 14px;
    min-width: 0;
}
.title {
    font-size: 20px;
    padding-right: 30px;
}
.info b {
    color: var(--fg);
}
.facts {
    display: flex;
    gap: 10px;
    flex-wrap: wrap;
}
.facts > div {
    padding: 10px 14px;
    border-radius: 12px;
    background: var(--bg-2);
    border: 1px solid var(--line);
}
.fv {
    font-size: 16px;
    font-weight: 700;
    display: flex;
    align-items: center;
    gap: 5px;
}
.proj {
    font-size: 13px;
    font-weight: 700;
    color: var(--bad);
}
.desc {
    font-size: 13px;
    color: var(--fg-2);
    white-space: pre-line;
    max-height: 120px;
    overflow-y: auto;
    user-select: text;
}
.actions {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
    align-items: center;
}
</style>
