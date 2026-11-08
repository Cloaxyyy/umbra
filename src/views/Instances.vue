<script setup lang="ts">
import { getCurrentWebview } from '@tauri-apps/api/webview'
import { open } from '@tauri-apps/plugin-dialog'
import { computed, onMounted, onUnmounted, ref, watch } from 'vue'
import { api, type Flags, type Mod, type Profile, type Status } from '../api'
import Icon from '../components/Icon.vue'
import { ApplyOption, ApplyPreset, CONTROLS, CurrentOption, IgnoredFlags, PRESETS } from '../flags'
import { Ago, Dur, Mb } from '../format'
import { ActiveAccount, LaunchMany, RefreshProfiles, RefreshRunning, store, Toast } from '../store'

type Tab = 'general' | 'tuning' | 'mods' | 'activity'
const COLORS = ['#9580ff', '#62d4ff', '#3fe0a0', '#ffbd5a', '#ff6283', '#ff8ad8', '#f5f5f7', '#7d8cff']

const sel = ref<string | null>(null)
const tab = ref<Tab>('general')
const draft = ref<Profile | null>(null)
const query = ref('')
const status = ref<Status | null>(null)
const mods = ref<Mod[]>([])
const saved = ref(false)
const creating = ref(false)
const newName = ref('')
const confirmDelete = ref(false)
const confirmModDelete = ref<string | null>(null)
const raw = ref('')
const rawError = ref('')
const pinInput = ref('')
const dragging = ref(false)
const importing = ref(false)

const list = computed(() => (store.profiles?.profiles ?? []).filter((p) => p.name.toLowerCase().includes(query.value.trim().toLowerCase())))
const defaultId = computed(() => store.profiles?.defaultId)
const ignored = computed(() => (draft.value ? IgnoredFlags(draft.value.flags) : []))
const enabledMods = computed(() => (draft.value?.mods ?? []).map((id) => mods.value.find((m) => m.id === id)).filter((m): m is Mod => !!m))
const otherMods = computed(() => mods.value.filter((m) => !draft.value?.mods.includes(m.id)))
const RunningHere = (id: string) => store.running.filter((r) => r.profileId === id).length

const VersionLine = (p: Profile) => (p.pinnedVersion ? `PINNED · ${p.pinnedVersion}` : `LIVE · v${status.value?.latest.version ?? '…'}`)
const Initial = (p: Profile) => p.name.trim().charAt(0).toUpperCase() || '?'

watch(
    [sel, () => store.profiles],
    () => {
        const p = store.profiles?.profiles.find((x) => x.id === sel.value) ?? store.profiles?.profiles[0]
        if (!p) return
        if (sel.value !== p.id) sel.value = p.id
        if (!draft.value || draft.value.id !== p.id) {
            draft.value = JSON.parse(JSON.stringify(p))
            raw.value = JSON.stringify(p.flags, null, 2)
            rawError.value = ''
            confirmDelete.value = false
            pinInput.value = p.pinnedVersion ?? ''
        } else {
            Object.assign(draft.value, { launches: p.launches, playtimeSecs: p.playtimeSecs, lastPlayed: p.lastPlayed, history: p.history })
        }
    },
    { immediate: true },
)

let saveTimer = 0
function SaveSoon() {
    clearTimeout(saveTimer)
    saveTimer = window.setTimeout(Save, 350)
}
async function Save() {
    if (!draft.value) return
    try {
        const p = await api.saveProfile(draft.value)
        const i = store.profiles!.profiles.findIndex((x) => x.id === p.id)
        if (i >= 0) store.profiles!.profiles[i] = p
        saved.value = true
        setTimeout(() => (saved.value = false), 1200)
    } catch (e) {
        Toast('bad', String(e))
    }
}

function SetColor(c: string) {
    draft.value!.color = c
    SaveSoon()
}
function ResetFont() {
    draft.value!.customFont = null
    SaveSoon()
}
function SetFlags(f: Flags) {
    draft.value!.flags = f
    raw.value = JSON.stringify(f, null, 2)
    SaveSoon()
}
function PickControl(id: string, idx: number) {
    const c = CONTROLS.find((x) => x.id === id)!
    SetFlags(ApplyOption(c, draft.value!.flags, idx))
}
function ApplyRaw() {
    rawError.value = ''
    try {
        const v = JSON.parse(raw.value)
        if (!v || typeof v !== 'object' || Array.isArray(v)) throw new Error('Must be a JSON object of flag names to values.')
        if (Object.values(v).some((x) => !['string', 'number', 'boolean'].includes(typeof x)))
            throw new Error('Values must be text, numbers or true/false.')
        SetFlags(v)
    } catch (e) {
        rawError.value = e instanceof Error ? e.message : String(e)
    }
}
function DropIgnored() {
    const f = { ...draft.value!.flags }
    for (const k of ignored.value) delete f[k]
    SetFlags(f)
}

