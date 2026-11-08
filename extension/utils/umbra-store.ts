import { reactive } from 'vue'
import { Umbra } from './messages'
import { JoinUrl } from './roblox'
import { lastAccounts } from './settings'
import type { Hello, Target, UmbraState } from './types'

export const umbra = reactive({
    hello: null as Hello | null,
    state: null as UmbraState | null,
    picked: [] as number[],
    profileId: '' as string,
    busy: false,
    error: '' as string,
    note: null as { kind: 'good' | 'bad'; text: string } | null,
})

let noteTimer = 0
export function Note(kind: 'good' | 'bad', text: string) {
    umbra.note = { kind, text }
    clearTimeout(noteTimer)
    noteTimer = window.setTimeout(() => (umbra.note = null), kind === 'bad' ? 6000 : 3500)
}

export const Connected = () => !!umbra.hello?.paired && !!umbra.state

export async function RefreshUmbra() {
    try {
        umbra.hello = await Umbra.hello()
        umbra.error = umbra.hello.error ?? ''
    } catch (e) {
        umbra.hello = { reachable: false, enabled: false, paired: false, version: null }
        umbra.error = e instanceof Error ? e.message : String(e)
        umbra.state = null
        return
    }
    if (!umbra.hello.paired) {
        umbra.state = null
        return
    }
    try {
        umbra.state = await Umbra.state()
        const ids = new Set(umbra.state.accounts.map((a) => a.userId))
        if (!umbra.picked.length) {
            const saved = (await lastAccounts.getValue()).filter((id) => ids.has(id))
            umbra.picked = saved.length ? saved : umbra.state.accounts.slice(0, 1).map((a) => a.userId)
        } else umbra.picked = umbra.picked.filter((id) => ids.has(id))
    } catch (e) {
        umbra.state = null
        Note('bad', String(e instanceof Error ? e.message : e))
    }
}

export async function Connect() {
    umbra.busy = true
    Note('good', 'Check Umbra on your desktop and press Allow.')
    try {
        await Umbra.pair()
        await RefreshUmbra()
        Note('good', 'Connected to Umbra.')
    } catch (e) {
        Note('bad', String(e instanceof Error ? e.message : e))
    }
    umbra.busy = false
}

export function TogglePick(id: number) {
    umbra.picked = umbra.picked.includes(id) ? umbra.picked.filter((x) => x !== id) : [...umbra.picked, id]
    lastAccounts.setValue(umbra.picked)
}

export async function Join(placeId: number, jobId?: string | null, place?: string) {
    if (!Connected() || !umbra.picked.length) {
        location.href = JoinUrl(placeId, jobId)
        return
    }
    const target: Target = { kind: 'place', place: place ?? String(placeId), jobId: jobId ?? undefined }
    umbra.busy = true
    try {
        const r = await Umbra.launch(umbra.picked, target, umbra.profileId || null)
        const names = umbra.state?.accounts.filter((a) => umbra.picked.includes(a.userId)).map((a) => a.displayName) ?? []
        if (r.errors.length) Note('bad', r.errors.map((e) => e.error).join(' '))
        else Note('good', r.started === 1 ? `Launching as ${names[0] ?? 'your account'}.` : `Launching ${r.started} accounts.`)
    } catch (e) {
        Note('bad', String(e instanceof Error ? e.message : e))
    }
    umbra.busy = false
}

export async function FollowJoin(username: string) {
    if (!Connected() || !umbra.picked.length) return false
    umbra.busy = true
    try {
        const r = await Umbra.launch(umbra.picked, { kind: 'follow', username }, umbra.profileId || null)
        if (r.errors.length) Note('bad', r.errors.map((e) => e.error).join(' '))
        else Note('good', `Joining ${username}.`)
    } catch (e) {
        Note('bad', String(e instanceof Error ? e.message : e))
    }
    umbra.busy = false
    return true
}
