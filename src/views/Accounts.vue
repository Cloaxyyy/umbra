<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref } from 'vue'
import { api, type Account, type Preset, type Presence, type Target } from '../api'
import Icon from '../components/Icon.vue'
import { ActiveAccount, AddAccount, LaunchMany, ProfileName, RefreshAccounts, RefreshRunning, store, Toast } from '../store'

async function SetInstance(a: Account, value: string) {
    try {
        await api.setAccountProfile(a.userId, value || null)
        a.profileId = value || null
    } catch (e) {
        Toast('bad', String(e))
    }
}

const presence = ref<Record<number, Presence>>({})
const selected = ref<Set<number>>(new Set())
const multiInstance = ref(true)
const confirmRemove = ref<number | null>(null)

const mode = ref<'place' | 'follow'>('place')
const place = ref('')
const jobId = ref('')
const username = ref('')
const joining = ref(false)

const active = computed(ActiveAccount)
const byId = computed(() => Object.fromEntries(store.accounts.map((a) => [a.userId, a])))
const RunningCount = (id: number) => store.running.filter((r) => r.userId === id).length
const canJoin = computed(
    () => selected.value.size > 0 && !joining.value && (mode.value === 'place' ? place.value.trim() : username.value.trim()),
)

function PresenceLabel(id: number): { cls: string; text: string } {
    const p = presence.value[id]
    if (!p) return { cls: '', text: 'Unknown' }
    switch (p.userPresenceType) {
        case 2:
            return { cls: 'pill-good', text: p.lastLocation ? `In ${p.lastLocation}` : 'In game' }
        case 1:
            return { cls: 'pill-accent', text: 'Online' }
        case 3:
            return { cls: 'pill-warn', text: 'In Studio' }
        default:
            return { cls: 'pill-off', text: 'Offline' }
    }
}

async function LoadPresence() {
    try {
        presence.value = Object.fromEntries((await api.presence()).map((p) => [p.userId, p]))
    } catch {}
}

async function Add() {
    await AddAccount()
    LoadPresence()
}

async function Remove(id: number) {
    try {
        await api.removeAccount(id)
    } catch (e) {
        Toast('bad', String(e))
    }
    selected.value.delete(id)
    confirmRemove.value = null
    await RefreshAccounts()
}

function Toggle(id: number) {
    const s = new Set(selected.value)
    s.has(id) ? s.delete(id) : s.add(id)
    selected.value = s
}

const OpenApp = (a: Account) => LaunchMany([a.userId], { kind: 'app' })

function CurrentTarget(): Target {
    return mode.value === 'place'
        ? { kind: 'place', place: place.value.trim(), jobId: jobId.value.trim() || undefined }
        : { kind: 'follow', username: username.value.trim() }
}
const SelectedIds = () => store.accounts.filter((a) => selected.value.has(a.userId)).map((a) => a.userId)

async function Join() {
    joining.value = true
    await LaunchMany(SelectedIds(), CurrentTarget())
    joining.value = false
}

const presets = ref<Preset[]>([])
const presetName = ref('')
const presetProfile = ref('')
const naming = ref(false)
const confirmPreset = ref<string | null>(null)
const launchingPreset = ref<string | null>(null)

async function LoadPresets() {
    try {
        presets.value = await api.presets()
    } catch (e) {
        Toast('bad', String(e))
    }
}

async function SavePreset() {
    try {
        const p = await api.savePreset({
            id: '',
            name: presetName.value,
            accountIds: SelectedIds(),
            target: CurrentTarget(),
            profileId: presetProfile.value || null,
        })
        Toast('good', `Saved ${p.name}.`)
        naming.value = false
        presetName.value = ''
        presetProfile.value = ''
        LoadPresets()
    } catch (e) {
        Toast('bad', String(e))
    }
}

async function DeletePreset(id: string) {
    try {
        await api.deletePreset(id)
    } catch (e) {
        Toast('bad', String(e))
    }
    confirmPreset.value = null
    LoadPresets()
}

async function RunPreset(p: Preset) {
    const ids = p.accountIds.filter((id) => byId.value[id])
    if (!ids.length) {
        Toast('bad', `None of the accounts in ${p.name} are signed in any more.`)
        return
    }
    launchingPreset.value = p.id
    await LaunchMany(ids, p.target, p.profileId ?? undefined)
    launchingPreset.value = null
}

