import { api, type AvatarRules, type Worn } from './api'

let rulesCache: Promise<AvatarRules> | null = null
export function Rules(): Promise<AvatarRules> {
    rulesCache ??= api.avatarRules().catch((e) => {
        rulesCache = null
        throw e
    })
    return rulesCache
}

export function WithItem(worn: Worn[], id: number, assetType: number, r: AvatarRules): number[] {
    const max = r.wearableAssetTypes.find((t) => t.id === assetType)?.maxNumber ?? 1
    let list = worn.filter((w) => w.id !== id)
    const sameType = list.filter((w) => w.assetType === assetType)
    if (max > 0 && sameType.length >= max) {
        const drop = new Set(sameType.slice(0, sameType.length - max + 1).map((w) => w.id))
        list = list.filter((w) => !drop.has(w.id))
    }
    return [...list.map((w) => w.id), id]
}

export const Without = (worn: Worn[], id: number) => worn.filter((w) => w.id !== id).map((w) => w.id)

export async function WearItem(userId: number, id: number, assetType: number): Promise<string> {
    const [r, av] = await Promise.all([Rules(), api.getAvatar(userId)])
    if (!r.wearableAssetTypes.some((t) => t.id === assetType && t.maxNumber > 0)) return "Roblox doesn't let you wear this kind of item."
    const bad = await api.avatarWear(userId, WithItem(av.assets, id, assetType, r))
    return bad.includes(id) ? "Roblox wouldn't put that item on." : 'Wearing it now. The picture can take a few seconds to update.'
}
