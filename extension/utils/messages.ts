import type { Hello, ItemValue, PlayerValue, Reply, Request, Target, UmbraState } from './types'

async function Send<T>(req: Request): Promise<T> {
    const res = (await browser.runtime.sendMessage(req)) as Reply<T> | undefined
    if (!res) throw new Error('The Umbra extension is restarting. Try again.')
    if (!res.ok) throw new Error(res.error)
    return res.data
}

export const Umbra = {
    hello: () => Send<Hello>({ type: 'umbra.hello' }),
    pair: () => Send<boolean>({ type: 'umbra.pair' }),
    unpair: () => Send<boolean>({ type: 'umbra.unpair' }),
    state: () => Send<UmbraState>({ type: 'umbra.state' }),
    launch: (userIds: number[], target: Target, profileId?: string | null) =>
        Send<{ started: number; errors: { userId: number; error: string }[] }>({ type: 'umbra.launch', userIds, target, profileId }),
    favorite: (universeId: number, rootPlaceId: number, name: string, icon: string | null) =>
        Send<{ added: boolean }>({ type: 'umbra.favorite', universeId, rootPlaceId, name, icon }),
    client: (pid: number, action: 'rejoin' | 'hop' | 'close') => Send<{ pid: number }>({ type: 'umbra.client', pid, action }),
}

export const Rolimons = {
    item: (id: number) => Send<ItemValue | null>({ type: 'rolimons.item', id }),
    player: (id: number) => Send<PlayerValue | null>({ type: 'rolimons.player', id }),
}

export const OpenOptions = () => Send<boolean>({ type: 'options.open' })