function Pin(guid: string | null) {
    const v = guid?.trim() || null
    if (v && !/^version-[0-9a-f]{16}$/i.test(v)) {
        Toast('bad', 'A build id looks like version-0123456789abcdef.')
        return
    }
    draft.value!.pinnedVersion = v ? v.toLowerCase() : null
    pinInput.value = draft.value!.pinnedVersion ?? ''
    SaveSoon()
}

async function Create() {
    try {
        const p = await api.createProfile(newName.value.trim() || 'New instance')
        await RefreshProfiles()
        sel.value = p.id
        tab.value = 'general'
        creating.value = false
        newName.value = ''
    } catch (e) {
        Toast('bad', String(e))
    }
}
async function Remove() {
    try {
        await api.deleteProfile(draft.value!.id)
        draft.value = null
        sel.value = null
        await RefreshProfiles()
        Toast('good', 'Instance deleted.')
    } catch (e) {
        Toast('bad', String(e))
    }
    confirmDelete.value = false
}
async function MakeDefault(id: string) {
    try {
        await api.setDefaultProfile(id)
        await RefreshProfiles()
    } catch (e) {
        Toast('bad', String(e))
    }
}

async function Play(p: Profile) {
    const a = ActiveAccount()
    try {
        if (a) await LaunchMany([a.userId], { kind: 'app' }, p.id)
        else {
            await api.launch(undefined, p.id)
            Toast('good', `Launching ${p.name}…`)
        }
    } catch (e) {
        Toast('bad', String(e))
    }
    RefreshRunning()
}

async function ImportPaths(paths: string[]) {
    importing.value = true
    for (const path of paths) {
        try {
            const m = await api.importMod(path)
            mods.value = await api.mods()
            draft.value!.mods = [m.id, ...draft.value!.mods]
            SaveSoon()
            Toast('good', `Added ${m.name} (${m.files} files) and turned it on for ${draft.value!.name}.`)
        } catch (e) {
            Toast('bad', String(e))
        }
    }
    importing.value = false
}
async function PickZip() {
    const r = await open({ multiple: true, filters: [{ name: 'Mod ZIP', extensions: ['zip'] }] })
    if (r) ImportPaths(Array.isArray(r) ? r : [r])
}
async function PickFolder() {
    const r = await open({ directory: true })
    if (typeof r === 'string') ImportPaths([r])
}
function ToggleMod(id: string) {
    const m = draft.value!.mods
    draft.value!.mods = m.includes(id) ? m.filter((x) => x !== id) : [...m, id]
    SaveSoon()
}
function MoveMod(id: string, by: -1 | 1) {
    const m = [...draft.value!.mods]
    const i = m.indexOf(id),
        j = i + by
    if (i < 0 || j < 0 || j >= m.length) return
    ;[m[i], m[j]] = [m[j], m[i]]
    draft.value!.mods = m
    SaveSoon()
}
async function DeleteMod(id: string) {
    try {
        await api.deleteMod(id)
        mods.value = await api.mods()
        await RefreshProfiles()
    } catch (e) {
        Toast('bad', String(e))
    }
    confirmModDelete.value = null
}
async function PickFont() {
    const r = await open({ filters: [{ name: 'Font', extensions: ['ttf', 'otf'] }] })
    if (typeof r !== 'string') return
    try {
        draft.value!.customFont = await api.importFont(r)
        SaveSoon()
    } catch (e) {
        Toast('bad', String(e))
    }
}

let unlistenDrop: (() => void) | undefined
onMounted(async () => {
    RefreshProfiles()
    try {
        mods.value = await api.mods()
    } catch (e) {
        Toast('bad', String(e))
    }
    api.status()
        .then((s) => (status.value = s))
        .catch(() => {})
    unlistenDrop = await getCurrentWebview().onDragDropEvent((e) => {
        if (tab.value !== 'mods' || !draft.value) return
        if (e.payload.type === 'enter' || e.payload.type === 'over') dragging.value = true
        else if (e.payload.type === 'leave') dragging.value = false
        else if (e.payload.type === 'drop') {
            dragging.value = false
            ImportPaths(e.payload.paths)
        }
    })
})
onUnmounted(() => {
    unlistenDrop?.()
    clearTimeout(saveTimer)
})
</script>

