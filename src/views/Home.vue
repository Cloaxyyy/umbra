<script lang="ts">
import type { ForYou } from '../api'

const FOR_YOU_CACHE = new Map<string, { at: number; data: ForYou | null }>()
const FOR_YOU_TTL = 10 * 60 * 1000
</script>

<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref, watch } from 'vue'
import { api, type FriendGame, type GameCard, type Progress, type Status } from '../api'
import GameTile from '../components/GameTile.vue'
import Icon from '../components/Icon.vue'
import { Ago, Compact, Dur } from '../format'
import { ActiveAccount, Go, LaunchMany, LoadCharts, QuickPlay, RefreshRunning, Toast } from '../store'

const status = ref<Status | null>(null)
const progress = ref<Progress | null>(null)
const busy = ref(false)
const trending = ref<GameCard[]>([])
const featured = ref<GameCard | null>(null)
const banners = ref<Record<string, string>>({})
const friends = ref<FriendGame[] | null>(null)
const friendsError = ref('')
const forYou = ref<ForYou | null | undefined>(undefined)
const unlisten: (() => void)[] = []

const active = computed(ActiveAccount)
const upToDate = computed(() => status.value && status.value.installed === status.value.latest.clientVersionUpload)
const percent = computed(() => (progress.value?.total ? Math.round((progress.value.done / progress.value.total) * 100) : 0))
const Mb = (n: number) => (n / 1_048_576).toFixed(0)
const friendRows = computed(() => (friends.value ?? []).flatMap((g) => g.friends.map((f) => ({ ...f, game: g.game }))))

async function RefreshStatus() {
    try {
        status.value = await api.status()
    } catch (e) {
        Toast('bad', String(e))
    }
}

async function Play() {
    busy.value = true
    progress.value = null
    try {
        if (active.value) await LaunchMany([active.value.userId], { kind: 'app' })
        else {
            await api.launch()
            RefreshRunning()
        }
    } catch (e) {
        Toast('bad', String(e))
    }
    busy.value = false
    progress.value = null
    RefreshStatus()
}

async function Install() {
    busy.value = true
    try {
        await api.install()
        Toast('good', 'Roblox is installed and up to date.')
    } catch (e) {
        Toast('bad', String(e))
    }
    busy.value = false
    progress.value = null
    RefreshStatus()
}

async function LoadFriends() {
    friendsError.value = ''
    if (!active.value) {
        friends.value = []
        return
    }
    friends.value = null
    try {
        friends.value = await api.friendsPlaying(active.value.userId)
    } catch (e) {
        friendsError.value = String(e)
        friends.value = []
    }
}

async function LoadForYou(force = false) {
    const key = String(active.value?.userId ?? 'local')
    const hit = FOR_YOU_CACHE.get(key)
    if (hit && !force && Date.now() - hit.at < FOR_YOU_TTL) {
        forYou.value = hit.data
        return
    }
    forYou.value = undefined
    try {
        const data = await api.forYou(active.value?.userId ?? null)
        FOR_YOU_CACHE.set(key, { at: Date.now(), data })
        if (key !== String(active.value?.userId ?? 'local')) return
        forYou.value = data
        const ids = (data?.jumpBackIn ?? []).map((p) => p.game.universeId).filter((id) => !banners.value[id])
        if (ids.length) banners.value = { ...banners.value, ...(await api.banners(ids)) }
    } catch (e) {
        forYou.value = null
        Toast('bad', String(e))
    }
}

const forYouTitle = computed(() => (active.value ? `Picked for ${active.value.displayName}` : 'Picked for you'))
const forYouNote = computed(() => {
    const n = forYou.value?.basedOn ?? 0
    const who = active.value ? 'this account has' : "you've"
    return `Based on ${n} game${n === 1 ? '' : 's'} ${who} played and the genres you spend time in.`
})

watch(
    () => active.value?.userId,
    () => {
        LoadFriends()
        LoadForYou()
    },
)

onMounted(async () => {
    unlisten.push(await api.onProgress((p) => (progress.value = p)))
    RefreshStatus()
    LoadFriends()
    LoadForYou()
    try {
        const charts = await LoadCharts()
        const trend = charts.find((c) => c.id === 'top-trending') ?? charts[0]
        const top = charts.find((c) => c.id === 'top-playing-now') ?? charts[0]
        trending.value = trend?.games.slice(0, 6) ?? []
        featured.value = top?.games[0] ?? null
        const ids = [...trending.value, ...(featured.value ? [featured.value] : [])].map((g) => g.universeId)
        banners.value = await api.banners(ids)
    } catch (e) {
        Toast('bad', String(e))
    }
})
onUnmounted(() => unlisten.forEach((u) => u()))
</script>

