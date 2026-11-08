<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue'
import { api, type Avatar, type AvatarRules, type BodyColors, type Outfit, type Owned, type Scales } from '../api'
import Icon from '../components/Icon.vue'
import { ActiveAccount, Go, store, Toast } from '../store'
import { Rules as LoadRules, WithItem, Without } from '../wardrobe'

type Tab = 'wearing' | 'wardrobe' | 'outfits' | 'body'

const SECTIONS: { label: string; types: number[] }[] = [
    { label: 'Hats', types: [8] },
    { label: 'Hair', types: [41] },
    { label: 'Faces', types: [18, 42] },
    { label: 'Neck & shoulder', types: [43, 44] },
    { label: 'Front & back', types: [45, 46] },
    { label: 'Waist', types: [47] },
    { label: 'Shirts', types: [11, 65, 64, 67, 68] },
    { label: 'Pants', types: [12, 66, 69, 72] },
    { label: 'Shoes', types: [70, 71] },
    { label: 'Classic T-shirts', types: [2] },
    { label: 'Heads', types: [17, 79] },
    { label: 'Animations', types: [51, 55, 53, 52, 50, 54, 48] },
]
const PARTS: { key: keyof BodyColors; label: string }[] = [
    { key: 'headColorId', label: 'Head' },
    { key: 'torsoColorId', label: 'Torso' },
    { key: 'leftArmColorId', label: 'Left arm' },
    { key: 'rightArmColorId', label: 'Right arm' },
    { key: 'leftLegColorId', label: 'Left leg' },
    { key: 'rightLegColorId', label: 'Right leg' },
]
const SCALE_LABELS: Record<string, string> = {
    height: 'Height',
    width: 'Width',
    head: 'Head size',
    proportion: 'Proportions',
    bodyType: 'Body type',
}

const userId = ref<number | null>(ActiveAccount()?.userId ?? null)
const tab = ref<Tab>('wearing')
const av = ref<Avatar | null>(null)
const r = ref<AvatarRules | null>(null)
const picture = ref<string | null>(null)
const drawing = ref(false)
const busy = ref(false)
const error = ref('')

const section = ref(0)
const owned = ref<(Owned & { assetType: number })[]>([])
const cursors = ref<Record<number, string | null>>({})
const loadingOwned = ref(false)
const outfits = ref<Outfit[] | null>(null)
const part = ref<keyof BodyColors | 'all'>('all')
const scales = ref<Scales | null>(null)

const account = computed(() => store.accounts.find((a) => a.userId === userId.value) ?? null)
const wornIds = computed(() => new Set(av.value?.assets.map((a) => a.id) ?? []))
const Hex = (id: number | undefined) => r.value?.bodyColorsPalette.find((p) => p.brickColorId === id)?.hexColor ?? '#555'
const moreOwned = computed(() => SECTIONS[section.value].types.some((t) => cursors.value[t]))

async function Draw() {
    if (!userId.value) return
    drawing.value = true
    try {
        picture.value = await api.renderAvatar(userId.value)
    } catch (e) {
        Toast('bad', String(e))
    }
    drawing.value = false
}

async function Reload() {
    if (!userId.value) return
    error.value = ''
    try {
        av.value = await api.getAvatar(userId.value)
        scales.value = { height: 1, width: 1, head: 1, depth: 1, proportion: 0, bodyType: 0, ...(av.value.scales as Partial<Scales>) }
    } catch (e) {
        error.value = String(e)
    }
    Draw()
}

async function Change(task: () => Promise<number[] | void>, done: string) {
    if (!userId.value || busy.value) return
    busy.value = true
    try {
        const bad = await task()
        Toast(
            bad && bad.length ? 'bad' : 'good',
            bad && bad.length ? `Roblox refused ${bad.length} item${bad.length > 1 ? 's' : ''}.` : done,
        )
        await Reload()
    } catch (e) {
        Toast('bad', String(e))
    }
    busy.value = false
}

const TakeOff = (id: number) => Change(() => api.avatarWear(userId.value!, Without(av.value!.assets, id)), 'Taken off.')
function ToggleItem(item: Owned & { assetType: number }) {
    if (!av.value || !r.value) return
    const ids = wornIds.value.has(item.id) ? Without(av.value.assets, item.id) : WithItem(av.value.assets, item.id, item.assetType, r.value)
    Change(() => api.avatarWear(userId.value!, ids), wornIds.value.has(item.id) ? 'Taken off.' : 'Put on.')
}
const SetType = (t: 'R6' | 'R15') => Change(() => api.avatarSetType(userId.value!, t), `Switched to ${t}.`)
const WearOutfit = (o: Outfit) => Change(() => api.avatarWearOutfit(userId.value!, o.id), `Wearing ${o.name}.`)
function Paint(brickColorId: number) {
    const cur = av.value!.bodyColors as unknown as BodyColors
    const next: BodyColors = { ...cur }
    for (const p of PARTS) if (part.value === 'all' || part.value === p.key) next[p.key] = brickColorId
    Change(() => api.avatarSetColors(userId.value!, next), 'Skin color changed.')
}
const SaveScales = () => Change(() => api.avatarSetScales(userId.value!, scales.value!), 'Body scale saved.')