<template>
    <div class="inst">
        <aside class="list">
            <div class="list-head">
                <h2>
                    Instances <span class="count">{{ store.profiles?.profiles.length ?? 0 }}</span>
                </h2>
                <button class="btn btn-join btn-sm" @click="creating = !creating"><Icon name="plus" :size="13" />New</button>
            </div>
            <form v-if="creating" class="new" @submit.prevent="Create">
                <input id="new-instance" v-model="newName" class="input" placeholder="Name, like Alt farm" maxlength="32" autofocus />
                <button class="btn btn-primary btn-sm" type="submit">Create</button>
            </form>
            <input id="instance-search" v-model="query" class="input" placeholder="Search instances…" />
            <div class="items">
                <div
                    v-for="p in list"
                    :key="p.id"
                    class="item"
                    :class="{ on: p.id === sel }"
                    role="button"
                    tabindex="0"
                    @click="sel = p.id"
                    @keydown.enter="sel = p.id"
                >
                    <span class="badge" :style="{ background: p.color }">{{ Initial(p) }}</span>
                    <div class="item-text">
                        <p class="item-name">{{ p.name }}<Icon v-if="p.id === defaultId" name="star" :size="12" class="star" /></p>
                        <p class="mono item-ver">{{ VersionLine(p) }}</p>
                        <div class="chips">
                            <span v-if="RunningHere(p.id)" class="chip live">{{ RunningHere(p.id) }} running</span>
                            <span v-if="p.mods.length" class="chip">{{ p.mods.length }} mods</span>
                            <span class="chip"><Icon name="clock" :size="11" />{{ Dur(p.playtimeSecs) }}</span>
                            <span class="chip"><Icon name="play" :size="10" />{{ p.launches }}×</span>
                        </div>
                    </div>
                    <button class="go" :aria-label="'Play ' + p.name" :title="'Play ' + p.name" @click.stop="Play(p)">
                        <Icon name="play" :size="14" />
                    </button>
                </div>
            </div>
        </aside>

        <section v-if="draft" class="detail">
            <header class="d-head">
                <span class="badge lg" :style="{ background: draft.color }">{{ Initial(draft) }}</span>
                <div style="min-width: 0; flex: 1">
                    <h1>{{ draft.name }}</h1>
                    <p class="mono faint">{{ VersionLine(draft) }}</p>
                </div>
                <span v-if="saved" class="pill pill-good">Saved</span>
                <button v-if="draft.id !== defaultId" class="btn" @click="MakeDefault(draft.id)">
                    <Icon name="star" :size="14" />Make default
                </button>
                <span v-else class="pill" title="Website Play buttons and accounts without their own instance use this one"
                    ><Icon name="star" :size="12" />Default</span
                >
                <button class="btn btn-primary" @click="Play(draft)"><Icon name="play" :size="14" />Play</button>
            </header>

            <nav class="tabs" role="tablist">
                <button
                    v-for="t in ['general', 'tuning', 'mods', 'activity'] as Tab[]"
                    :key="t"
                    role="tab"
                    :aria-selected="tab === t"
                    @click="tab = t"
                >
                    {{ { general: 'General', tuning: 'Tuning', mods: 'Mods', activity: 'Activity' }[t] }}
                </button>
            </nav>

            <div class="body">
                <template v-if="tab === 'general'">
                    <div class="field">
                        <label class="eyebrow" for="inst-name">Name</label>
                        <input
                            id="inst-name"
                            v-model="draft.name"
                            class="input"
                            maxlength="32"
                            style="max-width: 320px"
                            @input="SaveSoon"
                        />
                    </div>
                    <div class="field">
                        <span class="eyebrow">Color</span>
                        <div class="swatches">
                            <button
                                v-for="c in COLORS"
                                :key="c"
                                class="sw"
                                :style="{ background: c }"
                                :aria-pressed="draft.color === c"
                                :aria-label="'Color ' + c"
                                @click="SetColor(c)"
                            ></button>
                        </div>
                    </div>
                    <div class="field">
                        <span class="eyebrow">Roblox build</span>
                        <div class="seg">
                            <button :aria-pressed="!draft.pinnedVersion" @click="Pin(null)">Follow latest</button>
                            <button
                                :aria-pressed="!!draft.pinnedVersion"
                                @click="Pin(draft.pinnedVersion ?? status?.installed ?? status?.latest.clientVersionUpload ?? null)"
                            >
                                <Icon name="pin" :size="13" />Pin a build
                            </button>
                        </div>
                        <p class="muted small">
                            Pin a build to keep a version that works when a Roblox update breaks something. Roblox removes old builds after
                            a while, so pins don't last forever.
                        </p>
                        <div v-if="draft.pinnedVersion" class="row" style="gap: 8px; flex-wrap: wrap">
                            <input
                                id="pin-guid"
                                v-model="pinInput"
                                class="input mono"
                                style="width: 260px"
                                placeholder="version-0123456789abcdef"
                                @keydown.enter="Pin(pinInput)"
                            />
                            <button class="btn btn-sm" @click="Pin(pinInput)">Use this build</button>
                            <button
                                v-if="status?.latest.clientVersionUpload"
                                class="btn btn-ghost btn-sm"
                                @click="Pin(status!.latest.clientVersionUpload)"
                            >
                                Pin current latest
                            </button>
                        </div>
                    </div>
                    <div class="stats">
                        <div>
                            <p class="eyebrow">Launches</p>
                            <p class="mono val">{{ draft.launches }}</p>
                        </div>
                        <div>
                            <p class="eyebrow">Playtime</p>
                            <p class="mono val">{{ Dur(draft.playtimeSecs) }}</p>
                        </div>
                        <div>
                            <p class="eyebrow">Last played</p>
                            <p class="val">{{ draft.lastPlayed ? Ago(draft.lastPlayed) : 'Never' }}</p>
                        </div>
                        <div>
                            <p class="eyebrow">Accounts using it</p>
                            <p class="mono val">{{ store.accounts.filter((a) => (a.profileId ?? defaultId) === draft!.id).length }}</p>
                        </div>
                    </div>
                    <div class="danger">
                        <template v-if="confirmDelete">
                            <span class="muted small"
                                >Delete {{ draft.name }}? Its flags, mod choices and history go. Mods stay in the library.</span
                            >
                            <button class="btn btn-danger btn-sm" @click="Remove">Delete</button>
                            <button class="btn btn-ghost btn-sm" @click="confirmDelete = false">Keep</button>
                        </template>
                        <button
                            v-else
                            class="btn btn-ghost btn-sm"
                            :disabled="(store.profiles?.profiles.length ?? 0) <= 1"
                            @click="confirmDelete = true"
                        >
                            <Icon name="trash" :size="13" />Delete instance
                        </button>
                    </div>
                </template>

                <template v-else-if="tab === 'tuning'">
                    <div class="note">
                        Roblox only reads a short allow-list of flags (since September 2025). Every control here uses one of them. Frame
                        rate isn't on the list: use Roblox's in-game <b>Settings › Maximum Frame Rate</b>.
                    </div>
                    <div class="presets">
                        <button v-for="pr in PRESETS" :key="pr.id" class="preset" @click="SetFlags(ApplyPreset(pr.id, draft.flags))">
                            <b>{{ pr.label }}</b
                            ><span class="faint">{{ pr.hint }}</span>
                        </button>
                    </div>
                    <div class="controls">
                        <div v-for="c in CONTROLS" :key="c.id" class="control">
                            <div style="min-width: 0">
                                <p class="c-label">{{ c.label }}</p>
                                <p class="faint small">{{ c.hint }}</p>
                            </div>
                            <select
                                :id="'ctl-' + c.id"
                                class="select"
                                :value="CurrentOption(c, draft.flags)"
                                @change="PickControl(c.id, Number(($event.target as HTMLSelectElement).value))"
                            >
                                <option v-if="CurrentOption(c, draft.flags) === -1" :value="-1">Custom</option>
                                <option v-for="(o, i) in c.options" :key="i" :value="i">{{ o.label }}</option>
                            </select>
                        </div>
                    </div>
                    <div class="field">
                        <div class="row between">
                            <span class="eyebrow">All flags (advanced)</span
                            ><button class="btn btn-sm" @click="ApplyRaw">Save flags</button>
                        </div>
                        <textarea id="flags-raw" v-model="raw" rows="9" spellcheck="false"></textarea>
                        <p v-if="rawError" class="err small">{{ rawError }}</p>
                        <div v-if="ignored.length" class="warn-box">
                            <span
                                >Roblox ignores {{ ignored.length === 1 ? 'this flag' : 'these flags' }}:
                                <code v-for="k in ignored" :key="k">{{ k }}</code></span
                            >
                            <button class="btn btn-sm" @click="DropIgnored">Remove ignored</button>
                        </div>
                    </div>
                </template>

                <template v-else-if="tab === 'mods'">
                    <div class="row between" style="flex-wrap: wrap">
                        <div>
                            <h2>Asset mods</h2>
                            <p class="muted small">
                                Mods replace Roblox's textures, sounds, cursors and fonts. They can't change the game program or add
                                scripts. When two mods change the same file, the top one wins.
                            </p>
                        </div>
                        <div class="row" style="gap: 8px">
                            <button class="btn btn-join" :disabled="importing" @click="PickZip">
                                <Icon name="upload" :size="14" />Import ZIP
                            </button>
                            <button class="btn" :disabled="importing" @click="PickFolder">
                                <Icon name="folder" :size="14" />Import folder
                            </button>
                        </div>
                    </div>
                    <div class="drop" :class="{ over: dragging }">
                        <Icon name="upload" :size="22" />
                        <p v-if="importing">Importing…</p>
                        <p v-else>
                            Drop a mod <b>.zip</b> or folder here. It needs <code>content/</code>, <code>PlatformContent/</code> or
                            <code>ExtraContent/</code> inside.
                        </p>
                    </div>

                    <ul v-if="mods.length" class="mods">
                        <li v-for="(m, i) in enabledMods" :key="m.id" class="mod on">
                            <input :id="'mod-' + m.id" type="checkbox" checked @change="ToggleMod(m.id)" />
                            <span class="prio mono">{{ i + 1 }}</span>
                            <label :for="'mod-' + m.id" class="m-text"
                                ><b>{{ m.name }}</b
                                ><span class="faint">{{ m.files }} files · {{ Mb(m.bytes) }}</span></label
                            >
                            <button class="icon-btn" :disabled="i === 0" aria-label="Move up" @click="MoveMod(m.id, -1)">
                                <Icon name="up" :size="14" />
                            </button>
                            <button
                                class="icon-btn"
                                :disabled="i === enabledMods.length - 1"
                                aria-label="Move down"
                                @click="MoveMod(m.id, 1)"
                            >
                                <Icon name="down" :size="14" />
                            </button>
                        </li>
                        <li v-for="m in otherMods" :key="m.id" class="mod">
                            <input :id="'mod-' + m.id" type="checkbox" @change="ToggleMod(m.id)" />
                            <span class="prio"></span>
                            <label :for="'mod-' + m.id" class="m-text"
                                ><b>{{ m.name }}</b
                                ><span class="faint">{{ m.files }} files · {{ Mb(m.bytes) }} · off here</span></label
                            >
                            <template v-if="confirmModDelete === m.id">
                                <button class="btn btn-danger btn-sm" @click="DeleteMod(m.id)">Delete</button>
                                <button class="btn btn-ghost btn-sm" @click="confirmModDelete = null">Keep</button>
                            </template>
                            <button
                                v-else
                                class="icon-btn"
                                :aria-label="'Delete ' + m.name"
                                title="Delete from library"
                                @click="confirmModDelete = m.id"
                            >
                                <Icon name="trash" :size="14" />
                            </button>
                        </li>
                    </ul>
                    <p v-else class="faint small">No mods in the library yet.</p>

                    <div class="font card">
                        <div class="row between" style="flex-wrap: wrap">
                            <div>
                                <h2><Icon name="type" :size="15" /> Custom font</h2>
                                <p class="muted small">Replaces the font Roblox uses for all in-game text.</p>
                            </div>
                            <div class="row" style="gap: 8px">
                                <button class="btn" @click="PickFont">Choose font…</button>
                                <button v-if="draft.customFont" class="btn btn-ghost" @click="ResetFont">Reset</button>
                            </div>
                        </div>
                        <p class="eyebrow" style="margin-top: 6px">Current</p>
                        <p class="cur">{{ draft.customFont ?? 'Default' }}</p>
                    </div>
                </template>

                <template v-else>
                    <p v-if="!draft.history.length" class="muted">
                        Nothing yet. Sessions you play with {{ draft.name }} show up here with how long they lasted.
                    </p>
                    <ul v-else class="history">
                        <li v-for="(s, i) in draft.history" :key="i" class="sess">
                            <span class="dot" :class="{ bad: s.crashed }"></span>
                            <b>{{ s.label }}</b>
                            <span class="faint">{{ Ago(s.at) }}</span>
                            <span class="mono">{{ Dur(s.secs) }}</span>
                            <span v-if="s.crashed" class="pill pill-bad">Crashed</span>
                        </li>
                    </ul>
                </template>
            </div>
        </section>
    </div>