<template>
    <div class="home">
        <header class="welcome-row">
            <p class="welcome">
                <template v-if="active"
                    >Welcome back, <img v-if="active.avatar" :src="active.avatar" alt="" class="wav" /><b>{{
                        active.displayName
                    }}</b></template
                >
                <template v-else>Welcome to <b>Umbra</b></template>
            </p>
            <button class="link" @click="Go({ view: 'discover' })">Explore games <Icon name="chevronRight" :size="13" /></button>
        </header>

        <div class="grid">
            <div class="main-col">
                <section class="hero">
                    <img src="/eclipse.png" alt="" class="eclipse" aria-hidden="true" />
                    <div class="hero-body">
                        <div class="row" style="gap: 8px">
                            <span v-if="!status" class="pill">Checking Roblox…</span>
                            <span v-else-if="upToDate" class="pill pill-good"
                                ><span class="dot"></span>Ready · {{ status.latest.version }}</span
                            >
                            <span v-else-if="status.installed" class="pill pill-warn"><span class="dot"></span>Update available</span>
                            <span v-else class="pill pill-bad"><span class="dot"></span>Roblox not installed yet</span>
                        </div>
                        <h1>Step into the shadow.</h1>
                        <p class="muted">
                            Separate instances with their own mods and settings, all your accounts, and every server, from one launcher.
                            Umbra keeps its own copy of Roblox and never touches yours.
                        </p>
                        <div class="row" style="flex-wrap: wrap; gap: 10px">
                            <button class="btn btn-primary btn-lg" :disabled="busy || !status" @click="Play">
                                <Icon name="play" :size="16" />{{
                                    status && !upToDate ? 'Install & play' : active ? `Play as ${active.displayName}` : 'Play'
                                }}
                            </button>
                            <button v-if="status && !upToDate" class="btn btn-lg" :disabled="busy" @click="Install">
                                {{ status.installed ? 'Update only' : 'Install only' }}
                            </button>
                        </div>
                        <div v-if="progress" class="progress">
                            <div class="row between faint" style="font-size: 12px">
                                <span>
                                    <template v-if="progress.stage === 'manifest'">Reading package list</template>
                                    <template v-else-if="progress.stage === 'download'"
                                        >Downloading <code>{{ progress.detail }}</code></template
                                    >
                                    <template v-else-if="progress.stage === 'extract'"
                                        >Unpacking <code>{{ progress.detail }}</code></template
                                    >
                                    <template v-else>Finishing up</template>
                                </span>
                                <span class="mono">{{ Mb(progress.done) }} / {{ Mb(progress.total) }} MB · {{ percent }}%</span>
                            </div>
                            <div class="bar" role="progressbar" :aria-valuenow="percent" aria-valuemin="0" aria-valuemax="100">
                                <div class="fill" :style="{ width: percent + '%' }"></div>
                            </div>
                        </div>
                    </div>
                </section>

                <section v-if="forYou?.jumpBackIn.length" class="stack" style="gap: 14px">
                    <div class="sec-head">
                        <h2><Icon name="clock" :size="16" />Jump back in</h2>
                    </div>
                    <div class="tiles">
                        <div v-for="p in forYou.jumpBackIn.slice(0, 6)" :key="p.game.universeId" class="pick">
                            <GameTile :game="p.game" :banner="banners[p.game.universeId]" wide />
                            <p class="why">
                                <template v-if="p.secs">{{ Dur(p.secs) }} played · </template
                                >{{ p.lastPlayed ? Ago(p.lastPlayed) : 'On another device' }}
                            </p>
                        </div>
                    </div>
                </section>

                <section v-if="forYou === undefined || forYou?.picks.length" class="stack" style="gap: 14px">
                    <div class="sec-head">
                        <h2><Icon name="sparkle" :size="16" />{{ forYouTitle }}</h2>
                        <button class="link" :disabled="forYou === undefined" aria-label="Refresh picks" @click="LoadForYou(true)">
                            <Icon name="refresh" :size="13" />
                        </button>
                    </div>
                    <p v-if="forYou" class="faint note">{{ forYouNote }}</p>
                    <div class="picks">
                        <template v-if="forYou">
                            <div v-for="p in forYou.picks.slice(0, 12)" :key="p.game.universeId" class="pick">
                                <GameTile :game="p.game" />
                                <p class="why reason">{{ p.reason }}</p>
                            </div>
                        </template>
                        <template v-else>
                            <div v-for="i in 12" :key="i" class="ph">
                                <div class="skeleton ph-sq"></div>
                                <div class="skeleton ph-line"></div>
                            </div>
                        </template>
                    </div>
                </section>

                <section v-if="forYou !== undefined && !forYou?.picks.length" class="stack" style="gap: 14px">
                    <div class="sec-head">
                        <h2><Icon name="flame" :size="16" />Trending now</h2>
                        <button class="link" @click="Go({ view: 'discover' })">Browse games <Icon name="chevronRight" :size="13" /></button>
                    </div>
                    <div class="tiles">
                        <template v-if="trending.length">
                            <GameTile v-for="g in trending" :key="g.universeId" :game="g" :banner="banners[g.universeId]" wide />
                        </template>
                        <template v-else>
                            <div v-for="i in 6" :key="i" class="ph">
                                <div class="skeleton ph-img"></div>
                                <div class="skeleton ph-line"></div>
                            </div>
                        </template>
                    </div>
                </section>
            </div>

            <aside class="side-col">
                <section class="card friends">
                    <div class="sec-head">
                        <h2>
                            Friends <span class="faint">({{ friendRows.length }} in game)</span>
                        </h2>
                        <button v-if="active" class="link" aria-label="Refresh friends" @click="LoadFriends">
                            <Icon name="refresh" :size="13" />
                        </button>
                    </div>
                    <p v-if="!active" class="muted small">
                        <button class="link inline" @click="Go({ view: 'accounts' })">Add an account</button> to see which friends are
                        playing and join them.
                    </p>
                    <p v-else-if="friends === null" class="faint small">Looking for friends…</p>
                    <p v-else-if="friendsError" class="small err">{{ friendsError }}</p>
                    <p v-else-if="!friendRows.length" class="faint small">
                        None of {{ active.displayName }}'s friends are in a game right now.
                    </p>
                    <ul v-else class="friend-list">
                        <li v-for="f in friendRows" :key="f.userId" class="friend">
                            <img v-if="f.avatar" :src="f.avatar" alt="" class="fav" /><span v-else class="fav"></span>
                            <div class="f-text">
                                <b>{{ f.displayName }}</b>
                                <button class="f-game" @click="Go({ view: 'game', universeId: f.game.universeId, name: f.game.name })">
                                    {{ f.game.name }}
                                </button>
                            </div>
                            <button
                                class="btn btn-join btn-sm"
                                :title="'Join ' + f.displayName + ' in their server'"
                                @click="QuickPlay(f.placeId, f.gameId ?? undefined)"
                            >
                                <Icon name="play" :size="12" />Join
                            </button>
                        </li>
                    </ul>
                </section>

                <section v-if="featured" class="featured">
                    <button
                        class="feat-img"
                        :aria-label="'Open ' + featured.name"
                        @click="Go({ view: 'game', universeId: featured.universeId, name: featured.name })"
                    >
                        <img
                            v-if="banners[featured.universeId] || featured.icon"
                            :src="banners[featured.universeId] ?? featured.icon!"
                            alt=""
                        />
                        <span class="badge"><Icon name="sparkle" :size="12" />Most played now</span>
                    </button>
                    <div class="row between" style="align-items: flex-start">
                        <div style="min-width: 0">
                            <p class="feat-name">{{ featured.name }}</p>
                            <p class="faint small">
                                <span class="mono">{{ Compact(featured.playerCount) }}</span> playing
                            </p>
                        </div>
                        <button class="btn btn-join btn-sm" @click="QuickPlay(featured.rootPlaceId)">
                            <Icon name="play" :size="12" />Play
                        </button>
                    </div>
                </section>
            </aside>
        </div>
    </div>