async function LoadOwned(reset: boolean) {
    if (!userId.value) return
    loadingOwned.value = true
    const types = SECTIONS[section.value].types
    if (reset) {
        owned.value = []
        cursors.value = {}
    }
    try {
        for (const t of types) {
            if (!reset && !cursors.value[t]) continue
            const page = await api.avatarInventory(userId.value, t, reset ? null : cursors.value[t])
            owned.value.push(...page.items.map((i) => ({ ...i, assetType: t })))
            cursors.value[t] = page.nextCursor
        }
    } catch (e) {
        Toast('bad', String(e))
    }
    loadingOwned.value = false
}

async function LoadOutfits() {
    if (!userId.value) return
    outfits.value = null
    try {
        outfits.value = await api.avatarOutfits(userId.value)
    } catch (e) {
        Toast('bad', String(e))
        outfits.value = []
    }
}

watch(userId, () => {
    av.value = null
    picture.value = null
    outfits.value = null
    owned.value = []
    Reload()
    if (tab.value === 'wardrobe') LoadOwned(true)
    if (tab.value === 'outfits') LoadOutfits()
})
watch(tab, (t) => {
    if (t === 'wardrobe' && !owned.value.length) LoadOwned(true)
    if (t === 'outfits' && !outfits.value) LoadOutfits()
})
watch(section, () => LoadOwned(true))
watch(
    () => store.accounts.length,
    () => {
        if (!userId.value) userId.value = ActiveAccount()?.userId ?? null
    },
)

onMounted(async () => {
    try {
        r.value = await LoadRules()
    } catch (e) {
        error.value = String(e)
    }
    Reload()
})
</script>

