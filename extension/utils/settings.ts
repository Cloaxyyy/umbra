import { storage } from '#imports'
import { NO_THEME, PRESETS, Sanitize, type Theme } from './themes'

export interface Features {
    drawer: boolean
    servers: boolean
    values: boolean
    search: boolean
    hideAds: boolean
    autoOpen: boolean
}

export const DEFAULT_FEATURES: Features = {
    drawer: true,
    servers: true,
    values: true,
    search: true,
    hideAds: false,
    autoOpen: false,
}

export const features = storage.defineItem<Features>('local:features', { fallback: DEFAULT_FEATURES })
export const activeTheme = storage.defineItem<string>('local:activeTheme', { fallback: NO_THEME })
export const customThemes = storage.defineItem<Theme[]>('local:customThemes', { fallback: [] })
export const umbraToken = storage.defineItem<string>('local:umbraToken', { fallback: '' })
export const lastAccounts = storage.defineItem<number[]>('local:lastAccounts', { fallback: [] })

export async function LoadFeatures(): Promise<Features> {
    return { ...DEFAULT_FEATURES, ...(await features.getValue()) }
}

export async function CurrentTheme(): Promise<Theme | null> {
    const id = await activeTheme.getValue()
    if (id === NO_THEME) return null
    const custom = (await customThemes.getValue()).map((t) => Sanitize(t, t.id))
    return [...PRESETS, ...custom].find((t) => t.id === id) ?? null
}
