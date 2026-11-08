<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue'
import { Compact, Duration, LikePct, Since } from '@/utils/format'
import { Umbra } from '@/utils/messages'
import { Game, UniverseOf, type GameInfo } from '@/utils/roblox'
import { Connected, Join, Note, RefreshUmbra, umbra } from '@/utils/umbra-store'
import PlayAs from './PlayAs.vue'
import ServerList from './ServerList.vue'

const props = defineProps<{ placeId: number; servers: boolean }>()
const game = ref<GameInfo | null>(null)
const error = ref('')
const copied = ref('')

const privateLink = computed(() => /privateServerLinkCode=/.test(location.href))
const pct = computed(() => (game.value ? LikePct(game.value.upVotes, game.value.downVotes) : null))
const fav = computed(() => !!game.value && !!umbra.state?.favorites.includes(game.value.universeId))
const played = ref<{ secs: number; sessions: number; last: number } | null>(null)

async function LoadPlaytime(universeId: number) {
    if (!Connected()) return
    try {
        played.value = await Umbra.playtime(universeId)
    } catch {
        played.value = null
    }
}

async function Copy(what: string, text: string) {
    try {
        await navigator.clipboard.writeText(text)
        copied.value = what
        setTimeout(() => (copied.value = ''), 1400)
    } catch {
        Note('bad', "Couldn't copy that.")
    }
}

async function Star() {
    const g = game.value
    if (!g) return
    try {
        const r = await Umbra.favorite(g.universeId, g.rootPlaceId, g.name, g.icon)
        Note('good', r.added ? 'Added to your Umbra favorites.' : 'Removed from your Umbra favorites.')
        RefreshUmbra()
    } catch (e) {
        Note('bad', e instanceof Error ? e.message : String(e))
    }
}

function Play() {
    Join(props.placeId, null, privateLink.value ? location.href : undefined)
}

watch(
    () => [game.value?.universeId, !!umbra.state] as const,
    ([id, ok]) => {
        if (id && ok && !played.value) LoadPlaytime(id)
    },
)

onMounted(async () => {
    try {
        game.value = await Game(await UniverseOf(props.placeId))
    } catch (e) {
        error.value = e instanceof Error ? e.message : String(e)
    }
})
</script>

<template>
    <div class="stack">
        <p v-if="error" class="note bad">{{ error }}</p>
        <div v-else-if="game" class="card stack rise">
            <div class="row">
                <img v-if="game.icon" :src="game.icon" alt="" class="gicon" />
                <div style="min-width: 0">
                    <p class="title ellipsis">{{ game.name }}</p>
                    <p class="faint small ellipsis">
                        by {{ game.creator }}<template v-if="game.genre"> · {{ game.genre }}</template>
                    </p>
                </div>
            </div>
            <div class="stats">
                <div class="stat">
                    <b>{{ Compact(game.playing) }}</b
                    ><span>Playing</span>
                </div>
                <div class="stat">
                    <b>{{ Compact(game.visits) }}</b
                    ><span>Visits</span>
                </div>
                <div class="stat">
                    <b>{{ pct === null ? '?' : pct + '%' }}</b
                    ><span>Likes</span>
                </div>
                <div class="stat">
                    <b>{{ Compact(game.favorites) }}</b
                    ><span>Favorites</span>
                </div>
                <div class="stat">
                    <b>{{ game.maxPlayers }}</b
                    ><span>Per server</span>
                </div>
                <div class="stat">
                    <b class="small">{{ Since(game.updated) }}</b
                    ><span>Updated</span>
                </div>
            </div>
            <p class="faint small">Created {{ Since(game.created) }} · Universe {{ game.universeId }}</p>
            <p v-if="played?.sessions" class="note good small">
                You've played {{ Duration(played.secs) }} over {{ played.sessions }} session{{ played.sessions === 1 ? '' : 's' }}, last
                {{ Since(played.last) }}.
            </p>
        </div>
        <div v-else class="card faint small">Loading game…</div>

        <PlayAs />

        <div class="card stack">
            <p v-if="privateLink" class="note good">This is a private server link. Play joins that server.</p>
            <button class="btn btn-primary btn-wide play" :disabled="umbra.busy" @click="Play">
                {{
                    umbra.busy
                        ? 'Launching…'
                        : Connected() && umbra.picked.length > 1
                          ? `Play with ${umbra.picked.length} accounts`
                          : privateLink
                            ? 'Join private server'
                            : 'Play'
                }}
            </button>
            <div class="row wrap">
                <button v-if="Connected() && game" class="btn btn-sm" @click="Star">
                    {{ fav ? 'In Umbra favorites' : 'Add to Umbra favorites' }}
                </button>
                <button class="btn btn-ghost btn-sm" @click="Copy('place', String(placeId))">
                    {{ copied === 'place' ? 'Copied' : 'Copy place ID' }}
                </button>
                <button v-if="game" class="btn btn-ghost btn-sm" @click="Copy('universe', String(game.universeId))">
                    {{ copied === 'universe' ? 'Copied' : 'Copy universe ID' }}
                </button>
            </div>
        </div>

        <ServerList v-if="servers" :place-id="placeId" :max-players="game?.maxPlayers ?? 0" />
    </div>
</template>

<style scoped>
.gicon {
    width: 48px;
    height: 48px;
    border-radius: 12px;
    flex: none;
}
.ellipsis {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
}
.play {
    height: 40px;
    font-size: 14px;
}
</style>