<template>
    <section v-if="!store.accounts.length" class="card empty">
        <h1>Avatar</h1>
        <p class="muted">Add a Roblox account to change what it wears, switch outfits and pick skin colors here.</p>
        <button class="btn btn-primary" style="align-self: flex-start" @click="Go({ view: 'accounts' })">
            <Icon name="plus" :size="14" />Add account
        </button>
    </section>

    <div v-else class="av">
        <aside class="stage">
            <select id="av-account" v-model.number="userId" class="select" aria-label="Account">
                <option v-for="a in store.accounts" :key="a.userId" :value="a.userId">{{ a.displayName }} (@{{ a.name }})</option>
            </select>
            <div class="render" :class="{ dim: drawing || busy }">
                <img v-if="picture" :src="picture" :alt="(account?.displayName ?? 'Account') + ' avatar'" />
                <p v-if="drawing" class="drawing">Drawing…</p>
            </div>
            <div class="row between">
                <div class="seg" role="group" aria-label="Avatar type">
                    <button
                        v-for="t in ['R15', 'R6'] as const"
                        :key="t"
                        :aria-pressed="av?.avatarType === t"
                        :disabled="busy"
                        @click="av?.avatarType !== t && SetType(t)"
                    >
                        {{ t }}
                    </button>
                </div>
                <button class="btn btn-sm" :disabled="drawing" @click="Reload"><Icon name="refresh" :size="13" />Refresh</button>
            </div>
            <p v-if="error" class="err small">{{ error }}</p>
        </aside>

        <section class="panel">
            <nav class="tabs" role="tablist">
                <button
                    v-for="t in ['wearing', 'wardrobe', 'outfits', 'body'] as Tab[]"
                    :key="t"
                    role="tab"
                    :aria-selected="tab === t"
                    @click="tab = t"
                >
                    {{ { wearing: 'Wearing', wardrobe: 'Wardrobe', outfits: 'Outfits', body: 'Body' }[t] }}
                </button>
                <span v-if="busy" class="faint small saving">Saving…</span>
            </nav>

            <div class="body">
                <template v-if="tab === 'wearing'">
                    <p v-if="!av" class="faint small">Loading…</p>
                    <p v-else-if="!av.assets.length" class="muted">Nothing on. Pick items in Wardrobe.</p>
                    <div v-else class="items">
                        <div v-for="a in av.assets" :key="a.id" class="item worn">
                            <div class="ithumb"><img v-if="a.thumb" :src="a.thumb" alt="" loading="lazy" /></div>
                            <p class="iname">{{ a.name }}</p>
                            <p class="faint tiny">{{ a.assetTypeName }}</p>
                            <button class="off" :disabled="busy" :aria-label="'Take off ' + a.name" @click="TakeOff(a.id)">
                                <Icon name="close" :size="13" />
                            </button>
                        </div>
                    </div>
                </template>

                <template v-else-if="tab === 'wardrobe'">
                    <div class="chips">
                        <button v-for="(s, i) in SECTIONS" :key="s.label" class="chip" :aria-pressed="section === i" @click="section = i">
                            {{ s.label }}
                        </button>
                    </div>
                    <p v-if="loadingOwned && !owned.length" class="faint small">Loading what {{ account?.displayName }} owns…</p>
                    <p v-else-if="!owned.length" class="muted">
                        {{ account?.displayName }} doesn't own anything here yet. Find items in the Catalog.
                    </p>
                    <div class="items">
                        <button
                            v-for="o in owned"
                            :key="o.id"
                            class="item"
                            :class="{ worn: wornIds.has(o.id) }"
                            :disabled="busy"
                            @click="ToggleItem(o)"
                        >
                            <div class="ithumb">
                                <img v-if="o.thumb" :src="o.thumb" alt="" loading="lazy" /><span v-if="wornIds.has(o.id)" class="on"
                                    ><Icon name="check" :size="12"
                                /></span>
                            </div>
                            <p class="iname">{{ o.name }}</p>
                        </button>
                    </div>
                    <div class="row" style="justify-content: center">
                        <button v-if="moreOwned" class="btn" :disabled="loadingOwned" @click="LoadOwned(false)">Load more</button>
                    </div>
                </template>

                <template v-else-if="tab === 'outfits'">
                    <p v-if="outfits === null" class="faint small">Loading outfits…</p>
                    <p v-else-if="!outfits.length" class="muted">
                        No saved outfits. Save one in Roblox's avatar editor and it appears here.
                    </p>
                    <div v-else class="items">
                        <button v-for="o in outfits" :key="o.id" class="item" :disabled="busy" @click="WearOutfit(o)">
                            <div class="ithumb"><img v-if="o.thumb" :src="o.thumb" alt="" loading="lazy" /></div>
                            <p class="iname">{{ o.name }}</p>
                        </button>
                    </div>
                </template>

                <template v-else>
                    <div class="field">
                        <span class="eyebrow">Skin color</span>
                        <div class="parts">
                            <button class="chip" :aria-pressed="part === 'all'" @click="part = 'all'">Whole body</button>
                            <button v-for="p in PARTS" :key="p.key" class="chip" :aria-pressed="part === p.key" @click="part = p.key">
                                <span class="dotc" :style="{ background: Hex(av?.bodyColors[p.key]) }"></span>{{ p.label }}
                            </button>
                        </div>
                        <div class="palette">
                            <button
                                v-for="c in r?.bodyColorsPalette ?? []"
                                :key="c.brickColorId"
                                class="sw"
                                :style="{ background: c.hexColor }"
                                :title="c.name"
                                :aria-label="c.name"
                                :disabled="busy"
                                @click="Paint(c.brickColorId)"
                            ></button>
                        </div>
                    </div>
                    <div v-if="scales && r && av?.avatarType === 'R15'" class="field">
                        <span class="eyebrow">Body scale</span>
                        <label v-for="(label, key) in SCALE_LABELS" :key="key" class="slider">
                            <span>{{ label }}</span>
                            <input
                                :id="'scale-' + key"
                                v-model.number="(scales as any)[key]"
                                type="range"
                                :min="r.scales[key]?.min ?? 0"
                                :max="r.scales[key]?.max ?? 1"
                                :step="r.scales[key]?.increment ?? 0.01"
                            />
                            <span class="mono val">{{ Math.round(((scales as any)[key] as number) * 100) }}%</span>
                        </label>
                        <button class="btn btn-primary" style="align-self: flex-start" :disabled="busy" @click="SaveScales">
                            Save body scale
                        </button>
                    </div>
                    <p v-else-if="av?.avatarType === 'R6'" class="faint small">Body scale only applies to R15 avatars.</p>
                </template>
            </div>
        </section>
    </div>
</template>

<style scoped>
.empty {
    display: flex;
    flex-direction: column;
    gap: 10px;
    max-width: 60ch;
}
.av {
    display: grid;
    grid-template-columns: 340px minmax(0, 1fr);
    gap: 16px;
    align-items: start;
}
@media (max-width: 980px) {
    .av {
        grid-template-columns: 1fr;
    }
}
.small {
    font-size: 12px;
}
.tiny {
    font-size: 11px;
}
.err {
    color: var(--bad);
    user-select: text;
}

