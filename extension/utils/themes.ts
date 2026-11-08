export interface Theme {
    id: string
    name: string
    dark: boolean
    bg: string
    surface: string
    raised: string
    text: string
    muted: string
    accent: string
    accentText: string
    line: string
    radius: number
    font: string
    image: string
    imageDim: number
    imageBlur: number
    glass: number
}

export const NO_THEME = 'roblox'

export const FONTS: { label: string; value: string }[] = [
    { label: 'Roblox default', value: '' },
    { label: 'Segoe UI', value: "'Segoe UI Variable', 'Segoe UI', system-ui, sans-serif" },
    { label: 'Inter', value: "Inter, 'Segoe UI Variable', system-ui, sans-serif" },
    { label: 'Arial', value: 'Arial, Helvetica, sans-serif' },
    { label: 'Georgia', value: "Georgia, 'Times New Roman', serif" },
    { label: 'Consolas', value: "Consolas, 'Cascadia Code', ui-monospace, monospace" },
    { label: 'Trebuchet', value: "'Trebuchet MS', sans-serif" },
    { label: 'Comic Sans', value: "'Comic Sans MS', 'Comic Sans', cursive" },
]

const BASE = { radius: 12, font: '', image: '', imageDim: 0.55, imageBlur: 0, glass: 0.86 }

export const PRESETS: Theme[] = [
    {
        ...BASE,
        id: 'umbra',
        name: 'Umbra mono',
        dark: true,
        bg: '#000000',
        surface: '#0d0d0e',
        raised: '#17171a',
        text: '#f0f0f0',
        muted: '#a1a4a5',
        accent: '#ffffff',
        accentText: '#000000',
        line: '#ffffff17',
    },
    {
        ...BASE,
        id: 'oled',
        name: 'OLED',
        dark: true,
        bg: '#000000',
        surface: '#000000',
        raised: '#0c0c0c',
        text: '#ffffff',
        muted: '#9a9a9a',
        accent: '#3fe0a0',
        accentText: '#00140a',
        line: '#ffffff1f',
    },
    {
        ...BASE,
        id: 'eclipse',
        name: 'Eclipse',
        dark: true,
        bg: '#07060c',
        surface: '#110f1a',
        raised: '#1b1828',
        text: '#efedf7',
        muted: '#a29db9',
        accent: '#9580ff',
        accentText: '#0b0a12',
        line: '#ffffff14',
    },
    {
        ...BASE,
        id: 'ocean',
        name: 'Ocean',
        dark: true,
        bg: '#03101a',
        surface: '#0a1a28',
        raised: '#112637',
        text: '#e6f3ff',
        muted: '#8fb0c8',
        accent: '#4fc3f7',
        accentText: '#02121c',
        line: '#ffffff14',
    },
    {
        ...BASE,
        id: 'forest',
        name: 'Forest',
        dark: true,
        bg: '#06100a',
        surface: '#0d1b12',
        raised: '#15271b',
        text: '#e8f5ec',
        muted: '#93b39d',
        accent: '#5ad18a',
        accentText: '#04140a',
        line: '#ffffff14',
    },
    {
        ...BASE,
        id: 'sunset',
        name: 'Sunset',
        dark: true,
        bg: '#140a09',
        surface: '#1f1110',
        raised: '#2b1816',
        text: '#fff0ea',
        muted: '#c9a29a',
        accent: '#ff8a5c',
        accentText: '#1a0904',
        line: '#ffffff14',
    },
    {
        ...BASE,
        id: 'rose',
        name: 'Rose',
        dark: true,
        bg: '#12060c',
        surface: '#1c0c14',
        raised: '#28121e',
        text: '#ffeef6',
        muted: '#c79bb0',
        accent: '#ff6f9f',
        accentText: '#1a0410',
        line: '#ffffff14',
    },
    {
        ...BASE,
        id: 'snow',
        name: 'Snow',
        dark: false,
        bg: '#f4f4f5',
        surface: '#ffffff',
        raised: '#ececef',
        text: '#111113',
        muted: '#5d6066',
        accent: '#111113',
        accentText: '#ffffff',
        line: '#0000001a',
    },
]

