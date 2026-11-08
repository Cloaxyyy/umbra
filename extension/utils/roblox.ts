let csrf = ''

async function Json<T>(r: Response, what: string): Promise<T> {
    if (!r.ok) throw new Error(r.status === 401 ? 'Sign in to Roblox to use this.' : `${what} failed (${r.status}).`)
    return (await r.json()) as T
}

export async function Get<T>(url: string, what = 'Loading from Roblox'): Promise<T> {
    return Json<T>(await fetch(url, { credentials: 'include' }), what)
}

export async function Post<T>(url: string, body: unknown, what = 'Roblox request'): Promise<T> {
    for (let i = 0; i < 2; i++) {
        const r = await fetch(url, {
            method: 'POST',
            credentials: 'include',
            headers: { 'Content-Type': 'application/json', ...(csrf ? { 'x-csrf-token': csrf } : {}) },
            body: JSON.stringify(body),
        })
        const token = r.headers.get('x-csrf-token')
        if (r.status === 403 && token && token !== csrf) {
            csrf = token
            continue
        }
        return Json<T>(r, what)
    }
    throw new Error(`${what} failed.`)
}

export interface GameInfo {
    universeId: number
    rootPlaceId: number
    name: string
    creator: string
    playing: number
    visits: number
    favorites: number
    maxPlayers: number
    created: string
    updated: string
    genre: string
    upVotes: number
    downVotes: number
    icon: string | null
}

export interface Server {
    id: string
    playing: number
    maxPlayers: number
    ping: number | null
    fps: number | null
}

export interface UserInfo {
    id: number
    name: string
    displayName: string
    created: string
    isBanned: boolean
    hasVerifiedBadge: boolean
}

export interface Presence {
    userId: number
    userPresenceType: number
    lastLocation: string
    placeId: number | null
    rootPlaceId: number | null
    gameId: string | null
    universeId: number | null
}

export async function UniverseOf(placeId: number): Promise<number> {
    const v = await Get<{ universeId: number | null }>(`https://apis.roblox.com/universes/v1/places/${placeId}/universe`)
    if (!v.universeId) throw new Error('Roblox has no game for this place.')
    return v.universeId
}

export async function Game(universeId: number): Promise<GameInfo> {
    const [g, votes, icons] = await Promise.all([
        Get<{ data: Record<string, unknown>[] }>(`https://games.roblox.com/v1/games?universeIds=${universeId}`),
        Get<{ data: { upVotes: number; downVotes: number }[] }>(`https://games.roblox.com/v1/games/votes?universeIds=${universeId}`).catch(
            () => ({ data: [] }),
        ),
        GameIcons([universeId]).catch(() => ({}) as Record<number, string>),
    ])
    const d = g.data[0]
    if (!d) throw new Error('Roblox has no details for this game.')
    const creator = (d.creator as { name?: string } | undefined)?.name ?? ''
    return {
        universeId,
        rootPlaceId: Number(d.rootPlaceId),
        name: String(d.name ?? ''),
        creator,
        playing: Number(d.playing ?? 0),
        visits: Number(d.visits ?? 0),
        favorites: Number(d.favoritedCount ?? 0),
        maxPlayers: Number(d.maxPlayers ?? 0),
        created: String(d.created ?? ''),
        updated: String(d.updated ?? ''),
        genre: String(d.genre ?? ''),
        upVotes: votes.data[0]?.upVotes ?? 0,
        downVotes: votes.data[0]?.downVotes ?? 0,
        icon: icons[universeId] ?? null,
    }
}

export async function Servers(placeId: number, cursor: string | null, sortOrder: 'Asc' | 'Desc' = 'Desc') {
    const q = `sortOrder=${sortOrder}&limit=100&excludeFullGames=true${cursor ? `&cursor=${encodeURIComponent(cursor)}` : ''}`
    const v = await Get<{ data: Server[]; nextPageCursor: string | null }>(
        `https://games.roblox.com/v1/games/${placeId}/servers/Public?${q}`,
    )
    return { servers: v.data, next: v.nextPageCursor }
}

export async function GameIcons(ids: number[]): Promise<Record<number, string>> {
    if (!ids.length) return {}
    const v = await Get<{ data: { targetId: number; imageUrl: string | null }[] }>(
        `https://thumbnails.roblox.com/v1/games/icons?universeIds=${ids.join(',')}&returnPolicy=PlaceHolder&size=150x150&format=Png&isCircular=false`,
    )
    return Object.fromEntries(v.data.filter((d) => d.imageUrl).map((d) => [d.targetId, d.imageUrl!]))
}

