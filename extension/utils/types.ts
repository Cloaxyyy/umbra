export type Target = { kind: 'app' } | { kind: 'place'; place: string; jobId?: string } | { kind: 'follow'; username: string }

export interface UmbraAccount {
    userId: number
    name: string
    displayName: string
    avatar: string | null
    profileId: string | null
    autoRejoin?: boolean
}

export interface UmbraProfile {
    id: string
    name: string
    color: string
}

export interface UmbraClient {
    pid: number
    userId: number | null
    label: string
    profileId: string
    secs: number
    priority: string
    game: {
        placeId: number
        universeId: number
        jobId: string | null
        gameName: string | null
        gameIcon: string | null
        location: string | null
    } | null
}

export interface UmbraState {
    accounts: UmbraAccount[]
    profiles: { defaultId: string; profiles: UmbraProfile[]; games: Record<string, string> }
    running: UmbraClient[]
    favorites: number[]
}

export interface Hello {
    reachable: boolean
    enabled: boolean
    paired: boolean
    version: string | null
    error?: string
}

export interface ItemValue {
    name: string
    acronym: string
    rap: number
    value: number | null
    demand: number
    trend: number
    projected: boolean
    hyped: boolean
    rare: boolean
}

export interface PlayerValue {
    value: number
    rap: number
    rank: number | null
    premium: boolean
    lastOnline: number | null
}

export type Request =
    | { type: 'umbra.hello' }
    | { type: 'umbra.pair' }
    | { type: 'umbra.unpair' }
    | { type: 'umbra.state' }
    | { type: 'umbra.launch'; userIds: number[]; target: Target; profileId?: string | null }
    | { type: 'umbra.favorite'; universeId: number; rootPlaceId: number; name: string; icon: string | null }
    | { type: 'umbra.client'; pid: number; action: 'rejoin' | 'hop' | 'close' }
    | { type: 'umbra.playtime'; universeId: number }
    | { type: 'rolimons.item'; id: number }
    | { type: 'rolimons.player'; id: number }
    | { type: 'options.open' }

export type Reply<T = unknown> = { ok: true; data: T } | { ok: false; error: string }
