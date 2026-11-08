<script setup lang="ts">
import { computed } from 'vue'
import type { GameCard } from '../api'
import { Compact, LikePct } from '../format'
import { Go, QuickPlay } from '../store'
import Icon from './Icon.vue'

const props = defineProps<{ game: GameCard; banner?: string | null; wide?: boolean }>()
const pct = computed(() => LikePct(props.game.upVotes, props.game.downVotes))
const img = computed(() => (props.wide ? (props.banner ?? props.game.icon) : props.game.icon))
const Open = () => Go({ view: 'game', universeId: props.game.universeId, name: props.game.name })
</script>

<template>
    <article class="tile" :class="{ wide }">
        <button class="thumb" :aria-label="'Open ' + game.name" @click="Open">
            <img v-if="img" :src="img" alt="" loading="lazy" />
        </button>
        <div class="meta">
            <button class="info" @click="Open">
                <p class="name">{{ game.name }}</p>
                <p class="stats">
                    <Icon name="users" :size="13" /><span>{{ Compact(game.playerCount) }}</span>
                    <template v-if="pct !== null"
                        ><span class="sep">·</span><span class="like"><span class="dot"></span>{{ pct }}%</span></template
                    >
                </p>
            </button>
            <button
                class="play"
                :aria-label="'Play ' + game.name"
                title="Play with the active account"
                @click="QuickPlay(game.rootPlaceId)"
            >
                <Icon name="play" :size="14" />
            </button>
        </div>
    </article>
</template>

<style scoped>
.tile {
    display: flex;
    flex-direction: column;
    gap: 10px;
    min-width: 0;
}
.thumb {
    display: block;
    width: 100%;
    aspect-ratio: 1;
    border-radius: 14px;
    overflow: hidden;
    background: var(--surface-2);
    border: 1px solid var(--line);
    position: relative;
}
.wide .thumb {
    aspect-ratio: 16 / 9;
}
.thumb img {
    width: 100%;
    height: 100%;
    object-fit: cover;
    transition: transform 0.25s ease;
}
.thumb:hover img {
    transform: scale(1.04);
}
.thumb::after {
    content: '';
    position: absolute;
    inset: 0;
    border-radius: inherit;
    box-shadow: inset 0 0 0 1px rgba(255, 255, 255, 0.04);
    pointer-events: none;
}
.meta {
    display: flex;
    align-items: flex-start;
    gap: 8px;
}
.info {
    flex: 1;
    min-width: 0;
    text-align: left;
}
.name {
    font-weight: 700;
    font-size: 13.5px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
}
.info:hover .name {
    color: #fff;
}
.stats {
    display: flex;
    align-items: center;
    gap: 5px;
    font-size: 12px;
    color: var(--fg-3);
    font-weight: 600;
    font-variant-numeric: tabular-nums;
}
.sep {
    color: var(--fg-3);
}
.like {
    display: inline-flex;
    align-items: center;
    gap: 5px;
    color: var(--fg-2);
}
.like .dot {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    background: var(--good);
}
.play {
    width: 28px;
    height: 28px;
    flex: none;
    border-radius: 8px;
    display: grid;
    place-items: center;
    color: var(--fg-3);
}
.play:hover {
    background: var(--accent-soft);
    color: #c4b8ff;
}
</style>
