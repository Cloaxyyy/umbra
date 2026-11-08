<script setup lang="ts">
import { openUrl } from '@tauri-apps/plugin-opener'
import { computed, onMounted, ref, watch } from 'vue'
import { api, type Friend, type Player, type PlayerProfile } from '../api'
import Icon from '../components/Icon.vue'
import { Compact } from '../format'
import { ActiveAccount, Go, QuickPlay, store, Toast } from '../store'

const STATUS: Record<number, { text: string; cls: string }> = {
    2: { text: 'In game', cls: 'pill-good' },
    1: { text: 'Online', cls: 'pill-accent' },
    3: { text: 'In Studio', cls: 'pill-warn' },
    0: { text: 'Offline', cls: 'pill-off' },
}
const OFFLINE_PAGE = 40

const active = computed(ActiveAccount)
const friends = ref<Friend[] | null>(null)
const loadError = ref('')
const filter = ref('')
const showAllOffline = ref(false)

const query = ref('')
const results = ref<Player[] | null>(null)
const searching = ref(false)

const selected = ref<number | null>(store.route.userId ?? null)
const profile = ref<PlayerProfile | null>(null)
const loadingProfile = ref(false)

const shown = computed(() => {
    const f = filter.value.trim().toLowerCase()
    const list = friends.value ?? []
    return f ? list.filter((x) => x.displayName.toLowerCase().includes(f) || x.name.toLowerCase().includes(f)) : list
})
const groups = computed(() => [
    { id: 'game', label: 'In game', items: shown.value.filter((f) => f.status === 2) },
    { id: 'online', label: 'Online', items: shown.value.filter((f) => f.status === 1 || f.status === 3) },
    { id: 'offline', label: 'Offline', items: shown.value.filter((f) => f.status === 0) },
])
const joined = computed(() => (profile.value?.created ? new Date(profile.value.created).toLocaleDateString() : ''))
const friendOf = computed(() => friends.value?.find((f) => f.userId === selected.value) ?? null)
const joinTarget = computed(() => {
    const f = friendOf.value
    if (f?.status === 2 && f.placeId) return { placeId: f.placeId, gameId: f.gameId, where: f.lastLocation }
    const p = profile.value
    if (p?.status === 2 && p.placeId) return { placeId: p.placeId, gameId: p.gameId, where: p.lastLocation }
    return null
})

function StatusLine(f: { status: number | null; lastLocation: string | null }) {
    if (f.status === 2) return f.lastLocation ? `In ${f.lastLocation}` : 'In a game'
    return STATUS[f.status ?? 0]?.text ?? 'Offline'
}

async function LoadFriends() {
    loadError.value = ''
    if (!active.value) {
        friends.value = []
        return
    }
    friends.value = null
    try {
        friends.value = await api.friends(active.value.userId)
    } catch (e) {
        loadError.value = String(e)
        friends.value = []
    }
}

let debounce = 0
function OnSearch() {
    clearTimeout(debounce)
    if (query.value.trim().length < 3) {
        results.value = null
        return
    }
    debounce = window.setTimeout(RunSearch, 400)
}
async function RunSearch() {
    clearTimeout(debounce)
    const q = query.value.trim()
    if (q.length < 3) return
    searching.value = true
    try {
        results.value = await api.searchPlayers(q)
    } catch (e) {
        Toast('bad', String(e))
    }
    searching.value = false
}
function ClearSearch() {
    query.value = ''
    results.value = null
}

async function Open(userId: number) {
    selected.value = userId
    profile.value = null
    loadingProfile.value = true
    try {
        profile.value = await api.player(userId, active.value?.userId ?? null)
    } catch (e) {
        Toast('bad', String(e))
    }
    loadingProfile.value = false
}

async function Join() {
    const t = joinTarget.value
    if (t) await QuickPlay(t.placeId, t.gameId ?? undefined)
}

async function ViewOnRoblox() {
    if (!selected.value) return
    try {
        await openUrl(`https://www.roblox.com/users/${selected.value}/profile`)
    } catch (e) {
        Toast('bad', String(e))
    }
}