const HEX_RE = /^#([0-9a-f]{6})([0-9a-f]{2})?$/i

export function Rgba(hex: string, alpha = 1) {
    const m = hex.match(HEX_RE)
    if (!m) return hex
    const n = parseInt(m[1]!, 16)
    const a = m[2] ? (parseInt(m[2], 16) / 255) * alpha : alpha
    return `rgba(${(n >> 16) & 255}, ${(n >> 8) & 255}, ${n & 255}, ${Math.round(a * 1000) / 1000})`
}

const SAFE_URL_RE = /^(https:\/\/[^\s"'()\\]+|data:image\/(png|jpe?g|webp|gif);base64,[a-z0-9+/=]+)$/i

export function SafeImage(url: string) {
    return SAFE_URL_RE.test(url.trim()) ? url.trim() : ''
}

export function Clamp(n: number, lo: number, hi: number) {
    return Number.isFinite(n) ? Math.min(hi, Math.max(lo, n)) : lo
}

export function Sanitize(t: Partial<Theme> & { name?: string }, id?: string): Theme {
    const base = PRESETS[0]!
    const color = (v: unknown, fallback: string) => (typeof v === 'string' && HEX_RE.test(v) ? v : fallback)
    return {
        id: id ?? (typeof t.id === 'string' && t.id ? t.id : `custom-${Date.now().toString(36)}`),
        name: (typeof t.name === 'string' ? t.name : 'Custom theme').slice(0, 40) || 'Custom theme',
        dark: t.dark !== false,
        bg: color(t.bg, base.bg),
        surface: color(t.surface, base.surface),
        raised: color(t.raised, base.raised),
        text: color(t.text, base.text),
        muted: color(t.muted, base.muted),
        accent: color(t.accent, base.accent),
        accentText: color(t.accentText, base.accentText),
        line: color(t.line, base.line),
        radius: Clamp(Number(t.radius ?? base.radius), 0, 24),
        font: FONTS.some((f) => f.value === t.font) ? String(t.font) : '',
        image: SafeImage(String(t.image ?? '')),
        imageDim: Clamp(Number(t.imageDim ?? base.imageDim), 0, 0.95),
        imageBlur: Clamp(Number(t.imageBlur ?? base.imageBlur), 0, 30),
        glass: Clamp(Number(t.glass ?? base.glass), 0.3, 1),
    }
}

export function ThemeCss(t: Theme): string {
    const hasImage = !!t.image
    const surface = hasImage ? Rgba(t.surface, t.glass) : t.surface
    const raised = hasImage ? Rgba(t.raised, t.glass) : t.raised
    const shift = (a: number) => Rgba(t.dark ? '#ffffff' : '#000000', a)
    const vars = `
    --color-surface-0: ${hasImage ? 'transparent' : t.bg} !important;
    --color-surface-100: ${surface} !important;
    --color-surface-200: ${raised} !important;
    --color-surface-300: ${raised} !important;
    --color-content-default: ${t.text} !important;
    --color-content-emphasis: ${t.text} !important;
    --color-content-muted: ${t.muted} !important;
    --color-content-link: ${t.accent} !important;
    --color-stroke-default: ${t.line} !important;
    --color-stroke-muted: ${t.line} !important;
    --color-stroke-emphasis: ${Rgba(t.text, 0.3)} !important;
    --color-action-emphasis-background: ${t.accent} !important;
    --color-action-emphasis-border: ${t.accent} !important;
    --color-action-emphasis-foreground: ${t.accentText} !important;
    --color-action-soft-emphasis-background: ${Rgba(t.accent, 0.16)} !important;
    --color-action-soft-emphasis-foreground: ${t.text} !important;
    --color-action-standard-background: ${raised} !important;
    --color-action-standard-border: ${t.line} !important;
    --color-action-standard-foreground: ${t.text} !important;
    --color-action-subtle-foreground: ${t.text} !important;
    --color-action-link-foreground: ${t.accent} !important;
    --color-shift-100: ${shift(0.04)} !important;
    --color-shift-200: ${shift(0.07)} !important;
    --color-shift-300: ${shift(0.1)} !important;
    --color-shift-400: ${shift(0.14)} !important;`
    const font = t.font ? `body.rbx-body, body.rbx-body * { font-family: ${t.font} !important; }` : ''
    const image = hasImage
        ? `body.rbx-body::before {
    content: '';
    position: fixed;
    inset: -${t.imageBlur * 2}px;
    z-index: -1;
    pointer-events: none;
    background: ${t.bg} url("${t.image}") center / cover no-repeat;
    filter: blur(${t.imageBlur}px) brightness(${Math.round((1 - t.imageDim) * 100) / 100});
}`
        : ''
    return `:root, html, body, .light-theme, .dark-theme, body.rbx-body {${vars}
    color-scheme: ${t.dark ? 'dark' : 'light'} !important;
}
html, body.rbx-body, #wrap, #container-main, .container-main, .content {
    background-color: ${hasImage ? 'transparent' : t.bg} !important;
}
body.rbx-body { color: ${t.text}; }
#header, .rbx-header, #navigation, .rbx-left-col {
    background-color: ${surface} !important;
    border-color: ${t.line} !important;
}
.section-content, .dropdown-menu, .popover, .modal-content, .rbx-tabs-horizontal .nav-tabs, .card, .list-item.game-card .game-card-container {
    background-color: ${surface} !important;
    border-color: ${t.line} !important;
}
.section-content, .modal-content, .dropdown-menu, .popover, .btn-primary-md, .btn-primary-lg, .btn-secondary-md, .btn-control-md, .btn-growth-md, .btn-growth-lg, .btn-common-play-game-lg, .input-field, .form-control, .game-card-thumb, .thumbnail-2d-container {
    border-radius: ${t.radius}px !important;
}
.btn-primary-md, .btn-primary-lg, .btn-primary-sm, .btn-growth-md, .btn-growth-lg, .btn-growth-sm, .btn-common-play-game-lg {
    background-color: ${t.accent} !important;
    border-color: ${t.accent} !important;
    color: ${t.accentText} !important;
}
.btn-common-play-game-lg span, .btn-growth-md span, .btn-growth-lg span { color: ${t.accentText} !important; }
.input-field, .form-control, input.form-control, .rbx-navbar-search .input-field {
    background-color: ${raised} !important;
    color: ${t.text} !important;
    border-color: ${t.line} !important;
}
a.text-link, .text-link, .text-name.text-overflow, .link-menu { color: ${t.accent} !important; }
::selection { background: ${Rgba(t.accent, 0.35)}; }
${font}
${image}`
}

export function EncodeTheme(t: Theme): string {
    const { id: _id, ...rest } = t
    const json = JSON.stringify(rest)
    const bytes = new TextEncoder().encode(json)
    let bin = ''
    for (const b of bytes) bin += String.fromCharCode(b)
    return `umbra1.${btoa(bin).replace(/\+/g, '-').replace(/\//g, '_').replace(/=+$/, '')}`
}

export function DecodeTheme(code: string): Theme {
    const raw = code.trim().replace(/^umbra1\./, '')
    if (!raw || raw.length > 200_000) throw new Error('That theme code is empty or too long.')
    const b64 = raw.replace(/-/g, '+').replace(/_/g, '/')
    let json: unknown
    try {
        const bin = atob(b64 + '='.repeat((4 - (b64.length % 4)) % 4))
        json = JSON.parse(new TextDecoder().decode(Uint8Array.from(bin, (c) => c.charCodeAt(0))))
    } catch {
        throw new Error("That doesn't look like an Umbra theme code.")
    }
    if (!json || typeof json !== 'object') throw new Error("That doesn't look like an Umbra theme code.")
    return Sanitize(json as Partial<Theme>, `custom-${Date.now().toString(36)}`)
}
