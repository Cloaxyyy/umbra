<script setup lang="ts">
import { onMounted } from 'vue'
import { Umbra } from '@/utils/messages'
import { Connect, Note, RefreshUmbra, umbra } from '@/utils/umbra-store'

async function Disconnect() {
    try {
        await Umbra.unpair()
        await RefreshUmbra()
        Note('good', 'Disconnected from Umbra.')
    } catch (e) {
        Note('bad', e instanceof Error ? e.message : String(e))
    }
}

onMounted(() => RefreshUmbra().catch(() => {}))
</script>

<template>
    <div class="stack">
        <div class="row between">
            <div>
                <p class="title">Umbra desktop app</p>
                <p class="muted small">
                    <template v-if="umbra.state">Connected to Umbra {{ umbra.hello?.version }}.</template>
                    <template v-else-if="umbra.hello?.reachable && !umbra.hello.enabled"
                        >Umbra is running, but the extension connection is off in its settings.</template
                    >
                    <template v-else-if="umbra.hello?.reachable">Umbra is running. Connect to use your accounts here.</template>
                    <template v-else>Umbra isn't running on this PC.</template>
                </p>
            </div>
            <span v-if="umbra.state" class="pill good"><span class="dot"></span>Connected</span>
            <span v-else-if="umbra.hello?.reachable" class="pill warn"><span class="dot"></span>Not connected</span>
            <span v-else class="pill"><span class="dot"></span>Offline</span>
        </div>
        <p v-if="umbra.note" class="note" :class="umbra.note.kind" role="status">{{ umbra.note.text }}</p>
        <div class="row">
            <button
                v-if="!umbra.state"
                class="btn btn-primary"
                :disabled="umbra.busy || !umbra.hello?.reachable || !umbra.hello.enabled"
                @click="Connect"
            >
                {{ umbra.busy ? 'Waiting for Allow in Umbra…' : 'Connect' }}
            </button>
            <button v-else class="btn" @click="Disconnect">Disconnect</button>
            <button class="btn btn-ghost" @click="RefreshUmbra">Check again</button>
        </div>
        <p class="faint small">
            Umbra only answers this extension, only on this PC, and only after you press Allow in the app. It never shares your Roblox
            sessions with the extension.
        </p>
    </div>
</template>
