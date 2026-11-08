<script setup lang="ts">
import { openUrl } from '@tauri-apps/plugin-opener'
import { computed, onMounted, ref } from 'vue'
import { api, type Badge, type GameDetail, type Pass, type Server } from '../api'
import Icon from '../components/Icon.vue'
import { Compact, LikePct } from '../format'
import { ActiveAccount, Go, IsFavorite, LaunchMany, RefreshProfiles, store, Toast, ToggleFavorite } from '../store'

const props = defineProps<{ universeId: number }>()

const game = ref<GameDetail | null>(null)
const fav = computed(() => (game.value ? IsFavorite(game.value.universeId) : false))
function Star() {
    const g = game.value
    if (g) ToggleFavorite({ universeId: g.universeId, rootPlaceId: g.rootPlaceId, name: g.name, icon: g.icon })
}
const error = ref('')
const banner = ref(0)
const showMore = ref(false)

const picked = ref<Set<number>>(new Set())
const launching = ref<string | null>(null)

const servers = ref<Server[]>([])
const cursor = ref<string | null>(null)
const sort = ref<'most' | 'fewest' | 'ping'>('most')
const hideFull = ref(true)
const loadingServers = ref(false)
const copied = ref<string | null>(null)

const pct = computed(() => (game.value ? LikePct(game.value.upVotes, game.value.downVotes) : null))
const votes = computed(() => (game.value ? game.value.upVotes + game.value.downVotes : 0))
const updated = computed(() => {
    if (!game.value?.updated) return ''
    const days = Math.floor((Date.now() - new Date(game.value.updated).getTime()) / 86_400_000)
    return days <= 0
        ? 'Today'
        : days === 1
          ? 'Yesterday'
          : days < 30
            ? `${days} days ago`
            : new Date(game.value.updated).toLocaleDateString()
})
const shown = computed(() => {
    let list = hideFull.value ? servers.value.filter((s) => s.playing < s.maxPlayers) : servers.value.slice()
    if (sort.value === 'ping') list = list.slice().sort((a, b) => (a.ping ?? 9999) - (b.ping ?? 9999))
    return list
})
const pickedIds = computed(() => store.accounts.filter((a) => picked.value.has(a.userId)).map((a) => a.userId))

function Toggle(id: number) {
    const s = new Set(picked.value)
    s.has(id) ? s.delete(id) : s.add(id)
    picked.value = s
}

async function Join(serverId?: string) {
    if (!game.value) return
    if (!pickedIds.value.length) {
        if (!store.accounts.length) {
            Toast('bad', 'Add a Roblox account to play from Umbra.')
            Go({ view: 'accounts' })
        } else Toast('bad', 'Pick at least one account to play as.')
        return
    }
    launching.value = serverId ?? 'any'
    await LaunchMany(pickedIds.value, { kind: 'place', place: String(game.value.rootPlaceId), jobId: serverId })
    launching.value = null
}

const privateLink = ref('')
const LINK_RE = /privateServerLinkCode=[A-Za-z0-9_-]+/
async function JoinPrivate() {
    if (!game.value) return
    const m = privateLink.value.match(LINK_RE)
    if (!m) {
        Toast('bad', 'Paste a private server link with privateServerLinkCode in it. Open roblox.com/share links in a browser first.')
        return
    }
    if (!pickedIds.value.length) {
        Toast('bad', 'Pick at least one account to play as.')
        return
    }
    launching.value = 'private'
    await LaunchMany(pickedIds.value, { kind: 'place', place: `https://www.roblox.com/games/${game.value.rootPlaceId}?${m[0]}` })
    launching.value = null
}

const gameInstance = computed(() => (game.value ? (store.profiles?.games?.[String(game.value.rootPlaceId)] ?? '') : ''))
async function SetInstance(e: Event) {
    if (!game.value) return
    const id = (e.target as HTMLSelectElement).value
    try {
        await api.setGameProfile(game.value.rootPlaceId, id || null)
        await RefreshProfiles()
    } catch (err) {
        Toast('bad', String(err))
    }
}

