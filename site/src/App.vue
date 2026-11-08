<script setup lang="ts">
import { computed, nextTick, onMounted, onUnmounted, ref, watch } from 'vue'
import { SNIPPETS, Tokenize } from './code'
import Eclipse from './components/Eclipse.vue'
import Icon from './components/Icon.vue'
import { CARDS, FAQ, FEATURES, OVERLAY_POINTS, SHOWCASE } from './data'
import { StartMotion, SwapShot } from './motion'
import { DOWNLOAD_URL, GITHUB_URL, ISSUES_URL, LICENSE_URL, LOGO_URL, RELEASE_URL, RELEASES_URL } from './links'

const VERSION = '1.0.4'
const BASE = import.meta.env.BASE_URL
const SHOTS = `${BASE}shots/`
const Src = (id: string) => `${SHOTS}${id}-1440.webp`
const Set2x = (id: string) => `${SHOTS}${id}-1440.webp 1440w, ${SHOTS}${id}.webp 2880w`
const ECLIPSE = `${BASE}eclipse.webp`
const PHONE = '(max-width: 640px)'
const Phone = (id: string) => `${SHOTS}m/${id}.webp`

const menu = ref(false)
function CloseMenu() {
    menu.value = false
}
function OnKey(e: KeyboardEvent) {
    if (e.key === 'Escape') CloseMenu()
}

const active = ref(0)
const shot = computed(() => SHOWCASE[active.value] ?? SHOWCASE[0]!)
function OnTabKey(e: KeyboardEvent) {
    const n = SHOWCASE.length
    if (e.key === 'ArrowRight') active.value = (active.value + 1) % n
    else if (e.key === 'ArrowLeft') active.value = (active.value - 1 + n) % n
    else return
    e.preventDefault()
    document.getElementById(`tab-${shot.value.id}`)?.focus()
}

watch(active, () =>
    nextTick(() => {
        SwapShot(document.querySelector('#tour .window img'))
        document.getElementById(`tab-${shot.value.id}`)?.scrollIntoView({ inline: 'center', block: 'nearest', behavior: 'smooth' })
    }),
)

let stop: (() => void) | null = null
onMounted(() => {
    stop = StartMotion()
    window.addEventListener('keydown', OnKey)
})
onUnmounted(() => {
    stop?.()
    window.removeEventListener('keydown', OnKey)
})

const file = ref(0)
const snippet = computed(() => SNIPPETS[file.value] ?? SNIPPETS[0]!)
const lines = computed(() => Tokenize(snippet.value.code))

const open = ref<number | null>(0)
function Toggle(i: number) {
    open.value = open.value === i ? null : i
}
</script>