</template>

<style scoped>
.inst {
    display: grid;
    grid-template-columns: 300px minmax(0, 1fr);
    gap: 16px;
    align-items: start;
}
@media (max-width: 980px) {
    .inst {
        grid-template-columns: 1fr;
    }
}
.small {
    font-size: 12px;
}
.err {
    color: var(--bad);
}

.list {
    position: sticky;
    top: 0;
    display: flex;
    flex-direction: column;
    gap: 10px;
    padding: 14px;
    border-radius: var(--r-lg);
    background: var(--surface);
    border: 1px solid var(--line);
}
.list-head {
    display: flex;
    align-items: center;
    justify-content: space-between;
}
.list-head h2 {
    display: flex;
    align-items: center;
    gap: 8px;
}
.count {
    font-size: 11px;
    font-weight: 800;
    padding: 1px 7px;
    border-radius: 999px;
    background: var(--surface-3);
    color: var(--fg-2);
}
.new {
    display: flex;
    gap: 6px;
}
.new .input {
    flex: 1;
}
.items {
    display: flex;
    flex-direction: column;
    gap: 6px;
}
.item {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 10px;
    border-radius: 12px;
    border: 1px solid transparent;
    cursor: pointer;
}
.item:hover {
    background: var(--surface-2);
}
.item.on {
    background: var(--surface-2);
    border-color: var(--line-strong);
}
.badge {
    width: 40px;
    height: 40px;
    border-radius: 10px;
    display: grid;
    place-items: center;
    font-weight: 800;
    font-size: 16px;
    color: #0b0a12;
    flex: none;
}
.badge.lg {
    width: 52px;
    height: 52px;
    border-radius: 13px;
    font-size: 20px;
}
.item-text {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 2px;
}
.item-name {
    font-weight: 800;
    display: flex;
    align-items: center;
    gap: 6px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
}
.star {
    color: var(--warn);
    flex: none;
}
.item-ver {
    font-size: 11px;
    color: var(--fg-3);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
}
.chips {
    display: flex;
    flex-wrap: wrap;
    gap: 4px;
    margin-top: 3px;
}
.chip {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    height: 19px;
    padding: 0 7px;
    border-radius: 6px;
    background: var(--bg-2);
    border: 1px solid var(--line);
    font-size: 11px;
    font-weight: 700;
    color: var(--fg-2);
}
.chip.live {
    color: var(--good);
    border-color: rgba(63, 224, 160, 0.3);
}
.go {
    width: 34px;
    height: 34px;
    flex: none;
    border-radius: 50%;
    display: grid;
    place-items: center;
    background: rgba(63, 224, 160, 0.12);
    color: var(--good);
    border: 1px solid rgba(63, 224, 160, 0.25);
}
.go:hover {
    background: rgba(63, 224, 160, 0.22);
}

