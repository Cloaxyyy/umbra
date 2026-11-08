import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'

export interface Status {
    installed: string | null
    latest: { version: string; clientVersionUpload: string }
    isProtocolHandler: boolean
    dataDir: string
}
export interface Settings {
    channel: string
    multiInstance: boolean
    installedVersion: string | null
    serverLocation: boolean
    discordEnabled: boolean
    discordAppId: string
    discordJoinButton: boolean
    closeToTray: boolean
    autoCheckUpdates: boolean
    accent: string
    overlayEnabled: boolean
    overlayHotkey: string
    friendAlerts: boolean
    overlayFriendsHere: boolean
    overlayRecent: boolean
    overlayServerIp: boolean
    extensionBridge: boolean
}

export interface GameSession {
    userId: number
    placeId: number
    universeId: number
    jobId: string | null
    joinedAt: number
    serverIp: string | null
    gameName: string | null
    gameIcon: string | null
    location: string | null
}
export interface Progress {
    stage: 'manifest' | 'download' | 'extract' | 'done'
    detail: string
    done: number
    total: number
}
export type Flags = Record<string, string | number | boolean>
export interface Account {
    userId: number
    name: string
    displayName: string
    avatar: string | null
    profileId: string | null
}

export interface Presence {
    userId: number
    userPresenceType: 0 | 1 | 2 | 3
    lastLocation: string | null
    placeId: number | null
    gameId: string | null
}
export interface Instance {
    pid: number
    userId: number | null
    label: string
    profileId: string
    secs: number
    priority: 'normal' | 'low' | 'idle'
    game: GameSession | null
}
export interface Exited {
    pid: number
    label: string
    profileId: string
    secs: number
    code: number | null
    crashed: boolean
}
export interface RunningInfo {
    running: Instance[]
    exited: Exited[]
}
export interface Session {
    at: number
    label: string
    secs: number
    crashed: boolean
}
export interface Profile {
    id: string
    name: string
    color: string
    pinnedVersion: string | null
    flags: Flags
    mods: string[]
    customFont: string | null
    theme: ProfileTheme
    launches: number
    playtimeSecs: number
    lastPlayed: number | null
    history: Session[]
}
export interface BridgeGrant {
    id: string
    label: string
    origin: string
    created: number
}
export interface BridgeRequest {
    id: string
    label: string
    origin: string
}
export interface ProfileTheme {
    cursor: boolean
    tint: boolean
    deathSound: boolean
}
export interface ProfileStore {
    defaultId: string
    profiles: Profile[]
    games: Record<string, string>
}
export interface Badge {
    id: number
    name: string
    description: string
    awarded: number
    winRate: number
    icon: string | null
}
export interface Pass {
    id: number
    name: string
    price: number | null
    icon: string | null
}
export interface Preset {
    id: string
    name: string
    accountIds: number[]
    target: Target
    profileId: string | null
}
export interface Mod {
    id: string
    name: string
    files: number
    bytes: number
    added: number
}
export type Target = { kind: 'app' } | { kind: 'place'; place: string; jobId?: string } | { kind: 'follow'; username: string }
export interface GameCard {
    universeId: number
    rootPlaceId: number
    name: string
    playerCount: number
    upVotes: number
    downVotes: number
    icon: string | null
}
export interface Chart {
    id: string
    name: string
    games: GameCard[]
}
export interface GameDetail {
    universeId: number
    rootPlaceId: number
    name: string
    description: string
    creator: string
    playing: number
    visits: number
    maxPlayers: number
    updated: string
    genre: string
    upVotes: number
    downVotes: number
    icon: string | null
    banners: string[]
}
export interface Server {
    id: string
    playing: number
    maxPlayers: number
    ping: number | null
    fps: number | null
}
export interface ServerPage {
    servers: Server[]
    nextCursor: string | null
}
export interface FriendInGame {
    userId: number
    displayName: string
    avatar: string | null
    placeId: number
    gameId: string | null
}
export interface FriendGame {
    game: GameCard
    friends: FriendInGame[]
}
export interface Played {
    game: GameCard
    visits: number
    secs: number
    lastPlayed: number
}
export interface Pick {
    game: GameCard
    reason: string
}
export interface ForYou {
    userId: number
    jumpBackIn: Played[]
    picks: Pick[]
    basedOn: number
}
export interface CatalogItem {
    id: number
    itemType: 'Asset' | 'Bundle'
    assetType: number | null
    name: string
    description: string
    creator: string
    creatorVerified: boolean
    price: number | null
    lowestResale: number | null
    favorites: number
    limited: boolean
    thumb: string | null
}
export interface CatalogPage {
    items: CatalogItem[]
    nextCursor: string | null
}
export interface CatalogCategory {
    category: string
    categoryId: number
    name: string
    assetTypeIds: number[]
    subcategories: { subcategory: string; subcategoryId: number; name: string }[]
}
export interface Worn {
    id: number
    name: string
    assetType: number
    assetTypeName: string
    thumb: string | null
}
export interface Avatar {
    avatarType: 'R6' | 'R15'
    bodyColors: Record<string, number>
    scales: Record<string, number>
    assets: Worn[]
}
export interface Owned {
    id: number
    name: string
    thumb: string | null
}
export interface OwnedPage {
    items: Owned[]
    nextCursor: string | null
}
export interface Outfit {
    id: number
    name: string
    thumb: string | null
}
export interface AvatarRules {
    bodyColorsPalette: { brickColorId: number; hexColor: string; name: string }[]
    scales: Record<string, { min: number; max: number; increment: number }>
    wearableAssetTypes: { id: number; name: string; maxNumber: number }[]
}
export interface BodyColors {
    headColorId: number
    torsoColorId: number
    rightArmColorId: number
    leftArmColorId: number
    rightLegColorId: number
    leftLegColorId: number
}
export interface Scales {
    height: number
    width: number
    head: number
    depth: number
    proportion: number
    bodyType: number
}
export interface ItemValue {
    name: string
    acronym: string
    rap: number
    value: number | null
    shown: number
    demand: number
    trend: number
    projected: boolean
    hyped: boolean
    rare: boolean
}
export interface Limited {
    userAssetId: number
    assetId: number
    name: string
    serial: number | null
    rap: number
    info: ItemValue | null
    shown: number
    thumb: string | null
}
export interface Collection {
    items: Limited[]
    totalRap: number
    totalValue: number
}
export interface Partner {
    id: number
    name: string
    displayName: string
    avatar: string | null
}
export interface TradeSummary {
    id: number
    partner: Partner
    created: string
    expiration: string | null
    status: string
    isActive: boolean
}
export interface TradeSide {
    user: Partner
    items: Limited[]
    robux: number
    rap: number
    value: number
}
export interface TradeDetail {
    id: number
    status: string
    isActive: boolean
    give: TradeSide
    get: TradeSide
    netValue: number
    netRap: number
    warnings: string[]
}
export type TradeKind = 'inbound' | 'outbound' | 'completed' | 'inactive'

