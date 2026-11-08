<script setup lang="ts">
import { ref } from 'vue'
import { Duration } from '@/utils/format'
import { Umbra } from '@/utils/messages'
import { Note, RefreshUmbra, umbra } from '@/utils/umbra-store'

const busy = ref<number | null>(null)
const confirm = ref<number | null>(null)

const Who = (id: number | null) => umbra.state?.accounts.find((a) => a.userId === id)?.displayName ?? 'Default login'

async function Act(pid: number, action: 'rejoin' | 'hop' | 'close') {
    busy.value = pid
    confirm.value = null
    try {
        await Umbra.client(pid, action)
        Note(
            'good',
            action === 'rejoin' ? 'Rejoining the same server.' : action === 'hop' ? 'Hopping to another server.' : 'Roblox closed.',
        )
    } catch (e) {
        Note('bad', e instanceof Error ? e.message : String(e))
    }
    busy.value = null
    setTimeout(RefreshUmbra, 1500)
}
</script>

<template>
    <div class="card stack">
        <div class="row between">
            <span class="label">Running in Umbra</span><span class="faint small">{{ umbra.state?.running.length ?? 0 }}</span>
        </div>
        <p v-if="!umbra.state?.running.length" class="muted small">No Roblox clients are running from Umbra.</p>
        <div v-for="c in umbra.state?.running ?? []" :key="c.pid" class="client">
            <img v-if="c.game?.gameIcon" :src="c.game.gameIcon" alt="" class="cicon" /><span v-else class="cicon"></span>
            <div class="ctext">
                <b class="small">{{ c.game?.gameName ?? c.label }}</b>
                <span class="faint small">{{ Who(c.userId) }} · {{ Duration(c.secs) }}</span>
            </div>
            <div class="row">
                <template v-if="confirm === c.pid">
                    <button class="btn btn-sm danger" @click="confirm = null">Cancel</button>
                    <button class="btn btn-sm" @click="Act(c.pid, 'close')">Close</button>
                </template>
                <template v-else>
                    <button
                        class="btn btn-sm"
                        :disabled="busy === c.pid || !c.userId"
                        title="Rejoin the same server"
                        @click="Act(c.pid, 'rejoin')"
                    >
                        Rejoin
                    </button>
                    <button
                        class="btn btn-sm"
                        :disabled="busy === c.pid || !c.userId"
                        title="Hop to another server"
                        @click="Act(c.pid, 'hop')"
                    >
                        Hop
                    </button>
                    <button class="btn btn-ghost btn-sm" :disabled="busy === c.pid" @click="confirm = c.pid">Close…</button>
                </template>
            </div>
        </div>
    </div>
</template>

<style scoped>
.client {
    display: grid;
    grid-template-columns: 34px minmax(0, 1fr) auto;
    align-items: center;
    gap: 10px;
    padding: 8px;
    border-radius: 10px;
    background: var(--raised);
}
.cicon {
    width: 34px;
    height: 34px;
    border-radius: 8px;
    background: #222;
}
.ctext {
    display: flex;
    flex-direction: column;
    min-width: 0;
}
.ctext b {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
}
.danger {
    background: rgba(255, 98, 131, 0.14) !important;
    border-color: rgba(255, 98, 131, 0.45) !important;
    color: #ffc2cf !important;
}
</style>