const passes = ref<Pass[] | null>(null)
const badges = ref<Badge[] | null>(null)
const allBadges = ref(false)
const shownBadges = computed(() => (allBadges.value ? badges.value : badges.value?.slice(0, 8)) ?? [])
function OpenPass(id: number) {
    openUrl(`https://www.roblox.com/game-pass/${id}`).catch((e) => Toast('bad', String(e)))
}
function LoadExtras(universeId: number) {
    api.passes(universeId)
        .then((p) => (passes.value = p))
        .catch(() => (passes.value = []))
    api.badges(universeId)
        .then((b) => (badges.value = b))
        .catch(() => (badges.value = []))
}

async function LoadServers(reset: boolean) {
    if (!game.value) return
    loadingServers.value = true
    try {
        const page = await api.servers(game.value.rootPlaceId, sort.value !== 'fewest', hideFull.value, reset ? null : cursor.value)
        servers.value = reset ? page.servers : [...servers.value, ...page.servers]
        cursor.value = page.nextCursor
    } catch (e) {
        Toast('bad', String(e))
    }
    loadingServers.value = false
}

async function CopyId(id: string) {
    try {
        await navigator.clipboard.writeText(id)
        copied.value = id
        setTimeout(() => (copied.value = null), 1200)
    } catch {
        Toast('bad', "Couldn't copy. Select the ID and copy it by hand.")
    }
}

const PingClass = (p: number | null) => (p == null ? '' : p < 80 ? 'good' : p < 150 ? 'warn' : 'bad')

onMounted(async () => {
    const a = ActiveAccount()
    if (a) picked.value = new Set([a.userId])
    try {
        game.value = await api.game(props.universeId)
        if (store.route.universeId === props.universeId) store.route.name = game.value.name
        LoadServers(true)
        LoadExtras(game.value.universeId)
    } catch (e) {
        error.value = String(e)
    }
})
</script>

