<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { DEMAND, TREND } from '@/utils/format'
import { Rolimons } from '@/utils/messages'
import type { ItemValue } from '@/utils/types'
import { Note } from '@/utils/umbra-store'

const props = defineProps<{ id: number; itemType: 'asset' | 'bundle'; values: boolean }>()
const item = ref<ItemValue | null | undefined>(undefined)
const error = ref('')
const copied = ref(false)

async function CopyId() {
    try {
        await navigator.clipboard.writeText(String(props.id))
        copied.value = true
        setTimeout(() => (copied.value = false), 1400)
    } catch {
        Note('bad', "Couldn't copy that.")
    }
}

onMounted(async () => {
    if (!props.values || props.itemType !== 'asset') {
        item.value = null
        return
    }
    try {
        item.value = await Rolimons.item(props.id)
    } catch (e) {
        error.value = e instanceof Error ? e.message : String(e)
    }
})
</script>

<template>
    <div class="stack">
        <p v-if="error" class="note bad">{{ error }}</p>
        <div v-else-if="item" class="card stack rise">
            <div class="row between">
                <span class="label">Rolimons</span>
                <span class="row">
                    <span v-if="item.projected" class="pill warn"><span class="dot"></span>Projected</span>
                    <span v-if="item.hyped" class="pill good"><span class="dot"></span>Hyped</span>
                    <span v-if="item.rare" class="pill"><span class="dot"></span>Rare</span>
                </span>
            </div>
            <p class="title">
                {{ item.name }}<span v-if="item.acronym" class="faint"> · {{ item.acronym }}</span>
            </p>
            <div class="stats">
                <div class="stat">
                    <b>{{ (item.value ?? item.rap).toLocaleString() }}</b
                    ><span>{{ item.value ? 'Value' : 'Value (RAP)' }}</span>
                </div>
                <div class="stat">
                    <b>{{ item.rap.toLocaleString() }}</b
                    ><span>RAP</span>
                </div>
                <div class="stat">
                    <b class="small">{{ DEMAND[item.demand] ?? 'None' }}</b
                    ><span>Demand</span>
                </div>
                <div class="stat">
                    <b class="small">{{ TREND[item.trend] ?? 'None' }}</b
                    ><span>Trend</span>
                </div>
            </div>
            <a class="btn btn-sm" :href="`https://www.rolimons.com/item/${id}`" target="_blank" rel="noopener noreferrer"
                >Open on Rolimons</a
            >
        </div>
        <div v-else-if="item === null" class="card muted small">
            {{
                itemType === 'asset'
                    ? 'Rolimons has no value for this item. Only limiteds have values.'
                    : 'Bundles have no Rolimons values.'
            }}
        </div>
        <div v-else class="card faint small">Loading values…</div>
        <div class="card row between">
            <span class="faint small">{{ itemType === 'asset' ? 'Asset' : 'Bundle' }} {{ id }}</span>
            <button class="btn btn-ghost btn-sm" @click="CopyId">{{ copied ? 'Copied' : 'Copy ID' }}</button>
        </div>
    </div>
</template>

<style scoped>
a.btn {
    color: var(--fg);
    text-decoration: none;
}
</style>
