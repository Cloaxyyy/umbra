import type { Flags } from './api'

export const ALLOWED = new Set([
    'DFIntCSGLevelOfDetailSwitchingDistance',
    'DFIntCSGLevelOfDetailSwitchingDistanceL12',
    'DFIntCSGLevelOfDetailSwitchingDistanceL23',
    'DFIntCSGLevelOfDetailSwitchingDistanceL34',
    'FFlagHandleAltEnterFullscreenManually',
    'DFFlagTextureQualityOverrideEnabled',
    'DFIntTextureQualityOverride',
    'FIntDebugForceMSAASamples',
    'DFFlagDisableDPIScale',
    'FFlagDebugGraphicsPreferD3D11',
    'FFlagDebugSkyGray',
    'DFFlagDebugPauseVoxelizer',
    'DFIntDebugFRMQualityLevelOverride',
    'FIntFRMMaxGrassDistance',
    'FIntFRMMinGrassDistance',
    'FFlagDebugGraphicsPreferVulkan',
    'FFlagDebugGraphicsPreferOpenGL',
    'FIntGrassMovementReducedMotionFactor',
])

export const IgnoredFlags = (f: Flags) => Object.keys(f).filter((k) => !ALLOWED.has(k))

type Option = { label: string; set: Flags }

export interface Control {
    id: string
    label: string
    hint: string
    keys: string[]
    options: Option[]
}

const NONE: Option = { label: 'Roblox default', set: {} }
export const CONTROLS: Control[] = [
    {
        id: 'api',
        label: 'Graphics API',
        hint: 'Vulkan or OpenGL can help on some GPUs. Direct3D 11 is the most compatible.',
        keys: ['FFlagDebugGraphicsPreferD3D11', 'FFlagDebugGraphicsPreferVulkan', 'FFlagDebugGraphicsPreferOpenGL'],
        options: [
            NONE,
            { label: 'Direct3D 11', set: { FFlagDebugGraphicsPreferD3D11: true } },
            { label: 'Vulkan', set: { FFlagDebugGraphicsPreferVulkan: true } },
            { label: 'OpenGL', set: { FFlagDebugGraphicsPreferOpenGL: true } },
        ],
    },
    {
        id: 'lighting',
        label: 'Lighting quality',
        hint: "Forces Roblox's graphics level for lighting and shadows. Low frees up a lot of FPS.",
        keys: ['DFIntDebugFRMQualityLevelOverride'],
        options: [
            NONE,
            { label: 'Lowest', set: { DFIntDebugFRMQualityLevelOverride: 1 } },
            { label: 'Low', set: { DFIntDebugFRMQualityLevelOverride: 5 } },
            { label: 'Medium', set: { DFIntDebugFRMQualityLevelOverride: 10 } },
            { label: 'High', set: { DFIntDebugFRMQualityLevelOverride: 15 } },
            { label: 'Max', set: { DFIntDebugFRMQualityLevelOverride: 21 } },
        ],
    },
    {
        id: 'textures',
        label: 'Texture quality',
        hint: 'Overrides how sharp textures are, whatever the in-game graphics slider says.',
        keys: ['DFFlagTextureQualityOverrideEnabled', 'DFIntTextureQualityOverride'],
        options: [
            NONE,
            ...['Lowest', 'Low', 'Medium', 'Highest'].map((label, i) => ({
                label,
                set: { DFFlagTextureQualityOverrideEnabled: true, DFIntTextureQualityOverride: i },
            })),
        ],
    },
    {
        id: 'msaa',
        label: 'Anti-aliasing',
        hint: 'Smooths jagged edges. Off is fastest.',
        keys: ['FIntDebugForceMSAASamples'],
        options: [
            NONE,
            { label: 'Off', set: { FIntDebugForceMSAASamples: 0 } },
            { label: '2x', set: { FIntDebugForceMSAASamples: 2 } },
            { label: '4x', set: { FIntDebugForceMSAASamples: 4 } },
            { label: '8x', set: { FIntDebugForceMSAASamples: 8 } },
        ],
    },
    {
        id: 'mesh',
        label: 'Mesh detail distance',
        hint: 'How far away detailed models stay detailed. Lower is faster.',
        keys: [
            'DFIntCSGLevelOfDetailSwitchingDistance',
            'DFIntCSGLevelOfDetailSwitchingDistanceL12',
            'DFIntCSGLevelOfDetailSwitchingDistanceL23',
            'DFIntCSGLevelOfDetailSwitchingDistanceL34',
        ],
        options: [
            NONE,
            {
                label: 'Minimum',
                set: {
                    DFIntCSGLevelOfDetailSwitchingDistance: 0,
                    DFIntCSGLevelOfDetailSwitchingDistanceL12: 0,
                    DFIntCSGLevelOfDetailSwitchingDistanceL23: 0,
                    DFIntCSGLevelOfDetailSwitchingDistanceL34: 0,
                },
            },
        ],
    },
    {
        id: 'grass',
        label: 'Grass',
        hint: 'Removing grass helps on big terrain maps.',
        keys: ['FIntFRMMinGrassDistance', 'FIntFRMMaxGrassDistance', 'FIntGrassMovementReducedMotionFactor'],
        options: [
            NONE,
            { label: 'Still (no sway)', set: { FIntGrassMovementReducedMotionFactor: 0 } },
            { label: 'Off', set: { FIntFRMMinGrassDistance: 0, FIntFRMMaxGrassDistance: 0 } },
        ],
    },
    {
        id: 'voxel',
        label: 'Voxel lighting',
        hint: 'Freezing it stops lighting from recalculating as things move: fewer shadows, more FPS.',
        keys: ['DFFlagDebugPauseVoxelizer'],
        options: [NONE, { label: 'Frozen', set: { DFFlagDebugPauseVoxelizer: true } }],
    },
    {
        id: 'sky',
        label: 'Sky',
        hint: 'A flat gray sky is slightly cheaper to draw.',
        keys: ['FFlagDebugSkyGray'],
        options: [NONE, { label: 'Gray', set: { FFlagDebugSkyGray: true } }],
    },
    {
        id: 'dpi',
        label: 'DPI scaling',
        hint: 'Turn off if Roblox looks blurry on a high-DPI screen.',
        keys: ['DFFlagDisableDPIScale'],
        options: [NONE, { label: 'Off', set: { DFFlagDisableDPIScale: true } }],
    },
    {
        id: 'altenter',
        label: 'Alt+Enter fullscreen',
        hint: 'Lets Alt+Enter toggle fullscreen like other Windows games.',
        keys: ['FFlagHandleAltEnterFullscreenManually'],
        options: [NONE, { label: 'On', set: { FFlagHandleAltEnterFullscreenManually: false } }],
    },
]