watch(() => active.value?.userId, LoadFriends)
watch(
    () => store.route.userId,
    (u) => {
        if (u && u !== selected.value) Open(u)
    },
)
onMounted(() => {
    LoadFriends()
    if (selected.value) Open(selected.value)
})
</script>

<template>
    <div class="friends">
        <aside class="list">
            <label class="search">
                <Icon name="search" :size="15" />
                <input
                    id="player-search"
                    v-model="query"
                    class="search-input"
                    placeholder="Find any Roblox player…"
                    autocomplete="off"
                    @input="OnSearch"
                    @keydown.enter="RunSearch"
                    @keydown.esc="ClearSearch"
                />
                <button v-if="query" class="clear" aria-label="Clear search" @click="ClearSearch"><Icon name="close" :size="13" /></button>
            </label>

            <template v-if="results">
                <p class="group-label">{{ searching ? 'Searching…' : `${results.length} players` }}</p>
                <p v-if="!searching && !results.length" class="faint small pad">No players with that name.</p>
                <button
                    v-for="p in results"
                    :key="p.userId"
                    class="row-item"
                    :class="{ on: p.userId === selected }"
                    @click="Open(p.userId)"
                >
                    <img v-if="p.avatar" :src="p.avatar" alt="" class="av" /><span v-else class="av"></span>
                    <span class="txt">
                        <b>{{ p.displayName }}<Icon v-if="p.verified" name="check" :size="11" class="tick" /></b>
                        <span class="faint">@{{ p.name }}</span>
                    </span>
                </button>
            </template>

            <template v-else>
                <div class="head">
                    <h2>
                        Friends <span v-if="friends" class="count">{{ friends.length }}</span>
                    </h2>
                    <button v-if="active" class="btn btn-ghost btn-sm" aria-label="Refresh friends" @click="LoadFriends">
                        <Icon name="refresh" :size="13" />
                    </button>
                </div>
                <p v-if="!active" class="faint small pad">
                    <button class="link inline" @click="Go({ view: 'accounts' })">Add an account</button> to see its friends. Search works
                    without one.
                </p>
                <template v-else>
                    <input id="friend-filter" v-model="filter" class="input" :placeholder="`Filter ${active.displayName}'s friends`" />
                    <p v-if="loadError" class="err small pad">{{ loadError }}</p>
                    <template v-if="friends === null">
                        <div v-for="n in 8" :key="n" class="row-item ph">
                            <span class="av skeleton"></span><span class="skeleton line"></span>
                        </div>
                    </template>
                    <p v-else-if="!friends.length && !loadError" class="faint small pad">
                        {{ active.displayName }} has no friends on Roblox yet.
                    </p>
                    <template v-for="g in groups" :key="g.id">
                        <template v-if="g.items.length">
                            <p class="group-label">{{ g.label }} · {{ g.items.length }}</p>
                            <button
                                v-for="f in g.id === 'offline' && !showAllOffline ? g.items.slice(0, OFFLINE_PAGE) : g.items"
                                :key="f.userId"
                                class="row-item"
                                :class="{ on: f.userId === selected }"
                                @click="Open(f.userId)"
                            >
                                <span class="av-wrap">
                                    <img v-if="f.avatar" :src="f.avatar" alt="" class="av" /><span v-else class="av"></span>
                                    <span class="dot" :class="'s' + f.status"></span>
                                </span>
                                <span class="txt">
                                    <b>{{ f.displayName }}</b>
                                    <span :class="f.status === 2 ? 'playing' : 'faint'">{{ StatusLine(f) }}</span>
                                </span>
                            </button>
                            <button
                                v-if="g.id === 'offline' && !showAllOffline && g.items.length > OFFLINE_PAGE"
                                class="link more"
                                @click="showAllOffline = true"
                            >
                                Show {{ g.items.length - OFFLINE_PAGE }} more
                            </button>
                        </template>
                    </template>
                </template>
            </template>
        </aside>

        <section class="detail">
            <div v-if="!selected" class="empty">
                <Icon name="users" :size="28" />
                <p class="muted">Pick a friend or search for a player to see their profile.</p>
            </div>
            <div v-else-if="loadingProfile || !profile" class="stack" style="gap: 14px">
                <div class="skeleton" style="height: 220px; border-radius: 14px"></div>
                <div class="skeleton" style="height: 60px; border-radius: 12px"></div>
            </div>
            <template v-else>
                <div class="top">
                    <div class="render"><img v-if="profile.avatar" :src="profile.avatar" :alt="profile.displayName + ' avatar'" /></div>
                    <div class="who">
                        <h1>{{ profile.displayName }}<Icon v-if="profile.verified" name="check" :size="16" class="tick" /></h1>
                        <p class="faint">@{{ profile.name }}</p>
                        <div class="row" style="gap: 8px; flex-wrap: wrap">
                            <span v-if="profile.status !== null" class="pill" :class="STATUS[profile.status].cls"
                                ><span class="dot"></span>{{ StatusLine(profile) }}</span
                            >
                            <span v-if="profile.banned" class="pill pill-bad">Banned</span>
                            <span class="pill"><Icon name="calendar" :size="12" />Joined {{ joined }}</span>
                        </div>
                        <div class="actions">
                            <button v-if="joinTarget" class="btn btn-primary" @click="Join">
                                <Icon name="play" :size="14" />Join {{ active ? `as ${active.displayName}` : '' }}
                            </button>
                            <button class="btn" @click="ViewOnRoblox"><Icon name="external" :size="14" />View on Roblox</button>
                        </div>
                    </div>
                </div>

                <div class="counts">
                    <div>
                        <p class="eyebrow">Friends</p>
                        <p class="mono val">{{ Compact(profile.friends) }}</p>
                    </div>
                    <div>
                        <p class="eyebrow">Followers</p>
                        <p class="mono val">{{ Compact(profile.followers) }}</p>
                    </div>
                    <div>
                        <p class="eyebrow">Following</p>
                        <p class="mono val">{{ Compact(profile.following) }}</p>
                    </div>
                </div>

                <p v-if="profile.description" class="bio">{{ profile.description }}</p>

                <div v-if="profile.games.length" class="stack" style="gap: 10px">
                    <h2>Their games</h2>
                    <div class="games">
                        <button
                            v-for="g in profile.games"
                            :key="g.universeId"
                            class="game"
                            @click="Go({ view: 'game', universeId: g.universeId, name: g.name })"
                        >
                            <img v-if="g.icon" :src="g.icon" alt="" /><span v-else class="gi"></span>
                            <span class="txt"
                                ><b>{{ g.name }}</b
                                ><span class="faint">{{ Compact(g.visits) }} visits</span></span
                            >
                        </button>
                    </div>
                </div>
            </template>
        </section>
    </div>
