<script setup lang="ts">
import { listen } from '@tauri-apps/api/event'
import { Window } from '@tauri-apps/api/window'
import { computed, onMounted, onUnmounted, ref, watch } from 'vue'
import { api, type Account, type Friend, type Instance, type Profile, type ProfileTheme } from './api'
import Icon from './components/Icon.vue'
import { ApplyOption, ApplyPreset, CONTROLS, CurrentOption, PRESETS } from './flags'
import { Ago, Dur } from './format'
import { ApplyAccent, THEME_OPTIONS } from './theme'

type Tab = 'now' | 'friends' | 'alts' | 'tuning'
const PRIORITIES: { id: Instance['priority']; label: string }[] = [
    { id: 'normal', label: 'Normal' },
    { id: 'low', label: 'Low' },
    { id: 'idle', label: 'Idle' },
]

const pid = ref<number | null>(null)
const tab = ref<Tab>('now')
const clients = ref<Instance[]>([])
const accounts = ref<Account[]>([])
const profile = ref<Profile | null>(null)
const friends = ref<Friend[] | null>(null)
const busy = ref('')
const note = ref<{ kind: 'good' | 'bad'; text: string } | null>(null)
const confirmClose = ref(false)
const hotkey = ref('Ctrl+Shift+U')
const now = ref(Date.now())
const tuningDirty = ref(false)

const client = computed(() => clients.value.find((c) => c.pid === pid.value) ?? null)
const game = computed(() => client.value?.game ?? null)
const account = computed(() => accounts.value.find((a) => a.userId === client.value?.userId) ?? null)
const others = computed(() => accounts.value.filter((a) => a.userId !== client.value?.userId))
const inGame = computed(() => (friends.value ?? []).filter((f) => f.status === 2 && f.placeId))
const session = computed(() => (game.value ? Dur(Math.max(0, Math.floor(now.value / 1000) - game.value.joinedAt)) : ''))
const canMove = computed(() => !!client.value?.userId)

const extras = ref({ friendsHere: false, recent: false, serverIp: false })
const friendsHere = computed(() => (friends.value ?? []).filter((f) => f.gameId && f.gameId === game.value?.jobId))

interface Recent {
    placeId: number
    jobId: string
    name: string
    at: number
}
const RECENT_KEY = 'umbra.recentServers'
const RECENT_MAX = 8
function ReadRecent(): Recent[] {
    try {
        const v = JSON.parse(localStorage.getItem(RECENT_KEY) ?? '[]')
        return Array.isArray(v) ? v : []
    } catch {
        return []
    }
}
const recent = ref<Recent[]>(ReadRecent())
const recentShown = computed(() => recent.value.filter((r) => r.jobId !== game.value?.jobId).slice(0, 5))
watch(
    () => [game.value?.jobId, extras.value.recent] as const,
    ([jobId, on]) => {
        const g = game.value
        if (!on || !g || !jobId) return
        const entry: Recent = { placeId: g.placeId, jobId, name: g.gameName ?? `Place ${g.placeId}`, at: Math.floor(Date.now() / 1000) }
        recent.value = [entry, ...recent.value.filter((r) => r.jobId !== jobId)].slice(0, RECENT_MAX)
        try {
            localStorage.setItem(RECENT_KEY, JSON.stringify(recent.value))
        } catch {}
    },
)
function GoBack(r: Recent) {
    const p = pid.value
    if (!p) return
    return Run('to', async () => (pid.value = await api.moveClient(p, 'to', r.placeId, r.jobId)), `Going back to ${r.name}.`)
}
async function LoadSettings() {
    try {
        const s = await api.getSettings()
        ApplyAccent(s.accent)
        hotkey.value = s.overlayHotkey
        extras.value = { friendsHere: s.overlayFriendsHere, recent: s.overlayRecent, serverIp: s.overlayServerIp }
    } catch {}
}
function FlipTheme(key: keyof ProfileTheme) {
    const p = profile.value
    if (p) SaveTuning({ ...p, theme: { ...p.theme, [key]: !p.theme[key] } })
}

let noteTimer = 0
function Say(kind: 'good' | 'bad', text: string) {
    note.value = { kind, text }
    clearTimeout(noteTimer)
    noteTimer = window.setTimeout(() => (note.value = null), kind === 'bad' ? 6000 : 3000)
}

