<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { DEFAULT_FEATURES, features, LoadFeatures, type Features } from '@/utils/settings'

const ROWS: { key: keyof Features; title: string; text: string }[] = [
    { key: 'drawer', title: 'Umbra drawer', text: 'The Umbra tab on the right of roblox.com with tools for the page you are on.' },
    { key: 'servers', title: 'Server browser', text: 'Sort servers by players, ping or fps and join any of them, with any account.' },
    { key: 'values', title: 'Rolimons values', text: 'Item values, RAP, demand and trend, plus player values on profiles.' },
    { key: 'search', title: 'Quick search', text: 'Ctrl K anywhere on Roblox to find games, players and pages.' },
    { key: 'hideAds', title: 'Hide ads and sponsored items', text: 'Hides ad slots and sponsored catalog rows.' },
    { key: 'autoOpen', title: 'Open the drawer on game pages', text: 'Shows the server browser and Play as right away.' },
]

const f = ref<Features>(DEFAULT_FEATURES)
async function Flip(key: keyof Features) {
    f.value = { ...f.value, [key]: !f.value[key] }
    await features.setValue(f.value)
}
onMounted(async () => (f.value = await LoadFeatures()))
</script>

<template>
    <div>
        <div v-for="r in ROWS" :key="r.key" class="toggle">
            <div>
                <h3>{{ r.title }}</h3>
                <p>{{ r.text }}</p>
            </div>
            <button class="switch" role="switch" :aria-checked="f[r.key]" :aria-label="r.title" @click="Flip(r.key)"></button>
        </div>
    </div>
</template>
