import type { IconName } from './icons'

export interface Shot {
    id: string
    label: string
    icon: IconName
    title: string
    alt: string
}

export const SHOWCASE: Shot[] = [
    {
        id: 'home',
        label: 'Launchpad',
        icon: 'rocket',
        title: 'Favorites, recent games and online friends the second Umbra opens.',
        alt: 'The Umbra launchpad with favorite games, recently played games and friends who are in game',
    },
    {
        id: 'discover',
        label: 'Discover',
        icon: 'flame',
        title: "Roblox's trending, top playing and up-and-coming charts, plus search.",
        alt: 'The Discover page showing a grid of trending Roblox games',
    },
    {
        id: 'game',
        label: 'Games',
        icon: 'gamepad',
        title: 'Stats, screenshots, a private server box and an instance for each game.',
        alt: 'The game page for 99 Nights in the Forest with stats, play buttons and the instance picker',
    },
    {
        id: 'serverlist',
        label: 'Servers',
        icon: 'signal',
        title: 'Players, ping and frame rate for every public server.',
        alt: 'A server list with player counts, ping and fps for each server and a join button',
    },
    {
        id: 'friends',
        label: 'Friends',
        icon: 'users',
        title: 'Who is online, what they are playing, and a button to join them.',
        alt: 'The Friends page with a selected friend who is in 99 Nights in the Forest',
    },
    {
        id: 'catalog',
        label: 'Catalog',
        icon: 'bag',
        title: 'The avatar shop and an avatar editor, without opening a browser.',
        alt: 'The catalog page with a grid of avatar items and prices',
    },
    {
        id: 'palette',
        label: 'Ctrl K',
        icon: 'search',
        title: 'One search box for games, players, items and every page.',
        alt: 'The command palette open over the launchpad with a list of pages to jump to',
    },
]

export const CARDS: { id: string; title: string; text: string; alt: string }[] = [
    {
        id: 'accounts',
        title: 'Your main and your alts, at once',
        text: 'Run every account side by side, send them all into one server, and save the groups you launch every day.',
        alt: 'The Accounts page with three accounts, a join together panel and saved launch presets',
    },
    {
        id: 'tuning',
        title: 'A separate setup for every game',
        text: 'Instances with their own FastFlags, mods, font and Roblox version. Pick one per game and forget about it.',
        alt: 'The Instances page on the Tuning tab with graphics presets and settings for each flag',
    },
]

export const FEATURES: { icon: IconName; title: string; text: string }[] = [
    { icon: 'shield', title: 'Encrypted sessions', text: 'You sign in on roblox.com. Umbra keeps only the session, encrypted by Windows.' },
    { icon: 'users', title: 'Friend alerts', text: 'A popup when a friend starts playing, with a button to join them.' },
    { icon: 'enter', title: 'Private servers', text: 'Paste a private server link and join it with any of your accounts.' },
    { icon: 'star', title: 'Passes and badges', text: 'Every game pass and badge on the game page, with how rare each badge is.' },
    { icon: 'sparkle', title: 'Umbra theme', text: 'A clean cursor, accent colored menus and a softer death sound. Off by default.' },
    {
        icon: 'gauge',
        title: 'Allowed flags only',
        text: 'Tuning uses only the FastFlags Roblox still reads. The list updates from GitHub.',
    },
    { icon: 'activity', title: 'Discord status', text: 'Show the game you are in on your Discord profile, with a join button.' },
    { icon: 'swap', title: 'Trades', text: 'Review your trades with item values from Rolimons.' },
]

export const OVERLAY_POINTS: { title: string; text: string }[] = [
    { title: 'Rejoin or hop', text: 'Back into the same server or a fresh one in a couple of seconds.' },
    { title: 'Bring your alts', text: 'Send any other account straight into the server you are in.' },
    { title: 'Tune live', text: 'CPU priority changes instantly. Graphics apply on rejoin.' },
]

export const FAQ: { q: string; a: string }[] = [
    { q: 'Is Umbra free?', a: 'Yes. Umbra is free and open source under the MIT license.' },
    {
        q: 'Can it get my account banned?',
        a: "Umbra launches the official Roblox client. It doesn't inject code or change how Roblox runs, and it only sets the FastFlags Roblox allows. Like any third-party tool, you use it at your own risk.",
    },
    {
        q: 'Does it replace Bloxstrap?',
        a: 'It can. Umbra keeps its own copy of Roblox, so it never touches other launchers, and you can keep both.',
    },
    { q: 'Does it work on Mac or Linux?', a: 'No. Umbra is for Windows 10 and 11.' },
    {
        q: 'Where does Umbra keep my data?',
        a: 'Everything stays on your PC, in the Umbra folder under your local app data. Account sessions are encrypted by Windows.',
    },
    {
        q: 'How do updates work?',
        a: 'Umbra checks GitHub for new releases. Each update is signed, and Umbra refuses one that does not match.',
    },
]