function TargetLabel(t: Target) {
    if (t.kind === 'app') return 'Roblox app'
    if (t.kind === 'follow') return `Follow ${t.username}`
    return t.jobId ? `${t.place} on a set server` : t.place
}

const moving = ref<number | null>(null)
async function Move(pid: number, mode: 'rejoin' | 'hop') {
    moving.value = pid
    try {
        await api.moveClient(pid, mode)
        Toast('good', mode === 'rejoin' ? 'Rejoining the same server.' : 'Hopping to another server.')
    } catch (e) {
        Toast('bad', String(e))
    }
    moving.value = null
    RefreshRunning()
}

async function Close(pid: number) {
    try {
        await api.closeRunning(pid)
    } catch (e) {
        Toast('bad', String(e))
    }
    RefreshRunning()
}

let timer = 0
onMounted(async () => {
    try {
        multiInstance.value = (await api.getSettings()).multiInstance
    } catch {}
    await RefreshAccounts()
    LoadPresence()
    LoadPresets()
    timer = window.setInterval(LoadPresence, 20_000)
})
onUnmounted(() => clearInterval(timer))
</script>

<template>
    <div class="stack" style="gap: 22px">
        <header class="row between" style="align-items: flex-end; flex-wrap: wrap">
            <div>
                <h1>Your main and your alts. At the same time.</h1>
            </div>
            <button class="btn btn-primary" :disabled="store.adding" @click="Add">
                <Icon name="plus" :size="15" />{{ store.adding ? 'Waiting for login…' : 'Add account' }}
            </button>
        </header>

        <p v-if="store.adding" class="muted">Log in to Roblox in the window that opened. It closes by itself when you're in.</p>

        <section v-if="!store.accounts.length" class="card empty">
            <h2>No accounts yet</h2>
            <p class="muted">
                Add your main and any alts. Each one logs in through Roblox's own page in a private window, and Umbra keeps the session
                encrypted to your Windows account. Your password is never seen or saved.
            </p>
        </section>

        <section v-else class="grid">
            <article v-for="a in store.accounts" :key="a.userId" class="card acct" :class="{ picked: selected.has(a.userId) }">
                <label class="pick" :title="'Select ' + a.displayName">
                    <input :id="'pick-' + a.userId" type="checkbox" :checked="selected.has(a.userId)" @change="Toggle(a.userId)" />
                </label>
                <div class="row" style="gap: 14px">
                    <img v-if="a.avatar" :src="a.avatar" alt="" class="avatar" />
                    <div v-else class="avatar" aria-hidden="true"></div>
                    <div style="min-width: 0">
                        <p class="name">{{ a.displayName }}</p>
                        <p class="faint handle">@{{ a.name }}</p>
                    </div>
                </div>
                <div class="row" style="gap: 6px; flex-wrap: wrap">
                    <span class="pill" :class="PresenceLabel(a.userId).cls"
                        ><span class="dot"></span>{{ PresenceLabel(a.userId).text }}</span
                    >
                    <span v-if="a.userId === active?.userId" class="pill active-pill">Active</span>
                    <span v-if="RunningCount(a.userId)" class="pill">{{ RunningCount(a.userId) }} running</span>
                </div>
                <label class="inst-pick">
                    <span class="eyebrow">Instance</span>
                    <select
                        :id="'inst-' + a.userId"
                        class="select"
                        :value="a.profileId ?? ''"
                        @change="SetInstance(a, ($event.target as HTMLSelectElement).value)"
                    >
                        <option value="">
                            Default ({{ store.profiles?.profiles.find((p) => p.id === store.profiles?.defaultId)?.name ?? 'Main' }})
                        </option>
                        <option v-for="p in store.profiles?.profiles ?? []" :key="p.id" :value="p.id">{{ p.name }}</option>
                    </select>
                </label>
                <div class="row" style="gap: 6px; flex-wrap: wrap">
                    <button class="btn btn-join btn-sm" @click="OpenApp(a)"><Icon name="play" :size="12" />Open Roblox</button>
                    <button v-if="a.userId !== active?.userId" class="btn btn-sm" @click="store.activeId = a.userId">Make active</button>
                    <template v-if="confirmRemove === a.userId">
                        <span class="confirm">
                            <button class="btn btn-danger btn-sm" @click="confirmRemove = null">Cancel</button>
                            <button class="btn btn-sm" @click="Remove(a.userId)">Remove</button>
                        </span>
                    </template>
                    <button v-else class="btn btn-ghost btn-sm" @click="confirmRemove = a.userId">Remove…</button>
                </div>
            </article>
        </section>

        <section v-if="store.accounts.length" class="card stack" style="gap: 16px">
            <div class="row between" style="flex-wrap: wrap">
                <div>
                    <h2>Join together</h2>
                    <p class="muted">Tick the accounts above, then send them all into one game or server.</p>
                </div>
                <div class="seg" role="tablist">
                    <button role="tab" :aria-selected="mode === 'place'" @click="mode = 'place'">Game or server</button>
                    <button role="tab" :aria-selected="mode === 'follow'" @click="mode = 'follow'">Follow a player</button>
                </div>
            </div>
            <div v-if="mode === 'place'" class="fields">
                <label class="field"
                    ><span class="eyebrow">Game link or place ID</span>
                    <input id="join-place" v-model="place" class="input" placeholder="https://www.roblox.com/games/920587237/Adopt-Me" />
                </label>
                <label class="field"
                    ><span class="eyebrow">Server ID (optional)</span>
                    <input id="join-job" v-model="jobId" class="input mono" placeholder="Join a specific server" />
                </label>
            </div>
            <div v-else class="fields">
                <label class="field"
                    ><span class="eyebrow">Username</span>
                    <input id="join-user" v-model="username" class="input" placeholder="Their Roblox username" />
                </label>
            </div>
            <div class="row between" style="flex-wrap: wrap">
                <p v-if="selected.size > 1 && !multiInstance" class="warn">
                    Turn on Multiple instances in Settings to run more than one account.
                </p>
                <span v-else class="faint">{{ selected.size }} selected</span>
                <div class="row" style="gap: 8px">
                    <button class="btn" :disabled="!canJoin" @click="naming = !naming">Save as preset</button>
                    <button class="btn btn-primary" :disabled="!canJoin" @click="Join">
                        <Icon name="play" :size="14" />{{
                            joining ? 'Launching…' : `Join with ${selected.size || ''} ${selected.size === 1 ? 'account' : 'accounts'}`
                        }}
                    </button>
                </div>
            </div>
            <div v-if="naming && canJoin" class="fields">
                <label class="field"
                    ><span class="eyebrow">Preset name</span>
                    <input
                        id="preset-name"
                        v-model="presetName"
                        class="input"
                        maxlength="40"
                        placeholder="Alt farm"
                        @keydown.enter="SavePreset"
                    />
                </label>
                <label v-if="store.profiles" class="field"
                    ><span class="eyebrow">Instance</span>
                    <select id="preset-profile" v-model="presetProfile" class="select">
                        <option value="">Each account's own</option>
                        <option v-for="p in store.profiles.profiles" :key="p.id" :value="p.id">{{ p.name }}</option>
                    </select>
                </label>
                <div class="row" style="justify-content: flex-end; align-self: flex-end; gap: 8px">
                    <button class="btn btn-danger" @click="naming = false">Cancel</button>
                    <button class="btn" :disabled="!presetName.trim()" @click="SavePreset">Save preset</button>
                </div>
            </div>
        </section>

        <section v-if="presets.length" class="card stack" style="gap: 6px">
            <h2>Launch presets</h2>
            <p class="muted">One click sends the same accounts into the same place.</p>
            <div v-for="p in presets" :key="p.id" class="run row between" style="gap: 12px; flex-wrap: wrap">
                <div style="min-width: 0">
                    <p class="name">{{ p.name }}</p>
                    <p class="faint small">
                        {{ p.accountIds.map((id) => byId[id]?.displayName ?? 'Signed out').join(', ') }} · {{ TargetLabel(p.target)
                        }}<template v-if="p.profileId"> · {{ ProfileName(p.profileId) ?? 'Deleted instance' }}</template>
                    </p>
                </div>
                <div class="row" style="gap: 8px">
                    <template v-if="confirmPreset === p.id">
                        <span class="confirm">
                            <button class="btn btn-danger btn-sm" @click="confirmPreset = null">Cancel</button>
                            <button class="btn btn-sm" @click="DeletePreset(p.id)">Delete</button>
                        </span>
                    </template>
                    <button v-else class="btn btn-ghost btn-sm" @click="confirmPreset = p.id">Delete…</button>
                    <button class="btn btn-primary btn-sm" :disabled="!!launchingPreset" @click="RunPreset(p)">
                        <Icon name="play" :size="13" />{{ launchingPreset === p.id ? 'Launching…' : 'Launch' }}
                    </button>
                </div>
            </div>
        </section>

        <section class="card stack" style="gap: 6px">
            <h2>Running now</h2>
            <p v-if="!store.running.length" class="muted">No Roblox clients started from Umbra are running.</p>
            <div v-for="r in store.running" :key="r.pid" class="run row between">
                <div class="row" style="gap: 10px; min-width: 0">
                    <img v-if="r.userId && byId[r.userId]?.avatar" :src="byId[r.userId].avatar!" alt="" class="avatar xs" />
                    <span class="live-dot" aria-hidden="true"></span>
                    <b>{{ r.userId ? (byId[r.userId]?.displayName ?? 'Removed account') : 'Default login' }}</b>
                    <template v-if="r.game">
                        <img v-if="r.game.gameIcon" :src="r.game.gameIcon" alt="" class="avatar xs" />
                        <span class="in-game">In {{ r.game.gameName ?? 'a game' }}</span>
                        <span v-if="r.game.location" class="faint">· {{ r.game.location }}</span>
                    </template>
                    <span v-else class="faint">{{ r.label }}</span>
                    <span class="faint">· {{ ProfileName(r.profileId) ?? 'Instance' }}</span>
                    <span class="faint mono">PID {{ r.pid }}</span>
                </div>
                <div class="row" style="gap: 6px">
                    <template v-if="r.game && r.userId">
                        <button
                            class="btn btn-sm"
                            :disabled="moving === r.pid"
                            title="Rejoin the same server"
                            @click="Move(r.pid, 'rejoin')"
                        >
                            <Icon name="refresh" :size="12" />Rejoin
                        </button>
                        <button
                            class="btn btn-sm"
                            :disabled="moving === r.pid"
                            title="Join a different server of this game"
                            @click="Move(r.pid, 'hop')"
                        >
                            <Icon name="swap" :size="12" />Hop
                        </button>
                    </template>
                    <button class="btn btn-ghost btn-sm" @click="Close(r.pid)">Close</button>
                </div>
            </div>
        </section>
    </div>