export function CurrentOption(c: Control, f: Flags): number {
    if (!c.keys.some((k) => k in f)) return 0
    return c.options.findIndex((o, i) => i > 0 && c.keys.every((k) => (k in o.set ? f[k] === o.set[k] : !(k in f))))
}

export function ApplyOption(c: Control, f: Flags, index: number): Flags {
    const next = { ...f }
    for (const k of c.keys) delete next[k]
    return { ...next, ...c.options[index].set }
}

export const PRESETS: { id: string; label: string; hint: string; picks: Record<string, string> }[] = [
    { id: 'default', label: 'Roblox default', hint: 'Clear every tuning flag.', picks: {} },
    {
        id: 'potato',
        label: 'Potato',
        hint: 'Lowest settings. Good for alts and AFK farms.',
        picks: { lighting: 'Lowest', textures: 'Lowest', msaa: 'Off', mesh: 'Minimum', grass: 'Off', voxel: 'Frozen', sky: 'Gray' },
    },
    {
        id: 'quality',
        label: 'Quality',
        hint: 'Sharpest textures, max lighting, 4x anti-aliasing.',
        picks: { lighting: 'Max', textures: 'Highest', msaa: '4x' },
    },
]

export function ApplyPreset(id: string, f: Flags): Flags {
    const preset = PRESETS.find((p) => p.id === id)!
    let next = { ...f }
    for (const c of CONTROLS) {
        const idx = c.options.findIndex((o) => o.label === preset.picks[c.id])
        next = ApplyOption(c, next, idx > 0 ? idx : 0)
    }
    return next
}
