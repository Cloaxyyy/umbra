<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { Age, Compact, Since } from '@/utils/format'
import { Rolimons } from '@/utils/messages'
import { FriendIds, Headshots, Me, Names, PresenceOf, User, type Presence, type UserInfo } from '@/utils/roblox'
import type { PlayerValue } from '@/utils/types'
import { Connected, FollowJoin, Join, Note, umbra } from '@/utils/umbra-store'
import PlayAs from './PlayAs.vue'

const props = defineProps<{ userId: number; values: boolean }>()
const user = ref<UserInfo | null>(null)
const presence = ref<Presence | null>(null)
const value = ref<PlayerValue | null | undefined>(undefined)
const mutuals = ref<{ id: number; name: string; avatar: string | null }[] | null>(null)
const mutualCount = ref(0)
const avatar = ref<string | null>(null)
const error = ref('')
const copied = ref(false)

const status = computed(() => {
    const p = presence.value
    if (!p) return null
    return (
        [
            { cls: '', text: 'Offline' },
            { cls: 'good', text: 'Online' },
            { cls: 'good', text: p.lastLocation ? `In ${p.lastLocation}` : 'In game' },
            { cls: 'warn', text: 'In Studio' },
        ][p.userPresenceType] ?? { cls: '', text: 'Unknown' }
    )
})
const joinable = computed(() => presence.value?.userPresenceType === 2 && !!presence.value.placeId)

async function JoinThem() {
    const p = presence.value
    if (!p?.placeId) return
    if (Connected() && umbra.picked.length) {
        if (p.gameId) await Join(p.placeId, p.gameId)
        else if (user.value) await FollowJoin(user.value.name)
        return
    }
    Join(p.placeId, p.gameId)
}

async function CopyId() {
    try {
        await navigator.clipboard.writeText(String(props.userId))
        copied.value = true
        setTimeout(() => (copied.value = false), 1400)
    } catch {
        Note('bad', "Couldn't copy that.")
    }
}

async function LoadMutuals() {
    const me = await Me()
    if (!me || me === props.userId) {
        mutuals.value = []
        return
    }
    const [mine, theirs] = await Promise.all([FriendIds(me), FriendIds(props.userId)])
    const set = new Set(mine)
    const both = theirs.filter((id) => set.has(id))
    mutualCount.value = both.length
    const top = both.slice(0, 12)
    const [names, heads] = await Promise.all([Names(top), Headshots(top)])
    mutuals.value = top.map((id) => ({ id, name: names[id]?.displayName ?? `User ${id}`, avatar: heads[id] ?? null }))
}

onMounted(async () => {
    try {
        user.value = await User(props.userId)
    } catch (e) {
        error.value = e instanceof Error ? e.message : String(e)
        return
    }
    Headshots([props.userId])
        .then((h) => (avatar.value = h[props.userId] ?? null))
        .catch(() => {})
    PresenceOf([props.userId])
        .then((p) => (presence.value = p[0] ?? null))
        .catch(() => {})
    if (props.values)
        Rolimons.player(props.userId)
            .then((v) => (value.value = v))
            .catch(() => (value.value = null))
    LoadMutuals().catch(() => (mutuals.value = []))
})
</script>

<template>
    <div class="stack">
        <p v-if="error" class="note bad">{{ error }}</p>
        <div v-else-if="user" class="card stack rise">
            <div class="row">
                <img v-if="avatar" :src="avatar" alt="" class="head" /><span v-else class="head"></span>
                <div style="min-width: 0">
                    <p class="title">{{ user.displayName }}</p>
                    <p class="faint small">@{{ user.name }}</p>
                </div>
                <span v-if="status" class="pill" :class="status.cls" style="margin-left: auto"
                    ><span class="dot"></span>{{ status.text }}</span
                >
            </div>
            <div class="stats">
                <div class="stat">
                    <b>{{ Age(user.created) }}</b
                    ><span>Account age</span>
                </div>
                <div class="stat">
                    <b>{{ value ? Compact(value.value) : value === null ? 'None' : '…' }}</b
                    ><span>Rolimons value</span>
                </div>
                <div class="stat">
                    <b>{{ value ? Compact(value.rap) : value === null ? 'None' : '…' }}</b
                    ><span>RAP</span>
                </div>
            </div>
            <p class="faint small">
                Joined {{ new Date(user.created).toLocaleDateString()
                }}<template v-if="value?.rank"> · Value rank #{{ value.rank.toLocaleString() }}</template
                ><template v-if="value?.lastOnline"> · Seen {{ Since(value.lastOnline) }}</template>
            </p>
            <div class="row wrap">
                <button v-if="joinable" class="btn btn-primary" :disabled="umbra.busy" @click="JoinThem">
                    {{ Connected() && umbra.picked.length > 1 ? `Join with ${umbra.picked.length}` : 'Join their game' }}
                </button>
                <a class="btn btn-sm" :href="`https://www.rolimons.com/player/${userId}`" target="_blank" rel="noopener noreferrer"
                    >Rolimons</a
                >
                <button class="btn btn-ghost btn-sm" @click="CopyId">{{ copied ? 'Copied' : 'Copy user ID' }}</button>
            </div>
        </div>
        <div v-else class="card faint small">Loading player…</div>

        <PlayAs v-if="joinable" />

        <div v-if="mutuals && mutuals.length" class="card stack">
            <div class="row between">
                <span class="label">Mutual friends</span><span class="faint small">{{ mutualCount }}</span>
            </div>
            <div class="mutuals">
                <a v-for="m in mutuals" :key="m.id" :href="`https://www.roblox.com/users/${m.id}/profile`" class="mutual" :title="m.name">
                    <img v-if="m.avatar" :src="m.avatar" alt="" /><span v-else class="blank"></span>
                    <span class="small ellipsis">{{ m.name }}</span>
                </a>
            </div>
        </div>
    </div>
</template>

<style scoped>
.head {
    width: 48px;
    height: 48px;
    border-radius: 50%;
    background: var(--raised);
    flex: none;
}
.mutuals {
    display: grid;
    grid-template-columns: repeat(4, 1fr);
    gap: 8px;
}
.mutual {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 4px;
    color: var(--fg-2);
    text-decoration: none;
    min-width: 0;
}
.mutual img,
.mutual .blank {
    width: 44px;
    height: 44px;
    border-radius: 50%;
    background: var(--raised);
}
.ellipsis {
    max-width: 100%;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
}
a.btn {
    color: var(--fg);
    text-decoration: none;
}
</style>
