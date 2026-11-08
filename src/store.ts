import { reactive, watch } from 'vue'
import { api, type Account, type Chart, type Favorite, type Instance, type ProfileStore, type Target } from './api'
import { Sleep } from './format'

export type View = 'home' | 'discover' | 'game' | 'friends' | 'catalog' | 'avatar' | 'trades' | 'accounts' | 'instances' | 'settings'
export interface Route {
    view: View
    universeId?: number
    name?: string
    userId?: number
    query?: string
}

const ACTIVE_KEY = 'umbra.activeAccount'
function ReadActive(): number | null {
    try {
        const v = localStorage.getItem(ACTIVE_KEY)
        return v ? Number(v) : null
    } catch {
        return null
    }
}

export const store = reactive({
    route: { view: 'home' } as Route,
    back: [] as Route[],
    fwd: [] as Route[],
    accounts: [] as Account[],
    activeId: ReadActive(),
    running: [] as Instance[],
    profiles: null as ProfileStore | null,
    charts: null as Chart[] | null,
    favorites: [] as Favorite[],
    palette: false,
    adding: false,
    toast: null as { kind: 'good' | 'bad'; text: string; id: number } | null,
})

watch(
    () => store.activeId,
    (v) => {
        try {
            v == null ? localStorage.removeItem(ACTIVE_KEY) : localStorage.setItem(ACTIVE_KEY, String(v))
        } catch {}
    },
)

export function ActiveAccount(): Account | null {
    return store.accounts.find((a) => a.userId === store.activeId) ?? store.accounts[0] ?? null
}

const SameRoute = (a: Route, b: Route) => a.view === b.view && a.universeId === b.universeId && a.userId === b.userId && a.query === b.query
export function Go(r: Route) {
    if (SameRoute(r, store.route)) return
    store.back.push(store.route)
    store.fwd = []
    store.route = r
}
export function Back() {
    const r = store.back.pop()
    if (r) {
        store.fwd.push(store.route)
        store.route = r
    }
}
export function Forward() {
    const r = store.fwd.pop()
    if (r) {
        store.back.push(store.route)
        store.route = r
    }
}

let toastTimer = 0
export function Toast(kind: 'good' | 'bad', text: string) {
    store.toast = { kind, text, id: Date.now() }
    clearTimeout(toastTimer)
    toastTimer = window.setTimeout(() => (store.toast = null), kind === 'bad' ? 7000 : 3500)
}

export async function RefreshAccounts() {
    store.accounts = await api.accounts()
    if (store.activeId != null && !store.accounts.some((a) => a.userId === store.activeId)) store.activeId = null
}
export async function RefreshRunning() {
    try {
        const info = await api.running()
        store.running = info.running
        for (const e of info.exited.filter((x) => x.crashed)) Toast('bad', `Roblox (${e.label}) closed unexpectedly, exit code ${e.code}.`)
        if (info.exited.length) RefreshProfiles()
    } catch {}
}

export async function RefreshProfiles() {
    try {
        store.profiles = await api.profiles()
    } catch (e) {
        Toast('bad', String(e))
    }
}

export const ProfileName = (id: string | null | undefined) => store.profiles?.profiles.find((p) => p.id === id)?.name

export async function LoadCharts(force = false) {
    if (!store.charts || force) store.charts = await api.charts()
    return store.charts
}

export async function AddAccount() {
    if (store.adding) return
    store.adding = true
    try {
        const a = await api.addAccount()
        await RefreshAccounts()
        if (store.activeId == null) store.activeId = a.userId
        Toast('good', `Added ${a.displayName}.`)
    } catch (e) {
        if (String(e) !== 'Login cancelled.') Toast('bad', String(e))
    }
    store.adding = false
}

export async function LaunchMany(userIds: number[], target: Target, profileId?: string): Promise<number> {
    let ok = 0
    for (const [i, id] of userIds.entries()) {
        if (i > 0) await Sleep(1500)
        const name = store.accounts.find((a) => a.userId === id)?.displayName ?? 'Account'
        try {
            await api.launchAccount(id, target, profileId)
            ok++
        } catch (e) {
            Toast('bad', `${name}: ${e}`)
        }
        RefreshRunning()
    }
    if (ok && ok === userIds.length) Toast('good', ok === 1 ? 'Launching Roblox…' : `Launching ${ok} accounts…`)
    return ok
}

export async function QuickPlay(placeId: number, jobId?: string) {
    const a = ActiveAccount()
    if (!a) {
        Toast('bad', 'Add a Roblox account to play from Umbra.')
        Go({ view: 'accounts' })
        return
    }
    await LaunchMany([a.userId], { kind: 'place', place: String(placeId), jobId })
}

export async function RefreshFavorites() {
    try {
        store.favorites = await api.favorites()
    } catch (e) {
        Toast('bad', String(e))
    }
}

export const IsFavorite = (universeId: number) => store.favorites.some((f) => f.universeId === universeId)

export async function ToggleFavorite(game: { universeId: number; rootPlaceId: number; name: string; icon: string | null }) {
    try {
        const added = await api.toggleFavorite({ ...game, added: 0 })
        await RefreshFavorites()
        Toast('good', added ? `Added ${game.name} to favorites.` : `Removed ${game.name} from favorites.`)
    } catch (e) {
        Toast('bad', String(e))
    }
}