<template>
    <p v-if="error" class="err">{{ error }}</p>
    <div v-else-if="!game" class="stack" style="gap: 18px">
        <div class="skeleton" style="height: 300px; border-radius: 18px"></div>
        <div class="skeleton" style="height: 90px; border-radius: 14px"></div>
    </div>
    <div v-else class="game">
        <section class="head">
            <div class="banner">
                <img v-if="game.banners[banner]" :src="game.banners[banner]" alt="" />
                <div class="shade"></div>
                <div v-if="game.banners.length > 1" class="dots">
                    <button
                        v-for="(_, i) in game.banners"
                        :key="i"
                        :class="{ on: i === banner }"
                        :aria-label="'Screenshot ' + (i + 1)"
                        @click="banner = i"
                    ></button>
                </div>
                <div class="title-row">
                    <img v-if="game.icon" :src="game.icon" alt="" class="icon" />
                    <div style="min-width: 0">
                        <h1>{{ game.name }}</h1>
                        <p class="muted">
                            by <b>{{ game.creator }}</b
                            ><template v-if="game.genre"> · {{ game.genre }}</template>
                        </p>
                    </div>
                </div>
            </div>

            <div class="stats">
                <div class="stat">
                    <p class="eyebrow">Playing</p>
                    <p class="val mono">{{ Compact(game.playing) }}</p>
                </div>
                <div class="stat">
                    <p class="eyebrow">Visits</p>
                    <p class="val mono">{{ Compact(game.visits) }}</p>
                </div>
                <div class="stat">
                    <p class="eyebrow">Rating</p>
                    <p class="val mono">
                        <span v-if="pct !== null" class="rating">{{ pct }}%</span><span v-else>No votes</span>
                        <span class="faint small">({{ Compact(votes) }})</span>
                    </p>
                </div>
                <div class="stat">
                    <p class="eyebrow">Server size</p>
                    <p class="val mono">{{ game.maxPlayers }}</p>
                </div>
                <div class="stat">
                    <p class="eyebrow">Updated</p>
                    <p class="val">{{ updated }}</p>
                </div>
                <div class="stat launch">
                    <button
                        class="btn btn-lg"
                        :aria-pressed="fav"
                        :aria-label="fav ? 'Remove from favorites' : 'Add to favorites'"
                        :title="fav ? 'Remove from favorites' : 'Add to favorites'"
                        @click="Star"
                    >
                        <Icon name="star" :size="16" :filled="fav" :class="{ starred: fav }" />
                    </button>
                    <button class="btn btn-primary btn-lg" :disabled="!!launching" @click="Join()">
                        <Icon name="play" :size="16" />{{
                            launching === 'any' ? 'Launching…' : pickedIds.length > 1 ? `Play with ${pickedIds.length}` : 'Play'
                        }}
                    </button>
                </div>
            </div>
        </section>

        <section class="card play-as">
            <p class="eyebrow">Play as</p>
            <div v-if="store.accounts.length" class="chips">
                <button
                    v-for="a in store.accounts"
                    :key="a.userId"
                    class="chip"
                    :aria-pressed="picked.has(a.userId)"
                    @click="Toggle(a.userId)"
                >
                    <img v-if="a.avatar" :src="a.avatar" alt="" /><span v-else class="blank"></span>{{ a.displayName }}
                </button>
            </div>
            <p v-else class="muted small">
                <button class="link inline" @click="Go({ view: 'accounts' })">Add an account</button> to join games and servers from Umbra.
            </p>
            <label v-if="store.profiles" class="instance small"
                ><span class="faint">Instance</span
                ><select id="game-instance" class="select" :value="gameInstance" aria-label="Instance for this game" @change="SetInstance">
                    <option value="">Account default</option>
                    <option v-for="p in store.profiles.profiles" :key="p.id" :value="p.id">{{ p.name }}</option>
                </select></label
            >
        </section>

        <section class="card private">
            <input
                id="private-link"
                v-model="privateLink"
                class="input"
                placeholder="Paste a private server link"
                aria-label="Private server link"
                @keydown.enter="JoinPrivate"
            />
            <button class="btn" :disabled="!!launching || !privateLink.trim()" @click="JoinPrivate">
                <Icon name="play" :size="14" />{{ launching === 'private' ? 'Joining…' : 'Join private server' }}
            </button>
        </section>

        <section v-if="game.description" class="card desc">
            <p :class="{ clamp: !showMore }">{{ game.description }}</p>
            <button class="link" @click="showMore = !showMore">{{ showMore ? 'Show less' : 'Show more' }}</button>
        </section>

        <section v-if="passes?.length" class="stack" style="gap: 12px">
            <h2>Game passes</h2>
            <ul class="passes">
                <li v-for="p in passes" :key="p.id">
                    <button class="pass" :title="'Open ' + p.name + ' on Roblox'" @click="OpenPass(p.id)">
                        <img v-if="p.icon" :src="p.icon" alt="" /><span v-else class="blank"></span>
                        <span class="name">{{ p.name }}</span>
                        <span class="faint small mono">{{ p.price == null ? 'Off sale' : p.price.toLocaleString() + ' R$' }}</span>
                    </button>
                </li>
            </ul>
        </section>

        <section v-if="badges?.length" class="stack" style="gap: 12px">
            <div class="server-bar">
                <h2>Badges</h2>
                <span class="faint small">{{ badges.length }} badges</span>
            </div>
            <ul class="badges">
                <li v-for="b in shownBadges" :key="b.id" class="badge">
                    <img v-if="b.icon" :src="b.icon" alt="" /><span v-else class="blank"></span>
                    <div style="min-width: 0">
                        <p class="name">{{ b.name }}</p>
                        <p class="faint small clamp1">{{ b.description || 'No description' }}</p>
                    </div>
                    <div class="metric small">
                        <span class="mono">{{ (b.winRate * 100).toFixed(1) }}%</span><span class="faint">of players</span>
                    </div>
                    <div class="metric small">
                        <span class="mono">{{ Compact(b.awarded) }}</span
                        ><span class="faint">earned</span>
                    </div>
                </li>
            </ul>
            <div v-if="badges.length > 8" class="row" style="justify-content: center">
                <button class="btn" @click="allBadges = !allBadges">{{ allBadges ? 'Show fewer' : `Show all ${badges.length}` }}</button>
            </div>
        </section>

        <section class="stack" style="gap: 12px">
            <div class="server-bar">
                <h2>Servers</h2>
                <div class="row" style="gap: 8px; flex-wrap: wrap">
                    <select
                        id="server-sort"
                        v-model="sort"
                        class="select"
                        aria-label="Sort servers"
                        @change="sort !== 'ping' && LoadServers(true)"
                    >
                        <option value="most">Most players</option>
                        <option value="fewest">Fewest players</option>
                        <option value="ping">Best ping</option>
                    </select>
                    <label class="check"
                        ><input id="hide-full" v-model="hideFull" type="checkbox" @change="LoadServers(true)" />Hide full</label
                    >
                    <button class="btn" :disabled="loadingServers" @click="LoadServers(true)">
                        <Icon name="refresh" :size="14" />Refresh
                    </button>
                </div>
            </div>

            <p v-if="!loadingServers && !shown.length" class="muted">No open public servers right now.</p>
            <ul class="servers">
                <li v-for="s in shown" :key="s.id" class="server">
                    <div class="players">
                        <div class="row between small">
                            <span
                                ><b class="mono">{{ s.playing }}</b
                                ><span class="faint"> / {{ s.maxPlayers }} players</span></span
                            >
                        </div>
                        <div class="pbar">
                            <div :style="{ width: Math.min(100, (s.playing / Math.max(1, s.maxPlayers)) * 100) + '%' }"></div>
                        </div>
                    </div>
                    <div class="metric">
                        <Icon name="signal" :size="14" /><span class="mono" :class="PingClass(s.ping)">{{ s.ping ?? '?' }}</span
                        ><span class="faint">ms</span>
                    </div>
                    <div class="metric">
                        <Icon name="gauge" :size="14" /><span class="mono">{{ s.fps ? Math.round(s.fps) : '?' }}</span
                        ><span class="faint">fps</span>
                    </div>
                    <button class="sid mono" :title="'Copy server ID ' + s.id" @click="CopyId(s.id)">
                        {{ copied === s.id ? 'Copied' : s.id.slice(0, 8) }}<Icon name="copy" :size="12" />
                    </button>
                    <button class="btn btn-join" :disabled="!!launching" @click="Join(s.id)">
                        <Icon name="play" :size="14" />{{ launching === s.id ? 'Joining…' : 'Join' }}
                    </button>
                </li>
            </ul>
            <div class="row" style="justify-content: center">
                <button v-if="cursor" class="btn" :disabled="loadingServers" @click="LoadServers(false)">
                    {{ loadingServers ? 'Loading…' : 'Load more servers' }}
                </button>
                <span v-else-if="loadingServers" class="faint small">Loading servers…</span>
            </div>
        </section>
    </div>