async function Refresh() {
    try {
        const [r, a] = await Promise.all([api.running(), api.accounts()])
        clients.value = r.running
        accounts.value = a
        if (!client.value && clients.value.length === 1) Pick(clients.value[0].pid)
        const c = client.value
        if (c) {
            const store = await api.profiles()
            if (!tuningDirty.value) profile.value = JSON.parse(JSON.stringify(store.profiles.find((p) => p.id === c.profileId) ?? null))
        }
    } catch (e) {
        Say('bad', String(e))
    }
}

function Pick(p: number) {
    pid.value = p
    friends.value = null
    api.overlayPick(p)
}

async function Hide() {
    confirmClose.value = false
    await api.overlayHide()
}

async function Run(label: string, task: () => Promise<unknown>, done: string) {
    if (busy.value) return
    busy.value = label
    try {
        await task()
        Say('good', done)
    } catch (e) {
        Say('bad', String(e))
    }
    busy.value = ''
    Refresh()
}

function Move(mode: 'rejoin' | 'hop', done: string) {
    const p = pid.value
    if (!p) return
    return Run(mode, async () => (pid.value = await api.moveClient(p, mode)), done)
}

function MoveTo(f: Friend) {
    const p = pid.value
    if (!p || !f.placeId) return
    return Run('to', async () => (pid.value = await api.moveClient(p, 'to', f.placeId, f.gameId)), `Moving to ${f.displayName}'s server.`)
}

function SetPriority(level: Instance['priority']) {
    const p = pid.value
    if (!p) return
    return Run('priority', () => api.setClientPriority(p, level), `Priority set to ${level}. It applies right away.`)
}

function Bring(a: Account) {
    const g = game.value
    if (!g) return
    return Run(
        'bring',
        () => api.launchAccount(a.userId, { kind: 'place', place: String(g.placeId), jobId: g.jobId ?? undefined }),
        `Launching ${a.displayName} into this server.`,
    )
}

function CloseClient() {
    const p = pid.value
    if (!p) return
    confirmClose.value = false
    return Run('close', () => api.closeRunning(p), 'Roblox closed.')
}

async function Copy(text: string, what: string) {
    try {
        await navigator.clipboard.writeText(text)
        Say('good', `${what} copied.`)
    } catch {
        Say('bad', "Couldn't copy. Select the text and copy it by hand.")
    }
}

async function Star() {
    const g = game.value
    if (!g?.gameName) return
    try {
        const added = await api.toggleFavorite({
            universeId: g.universeId,
            rootPlaceId: g.placeId,
            name: g.gameName,
            icon: g.gameIcon,
            added: 0,
        })
        Say('good', added ? 'Added to favorites.' : 'Removed from favorites.')
    } catch (e) {
        Say('bad', String(e))
    }
}

async function LoadFriends() {
    const u = client.value?.userId
    if (!u) {
        friends.value = []
        return
    }
    friends.value = null
    try {
        friends.value = await api.friends(u)
    } catch (e) {
        Say('bad', String(e))
        friends.value = []
    }
}

async function SaveTuning(next: Profile) {
    profile.value = next
    tuningDirty.value = true
    try {
        profile.value = await api.saveProfile(next)
    } catch (e) {
        Say('bad', String(e))
    }
}
function PickControl(id: string, idx: number) {
    const p = profile.value
    if (!p) return
    SaveTuning({
        ...p,
        flags: ApplyOption(
            CONTROLS.find((c) => c.id === id)!,
            p.flags,
            idx,
        ),
    })
}
function Preset(id: string) {
    const p = profile.value
    if (p) SaveTuning({ ...p, flags: ApplyPreset(id, p.flags) })
}
async function ApplyAndRejoin() {
    await Move('rejoin', 'Rejoining with your new settings.')
    tuningDirty.value = false
}

async function OpenUmbra() {
    const main = await Window.getByLabel('main')
    await main?.show()
    await main?.unminimize()
    await main?.setFocus()
}

function SetTab(t: Tab) {
    tab.value = t
    if (t === 'friends' && friends.value === null) LoadFriends()
}