.detail {
    border-radius: var(--r-lg);
    background: var(--surface);
    border: 1px solid var(--line);
    min-width: 0;
}
.d-head {
    display: flex;
    align-items: center;
    gap: 14px;
    padding: 18px 20px;
    border-bottom: 1px solid var(--line);
    flex-wrap: wrap;
}
.d-head h1 {
    font-size: 22px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
}
.d-head .faint {
    font-size: 12px;
}
.tabs {
    display: flex;
    gap: 4px;
    padding: 0 16px;
    border-bottom: 1px solid var(--line);
}
.tabs button {
    padding: 12px 12px;
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
.body {
    padding: 20px;
    display: flex;
    flex-direction: column;
    gap: 20px;
}

.field {
    display: flex;
    flex-direction: column;
    gap: 8px;
}
.swatches {
    display: flex;
    gap: 8px;
}
.sw {
    width: 28px;
    height: 28px;
    border-radius: 8px;
    border: 2px solid transparent;
}
.sw[aria-pressed='true'] {
    border-color: var(--fg);
    box-shadow: 0 0 0 2px var(--bg);
}
.seg {
    display: inline-flex;
    align-self: flex-start;
    background: var(--bg-2);
    border: 1px solid var(--line);
    border-radius: 10px;
    padding: 3px;
}
.seg button {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    padding: 6px 12px;
    border-radius: 7px;
    color: var(--fg-2);
    font-weight: 700;
    font-size: 13px;
}
.seg button[aria-pressed='true'] {
    background: var(--surface-3);
    color: var(--fg);
}
.stats {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(140px, 1fr));
    gap: 10px;
}
.stats > div {
    padding: 12px 14px;
    border-radius: 12px;
    background: var(--bg-2);
    border: 1px solid var(--line);
}
.val {
    font-size: 17px;
    font-weight: 700;
}
.danger {
    display: flex;
    align-items: center;
    gap: 8px;
    flex-wrap: wrap;
    padding-top: 6px;
    border-top: 1px solid var(--line);
}

