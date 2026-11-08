<script setup lang="ts">
import { onMounted } from 'vue'
import Clients from '@/components/Clients.vue'
import Connection from '@/components/Connection.vue'
import ThemeGrid from '@/components/ThemeGrid.vue'
import { umbra } from '@/utils/umbra-store'

function Options() {
    browser.runtime.openOptionsPage()
    window.close()
}

onMounted(() => document.body.classList.add('u'))
</script>

<template>
    <main class="u pop stack">
        <header class="row between">
            <span class="row brand">
                <img src="/icon/32.png" alt="" width="22" height="22" />
                <b>Umbra</b>
            </span>
            <button class="btn btn-ghost btn-sm" @click="Options">Settings</button>
        </header>
        <section class="card"><Connection /></section>
        <Clients v-if="umbra.state" />
        <section class="card stack">
            <div class="row between">
                <span class="label">Roblox theme</span>
                <button class="btn btn-ghost btn-sm" @click="Options">Customize</button>
            </div>
            <ThemeGrid />
        </section>
    </main>
</template>

<style scoped>
.pop {
    width: 380px;
    padding: 12px;
    gap: 10px;
}
.brand b {
    font-size: 15px;
}
</style>