</template>

<style scoped>
.grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(250px, 1fr));
    gap: 14px;
}
.acct {
    position: relative;
    display: flex;
    flex-direction: column;
    gap: 14px;
    transition: border-color 0.15s;
}
.acct.picked {
    border-color: rgba(255, 255, 255, 0.5);
}
.pick {
    position: absolute;
    top: 16px;
    right: 16px;
}
.pick input {
    width: 18px;
    height: 18px;
    accent-color: var(--accent);
    cursor: pointer;
    margin: 0;
}
.avatar {
    width: 52px;
    height: 52px;
    border-radius: 14px;
    background: var(--surface-2);
    flex: none;
}
.avatar.xs {
    width: 24px;
    height: 24px;
    border-radius: 6px;
}
.name {
    font-weight: 800;
    font-size: 15px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    padding-right: 24px;
}
.handle {
    font-size: 13px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
}
.active-pill {
    color: #000000;
    background: #ffffff;
    border-color: transparent;
}
.empty {
    display: flex;
    flex-direction: column;
    gap: 8px;
    max-width: 70ch;
}
.warn {
    color: var(--warn);
    font-size: 13px;
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
    font-weight: 600;
    font-size: 13px;
}
.seg button[aria-selected='true'] {
    background: #ffffff;
    border-color: transparent;
    color: #000000;
}
.fields {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(240px, 1fr));
    gap: 12px;
}
.field {
    display: flex;
    flex-direction: column;
    gap: 6px;
}
.field .input {
    width: 100%;
}
.run {
    padding: 10px 0;
    border-top: 1px solid var(--line);
    font-size: 13px;
}
.run:first-of-type {
    border-top: 0;
}
.inst-pick {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 10px;
}
.inst-pick .select {
    height: 30px;
    font-size: 12px;
    min-width: 0;
    flex: 1;
    max-width: 170px;
}
.in-game {
    color: var(--good);
    font-weight: 700;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
}
.live-dot {
    width: 7px;
    height: 7px;
    border-radius: 50%;
    background: var(--good);
    box-shadow: 0 0 0 3px rgba(63, 224, 160, 0.18);
}
</style>