function OnKey(e: KeyboardEvent) {
    if (e.key === 'Escape') Hide()
}

let unlisten: (() => void) | undefined
let ticker = 0
onMounted(async () => {
    window.addEventListener('keydown', OnKey)
    ticker = window.setInterval(() => (now.value = Date.now()), 1000)
    unlisten = await listen<number | null>('overlay-open', async (e) => {
        confirmClose.value = false
        if (!matchMedia('(prefers-reduced-motion: reduce)').matches)
            document.querySelector('.ov')?.animate(
                [
                    { opacity: 0, transform: 'translateX(18px)' },
                    { opacity: 1, transform: 'none' },
                ],
                { duration: 240, easing: 'cubic-bezier(0.2, 0.8, 0.2, 1)' },
            )
        if (e.payload) Pick(e.payload)
        await Promise.all([Refresh(), LoadSettings()])
        if (tab.value === 'friends' || extras.value.friendsHere) LoadFriends()
    })
    await LoadSettings()
    const t = await api.overlayTarget()
    if (t) pid.value = t
    Refresh()
})
onUnmounted(() => {
    window.removeEventListener('keydown', OnKey)
    clearInterval(ticker)
    unlisten?.()
})
</script>

<template>
    <div class="ov">
        <header class="bar" data-tauri-drag-region>
            <img src="/logo.png" alt="" class="logo" data-tauri-drag-region />
            <span class="brand" data-tauri-drag-region>Umbra</span>
            <select
                v-if="clients.length > 1"
                id="ov-client"
                class="select pick"
                aria-label="Roblox client"
                :value="pid ?? ''"
                @change="Pick(Number(($event.target as HTMLSelectElement).value))"
            >
                <option v-for="c in clients" :key="c.pid" :value="c.pid">
                    {{ accounts.find((a) => a.userId === c.userId)?.displayName ?? 'Default login' }} · {{ c.game?.gameName ?? c.label }}
                </option>
            </select>
            <button class="x" aria-label="Close overlay" title="Close (Esc)" @click="Hide"><Icon name="close" :size="15" /></button>
        </header>

        <div v-if="!client" class="empty">
            <Icon name="gamepad" :size="26" />
            <p v-if="clients.length" class="muted">Pick which Roblox window to control.</p>
            <p v-else class="muted">
                No Roblox client started from Umbra is running. Launch one from Umbra, then press {{ hotkey }} in game.
            </p>
            <button v-for="c in clients" :key="c.pid" class="btn" @click="Pick(c.pid)">
                {{ accounts.find((a) => a.userId === c.userId)?.displayName ?? 'Default login' }} · {{ c.game?.gameName ?? c.label }}
            </button>
            <button class="btn btn-ghost" @click="OpenUmbra">Open Umbra</button>
        </div>

        <template v-else>
            <section class="now">
                <img v-if="game?.gameIcon" :src="game.gameIcon" alt="" class="gicon" /><span v-else class="gicon"></span>
                <div class="gtext">
                    <b>{{ game?.gameName ?? (game ? 'Loading game…' : 'Not in a game yet') }}</b>
                    <span class="faint">
                        {{ account?.displayName ?? 'Default login' }}<template v-if="profile"> · {{ profile.name }}</template
                        ><template v-if="session"> · {{ session }}</template>
                    </span>
                    <span v-if="game?.location" class="faint">Server in {{ game.location }}</span>
                </div>
            </section>

            <nav class="tabs" role="tablist">
                <button
                    v-for="t in ['now', 'friends', 'alts', 'tuning'] as Tab[]"
                    :key="t"
                    role="tab"
                    :aria-selected="tab === t"
                    @click="SetTab(t)"
                >
                    {{ { now: 'Now', friends: 'Friends', alts: 'Alts', tuning: 'Tuning' }[t] }}
                </button>
            </nav>

            <div class="body">
                <template v-if="tab === 'now'">
                    <div class="grid">
                        <button class="act" :disabled="!game || !canMove || !!busy" @click="Move('rejoin', 'Rejoining the same server.')">
                            <Icon name="refresh" :size="16" /><span>{{ busy === 'rejoin' ? 'Rejoining…' : 'Rejoin server' }}</span>
                        </button>
                        <button class="act" :disabled="!game || !canMove || !!busy" @click="Move('hop', 'Hopping to another server.')">
                            <Icon name="swap" :size="16" /><span>{{ busy === 'hop' ? 'Hopping…' : 'Server hop' }}</span>
                        </button>
                        <button class="act" :disabled="!game?.jobId" @click="Copy(game!.jobId!, 'Server ID')">
                            <Icon name="copy" :size="16" /><span>Copy server ID</span>
                        </button>
                        <button
                            class="act"
                            :disabled="!game?.jobId"
                            @click="
                                Copy(
                                    `https://www.roblox.com/games/start?placeId=${game!.placeId}&gameInstanceId=${game!.jobId}`,
                                    'Invite link',
                                )
                            "
                        >
                            <Icon name="external" :size="16" /><span>Copy invite link</span>
                        </button>
                        <button class="act" :disabled="!game?.gameName" @click="Star">
                            <Icon name="star" :size="16" /><span>Favorite game</span>
                        </button>
                        <button class="act" :disabled="!!busy" @click="OpenUmbra">
                            <Icon name="rocket" :size="16" /><span>Open Umbra</span>
                        </button>
                    </div>
                    <p v-if="!canMove" class="faint small">
                        This client was started without a saved account, so rejoin, hop and moving aren't available.
                    </p>

                    <div v-if="extras.serverIp && game?.serverIp" class="block">
                        <div class="row between">
                            <h3>Server address</h3>
                            <button class="btn btn-sm" @click="Copy(game.serverIp, 'Server address')">
                                <Icon name="copy" :size="12" />Copy
                            </button>
                        </div>
                        <p class="mono small">{{ game.serverIp }}</p>
                    </div>

                    <div v-if="extras.friendsHere && client.userId" class="block">
                        <h3>Friends in your server</h3>
                        <p v-if="friends === null" class="faint small">Loading friends…</p>
                        <p v-else-if="!friendsHere.length" class="faint small">No friends in this server right now.</p>
                        <div v-for="f in friendsHere" :key="f.userId" class="item">
                            <img v-if="f.avatar" :src="f.avatar" alt="" class="av" /><span v-else class="av"></span>
                            <span class="itxt"
                                ><b>{{ f.displayName }}</b
                                ><span class="faint">@{{ f.name }}</span></span
                            >
                        </div>
                    </div>

                    <div v-if="extras.recent && recentShown.length" class="block">
                        <h3>Recent servers</h3>
                        <div v-for="r in recentShown" :key="r.jobId" class="item">
                            <span class="itxt"
                                ><b>{{ r.name }}</b
                                ><span class="faint">{{ Ago(r.at) }} · {{ r.jobId.slice(0, 8) }}</span></span
                            >
                            <button class="btn btn-sm" :disabled="!!busy || !canMove" @click="GoBack(r)">
                                <Icon name="enter" :size="12" />Go back
                            </button>
                        </div>
                    </div>

                    <div class="block">
                        <div class="row between">
                            <h3>CPU priority</h3>
                            <span class="pill pill-good"><span class="dot"></span>Live</span>
                        </div>
                        <p class="faint small">Lower it on alts so your main stays smooth. Changes apply instantly.</p>
                        <div class="seg">
                            <button
                                v-for="p in PRIORITIES"
                                :key="p.id"
                                :aria-pressed="client.priority === p.id"
                                :disabled="!!busy"
                                @click="SetPriority(p.id)"
                            >
                                {{ p.label }}
                            </button>
                        </div>
                    </div>

                    <div class="block">
                        <h3>Close this client</h3>
                        <div class="confirm">
                            <template v-if="confirmClose">
                                <button class="btn btn-danger btn-sm" @click="confirmClose = false">Cancel</button>
                                <button class="btn btn-sm" @click="CloseClient">Close Roblox</button>
                            </template>
                            <button v-else class="btn btn-sm" @click="confirmClose = true">Close…</button>
                        </div>
                    </div>
                </template>

                <template v-else-if="tab === 'friends'">
                    <p v-if="!client.userId" class="faint small">Friends need a saved account on this client.</p>
                    <p v-else-if="friends === null" class="faint small">Loading friends…</p>
                    <p v-else-if="!inGame.length" class="faint small">
                        None of {{ account?.displayName }}'s friends are in a game right now.
                    </p>
                    <div v-for="f in inGame" :key="f.userId" class="item">
                        <img v-if="f.avatar" :src="f.avatar" alt="" class="av" /><span v-else class="av"></span>
                        <span class="itxt"
                            ><b>{{ f.displayName }}</b
                            ><span class="playing">{{ f.lastLocation ?? 'In a game' }}</span></span
                        >
                        <button
                            class="btn btn-sm"
                            :disabled="!!busy || !canMove"
                            title="Move this client to their server"
                            @click="MoveTo(f)"
                        >
                            <Icon name="play" :size="12" />Join
                        </button>
                    </div>
                </template>

                <template v-else-if="tab === 'alts'">
                    <p class="faint small">Send another account into this exact server. Each one opens its own Roblox window.</p>
                    <p v-if="!others.length" class="faint small">Add more accounts in Umbra to use this.</p>
                    <div v-for="a in others" :key="a.userId" class="item">
                        <img v-if="a.avatar" :src="a.avatar" alt="" class="av" /><span v-else class="av"></span>
                        <span class="itxt"
                            ><b>{{ a.displayName }}</b
                            ><span class="faint">@{{ a.name }}</span></span
                        >
                        <button class="btn btn-sm" :disabled="!game || !!busy" @click="Bring(a)">
                            <Icon name="play" :size="12" />Join here
                        </button>
                    </div>
                </template>

                <template v-else>
                    <div class="restart">
                        <Icon name="refresh" :size="14" />
                        <span>Roblox reads these settings when it starts, so they apply after you rejoin.</span>
                    </div>
                    <p v-if="!profile" class="faint small">Loading this client's instance…</p>
                    <template v-else>
                        <div class="presets">
                            <button v-for="pr in PRESETS" :key="pr.id" class="btn btn-sm" @click="Preset(pr.id)">{{ pr.label }}</button>
                        </div>
                        <div v-for="c in CONTROLS" :key="c.id" class="ctl">
                            <span>{{ c.label }}</span>
                            <select
                                :id="'ov-ctl-' + c.id"
                                class="select"
                                :value="CurrentOption(c, profile.flags)"
                                @change="PickControl(c.id, Number(($event.target as HTMLSelectElement).value))"
                            >
                                <option v-if="CurrentOption(c, profile.flags) === -1" :value="-1">Custom</option>
                                <option v-for="(o, i) in c.options" :key="i" :value="i">{{ o.label }}</option>
                            </select>
                        </div>
                        <div class="block">
                            <h3>Umbra theme</h3>
                            <p class="faint small">Off unless you turn it on. Applies when you rejoin.</p>
                            <div v-for="o in THEME_OPTIONS" :key="o.key" class="ctl">
                                <span>{{ o.label }}</span>
                                <button
                                    class="switch"
                                    role="switch"
                                    :aria-checked="profile.theme[o.key]"
                                    :aria-label="o.label"
                                    @click="FlipTheme(o.key)"
                                ></button>
                            </div>
                        </div>
                        <button class="btn btn-primary wide" :disabled="!game || !canMove || !!busy" @click="ApplyAndRejoin">
                            <Icon name="refresh" :size="14" />{{ busy === 'rejoin' ? 'Rejoining…' : 'Apply and rejoin' }}
                        </button>
                        <p class="faint small">Saved to the {{ profile.name }} instance, so every client using it gets these settings.</p>
                    </template>
                </template>
            </div>
        </template>

        <footer class="foot">
            <span v-if="note" :class="note.kind" role="status">{{ note.text }}</span>
            <span v-else class="faint"
                ><kbd>{{ hotkey }}</kbd> or <kbd>Esc</kbd> to go back to the game</span
            >
        </footer>
    </div>