</template>

<style scoped>
.friends {
    display: grid;
    grid-template-columns: 330px minmax(0, 1fr);
    gap: 16px;
    align-items: start;
}
@media (max-width: 960px) {
    .friends {
        grid-template-columns: 1fr;
    }
}
.small {
    font-size: 12px;
}
.pad {
    padding: 4px 6px;
}
.err {
    color: var(--bad);
    user-select: text;
}
.list {
    position: sticky;
    top: 0;
    max-height: calc(100vh - 110px);
    overflow-y: auto;
    display: flex;
    flex-direction: column;
    gap: 4px;
    padding: 12px;
    border-radius: var(--r-lg);
    background: var(--surface);
    border: 1px solid var(--line);
}
.search {
    display: flex;
    align-items: center;
    gap: 8px;
    height: 38px;
    padding: 0 10px;
    border-radius: 10px;
    background: var(--bg-2);
    border: 1px solid var(--line-strong);
    color: var(--fg-3);
    margin-bottom: 8px;
}
.search-input {
    flex: 1;
    min-width: 0;
    background: transparent;
    border: 0;
    outline: none;
    font: inherit;
    font-size: 13px;
    font-weight: 600;
    color: var(--fg);
    user-select: text;
}
.clear {
    width: 22px;
    height: 22px;
    border-radius: 6px;
    display: grid;
    place-items: center;
}
.clear:hover {
    background: var(--surface-3);
    color: var(--fg);
}
.head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 2px 4px 6px;
}
.head h2 {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 15px;
}
.count {
    font-size: 11px;
    font-weight: 800;
    padding: 1px 7px;
    border-radius: 999px;
    background: var(--surface-3);
    color: var(--fg-2);
}
.list .input {
    margin-bottom: 6px;
}
.group-label {
    font-size: 12px;
    font-weight: 700;
    color: var(--fg-3);
    padding: 10px 6px 4px;
}
.row-item {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 7px 6px;
    border-radius: 10px;
    text-align: left;
    width: 100%;
}
.row-item:hover {
    background: var(--surface-2);
}
.row-item.on {
    background: var(--select);
    color: var(--select-fg);
}
.row-item.on .faint,
.row-item.on .playing {
    color: rgba(0, 0, 0, 0.65);
}
.av-wrap {
    position: relative;
    flex: none;
}
.av {
    width: 36px;
    height: 36px;
    border-radius: 10px;
    background: var(--surface-3);
    flex: none;
    display: block;
}
.dot {
    position: absolute;
    right: -2px;
    bottom: -2px;
    width: 11px;
    height: 11px;
    border-radius: 50%;
    border: 2px solid var(--bg);
    background: var(--fg-3);
}
.dot.s2 {
    background: var(--good);
}
.dot.s1 {
    background: var(--corona);
}
.dot.s3 {
    background: var(--warn);
}
.txt {
    display: flex;
    flex-direction: column;
    min-width: 0;
    line-height: 1.3;
    font-size: 12px;
}
.txt b {
    font-size: 13px;
    display: flex;
    align-items: center;
    gap: 4px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
}
.txt span {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
}
.playing {
    color: var(--good);
    font-weight: 600;
}
.tick {
    color: var(--corona);
    flex: none;
}
.ph .line {
    height: 12px;
    width: 60%;
    border-radius: 6px;
}
.link.inline {
    font-size: 12px;
    color: var(--accent);
}
.more {
    align-self: center;
    padding: 6px;
}

