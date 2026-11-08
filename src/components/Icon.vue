<script setup lang="ts">
import { ICONS, type IconName } from '../icons'
withDefaults(defineProps<{ name: IconName; size?: number; stroke?: number; filled?: boolean }>(), { size: 18, stroke: 1.9, filled: false })
</script>

<template>
    <svg
        :width="size"
        :height="size"
        viewBox="0 0 24 24"
        :fill="filled ? 'currentColor' : 'none'"
        stroke="currentColor"
        :stroke-width="stroke"
        stroke-linecap="round"
        stroke-linejoin="round"
        aria-hidden="true"
    >
        <path v-for="(d, i) in ICONS[name]" :key="i" :d="d" />
    </svg>
</template>