</template>

<style scoped>
.ov {
    height: 100vh;
    display: flex;
    flex-direction: column;
    background: var(--panel);
    border: 1px solid var(--line-strong);
}
.bar {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 10px 10px 10px 14px;
    border-bottom: 1px solid var(--line);
}
.logo {
    width: 18px;
    height: 18px;
}
.brand {
    font-weight: 800;
    font-size: 13px;
    flex: 1;
}
.pick {
    height: 30px;
    font-size: 12px;
    max-width: 230px;
}
.x {
    width: 30px;
    height: 30px;
    border-radius: 8px;
    display: grid;
    place-items: center;
    color: var(--fg-2);
}
.x:hover {
    background: var(--surface-3);
    color: var(--fg);
}
.empty {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 12px;
    padding: 24px;
    text-align: center;
    color: var(--fg-3);
}
.now {
    display: flex;
    gap: 12px;
    align-items: center;
    padding: 14px;
}
.gicon {
    width: 54px;
    height: 54px;
    border-radius: 12px;
    background: var(--surface-3);
    flex: none;
}
.gtext {
    display: flex;
    flex-direction: column;
    min-width: 0;
    line-height: 1.35;
    font-size: 12px;
}
.gtext b {
    font-size: 15px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
}
.tabs {
    display: flex;
    gap: 2px;
    padding: 0 10px;
    border-bottom: 1px solid var(--line);
}
.tabs button {
    padding: 10px 10px;
    font-weight: 700;
    font-size: 13px;
    color: var(--fg-3);
    border-bottom: 2px solid transparent;
    margin-bottom: -1px;
}
.tabs button[aria-selected='true'] {
    color: var(--fg);
    border-bottom-color: var(--fg);
}
.body {
    flex: 1;
    overflow-y: auto;
    padding: 14px;
    display: flex;
    flex-direction: column;
    gap: 12px;
}
.small {
    font-size: 12px;
}
.grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 8px;
}
.act {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 12px;
    border-radius: 12px;
    border: 1px solid var(--line);
    background: var(--surface);
    font-size: 13px;
    font-weight: 700;
    text-align: left;
}
.act:hover:not(:disabled) {
    background: var(--surface-3);
}
.act:disabled {
    opacity: 0.4;
}
.act svg {
    color: var(--accent);
    flex: none;
}
.block {
    display: flex;
    flex-direction: column;
    gap: 8px;
    padding-top: 12px;
    border-top: 1px solid var(--line);
}
.block h3 {
    font-size: 14px;
    margin: 0;
}
.seg {
    display: flex;
    background: var(--bg-2);
    border: 1px solid var(--line);
    border-radius: 10px;
    padding: 3px;
}
.seg button {
    flex: 1;
    padding: 6px;
    border-radius: 7px;
    font-weight: 700;
    font-size: 13px;
    color: var(--fg-2);
}
.seg button[aria-pressed='true'] {
    background: var(--select);
    color: var(--select-fg);
}
.item {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 6px 4px;
}
.av {
    width: 34px;
    height: 34px;
    border-radius: 9px;
    background: var(--surface-3);
    flex: none;
}
.itxt {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    line-height: 1.3;
    font-size: 12px;
}
.itxt b {
    font-size: 13px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
}
.itxt span {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
}
.playing {
    color: var(--good);
    font-weight: 600;
}
.restart {
    display: flex;
    gap: 8px;
    align-items: flex-start;
    padding: 10px 12px;
    border-radius: 10px;
    border: 1px solid var(--line-strong);
    background: var(--surface-2);
    font-size: 12px;
    color: var(--fg-2);
}
.restart svg {
    color: var(--warn);
    flex: none;
    margin-top: 2px;
}
.presets {
    display: flex;
    gap: 6px;
}
.ctl {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 10px;
    font-size: 13px;
    font-weight: 600;
}
.ctl .select {
    height: 30px;
    font-size: 12px;
    min-width: 150px;
}
.wide {
    width: 100%;
}
.foot {
    padding: 10px 14px;
    border-top: 1px solid var(--line);
    font-size: 12px;
    font-weight: 600;
    min-height: 40px;
}
.foot .good {
    color: var(--good);
}
.foot .bad {
    color: var(--bad);
    user-select: text;
}
kbd {
    font-family: var(--mono);
    font-size: 11px;
    padding: 1px 6px;
    border-radius: 5px;
    border: 1px solid var(--line-strong);
    color: var(--fg-2);
}
</style>