.note {
    font-size: 13px;
    color: var(--fg-2);
    padding: 12px 14px;
    border-radius: 12px;
    background: var(--accent-soft);
    border: 1px solid var(--accent-line);
}
.note b {
    color: var(--fg);
}
.presets {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(180px, 1fr));
    gap: 10px;
}
.preset {
    display: flex;
    flex-direction: column;
    gap: 2px;
    text-align: left;
    padding: 12px 14px;
    border-radius: 12px;
    background: var(--bg-2);
    border: 1px solid var(--line);
    font-size: 12px;
}
.preset b {
    font-size: 14px;
}
.preset:hover {
    border-color: var(--accent-line);
}
.controls {
    display: flex;
    flex-direction: column;
}
.control {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 16px;
    padding: 12px 0;
    border-top: 1px solid var(--line);
}
.control:first-child {
    border-top: 0;
}
.c-label {
    font-weight: 700;
}
.control .select {
    min-width: 170px;
}
.warn-box {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 10px;
    flex-wrap: wrap;
    padding: 10px 12px;
    border-radius: 10px;
    background: rgba(255, 189, 90, 0.08);
    border: 1px solid rgba(255, 189, 90, 0.3);
    color: var(--warn);
    font-size: 12px;
}
.warn-box code {
    margin: 0 4px;
    color: var(--fg);
}

