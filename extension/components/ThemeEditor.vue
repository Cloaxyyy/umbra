<script setup lang="ts">
import { computed, ref } from 'vue'
import { EncodeTheme, FONTS, Rgba, SafeImage, Sanitize, type Theme } from '@/utils/themes'

const props = defineProps<{ theme: Theme }>()
const emit = defineEmits<{ save: [theme: Theme]; cancel: [] }>()

const MAX_IMAGE = 1_500_000
const t = ref<Theme>({ ...props.theme })
const imageUrl = ref(t.value.image.startsWith('data:') ? '' : t.value.image)
const error = ref('')
const copied = ref(false)

const COLORS: { key: keyof Theme; label: string }[] = [
    { key: 'bg', label: 'Page background' },
    { key: 'surface', label: 'Panels' },
    { key: 'raised', label: 'Raised panels and inputs' },
    { key: 'text', label: 'Text' },
    { key: 'muted', label: 'Secondary text' },
    { key: 'accent', label: 'Accent and buttons' },
    { key: 'accentText', label: 'Text on accent' },
    { key: 'line', label: 'Borders' },
]

const Hex6 = (v: string) => v.slice(0, 7)
function SetColor(key: keyof Theme, v: string) {
    const old = String(t.value[key])
    const alpha = old.length === 9 ? old.slice(7) : ''
    t.value = { ...t.value, [key]: v + alpha }
}
function SetUrl() {
    const safe = SafeImage(imageUrl.value)
    if (imageUrl.value && !safe) {
        error.value = 'Use an https image link.'
        return
    }
    error.value = ''
    t.value = { ...t.value, image: safe }
}
function Upload(e: Event) {
    const file = (e.target as HTMLInputElement).files?.[0]
    if (!file) return
    if (!/^image\/(png|jpe?g|webp|gif)$/.test(file.type)) {
        error.value = 'Pick a PNG, JPG, WebP or GIF image.'
        return
    }
    if (file.size > MAX_IMAGE) {
        error.value = 'That image is over 1.5 MB. Pick a smaller one.'
        return
    }
    const r = new FileReader()
    r.onload = () => {
        const url = SafeImage(String(r.result ?? ''))
        if (!url) error.value = "That image couldn't be read."
        else {
            error.value = ''
            imageUrl.value = ''
            t.value = { ...t.value, image: url }
        }
    }
    r.readAsDataURL(file)
}
function ClearImage() {
    imageUrl.value = ''
    t.value = { ...t.value, image: '' }
}
async function Share() {
    try {
        await navigator.clipboard.writeText(EncodeTheme(Sanitize(t.value, t.value.id)))
        copied.value = true
        setTimeout(() => (copied.value = false), 1600)
    } catch {
        error.value = "Couldn't copy the theme code."
    }
}

const preview = computed(() => {
    const v = t.value
    const glass = v.image ? Rgba(v.surface, v.glass) : v.surface
    return {
        page: { background: v.image ? `${v.bg} url("${v.image}") center / cover` : v.bg, fontFamily: v.font || 'inherit' },
        dim: {
            background: `rgba(0, 0, 0, ${v.image ? v.imageDim : 0})`,
            backdropFilter: v.imageBlur ? `blur(${v.imageBlur / 2}px)` : 'none',
        },
        nav: { background: glass, borderColor: v.line, color: v.text },
        card: { background: glass, borderColor: v.line, borderRadius: v.radius + 'px', color: v.text },
        raised: { background: v.image ? Rgba(v.raised, v.glass) : v.raised, borderRadius: Math.max(4, v.radius - 4) + 'px' },
        muted: { color: v.muted },
        button: { background: v.accent, color: v.accentText, borderRadius: Math.max(4, v.radius - 2) + 'px' },
        link: { color: v.accent },
    }
})
</script>