export async function Headshots(ids: number[]): Promise<Record<number, string>> {
    if (!ids.length) return {}
    const out: Record<number, string> = {}
    for (let i = 0; i < ids.length; i += 100) {
        const chunk = ids.slice(i, i + 100)
        const v = await Get<{ data: { targetId: number; imageUrl: string | null }[] }>(
            `https://thumbnails.roblox.com/v1/users/avatar-headshot?userIds=${chunk.join(',')}&size=150x150&format=Png&isCircular=false`,
        )
        for (const d of v.data) if (d.imageUrl) out[d.targetId] = d.imageUrl
    }
    return out
}

export const User = (id: number) => Get<UserInfo>(`https://users.roblox.com/v1/users/${id}`)

export async function Me(): Promise<number | null> {
    try {
        return (await Get<{ id: number }>('https://users.roblox.com/v1/users/authenticated')).id
    } catch {
        return null
    }
}

export async function PresenceOf(ids: number[]): Promise<Presence[]> {
    const v = await Post<{ userPresences: Presence[] }>(
        'https://presence.roblox.com/v1/presence/users',
        { userIds: ids },
        'Loading presence',
    )
    return v.userPresences
}

export async function FriendIds(userId: number): Promise<number[]> {
    const v = await Get<{ data: { id: number }[] }>(`https://friends.roblox.com/v1/users/${userId}/friends`)
    return v.data.map((f) => f.id)
}

export async function Names(ids: number[]): Promise<Record<number, { name: string; displayName: string }>> {
    if (!ids.length) return {}
    const v = await Post<{ data: { id: number; name: string; displayName: string }[] }>(
        'https://users.roblox.com/v1/users',
        { userIds: ids.slice(0, 100), excludeBannedUsers: false },
        'Loading names',
    )
    return Object.fromEntries(v.data.map((u) => [u.id, { name: u.name, displayName: u.displayName }]))
}

export interface SearchGame {
    universeId: number
    rootPlaceId: number
    name: string
    playing: number
}

export async function SearchGames(q: string): Promise<SearchGame[]> {
    const sid = crypto.randomUUID()
    const v = await Get<{ searchResults?: { contentGroupType: string; contents?: Record<string, unknown>[] }[] }>(
        `https://apis.roblox.com/search-api/omni-search?searchQuery=${encodeURIComponent(q)}&sessionId=${sid}&pageType=all`,
    )
    return (v.searchResults ?? [])
        .filter((g) => g.contentGroupType === 'Game')
        .flatMap((g) => g.contents ?? [])
        .filter((c) => c.isSponsored !== true)
        .map((c) => ({
            universeId: Number(c.universeId),
            rootPlaceId: Number(c.rootPlaceId),
            name: String(c.name ?? ''),
            playing: Number(c.playerCount ?? 0),
        }))
        .filter((g) => g.rootPlaceId > 0)
        .slice(0, 6)
}

export async function SearchUsers(q: string): Promise<{ id: number; name: string; displayName: string }[]> {
    const v = await Get<{ data?: { id: number; name: string; displayName: string }[] }>(
        `https://users.roblox.com/v1/users/search?keyword=${encodeURIComponent(q)}&limit=10`,
    )
    return (v.data ?? []).slice(0, 5)
}

export function JoinUrl(placeId: number, jobId?: string | null) {
    return `https://www.roblox.com/games/start?placeId=${placeId}${jobId ? `&gameInstanceId=${jobId}` : ''}`
}

export function PageKind(
    url = location.href,
):
    | { kind: 'game'; placeId: number }
    | { kind: 'user'; userId: number }
    | { kind: 'item'; id: number; itemType: 'asset' | 'bundle' }
    | { kind: 'other' } {
    const u = new URL(url)
    const game = u.pathname.match(/^\/(?:[a-z]{2}\/)?games\/(\d+)/i)
    if (game) return { kind: 'game', placeId: Number(game[1]) }
    const user = u.pathname.match(/^\/(?:[a-z]{2}\/)?users\/(\d+)/i)
    if (user) return { kind: 'user', userId: Number(user[1]) }
    const item = u.pathname.match(/^\/(?:[a-z]{2}\/)?(catalog|bundles)\/(\d+)/i)
    if (item) return { kind: 'item', id: Number(item[2]), itemType: item[1]!.toLowerCase() === 'bundles' ? 'bundle' : 'asset' }
    return { kind: 'other' }
}