.drop {
    display: flex;
    align-items: center;
    gap: 14px;
    padding: 18px;
    border-radius: 14px;
    border: 1.5px dashed var(--line-strong);
    color: var(--fg-2);
    font-size: 13px;
}
.drop.over {
    border-color: var(--accent);
    background: var(--accent-soft);
    color: var(--fg);
}
.mods {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 6px;
}
.mod {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 10px 12px;
    border-radius: 12px;
    background: var(--bg-2);
    border: 1px solid var(--line);
}
.mod.on {
    border-color: var(--accent-line);
}
.mod input {
    width: 16px;
    height: 16px;
    accent-color: var(--accent);
    margin: 0;
    cursor: pointer;
}
.prio {
    width: 18px;
    font-size: 12px;
    color: var(--accent);
    font-weight: 700;
}
.m-text {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    cursor: pointer;
}
.m-text b {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
}
.m-text span {
    font-size: 12px;
}
.icon-btn {
    width: 28px;
    height: 28px;
    border-radius: 7px;
    display: grid;
    place-items: center;
    color: var(--fg-3);
}
.icon-btn:hover:not(:disabled) {
    background: var(--surface-3);
    color: var(--fg);
}
.icon-btn:disabled {
    opacity: 0.3;
}
.font {
    background: var(--bg-2);
}
.font h2 {
    display: flex;
    align-items: center;
    gap: 8px;
}
.cur {
    font-weight: 800;
    font-size: 16px;
}

.history {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
}
.sess {
    display: grid;
    grid-template-columns: 10px 1fr auto auto auto;
    align-items: center;
    gap: 12px;
    padding: 10px 0;
    border-top: 1px solid var(--line);
    font-size: 13px;
}
.sess:first-child {
    border-top: 0;
}
.sess .dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: var(--good);
}
.sess .dot.bad {
    background: var(--bad);
}
</style>