</template>

<style scoped>
.starred {
    color: var(--warn);
}
.stat.launch {
    display: flex;
    gap: 8px;
}
.game {
    display: flex;
    flex-direction: column;
    gap: 16px;
}
.err {
    color: var(--bad);
    user-select: text;
}
.small {
    font-size: 12px;
}
.head {
    border-radius: 18px;
    overflow: hidden;
    border: 1px solid var(--line);
    background: var(--surface);
}
.banner {
    position: relative;
    aspect-ratio: 16 / 4.4;
    min-height: 200px;
    background: var(--surface-2);
}
.banner > img {
    position: absolute;
    inset: 0;
    width: 100%;
    height: 100%;
    object-fit: cover;
}
.shade {
    position: absolute;
    inset: 0;
    background: linear-gradient(180deg, rgba(0, 0, 0, 0.1) 30%, rgba(16, 16, 17, 0.96) 100%);
}
.dots {
    position: absolute;
    top: 14px;
    right: 14px;
    display: flex;
    gap: 6px;
}
.dots button {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: rgba(255, 255, 255, 0.35);
}
.dots button.on {
    background: #fff;
    width: 20px;
    border-radius: 999px;
}
.title-row {
    position: absolute;
    left: 22px;
    right: 22px;
    bottom: 18px;
    display: flex;
    align-items: flex-end;
    gap: 16px;
}
.icon {
    width: 84px;
    height: 84px;
    border-radius: 18px;
    border: 2px solid rgba(255, 255, 255, 0.08);
    box-shadow: 0 10px 30px rgba(0, 0, 0, 0.5);
    flex: none;
}
.title-row h1 {
    font-size: 26px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
}
.title-row b {
    color: var(--fg);
}
.stats {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    border-top: 1px solid var(--line);
}
.stat {
    padding: 14px 22px;
    border-right: 1px solid var(--line);
    display: flex;
    flex-direction: column;
    gap: 2px;
}
.stat .val {
    font-size: 17px;
    font-weight: 700;
}
.rating {
    color: var(--good);
    font-style: italic;
}
.stat.launch {
    margin-left: auto;
    border-right: 0;
}