<template>
    <div class="editor">
        <div class="stack controls">
            <label class="stack field">
                <span class="label">Name</span>
                <input v-model="t.name" class="input" maxlength="40" placeholder="My theme" />
            </label>
            <div class="toggle">
                <div>
                    <h3>Dark theme</h3>
                    <p>Puts Roblox in dark mode under your colors.</p>
                </div>
                <button class="switch" role="switch" :aria-checked="t.dark" aria-label="Dark theme" @click="t.dark = !t.dark"></button>
            </div>
            <div class="colors">
                <label v-for="c in COLORS" :key="c.key" class="color">
                    <input
                        type="color"
                        :value="Hex6(String(t[c.key]))"
                        @input="SetColor(c.key, ($event.target as HTMLInputElement).value)"
                    />
                    <span class="small">{{ c.label }}</span>
                </label>
            </div>
            <label class="stack field">
                <span class="row between"
                    ><span class="label">Corner roundness</span><span class="faint small mono">{{ t.radius }}px</span></span
                >
                <input v-model.number="t.radius" type="range" min="0" max="24" step="1" />
            </label>
            <label class="stack field">
                <span class="label">Font</span>
                <select v-model="t.font" class="select">
                    <option v-for="f in FONTS" :key="f.label" :value="f.value">{{ f.label }}</option>
                </select>
            </label>
            <div class="stack field">
                <span class="label">Background image</span>
                <div class="row">
                    <input v-model="imageUrl" class="input" placeholder="https://example.com/wallpaper.jpg" @change="SetUrl" />
                    <label class="btn"
                        >Upload<input type="file" accept="image/png,image/jpeg,image/webp,image/gif" hidden @change="Upload"
                    /></label>
                    <button v-if="t.image" class="btn btn-ghost" @click="ClearImage">Remove</button>
                </div>
            </div>
            <template v-if="t.image">
                <label class="stack field">
                    <span class="row between"
                        ><span class="label">Dim image</span><span class="faint small mono">{{ Math.round(t.imageDim * 100) }}%</span></span
                    >
                    <input v-model.number="t.imageDim" type="range" min="0" max="0.95" step="0.05" />
                </label>
                <label class="stack field">
                    <span class="row between"
                        ><span class="label">Blur image</span><span class="faint small mono">{{ t.imageBlur }}px</span></span
                    >
                    <input v-model.number="t.imageBlur" type="range" min="0" max="30" step="1" />
                </label>
                <label class="stack field">
                    <span class="row between"
                        ><span class="label">Panel opacity</span
                        ><span class="faint small mono">{{ Math.round(t.glass * 100) }}%</span></span
                    >
                    <input v-model.number="t.glass" type="range" min="0.3" max="1" step="0.02" />
                </label>
            </template>
            <p v-if="error" class="note bad">{{ error }}</p>
            <div class="row actions">
                <button class="btn btn-ghost" @click="Share">{{ copied ? 'Code copied' : 'Copy share code' }}</button>
                <span class="row">
                    <button class="btn danger" @click="emit('cancel')">Cancel</button>
                    <button class="btn" @click="emit('save', Sanitize(t, t.id))">Save theme</button>
                </span>
            </div>
        </div>

        <div class="preview" :style="preview.page" aria-label="Preview">
            <div class="pv-dim" :style="preview.dim"></div>
            <div class="pv-nav" :style="preview.nav">
                <b>Roblox</b><span :style="preview.muted">Charts</span><span :style="preview.muted">Marketplace</span>
            </div>
            <div class="pv-card" :style="preview.card">
                <div class="pv-thumb" :style="preview.raised"></div>
                <div class="stack" style="gap: 6px">
                    <b>99 Nights in the Forest</b>
                    <span class="small" :style="preview.muted">By <span :style="preview.link">Grandma's Favourite Games</span></span>
                    <span class="pv-btn" :style="preview.button">Play</span>
                </div>
            </div>
            <div class="pv-row">
                <div v-for="n in 3" :key="n" class="pv-card small-card" :style="preview.card">
                    <div class="pv-mini" :style="preview.raised"></div>
                    <span class="small">Game {{ n }}</span>
                    <span class="small" :style="preview.muted">92% liked</span>
                </div>
            </div>
        </div>
    </div>
</template>

<style scoped>
.editor {
    display: grid;
    grid-template-columns: minmax(0, 1fr) minmax(0, 1fr);
    gap: 20px;
    align-items: start;
}
.controls {
    gap: 14px;
}
.field {
    gap: 6px;
}
.colors {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 8px;
}
.color {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 6px 8px;
    border-radius: 10px;
    background: var(--raised);
    cursor: pointer;
}
.color input {
    width: 28px;
    height: 28px;
    border: 0;
    padding: 0;
    background: none;
    cursor: pointer;
}
input[type='range'] {
    width: 100%;
    accent-color: #ffffff;
}
.actions {
    justify-content: space-between;
    flex-wrap: wrap;
}
.danger {
    background: rgba(255, 98, 131, 0.14) !important;
    border-color: rgba(255, 98, 131, 0.45) !important;
    color: #ffc2cf !important;
}
.preview {
    position: sticky;
    top: 16px;
    min-height: 420px;
    padding: 0 0 16px;
    border-radius: 16px;
    overflow: hidden;
    border: 1px solid var(--line-2);
    display: flex;
    flex-direction: column;
    gap: 14px;
    isolation: isolate;
}
.pv-dim {
    position: absolute;
    inset: 0;
    z-index: -1;
}
.pv-nav {
    display: flex;
    gap: 16px;
    align-items: center;
    height: 44px;
    padding: 0 16px;
    border-bottom: 1px solid;
}
.pv-card {
    margin: 0 16px;
    display: flex;
    gap: 14px;
    padding: 14px;
    border: 1px solid;
}
.pv-thumb {
    width: 96px;
    height: 96px;
    flex: none;
}
.pv-btn {
    align-self: flex-start;
    padding: 6px 18px;
    font-weight: 700;
}
.pv-row {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: 10px;
    margin: 0 16px;
}
.pv-row .pv-card {
    margin: 0;
}
.small-card {
    flex-direction: column;
    gap: 4px;
    padding: 10px;
}
.pv-mini {
    height: 54px;
}
@media (max-width: 860px) {
    .editor {
        grid-template-columns: 1fr;
    }
    .preview {
        position: static;
    }
}
</style>