export interface Player {
    userId: number
    name: string
    displayName: string
    verified: boolean
    avatar: string | null
}
export interface Friend {
    userId: number
    name: string
    displayName: string
    avatar: string | null
    status: 0 | 1 | 2 | 3
    lastLocation: string | null
    placeId: number | null
    gameId: string | null
    universeId: number | null
}
export interface CreatedGame {
    universeId: number
    rootPlaceId: number
    name: string
    visits: number
    icon: string | null
}
export interface PlayerProfile {
    userId: number
    name: string
    displayName: string
    description: string
    created: string
    verified: boolean
    banned: boolean
    friends: number
    followers: number
    following: number
    avatar: string | null
    status: 0 | 1 | 2 | 3 | null
    lastLocation: string | null
    placeId: number | null
    gameId: string | null
    games: CreatedGame[]
}
export interface Favorite {
    universeId: number
    rootPlaceId: number
    name: string
    icon: string | null
    added: number
}

export const api = {
    status: () => invoke<Status>('get_status'),
    install: () => invoke<string>('install'),
    launch: (arg?: string, profileId?: string) => invoke<number>('launch', { arg, profileId }),
    getSettings: () => invoke<Settings>('get_settings'),
    setSettings: (settings: Settings) => invoke<void>('set_settings', { settings }),
    registerProtocol: () => invoke<void>('register_protocol'),
    openDataFolder: () => invoke<void>('open_data_folder'),
    accounts: () => invoke<Account[]>('list_accounts'),
    addAccount: () => invoke<Account>('add_account'),
    removeAccount: (userId: number) => invoke<void>('remove_account', { userId }),
    presence: () => invoke<Presence[]>('get_presence'),
    launchAccount: (userId: number, target: Target, profileId?: string) => invoke<number>('launch_account', { userId, target, profileId }),
    setAccountProfile: (userId: number, profileId: string | null) => invoke<void>('set_account_profile', { userId, profileId }),
    running: () => invoke<RunningInfo>('list_running'),
    profiles: () => invoke<ProfileStore>('list_profiles'),
    createProfile: (name: string) => invoke<Profile>('create_profile', { name }),
    saveProfile: (profile: Profile) => invoke<Profile>('save_profile', { profile }),
    deleteProfile: (id: string) => invoke<void>('delete_profile', { id }),
    setDefaultProfile: (id: string) => invoke<void>('set_default_profile', { id }),
    setGameProfile: (placeId: number, profileId: string | null) => invoke<void>('set_game_profile', { placeId, profileId }),
    badges: (universeId: number) => invoke<Badge[]>('get_badges', { universeId }),
    passes: (universeId: number) => invoke<Pass[]>('get_passes', { universeId }),
    presets: () => invoke<Preset[]>('list_presets'),
    savePreset: (preset: Preset) => invoke<Preset>('save_preset', { preset }),
    deletePreset: (id: string) => invoke<void>('delete_preset', { id }),
    flagAllowlist: () => invoke<string[]>('get_flag_allowlist'),
    bridgeAnswer: (id: string, allow: boolean) => invoke<void>('bridge_answer', { id, allow }),
    bridgeGrants: () => invoke<BridgeGrant[]>('bridge_grants'),
    bridgeRevoke: (id: string | null) => invoke<void>('bridge_revoke', { id }),
    onBridgePair: (cb: (r: BridgeRequest) => void) => listen<BridgeRequest>('bridge-pair', (e) => cb(e.payload)),
    onFavoritesChanged: (cb: () => void) => listen('favorites-changed', () => cb()),
    mods: () => invoke<Mod[]>('list_mods'),
    importMod: (path: string) => invoke<Mod>('import_mod', { path }),
    deleteMod: (id: string) => invoke<void>('delete_mod', { id }),
    importFont: (path: string) => invoke<string>('import_font', { path }),
    closeRunning: (pid: number) => invoke<void>('close_running', { pid }),
    getAutostart: () => invoke<boolean>('get_autostart'),
    setAutostart: (on: boolean) => invoke<void>('set_autostart', { on }),
    discordStatus: () => invoke<string | null>('discord_status'),
    catalogCategories: () => invoke<CatalogCategory[]>('catalog_categories'),
    searchCatalog: (keyword: string, category: number, subcategory: number | null, sort: number, cursor?: string | null) =>
        invoke<CatalogPage>('search_catalog', { keyword, category, subcategory, sort, cursor }),
    avatarRules: () => invoke<AvatarRules>('avatar_rules'),
    getAvatar: (userId: number) => invoke<Avatar>('get_avatar', { userId }),
    renderAvatar: (userId: number) => invoke<string>('render_avatar', { userId }),
    avatarInventory: (userId: number, assetType: number, cursor?: string | null) =>
        invoke<OwnedPage>('avatar_inventory', { userId, assetType, cursor }),
    avatarOwns: (userId: number, itemType: string, id: number) => invoke<boolean>('avatar_owns', { userId, itemType, id }),
    avatarOutfits: (userId: number) => invoke<Outfit[]>('avatar_outfits', { userId }),
    avatarWear: (userId: number, assetIds: number[]) => invoke<number[]>('avatar_wear', { userId, assetIds }),
    avatarWearOutfit: (userId: number, outfitId: number) => invoke<number[]>('avatar_wear_outfit', { userId, outfitId }),
    avatarSetColors: (userId: number, colors: BodyColors) => invoke<void>('avatar_set_colors', { userId, colors }),
    avatarSetScales: (userId: number, scales: Scales) => invoke<void>('avatar_set_scales', { userId, scales }),
    avatarSetType: (userId: number, avatarType: 'R6' | 'R15') => invoke<void>('avatar_set_type', { userId, avatarType }),
    collection: (userId: number) => invoke<Collection>('get_collection', { userId }),
    trades: (userId: number, kind: TradeKind) => invoke<TradeSummary[]>('get_trades', { userId, kind }),
    trade: (userId: number, tradeId: number) => invoke<TradeDetail>('get_trade', { userId, tradeId }),
    declineTrade: (userId: number, tradeId: number) => invoke<void>('decline_trade', { userId, tradeId }),
    itemValues: (assetIds: number[]) => invoke<Record<string, ItemValue>>('get_item_values', { assetIds }),
    charts: () => invoke<Chart[]>('get_charts'),
    banners: (universeIds: number[]) => invoke<Record<string, string>>('get_banners', { universeIds }),
    search: (query: string) => invoke<GameCard[]>('search_games', { query }),
    game: (universeId: number) => invoke<GameDetail>('get_game', { universeId }),
    servers: (placeId: number, mostPlayers: boolean, excludeFull: boolean, cursor?: string | null) =>
        invoke<ServerPage>('get_servers', { placeId, mostPlayers, excludeFull, cursor }),
    friends: (userId: number) => invoke<Friend[]>('get_friends', { userId }),
    searchPlayers: (keyword: string) => invoke<Player[]>('search_players', { keyword }),
    player: (userId: number, asUser?: number | null) => invoke<PlayerProfile>('get_player', { userId, asUser }),
    favorites: () => invoke<Favorite[]>('list_favorites'),
    overlayTarget: () => invoke<number | null>('overlay_target'),
    overlayPick: (pid: number) => invoke<void>('overlay_pick', { pid }),
    overlayHide: () => invoke<void>('overlay_hide'),
    setClientPriority: (pid: number, level: 'normal' | 'low' | 'idle') => invoke<void>('set_client_priority', { pid, level }),
    setOverlayHotkey: (hotkey: string, enabled: boolean) => invoke<void>('set_overlay_hotkey', { hotkey, enabled }),
    moveClient: (pid: number, mode: 'rejoin' | 'hop' | 'to', placeId?: number | null, jobId?: string | null) =>
        invoke<number>('move_client', { pid, mode, placeId, jobId }),
    toggleFavorite: (favorite: Favorite) => invoke<boolean>('toggle_favorite', { favorite }),
    friendsPlaying: (userId: number) => invoke<FriendGame[]>('get_friends_playing', { userId }),
    forYou: (userId: number | null) => invoke<ForYou | null>('get_for_you', { userId }),
    onProgress: (cb: (p: Progress) => void) => listen<Progress>('install-progress', (e) => cb(e.payload)),
    onLaunchError: (cb: (msg: string) => void) => listen<string>('launch-error', (e) => cb(e.payload)),
}