.detail {
    min-width: 0;
    padding: 20px;
    border-radius: var(--r-lg);
    background: var(--surface);
    border: 1px solid var(--line);
    display: flex;
    flex-direction: column;
    gap: 18px;
}
.empty {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 10px;
    padding: 60px 20px;
    color: var(--fg-3);
    text-align: center;
}
.top {
    display: flex;
    gap: 20px;
    align-items: flex-start;
    flex-wrap: wrap;
}
.render {
    width: 220px;
    aspect-ratio: 1;
    border-radius: 14px;
    background: rgba(255, 255, 255, 0.035);
    border: 1px solid var(--line);
    flex: none;
}
.render img {
    width: 100%;
    height: 100%;
    object-fit: contain;
}
.who {
    display: flex;
    flex-direction: column;
    gap: 8px;
    min-width: 0;
    flex: 1;
}
.who h1 {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 26px;
    overflow-wrap: anywhere;
}
.actions {
    display: flex;
    gap: 8px;
    flex-wrap: wrap;
    margin-top: 8px;
}
.counts {
    display: grid;
    grid-template-columns: repeat(3, minmax(0, 1fr));
    gap: 10px;
}
.counts > div {
    padding: 12px 14px;
    border-radius: 12px;
    background: var(--bg-2);
    border: 1px solid var(--line);
}
.val {
    font-size: 18px;
    font-weight: 700;
}
.bio {
    white-space: pre-line;
    color: var(--fg-2);
    font-size: 13px;
    max-width: 80ch;
    user-select: text;
}
.games {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(220px, 1fr));
    gap: 8px;
}
.game {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 8px;
    border-radius: 12px;
    border: 1px solid var(--line);
    text-align: left;
    min-width: 0;
}
.game:hover {
    background: var(--surface-2);
}
.game img,
.gi {
    width: 44px;
    height: 44px;
    border-radius: 10px;
    background: var(--surface-3);
    flex: none;
}
</style>