.play-as {
    display: flex;
    align-items: center;
    gap: 14px;
    padding: 12px 16px;
    flex-wrap: wrap;
}
.chips {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
}
.chip {
    display: inline-flex;
    align-items: center;
    gap: 8px;
    height: 32px;
    padding: 0 12px 0 4px;
    border-radius: 999px;
    background: var(--surface-2);
    border: 1px solid var(--line);
    font-size: 13px;
    font-weight: 700;
    color: var(--fg-2);
}
.chip img,
.chip .blank {
    width: 24px;
    height: 24px;
    border-radius: 50%;
    background: var(--surface-3);
}
.chip[aria-pressed='true'] {
    background: #ffffff;
    border-color: transparent;
    color: #000000;
}
.link.inline {
    font-size: 12px;
    color: var(--accent);
}

.instance {
    display: inline-flex;
    align-items: center;
    gap: 8px;
    margin-left: auto;
}
.private {
    display: flex;
    gap: 10px;
    padding: 12px 16px;
}
.private .input {
    flex: 1;
    min-width: 0;
}

.passes {
    list-style: none;
    margin: 0;
    padding: 0;
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(150px, 1fr));
    gap: 10px;
}
.pass {
    display: flex;
    flex-direction: column;
    gap: 6px;
    padding: 12px;
    border-radius: 14px;
    background: var(--surface);
    border: 1px solid var(--line);
    color: inherit;
    text-align: left;
    width: 100%;
    height: 100%;
}
.pass:hover {
    border-color: var(--line-strong);
}
.pass img,
.pass .blank {
    width: 100%;
    aspect-ratio: 1;
    border-radius: 10px;
    background: var(--surface-2);
    object-fit: cover;
}
.pass .name,
.badge .name {
    font-size: 13px;
    font-weight: 700;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
}
.badges {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 8px;
}
.badge {
    display: grid;
    grid-template-columns: 44px minmax(0, 1fr) 110px 90px;
    align-items: center;
    gap: 14px;
    padding: 10px 14px;
    border-radius: 14px;
    background: var(--surface);
    border: 1px solid var(--line);
}
.badge img,
.badge .blank {
    width: 44px;
    height: 44px;
    border-radius: 50%;
    background: var(--surface-2);
}
.clamp1 {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
}

.desc {
    display: flex;
    flex-direction: column;
    gap: 8px;
    align-items: flex-start;
}
.desc p {
    white-space: pre-line;
    color: var(--fg-2);
    font-size: 13px;
    max-width: 90ch;
    user-select: text;
}
.desc .clamp {
    display: -webkit-box;
    -webkit-line-clamp: 3;
    -webkit-box-orient: vertical;
    overflow: hidden;
}

.server-bar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    flex-wrap: wrap;
}
.servers {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 8px;
}
.server {
    display: grid;
    grid-template-columns: minmax(160px, 1fr) 90px 90px 110px auto;
    align-items: center;
    gap: 16px;
    padding: 12px 14px 12px 18px;
    border-radius: 14px;
    background: var(--surface);
    border: 1px solid var(--line);
}
.server:hover {
    border-color: var(--line-strong);
}
.players {
    display: flex;
    flex-direction: column;
    gap: 6px;
    min-width: 0;
}
.pbar {
    height: 5px;
    border-radius: 999px;
    background: var(--bg-2);
    overflow: hidden;
}
.pbar div {
    height: 100%;
    background: var(--accent);
    border-radius: 999px;
}
.metric {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    font-size: 13px;
    color: var(--fg-3);
}
.metric .mono {
    color: var(--fg);
    font-weight: 600;
}
.metric .good {
    color: var(--good);
}
.metric .warn {
    color: var(--warn);
}
.metric .bad {
    color: var(--bad);
}
.sid {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    font-size: 12px;
    color: var(--fg-3);
    padding: 4px 8px;
    border-radius: 6px;
    justify-self: start;
}
.sid:hover {
    background: var(--surface-2);
    color: var(--fg);
}
@media (max-width: 900px) {
    .server {
        grid-template-columns: 1fr auto auto;
    }
    .metric:nth-of-type(2),
    .sid {
        display: none;
    }
}
</style>
