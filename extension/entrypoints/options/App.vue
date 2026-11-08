<script setup lang="ts">
import { onMounted, ref } from 'vue'
import Connection from '@/components/Connection.vue'
import FeatureToggles from '@/components/FeatureToggles.vue'
import ThemeEditor from '@/components/ThemeEditor.vue'
import ThemeGrid from '@/components/ThemeGrid.vue'
import { activeTheme, customThemes } from '@/utils/settings'
import { DecodeTheme, PRESETS, Sanitize, type Theme } from '@/utils/themes'

type Tab = 'themes' | 'features' | 'connection'
const TABS: { id: Tab; label: string }[] = [
    { id: 'themes', label: 'Themes' },
    { id: 'features', label: 'Features' },
    { id: 'connection', label: 'Umbra app' },
]
const MAX_CUSTOM = 30

const tab = ref<Tab>('themes')
const editing = ref<Theme | null>(null)
const code = ref('')
const msg = ref<{ kind: 'good' | 'bad'; text: string } | null>(null)
const confirmDelete = ref<Theme | null>(null)

function Say(kind: 'good' | 'bad', text: string) {
    msg.value = { kind, text }
    setTimeout(() => (msg.value = null), kind === 'bad' ? 6000 : 3000)
}

async function Save(t: Theme) {
    const list = (await customThemes.getValue()).map((x) => Sanitize(x, x.id))
    const i = list.findIndex((x) => x.id === t.id)
    if (i >= 0) list[i] = t
    else if (list.length >= MAX_CUSTOM) return Say('bad', `You can keep up to ${MAX_CUSTOM} custom themes.`)
    else list.push(t)
    try {
        await customThemes.setValue(list)
        await activeTheme.setValue(t.id)
        editing.value = null
        Say('good', `Saved ${t.name}. It's on now.`)
    } catch (e) {
        Say('bad', e instanceof Error ? e.message : 'Saving failed. The background image may be too large.')
    }
}

function NewTheme() {
    editing.value = Sanitize({ ...PRESETS[0]!, name: 'My theme' }, `custom-${Date.now().toString(36)}`)
}
function Duplicate(t: Theme) {
    editing.value = Sanitize({ ...t, name: `${t.name} copy` }, `custom-${Date.now().toString(36)}`)
}
async function Remove(t: Theme) {
    const list = (await customThemes.getValue()).filter((x) => x.id !== t.id)
    await customThemes.setValue(list)
    if ((await activeTheme.getValue()) === t.id) await activeTheme.setValue(PRESETS[0]!.id)
    confirmDelete.value = null
    Say('good', `Deleted ${t.name}.`)
}
function Import() {
    try {
        editing.value = DecodeTheme(code.value)
        code.value = ''
    } catch (e) {
        Say('bad', e instanceof Error ? e.message : String(e))
    }
}

onMounted(() => document.body.classList.add('u'))
</script>

<template>
    <div class="u page">
        <aside class="side">
            <span class="row brand"><img src="/icon/48.png" alt="" width="28" height="28" /><b>Umbra for Roblox</b></span>
            <nav class="stack" aria-label="Settings">
                <button v-for="t in TABS" :key="t.id" class="nav" :aria-pressed="tab === t.id" @click="tab = t.id">{{ t.label }}</button>
            </nav>
        </aside>

        <main class="main stack">
            <p v-if="msg" class="note" :class="msg.kind" role="status">{{ msg.text }}</p>

            <template v-if="tab === 'themes'">
                <header>
                    <h1>Themes</h1>
                    <p class="muted">Restyle roblox.com. Themes apply instantly to every Roblox tab.</p>
                </header>

                <section v-if="editing" class="card">
                    <ThemeEditor :key="editing.id" :theme="editing" @save="Save" @cancel="editing = null" />
                </section>

                <template v-else>
                    <section class="card stack">
                        <div class="row between">
                            <span class="label">Pick a theme</span>
                            <button class="btn btn-primary btn-sm" @click="NewTheme">New theme</button>
                        </div>
                        <ThemeGrid manage @edit="(t) => (editing = t)" @duplicate="Duplicate" @remove="(t) => (confirmDelete = t)" />
                        <div v-if="confirmDelete" class="row between note">
                            <span>Delete {{ confirmDelete.name }}?</span>
                            <span class="row">
                                <button class="btn btn-sm danger" @click="confirmDelete = null">Cancel</button>
                                <button class="btn btn-sm" @click="Remove(confirmDelete)">Delete</button>
                            </span>
                        </div>
                    </section>
                    <section class="card stack">
                        <span class="label">Import a theme</span>
                        <p class="muted small">Paste a share code someone sent you. You can review it before saving.</p>
                        <div class="row">
                            <input v-model="code" class="input" placeholder="umbra1.…" @keydown.enter="Import" />
                            <button class="btn" :disabled="!code.trim()" @click="Import">Import</button>
                        </div>
                    </section>
                </template>
            </template>

            <template v-else-if="tab === 'features'">
                <header>
                    <h1>Features</h1>
                    <p class="muted">Turn parts of the extension on or off. Changes apply right away.</p>
                </header>
                <section class="card"><FeatureToggles /></section>
            </template>

            <template v-else>
                <header>
                    <h1>Umbra app</h1>
                    <p class="muted">Connect to the Umbra desktop app to launch with your alts and control running clients.</p>
                </header>
                <section class="card"><Connection /></section>
            </template>
        </main>
    </div>
</template>

<style scoped>
.page {
    display: grid;
    grid-template-columns: 240px minmax(0, 1fr);
    min-height: 100vh;
}
.side {
    display: flex;
    flex-direction: column;
    gap: 24px;
    padding: 24px 16px;
    border-right: 1px solid var(--line);
}
.brand b {
    font-size: 15px;
}
.nav {
    height: 36px;
    padding: 0 12px;
    border-radius: 10px;
    text-align: left;
    font-weight: 600;
    color: var(--fg-2) !important;
}
.nav:hover {
    color: var(--fg) !important;
}
.nav[aria-pressed='true'] {
    background: #ffffff !important;
    color: #000000 !important;
}
.main {
    max-width: 1040px;
    padding: 32px 40px 64px;
    gap: 16px;
}
h1 {
    margin: 0 0 4px;
    font-size: 28px;
    letter-spacing: -0.03em;
}
.danger {
    background: rgba(255, 98, 131, 0.14) !important;
    border-color: rgba(255, 98, 131, 0.45) !important;
    color: #ffc2cf !important;
}
@media (max-width: 760px) {
    .page {
        grid-template-columns: 1fr;
    }
    .side {
        border-right: 0;
        border-bottom: 1px solid var(--line);
    }
    .main {
        padding: 20px 16px 48px;
    }
}
</style>