.stage {
    position: sticky;
    top: 0;
    display: flex;
    flex-direction: column;
    gap: 12px;
    padding: 14px;
    border-radius: var(--r-lg);
    background: var(--surface);
    border: 1px solid var(--line);
}
.render {
    position: relative;
    aspect-ratio: 1;
    border-radius: 14px;
    overflow: hidden;
    border: 1px solid var(--line);
    background: radial-gradient(60% 50% at 50% 85%, rgba(149, 128, 255, 0.28), transparent 70%),
        radial-gradient(90% 90% at 50% 30%, #1f1b30, #0e0c17);
}
.render img {
    width: 100%;
    height: 100%;
    object-fit: contain;
    transition: opacity 0.2s;
}
.render.dim img {
    opacity: 0.55;
}
.drawing {
    position: absolute;
    inset: auto 0 12px;
    text-align: center;
    font-size: 12px;
    font-weight: 700;
    color: var(--fg-2);
}
.seg {
    display: inline-flex;
    background: var(--bg-2);
    border: 1px solid var(--line);
    border-radius: 10px;
    padding: 3px;
}
.seg button {
    padding: 5px 14px;
    border-radius: 7px;
    color: var(--fg-2);
    font-weight: 800;
    font-size: 12px;
}
.seg button[aria-pressed='true'] {
    background: var(--surface-3);
    color: var(--fg);
}

.panel {
    border-radius: var(--r-lg);
    background: var(--surface);
    border: 1px solid var(--line);
    min-width: 0;
}
.tabs {
    display: flex;
    align-items: center;
    gap: 4px;
    padding: 0 16px;
    border-bottom: 1px solid var(--line);
}
.tabs button {
    padding: 12px;
    font-weight: 700;
    font-size: 13px;
    color: var(--fg-3);
    border-bottom: 2px solid transparent;
    margin-bottom: -1px;
}
.tabs button:hover {
    color: var(--fg);
}
.tabs button[aria-selected='true'] {
    color: var(--fg);
    border-bottom-color: var(--accent);
}
.saving {
    margin-left: auto;
}
.body {
    padding: 18px;
    display: flex;
    flex-direction: column;
    gap: 16px;
}

.chips,
.parts {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
}
.chip {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    height: 30px;
    padding: 0 12px;
    border-radius: 999px;
    background: var(--bg-2);
    border: 1px solid var(--line);
    font-size: 12px;
    font-weight: 700;
    color: var(--fg-2);
}
.chip:hover {
    color: var(--fg);
}
.chip[aria-pressed='true'] {
    background: var(--accent-soft);
    border-color: var(--accent-line);
    color: var(--fg);
}
.dotc {
    width: 12px;
    height: 12px;
    border-radius: 50%;
    border: 1px solid rgba(255, 255, 255, 0.2);
}

.items {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(118px, 1fr));
    gap: 12px;
}
.item {
    position: relative;
    display: flex;
    flex-direction: column;
    gap: 4px;
    text-align: left;
    min-width: 0;
}
.ithumb {
    position: relative;
    aspect-ratio: 1;
    border-radius: 12px;
    border: 1px solid var(--line);
    background: radial-gradient(80% 80% at 50% 35%, #232036, #14121f);
    overflow: hidden;
}
.ithumb img {
    width: 100%;
    height: 100%;
    object-fit: contain;
}
.item:hover:not(:disabled) .ithumb {
    border-color: var(--line-strong);
}
.item.worn .ithumb {
    border-color: var(--accent);
    box-shadow: 0 0 0 1px var(--accent) inset;
}
.on {
    position: absolute;
    top: 6px;
    right: 6px;
    width: 20px;
    height: 20px;
    border-radius: 50%;
    display: grid;
    place-items: center;
    background: var(--accent);
    color: #fff;
}
.iname {
    font-size: 12px;
    font-weight: 700;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
}
.off {
    position: absolute;
    top: 6px;
    right: 6px;
    width: 24px;
    height: 24px;
    border-radius: 50%;
    display: grid;
    place-items: center;
    background: rgba(8, 7, 13, 0.8);
    color: var(--fg-2);
}
.off:hover:not(:disabled) {
    background: var(--bad);
    color: #fff;
}

.field {
    display: flex;
    flex-direction: column;
    gap: 10px;
}
.palette {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(30px, 1fr));
    gap: 6px;
    max-width: 560px;
}
.sw {
    aspect-ratio: 1;
    border-radius: 8px;
    border: 2px solid transparent;
}
.sw:hover:not(:disabled) {
    border-color: var(--fg);
}
.slider {
    display: grid;
    grid-template-columns: 110px 1fr 52px;
    align-items: center;
    gap: 12px;
    font-size: 13px;
    font-weight: 600;
    max-width: 560px;
}
.slider input {
    accent-color: var(--accent);
}
.val {
    text-align: right;
    color: var(--fg-2);
}
</style>