<template>
    <a class="skip" href="#main">Skip to content</a>

    <header class="nav" :class="{ open: menu }">
        <div class="wrap nav-row">
            <a class="brand" href="#top" aria-label="Umbra home"><img :src="LOGO_URL" alt="" width="24" height="24" />Umbra</a>
            <nav class="nav-links" aria-label="Sections">
                <a href="#tour">Tour</a>
                <a href="#players">Features</a>
                <a href="#overlay">Overlay</a>
                <a href="#source">Open source</a>
                <a href="#faq">FAQ</a>
            </nav>
            <div class="nav-cta">
                <a class="link hide-sm" :href="GITHUB_URL" target="_blank" rel="noopener noreferrer">GitHub</a>
                <a class="btn" :href="DOWNLOAD_URL" target="_blank" rel="noopener noreferrer">Download</a>
                <button
                    class="menu-btn"
                    :aria-expanded="menu"
                    aria-controls="phone-menu"
                    :aria-label="menu ? 'Close menu' : 'Open menu'"
                    @click="menu = !menu"
                >
                    <Icon :name="menu ? 'close' : 'menu'" :size="20" />
                </button>
            </div>
        </div>
        <nav v-show="menu" id="phone-menu" class="phone-menu" aria-label="Sections">
            <a href="#tour" @click="CloseMenu">Tour</a>
            <a href="#players" @click="CloseMenu">Features</a>
            <a href="#overlay" @click="CloseMenu">Overlay</a>
            <a href="#source" @click="CloseMenu">Open source</a>
            <a href="#faq" @click="CloseMenu">FAQ</a>
            <a :href="GITHUB_URL" target="_blank" rel="noopener noreferrer" @click="CloseMenu">GitHub</a>
        </nav>
    </header>

    <main id="main">
        <section id="top" class="hero">
            <div class="beams" aria-hidden="true"><span></span><span></span><span></span><span></span></div>
            <div class="wrap hero-grid">
                <div class="hero-text">
                    <a class="pill" :href="RELEASE_URL" target="_blank" rel="noopener noreferrer"
                        >Umbra {{ VERSION }} is out<Icon name="right" :size="14"
                    /></a>
                    <h1 class="fade">Roblox for<br />power players</h1>
                    <p class="lead">
                        The best way to run every account, pick every server and tune every game. Free and open source, for Windows.
                    </p>
                    <div class="cta">
                        <a class="btn btn-lg" :href="DOWNLOAD_URL" target="_blank" rel="noopener noreferrer">Download for Windows</a>
                        <a class="link link-lg" :href="GITHUB_URL" target="_blank" rel="noopener noreferrer">View on GitHub</a>
                    </div>
                </div>
                <div class="hero-art" aria-hidden="true">
                    <Eclipse :fallback="ECLIPSE" />
                </div>
            </div>
        </section>

        <section id="tour" class="section center wrap">
            <div class="tile" aria-hidden="true"><Icon name="rocket" :size="30" :stroke="1.6" /></div>
            <h2 class="fade">See the <span class="tint">whole app</span></h2>
            <p class="sub">
                Real screenshots of Umbra with real games from Roblox. Everything you do with Roblox, in one window that opens in a second.
            </p>

            <div class="langs" role="tablist" aria-label="App pages" @keydown="OnTabKey">
                <button
                    v-for="(s, i) in SHOWCASE"
                    :id="`tab-${s.id}`"
                    :key="s.id"
                    role="tab"
                    :aria-selected="active === i"
                    :aria-controls="`panel-${s.id}`"
                    :tabindex="active === i ? 0 : -1"
                    @click="active = i"
                >
                    <span class="lang-tile"><Icon :name="s.icon" :size="20" :stroke="1.7" /></span>
                    <span class="lang-label">{{ s.label }}</span>
                </button>
            </div>

            <div :id="`panel-${shot.id}`" class="window" role="tabpanel" :aria-labelledby="`tab-${shot.id}`">
                <div class="window-bar">
                    <span class="window-title"><Icon :name="shot.icon" :size="14" />{{ shot.label }}</span>
                    <span class="window-note">{{ shot.title }}</span>
                </div>
                <picture :key="shot.id">
                    <source :media="PHONE" :srcset="Phone(shot.id)" />
                    <img
                        :src="Src(shot.id)"
                        :srcset="Set2x(shot.id)"
                        sizes="(min-width: 1180px) 1100px, 100vw"
                        width="1440"
                        height="900"
                        :alt="shot.alt"
                        loading="lazy"
                    />
                </picture>
            </div>
            <p class="caption">{{ shot.title }}</p>
        </section>

        <section id="players" class="section wrap">
            <h2 class="fade left">Built for people<br />who play a lot</h2>
            <p class="sub left">Alts, servers and settings should take one click, not five tabs. Umbra puts all of it in one place.</p>

            <div class="cards">
                <article v-for="c in CARDS" :key="c.id" class="card">
                    <div class="card-shot">
                        <img
                            :src="Src(c.id)"
                            :srcset="Set2x(c.id)"
                            sizes="(min-width: 1180px) 560px, 100vw"
                            width="1440"
                            height="900"
                            :alt="c.alt"
                            loading="lazy"
                        />
                    </div>
                    <div class="card-text">
                        <h3>{{ c.title }}</h3>
                        <p>{{ c.text }}</p>
                    </div>
                </article>
            </div>

            <ul class="list">
                <li v-for="f in FEATURES" :key="f.title">
                    <Icon :name="f.icon" :size="18" :stroke="1.7" />
                    <h3>{{ f.title }}</h3>
                    <p>{{ f.text }}</p>
                </li>
            </ul>
        </section>

        <section id="overlay" class="section center wrap">
            <div class="tile" aria-hidden="true"><Icon name="gamepad" :size="30" :stroke="1.6" /></div>
            <h2 class="fade">Your controls, <span class="tint">in the game</span></h2>
            <p class="sub">
                Press <kbd>Ctrl</kbd> <kbd>Shift</kbd> <kbd>U</kbd> in any Roblox window Umbra started. The overlay opens on top of the
                game, and Esc puts you right back.
            </p>
            <div class="window shadow">
                <picture>
                    <source :media="PHONE" :srcset="Phone('overlay')" />
                    <img
                        :src="Src('overlay')"
                        :srcset="Set2x('overlay')"
                        sizes="(min-width: 1180px) 1100px, 100vw"
                        width="1440"
                        height="900"
                        alt="The Umbra overlay open on the right side of 99 Nights in the Forest, with rejoin, server hop and friends in the server"
                        loading="lazy"
                    />
                </picture>
            </div>
            <ul class="trio">
                <li v-for="p in OVERLAY_POINTS" :key="p.title">
                    <h3>{{ p.title }}</h3>
                    <p>{{ p.text }}</p>
                </li>
            </ul>
        </section>

        <section id="source" class="section center wrap">
            <div class="tile" aria-hidden="true"><Icon name="shield" :size="30" :stroke="1.6" /></div>
            <h2 class="fade">Open source, <span class="tint">every line</span></h2>
            <p class="sub">
                MIT licensed and built in the open. No injected code and no modified client. Read exactly what runs on your PC.
            </p>

            <div class="window code">
                <div class="window-bar">
                    <div class="files" role="tablist" aria-label="Source files">
                        <button v-for="(s, i) in SNIPPETS" :key="s.file" role="tab" :aria-selected="file === i" @click="file = i">
                            {{ s.label }}
                        </button>
                    </div>
                    <span class="window-note mono">{{ snippet.file }}</span>
                </div>
                <pre><code><span v-for="(line, i) in lines" :key="`${file}-${i}`" class="ln"><span class="num">{{ i + 1 }}</span><span v-for="(tk, j) in line" :key="j" :class="tk.c">{{ tk.t }}</span></span></code></pre>
                <div class="window-foot">
                    <a :href="GITHUB_URL" target="_blank" rel="noopener noreferrer"><Icon name="github" :size="14" />View on GitHub</a>
                    <a :href="LICENSE_URL" target="_blank" rel="noopener noreferrer"><Icon name="shield" :size="14" />MIT license</a>
                </div>
            </div>
        </section>

        <section id="faq" class="section wrap faq">
            <div>
                <h2 class="fade left">Questions</h2>
                <p class="sub left">
                    Something else? <a class="inline" :href="ISSUES_URL" target="_blank" rel="noopener noreferrer">Open an issue</a> on
                    GitHub.
                </p>
            </div>
            <ul>
                <li v-for="(f, i) in FAQ" :key="f.q" :class="{ on: open === i }">
                    <h3>
                        <button :id="`q-${i}`" :aria-expanded="open === i" :aria-controls="`a-${i}`" @click="Toggle(i)">
                            {{ f.q }}<Icon name="plus" :size="18" />
                        </button>
                    </h3>
                    <p v-show="open === i" :id="`a-${i}`" role="region" :aria-labelledby="`q-${i}`">{{ f.a }}</p>
                </li>
            </ul>
        </section>

        <section class="final">
            <div class="beams soft" aria-hidden="true"><span></span><span></span><span></span></div>
            <div class="wrap final-inner">
                <h2 class="serif fade">Roblox, reimagined.<br />Available today.</h2>
                <div class="cta">
                    <a class="btn btn-lg" :href="DOWNLOAD_URL" target="_blank" rel="noopener noreferrer">Download Umbra {{ VERSION }}</a>
                    <a class="link link-lg" :href="RELEASE_URL" target="_blank" rel="noopener noreferrer">What's new</a>
                </div>
            </div>
        </section>
    </main>

    <footer class="foot">
        <div class="wrap foot-grid">
            <div class="foot-brand">
                <span class="brand"><img :src="LOGO_URL" alt="" width="24" height="24" />Umbra</span>
                <p>A free, open-source Roblox launcher for Windows.</p>
                <p class="faint">Not affiliated with Roblox Corporation.</p>
            </div>
            <nav aria-label="Product">
                <h3>Product</h3>
                <a :href="DOWNLOAD_URL" target="_blank" rel="noopener noreferrer">Download</a>
                <a :href="RELEASES_URL" target="_blank" rel="noopener noreferrer">Releases</a>
                <a href="#faq">FAQ</a>
            </nav>
            <nav aria-label="Project">
                <h3>Project</h3>
                <a :href="GITHUB_URL" target="_blank" rel="noopener noreferrer">GitHub</a>
                <a :href="ISSUES_URL" target="_blank" rel="noopener noreferrer">Report a bug</a>
                <a :href="LICENSE_URL" target="_blank" rel="noopener noreferrer">License</a>
            </nav>
        </div>
    </footer>
</template>
