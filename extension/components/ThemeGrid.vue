<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { activeTheme, customThemes } from '@/utils/settings'
import { NO_THEME, PRESETS, Sanitize, type Theme } from '@/utils/themes'

const props = defineProps<{ manage?: boolean }>()
const emit = defineEmits<{ edit: [theme: Theme]; duplicate: [theme: Theme]; remove: [theme: Theme] }>()

const active = ref(NO_THEME)
const custom = ref<Theme[]>([])
const all = computed(() => [...PRESETS, ...custom.value])

async function Load() {
    active.value = await activeTheme.getValue()
    custom.value = (await customThemes.getValue()).map((t) => Sanitize(t, t.id))
}
async function Use(id: string) {
    active.value = id
    await activeTheme.setValue(id)
}
const IsCustom = (t: Theme) => custom.value.some((c) => c.id === t.id)

onMounted(() => {
    Load()
    activeTheme.watch(Load)
    customThemes.watch(Load)
})
</script>

<template>
    <div class="grid">
        <button class="theme" :aria-pressed="active === NO_THEME" @click="Use(NO_THEME)">
            <span class="swatch roblox"><span></span><span></span><span></span></span>
            <span class="name">Roblox default</span>
        </button>
        <div v-for="t in all" :key="t.id" class="cell">
            <button class="theme" :aria-pressed="active === t.id" @click="Use(t.id)">
                <span
                    class="swatch"
                    :style="{
                        background: t.image ? `${t.bg} url('${t.image}') center / cover` : t.bg,
                        borderRadius: Math.min(t.radius, 14) + 'px',
                    }"
                >
                    <span :style="{ background: t.surface, borderRadius: Math.min(t.radius, 10) / 2 + 'px' }"></span>
                    <span :style="{ background: t.text }"></span>
                    <span :style="{ background: t.accent }"></span>
                </span>
                <span class="name">{{ t.name }}</span>
            </button>
            <div v-if="props.manage" class="tools">
                <button v-if="IsCustom(t)" class="btn btn-ghost btn-sm" @click="emit('edit', t)">Edit</button>
                <button class="btn btn-ghost btn-sm" @click="emit('duplicate', t)">Copy</button>
                <button v-if="IsCustom(t)" class="btn btn-ghost btn-sm" @click="emit('remove', t)">Delete</button>
            </div>
        </div>
    </div>
</template>

<style scoped>
.grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(104px, 1fr));
    gap: 10px;
}
.cell {
    display: flex;
    flex-direction: column;
    gap: 4px;
}
.theme {
    display: flex;
    flex-direction: column;
    gap: 6px;
    padding: 6px;
    border-radius: 12px;
    border: 1px solid var(--line) !important;
    text-align: left;
    transition: border-color 0.15s;
}
.theme:hover {
    border-color: var(--line-2) !important;
}
.theme[aria-pressed='true'] {
    border-color: #ffffff !important;
}
.swatch {
    position: relative;
    height: 56px;
    padding: 8px;
    display: grid;
    grid-template-columns: 1fr auto;
    grid-template-rows: 1fr auto;
    gap: 4px;
    overflow: hidden;
    border: 1px solid rgba(255, 255, 255, 0.08);
}
.swatch span:nth-child(1) {
    grid-row: 1 / 3;
}
.swatch span:nth-child(2) {
    width: 18px;
    height: 4px;
    border-radius: 2px;
    align-self: start;
}
.swatch span:nth-child(3) {
    width: 18px;
    height: 10px;
    border-radius: 4px;
}
.swatch.roblox {
    background: #f2f4f5;
    border-radius: 8px;
}
.swatch.roblox span:nth-child(1) {
    background: #ffffff;
}
.swatch.roblox span:nth-child(2) {
    background: #393b3d;
}
.swatch.roblox span:nth-child(3) {
    background: #335fff;
}
.name {
    font-size: 12px;
    font-weight: 600;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
}
.tools {
    display: flex;
    gap: 2px;
    justify-content: center;
}
</style>
