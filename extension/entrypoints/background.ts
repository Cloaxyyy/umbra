import type { Hello, ItemValue, PlayerValue, Reply, Request } from '@/utils/types'
import { umbraToken } from '@/utils/settings'

const BRIDGE = 'http://127.0.0.1:27150'
const VALUES_TTL = 10 * 60 * 1000

let values: { at: number; items: Record<string, unknown[]> } | null = null

async function Bridge<T>(path: string, init: RequestInit = {}, auth = true): Promise<T> {
    const token = await umbraToken.getValue()
    let r: Response
    try {
        r = await fetch(BRIDGE + path, {
            ...init,
            headers: {
                'Content-Type': 'application/json',
                'X-Umbra-Client': 'extension',
                ...(auth && token ? { Authorization: `Bearer ${token}` } : {}),
                ...(init.headers ?? {}),
            },
        })
    } catch (e) {
        throw new Error(`Umbra isn’t running. Open the Umbra app and try again. (${e instanceof Error ? e.message : e})`)
    }
    const body = (await r.json().catch(() => ({}))) as { error?: string }
    if (r.status === 401) await umbraToken.setValue('')
    if (!r.ok) throw new Error(body.error ?? `Umbra answered with ${r.status}.`)
    return body as T
}

async function Hello(): Promise<Hello> {
    try {
        const h = await Bridge<{ enabled: boolean; paired: boolean; version: string }>('/v1/hello')
        return { reachable: true, enabled: h.enabled, paired: h.paired, version: h.version }
    } catch (e) {
        return { reachable: false, enabled: false, paired: false, version: null, error: e instanceof Error ? e.message : String(e) }
    }
}

function BrowserLabel() {
    const ua = navigator.userAgent
    const name = ua.includes('Edg/') ? 'Edge' : ua.includes('Firefox/') ? 'Firefox' : ua.includes('OPR/') ? 'Opera' : 'Chrome'
    return `Umbra extension in ${name}`
}

async function Pair(): Promise<boolean> {
    const r = await Bridge<{ token: string }>('/v1/pair', { method: 'POST', body: JSON.stringify({ label: BrowserLabel() }) }, false)
    await umbraToken.setValue(r.token)
    return true
}

async function Values(): Promise<Record<string, unknown[]>> {
    if (values && Date.now() - values.at < VALUES_TTL) return values.items
    const r = await fetch('https://api.rolimons.com/items/v2/itemdetails')
    if (!r.ok) {
        if (values) return values.items
        throw new Error(`Rolimons answered with ${r.status}.`)
    }
    const v = (await r.json()) as { items?: Record<string, unknown[]> }
    values = { at: Date.now(), items: v.items ?? {} }
    return values.items
}

async function Item(id: number): Promise<ItemValue | null> {
    const a = (await Values())[String(id)]
    if (!a) return null
    const n = (i: number) => (typeof a[i] === 'number' ? (a[i] as number) : -1)
    return {
        name: String(a[0] ?? ''),
        acronym: String(a[1] ?? ''),
        rap: Math.max(0, n(2)),
        value: n(3) >= 0 ? n(3) : null,
        demand: n(5),
        trend: n(6),
        projected: n(7) === 1,
        hyped: n(8) === 1,
        rare: n(9) === 1,
    }
}

async function Player(id: number): Promise<PlayerValue | null> {
    const r = await fetch(`https://api.rolimons.com/players/v1/playerinfo/${id}`)
    if (!r.ok) return null
    const v = (await r.json()) as Record<string, unknown>
    if (v.success !== true) return null
    return {
        value: Number(v.value ?? 0),
        rap: Number(v.rap ?? 0),
        rank: typeof v.rank === 'number' ? v.rank : null,
        premium: v.premium === true,
        lastOnline: typeof v.last_online === 'number' ? v.last_online : null,
    }
}

async function Handle(req: Request): Promise<unknown> {
    switch (req.type) {
        case 'umbra.hello':
            return Hello()
        case 'umbra.pair':
            return Pair()
        case 'umbra.unpair':
            await Bridge('/v1/unpair', { method: 'POST', body: '{}' }).catch(() => {})
            await umbraToken.setValue('')
            return true
        case 'umbra.state':
            return Bridge('/v1/state')
        case 'umbra.launch':
            return Bridge('/v1/launch', {
                method: 'POST',
                body: JSON.stringify({ userIds: req.userIds, target: req.target, profileId: req.profileId ?? null }),
            })
        case 'umbra.favorite':
            return Bridge('/v1/favorite', {
                method: 'POST',
                body: JSON.stringify({
                    universeId: req.universeId,
                    rootPlaceId: req.rootPlaceId,
                    name: req.name,
                    icon: req.icon,
                    added: 0,
                }),
            })
        case 'umbra.client':
            return Bridge('/v1/client', { method: 'POST', body: JSON.stringify({ pid: req.pid, action: req.action }) })
        case 'umbra.playtime':
            return Bridge(`/v1/playtime?universeId=${Number(req.universeId)}`)
        case 'rolimons.item':
            return Item(req.id)
        case 'rolimons.player':
            return Player(req.id)
        case 'options.open':
            await browser.runtime.openOptionsPage()
            return true
    }
}

export default defineBackground(() => {
    browser.runtime.onMessage.addListener((msg: unknown, sender, respond: (r: Reply) => void) => {
        if (sender.id !== browser.runtime.id) return false
        Handle(msg as Request)
            .then((data) => respond({ ok: true, data }))
            .catch((e: unknown) => respond({ ok: false, error: e instanceof Error ? e.message : String(e) }))
        return true
    })
})