</template>

<style scoped>
.home {
    display: flex;
    flex-direction: column;
    gap: 16px;
}
.welcome-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
}
.welcome {
    display: flex;
    align-items: center;
    gap: 8px;
    color: var(--fg-2);
    font-weight: 600;
    font-size: 14px;
}
.welcome b {
    color: var(--fg);
}
.wav {
    width: 22px;
    height: 22px;
    border-radius: 6px;
    background: var(--surface-2);
}
.grid {
    display: grid;
    grid-template-columns: minmax(0, 1fr) 300px;
    gap: 20px;
    align-items: start;
}
@media (max-width: 1120px) {
    .grid {
        grid-template-columns: 1fr;
    }
}
.main-col {
    display: flex;
    flex-direction: column;
    gap: 26px;
    min-width: 0;
}
.side-col {
    display: flex;
    flex-direction: column;
    gap: 16px;
}

.hero {
    position: relative;
    overflow: hidden;
    border-radius: 18px;
    min-height: 280px;
    display: flex;
    align-items: center;
    background: radial-gradient(60% 90% at 85% 50%, rgba(98, 140, 255, 0.18), transparent 70%),
        radial-gradient(40% 70% at 70% 30%, rgba(149, 128, 255, 0.22), transparent 70%), linear-gradient(180deg, #110f1c, #0b0a12);
    border: 1px solid var(--line);
}
.hero::after {
    content: '';
    position: absolute;
    inset: 0;
    background-image: radial-gradient(1px 1px at 12% 22%, #fff8, transparent), radial-gradient(1px 1px at 32% 78%, #fff6, transparent),
        radial-gradient(1px 1px at 55% 15%, #fff5, transparent), radial-gradient(1.5px 1.5px at 48% 60%, #fff7, transparent),
        radial-gradient(1px 1px at 22% 50%, #fff4, transparent), radial-gradient(1px 1px at 62% 88%, #fff5, transparent);
    pointer-events: none;
}
.eclipse {
    position: absolute;
    right: -40px;
    top: 50%;
    width: 360px;
    height: 360px;
    transform: translateY(-50%);
    filter: drop-shadow(0 0 40px rgba(98, 160, 255, 0.35));
    animation: drift 18s ease-in-out infinite alternate;
    pointer-events: none;
}
@keyframes drift {
    from {
        transform: translateY(-50%) rotate(-6deg);
    }
    to {
        transform: translateY(-50%) rotate(6deg);
    }
}
.hero-body {
    position: relative;
    z-index: 1;
    padding: 30px 32px;
    display: flex;
    flex-direction: column;
    gap: 14px;
    max-width: 520px;
}
.hero-body h1 {
    font-size: 32px;
}
.progress {
    display: flex;
    flex-direction: column;
    gap: 8px;
    max-width: 460px;
}
.bar {
    height: 6px;
    border-radius: 999px;
    background: rgba(255, 255, 255, 0.06);
    overflow: hidden;
}
.fill {
    height: 100%;
    background: var(--grad);
    border-radius: 999px;
    transition: width 0.2s linear;
}

.tiles {
    display: grid;
    grid-template-columns: repeat(3, minmax(0, 1fr));
    gap: 16px 14px;
}
@media (max-width: 860px) {
    .tiles {
        grid-template-columns: repeat(2, minmax(0, 1fr));
    }
}
.ph {
    display: flex;
    flex-direction: column;
    gap: 10px;
}
.ph-img {
    aspect-ratio: 16 / 9;
    border-radius: 14px;
}
.ph-line {
    height: 12px;
    width: 70%;
    border-radius: 6px;
}
.ph-sq {
    aspect-ratio: 1;
    border-radius: 14px;
}
.picks {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(140px, 1fr));
    gap: 18px 14px;
}
.pick {
    display: flex;
    flex-direction: column;
    gap: 4px;
    min-width: 0;
}
.why {
    font-size: 11.5px;
    font-weight: 600;
    color: var(--fg-3);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
}
.why.reason {
    color: #b9adff;
    white-space: normal;
    display: -webkit-box;
    -webkit-line-clamp: 2;
    -webkit-box-orient: vertical;
    line-height: 1.35;
}
.note {
    font-size: 12px;
    margin-top: -8px;
}

.friends {
    display: flex;
    flex-direction: column;
    gap: 12px;
    padding: 16px;
}
.small {
    font-size: 13px;
}
.err {
    color: var(--bad);
}
.link.inline {
    font-size: 13px;
    color: var(--accent);
}
.friend-list {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 4px;
    max-height: 300px;
    overflow-y: auto;
}
.friend {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 6px 4px;
    border-radius: 10px;
}
.friend:hover {
    background: var(--surface-2);
}
.fav {
    width: 34px;
    height: 34px;
    border-radius: 10px;
    background: var(--surface-3);
    flex: none;
}
.f-text {
    display: flex;
    flex-direction: column;
    min-width: 0;
    flex: 1;
    line-height: 1.3;
}
.f-text b {
    font-size: 13px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
}
.f-game {
    font-size: 12px;
    color: var(--good);
    text-align: left;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    font-weight: 600;
}
.f-game:hover {
    text-decoration: underline;
}

.featured {
    display: flex;
    flex-direction: column;
    gap: 10px;
}
.feat-img {
    position: relative;
    display: block;
    width: 100%;
    aspect-ratio: 16 / 9;
    border-radius: 14px;
    overflow: hidden;
    background: var(--surface-2);
    border: 1px solid var(--line);
}
.feat-img img {
    width: 100%;
    height: 100%;
    object-fit: cover;
}
.badge {
    position: absolute;
    top: 10px;
    left: 10px;
    display: inline-flex;
    align-items: center;
    gap: 5px;
    padding: 3px 9px;
    border-radius: 999px;
    background: rgba(8, 7, 13, 0.75);
    backdrop-filter: blur(6px);
    font-size: 11px;
    font-weight: 800;
    letter-spacing: 0.04em;
    text-transform: uppercase;
    color: #d8d0ff;
}
.feat-name {
    font-weight: 800;
    font-size: 14px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
}
</style>
