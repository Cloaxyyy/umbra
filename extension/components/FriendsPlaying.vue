<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { FriendsInGame } from '@/utils/roblox'
import { Connected, Join, umbra } from '@/utils/umbra-store'

type Row = Awaited<ReturnType<typeof FriendsInGame>>[number]
const rows = ref<Row[] | null>(null)
const error = ref('')
const loading = ref(false)

async function Load() {
    loading.value = true
    error.value = ''
    try {
        rows.value = await FriendsInGame()
    } catch (e) {
        error.value = e instanceof Error ? e.message : String(e)
        rows.value = []
    }
    loading.value = false
}

onMounted(Load)
</script>

<template>
    <div class="card stack">
        <div class="row between">
            <span class="label">Friends playing</span>
            <button class="btn btn-ghost btn-sm" :disabled="loading" @click="Load">{{ loading ? 'Loading…' : 'Refresh' }}</button>
        </div>
        <p v-if="error" class="muted small">{{ error }}</p>
        <p v-else-if="rows && !rows.length" class="muted small">None of your friends are in a game right now.</p>
        <p v-else-if="!rows" class="faint small">Loading friends…</p>
        <div v-for="f in rows ?? []" :key="f.id" class="friend rise">
            <img v-if="f.avatar" :src="f.avatar" alt="" class="av" /><span v-else class="av"></span>
            <a class="ftext" :href="`https://www.roblox.com/users/${f.id}/profile`">
                <b class="small">{{ f.name }}</b>
                <span class="faint small">{{ f.game }}</span>
            </a>
            <button
                class="btn btn-sm"
                :disabled="umbra.busy || !f.placeId"
                :title="f.placeId ? 'Join their server' : 'Their game is hidden'"
                @click="f.placeId && Join(f.placeId, f.jobId)"
            >
                {{ Connected() && umbra.picked.length > 1 ? `Join with ${umbra.picked.length}` : 'Join' }}
            </button>
        </div>
    </div>
</template>

<style scoped>
.friend {
    display: grid;
    grid-template-columns: 34px minmax(0, 1fr) auto;
    align-items: center;
    gap: 10px;
    padding: 6px;
    border-radius: 10px;
}
.friend:hover {
    background: var(--raised);
}
.av {
    width: 34px;
    height: 34px;
    border-radius: 50%;
    background: var(--raised);
}
.ftext {
    display: flex;
    flex-direction: column;
    min-width: 0;
    color: var(--fg);
    text-decoration: none;
}
.ftext span,
.ftext b {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
}
</style>
