<script setup lang="ts">
import { Connect, TogglePick, umbra } from '@/utils/umbra-store'
</script>

<template>
    <div class="card stack">
        <div class="row between">
            <span class="label">Play as</span>
            <span v-if="umbra.state" class="pill good"><span class="dot"></span>Umbra</span>
            <span v-else-if="umbra.hello?.reachable" class="pill warn"><span class="dot"></span>Not connected</span>
            <span v-else class="pill"><span class="dot"></span>Roblox launcher</span>
        </div>

        <template v-if="umbra.state">
            <div v-if="umbra.state.accounts.length" class="row wrap">
                <button
                    v-for="a in umbra.state.accounts"
                    :key="a.userId"
                    class="chip"
                    :aria-pressed="umbra.picked.includes(a.userId)"
                    @click="TogglePick(a.userId)"
                >
                    <img v-if="a.avatar" :src="a.avatar" alt="" /><span v-else class="blank"></span>{{ a.displayName }}
                </button>
            </div>
            <p v-else class="muted small">Add an account in Umbra to launch from here.</p>
            <label class="row between small">
                <span class="faint">Instance</span>
                <select v-model="umbra.profileId" class="select" aria-label="Umbra instance">
                    <option value="">Account or game default</option>
                    <option v-for="p in umbra.state.profiles.profiles" :key="p.id" :value="p.id">{{ p.name }}</option>
                </select>
            </label>
        </template>
        <template v-else-if="umbra.hello?.reachable && umbra.hello.enabled">
            <p class="muted small">Connect to launch with any of your Umbra accounts, pick an instance and control running clients.</p>
            <button class="btn btn-primary btn-wide" :disabled="umbra.busy" @click="Connect">
                {{ umbra.busy ? 'Waiting for Umbra…' : 'Connect to Umbra' }}
            </button>
        </template>
        <p v-else-if="umbra.hello?.reachable" class="muted small">
            The extension connection is off. Turn it on in Umbra under Settings, Browser extension.
        </p>
        <template v-else>
            <p class="muted small">Open the Umbra app to play with your alts. Joins use the normal Roblox launcher until then.</p>
            <p v-if="umbra.error" class="faint small">{{ umbra.error }}</p>
        </template>
    </div>
</template>
